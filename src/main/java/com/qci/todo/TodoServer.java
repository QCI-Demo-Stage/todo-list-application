package com.qci.todo;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import io.javalin.Javalin;
import io.javalin.http.Context;
import io.javalin.http.HandlerType;
import io.javalin.http.HttpStatus;

/** HTTP routing and environment helpers; mirrors the prior Axum server contract. */
public final class TodoServer {

    private TodoServer() {}

    /**
     * Project root for asset resolution. Tests may set system property {@code todo.workspace.root}
     * to a directory containing {@code api-spec.yaml} and {@code swagger/index.html}.
     */
    public static Path workspaceRoot() {
        String override = System.getProperty("todo.workspace.root");
        if (override != null && !override.isBlank()) {
            return Path.of(override).toAbsolutePath().normalize();
        }
        return Path.of("").toAbsolutePath().normalize();
    }

    /** Unset {@code NODE_ENV} behaves like {@code staging}. */
    public static String deployEnvFromProcess() {
        String v = System.getenv("NODE_ENV");
        return v == null || v.isBlank() ? "staging" : v;
    }

    /** Default port {@code 8000}; invalid or missing {@code PORT} falls back to {@code 8000}. */
    public static int listenPortFromProcess() {
        String raw = System.getenv("PORT");
        if (raw == null) {
            return 8000;
        }
        String trimmed = raw.trim();
        try {
            int p = Integer.parseInt(trimmed);
            return p > 0 && p <= 65535 ? p : 8000;
        } catch (NumberFormatException e) {
            return 8000;
        }
    }

    public static byte[][] loadAssets(Path workspaceRoot) throws IOException {
        Path specPath = workspaceRoot.resolve("api-spec.yaml");
        Path swaggerPath = workspaceRoot.resolve("swagger").resolve("index.html");
        byte[] openapiSpec = Files.readAllBytes(specPath);
        byte[] swaggerIndexHtml = Files.readAllBytes(swaggerPath);
        return new byte[][] { openapiSpec, swaggerIndexHtml };
    }

    public static boolean docsEnabledForEnv(String deployEnv) {
        return "development".equals(deployEnv) || "staging".equals(deployEnv);
    }

    /** Loads spec and Swagger HTML from {@code root}, then builds the app (fails if assets are missing). */
    public static Javalin createAppFromWorkspace(String deployEnv, Path root) throws IOException {
        byte[][] assets = loadAssets(root);
        return createApp(deployEnv, assets[0], assets[1]);
    }

    public static Javalin createApp(String deployEnv, byte[] openapiSpec, byte[] swaggerIndexHtml) {
        boolean docsEnabled = docsEnabledForEnv(deployEnv);

        Javalin app = Javalin.create(config -> {
            config.showJavalinBanner = false;
            config.router.ignoreTrailingSlashes = false;
        });

        app.before(ctx -> {
            if (ctx.method() != HandlerType.GET) {
                ctx.status(HttpStatus.METHOD_NOT_ALLOWED);
                ctx.skipRemainingHandlers();
            }
        });

        app.get("/health", ctx -> {
            ctx.contentType("application/json");
            ctx.result("{\"status\":\"ok\"}");
        });

        app.get("/api-spec.yaml", ctx -> serveOpenApiSpec(docsEnabled, ctx, openapiSpec));

        app.get("/api-docs", ctx -> handleApiDocsRoot(docsEnabled, ctx));

        app.get("/api-docs/", ctx -> serveSwaggerUi(docsEnabled, ctx, swaggerIndexHtml));

        app.get("/api-docs/*", ctx -> serveSwaggerUi(docsEnabled, ctx, swaggerIndexHtml));

        return app;
    }

    private static void handleApiDocsRoot(boolean docsEnabled, Context ctx) {
        if (!docsEnabled) {
            ctx.status(HttpStatus.NOT_FOUND);
            return;
        }
        ctx.redirect("/api-docs/", HttpStatus.MOVED_PERMANENTLY);
    }

    private static void serveSwaggerUi(boolean docsEnabled, Context ctx, byte[] swaggerIndexHtml) {
        if (!docsEnabled) {
            ctx.status(HttpStatus.NOT_FOUND);
            return;
        }
        ctx.contentType("text/html; charset=utf-8");
        ctx.result(swaggerIndexHtml);
    }

    private static void serveOpenApiSpec(boolean docsEnabled, Context ctx, byte[] openapiSpec) {
        if (!docsEnabled) {
            ctx.status(HttpStatus.NOT_FOUND);
            return;
        }
        ctx.contentType("application/yaml; charset=utf-8");
        ctx.result(openapiSpec);
    }
}
