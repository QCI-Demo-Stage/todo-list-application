# Todo List Application

A todo list application repository used for demos and integration work.

## Overview

This project hosts the todo list application source and related artifacts (including API specifications where applicable).

## API flow

High-level request routing for the Todo REST service is documented in **[`docs/api-flow.md`](docs/api-flow.md)** (Mermaid diagram). The OpenAPI contract lives in [`api-spec.yaml`](api-spec.yaml). By default the server runs in **staging** mode on port **8000**. In staging or development, interactive docs are served from `/api-docs`.

## Prerequisites

- **Rust** (stable toolchain with Cargo)
- **Git**

## Getting started

1. Clone the repository:

   ```bash
   git clone https://github.com/QCI-Demo-Stage/todo-list-application.git
   cd todo-list-application
   ```

2. Run the API server (default port **8000**, default `NODE_ENV` **staging** so `/api-docs` is available):

   ```bash
   cargo run
   ```

   For an optimized binary:

   ```bash
   cargo run --release
   ```

3. Run tests:

   ```bash
   cargo test
   ```

## Configuration

Do not commit secrets. Copy environment templates if provided and use a local `.env` file (ignored by Git).

Environment variables match the prior Node deployment story:

- **`PORT`** — listen port (default **8000**; invalid values fall back to **8000**).
- **`NODE_ENV`** — **`staging`** or **`development`** serves `/api-spec.yaml` and `/api-docs`; other values (including **`production`**) hide docs while **`/health`** stays available. When unset, the server behaves like **`staging`**.

## Contributing

Use feature branches and open pull requests against `main`. Keep commits focused and descriptions clear.

## License

See the repository license file when one is added.
