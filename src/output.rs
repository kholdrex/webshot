use crate::error::{Result, WebshotError};
use crate::screenshot::ImageFormat;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Output handler for managing file operations and format conversions
pub struct OutputHandler;

impl OutputHandler {
    /// Ensure the output directory exists
    pub fn ensure_output_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            if !parent.exists() {
                info!("Creating output directory: {}", parent.display());
                std::fs::create_dir_all(parent)?;
            }
        }
        Ok(())
    }

    /// Generate a default filename based on URL and timestamp
    pub fn generate_filename(url: &str, format: ImageFormat) -> String {
        use chrono::Utc;
        use url::Url;

        let parsed_url = Url::parse(url).ok();
        let domain = parsed_url
            .as_ref()
            .and_then(|u| u.host_str())
            .unwrap_or("unknown");

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let sanitized_domain = sanitize_filename(domain);

        format!(
            "{}_{}.{}",
            sanitized_domain,
            timestamp,
            format.extension()
        )
    }

    /// Validate that the output path has a supported extension
    pub fn validate_output_path<P: AsRef<Path>>(path: P) -> Result<ImageFormat> {
        let path = path.as_ref();
        
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| WebshotError::config("No file extension found".to_string()))?;

        match extension.as_str() {
            "png" => Ok(ImageFormat::Png),
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "webp" => Ok(ImageFormat::WebP),
            "pdf" => Ok(ImageFormat::Pdf),
            _ => Err(WebshotError::UnsupportedFormat { format: extension }),
        }
    }

    /// Convert image data between formats
    pub fn convert_image(
        data: &[u8],
        source_format: ImageFormat,
        target_format: ImageFormat,
        quality: Option<u8>,
    ) -> Result<Vec<u8>> {
        if source_format == target_format {
            return Ok(data.to_vec());
        }

        let img = match source_format {
            ImageFormat::Png => image::load_from_memory_with_format(data, image::ImageFormat::Png)?,
            ImageFormat::Jpeg => image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)?,
            ImageFormat::WebP => image::load_from_memory_with_format(data, image::ImageFormat::WebP)?,
            ImageFormat::Pdf => {
                return Err(WebshotError::config(
                    "Cannot convert from PDF format".to_string(),
                ));
            }
        };

        let mut output = Vec::new();

        match target_format {
            ImageFormat::Png => {
                let encoder = image::codecs::png::PngEncoder::new(&mut output);
                img.write_with_encoder(encoder)?;
            }
            ImageFormat::Jpeg => {
                let quality = quality.unwrap_or(90);
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);
                img.write_with_encoder(encoder)?;
            }
            ImageFormat::WebP => {
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
                img.write_with_encoder(encoder)?;
            }
            ImageFormat::Pdf => {
                return Err(WebshotError::config(
                    "Cannot convert to PDF format using image conversion".to_string(),
                ));
            }
        }

        Ok(output)
    }

    /// Optimize image file size
    pub fn optimize_image<P: AsRef<Path>>(path: P, format: ImageFormat) -> Result<()> {
        let path = path.as_ref();
        debug!("Optimizing image: {}", path.display());

        match format {
            ImageFormat::Png => {
                // For PNG, we could implement oxipng optimization here
                // For now, just validate the file is readable
                let _img = image::open(path)?;
            }
            ImageFormat::Jpeg => {
                // For JPEG, we could implement mozjpeg optimization here
                // For now, just validate the file is readable
                let _img = image::open(path)?;
            }
            ImageFormat::WebP => {
                // For WebP, we could implement WebP optimization here
                // For now, just validate the file is readable
                let _img = image::open(path)?;
            }
            ImageFormat::Pdf => {
                // PDF optimization would require additional libraries
                debug!("PDF optimization not implemented");
            }
        }

        Ok(())
    }

    /// Get file size in a human-readable format
    pub fn get_file_size<P: AsRef<Path>>(path: P) -> Result<String> {
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        
        Ok(format_file_size(size))
    }

    /// Create a temporary file with the given extension
    pub fn create_temp_file(extension: &str) -> Result<PathBuf> {
        use tempfile::NamedTempFile;
        
        let temp_file = NamedTempFile::with_suffix(&format!(".{}", extension))?;
        let path = temp_file.path().to_path_buf();
        
        // Keep the file but close the handle
        temp_file.keep()
            .map_err(|e| WebshotError::config(format!("Failed to keep temp file: {}", e)))?;
        
        Ok(path)
    }

    /// Clean up temporary files
    pub fn cleanup_temp_files(paths: &[PathBuf]) {
        for path in paths {
            if path.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    debug!("Failed to remove temp file {}: {}", path.display(), e);
                }
            }
        }
    }

    /// Resolve output path with proper extension
    pub fn resolve_output_path<P: AsRef<Path>>(
        output: Option<P>,
        url: &str,
        format: ImageFormat,
    ) -> PathBuf {
        match output {
            Some(path) => {
                let path = path.as_ref();
                if path.extension().is_some() {
                    path.to_path_buf()
                } else {
                    // Add extension if missing
                    path.with_extension(format.extension())
                }
            }
            None => {
                // Generate filename from URL
                PathBuf::from(Self::generate_filename(url, format))
            }
        }
    }

    /// Check if file already exists and handle overwrites
    pub fn handle_existing_file<P: AsRef<Path>>(path: P, overwrite: bool) -> Result<()> {
        let path = path.as_ref();
        
        if path.exists() && !overwrite {
            return Err(WebshotError::config(format!(
                "File already exists: {}. Use --force to overwrite.",
                path.display()
            )));
        }
        
        Ok(())
    }
}

