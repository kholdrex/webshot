# WebShot

A fast command-line tool for taking website screenshots, built in Rust.

## Features

- Take full-page or element-specific screenshots
- Generate PDFs from web pages
- Execute JavaScript before capturing
- Batch processing with YAML configs
- Support for PNG, JPEG, WebP, and PDF formats
- Custom viewports and mobile emulation
- Wait for elements or timeouts
- Extract text content from pages
- Multiple comparison algorithms (pixel-diff, SSIM, MSE, PSNR)
- Generate difference images highlighting changes
- Visual regression testing support
- Configurable similarity thresholds
- Works on Windows, macOS, and Linux

## Installation

### From Source

```bash
git clone https://github.com/kholdrex/webshot.git
cd webshot
cargo install --path .
```

You'll need Chrome or Chromium installed. The tool will find it automatically.

## Quick Start

```bash
# Basic screenshot
webshot https://example.com

# Custom size and output
webshot https://example.com -o screenshot.png -w 1920 -h 1080

# Screenshot just the header
webshot https://github.com -s ".Header" -o header.png

# Generate a PDF
webshot pdf https://example.com -o page.pdf

# Screenshot in WebP format
webshot https://example.com -o screenshot.webp

# Extract text content
webshot text https://example.com
```

## Usage

### Basic Options

- `-o, --output` - Output file path
- `-w, --width` - Viewport width (default: 1280)
- `-h, --height` - Viewport height (default: 800)
- `-s, --selector` - CSS selector for element screenshots
- `-j, --javascript` - JavaScript to run before screenshot
- `--wait-for` - Wait for element to appear
- `-t, --timeout` - Timeout in seconds (default: 30)
- `--retina` - Enable high-DPI mode
- `-q, --quality` - JPEG/WebP quality 1-100
- `-v, --verbose` - Verbose logging

### Subcommands

#### `screenshot`
Basic screenshot with full options:
```bash
webshot screenshot https://example.com -o test.png -w 1920 -h 1080
```

#### `pdf`
Generate PDF from webpage:
```bash
webshot pdf https://example.com -o page.pdf --landscape --background
```

#### `multi`
Process multiple screenshots from YAML config:
```bash
webshot multi config.yaml -o output/ -p 4
```

#### `text`
Extract text content:
```bash
webshot text https://example.com -s "article" -o content.txt
```

#### `compare`
Compare two images for differences:
```bash
# Basic comparison
webshot compare image1.png image2.png

# Use different algorithm with threshold
webshot compare baseline.png current.png -a ssim -t 0.05

# Generate difference image
webshot compare old.png new.png --diff-image --diff-path diff.png

# Output results as JSON
webshot compare img1.png img2.png --format json -o results.json

# Ignore anti-aliasing differences
webshot compare baseline.png current.png --ignore-antialiasing
```

## Configuration Files

For batch processing, create a YAML file:

```yaml
# Simple config
screenshots:
  - url: "https://example.com"
    output: "example.png"
    
  - url: "https://github.com"
    output: "github.png"
    width: 1920
    height: 1080
```

Advanced config with defaults:

```yaml
defaults:
  width: 1280
  height: 800
  timeout: 30
  output_dir: "screenshots"

screenshots:
  - url: "https://github.com"
    output: "github-header.png"
    selector: ".Header"

  - url: "https://example.com"
    output: "interactive.png"
    javascript: "document.querySelector('button').click();"

  - url: "https://spa-app.com"
    output: "spa-loaded.png"
    wait_for: ".content"
    timeout: 15
```

### Configuration Options

- `url` - Target URL (required)
- `output` - Output file path (required)
- `width`, `height` - Viewport dimensions
- `selector` - CSS selector for element screenshots
- `javascript` - JavaScript code to execute
- `wait_for` - CSS selector to wait for
- `timeout` - Timeout in seconds
- `retina` - Enable retina mode
- `quality` - JPEG/WebP quality 1-100
- `wait` - Wait time before screenshot
- `user_agent` - Custom user agent
- `headers` - Custom HTTP headers
- `cookies` - Cookies to set
- `auth` - Basic authentication (username/password)

## Examples

### Mobile Screenshots
```bash
# iPhone viewport
webshot https://example.com -w 390 -h 844 -o mobile.png

# iPad viewport  
webshot https://example.com -w 820 -h 1180 -o tablet.png
```

### JavaScript Execution
```bash
# Click elements and modify page
webshot https://app.example.com -j "
  document.querySelector('#username').value = 'user';
  document.querySelector('#password').value = 'pass';
  document.querySelector('#login').click();
  await new Promise(r => setTimeout(r, 2000));
" --wait-for ".dashboard" -o dashboard.png
```

### Custom Chrome Setup
```bash
# Use custom Chrome path
webshot https://example.com --chrome-path="/opt/google/chrome/chrome"

# Add Chrome flags
webshot https://localhost:3000 --chrome-flag="--disable-web-security"
```

## Troubleshooting

**Chrome not found**: Use `--chrome-path` to specify location manually

**Element not found**: Check CSS selector syntax, use `--wait-for` for dynamic content

**Timeouts**: Increase with `-t` flag, check network connection

**JavaScript errors**: Use `-v` for verbose logging

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Integration tests (needs Chrome)
cargo test --test integration
```

## License

MIT License - see LICENSE file for details.

Built with [headless_chrome](https://github.com/rust-headless-chrome/rust-headless-chrome) and [clap](https://github.com/clap-rs/clap).