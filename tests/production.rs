//! Exercise real TLS handshakes and diagnostics through the installed CLI surface.

use std::{path::Path, process::Stdio, time::Duration};

use rcgen::{
    BasicConstraints, CertificateParams, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

fn binary(directory: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mcp-gmap-rs"));
    command
        .env_clear()
        .current_dir(directory)
        .arg("--no-env-file")
        .kill_on_drop(true);
    if let Some(profile) = std::env::var_os("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }
    command
}

struct Certificates {
    ca: String,
    server: String,
    server_key: String,
    client: String,
    client_key: String,
}

fn certificates() -> Certificates {
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    let ca_key = KeyPair::generate().unwrap();
    let ca = ca_params.self_signed(&ca_key).unwrap().pem();
    let issuer = Issuer::new(ca_params, ca_key);
    let mut params = CertificateParams::new(vec!["127.0.0.1".into(), "localhost".into()]).unwrap();
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let server_key = KeyPair::generate().unwrap();
    let server = params.signed_by(&server_key, &issuer).unwrap().pem();
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
    let client_key = KeyPair::generate().unwrap();
    let client = params.signed_by(&client_key, &issuer).unwrap().pem();
    Certificates {
        ca,
        server,
        server_key: server_key.serialize_pem(),
        client,
        client_key: client_key.serialize_pem(),
    }
}

fn write_certificates(directory: &Path, certs: &Certificates) {
    for (name, contents) in [
        ("server.pem", &certs.server),
        ("server-key.pem", &certs.server_key),
        ("ca.pem", &certs.ca),
    ] {
        std::fs::write(directory.join(name), contents).unwrap();
    }
}

fn client(certs: &Certificates, identity: bool) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .pool_max_idle_per_host(0)
        .add_root_certificate(reqwest::Certificate::from_pem(certs.ca.as_bytes()).unwrap());
    if identity {
        builder = builder.identity(
            reqwest::Identity::from_pem(format!("{}{}", certs.client, certs.client_key).as_bytes())
                .unwrap(),
        );
    }
    builder.build().unwrap()
}

async fn server(
    directory: &Path,
    mtls: bool,
) -> (
    Child,
    String,
    tokio::io::Lines<BufReader<tokio::process::ChildStderr>>,
) {
    let mut command = binary(directory);
    command.env("MCP_HTTP_TOKEN", "fixture-http-token").args([
        "serve",
        "http",
        "--listen",
        "127.0.0.1:0",
        "--tls-cert",
        "server.pem",
        "--tls-key",
        "server-key.pem",
        "--metrics",
        "--rate-limit",
        "2",
    ]);
    if mtls {
        command.args(["--tls-client-ca", "ca.pem"]);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
    let line = tokio::time::timeout(Duration::from_secs(10), lines.next_line())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let url = line
        .strip_prefix("MCP Streamable HTTP listening on ")
        .unwrap()
        .to_owned();
    assert!(url.starts_with("https://"));
    (child, url, lines)
}

#[tokio::test]
async fn tls_serves_mcp_and_metrics_and_rejects_untrusted_certificates() {
    let directory = tempfile::tempdir().unwrap();
    let certs = certificates();
    write_certificates(directory.path(), &certs);
    let (mut child, url, _logs) = server(directory.path(), false).await;
    let http = client(&certs, false);
    let health = url.replace("/mcp", "/health");
    assert_eq!(
        http.get(&health)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert!(
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(&health)
            .send()
            .await
            .is_err()
    );
    let metrics = url.replace("/mcp", "/metrics");
    assert_eq!(http.get(&metrics).send().await.unwrap().status(), 401);
    assert_eq!(
        http.get(&metrics)
            .bearer_auth("fixture-http-token")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"tls-test","version":"1"}}});
    let response = http
        .post(&url)
        .bearer_auth("fixture-http-token")
        .header("accept", "application/json, text/event-stream")
        .json(&initialize)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert!(response.headers().contains_key("mcp-session-id"));
    child.kill().await.unwrap();
}

