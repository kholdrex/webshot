use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs, path::Path};
use tempfile::TempDir;

const TEST_URL: &str = "https://httpbin.org/html";

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_basic_screenshot() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("-w")
        .arg("800")
        .arg("-H")
        .arg("600");

    cmd.assert().success();
    assert!(output_path.exists());

    // Check that the file is not empty
    let metadata = fs::metadata(&output_path).unwrap();
    assert!(metadata.len() > 0);
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_pdf_generation() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test.pdf");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("pdf").arg(TEST_URL).arg("-o").arg(&output_path);

    cmd.assert().success();
    assert!(output_path.exists());

    // Check that the file is not empty and starts with PDF header
    let content = fs::read(&output_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.starts_with(b"%PDF"));
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_element_screenshot() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("element.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("-s")
        .arg("h1");

    cmd.assert().success();
    assert!(output_path.exists());
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_javascript_execution() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("js-test.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("-j")
        .arg("document.body.style.backgroundColor = 'red'");

    cmd.assert().success();
    assert!(output_path.exists());
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_text_extraction() {
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("text").arg(TEST_URL).arg("-s").arg("h1");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Herman Melville"));
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_config_processing() {
    let temp_dir = TempDir::new().unwrap();

    // Create a simple config file
    let config_content = format!(
        r#"
screenshots:
  - url: "{}"
    output: "test1.png"
    width: 800
    height: 600
  - url: "{}"
    output: "test2.png"
    width: 1200
    height: 800
"#,
        TEST_URL, TEST_URL
    );

    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("multi")
        .arg(&config_path)
        .arg("-o")
        .arg(temp_dir.path())
        .arg("-p")
        .arg("2");

    cmd.assert().success();

    assert!(temp_dir.path().join("test1.png").exists());
    assert!(temp_dir.path().join("test2.png").exists());
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_jpeg_quality() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("quality.jpg");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("-q")
        .arg("50");

    cmd.assert().success();
    assert!(output_path.exists());

    // Check that it's a JPEG file
    let content = fs::read(&output_path).unwrap();
    assert!(content.starts_with(&[0xFF, 0xD8, 0xFF])); // JPEG header
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_retina_mode() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("retina.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("--retina");

    cmd.assert().success();
    assert!(output_path.exists());
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_wait_for_element() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("wait.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("--wait-for")
        .arg("h1");

    cmd.assert().success();
    assert!(output_path.exists());
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_custom_user_agent() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("user-agent.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("https://httpbin.org/user-agent")
        .arg("-o")
        .arg(&output_path)
        .arg("--user-agent")
        .arg("WebshotBot/1.0");

    cmd.assert().success();
    assert!(output_path.exists());
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium startup before validation completes"]
async fn test_error_handling_invalid_url() {
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("not-a-valid-url");

    cmd.assert().failure();
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_error_handling_invalid_selector() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("invalid.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("-s")
        .arg("invalid-selector-that-does-not-exist");

    cmd.assert().failure();
}

#[tokio::test]
async fn test_help_output() {
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("webshot"))
        .stdout(predicate::str::contains("screenshot"))
        .stdout(predicate::str::contains("-H, --height"))
        .stdout(predicate::str::contains("-h, --help"));
}

#[tokio::test]
async fn test_screenshot_help_height_short_flag() {
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.args(["screenshot", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("-H, --height"))
        .stdout(predicate::str::contains("-h, --help"));
}

#[tokio::test]
async fn test_cli_rejects_non_web_url_before_browser_startup() {
    for args in [
        vec!["screenshot", "file:///etc/passwd"],
        vec!["pdf", "file:///etc/passwd"],
        vec!["text", "data:text/html,<h1>Test</h1>"],
    ] {
        let mut cmd = Command::cargo_bin("webshot").unwrap();
        cmd.args(args);

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Unsupported URL scheme"));
    }
}

#[test]
fn test_readme_uses_actual_height_short_flag() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = fs::read_to_string(readme_path).unwrap();

    assert!(
        !readme.contains("-h, --height"),
        "README should document -H for --height because -h is clap help"
    );
    assert!(
        readme.contains("-H, --height"),
        "README should document the actual short flag for --height"
    );
    assert!(
        readme.contains("webshot https://example.com -o screenshot.png -w 1920 -H 1080"),
        "README quick-start example should use -H for viewport height"
    );
    assert!(
        readme.contains("webshot screenshot https://example.com -o test.png -w 1920 -H 1080"),
        "README screenshot subcommand example should use -H for viewport height"
    );
}

#[test]
fn test_readme_release_flow_matches_github_actions() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(manifest_dir.join("README.md"))
        .expect("README.md must exist for release-flow doc test");
    let workflow = fs::read_to_string(manifest_dir.join(".github/workflows/rust.yml"))
        .expect(".github/workflows/rust.yml must exist for release-flow doc test");
    let workflow_yaml: serde_yaml::Value =
        serde_yaml::from_str(&workflow).expect("rust.yml must be valid YAML");

    let workflow_name = workflow_yaml["name"]
        .as_str()
        .expect("workflow should have a name");
    let triggers = &workflow_yaml["on"];
    let jobs = &workflow_yaml["jobs"];
    let test_job = &jobs["test"];
    let release_job = &jobs["release"];

    assert_eq!(workflow_name, "Rust CI");
    assert_eq!(test_job["name"].as_str(), Some("Test"));
    assert_eq!(release_job["name"].as_str(), Some("Create Release"));
    assert_eq!(release_job["needs"].as_str(), Some("test"));
    assert_eq!(
        release_job["if"].as_str(),
        Some("startsWith(github.ref, 'refs/tags/')")
    );

    for branch in ["master", "main"] {
        assert!(
            yaml_sequence_contains(&triggers["push"]["branches"], branch),
            "push trigger should include {branch}"
        );
        assert!(
            yaml_sequence_contains(&triggers["pull_request"]["branches"], branch),
            "pull_request trigger should include {branch}"
        );
    }
    assert!(
        yaml_sequence_contains(&triggers["push"]["tags"], "v*"),
        "push trigger should include v* tags"
    );

    assert!(
        job_has_run_step(test_job, "cargo test --verbose --all-features"),
        "Test job should run the documented test command"
    );
    assert!(
        job_has_run_step(test_job, "cargo build --release --verbose"),
        "Test job should build the release binary before artifact upload"
    );
    assert!(
        job_uses_action(test_job, "actions/upload-artifact@v4"),
        "Test job should upload the release binary artifact"
    );
    assert_eq!(
        find_action_step(test_job, "actions/upload-artifact@v4")["with"]["name"].as_str(),
        Some("webshot-binary")
    );
    assert_eq!(
        find_action_step(test_job, "actions/upload-artifact@v4")["with"]["path"].as_str(),
        Some("target/release/webshot")
    );
    assert_eq!(
        find_action_step(test_job, "actions/upload-artifact@v4")["with"]["retention-days"].as_i64(),
        Some(7)
    );

    assert!(
        job_has_run_step(release_job, "cargo build --release --verbose"),
        "Create Release job should rebuild the release binary after Test succeeds"
    );
    assert!(
        job_uses_action(release_job, "softprops/action-gh-release@v1"),
        "Create Release job should publish through the documented release action"
    );
    assert_eq!(
        find_action_step(release_job, "softprops/action-gh-release@v1")["with"]["files"].as_str(),
        Some("target/release/webshot")
    );
    assert_eq!(
        find_action_step(release_job, "softprops/action-gh-release@v1")["with"]
            ["generate_release_notes"]
            .as_bool(),
        Some(true)
    );

    let documented_details = [
        ("workflow name", "`Rust CI` workflow"),
        ("push branches", "pushes to `master` or `main`"),
        (
            "pull request target branches",
            "`pull_request` events targeting `master` or `main`",
        ),
        ("tag release trigger", "pushed tags matching `v*`"),
        (
            "pull request job scope",
            "Pull requests run only the `Test` job",
        ),
        (
            "release job tag scope",
            "releases are triggered only by `v*` tags",
        ),
        ("test command", "`cargo test --verbose --all-features`"),
        ("release build command", "`cargo build --release --verbose`"),
        ("release binary path", "`target/release/webshot`"),
        ("artifact name", "`webshot-binary`"),
        ("artifact retention", "for seven days"),
        (
            "test dependency",
            "If it succeeds, the `Create Release` job",
        ),
        ("release action", "`softprops/action-gh-release@v1`"),
        ("generated release notes", "`generate_release_notes: true`"),
    ];

    for (description, detail) in documented_details {
        assert!(
            readme.contains(detail),
            "README release flow should document {description}: {detail}"
        );
    }
}

fn yaml_sequence_contains(value: &serde_yaml::Value, expected: &str) -> bool {
    value
        .as_sequence()
        .map(|items| items.iter().any(|item| item.as_str() == Some(expected)))
        .unwrap_or(false)
}

fn job_steps(job: &serde_yaml::Value) -> &[serde_yaml::Value] {
    job["steps"]
        .as_sequence()
        .expect("workflow job should define steps")
}

fn job_has_run_step(job: &serde_yaml::Value, expected: &str) -> bool {
    job_steps(job)
        .iter()
        .any(|step| step["run"].as_str() == Some(expected))
}

fn job_uses_action(job: &serde_yaml::Value, action: &str) -> bool {
    job_steps(job)
        .iter()
        .any(|step| step["uses"].as_str() == Some(action))
}

fn find_action_step<'a>(job: &'a serde_yaml::Value, action: &str) -> &'a serde_yaml::Value {
    job_steps(job)
        .iter()
        .find(|step| step["uses"].as_str() == Some(action))
        .expect("workflow job should include documented action step")
}

#[tokio::test]
async fn test_version_output() {
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_verbose_logging() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("verbose.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL).arg("-o").arg(&output_path).arg("-v");

    cmd.assert().success();
    assert!(output_path.exists());
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_timeout_handling() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("timeout.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    // Test timeout by waiting for an element that doesn't exist
    cmd.arg("https://httpbin.org/html")
        .arg("-o")
        .arg(&output_path)
        .arg("--wait-for")
        .arg(".non-existent-element-that-will-never-appear")
        .arg("-t")
        .arg("2"); // 2 second timeout

    // This should timeout and fail when waiting for non-existent element
    cmd.timeout(std::time::Duration::from_secs(8))
        .assert()
        .failure();
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium startup before validation completes"]
async fn test_config_validation() {
    let temp_dir = TempDir::new().unwrap();

    // Create an invalid config file
    let config_content = r#"
screenshots:
  - url: "not-a-valid-url"
    output: "test.png"
"#;

    let config_path = temp_dir.path().join("invalid-config.yaml");
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("multi").arg(&config_path);

    cmd.assert().failure();
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_subcommand_screenshot() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("subcommand.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("screenshot")
        .arg(TEST_URL)
        .arg("-o")
        .arg(&output_path);

    cmd.assert().success();
    assert!(output_path.exists());
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_subcommand_pdf() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("subcommand.pdf");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("pdf")
        .arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("--landscape");

    cmd.assert().success();
    assert!(output_path.exists());

    let content = fs::read(&output_path).unwrap();
    assert!(content.starts_with(b"%PDF"));
}

#[tokio::test]
#[ignore = "requires Chrome/Chromium and network access"]
async fn test_parallel_processing() {
    let temp_dir = TempDir::new().unwrap();

    // Create config with multiple screenshots
    let config_content = format!(
        r#"
screenshots:
  - url: "{}"
    output: "parallel1.png"
  - url: "{}" 
    output: "parallel2.png"
  - url: "{}"
    output: "parallel3.png"
  - url: "{}"
    output: "parallel4.png"
"#,
        TEST_URL, TEST_URL, TEST_URL, TEST_URL
    );

    let config_path = temp_dir.path().join("parallel-config.yaml");
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("multi")
        .arg(&config_path)
        .arg("-o")
        .arg(temp_dir.path())
        .arg("-p")
        .arg("4");

    cmd.assert().success();

    // Check all files were created
    for i in 1..=4 {
        assert!(temp_dir.path().join(format!("parallel{}.png", i)).exists());
    }
}

// Helper function to create a test image
fn create_test_image(width: u32, height: u32, color: [u8; 3], path: &std::path::Path) {
    let img: image::RgbImage = image::ImageBuffer::from_fn(width, height, |_, _| image::Rgb(color));
    img.save(path).unwrap();
}

#[tokio::test]
async fn test_compare_identical_images() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");

    // Create two identical images
    create_test_image(100, 100, [255, 0, 0], &img1_path);
    create_test_image(100, 100, [255, 0, 0], &img2_path);

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare").arg(&img1_path).arg(&img2_path);

    // Should exit with code 0 (similar images)
    cmd.assert().code(0);
}

#[tokio::test]
async fn test_compare_different_images() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");

    // Create two different images
    create_test_image(100, 100, [255, 0, 0], &img1_path); // Red
    create_test_image(100, 100, [0, 255, 0], &img2_path); // Green

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare").arg(&img1_path).arg(&img2_path);

    // Should exit with code 1 (different images)
    cmd.assert().code(1);
}

#[tokio::test]
async fn test_compare_with_diff_image() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");
    let diff_path = temp_dir.path().join("diff.png");

    // Create two different images
    create_test_image(100, 100, [255, 0, 0], &img1_path);
    create_test_image(100, 100, [0, 255, 0], &img2_path);

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare")
        .arg(&img1_path)
        .arg(&img2_path)
        .arg("--diff-image")
        .arg("--diff-path")
        .arg(&diff_path);

    cmd.assert().code(1);

    // Check that diff image was created
    assert!(diff_path.exists());
    let metadata = fs::metadata(&diff_path).unwrap();
    assert!(metadata.len() > 0);
}

#[tokio::test]
async fn test_compare_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");
    let output_path = temp_dir.path().join("results.json");

    // Create two different images
    create_test_image(50, 50, [255, 0, 0], &img1_path);
    create_test_image(50, 50, [0, 255, 0], &img2_path);

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare")
        .arg(&img1_path)
        .arg(&img2_path)
        .arg("--format")
        .arg("json")
        .arg("-o")
        .arg(&output_path);

    cmd.assert().code(1);

    // Check that JSON output was created
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();

    // Parse and validate JSON structure
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json["similar"].is_boolean());
    assert!(json["similarity"].is_number());
    assert!(json["algorithm"].is_string());
    assert!(json["threshold"].is_number());
    assert!(json["total_pixels"].is_number());
}

#[tokio::test]
async fn test_compare_different_algorithms() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");

    // Create slightly different images
    create_test_image(50, 50, [255, 0, 0], &img1_path);
    create_test_image(50, 50, [250, 5, 5], &img2_path);

    // Test different algorithms
    let algorithms = ["pixel-diff", "ssim", "mse", "psnr"];

    for algorithm in &algorithms {
        let mut cmd = Command::cargo_bin("webshot").unwrap();
        cmd.arg("compare")
            .arg(&img1_path)
            .arg(&img2_path)
            .arg("-a")
            .arg(algorithm)
            .arg("--format")
            .arg("json");

        let assertion = cmd.assert();
        let output = assertion.get_output();
        let stdout = String::from_utf8(output.stdout.clone()).unwrap();

        if !stdout.is_empty() {
            let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
            // Algorithm names in JSON are PascalCase
            let expected_algorithm = match *algorithm {
                "pixel-diff" => "PixelDiff",
                "ssim" => "SSIM",
                "mse" => "MSE",
                "psnr" => "PSNR",
                _ => *algorithm,
            };
            assert_eq!(json["algorithm"].as_str().unwrap(), expected_algorithm);
        }
    }
}

#[tokio::test]
async fn test_compare_with_threshold() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");

    // Create more different images for the strict test
    create_test_image(50, 50, [255, 0, 0], &img1_path);
    create_test_image(50, 50, [200, 50, 50], &img2_path); // More different colors

    // Test with strict threshold (should be different)
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare")
        .arg(&img1_path)
        .arg(&img2_path)
        .arg("-t")
        .arg("0.01"); // Very strict threshold, no anti-aliasing flag

    cmd.assert().code(1); // Should be different with strict threshold

    // Test with lenient threshold using more similar images
    let similar_img1_path = temp_dir.path().join("similar1.png");
    let similar_img2_path = temp_dir.path().join("similar2.png");
    create_test_image(50, 50, [255, 0, 0], &similar_img1_path);
    create_test_image(50, 50, [253, 2, 2], &similar_img2_path); // Very similar colors

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare")
        .arg(&similar_img1_path)
        .arg(&similar_img2_path)
        .arg("-t")
        .arg("0.5") // Lenient threshold
        .arg("--ignore-antialiasing");

    cmd.assert().code(0); // Should be similar with lenient threshold
}

#[tokio::test]
async fn test_compare_custom_diff_color() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");
    let diff_path = temp_dir.path().join("diff.png");

    // Create different images
    create_test_image(50, 50, [255, 0, 0], &img1_path);
    create_test_image(50, 50, [0, 255, 0], &img2_path);

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare")
        .arg(&img1_path)
        .arg(&img2_path)
        .arg("--diff-image")
        .arg("--diff-path")
        .arg(&diff_path)
        .arg("--diff-color")
        .arg("0,0,255"); // Blue highlighting

    cmd.assert().code(1);

    // Check that diff image was created
    assert!(diff_path.exists());
}

