# Troubleshooting

First identify the failing connection: a local configuration error, the MCP client-to-server connection, or an outbound Google request. Correct the relevant layer, then rerun the affected operation.

## Start offline

```sh
mcp-gmap-rs --version
mcp-gmap-rs tools
mcp-gmap-rs doctor
```

`doctor` prints JSON even when checks fail. Inspect its exit status and `checks[].status`: `pass`, `fail` or `not_tested`. Offline checks do not prove IAM, API enablement, billing, certificate trust/expiry or ADC access. `/health` only proves process liveness.

If a malformed local dotenv file blocks discovery, `--no-env-file tools` skips it. This can change the selected tool set, so do not mistake it for verification of the intended file configuration.

## Interpret common failures

| Symptom | Next check |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | API-key mode is selected or this operation reaches a key-only endpoint. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | Set the consumer project ID/number for OAuth. |
| Google 403 / API disabled | Enable the exact API in the credential’s consumer project; check billing and API-key restrictions. |
| `serviceusage.services.use` denied | Grant the service account `roles/serviceusage.serviceUsageConsumer` on the quota project; allow IAM propagation, then retry. |
| API-key IP denial | Allow the server or cluster’s public egress IP, not its private pod IP. |
| MCP HTTP 401 | Send the server’s bearer token; a Google access token authenticates a different connection. |
| Host/Origin rejected | Match the configured Host and browser origin exactly, including relevant ports. |
| HTTP 429 from MCP | The process-wide limiter is exhausted; honor `Retry-After` and check client concurrency. |
| OAuth timeout | Check token/STS/IAM or metadata connectivity, system clock and per-request timeout. |
| TLS connection fails | Check the certificate chain, hostname, trust and optional mTLS client certificate separately from HTTP authorization. |
| Exit 0 but incomplete data | Inspect matrix/composite per-item errors, not just the process exit status. |

## Verify live access deliberately

Live probes make potentially billable calls. Scope them to the API dependencies you need:

```sh
mcp-gmap-rs --enabled-tools maps_weather doctor --live
```

A passing probe does not guarantee every geographic region or operation. After a Google Cloud change, rerun the same failed call rather than inferring success from console settings. See [authentication](./authentication) for IAM and credential sources, [configuration](./configuration) for precedence and [production operations](./production) for budgets, deadlines and metrics. Never print credential JSON or dump the environment into an issue report.
