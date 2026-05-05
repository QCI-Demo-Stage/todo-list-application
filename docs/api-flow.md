# Todo API — request flow

High-level routing for the Todo REST service defined in [`api-spec.yaml`](../api-spec.yaml). The Java server ([Javalin](https://javalin.io)) runs on port **8000** by default. When `NODE_ENV` is **staging** or **development**, the OpenAPI spec and Swagger UI are served; in **production**, documentation routes return 404 while `/health` remains available.

```mermaid
flowchart TB
  Client[Client / API consumer]
  Server[Java HTTP server (Javalin)]
  Health[GET /health]
  Docs["/api-docs — Swagger UI\n(staging & development)"]
  ListCreate["GET /todos — list\nPOST /todos — create"]
  ById["GET /todos/:todoId — read\nPUT /todos/:todoId — replace\nPATCH /todos/:todoId — partial update\nDELETE /todos/:todoId — delete"]

  Client --> Server
  Server --> Health
  Server --> Docs
  Server --> ListCreate
  Server --> ById
```

## Spec-only routes (this PR)

The current server implements **health checks** and **API documentation** (`/api-spec.yaml`, `/api-docs`). CRUD endpoints in the diagram match the contract in `api-spec.yaml`; a future implementation can wire them to persistence.
