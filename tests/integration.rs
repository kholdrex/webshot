use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

const TEST_URL: &str = "https://httpbin.org/html";

#[tokio::test]
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
async fn test_pdf_generation() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test.pdf");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("pdf")
        .arg(TEST_URL)
        .arg("-o")
        .arg(&output_path);

    cmd.assert().success();
    assert!(output_path.exists());
    
    // Check that the file is not empty and starts with PDF header
    let content = fs::read(&output_path).unwrap();
    assert!(content.len() > 0);
    assert!(content.starts_with(b"%PDF"));
}

#[tokio::test]
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
async fn test_text_extraction() {
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("text")
        .arg(TEST_URL)
        .arg("-s")
        .arg("h1");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Herman Melville"));
}

#[tokio::test]
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
async fn test_error_handling_invalid_url() {
    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg("not-a-valid-url");

    cmd.assert().failure();
}

#[tokio::test]
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
        .stdout(predicate::str::contains("screenshot"));
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
async fn test_verbose_logging() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("verbose.png");

    let mut cmd = Command::cargo_bin("webshot").unwrap();
    cmd.arg(TEST_URL)
        .arg("-o")
        .arg(&output_path)
        .arg("-v");

    cmd.assert().success();
    assert!(output_path.exists());
}

#[tokio::test]
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
        assert!(temp_dir.path().join(&format!("parallel{}.png", i)).exists());
    }
}


