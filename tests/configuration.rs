use std::{path::Path, process::Stdio, time::Duration};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

fn binary(directory: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mcp-gmap-rs"));
    command
        .env_clear()
        .current_dir(directory)
        .kill_on_drop(true);
    if let Some(profile) = std::env::var_os("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }
    command
}

fn tool_name(output: &std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tools: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(tools.as_array().unwrap().len(), 1);
    tools[0]["name"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn dotenv_environment_and_cli_layers_have_increasing_precedence() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join(".env"),
        "# local settings\nexport GOOGLE_MAPS_ENABLED_TOOLS='maps_geocode'\n",
    )
    .unwrap();
    assert_eq!(
        tool_name(
            &binary(directory.path())
                .arg("tools")
                .output()
                .await
                .unwrap()
        ),
        "maps_geocode"
    );
    assert_eq!(
        tool_name(
            &binary(directory.path())
                .env("GOOGLE_MAPS_ENABLED_TOOLS", "maps_timezone")
                .arg("tools")
                .output()
                .await
                .unwrap()
        ),
        "maps_timezone"
    );
    assert_eq!(
        tool_name(
            &binary(directory.path())
                .env("GOOGLE_MAPS_ENABLED_TOOLS", "maps_timezone")
                .args(["tools", "--enabled-tools", "maps_elevation"])
                .output()
                .await
                .unwrap()
        ),
        "maps_elevation"
    );
}

#[tokio::test]
async fn defaults_work_without_a_file_and_parent_dotenv_files_are_not_searched() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join(".env"), "malformed file\n").unwrap();
    let child = directory.path().join("child");
    std::fs::create_dir(&child).unwrap();
    let output = binary(&child).arg("tools").output().await.unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        18
    );
}

#[tokio::test]
async fn explicit_file_selection_and_disabling_are_respected() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("mounted.env");
    std::fs::write(&config, "GOOGLE_MAPS_ENABLED_TOOLS=maps_timezone\n").unwrap();
    std::fs::write(
        directory.path().join(".env"),
        "GOOGLE_MAPS_ENABLED_TOOLS=maps_geocode\n",
    )
    .unwrap();
    assert_eq!(
        tool_name(
            &binary(directory.path())
                .env("MCP_ENV_FILE", &config)
                .arg("tools")
                .output()
                .await
                .unwrap()
        ),
        "maps_timezone"
    );
    assert_eq!(
        tool_name(
            &binary(directory.path())
                .env("MCP_ENV_FILE", "missing.env")
                .args(["--env-file", "mounted.env", "tools"])
                .output()
                .await
                .unwrap()
        ),
        "maps_timezone"
    );
    let output = binary(directory.path())
        .env("MCP_ENV_FILE", "missing.env")
        .args(["--no-env-file", "tools"])
        .output()
        .await
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        18
    );
    let missing = binary(directory.path())
        .args(["--env-file", "missing.env", "tools"])
        .output()
        .await
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("cannot read dotenv file"));
}

#[tokio::test]
async fn malformed_dotenv_fails_without_echoing_credentials_and_can_be_disabled() {
    let directory = tempfile::tempdir().unwrap();
    let secret = "unique-secret-must-not-be-printed";
    std::fs::write(
        directory.path().join(".env"),
        format!("GOOGLE_MAPS_API_KEY=\"{secret}\n"),
    )
    .unwrap();
    let output = binary(directory.path())
        .arg("tools")
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("cannot parse dotenv file"));
    assert!(!error.contains(secret));
    assert!(
        binary(directory.path())
            .args(["--no-env-file", "tools"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
}

#[tokio::test]
async fn timeout_validation_applies_to_the_effective_value_after_overrides() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join(".env"), "GOOGLE_MAPS_TIMEOUT=0\n").unwrap();
    let invalid = binary(directory.path())
        .arg("tools")
        .output()
        .await
        .unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("GOOGLE_MAPS_TIMEOUT"));
    assert!(
        binary(directory.path())
            .env("GOOGLE_MAPS_TIMEOUT", "5")
            .arg("tools")
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    assert!(
        !binary(directory.path())
            .env("GOOGLE_MAPS_TIMEOUT", "not-a-number")
            .arg("tools")
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    assert!(
        binary(directory.path())
            .env("GOOGLE_MAPS_TIMEOUT", "not-a-number")
            .args(["tools", "--timeout", "10"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
}

#[tokio::test]
async fn empty_environment_token_clears_the_dotenv_token() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join(".env"),
        "MCP_HTTP_TOKEN=file-token\nMCP_HTTP_LISTEN=0.0.0.0:0\n",
    )
    .unwrap();
    let output = binary(directory.path())
        .env("MCP_HTTP_TOKEN", "")
        .args(["serve", "http"])
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("required when binding to a non-loopback")
    );
}

#[tokio::test]
async fn whitespace_dotenv_tokens_are_rejected_without_echoing_them() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join(".env"), "MCP_HTTP_TOKEN='   '\n").unwrap();
    let output = binary(directory.path())
        .args(["serve", "http"])
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MCP_HTTP_TOKEN cannot be whitespace")
    );
}

