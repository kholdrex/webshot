pub mod browser;
pub mod comparison;
pub mod config;
pub mod error;
pub mod output;
pub mod screenshot;

pub use error::{Result, WebshotError};

// Re-export commonly used types
pub use browser::Browser;
pub use comparison::{ComparisonOptions, ComparisonResult, ImageComparator};
pub use config::{Config, ScreenshotConfig};
pub use screenshot::ScreenshotOptions;
