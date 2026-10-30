use std::{process::Stdio, time::Duration};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

fn binary() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mcp-gmap-rs"));
    for name in [
        "GOOGLE_MAPS_API_KEY",
        "GOOGLE_MAPS_ENABLED_TOOLS",
        "GOOGLE_MAPS_TIMEOUT",
        "GOOGLE_MAPS_LANGUAGE",
        "MCP_HTTP_TOKEN",
        "RUST_LOG",
    ] {
        command.env_remove(name);
    }
    command.kill_on_drop(true);
    command
}

#[tokio::test]
async fn http_executable_starts_on_an_ephemeral_port() {
    let mut child = binary()
        .args(["serve", "http", "--listen", "127.0.0.1:0"])
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
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let health = http
        .get(url.replace("/mcp", "/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(health.status(), 200);
    assert_eq!(health.text().await.unwrap(), "ok");
    let response = http.post(url).header("accept", "application/json, text/event-stream")
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"binary-http-test","version":"1"}}}))
        .send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert!(response.headers().contains_key("mcp-session-id"));
    child.kill().await.unwrap();
}

#[tokio::test]
async fn cli_discovers_tools_without_a_key_and_reports_failures() {
    let tools = binary().arg("tools").output().await.unwrap();
    assert!(
        tools.status.success(),
        "{}",
        String::from_utf8_lossy(&tools.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&tools.stdout)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        18
    );
    let filtered = binary()
        .args(["--enabled-tools", "maps_geocode", "tools"])
        .output()
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&filtered.stdout)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let schema = binary()
        .args(["schema", "search-nearby"])
        .output()
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&schema.stdout).unwrap()["properties"]["center"]["type"],
        "object"
    );
    let error = binary()
        .args(["geocode", "--address", "Paris"])
        .output()
        .await
        .unwrap();
    assert!(!error.status.success());
    assert!(error.stdout.is_empty());
    assert!(String::from_utf8_lossy(&error.stderr).contains("GOOGLE_MAPS_API_KEY"));
    let invalid = binary()
        .args(["call", "maps_geocode", "--args", "{} word"])
        .output()
        .await
        .unwrap();
    assert!(!invalid.status.success());
}

#[tokio::test]
async fn cli_json_stdin_and_non_loopback_guard_work() {
    let mut child = binary()
        .args(["call", "geocode", "--args", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"address\":\"Paris\"}")
        .await
        .unwrap();
    let result = child.wait_with_output().await.unwrap();
    assert!(String::from_utf8_lossy(&result.stderr).contains("GOOGLE_MAPS_API_KEY"));
    let remote = binary()
        .args(["serve", "http", "--listen", "0.0.0.0:0"])
        .output()
        .await
        .unwrap();
    assert!(!remote.status.success());
    assert!(String::from_utf8_lossy(&remote.stderr).contains("MCP_HTTP_TOKEN"));
}

#[tokio::test]
async fn stdio_executable_initializes_lists_tools_and_returns_tool_errors() {
    // No args is the documented default stdio mode.
    let mut child = binary()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    stdin.write_all(format!("{}\n",json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"executable-test","version":"1"}}})).as_bytes()).await.unwrap();
    let line = tokio::time::timeout(Duration::from_secs(10), lines.next_line())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let init: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], "mcp-gmap-rs");
    stdin
        .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
        .await
        .unwrap();
    for (id, method, params) in [
        (2, "tools/list", json!({})),
        (
            3,
            "tools/call",
            json!({"name":"maps_geocode","arguments":{"address":"Paris"}}),
        ),
    ] {
        stdin
            .write_all(
                format!(
                    "{}\n",
                    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        let response = loop {
            let line = tokio::time::timeout(Duration::from_secs(5), lines.next_line())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            let response: Value = serde_json::from_str(&line).unwrap();
            if response["id"] == id {
                break response;
            }
        };
        if id == 2 {
            assert_eq!(response["result"]["tools"].as_array().unwrap().len(), 18);
        } else {
            assert_eq!(response["result"]["isError"], true);
        }
    }
    drop(stdin);
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(status.success());
}