/// Sanitize a filename by removing or replacing invalid characters
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim_matches('.')
        .to_string()
}

/// Format file size in human-readable format
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.com"), "test.com");
        assert_eq!(sanitize_filename("test/file.png"), "test_file.png");
        assert_eq!(sanitize_filename("test:file?.png"), "test_file_.png");
        assert_eq!(sanitize_filename("normal_file.png"), "normal_file.png");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_generate_filename() {
        let filename = OutputHandler::generate_filename("https://example.com/path", ImageFormat::Png);
        assert!(filename.starts_with("example.com_"));
        assert!(filename.ends_with(".png"));

        let filename = OutputHandler::generate_filename("invalid-url", ImageFormat::Jpeg);
        assert!(filename.starts_with("unknown_"));
        assert!(filename.ends_with(".jpg"));
    }

    #[test]
    fn test_validate_output_path() {
        assert_eq!(
            OutputHandler::validate_output_path("test.png").unwrap(),
            ImageFormat::Png
        );
        assert_eq!(
            OutputHandler::validate_output_path("test.jpg").unwrap(),
            ImageFormat::Jpeg
        );
        assert_eq!(
            OutputHandler::validate_output_path("test.jpeg").unwrap(),
            ImageFormat::Jpeg
        );
        assert_eq!(
            OutputHandler::validate_output_path("test.pdf").unwrap(),
            ImageFormat::Pdf
        );
        assert_eq!(
            OutputHandler::validate_output_path("test.webp").unwrap(),
            ImageFormat::WebP
        );

        assert!(OutputHandler::validate_output_path("test.gif").is_err());
        assert!(OutputHandler::validate_output_path("test").is_err());
    }

    #[test]
    fn test_ensure_output_dir() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("subdir").join("test.png");

        assert!(!test_path.parent().unwrap().exists());
        OutputHandler::ensure_output_dir(&test_path).unwrap();
        assert!(test_path.parent().unwrap().exists());
    }

    #[test]
    fn test_resolve_output_path() {
        // With extension
        let path = OutputHandler::resolve_output_path(
            Some("test.png"),
            "https://example.com",
            ImageFormat::Png,
        );
        assert_eq!(path, PathBuf::from("test.png"));

        // Without extension
        let path = OutputHandler::resolve_output_path(
            Some("test"),
            "https://example.com",
            ImageFormat::Png,
        );
        assert_eq!(path, PathBuf::from("test.png"));

        // No output path provided
        let path = OutputHandler::resolve_output_path(
            None::<&str>,
            "https://example.com",
            ImageFormat::Png,
        );
        assert!(path.to_string_lossy().contains("example.com"));
        assert!(path.to_string_lossy().ends_with(".png"));
    }
}
