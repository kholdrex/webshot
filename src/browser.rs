use crate::config::{validate_navigation_url, Config, ScreenshotConfig};
use crate::error::{Result, WebshotError};
use crate::output::OutputHandler;
use crate::screenshot::{ImageFormat, ScreenshotOptions, ScrollMode};
use headless_chrome::protocol::cdp::Page;
use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::{Browser as ChromeBrowser, LaunchOptions, Tab};
use image::{DynamicImage, ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Page metrics for determining scroll requirements
#[derive(Debug, Clone, Deserialize, Serialize)]
struct PageMetrics {
    content_height: f64,
    viewport_height: f64,
    viewport_width: f64,
}

/// Browser automation wrapper
pub struct Browser {
    browser: ChromeBrowser,
    javascript_enabled: bool,
}

impl Browser {
    /// Create a new browser instance
    pub async fn new(
        chrome_path: Option<PathBuf>,
        chrome_flags: Vec<String>,
        javascript_enabled: bool,
    ) -> Result<Self> {
        info!("Launching browser...");

        let mut args_str = vec![
            "--no-sandbox",
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--disable-setuid-sandbox",
            "--no-first-run",
        ];

        // Collect additional flags
        let mut flag_strings = Vec::new();
        for flag in chrome_flags {
            flag_strings.push(flag);
        }

        // Disable JavaScript if requested
        if !javascript_enabled {
            flag_strings.push("--disable-javascript".to_string());
        }

        // Convert to OsStr refs
        for flag in &flag_strings {
            args_str.push(flag.as_str());
        }

        let args_os: Vec<std::ffi::OsString> = args_str.iter().map(|s| (*s).into()).collect();
        let args_refs: Vec<&std::ffi::OsStr> = args_os.iter().map(|s| s.as_os_str()).collect();

        let launch_options = if let Some(path) = chrome_path {
            LaunchOptions::default_builder()
                .headless(true)
                .sandbox(false)
                .args(args_refs)
                .path(Some(path))
                .build()
                .unwrap()
        } else {
            LaunchOptions::default_builder()
                .headless(true)
                .sandbox(false)
                .args(args_refs)
                .build()
                .unwrap()
        };

        let browser = ChromeBrowser::new(launch_options)
            .map_err(|e| WebshotError::browser_launch(e.to_string()))?;

        debug!("Browser launched successfully");

        Ok(Self {
            browser,
            javascript_enabled,
        })
    }

    /// Take a screenshot of a webpage
    pub async fn screenshot<P: AsRef<Path>>(
        &self,
        url: &str,
        output_path: P,
        options: &ScreenshotOptions,
    ) -> Result<()> {
        validate_navigation_url(url, "screenshot API")?;
        options.validate()?;

        let tab = self
            .browser
            .new_tab()
            .map_err(|e| WebshotError::Tab(e.to_string()))?;
        self.setup_tab(&tab, options).await?;

        info!("Navigating to: {}", url);
        tab.navigate_to(url)
            .map_err(|e| WebshotError::navigation(e.to_string()))?;

        // Wait for page load
        tab.wait_until_navigated()
            .map_err(|e| WebshotError::navigation(e.to_string()))?;

        // Execute custom JavaScript if provided
        if let Some(script) = &options.javascript {
            if self.javascript_enabled {
                info!("Executing JavaScript: {}", script);
                tab.evaluate(script, false)
                    .map_err(|e| WebshotError::javascript(e.to_string()))?;
            } else {
                warn!("JavaScript disabled, skipping script execution");
            }
        }

        // Wait for specific element if requested
        if let Some(selector) = &options.wait_for {
            info!("Waiting for element: {}", selector);
            self.wait_for_element(&tab, selector, options.timeout)
                .await?;
        }

        // Additional wait time
        if options.wait > 0 {
            info!("Waiting {} seconds before screenshot", options.wait);
            sleep(Duration::from_secs(options.wait)).await;
        }

        let format = options.output_format(&output_path)?;

        match format {
            ImageFormat::Pdf => {
                return Err(WebshotError::screenshot(
                    "PDF generation not supported in screenshot method, use pdf() method instead",
                ));
            }
            ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP => {
                self.take_image_screenshot(&tab, &output_path, options, format)
                    .await?;
            }
        }

        info!("Screenshot saved to: {}", output_path.as_ref().display());
        Ok(())
    }

    /// Generate a PDF from a webpage
    #[allow(clippy::too_many_arguments)]
    pub async fn pdf<P: AsRef<Path>>(
        &self,
        url: &str,
        output_path: P,
        _format: &str,
        landscape: bool,
        background: bool,
        scale: f64,
        javascript: Option<String>,
        wait_for: Option<String>,
        timeout: u64,
        user_agent: Option<String>,
    ) -> Result<()> {
        validate_navigation_url(url, "pdf API")?;
        let tab = self
            .browser
            .new_tab()
            .map_err(|e| WebshotError::Tab(e.to_string()))?;

        // Set up the tab
        if let Some(user_agent) = user_agent {
            tab.set_user_agent(&user_agent, None, None)
                .map_err(WebshotError::Browser)?;
        }

        info!("Navigating to: {}", url);
        tab.navigate_to(url)
            .map_err(|e| WebshotError::navigation(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| WebshotError::navigation(e.to_string()))?;

        // Execute custom JavaScript if provided
        if let Some(script) = &javascript {
            if self.javascript_enabled {
                info!("Executing JavaScript: {}", script);
                tab.evaluate(script, false)
                    .map_err(|e| WebshotError::javascript(e.to_string()))?;
            } else {
                warn!("JavaScript disabled, skipping script execution");
            }
        }

        // Wait for specific element if requested
        if let Some(selector) = &wait_for {
            info!("Waiting for element: {}", selector);
            self.wait_for_element(&tab, selector, timeout).await?;
        }

        info!("Generating PDF...");

        let pdf_options = PrintToPdfOptions {
            landscape: Some(landscape),
            display_header_footer: Some(false),
            print_background: Some(background),
            scale: Some(scale),
            paper_width: None,
            paper_height: None,
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            page_ranges: None,
            ignore_invalid_page_ranges: None,
            header_template: None,
            footer_template: None,
            prefer_css_page_size: Some(true),
            transfer_mode: None,
            generate_document_outline: Some(false),
            generate_tagged_pdf: Some(false),
        };

        let pdf_data = tab
            .print_to_pdf(Some(pdf_options))
            .map_err(|e| WebshotError::pdf(e.to_string()))?;
        OutputHandler::ensure_output_dir(&output_path)?;
        std::fs::write(&output_path, pdf_data)?;

        info!("PDF saved to: {}", output_path.as_ref().display());
        Ok(())
    }

    /// Extract text content from a webpage
    pub async fn extract_text(
        &self,
        url: &str,
        selector: Option<String>,
        javascript: Option<String>,
        wait_for: Option<String>,
        timeout: u64,
        user_agent: Option<String>,
    ) -> Result<String> {
        validate_navigation_url(url, "text API")?;
        let tab = self
            .browser
            .new_tab()
            .map_err(|e| WebshotError::Tab(e.to_string()))?;

        // Set up the tab
        if let Some(user_agent) = user_agent {
            tab.set_user_agent(&user_agent, None, None)
                .map_err(WebshotError::Browser)?;
        }

        info!("Navigating to: {}", url);
        tab.navigate_to(url)
            .map_err(|e| WebshotError::navigation(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| WebshotError::navigation(e.to_string()))?;

        // Execute custom JavaScript if provided
        if let Some(script) = &javascript {
            if self.javascript_enabled {
                info!("Executing JavaScript: {}", script);
                tab.evaluate(script, false)
                    .map_err(|e| WebshotError::javascript(e.to_string()))?;
            } else {
                warn!("JavaScript disabled, skipping script execution");
            }
        }

        // Wait for specific element if requested
        if let Some(selector_str) = &wait_for {
            info!("Waiting for element: {}", selector_str);
            self.wait_for_element(&tab, selector_str, timeout).await?;
        }

        let text = if let Some(selector_str) = selector {
            info!("Extracting text from element: {}", selector_str);
            let element = tab
                .find_element(&selector_str)
                .map_err(|_e| WebshotError::element_not_found(selector_str))?;
            element.get_inner_text().map_err(WebshotError::Browser)?
        } else {
            info!("Extracting text from entire page");
            tab.get_content().map_err(WebshotError::Browser)?
        };

        Ok(text)
    }

    /// Process multiple screenshots from configuration
    pub async fn process_config(
        &self,
        config: &Config,
        output_dir: Option<PathBuf>,
        parallel: usize,
    ) -> Result<()> {
        config.validate()?;

        info!(
            "Processing {} screenshots with {} parallel tasks",
            config.screenshots.len(),
            parallel
        );

        use futures::stream::{self, StreamExt};

        let semaphore = Arc::new(tokio::sync::Semaphore::new(parallel));

        let tasks = config.screenshots.iter().map(|screenshot_config| {
            let semaphore = semaphore.clone();
            let screenshot_config = screenshot_config.clone();
            let output_dir = output_dir.clone();

            async move {
                let _permit = semaphore.acquire().await.unwrap();
                self.process_single_screenshot(screenshot_config, output_dir)
                    .await
            }
        });

        let results: Vec<Result<()>> = stream::iter(tasks)
            .buffer_unordered(parallel)
            .collect()
            .await;

        // Check for errors
        for (i, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                warn!("Screenshot {} failed: {}", i, e);
            }
        }

        Ok(())
    }

    async fn setup_tab(&self, tab: &Tab, options: &ScreenshotOptions) -> Result<()> {
        // Set viewport using emulation
        tab.set_default_timeout(std::time::Duration::from_secs(options.timeout));

        tab.call_method(
            headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride {
                width: options.width,
                height: options.height,
                device_scale_factor: options.device_scale_factor(),
                mobile: false,
                scale: None,
                screen_width: None,
                screen_height: None,
                position_x: None,
                position_y: None,
                dont_set_visible_size: None,
                screen_orientation: None,
                viewport: None,
                display_feature: None,
                device_posture: None,
            },
        )
        .map_err(WebshotError::Browser)?;

        // Set user agent if provided
        if let Some(user_agent) = &options.user_agent {
            tab.set_user_agent(user_agent, None, None)
                .map_err(WebshotError::Browser)?;
        }

        Ok(())
    }

    async fn wait_for_element(&self, tab: &Tab, selector: &str, timeout: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(timeout);

        loop {
            if start.elapsed() > timeout_duration {
                return Err(WebshotError::timeout(format!(
                    "waiting for element: {}",
                    selector
                )));
            }

            if tab.find_element(selector).is_ok() {
                debug!("Element found: {}", selector);
                return Ok(());
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    async fn take_image_screenshot<P: AsRef<Path>>(
        &self,
        tab: &Tab,
        output_path: P,
        options: &ScreenshotOptions,
        format: ImageFormat,
    ) -> Result<()> {
        let screenshot_data = match options.scroll_mode {
            ScrollMode::Viewport => {
                if let Some(selector) = &options.selector {
                    info!("Taking element screenshot: {}", selector);
                    let element =
                        tab.find_element(selector)
                            .map_err(|_e| WebshotError::ElementNotFound {
                                selector: selector.clone(),
                            })?;
                    element
                        .capture_screenshot(Page::CaptureScreenshotFormatOption::Png)
                        .map_err(|e| WebshotError::screenshot(e.to_string()))?
                } else {
                    info!("Taking viewport screenshot");
                    tab.capture_screenshot(
                        Page::CaptureScreenshotFormatOption::Png,
                        None,
                        None,
                        true, // from_surface: required for reliable headless capture
                    )
                    .map_err(|e| WebshotError::screenshot(e.to_string()))?
                }
            }
            ScrollMode::FullPage => {
                info!("Taking full page scrolling screenshot");
                self.capture_full_page_screenshot(tab, options).await?
            }
            ScrollMode::FullElement => {
                if let Some(selector) = &options.selector {
                    info!("Taking full element scrolling screenshot: {}", selector);
                    self.capture_full_element_screenshot(tab, selector, options)
                        .await?
                } else {
                    return Err(WebshotError::config(
                        "FullElement scroll mode requires a selector".to_string(),
                    ));
                }
            }
        };

        self.save_screenshot_data(&screenshot_data, &output_path, options, format)
            .await
    }

    async fn capture_full_page_screenshot(
        &self,
        tab: &Tab,
        options: &ScreenshotOptions,
    ) -> Result<Vec<u8>> {
        // Get page dimensions
        let page_metrics = self.get_page_metrics(tab).await?;
        let content_height = page_metrics.content_height as u32;
        let viewport_height = options.height;

        // Apply max height limit
        let effective_height = if let Some(max_height) = options.max_height {
            content_height.min(max_height)
        } else {
            content_height
        };

        info!(
            "Page content height: {}px, capturing up to: {}px",
            content_height, effective_height
        );

        if effective_height <= viewport_height {
            // Page fits in viewport, take single screenshot
            return tab
                .capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
                .map_err(|e| WebshotError::screenshot(e.to_string()));
        }

        // Calculate number of screenshots needed
        let num_screenshots = effective_height.div_ceil(viewport_height);
        let mut screenshots = Vec::new();

        // Reset scroll position
        tab.evaluate("window.scrollTo(0, 0)", true)
            .map_err(|e| WebshotError::javascript(e.to_string()))?;

        sleep(Duration::from_millis(options.scroll_delay)).await;

        for i in 0..num_screenshots {
            let scroll_y = i * viewport_height;

            // Scroll to position
            if i > 0 {
                tab.evaluate(&format!("window.scrollTo(0, {})", scroll_y), true)
                    .map_err(|e| WebshotError::javascript(e.to_string()))?;

                sleep(Duration::from_millis(options.scroll_delay)).await;
            }

            info!(
                "Capturing screenshot {}/{} at scroll position {}px",
                i + 1,
                num_screenshots,
                scroll_y
            );

            let screenshot_data = tab
                .capture_screenshot(
                    Page::CaptureScreenshotFormatOption::Png,
                    None,
                    None,
                    true, // from_surface: required for reliable headless capture
                )
                .map_err(|e| WebshotError::screenshot(e.to_string()))?;

            let img = image::load_from_memory(&screenshot_data)?;
            screenshots.push(img);
        }

        // Stitch screenshots together
        let stitched_image =
            self.stitch_screenshots(screenshots, options.width, effective_height)?;

        // Convert back to PNG bytes
        let mut png_data = Vec::new();
        stitched_image.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png,
        )?;

        Ok(png_data)
    }

    async fn capture_full_element_screenshot(
        &self,
        tab: &Tab,
        selector: &str,
        options: &ScreenshotOptions,
    ) -> Result<Vec<u8>> {
        // Find the element
        let element = tab
            .find_element(selector)
            .map_err(|_e| WebshotError::ElementNotFound {
                selector: selector.to_string(),
            })?;

        // Get element dimensions and position
        let element_info = tab
            .evaluate(
                &format!(
                    r#"
            (() => {{
                const el = document.querySelector('{}');
                if (!el) return "null";
                const rect = el.getBoundingClientRect();
                return JSON.stringify({{
                    x: rect.left + window.scrollX,
                    y: rect.top + window.scrollY,
                    width: el.scrollWidth || rect.width,
                    height: el.scrollHeight || rect.height,
                    viewportHeight: window.innerHeight
                }});
            }})()
            "#,
                    selector.replace('\'', "\\'")
                ),
                true,
            )
            .map_err(|e| WebshotError::javascript(e.to_string()))?;

        let element_json = element_info
            .value
            .as_ref()
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                WebshotError::javascript("element info evaluation returned no value".to_string())
            })?;
        let element_data: serde_json::Value = serde_json::from_str(element_json).map_err(|e| {
            WebshotError::javascript(format!("Failed to parse element info: {}", e))
        })?;

        if element_data.is_null() {
            return Err(WebshotError::ElementNotFound {
                selector: selector.to_string(),
            });
        }

        let element_height = element_data["height"].as_f64().unwrap() as u32;
        let element_y = element_data["y"].as_f64().unwrap() as u32;
        let viewport_height = options.height;

        // Apply max height limit
        let effective_height = if let Some(max_height) = options.max_height {
            element_height.min(max_height)
        } else {
            element_height
        };

        if effective_height <= viewport_height {
            // Element fits in viewport, use regular element screenshot
            return element
                .capture_screenshot(Page::CaptureScreenshotFormatOption::Png)
                .map_err(|e| WebshotError::screenshot(e.to_string()));
        }

        // Scroll to element first
        tab.evaluate(&format!("window.scrollTo(0, {})", element_y), true)
            .map_err(|e| WebshotError::javascript(e.to_string()))?;

        sleep(Duration::from_millis(options.scroll_delay)).await;

        // Calculate number of screenshots needed for the element
        let num_screenshots = effective_height.div_ceil(viewport_height);
        let mut screenshots = Vec::new();

        for i in 0..num_screenshots {
            let scroll_y = element_y + (i * viewport_height);

            if i > 0 {
                tab.evaluate(&format!("window.scrollTo(0, {})", scroll_y), true)
                    .map_err(|e| WebshotError::javascript(e.to_string()))?;

                sleep(Duration::from_millis(options.scroll_delay)).await;
            }

            info!(
                "Capturing element screenshot {}/{} at scroll position {}px",
                i + 1,
                num_screenshots,
                scroll_y
            );

            let screenshot_data = tab
                .capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
                .map_err(|e| WebshotError::screenshot(e.to_string()))?;

            let img = image::load_from_memory(&screenshot_data)?;
            screenshots.push(img);
        }

        // Stitch screenshots together
        let element_width = element_data["width"].as_f64().unwrap() as u32;
        let stitched_image =
            self.stitch_screenshots(screenshots, element_width, effective_height)?;

        // Convert back to PNG bytes
        let mut png_data = Vec::new();
        stitched_image.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png,
        )?;

        Ok(png_data)
    }

    fn stitch_screenshots(
        &self,
        screenshots: Vec<DynamicImage>,
        width: u32,
        total_height: u32,
    ) -> Result<DynamicImage> {
        if screenshots.is_empty() {
            return Err(WebshotError::screenshot(
                "No screenshots to stitch".to_string(),
            ));
        }

        let mut stitched = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, total_height);
        let mut current_y = 0u32;

        for (i, screenshot) in screenshots.iter().enumerate() {
            let screenshot_rgba = screenshot.to_rgba8();
            let screenshot_height = screenshot_rgba.height();

            // Calculate how much of this screenshot to use
            let remaining_height = total_height - current_y;
            let copy_height = screenshot_height.min(remaining_height);

            // Copy pixels from this screenshot
            for y in 0..copy_height {
                for x in 0..width {
                    if let Some(pixel) = screenshot_rgba.get_pixel_checked(x, y) {
                        stitched.put_pixel(x, current_y + y, *pixel);
                    }
                }
            }

            current_y += copy_height;
            info!(
                "Stitched screenshot {}/{}, current height: {}px",
                i + 1,
                screenshots.len(),
                current_y
            );

            if current_y >= total_height {
                break;
            }
        }

        Ok(DynamicImage::ImageRgba8(stitched))
    }

    async fn get_page_metrics(&self, tab: &Tab) -> Result<PageMetrics> {
        let metrics_result = tab
            .evaluate(
                r#"
            (() => {
                const body = document.body;
                const html = document.documentElement;

                const height = Math.max(
                    body.scrollHeight, body.offsetHeight,
                    html.clientHeight, html.scrollHeight, html.offsetHeight
                );

                return JSON.stringify({
                    content_height: height,
                    viewport_height: window.innerHeight,
                    viewport_width: window.innerWidth
                });
            })()
            "#,
                true,
            )
            .map_err(|e| WebshotError::javascript(e.to_string()))?;

        let metrics_json = metrics_result
            .value
            .as_ref()
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                WebshotError::javascript("page metrics evaluation returned no value".to_string())
            })?;
        let metrics: PageMetrics = serde_json::from_str(metrics_json).map_err(|e| {
            WebshotError::javascript(format!("Failed to parse page metrics: {}", e))
        })?;

        Ok(metrics)
    }

    async fn save_screenshot_data<P: AsRef<Path>>(
        &self,
        screenshot_data: &[u8],
        output_path: P,
        options: &ScreenshotOptions,
        format: ImageFormat,
    ) -> Result<()> {
        OutputHandler::ensure_output_dir(&output_path)?;

        match format {
            ImageFormat::Png => {
                std::fs::write(&output_path, screenshot_data)?;
            }
            ImageFormat::Jpeg => {
                // Convert PNG to JPEG
                let img = image::load_from_memory(screenshot_data)?;
                let mut output = std::fs::File::create(&output_path)?;
                let quality = options.quality.unwrap_or(90);

                let encoder =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);
                img.write_with_encoder(encoder)?;
            }
            ImageFormat::WebP => {
                // Convert PNG to WebP
                let img = image::load_from_memory(screenshot_data)?;
                let mut output = std::fs::File::create(&output_path)?;

                let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
                img.write_with_encoder(encoder)?;
            }
            ImageFormat::Pdf => {
                return Err(WebshotError::screenshot(
                    "PDF format should be handled by pdf() method",
                ));
            }
        }

        Ok(())
    }

    async fn process_single_screenshot(
        &self,
        config: ScreenshotConfig,
        output_dir: Option<PathBuf>,
    ) -> Result<()> {
        validate_navigation_url(&config.url, "batch screenshot API")?;
        let tab = self
            .browser
            .new_tab()
            .map_err(|e| WebshotError::Tab(e.to_string()))?;

        // Determine output path
        let output_path = if let Some(dir) = output_dir {
            dir.join(&config.output)
        } else {
            config.output.clone()
        };

        OutputHandler::ensure_output_dir(&output_path)?;

        let options = ScreenshotOptions {
            width: config.width,
            height: config.height,
            selector: config.selector.clone(),
            javascript: config.javascript.clone(),
            wait_for: config.wait_for.clone(),
            timeout: config.timeout,
            retina: config.retina,
            quality: config.quality,
            wait: config.wait,
            user_agent: config.user_agent.clone(),
            scroll_mode: config.scroll_mode,
            max_height: config.max_height,
            scroll_delay: config.scroll_delay,
        };

        self.setup_tab(&tab, &options).await?;

        info!("Processing: {} -> {}", config.url, output_path.display());

        // Set cookies if any
        for cookie in &config.cookies {
            let cookie_param = headless_chrome::protocol::cdp::Network::CookieParam {
                name: cookie.name.clone(),
                value: cookie.value.clone(),
                url: None,
                domain: cookie.domain.clone(),
                path: cookie.path.clone(),
                secure: cookie.secure,
                http_only: cookie.http_only,
                same_site: None,
                expires: None,
                priority: None,
                same_party: None,
                source_scheme: None,
                source_port: None,
                partition_key: None,
            };
            tab.set_cookies(vec![cookie_param])
                .map_err(WebshotError::Browser)?;
        }

        // Set custom headers
        if !config.headers.is_empty() {
            let headers: std::collections::HashMap<&str, &str> = config
                .headers
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            tab.set_extra_http_headers(headers)
                .map_err(WebshotError::Browser)?;
        }

        // Handle authentication
        if let Some(auth) = &config.auth {
            tab.authenticate(Some(auth.username.clone()), Some(auth.password.clone()))
                .map_err(WebshotError::Browser)?;
        }

        // Navigate and process
        tab.navigate_to(&config.url)
            .map_err(|e| WebshotError::navigation(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| WebshotError::navigation(e.to_string()))?;

        // Execute JavaScript
        if let Some(script) = &config.javascript {
            if self.javascript_enabled {
                tab.evaluate(script, false)
                    .map_err(|e| WebshotError::javascript(e.to_string()))?;
            }
        }

        // Wait for element
        if let Some(selector) = &config.wait_for {
            self.wait_for_element(&tab, selector, config.timeout)
                .await?;
        }

        // Wait before screenshot
        if config.wait > 0 {
            sleep(Duration::from_secs(config.wait)).await;
        }

        // Take screenshot
        let format = options.output_format(&output_path)?;
        match format {
            ImageFormat::Pdf => {
                let pdf_options = PrintToPdfOptions {
                    landscape: Some(false),
                    display_header_footer: Some(false),
                    print_background: Some(true),
                    scale: Some(1.0),
                    paper_width: None,
                    paper_height: None,
                    margin_top: None,
                    margin_bottom: None,
                    margin_left: None,
                    margin_right: None,
                    page_ranges: None,
                    ignore_invalid_page_ranges: None,
                    header_template: None,
                    footer_template: None,
                    prefer_css_page_size: Some(true),
                    transfer_mode: None,
                    generate_document_outline: Some(false),
                    generate_tagged_pdf: Some(false),
                };

                let pdf_data = tab
                    .print_to_pdf(Some(pdf_options))
                    .map_err(|e| WebshotError::pdf(e.to_string()))?;
                std::fs::write(&output_path, pdf_data)?;
            }
            _ => {
                self.take_image_screenshot(&tab, &output_path, &options, format)
                    .await?;
            }
        }

        Ok(())
    }
}
