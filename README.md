# apitester

A declarative HTTP test and load runner CLI written in Rust. Define your requests and assertions in YAML, then run functional tests or load tests from the command line.

## Installation

```bash
cargo build --release
# binary at target/release/apitester
```

## Usage

### Functional tests

Run requests sequentially and check assertions:

```bash
apitester test config.yaml
```

Run requests in parallel:

```bash
apitester test config.yaml --parallel
```

Change output format:

```bash
apitester test config.yaml --output json
apitester test config.yaml --output tap
```

### Load test

Run a sustained load test using the `load` section of the config:

```bash
apitester load config.yaml
```

### Run both

Run functional tests followed by a load test:

```bash
apitester run config.yaml
```

## Config format

```yaml
base_url: https://api.example.com
timeout: 30s  # optional, default 30s

requests:
  - name: health check
    method: GET       # default GET
    path: /health
    headers:
      Authorization: Bearer token123
    body: '{"key": "value"}'  # optional
    assert:
      status: 200
      body_contains: "ok"
      body_json:
        status: ok
      headers:
        content-type: application/json
      latency_lt: 500ms

load:
  concurrency: 10
  duration: 30s
  ramp_up: 5s         # optional
  requests:           # optional, defaults to all requests
    - health check
```

## Assertions

| Field | Description |
|---|---|
| `status` | Expected HTTP status code |
| `body_contains` | Substring that must appear in the response body |
| `body_json` | Expected JSON value (full equality) |
| `headers` | Expected header key/value pairs |
| `latency_lt` | Maximum acceptable latency (e.g. `200ms`, `1s`) |

## Output formats

- `terminal` — colored pass/fail table (default)
- `json` — machine-readable JSON array
- `tap` — TAP version 14, compatible with most CI test reporters

## Load test output

```
Load Test Results
  total requests: 399
  errors:         0
  rps:            22.01
  p50:            91ms
  p90:            191ms
  p99:            626ms
```
