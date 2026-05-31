use std::path::PathBuf;
use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum WebshotError {
    #[error("Browser error: {0}")]
    Browser(#[from] anyhow::Error),

    #[error("Browser launch failed: {0}. Verify that Chrome or Chromium is installed and reachable, or pass --chrome-path with the executable path. In containers, also try --chrome-flag=--no-sandbox and confirm the process can write to its temporary directory.")]
    BrowserLaunch(String),

    #[error("Tab error: {0}")]
    Tab(String),

    #[error("Navigation failed: {0}. Check that the URL includes a supported scheme such as https://, the page is reachable from this machine, and the timeout is long enough for the page to load.")]
    Navigation(String),

    #[error("Screenshot error: {0}")]
    Screenshot(String),

    #[error("Element not found for selector '{selector}'. Check that the selector is correct, the element is present after page load, or use --wait-for/--timeout when content appears asynchronously.")]
    ElementNotFound { selector: String },

    #[error("JavaScript execution error: {0}")]
    JavaScript(String),

    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid file path: {path}")]
    InvalidPath { path: PathBuf },

    #[error("Unsupported image format: {format}")]
    UnsupportedFormat { format: String },

    #[error("Timeout waiting for condition: {condition}")]
    Timeout { condition: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("PDF generation error: {0}")]
    Pdf(String),

    #[error("Invalid viewport dimensions: width={width}, height={height}")]
    InvalidViewport { width: u32, height: u32 },
}

/// Result type alias
pub type Result<T> = std::result::Result<T, WebshotError>;

impl WebshotError {
    /// Create a navigation error
    pub fn navigation(msg: impl Into<String>) -> Self {
        Self::Navigation(msg.into())
    }

    /// Create a browser launch error
    pub fn browser_launch(msg: impl Into<String>) -> Self {
        Self::BrowserLaunch(msg.into())
    }

    /// Create a screenshot error
    pub fn screenshot(msg: impl Into<String>) -> Self {
        Self::Screenshot(msg.into())
    }

    /// Create a JavaScript error
    pub fn javascript(msg: impl Into<String>) -> Self {
        Self::JavaScript(msg.into())
    }

    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a PDF error
    pub fn pdf(msg: impl Into<String>) -> Self {
        Self::Pdf(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(condition: impl Into<String>) -> Self {
        Self::Timeout {
            condition: condition.into(),
        }
    }

    /// Create an element-not-found error.
    pub fn element_not_found(selector: impl Into<String>) -> Self {
        Self::ElementNotFound {
            selector: selector.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_launch_error_includes_chrome_path_guidance() {
        let error = WebshotError::browser_launch("No such file or directory");
        let message = error.to_string();

        assert!(message.contains("Browser launch failed"));
        assert!(message.contains("No such file or directory"));
        assert!(message.contains("--chrome-path"));
        assert!(message.contains("Chrome or Chromium"));
    }

    #[test]
    fn navigation_error_includes_url_and_timeout_guidance() {
        let error = WebshotError::navigation("net::ERR_NAME_NOT_RESOLVED");
        let message = error.to_string();

        assert!(message.contains("Navigation failed"));
        assert!(message.contains("net::ERR_NAME_NOT_RESOLVED"));
        assert!(message.contains("https://"));
        assert!(message.contains("timeout"));
    }

    #[test]
    fn missing_element_error_includes_selector_and_wait_guidance() {
        let error = WebshotError::element_not_found(".loaded-later");
        let message = error.to_string();

        assert!(message.contains(".loaded-later"));
        assert!(message.contains("--wait-for"));
        assert!(message.contains("--timeout"));
    }
}
