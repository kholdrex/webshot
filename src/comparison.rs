use crate::error::{Result, WebshotError};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info};

/// Image comparison algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonAlgorithm {
    /// Pixel-by-pixel difference
    PixelDiff,
    /// Structural Similarity Index (SSIM)
    SSIM,
    /// Mean Squared Error
    MSE,
    /// Peak Signal-to-Noise Ratio
    PSNR,
}

impl Default for ComparisonAlgorithm {
    fn default() -> Self {
        Self::PixelDiff
    }
}

/// Comparison options
#[derive(Debug, Clone)]
pub struct ComparisonOptions {
    /// Comparison algorithm to use
    pub algorithm: ComparisonAlgorithm,
    /// Threshold for considering images different (0.0 to 1.0)
    pub threshold: f64,
    /// Generate difference image highlighting changes
    pub generate_diff_image: bool,
    /// Output path for difference image
    pub diff_output_path: Option<std::path::PathBuf>,
    /// Ignore minor color differences (useful for anti-aliasing)
    pub ignore_antialiasing: bool,
    /// Color to highlight differences in diff image
    pub diff_color: (u8, u8, u8),
}

impl Default for ComparisonOptions {
    fn default() -> Self {
        Self {
            algorithm: ComparisonAlgorithm::default(),
            threshold: 0.1,
            generate_diff_image: false,
            diff_output_path: None,
            ignore_antialiasing: false,
            diff_color: (255, 0, 0), // Red
        }
    }
}

/// Result of image comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Are the images considered similar based on threshold
    pub similar: bool,
    /// Similarity score (0.0 = completely different, 1.0 = identical)
    pub similarity: f64,
    /// Number of different pixels (for pixel diff algorithm)
    pub different_pixels: Option<u32>,
    /// Total number of pixels compared
    pub total_pixels: u32,
    /// Algorithm used for comparison
    pub algorithm: ComparisonAlgorithm,
    /// Threshold used
    pub threshold: f64,
    /// Path to generated difference image (if created)
    pub diff_image_path: Option<std::path::PathBuf>,
}

/// Image comparison engine
pub struct ImageComparator;

impl ImageComparator {
    /// Compare two images from file paths
    pub fn compare_files<P1: AsRef<Path>, P2: AsRef<Path>>(
        image1_path: P1,
        image2_path: P2,
        options: &ComparisonOptions,
    ) -> Result<ComparisonResult> {
        info!("Loading images for comparison");
        let image1 = image::open(&image1_path)
            .map_err(|e| WebshotError::config(format!("Failed to load first image: {}", e)))?;
        let image2 = image::open(&image2_path)
            .map_err(|e| WebshotError::config(format!("Failed to load second image: {}", e)))?;
        
        Self::compare_images(&image1, &image2, options)
    }

    /// Compare two images directly
    pub fn compare_images(
        image1: &DynamicImage,
        image2: &DynamicImage,
        options: &ComparisonOptions,
    ) -> Result<ComparisonResult> {
        // Convert to RGB and ensure same dimensions
        let img1 = image1.to_rgb8();
        let img2 = image2.to_rgb8();

        if img1.dimensions() != img2.dimensions() {
            return Err(WebshotError::config(format!(
                "Image dimensions don't match: {:?} vs {:?}",
                img1.dimensions(),
                img2.dimensions()
            )));
        }

        let (width, height) = img1.dimensions();
        let total_pixels = width * height;

        info!("Comparing images using {:?} algorithm", options.algorithm);
        
        let (similarity, different_pixels) = match options.algorithm {
            ComparisonAlgorithm::PixelDiff => Self::pixel_diff_comparison(&img1, &img2, options),
            ComparisonAlgorithm::SSIM => (Self::ssim_comparison(&img1, &img2)?, None),
            ComparisonAlgorithm::MSE => (Self::mse_comparison(&img1, &img2), None),
            ComparisonAlgorithm::PSNR => (Self::psnr_comparison(&img1, &img2), None),
        };

        let similar = similarity >= (1.0 - options.threshold);

        let mut result = ComparisonResult {
            similar,
            similarity,
            different_pixels,
            total_pixels,
            algorithm: options.algorithm,
            threshold: options.threshold,
            diff_image_path: None,
        };

        // Generate difference image if requested
        if options.generate_diff_image {
            if let Some(diff_path) = &options.diff_output_path {
                info!("Generating difference image");
                Self::generate_diff_image(&img1, &img2, diff_path, options)?;
                result.diff_image_path = Some(diff_path.clone());
            }
        }

        Ok(result)
    }

