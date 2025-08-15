use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use webshot::{Browser, Config, Result, ScreenshotOptions, ComparisonOptions, ImageComparator};

#[derive(Parser)]
#[command(
    name = "webshot",
    version = env!("CARGO_PKG_VERSION"),
    about = "Take screenshots of websites from the command line",
    long_about = "A fast command-line tool for taking website screenshots, generating PDFs, \
                  and extracting web content. Built with Rust and Chrome DevTools."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// URL to screenshot (if no subcommand provided)
    #[arg(value_name = "URL")]
    url: Option<String>,

    /// Output file path
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Viewport width
    #[arg(short, long, default_value = "1280")]
    width: u32,

    /// Viewport height
    #[arg(short = 'H', long, default_value = "800")]
    height: u32,

    /// CSS selector for element screenshot
    #[arg(short, long, value_name = "SELECTOR")]
    selector: Option<String>,

    /// JavaScript to execute before screenshot
    #[arg(short, long, value_name = "SCRIPT")]
    javascript: Option<String>,

    /// Wait for element to appear (CSS selector)
    #[arg(long, value_name = "SELECTOR")]
    wait_for: Option<String>,

    /// Timeout in seconds
    #[arg(short, long, default_value = "30")]
    timeout: u64,

    /// Enable retina/high-DPI mode
    #[arg(long)]
    retina: bool,

    /// JPEG/WebP quality (1-100, only for JPEG and WebP output)
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: Option<u8>,

    /// Wait time in seconds before taking screenshot
    #[arg(long, default_value = "0")]
    wait: u64,

    /// Custom user agent string
    #[arg(long)]
    user_agent: Option<String>,

    /// Verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Disable JavaScript
    #[arg(long)]
    no_javascript: bool,

    /// Custom Chrome/Chromium executable path
    #[arg(long)]
    chrome_path: Option<PathBuf>,

    /// Additional Chrome flags
    #[arg(long, action = clap::ArgAction::Append)]
    chrome_flag: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Take a single screenshot
    #[command(alias = "shot")]
    Screenshot {
        /// URL to screenshot
        url: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Viewport width
        #[arg(short, long, default_value = "1280")]
        width: u32,
        /// Viewport height
        #[arg(short = 'H', long, default_value = "800")]
        height: u32,
        /// CSS selector for element screenshot
        #[arg(short, long)]
        selector: Option<String>,
        /// JavaScript to execute
        #[arg(short, long)]
        javascript: Option<String>,
        /// Wait for element
        #[arg(long)]
        wait_for: Option<String>,
        /// Timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Enable retina mode
        #[arg(long)]
        retina: bool,
        /// JPEG/WebP quality
        #[arg(short, long)]
        quality: Option<u8>,
        /// Wait time before screenshot
        #[arg(long, default_value = "0")]
        wait: u64,
    },
    /// Generate PDF from webpage
    Pdf {
        /// URL to convert to PDF
        url: String,
        /// Output PDF file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Page format (A4, Letter, etc.)
        #[arg(long, default_value = "A4")]
        format: String,
        /// Landscape orientation
        #[arg(long)]
        landscape: bool,
        /// Print background graphics
        #[arg(long)]
        background: bool,
        /// Scale factor (0.1 to 2.0)
        #[arg(long, default_value = "1.0")]
        scale: f64,
        /// JavaScript to execute
        #[arg(short, long)]
        javascript: Option<String>,
        /// Wait for element
        #[arg(long)]
        wait_for: Option<String>,
        /// Timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,
    },
    /// Process multiple screenshots from YAML config
    Multi {
        /// Configuration file path
        config_file: PathBuf,
        /// Override output directory
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        /// Parallel processing (number of concurrent tasks)
        #[arg(short, long, default_value = "4")]
        parallel: usize,
    },
    /// Extract text content from webpage
    Text {
        /// URL to extract text from
        url: String,
        /// CSS selector for specific element
        #[arg(short, long)]
        selector: Option<String>,
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// JavaScript to execute
        #[arg(short, long)]
        javascript: Option<String>,
        /// Wait for element
        #[arg(long)]
        wait_for: Option<String>,
        /// Timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,
    },
    /// Compare two images for differences
    #[command(alias = "diff")]
    Compare {
        /// First image to compare
        image1: PathBuf,
        /// Second image to compare
        image2: PathBuf,
        /// Output file for comparison results (JSON format)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Comparison algorithm
        #[arg(short, long, default_value = "pixel-diff")]
        algorithm: String,
        /// Similarity threshold (0.0-1.0, higher means more strict)
        #[arg(short, long, default_value = "0.1")]
        threshold: f64,
        /// Generate difference image highlighting changes
        #[arg(long)]
        diff_image: bool,
        /// Path for difference image (required if --diff-image is used)
        #[arg(long)]
        diff_path: Option<PathBuf>,
        /// Ignore anti-aliasing differences
        #[arg(long)]
        ignore_antialiasing: bool,
        /// Color for highlighting differences (RGB format: 255,0,0)
        #[arg(long, default_value = "255,0,0")]
        diff_color: String,
        /// Output format for results (json, text)
        #[arg(long, default_value = "text")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose);

    // Extract values we need from cli to avoid borrow checker issues
    let chrome_path = cli.chrome_path.clone();
    let chrome_flags = cli.chrome_flag.clone();
    let no_javascript = cli.no_javascript;
    let user_agent = cli.user_agent.clone();

    // Handle the command
    match cli.command {
        Some(Commands::Screenshot {
            url,
            output,
            width,
            height,
            selector,
            javascript,
            wait_for,
            timeout,
            retina,
            quality,
            wait,
        }) => {
            take_screenshot(
                &url, output, width, height, selector, javascript, wait_for, timeout, retina,
                quality, wait, chrome_path, chrome_flags, no_javascript, user_agent,
            )
            .await
        }
        Some(Commands::Pdf {
            url,
            output,
            format,
            landscape,
            background,
            scale,
            javascript,
            wait_for,
            timeout,
        }) => {
            generate_pdf(
                &url, output, &format, landscape, background, scale, javascript, wait_for,
                timeout, chrome_path, chrome_flags, no_javascript, user_agent,
            )
            .await
        }
        Some(Commands::Multi {
            config_file,
            output_dir,
            parallel,
        }) => process_config(&config_file, output_dir, parallel, chrome_path, chrome_flags, no_javascript).await,
        Some(Commands::Text {
            url,
            selector,
            output,
            javascript,
            wait_for,
            timeout,
        }) => {
            extract_text(&url, selector, output, javascript, wait_for, timeout, chrome_path, chrome_flags, no_javascript, user_agent).await
        }
        Some(Commands::Compare {
            image1,
            image2,
            output,
            algorithm,
            threshold,
            diff_image,
            diff_path,
            ignore_antialiasing,
            diff_color,
            format,
        }) => {
            compare_images(
                &image1, &image2, output, &algorithm, threshold, diff_image, diff_path,
                ignore_antialiasing, &diff_color, &format,
            ).await
        }
        None => {
            // Default behavior: screenshot with URL as positional argument
            if let Some(url) = &cli.url {
                take_screenshot(
                    url,
                    cli.output,
                    cli.width,
                    cli.height,
                    cli.selector,
                    cli.javascript,
                    cli.wait_for,
                    cli.timeout,
                    cli.retina,
                    cli.quality,
                    cli.wait,
                    chrome_path,
                    chrome_flags,
                    no_javascript,
                    user_agent,
                )
                .await
            } else {
                eprintln!("Error: URL is required when no subcommand is provided");
                eprintln!("Use 'webshot --help' for usage information");
                std::process::exit(1);
            }
        }
    }
}