#[tokio::test]
async fn http_settings_authentication_and_allowlists_follow_the_same_layers() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join(".env"), "MCP_HTTP_LISTEN=invalid\nMCP_HTTP_TOKEN=file-token\nMCP_HTTP_ALLOWED_HOSTS=file.example\nMCP_HTTP_ALLOWED_ORIGINS=https://file.example:443\n").unwrap();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    for cli in [false, true] {
        let mut command = binary(directory.path());
        command
            .args(["serve", "http"])
            .env("MCP_HTTP_LISTEN", "127.0.0.1:0")
            .env("MCP_HTTP_TOKEN", "env-token")
            .env("MCP_HTTP_ALLOWED_HOSTS", " env.example , second.example ")
            .env("MCP_HTTP_ALLOWED_ORIGINS", "https://env.example:443");
        if cli {
            command.env("MCP_HTTP_LISTEN", "invalid").args([
                "--listen",
                "127.0.0.1:0",
                "--allowed-host",
                "cli.example",
                "--allowed-host",
                "second.example",
                "--allowed-origin",
                "https://cli.example:443",
            ]);
        }
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
        let line = tokio::time::timeout(Duration::from_secs(10), stderr.next_line())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let url = line
            .strip_prefix("MCP Streamable HTTP listening on ")
            .unwrap();
        assert_eq!(
            client
                .get(url.replace("/mcp", "/health"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"configuration-test","version":"1"}}});
        let (host, origin) = if cli {
            ("cli.example", "https://cli.example:443")
        } else {
            ("env.example", "https://env.example:443")
        };
        let request = || {
            client
                .post(url)
                .header("accept", "application/json, text/event-stream")
                .header("host", host)
                .header("origin", origin)
                .json(&initialize)
        };
        assert_eq!(
            request()
                .bearer_auth("file-token")
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
        assert_eq!(
            request()
                .bearer_auth("env-token")
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        let replaced = client
            .post(url)
            .bearer_auth("env-token")
            .header("accept", "application/json, text/event-stream")
            .header("host", if cli { "env.example" } else { "file.example" })
            .json(&initialize)
            .send()
            .await
            .unwrap();
        assert!(!replaced.status().is_success());
        child.kill().await.unwrap();
    }
}

#[tokio::test]
async fn authentication_mode_is_validated_and_can_be_overridden() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join(".env"),
        "GOOGLE_MAPS_AUTH_MODE=unknown\n",
    )
    .unwrap();
    let invalid = binary(directory.path())
        .arg("tools")
        .output()
        .await
        .unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("GOOGLE_MAPS_AUTH_MODE"));
    assert!(
        binary(directory.path())
            .env("GOOGLE_MAPS_AUTH_MODE", "oauth")
            .arg("tools")
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    assert!(
        binary(directory.path())
            .env("GOOGLE_MAPS_AUTH_MODE", "unknown")
            .args(["tools", "--auth-mode", "api-key"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
}

#[tokio::test]
async fn dotenv_service_account_files_load_lazily_and_errors_do_not_echo_private_material() {
    let directory = tempfile::tempdir().unwrap();
    let sentinel = "private-material-must-never-be-printed";
    std::fs::write(directory.path().join("account.json"), serde_json::to_vec(&json!({"type":"service_account","project_id":"fixture-project","client_email":"fixture@example.iam.gserviceaccount.com","private_key":sentinel,"private_key_id":"fixture"})).unwrap()).unwrap();
    std::fs::write(directory.path().join(".env"), "GOOGLE_MAPS_AUTH_MODE=oauth\nGOOGLE_APPLICATION_CREDENTIALS=account.json\nGOOGLE_CLOUD_QUOTA_PROJECT=fixture-project\nGOOGLE_MAPS_TIMEOUT=1\n").unwrap();
    assert!(
        binary(directory.path())
            .arg("tools")
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    let invalid = binary(directory.path())
        .args(["search-places", "--query", "cafe"])
        .output()
        .await
        .unwrap();
    assert!(!invalid.status.success());
    assert!(
        String::from_utf8_lossy(&invalid.stderr).contains("Google OAuth authentication failed")
    );
    assert!(!String::from_utf8_lossy(&invalid.stderr).contains(sentinel));
    assert!(invalid.stdout.is_empty());
}
