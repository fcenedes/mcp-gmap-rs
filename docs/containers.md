# Docker and Kubernetes

For Google API-key creation, API enablement and service-account OAuth, follow the
[Google Cloud setup guide](google-cloud-setup.md). The same layered settings work
in containers. OAuth can use a mounted JSON credential file or the workload's ADC
identity; credentials are supplied at runtime.

Build from this source checkout:

```sh
docker build -t mcp-gmap-rs:local .
```

The multi-stage Dockerfile compiles with Rust 1.94.1 and the lockfile. The runtime
contains the executable, Debian CA certificates and licenses, runs as UID/GID
10001, and supports a read-only root filesystem. The allowlisted build context
excludes local `.env` files, Git data and build output. No credentials are needed
to build. BuildKit caches Cargo downloads and build output for subsequent builds.

The same Dockerfile builds natively on Linux amd64 and arm64. To build both and
push to your registry with a configured Buildx builder:

```sh
docker buildx build --platform linux/amd64,linux/arm64 \
  -t registry.example.com/your-team/mcp-gmap-rs:YOUR_VERSION --push .
```

No container registry publication is configured automatically. Replace image
names and versions with the image you built or published.

## Run HTTP, CLI or stdio

The image defaults to `serve http` on `0.0.0.0:3000`. Supply a bearer token at
runtime, for example through the same `.env` format shown in `.env.example`:

```sh
docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges \
  --env-file .env -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

Ensure `.env` contains `MCP_HTTP_TOKEN` as well as `GOOGLE_MAPS_API_KEY` for Google
calls. The former is required because the container listens outside loopback.
Clients use `http://127.0.0.1:3000/mcp` and `Authorization: Bearer <token>`.
`GET /health` needs no token and does not contact Google.

Docker's `--env-file` injects environment variables. To exercise the application's
file loader instead, mount a readable file and select it:

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  -e GOOGLE_MAPS_LANGUAGE=fr -p 127.0.0.1:3000:3000 \
  mcp-gmap-rs:local --env-file /etc/mcp-gmap/runtime.env serve http
```

The environment overrides values in the mounted file, and CLI flags override
both. The image's `MCP_HTTP_LISTEN=0.0.0.0:3000` is also an environment value;
change it with `-e MCP_HTTP_LISTEN=...` or `--listen` if needed. To use environment
variables exclusively, pass `--no-env-file serve http`.

Override the image command for CLI or stdio use:

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm --env-file .env mcp-gmap-rs:local geocode --address Paris
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

Use `-i` for stdio, without allocating a TTY. When writing CLI output files, mount
a directory writable by UID 10001 and use `--output` within it.

## Kubernetes

The example `deploy/kubernetes.yaml` supplies non-secret settings through a
ConfigMap and credentials through an existing Secret named `mcp-gmap-rs`.
Create that Secret from a local file containing `GOOGLE_MAPS_API_KEY` and
`MCP_HTTP_TOKEN` (the file is not committed):

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
```

Replace `mcp-gmap-rs:local` in the Deployment with your registry image, or load
your local image into your development cluster. Then apply the example:

```sh
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

The example targets the `default` namespace. Adjust the configured service DNS
hostnames when using another namespace or an ingress. The ClusterIP Service is
private to the cluster; use a TLS ingress/proxy for remote clients, forwarding a
Host listed in `MCP_HTTP_ALLOWED_HOSTS`, authorization, and MCP/SSE headers. Add
explicit browser origins, including ports, when required.

The deployment uses `--no-env-file` because all configuration is injected through
`envFrom`. As an alternative, mount a Secret containing a dotenv file, remove
that flag and set `args: ["--env-file", "/etc/mcp-gmap/runtime.env", "serve", "http"]`.
The projected file must be readable by UID 10001; environment variables from
Secrets or ConfigMaps still override it. See the upstream Kubernetes guidance on
[environment injection](https://kubernetes.io/docs/tasks/inject-data-application/define-environment-variable-container/)
and [Secret volumes](https://kubernetes.io/docs/concepts/configuration/secret/).

Configuration is read at startup. Restart pods after Secret/ConfigMap changes.
Probes use `/health` for HTTP availability; a healthy pod does not prove that a
Google API is enabled or its key is authorized. Google calls require outbound
HTTPS, working DNS and a key permitted for the cluster's egress address.

Keep one replica for the default in-memory MCP session store. Multiple replicas
need session affinity so each session reaches its owning pod; rolling restarts
discard sessions and clients must reconnect. Resource requests/limits in the
example are starting values to tune for your workload.

SIGTERM triggers graceful MCP/HTTP shutdown; Kubernetes allows 30 seconds in the
example. The image also runs with dropped capabilities and no privilege escalation.

## Production HTTPS, metrics and limits

The [production guide](production.md) covers native TLS/mTLS, certificate reload,
the optional `deploy/gateway.yaml` HTTPS Gateway, metrics, request budgets, HTTP
rate limits, JSON logs and offline/live `doctor` checks. Pass the same TLS flags
after the image name, or inject `MCP_TLS_CERT`, `MCP_TLS_KEY` and optional
`MCP_TLS_CLIENT_CA` with read-only PEM mounts. Keep all real keys outside the image.
The default ConfigMap sets request limits and JSON logging; metrics remain off
until explicitly enabled. Its default plaintext Service is intended for the
private gateway-to-pod hop.