fn init_logging(verbose: u8) {
    let filter = match verbose {
        0 => "webshot=warn",
        1 => "webshot=info",
        2 => "webshot=debug",
        _ => "webshot=trace,headless_chrome=debug",
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

async fn take_screenshot(
    url: &str,
    output: Option<PathBuf>,
    width: u32,
    height: u32,
    selector: Option<String>,
    javascript: Option<String>,
    wait_for: Option<String>,
    timeout: u64,
    retina: bool,
    quality: Option<u8>,
    wait: u64,
    chrome_path: Option<PathBuf>,
    chrome_flags: Vec<String>,
    no_javascript: bool,
    user_agent: Option<String>,
) -> Result<()> {
    info!("Taking screenshot of: {}", url);

    let browser = Browser::new(
        chrome_path,
        chrome_flags,
        !no_javascript,
    )
    .await?;

    let options = ScreenshotOptions {
        width,
        height,
        selector,
        javascript,
        wait_for,
        timeout,
        retina,
        quality,
        wait,
        user_agent,
    };

    let output_path = output.as_ref().map(|p| p.clone()).unwrap_or_else(|| {
        // Determine format from output path or default to PNG
        let format = if let Some(ref output_path) = output {
            if let Some(ext) = output_path.extension() {
                match ext.to_str().unwrap_or("").to_lowercase().as_str() {
                    "jpg" | "jpeg" => "jpg",
                    "webp" => "webp",
                    "pdf" => "pdf",
                    _ => "png",
                }
            } else {
                "png"
            }
        } else {
            "png"
        };
        
        PathBuf::from(format!(
            "screenshot_{}.{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            format
        ))
    });

    browser.screenshot(url, &output_path, &options).await?;

    println!("Screenshot saved to: {}", output_path.display());
    Ok(())
}

async fn generate_pdf(
    url: &str,
    output: Option<PathBuf>,
    format: &str,
    landscape: bool,
    background: bool,
    scale: f64,
    javascript: Option<String>,
    wait_for: Option<String>,
    timeout: u64,
    chrome_path: Option<PathBuf>,
    chrome_flags: Vec<String>,
    no_javascript: bool,
    user_agent: Option<String>,
) -> Result<()> {
    info!("Generating PDF of: {}", url);

    let browser = Browser::new(
        chrome_path,
        chrome_flags,
        !no_javascript,
    )
    .await?;

    let output_path = output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "page_{}.pdf",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        ))
    });

    browser
        .pdf(
            url,
            &output_path,
            format,
            landscape,
            background,
            scale,
            javascript,
            wait_for,
            timeout,
            user_agent,
        )
        .await?;

    println!("PDF saved to: {}", output_path.display());
    Ok(())
}