#[tokio::test]
async fn mtls_requires_a_client_certificate_and_keeps_bearer_authentication() {
    let directory = tempfile::tempdir().unwrap();
    let certs = certificates();
    write_certificates(directory.path(), &certs);
    let (mut child, url, _logs) = server(directory.path(), true).await;
    let health = url.replace("/mcp", "/health");
    assert!(client(&certs, false).get(&health).send().await.is_err());
    let wrong = certificates();
    let wrong_identity = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .add_root_certificate(reqwest::Certificate::from_pem(certs.ca.as_bytes()).unwrap())
        .identity(
            reqwest::Identity::from_pem(format!("{}{}", wrong.client, wrong.client_key).as_bytes())
                .unwrap(),
        )
        .build()
        .unwrap();
    assert!(wrong_identity.get(&health).send().await.is_err());
    let http = client(&certs, true);
    assert_eq!(http.get(&health).send().await.unwrap().status(), 200);
    assert_eq!(
        http.get(url.replace("/mcp", "/metrics"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    child.kill().await.unwrap();
}

#[cfg(unix)]
fn signal(child: &Child, name: &str) {
    assert!(
        std::process::Command::new("kill")
            .args([name, &child.id().unwrap().to_string()])
            .status()
            .unwrap()
            .success()
    );
}

#[cfg(unix)]
#[tokio::test]
async fn certificate_reload_is_atomic_keeps_last_good_config_and_sigterm_exits_cleanly() {
    let directory = tempfile::tempdir().unwrap();
    let original = certificates();
    write_certificates(directory.path(), &original);
    let (mut child, url, mut logs) = server(directory.path(), false).await;
    let health = url.replace("/mcp", "/health");
    assert_eq!(
        client(&original, false)
            .get(&health)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let replacement = certificates();
    write_certificates(directory.path(), &replacement);
    signal(&child, "-HUP");
    let http = client(&replacement, false);
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if http.get(&health).send().await.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    assert!(client(&original, false).get(&health).send().await.is_err());
    std::fs::write(
        directory.path().join("server-key.pem"),
        "invalid-private-material-sentinel",
    )
    .unwrap();
    signal(&child, "-HUP");
    let failure = tokio::time::timeout(Duration::from_secs(5), logs.next_line())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(failure.contains("previous configuration retained"));
    assert!(!failure.contains("invalid-private-material-sentinel"));
    assert_eq!(http.get(&health).send().await.unwrap().status(), 200);
    signal(&child, "-TERM");
    assert!(
        tokio::time::timeout(Duration::from_secs(5), child.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
}

#[tokio::test]
async fn doctor_is_offline_by_default_redacts_credentials_and_reports_failure_via_exit_status() {
    let directory = tempfile::tempdir().unwrap();
    let sentinel = "doctor-key-must-not-be-printed";
    let output = binary(directory.path())
        .env("GOOGLE_MAPS_API_KEY", sentinel)
        .arg("doctor")
        .output()
        .await
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["live"], false);
    assert_eq!(report["ok"], true);
    assert_eq!(report["enabled_tools"].as_array().unwrap().len(), 42);
    assert!(!String::from_utf8_lossy(&output.stdout).contains(sentinel));
    let output = binary(directory.path())
        .arg("doctor")
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["ok"],
        false
    );
}

#[tokio::test]
async fn doctor_checks_explicit_oauth_credentials_and_reports_deferred_adc_without_contacting_google()
 {
    let directory = tempfile::tempdir().unwrap();
    let run = || {
        let mut command = binary(directory.path());
        command
            .env("GOOGLE_CLOUD_QUOTA_PROJECT", "fixture-project")
            .args([
                "--auth-mode",
                "oauth",
                "--enabled-tools",
                "maps_weather",
                "doctor",
            ]);
        command
    };
    let output = run().output().await.unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["name"] == "oauth_credentials" && check["status"] == "not_tested")
    );
    std::fs::write(directory.path().join("credentials.json"),serde_json::to_vec(&json!({"type":"authorized_user","client_id":"fixture-client","client_secret":"fixture-client-secret","refresh_token":"fixture-refresh-token","token_uri":"http://127.0.0.1:1/must-not-be-called"})).unwrap()).unwrap();
    let output = run()
        .env("GOOGLE_APPLICATION_CREDENTIALS", "credentials.json")
        .output()
        .await
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["name"] == "oauth_credentials" && check["status"] == "pass")
    );
    let output = run()
        .env("GOOGLE_APPLICATION_CREDENTIALS", "missing.json")
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("fixture-refresh-token"));
    let output = run()
        .env_remove("GOOGLE_CLOUD_QUOTA_PROJECT")
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["checks"].as_array().unwrap().iter().any(|check| {
        check["message"]
            .as_str()
            .is_some_and(|message| message.contains("GOOGLE_CLOUD_QUOTA_PROJECT"))
    }));
}