    /// Pixel-by-pixel difference comparison
    fn pixel_diff_comparison(
        img1: &RgbImage,
        img2: &RgbImage,
        options: &ComparisonOptions,
    ) -> (f64, Option<u32>) {
        let mut different_pixels = 0u32;
        let (width, height) = img1.dimensions();

        for y in 0..height {
            for x in 0..width {
                let pixel1 = img1.get_pixel(x, y);
                let pixel2 = img2.get_pixel(x, y);

                if !Self::pixels_similar(pixel1, pixel2, options.ignore_antialiasing) {
                    different_pixels += 1;
                }
            }
        }

        let total_pixels = width * height;
        let similarity = 1.0 - (different_pixels as f64 / total_pixels as f64);
        
        debug!("Pixel diff: {}/{} different pixels", different_pixels, total_pixels);
        (similarity, Some(different_pixels))
    }

    /// Check if two pixels are similar (with optional anti-aliasing tolerance)
    fn pixels_similar(pixel1: &Rgb<u8>, pixel2: &Rgb<u8>, ignore_antialiasing: bool) -> bool {
        if pixel1 == pixel2 {
            return true;
        }

        // For basic comparison, use a small threshold for minor differences
        let threshold = if ignore_antialiasing { 10 } else { 2 };
        let r_diff = (pixel1[0] as i16 - pixel2[0] as i16).abs();
        let g_diff = (pixel1[1] as i16 - pixel2[1] as i16).abs();
        let b_diff = (pixel1[2] as i16 - pixel2[2] as i16).abs();
        
        r_diff <= threshold && g_diff <= threshold && b_diff <= threshold
    }

    /// Structural Similarity Index (SSIM) comparison
    fn ssim_comparison(img1: &RgbImage, img2: &RgbImage) -> Result<f64> {
        // Convert to grayscale for SSIM calculation
        let gray1 = Self::rgb_to_grayscale(img1);
        let gray2 = Self::rgb_to_grayscale(img2);
        
        let ssim = Self::calculate_ssim(&gray1, &gray2)?;
        Ok(ssim)
    }

    /// Mean Squared Error comparison
    fn mse_comparison(img1: &RgbImage, img2: &RgbImage) -> f64 {
        let (width, height) = img1.dimensions();
        let mut mse = 0.0;

        for y in 0..height {
            for x in 0..width {
                let pixel1 = img1.get_pixel(x, y);
                let pixel2 = img2.get_pixel(x, y);

                for i in 0..3 {
                    let diff = pixel1[i] as f64 - pixel2[i] as f64;
                    mse += diff * diff;
                }
            }
        }

        mse /= (width * height * 3) as f64;
        
        // Convert MSE to similarity (lower MSE = higher similarity)
        1.0 / (1.0 + mse / 255.0)
    }

    /// Peak Signal-to-Noise Ratio comparison
    fn psnr_comparison(img1: &RgbImage, img2: &RgbImage) -> f64 {
        let mse = {
            let (width, height) = img1.dimensions();
            let mut mse = 0.0;

            for y in 0..height {
                for x in 0..width {
                    let pixel1 = img1.get_pixel(x, y);
                    let pixel2 = img2.get_pixel(x, y);

                    for i in 0..3 {
                        let diff = pixel1[i] as f64 - pixel2[i] as f64;
                        mse += diff * diff;
                    }
                }
            }

            mse / (width * height * 3) as f64
        };

        if mse == 0.0 {
            return 1.0; // Identical images
        }

        let psnr = 20.0 * (255.0_f64).log10() - 10.0 * mse.log10();
        
        // Convert PSNR to similarity (higher PSNR = higher similarity)
        // Typical PSNR values: 30-50 dB is good, >50 dB is very good
        // Normalize PSNR to 0-1 range, where 30dB = 0.3, 50dB = 0.5, etc.
        if psnr < 0.0 {
            0.0
        } else {
            (psnr / 100.0).min(1.0)
        }
    }