async fn process_config(
    config_file: &PathBuf,
    output_dir: Option<PathBuf>,
    parallel: usize,
    chrome_path: Option<PathBuf>,
    chrome_flags: Vec<String>,
    no_javascript: bool,
) -> Result<()> {
    info!("Processing config file: {}", config_file.display());

    let config = Config::from_file(config_file)?;
    let browser = Browser::new(
        chrome_path,
        chrome_flags,
        !no_javascript,
    )
    .await?;

    browser.process_config(&config, output_dir, parallel).await?;

    println!("Batch processing completed successfully");
    Ok(())
}

async fn extract_text(
    url: &str,
    selector: Option<String>,
    output: Option<PathBuf>,
    javascript: Option<String>,
    wait_for: Option<String>,
    timeout: u64,
    chrome_path: Option<PathBuf>,
    chrome_flags: Vec<String>,
    no_javascript: bool,
    user_agent: Option<String>,
) -> Result<()> {
    info!("Extracting text from: {}", url);

    let browser = Browser::new(
        chrome_path,
        chrome_flags,
        !no_javascript,
    )
    .await?;

    let text = browser
        .extract_text(url, selector, javascript, wait_for, timeout, user_agent)
        .await?;

    match output {
        Some(path) => {
            std::fs::write(&path, &text)?;
            println!("Text saved to: {}", path.display());
        }
        None => {
            println!("{}", text);
        }
    }

    Ok(())
}

