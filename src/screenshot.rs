use crate::error::{Result, WebshotError};
use std::path::Path;

/// Scrolling screenshot mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScrollMode {
    /// Standard viewport-only screenshot
    Viewport,
    /// Capture full page by scrolling and stitching
    FullPage,
    /// Capture element and its full scrollable content
    FullElement,
}

impl Default for ScrollMode {
    fn default() -> Self {
        Self::Viewport
    }
}

/// Screenshot configuration options
#[derive(Debug, Clone)]
pub struct ScreenshotOptions {
    /// Viewport width
    pub width: u32,
    /// Viewport height
    pub height: u32,
    /// CSS selector for element screenshot
    pub selector: Option<String>,
    /// JavaScript to execute before screenshot
    pub javascript: Option<String>,
    /// Element to wait for before taking screenshot
    pub wait_for: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Enable retina/high-DPI mode
    pub retina: bool,
    /// JPEG quality (1-100)
    pub quality: Option<u8>,
    /// Wait time before taking screenshot
    pub wait: u64,
    /// Custom user agent
    pub user_agent: Option<String>,
    /// Scrolling screenshot mode
    pub scroll_mode: ScrollMode,
    /// Maximum height for full page screenshots (safety limit)
    pub max_height: Option<u32>,
    /// Scroll delay between captures (milliseconds)
    pub scroll_delay: u64,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 800,
            selector: None,
            javascript: None,
            wait_for: None,
            timeout: 30,
            retina: false,
            quality: None,
            wait: 0,
            user_agent: None,
            scroll_mode: ScrollMode::default(),
            max_height: Some(30000), // 30k pixels default max height
            scroll_delay: 100,       // 100ms delay between scrolls
        }
    }
}

impl ScreenshotOptions {
    /// Create new screenshot options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set viewport dimensions
    pub fn viewport(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set CSS selector for element screenshot
    pub fn selector<S: Into<String>>(mut self, selector: S) -> Self {
        self.selector = Some(selector.into());
        self
    }

    /// Set JavaScript to execute
    pub fn javascript<S: Into<String>>(mut self, script: S) -> Self {
        self.javascript = Some(script.into());
        self
    }

    /// Set element to wait for
    pub fn wait_for<S: Into<String>>(mut self, selector: S) -> Self {
        self.wait_for = Some(selector.into());
        self
    }

    /// Set timeout in seconds
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable retina mode
    pub fn retina(mut self) -> Self {
        self.retina = true;
        self
    }

    /// Set JPEG quality
    pub fn quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality);
        self
    }

    /// Set wait time before screenshot
    pub fn wait(mut self, wait: u64) -> Self {
        self.wait = wait;
        self
    }

    /// Set custom user agent
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Enable full page scrolling screenshot
    pub fn full_page(mut self) -> Self {
        self.scroll_mode = ScrollMode::FullPage;
        self
    }

    /// Enable full element scrolling screenshot
    pub fn full_element(mut self) -> Self {
        self.scroll_mode = ScrollMode::FullElement;
        self
    }

    /// Set scroll mode
    pub fn scroll_mode(mut self, mode: ScrollMode) -> Self {
        self.scroll_mode = mode;
        self
    }

    /// Set maximum height for full page screenshots
    pub fn max_height(mut self, height: u32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set scroll delay between captures
    pub fn scroll_delay(mut self, delay: u64) -> Self {
        self.scroll_delay = delay;
        self
    }

    /// Validate the options
    pub fn validate(&self) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(WebshotError::InvalidViewport {
                width: self.width,
                height: self.height,
            });
        }

        if let Some(quality) = self.quality {
            if !(1..=100).contains(&quality) {
                return Err(WebshotError::config(format!(
                    "JPEG quality must be between 1-100, got: {}",
                    quality
                )));
            }
        }

        if self.timeout == 0 {
            return Err(WebshotError::config(
                "Timeout must be greater than 0".to_string(),
            ));
        }

        // Validate max height for full page screenshots
        if let Some(max_height) = self.max_height {
            if max_height == 0 {
                return Err(WebshotError::config(
                    "Max height must be greater than 0".to_string(),
                ));
            }
            if max_height > 100000 {
                return Err(WebshotError::config(
                    "Max height should not exceed 100,000 pixels for safety".to_string(),
                ));
            }
        }

        // Validate scroll mode compatibility
        if self.scroll_mode == ScrollMode::FullElement && self.selector.is_none() {
            return Err(WebshotError::config(
                "FullElement scroll mode requires a selector to be specified".to_string(),
            ));
        }

        Ok(())
    }

    /// Get device scale factor based on retina setting
    pub fn device_scale_factor(&self) -> f64 {
        if self.retina {
            2.0
        } else {
            1.0
        }
    }

    /// Determine output format from file path
    pub fn output_format<P: AsRef<Path>>(&self, path: P) -> Result<ImageFormat> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| WebshotError::config("No file extension found".to_string()))?;

        match extension.as_str() {
            "png" => Ok(ImageFormat::Png),
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "pdf" => Ok(ImageFormat::Pdf),
            "webp" => Ok(ImageFormat::WebP),
            _ => Err(WebshotError::UnsupportedFormat { format: extension }),
        }
    }
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Pdf,
}