    /// Convert RGB image to grayscale
    fn rgb_to_grayscale(img: &RgbImage) -> ImageBuffer<image::Luma<u8>, Vec<u8>> {
        let (width, height) = img.dimensions();
        let mut gray = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                let gray_value = (0.299 * pixel[0] as f64 
                    + 0.587 * pixel[1] as f64 
                    + 0.114 * pixel[2] as f64) as u8;
                gray.put_pixel(x, y, image::Luma([gray_value]));
            }
        }

        gray
    }

    /// Calculate SSIM for grayscale images
    fn calculate_ssim(
        img1: &ImageBuffer<image::Luma<u8>, Vec<u8>>,
        img2: &ImageBuffer<image::Luma<u8>, Vec<u8>>,
    ) -> Result<f64> {
        let (width, height) = img1.dimensions();
        
        // Constants for SSIM calculation
        let k1 = 0.01_f64;
        let k2 = 0.03_f64;
        let l = 255.0_f64; // Dynamic range
        let c1 = (k1 * l).powi(2);
        let c2 = (k2 * l).powi(2);

        // Calculate means
        let mut sum1 = 0.0;
        let mut sum2 = 0.0;
        let total_pixels = (width * height) as f64;

        for y in 0..height {
            for x in 0..width {
                sum1 += img1.get_pixel(x, y)[0] as f64;
                sum2 += img2.get_pixel(x, y)[0] as f64;
            }
        }

        let mean1 = sum1 / total_pixels;
        let mean2 = sum2 / total_pixels;

        // Calculate variances and covariance
        let mut var1 = 0.0;
        let mut var2 = 0.0;
        let mut covar = 0.0;

        for y in 0..height {
            for x in 0..width {
                let val1 = img1.get_pixel(x, y)[0] as f64;
                let val2 = img2.get_pixel(x, y)[0] as f64;
                
                let diff1 = val1 - mean1;
                let diff2 = val2 - mean2;
                
                var1 += diff1 * diff1;
                var2 += diff2 * diff2;
                covar += diff1 * diff2;
            }
        }

        var1 /= total_pixels - 1.0;
        var2 /= total_pixels - 1.0;
        covar /= total_pixels - 1.0;

        // Calculate SSIM
        let numerator = (2.0 * mean1 * mean2 + c1) * (2.0 * covar + c2);
        let denominator = (mean1 * mean1 + mean2 * mean2 + c1) * (var1 + var2 + c2);
        
        let ssim = numerator / denominator;
        Ok(ssim.max(0.0).min(1.0))
    }

    /// Generate a difference image highlighting changes
    fn generate_diff_image<P: AsRef<Path>>(
        img1: &RgbImage,
        img2: &RgbImage,
        output_path: P,
        options: &ComparisonOptions,
    ) -> Result<()> {
        let (width, height) = img1.dimensions();
        let mut diff_img = RgbImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let pixel1 = img1.get_pixel(x, y);
                let pixel2 = img2.get_pixel(x, y);

                if Self::pixels_similar(pixel1, pixel2, options.ignore_antialiasing) {
                    // Keep original pixel (could be grayscale for subtle effect)
                    diff_img.put_pixel(x, y, *pixel1);
                } else {
                    // Highlight difference
                    diff_img.put_pixel(x, y, Rgb([
                        options.diff_color.0,
                        options.diff_color.1,
                        options.diff_color.2,
                    ]));
                }
            }
        }

        diff_img.save(&output_path)
            .map_err(|e| WebshotError::config(format!("Failed to save diff image: {}", e)))?;

        info!("Difference image saved to: {}", output_path.as_ref().display());
        Ok(())
    }
}

impl ComparisonOptions {
    /// Create new comparison options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set comparison algorithm
    pub fn algorithm(mut self, algorithm: ComparisonAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Set similarity threshold
    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Enable difference image generation
    pub fn generate_diff_image<P: AsRef<Path>>(mut self, output_path: P) -> Self {
        self.generate_diff_image = true;
        self.diff_output_path = Some(output_path.as_ref().to_path_buf());
        self
    }

    /// Enable anti-aliasing tolerance
    pub fn ignore_antialiasing(mut self) -> Self {
        self.ignore_antialiasing = true;
        self
    }

    /// Set difference highlight color
    pub fn diff_color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.diff_color = (r, g, b);
        self
    }

