# SGuard

> High-performance, asynchronous edge API Gateway and reverse proxy built in Rust with Tokio and Hyper.

[![Rust](https://img.shields.io/badge/rust-edition%202021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

---

## Overview

**SGuard** is an asynchronous API Gateway designed for microservice architectures. It combines the declarative routing and pluggable security patterns of frameworks like Spring Cloud Gateway with the performance, safety, and concurrency guarantees of Rust.

For full architectural details, see the [Architecture Document](docs/ARCHITECTURE.md).

---

## Key Features

- **Asynchronous & Non-Blocking**: Built on Tokio and Hyper 1.x for high-throughput, low-latency request handling.
- **True Payload Streaming**: Streams both request and response bodies end-to-end without buffering payloads in memory.
- **Pre-Compiled Filter Chain**: Functional, zero-allocation filter chain pre-assembled at startup.
- **Pluggable Security**: Extensible authentication system supporting JWT Bearer tokens, HTTP Basic Auth, Cookie sessions, and SAML.
- **Declarative Routing**: Dynamic route configuration via `routes.yaml` with extensible predicates (`Path`, `Host`, `Query`, `Cookie`, `Regex`).
- **Resilience & Admission Control**: Bounded downstream concurrency (20,000 connections), upstream admission limiting (256 inflight, 2048 queue with fast load shedding), and safe idempotent request retries on stale pooled connections.
- **Diagnostic Telemetry**: Automatic error categorization and non-blocking background health metrics reporting.

---

## Architecture Overview

```mermaid
flowchart LR
    Client(["Downstream Client"]) -->|TCP :8080| Listener["sguard-http"]
    Listener --> Chain["sguard-filter Pipeline\n(Auth, CSRF, Logging, Routing)"]
    Chain --> Admission["sguard-proxy Admission\n(Max 256 Inflight)"]
    Admission --> Pool["Connection Pool\n(KeepAlive, Auto-Retry)"]
    Pool --> Upstream["Upstream Microservices"]
```

Read more in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Workspace Structure

The project is structured as a Cargo workspace:

- [`sguard`](sguard): Main gateway binary and bootstrapping.
- [`sguard-core`](sguard-core): Common traits, domain types (`HttpRequest`, `HttpResponse`, `RequestContext`), and filter abstractions.
- [`sguard-http`](sguard-http): Tokio TCP listener, connection bounding, Hyper HTTP/1.1 driver, and error-to-response mapping.
- [`sguard-filter`](sguard-filter): Filter chain engine and built-in filters (Security, Auth, Logging, Routing, Headers).
- [`sguard-security`](sguard-security): Pluggable authentication converters, managers, providers (JWT, Basic, Cookie, SAML).
- [`sguard-routing`](sguard-routing): Declarative route resolution from `routes.yaml` with predicate matching.
- [`sguard-proxy`](sguard-proxy): Upstream proxying, connection pooling, load shedding, and hop-by-hop header cleanup.
- [`sguard-error`](sguard-error): Centralized typed error definitions and classification.

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.79+ recommended, 2021 edition)
- (Optional) Docker for containerized deployment

### Configuration

1. **Routing Configuration**: Define upstream services and predicates in `routes.yaml`:
   ```yaml
   routes:
     - id: user-service
       uri: http://localhost:5000
       predicates:
         - Path=/user/api/**
   ```

2. **Logging Configuration**: Configure appenders and log levels in `log4rs.yaml`.

### Build & Run

Run the gateway locally:
```bash
cargo run --release -p sguard
```

Run tests:
```bash
cargo test --workspace
```

### Docker Deployment

Build and run using the multi-stage Dockerfile:
```bash
docker build -t sguard:latest .
docker run -p 8080:8080 sguard:latest
```

---

## Performance Testing

Benchmark scripts using `wrk` and Lua scripts are located in `test/performance/`:

```bash
cd test/performance
TOKEN="<your-jwt-token>" ./run.sh
```

---

## Documentation

- [Architecture Specification](docs/ARCHITECTURE.md)
