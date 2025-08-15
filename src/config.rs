use crate::error::{Result, WebshotError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Batch processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// List of screenshots to take
    pub screenshots: Vec<ScreenshotConfig>,
    /// Global settings that apply to all screenshots
    #[serde(default)]
    pub defaults: DefaultConfig,
}

/// Individual screenshot configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScreenshotConfig {
    /// Target URL
    pub url: String,
    /// Output file path
    pub output: PathBuf,
    /// Viewport width
    #[serde(default = "default_width")]
    pub width: u32,
    /// Viewport height
    #[serde(default = "default_height")]
    pub height: u32,
    /// CSS selector for element screenshot
    pub selector: Option<String>,
    /// JavaScript to execute before screenshot
    pub javascript: Option<String>,
    /// Element to wait for before taking screenshot
    pub wait_for: Option<String>,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Enable retina/high-DPI mode
    #[serde(default)]
    pub retina: bool,
    /// JPEG quality (1-100)
    pub quality: Option<u8>,
    /// Wait time before taking screenshot
    #[serde(default)]
    pub wait: u64,
    /// Custom user agent
    pub user_agent: Option<String>,
    /// Output format override
    pub format: Option<String>,
    /// Custom headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    /// Cookies to set
    #[serde(default)]
    pub cookies: Vec<CookieConfig>,
    /// Authentication credentials
    pub auth: Option<AuthConfig>,
    /// Comparison configuration for visual regression testing
    pub comparison: Option<ComparisonConfig>,
}

/// Cookie configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CookieConfig {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
}

/// Comparison configuration for visual regression testing
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComparisonConfig {
    /// Base image path for comparison
    pub baseline_path: Option<String>,
    /// Comparison algorithm to use
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    /// Threshold for considering images similar (0.0-1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f64,
    /// Generate difference image
    #[serde(default)]
    pub generate_diff: bool,
    /// Path for difference image output
    pub diff_output_path: Option<String>,
    /// Ignore anti-aliasing differences
    #[serde(default)]
    pub ignore_antialiasing: bool,
    /// Color for highlighting differences (RGB format: "255,0,0")
    #[serde(default = "default_diff_color")]
    pub diff_color: String,
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub username: String,
    pub password: String,
}

/// Default configuration applied to all screenshots
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefaultConfig {
    /// Default viewport width
    #[serde(default = "default_width")]
    pub width: u32,
    /// Default viewport height
    #[serde(default = "default_height")]
    pub height: u32,
    /// Default timeout
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Default user agent
    pub user_agent: Option<String>,
    /// Default output directory
    pub output_dir: Option<PathBuf>,
    /// Default wait time
    #[serde(default)]
    pub wait: u64,
    /// Default retina mode
    #[serde(default)]
    pub retina: bool,
    /// Default JPEG quality
    pub quality: Option<u8>,
    /// Global headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    /// Global cookies
    #[serde(default)]
    pub cookies: Vec<CookieConfig>,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            timeout: default_timeout(),
            user_agent: None,
            output_dir: None,
            wait: 0,
            retina: false,
            quality: None,
            headers: std::collections::HashMap::new(),
            cookies: Vec::new(),
        }
    }
}

impl Config {
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let mut config: Config = serde_yaml::from_str(&content)?;

        // Apply defaults to screenshots that don't have values set
        for screenshot in &mut config.screenshots {
            if screenshot.width == default_width() && config.defaults.width != default_width() {
                screenshot.width = config.defaults.width;
            }
            if screenshot.height == default_height() && config.defaults.height != default_height() {
                screenshot.height = config.defaults.height;
            }
            if screenshot.timeout == default_timeout() && config.defaults.timeout != default_timeout() {
                screenshot.timeout = config.defaults.timeout;
            }
            if screenshot.user_agent.is_none() && config.defaults.user_agent.is_some() {
                screenshot.user_agent = config.defaults.user_agent.clone();
            }
            if screenshot.quality.is_none() && config.defaults.quality.is_some() {
                screenshot.quality = config.defaults.quality;
            }
            
            // Merge headers
            for (key, value) in &config.defaults.headers {
                screenshot.headers.entry(key.clone()).or_insert_with(|| value.clone());
            }
            
            // Merge cookies
            if screenshot.cookies.is_empty() && !config.defaults.cookies.is_empty() {
                screenshot.cookies = config.defaults.cookies.clone();
            }

            // Resolve output path relative to output_dir if set
            if let Some(output_dir) = &config.defaults.output_dir {
                if screenshot.output.is_relative() {
                    screenshot.output = output_dir.join(&screenshot.output);
                }
            }
        }