#[tokio::test]
async fn test_compare_dimension_mismatch() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");

    // Create images with different dimensions
    create_test_image(100, 100, [255, 0, 0], &img1_path);
    create_test_image(200, 100, [255, 0, 0], &img2_path);

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare").arg(&img1_path).arg(&img2_path);

    // Should fail with error due to dimension mismatch
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("dimensions don't match"));
}

#[tokio::test]
async fn test_compare_invalid_files() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_path = temp_dir.path().join("nonexistent.png");
    let valid_path = temp_dir.path().join("valid.png");

    create_test_image(50, 50, [255, 0, 0], &valid_path);

    // Test with non-existent first image
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare").arg(&nonexistent_path).arg(&valid_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to load first image"));

    // Test with non-existent second image
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare").arg(&valid_path).arg(&nonexistent_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to load second image"));
}

#[tokio::test]
async fn test_compare_invalid_algorithm() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");

    create_test_image(50, 50, [255, 0, 0], &img1_path);
    create_test_image(50, 50, [0, 255, 0], &img2_path);

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare")
        .arg(&img1_path)
        .arg(&img2_path)
        .arg("-a")
        .arg("invalid-algorithm");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unknown algorithm"));
}

#[tokio::test]
async fn test_compare_text_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let img1_path = temp_dir.path().join("img1.png");
    let img2_path = temp_dir.path().join("img2.png");

    create_test_image(50, 50, [255, 0, 0], &img1_path);
    create_test_image(50, 50, [0, 255, 0], &img2_path);

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("compare")
        .arg(&img1_path)
        .arg(&img2_path)
        .arg("--format")
        .arg("text");

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("Image Comparison Results"))
        .stdout(predicate::str::contains("Algorithm:"))
        .stdout(predicate::str::contains("Similarity:"))
        .stdout(predicate::str::contains("Similar:"));
}
