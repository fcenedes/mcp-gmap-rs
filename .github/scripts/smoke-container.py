"""Exercise the built container without Google credentials or live API calls."""

import argparse
import json
import select
import ssl
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path


HARDENING = ["--read-only", "--cap-drop=ALL", "--security-opt=no-new-privileges"]
INITIALIZE = {
    "jsonrpc": "2.0", "id": 1, "method": "initialize",
    "params": {
        "protocolVersion": "2025-11-25", "capabilities": {},
        "clientInfo": {"name": "container-smoke", "version": "1"},
    },
}


def docker(*args, check=True):
    return subprocess.run(["docker", *args], check=check, text=True, capture_output=True, timeout=60)


def json_response(response):
    body = response.read().decode()
    if response.headers.get_content_type() == "text/event-stream":
        return next(json.loads(line[5:]) for line in body.splitlines()
                    if line.startswith("data:") and line[5:].strip())
    return json.loads(body)


def check_stdio(image):
    name = "mcp-gmap-stdio-smoke-" + uuid.uuid4().hex
    process = subprocess.Popen(
        ["docker", "run", "--rm", "--name", name, "-i", *HARDENING, image, "serve", "stdio"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
    )
    try:
        def request(value):
            process.stdin.write(json.dumps(value) + "\n")
            process.stdin.flush()

        def response():
            assert select.select([process.stdout], [], [], 15)[0], "stdio response timed out"
            return json.loads(process.stdout.readline())

        request(INITIALIZE)
        assert response()["result"]["serverInfo"]["name"] == "mcp-gmap-rs"
        request({"jsonrpc": "2.0", "method": "notifications/initialized"})
        request({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
        assert len(response()["result"]["tools"]) == 23
        process.stdin.close()
        assert process.wait(timeout=15) == 0, process.stderr.read()
    finally:
        docker("rm", "--force", name, check=False)
        if process.poll() is None:
            process.kill()
            process.wait(timeout=15)


def check_http(image):
    with tempfile.TemporaryDirectory() as directory:
        dotenv = Path(directory) / "runtime.env"
        dotenv.write_text(
            "MCP_HTTP_TOKEN=file-token\n"
            "MCP_HTTP_ALLOWED_HOSTS=container.example:3000\n"
            "GOOGLE_MAPS_ENABLED_TOOLS=maps_geocode\n"
        )
        dotenv.chmod(0o644)
        container = docker(
            "run", "--detach", *HARDENING, "--publish", "127.0.0.1::3000",
            "--mount", f"type=bind,src={dotenv},dst=/app/.env,readonly",
            "--env", "MCP_HTTP_TOKEN=environment-token",
            "--env", "GOOGLE_MAPS_ENABLED_TOOLS=maps_timezone", image,
        ).stdout.strip()
        try:
            port = docker("port", container, "3000/tcp").stdout.strip().rsplit(":", 1)[1]
            base = f"http://127.0.0.1:{port}"
            # Local container probes must not use a developer's outbound HTTP proxy.
            http = urllib.request.build_opener(urllib.request.ProxyHandler({}))
            deadline = time.monotonic() + 30
            while True:
                try:
                    with http.open(base + "/health", timeout=2) as response:
                        assert response.read() == b"ok"
                    break
                except (OSError, urllib.error.URLError):
                    if time.monotonic() >= deadline:
                        raise AssertionError(docker("logs", container).stderr)
                    time.sleep(0.25)

            def post(value, token="environment-token", session=None):
                headers = {
                    "Content-Type": "application/json",
                    "Accept": "application/json, text/event-stream",
                    "Host": "container.example:3000",
                    "Authorization": "Bearer " + token,
                }
                if session:
                    headers["Mcp-Session-Id"] = session
                    headers["MCP-Protocol-Version"] = "2025-11-25"
                return http.open(urllib.request.Request(
                    base + "/mcp", data=json.dumps(value).encode(), headers=headers,
                ), timeout=5)

            try:
                post(INITIALIZE, token="file-token")
            except urllib.error.HTTPError as error:
                assert error.code == 401
            else:
                raise AssertionError("dotenv token overrode the environment token")
            with post(INITIALIZE) as response:
                session = response.headers["Mcp-Session-Id"]
                assert json_response(response)["result"]["serverInfo"]["name"] == "mcp-gmap-rs"
            with post({"jsonrpc": "2.0", "method": "notifications/initialized"}, session=session) as response:
                assert response.status == 202
            with post({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}, session=session) as response:
                names = [tool["name"] for tool in json_response(response)["result"]["tools"]]
                assert names == ["maps_timezone"], names
            docker("stop", "--time", "10", container)
            assert docker("inspect", "--format", "{{.State.ExitCode}}", container).stdout.strip() == "0"
        finally:
            docker("rm", "--force", container, check=False)


def check_tls(image):
    with tempfile.TemporaryDirectory() as directory:
        cert, key = Path(directory) / "cert.pem", Path(directory) / "key.pem"
        subprocess.run([
            "openssl", "req", "-x509", "-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:P-256",
            "-nodes", "-days", "1", "-subj", "/CN=localhost",
            "-addext", "subjectAltName=DNS:localhost,IP:127.0.0.1",
            "-keyout", str(key), "-out", str(cert),
        ], check=True, capture_output=True, timeout=15)
        # Synthetic one-day test credentials must be readable by the container UID.
        Path(directory).chmod(0o755)
        key.chmod(0o644)
        cert.chmod(0o644)
        container = docker(
            "run", "--detach", *HARDENING, "--publish", "127.0.0.1::3000",
            "--mount", f"type=bind,src={directory},dst=/tls,readonly",
            "--env", "MCP_HTTP_TOKEN=tls-fixture-token",
            "--env", "MCP_HTTP_ALLOWED_HOSTS=tls.example:3000", image,
            "serve", "http", "--tls-cert", "/tls/cert.pem", "--tls-key", "/tls/key.pem",
            "--metrics", "--rate-limit", "1",
        ).stdout.strip()
        try:
            port = docker("port", container, "3000/tcp").stdout.strip().rsplit(":", 1)[1]
            base = f"https://127.0.0.1:{port}"
            context = ssl.create_default_context(cafile=str(cert))
            http = urllib.request.build_opener(urllib.request.ProxyHandler({}), urllib.request.HTTPSHandler(context=context))
            deadline = time.monotonic() + 30
            while True:
                try:
                    with http.open(base + "/health", timeout=2) as response:
                        assert response.read() == b"ok"
                    break
                except (OSError, urllib.error.URLError):
                    if time.monotonic() >= deadline:
                        raise AssertionError(docker("logs", container).stderr)
                    time.sleep(0.25)
            headers = {
                "Content-Type": "application/json", "Accept": "application/json, text/event-stream",
                "Host": "tls.example:3000", "Authorization": "Bearer tls-fixture-token",
            }
            request = lambda: urllib.request.Request(base + "/mcp", data=json.dumps(INITIALIZE).encode(), headers=headers)
            with http.open(request(), timeout=5) as response:
                assert json_response(response)["result"]["serverInfo"]["name"] == "mcp-gmap-rs"
            try:
                http.open(request(), timeout=5)
            except urllib.error.HTTPError as error:
                assert error.code == 429 and error.headers.get("Retry-After")
            else:
                raise AssertionError("HTTP rate limit was not enforced")
            try:
                http.open(base + "/metrics", timeout=5)
            except urllib.error.HTTPError as error:
                assert error.code == 401
            else:
                raise AssertionError("Metrics did not require the bearer token")
            with http.open(urllib.request.Request(base + "/metrics", headers=headers), timeout=5) as response:
                assert "mcp_gmap_http_rate_limited_total 1" in response.read().decode()
            docker("kill", "--signal=HUP", container)
            with http.open(base + "/health", timeout=5) as response:
                assert response.status == 200
            docker("stop", "--time", "10", container)
            assert docker("inspect", "--format", "{{.State.ExitCode}}", container).stdout.strip() == "0"
        finally:
            docker("rm", "--force", container, check=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image")
    image = parser.parse_args().image
    assert docker("image", "inspect", "--format", "{{.Config.User}}", image).stdout.strip() == "10001:10001"
    docker("run", "--rm", *HARDENING, "--entrypoint", "/bin/sh", image, "-c",
           "test ! -e /app/.env && test -s /etc/ssl/certs/ca-certificates.crt")
    tools = json.loads(docker("run", "--rm", *HARDENING, image, "tools").stdout)
    assert len(tools) == 23
    oauth_tools = json.loads(docker(
        "run", "--rm", *HARDENING, "--env", "GOOGLE_MAPS_AUTH_MODE=oauth",
        "--env", "GOOGLE_APPLICATION_CREDENTIALS=/not-mounted/credentials.json",
        image, "tools",
    ).stdout)
    assert len(oauth_tools) == 23
    doctor = docker("run", "--rm", *HARDENING, "--env", "GOOGLE_MAPS_API_KEY=offline-fixture-key",
                    "--env", "MCP_HTTP_TOKEN=offline-fixture-token", image, "doctor").stdout
    assert json.loads(doctor)["ok"] and "offline-fixture-key" not in doctor
    unconfigured = docker("run", "--rm", *HARDENING, image, check=False)
    assert unconfigured.returncode != 0 and "MCP_HTTP_TOKEN" in unconfigured.stderr
    check_stdio(image)
    check_http(image)
    check_tls(image)
    print("Container passed: non-root/read-only, CLI, stdio, HTTP/TLS, metrics, limits, doctor, mounted dotenv/environment overrides, auth, SIGHUP and SIGTERM")


if __name__ == "__main__":
    main()