        Ok(config)
    }

    /// Save configuration to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.screenshots.is_empty() {
            return Err(WebshotError::config("No screenshots defined in configuration"));
        }

        for (i, screenshot) in self.screenshots.iter().enumerate() {
            // Validate URL
            url::Url::parse(&screenshot.url)
                .map_err(|e| WebshotError::config(format!("Invalid URL in screenshot {}: {}", i, e)))?;

            // Validate viewport dimensions
            if screenshot.width == 0 || screenshot.height == 0 {
                return Err(WebshotError::InvalidViewport {
                    width: screenshot.width,
                    height: screenshot.height,
                });
            }

            // Validate JPEG quality
            if let Some(quality) = screenshot.quality {
                if !(1..=100).contains(&quality) {
                    return Err(WebshotError::config(format!(
                        "JPEG quality must be between 1-100, got: {}",
                        quality
                    )));
                }
            }

            // Validate timeout
            if screenshot.timeout == 0 {
                return Err(WebshotError::config(format!(
                    "Timeout must be greater than 0, got: {}",
                    screenshot.timeout
                )));
            }

            // Validate output file extension if format is not specified
            if screenshot.format.is_none() {
                let extension = screenshot
                    .output
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_lowercase());

                match extension.as_deref() {
                    Some("png") | Some("jpg") | Some("jpeg") | Some("pdf") => {}
                    Some(ext) => {
                        return Err(WebshotError::UnsupportedFormat {
                            format: ext.to_string(),
                        });
                    }
                    None => {
                        return Err(WebshotError::config(format!(
                            "Output file must have a valid extension: {}",
                            screenshot.output.display()
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

fn default_width() -> u32 {
    1280
}

fn default_height() -> u32 {
    800
}

fn default_timeout() -> u64 {
    30
}

fn default_algorithm() -> String {
    "pixel-diff".to_string()
}

fn default_threshold() -> f64 {
    0.1
}

fn default_diff_color() -> String {
    "255,0,0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_config_serialization() {
        let config = Config {
            screenshots: vec![ScreenshotConfig {
                url: "https://example.com".to_string(),
                output: PathBuf::from("test.png"),
                width: 1920,
                height: 1080,
                selector: Some(".header".to_string()),
                javascript: None,
                wait_for: None,
                timeout: 30,
                retina: false,
                quality: None,
                wait: 0,
                user_agent: None,
                format: None,
                headers: std::collections::HashMap::new(),
                cookies: Vec::new(),
                auth: None,
                comparison: None,
            }],
            defaults: DefaultConfig::default(),
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(config.screenshots.len(), deserialized.screenshots.len());
        assert_eq!(config.screenshots[0].url, deserialized.screenshots[0].url);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config {
            screenshots: vec![ScreenshotConfig {
                url: "https://example.com".to_string(),
                output: PathBuf::from("test.png"),
                width: 1920,
                height: 1080,
                selector: None,
                javascript: None,
                wait_for: None,
                timeout: 30,
                retina: false,
                quality: None,
                wait: 0,
                user_agent: None,
                format: None,
                headers: std::collections::HashMap::new(),
                cookies: Vec::new(),
                auth: None,
                comparison: None,
            }],
            defaults: DefaultConfig::default(),
        };

        assert!(config.validate().is_ok());

        // Test invalid URL
        config.screenshots[0].url = "not-a-url".to_string();
        assert!(config.validate().is_err());

        // Test invalid dimensions
        config.screenshots[0].url = "https://example.com".to_string();
        config.screenshots[0].width = 0;
        assert!(config.validate().is_err());
    }
}