impl ImageFormat {
    /// Get the default file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::WebP => "webp",
            ImageFormat::Pdf => "pdf",
        }
    }

    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Pdf => "application/pdf",
        }
    }

    /// Check if this format supports quality settings
    pub fn supports_quality(&self) -> bool {
        matches!(self, ImageFormat::Jpeg | ImageFormat::WebP)
    }

    /// Check if this format supports transparency
    pub fn supports_transparency(&self) -> bool {
        matches!(self, ImageFormat::Png | ImageFormat::WebP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_screenshot_options_builder() {
        let options = ScreenshotOptions::new()
            .viewport(1920, 1080)
            .selector(".header")
            .javascript("console.log('test')")
            .wait_for(".content")
            .timeout(60)
            .retina()
            .quality(90)
            .wait(5)
            .user_agent("Custom Agent")
            .full_page()
            .max_height(50000)
            .scroll_delay(200);

        assert_eq!(options.width, 1920);
        assert_eq!(options.height, 1080);
        assert_eq!(options.selector.as_deref(), Some(".header"));
        assert_eq!(options.javascript.as_deref(), Some("console.log('test')"));
        assert_eq!(options.wait_for.as_deref(), Some(".content"));
        assert_eq!(options.timeout, 60);
        assert!(options.retina);
        assert_eq!(options.quality, Some(90));
        assert_eq!(options.wait, 5);
        assert_eq!(options.user_agent.as_deref(), Some("Custom Agent"));
        assert_eq!(options.scroll_mode, ScrollMode::FullPage);
        assert_eq!(options.max_height, Some(50000));
        assert_eq!(options.scroll_delay, 200);
    }

    #[test]
    fn test_options_validation() {
        let mut options = ScreenshotOptions::new();
        assert!(options.validate().is_ok());

        options.width = 0;
        assert!(options.validate().is_err());

        options.width = 1280;
        options.quality = Some(150);
        assert!(options.validate().is_err());

        options.quality = Some(80);
        options.timeout = 0;
        assert!(options.validate().is_err());
    }

    #[test]
    fn test_output_format_detection() {
        let options = ScreenshotOptions::new();

        assert_eq!(
            options.output_format(PathBuf::from("test.png")).unwrap(),
            ImageFormat::Png
        );
        assert_eq!(
            options.output_format(PathBuf::from("test.jpg")).unwrap(),
            ImageFormat::Jpeg
        );
        assert_eq!(
            options.output_format(PathBuf::from("test.jpeg")).unwrap(),
            ImageFormat::Jpeg
        );
        assert_eq!(
            options.output_format(PathBuf::from("test.pdf")).unwrap(),
            ImageFormat::Pdf
        );
        assert_eq!(
            options.output_format(PathBuf::from("test.webp")).unwrap(),
            ImageFormat::WebP
        );

        assert!(options.output_format(PathBuf::from("test.gif")).is_err());
        assert!(options.output_format(PathBuf::from("test")).is_err());
    }

    #[test]
    fn test_device_scale_factor() {
        let options = ScreenshotOptions::new();
        assert_eq!(options.device_scale_factor(), 1.0);

        let retina_options = options.retina();
        assert_eq!(retina_options.device_scale_factor(), 2.0);
    }

    #[test]
    fn test_image_format() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Pdf.extension(), "pdf");
        assert_eq!(ImageFormat::WebP.extension(), "webp");

        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(ImageFormat::WebP.mime_type(), "image/webp");

        assert!(!ImageFormat::Png.supports_quality());
        assert!(ImageFormat::Jpeg.supports_quality());
        assert!(!ImageFormat::Pdf.supports_quality());
        assert!(ImageFormat::WebP.supports_quality());

        assert!(ImageFormat::Png.supports_transparency());
        assert!(!ImageFormat::Jpeg.supports_transparency());
        assert!(!ImageFormat::Pdf.supports_transparency());
        assert!(ImageFormat::WebP.supports_transparency());
    }

    #[test]
    fn test_scroll_mode_defaults() {
        let options = ScreenshotOptions::new();
        assert_eq!(options.scroll_mode, ScrollMode::Viewport);
        assert_eq!(options.max_height, Some(30000));
        assert_eq!(options.scroll_delay, 100);
    }

    #[test]
    fn test_scroll_mode_builder() {
        let full_page_options = ScreenshotOptions::new().full_page();
        assert_eq!(full_page_options.scroll_mode, ScrollMode::FullPage);

        let full_element_options = ScreenshotOptions::new().selector(".content").full_element();
        assert_eq!(full_element_options.scroll_mode, ScrollMode::FullElement);
    }

    #[test]
    fn test_scroll_validation() {
        // Full element mode requires a selector
        let mut options = ScreenshotOptions::new().full_element();
        assert!(options.validate().is_err());

        // With selector it should work
        options = options.selector(".content");
        assert!(options.validate().is_ok());

        // Max height validation
        options = ScreenshotOptions::new().max_height(0);
        assert!(options.validate().is_err());

        options = ScreenshotOptions::new().max_height(200000); // Too large
        assert!(options.validate().is_err());
    }
}