#[tokio::test]
async fn doctor_validates_tls_key_matching_and_remote_authentication_without_binding_a_socket() {
    let directory = tempfile::tempdir().unwrap();
    let certs = certificates();
    write_certificates(directory.path(), &certs);
    let run = || {
        let mut command = binary(directory.path());
        command
            .env("GOOGLE_MAPS_API_KEY", "fixture-key")
            .env("MCP_TLS_CERT", "server.pem")
            .env("MCP_TLS_KEY", "server-key.pem")
            .arg("doctor");
        command
    };
    let output = run().output().await.unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["name"] == "tls" && check["status"] == "pass")
    );
    let output = run()
        .env("MCP_HTTP_LISTEN", "0.0.0.0:3000")
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["name"] == "http_authentication" && check["status"] == "fail")
    );
    std::fs::write(
        directory.path().join("server-key.pem"),
        certificates().server_key,
    )
    .unwrap();
    let output = run().output().await.unwrap();
    assert!(!output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["name"] == "tls" && check["status"] == "fail")
    );
    let output = run()
        .env("MCP_HTTP_LISTEN", "invalid")
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["name"] == "http_configuration" && check["status"] == "fail")
    );
}

#[tokio::test]
async fn transport_and_budget_settings_obey_layers_and_reject_invalid_values_without_echoing_them()
{
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("runtime.env"),
        "GOOGLE_MAPS_MAX_REQUESTS=2\nGOOGLE_MAPS_MAX_RETRIES=0\nGOOGLE_MAPS_TOOL_TIMEOUT=10\n",
    )
    .unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_mcp-gmap-rs"));
    let output = command
        .env_clear()
        .current_dir(directory.path())
        .env("GOOGLE_MAPS_API_KEY", "fixture-key")
        .env("GOOGLE_MAPS_MAX_REQUESTS", "3")
        .args(["--env-file", "runtime.env", "--max-requests", "4", "doctor"])
        .output()
        .await
        .unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["limits"],
        json!({"google_attempts_per_tool":4,"retries_per_request":0,"tool_timeout_seconds":10})
    );
    for (name, value) in [
        ("MCP_METRICS_ENABLED", "secret-sentinel"),
        ("MCP_HTTP_RATE_LIMIT", "secret-sentinel"),
        ("GOOGLE_MAPS_MAX_REQUESTS", "0"),
        ("GOOGLE_MAPS_MAX_RETRIES", "6"),
        ("GOOGLE_MAPS_TOOL_TIMEOUT", "0"),
        ("MCP_LOG_FORMAT", "secret-sentinel"),
    ] {
        let output = binary(directory.path())
            .env(name, value)
            .arg("tools")
            .output()
            .await
            .unwrap();
        assert!(!output.status.success(), "{name}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(name));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("secret-sentinel"));
    }
    let output = binary(directory.path())
        .args(["serve", "http", "--tls-cert", "missing.pem"])
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must be supplied together"));
}

#[tokio::test]
async fn json_logs_preserve_cli_json_and_omit_tool_arguments_and_secrets() {
    let directory = tempfile::tempdir().unwrap();
    let output = binary(directory.path())
        .env("RUST_LOG", "info")
        .args([
            "--log-format",
            "json",
            "geocode",
            "--address",
            "private-address-sentinel",
        ])
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    let logs = String::from_utf8_lossy(&output.stderr);
    let event: Value = serde_json::from_str(logs.lines().next().unwrap()).unwrap();
    assert_eq!(event["fields"]["tool"], "maps_geocode");
    assert_eq!(event["fields"]["success"], false);
    assert!(!logs.contains("private-address-sentinel"));
    assert!(output.stdout.is_empty());
}