/// Compare two images and output results
async fn compare_images(
    image1_path: &std::path::Path,
    image2_path: &std::path::Path,
    output: Option<PathBuf>,
    algorithm: &str,
    threshold: f64,
    diff_image: bool,
    diff_path: Option<PathBuf>,
    ignore_antialiasing: bool,
    diff_color: &str,
    output_format: &str,
) -> Result<()> {
    use webshot::comparison::{ComparisonAlgorithm};

    // Parse algorithm
    let algorithm = match algorithm.to_lowercase().as_str() {
        "pixel-diff" | "pixel" => ComparisonAlgorithm::PixelDiff,
        "ssim" => ComparisonAlgorithm::SSIM,
        "mse" => ComparisonAlgorithm::MSE,
        "psnr" => ComparisonAlgorithm::PSNR,
        _ => return Err(webshot::WebshotError::config(format!(
            "Unknown algorithm: {}. Supported: pixel-diff, ssim, mse, psnr", algorithm
        ))),
    };

    // Parse diff color
    let diff_color = parse_rgb_color(diff_color)?;

    // Validate inputs
    if diff_image && diff_path.is_none() {
        return Err(webshot::WebshotError::config(
            "Diff path must be specified when --diff-image is used".to_string(),
        ));
    }

    // Build comparison options
    let mut options = ComparisonOptions::new()
        .algorithm(algorithm)
        .threshold(threshold)
        .diff_color(diff_color.0, diff_color.1, diff_color.2);

    if ignore_antialiasing {
        options = options.ignore_antialiasing();
    }

    if diff_image {
        if let Some(path) = diff_path {
            options = options.generate_diff_image(path);
        }
    }

    options.validate()?;

    info!("Comparing images: {} vs {}", image1_path.display(), image2_path.display());
    
    // Perform comparison
    let result = ImageComparator::compare_files(image1_path, image2_path, &options)?;

    // Output results
    match output_format.to_lowercase().as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| webshot::WebshotError::config(format!("JSON serialization failed: {}", e)))?;
            
            if let Some(output_path) = output {
                std::fs::write(output_path, json)?;
                info!("Comparison results saved to JSON file");
            } else {
                println!("{}", json);
            }
        }
        "text" => {
            let text_output = format_comparison_result(&result);
            
            if let Some(output_path) = output {
                std::fs::write(output_path, text_output)?;
                info!("Comparison results saved to text file");
            } else {
                println!("{}", text_output);
            }
        }
        _ => return Err(webshot::WebshotError::config(format!(
            "Unknown output format: {}. Supported: json, text", output_format
        ))),
    }

    // Exit with appropriate code
    if result.similar {
        info!("Images are similar (similarity: {:.2}%)", result.similarity * 100.0);
        std::process::exit(0);
    } else {
        info!("Images are different (similarity: {:.2}%)", result.similarity * 100.0);
        std::process::exit(1);
    }
}

/// Parse RGB color string (format: "255,0,0")
fn parse_rgb_color(color_str: &str) -> Result<(u8, u8, u8)> {
    let parts: Vec<&str> = color_str.split(',').collect();
    if parts.len() != 3 {
        return Err(webshot::WebshotError::config(format!(
            "Invalid color format: {}. Expected format: R,G,B (e.g., 255,0,0)", color_str
        )));
    }

    let r = parts[0].trim().parse::<u8>()
        .map_err(|_| webshot::WebshotError::config(format!("Invalid red value: {}", parts[0])))?;
    let g = parts[1].trim().parse::<u8>()
        .map_err(|_| webshot::WebshotError::config(format!("Invalid green value: {}", parts[1])))?;
    let b = parts[2].trim().parse::<u8>()
        .map_err(|_| webshot::WebshotError::config(format!("Invalid blue value: {}", parts[2])))?;

    Ok((r, g, b))
}

/// Format comparison result as human-readable text
fn format_comparison_result(result: &webshot::ComparisonResult) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("Image Comparison Results\n"));
    output.push_str(&format!("========================\n\n"));
    
    output.push_str(&format!("Algorithm: {:?}\n", result.algorithm));
    output.push_str(&format!("Threshold: {:.2}\n", result.threshold));
    output.push_str(&format!("Similarity: {:.4} ({:.2}%)\n", result.similarity, result.similarity * 100.0));
    output.push_str(&format!("Similar: {}\n", if result.similar { "YES" } else { "NO" }));
    
    if let Some(diff_pixels) = result.different_pixels {
        output.push_str(&format!("Different pixels: {}/{} ({:.2}%)\n", 
            diff_pixels, 
            result.total_pixels,
            (diff_pixels as f64 / result.total_pixels as f64) * 100.0
        ));
    }
    
    output.push_str(&format!("Total pixels: {}\n", result.total_pixels));
    
    if let Some(diff_path) = &result.diff_image_path {
        output.push_str(&format!("Difference image: {}\n", diff_path.display()));
    }
    
    output
}