    /// Validate the options
    pub fn validate(&self) -> Result<()> {
        if !(0.0..=1.0).contains(&self.threshold) {
            return Err(WebshotError::config(format!(
                "Threshold must be between 0.0 and 1.0, got: {}",
                self.threshold
            )));
        }

        if self.generate_diff_image && self.diff_output_path.is_none() {
            return Err(WebshotError::config(
                "Diff output path must be specified when generating diff image".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use tempfile::TempDir;

    fn create_test_image(width: u32, height: u32, color: [u8; 3]) -> RgbImage {
        ImageBuffer::from_fn(width, height, |_, _| Rgb(color))
    }

    #[test]
    fn test_identical_images() {
        let img1 = create_test_image(100, 100, [255, 0, 0]);
        let img2 = create_test_image(100, 100, [255, 0, 0]);
        
        let options = ComparisonOptions::new();
        let result = ImageComparator::compare_images(&img1.into(), &img2.into(), &options).unwrap();
        
        assert!(result.similar);
        assert_eq!(result.similarity, 1.0);
        assert_eq!(result.different_pixels, Some(0));
    }

    #[test]
    fn test_different_images() {
        let img1 = create_test_image(100, 100, [255, 0, 0]);
        let img2 = create_test_image(100, 100, [0, 255, 0]);
        
        let options = ComparisonOptions::new();
        let result = ImageComparator::compare_images(&img1.into(), &img2.into(), &options).unwrap();
        
        assert!(!result.similar);
        assert_eq!(result.similarity, 0.0);
        assert_eq!(result.different_pixels, Some(10000));
    }

    #[test]
    fn test_dimension_mismatch() {
        let img1 = create_test_image(100, 100, [255, 0, 0]);
        let img2 = create_test_image(200, 100, [255, 0, 0]);
        
        let options = ComparisonOptions::new();
        let result = ImageComparator::compare_images(&img1.into(), &img2.into(), &options);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_diff_image_generation() {
        let temp_dir = TempDir::new().unwrap();
        let diff_path = temp_dir.path().join("diff.png");
        
        let img1 = create_test_image(10, 10, [255, 0, 0]);
        let img2 = create_test_image(10, 10, [0, 255, 0]);
        
        let options = ComparisonOptions::new()
            .generate_diff_image(&diff_path);
        
        let result = ImageComparator::compare_images(&img1.into(), &img2.into(), &options).unwrap();
        
        assert!(diff_path.exists());
        assert_eq!(result.diff_image_path, Some(diff_path));
    }

    #[test]
    fn test_comparison_algorithms() {
        // Test different algorithms with identical images first
        for algorithm in [
            ComparisonAlgorithm::PixelDiff,
            ComparisonAlgorithm::SSIM,
            ComparisonAlgorithm::MSE,
            ComparisonAlgorithm::PSNR,
        ] {
            let img1 = create_test_image(50, 50, [255, 0, 0]);
            let img2 = create_test_image(50, 50, [255, 0, 0]); // Identical
            
            let options = ComparisonOptions::new().algorithm(algorithm);
            let result = ImageComparator::compare_images(&img1.into(), &img2.into(), &options).unwrap();
            
            // Identical images should have high similarity
            assert!(result.similarity >= 0.9, "Algorithm {:?} didn't recognize identical images: {}", algorithm, result.similarity);
            assert_eq!(result.algorithm, algorithm);
        }
        
        // Test with completely different images
        for algorithm in [
            ComparisonAlgorithm::PixelDiff,
            ComparisonAlgorithm::SSIM,
            ComparisonAlgorithm::MSE,
            ComparisonAlgorithm::PSNR,
        ] {
            let img1 = create_test_image(50, 50, [255, 0, 0]); // Red
            let img2 = create_test_image(50, 50, [0, 255, 0]); // Green - completely different
            
            let options = ComparisonOptions::new().algorithm(algorithm);
            let result = ImageComparator::compare_images(&img1.into(), &img2.into(), &options).unwrap();
            
            // Completely different images should have low similarity
            // SSIM measures structural similarity, so solid colors might still be similar
            let max_similarity = if algorithm == ComparisonAlgorithm::SSIM { 0.9 } else { 0.5 };
            assert!(result.similarity <= max_similarity, "Algorithm {:?} gave too high similarity for different images: {}", algorithm, result.similarity);
            assert!(result.similarity >= 0.0, "Algorithm {:?} returned negative similarity: {}", algorithm, result.similarity);
            assert_eq!(result.algorithm, algorithm);
        }
    }
}
