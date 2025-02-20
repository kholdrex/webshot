use std::path::PathBuf;
use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum WebshotError {
    #[error("Browser error: {0}")]
    Browser(#[from] anyhow::Error),

    #[error("Browser launch error: {0}")]
    BrowserLaunch(String),

    #[error("Tab error: {0}")]
    Tab(String),

    #[error("Navigation error: {0}")]
    Navigation(String),

    #[error("Screenshot error: {0}")]
    Screenshot(String),

    #[error("Element not found: {selector}")]
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
}
