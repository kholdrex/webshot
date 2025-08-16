# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2025-08-16

### Added
- Advanced image comparison functionality with multiple algorithms
  - Pixel-by-pixel difference comparison
  - Structural Similarity Index (SSIM) for perceptual similarity
  - Mean Squared Error (MSE) and Peak Signal-to-Noise Ratio (PSNR) metrics
  - Configurable similarity thresholds and anti-aliasing tolerance
  - Difference image generation highlighting visual changes
  - Support for JSON and text output formats
- Full WebP image format support
  - Lossless WebP encoding for high-quality screenshots
  - WebP quality control (1-100) for lossy compression
  - Automatic format detection from file extensions
- New `compare` subcommand for image analysis
  - Direct image-to-image comparison with detailed metrics
  - Batch comparison support through configuration files
  - Visual regression testing capabilities
  - Exit codes for CI/CD integration (0 = similar, 1 = different)

## [0.1.0] - 2025-08-14

### Added
- Initial release of WebShot command-line tool
- Basic screenshot functionality for websites
- PDF generation from web pages
- Text extraction from web content
- Element-specific screenshots using CSS selectors
- JavaScript execution before capturing screenshots
- Batch processing with YAML configuration files
- Support for multiple output formats (PNG, JPEG, PDF)
- Custom viewport dimensions and mobile emulation
- Retina/high-DPI screenshot support
- Wait conditions for dynamic content
- Custom user agent and Chrome flags support
- Parallel processing for batch operations
- Comprehensive CLI with subcommands:
  - `screenshot` - Take single screenshots
  - `pdf` - Generate PDFs from web pages
  - `multi` - Process batch configurations
  - `text` - Extract text content
- Cookie and authentication support
- Custom headers for HTTP requests
- Timeout configuration and error handling
- Verbose logging options

### Features
- Cross-platform support (Windows, macOS, Linux)
- Automatic Chrome/Chromium detection
- Built-in image format conversion
- Filename sanitization and path resolution
- Comprehensive error handling with detailed messages
- Configuration validation and helpful error reporting
- Integration tests covering all major functionality

### Documentation
- Complete README with usage examples
- Example configuration files for different use cases
- Installation and troubleshooting guides
- CLI help documentation

### Dependencies
- Uses headless Chrome for browser automation
- Clap for command-line interface
- Tokio for async runtime
- Serde for configuration parsing
- Image processing with the image crate
- Comprehensive logging with tracing
