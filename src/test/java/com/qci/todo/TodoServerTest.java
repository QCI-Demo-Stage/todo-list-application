package com.qci.todo;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;

import org.junit.jupiter.api.Test;

import io.javalin.Javalin;

class TodoServerTest {

    private static final HttpClient CLIENT =
            HttpClient.newBuilder()
                    .connectTimeout(Duration.ofSeconds(2))
                    .followRedirects(HttpClient.Redirect.NEVER)
                    .build();

    private static byte[] tinySpec() {
        return "openapi: 3.0.3\n".getBytes();
    }

    private static byte[] tinyHtml() {
        return "<!DOCTYPE html><html></html>".getBytes();
    }

    private static int startPort(Javalin app) {
        app.start(0);
        return app.port();
    }

    private static HttpResponse<String> get(String url) throws IOException, InterruptedException {
        HttpRequest req =
                HttpRequest.newBuilder()
                        .uri(URI.create(url))
                        .timeout(Duration.ofSeconds(5))
                        .GET()
                        .build();
        return CLIENT.send(req, HttpResponse.BodyHandlers.ofString());
    }

    private static HttpResponse<String> post(String url) throws IOException, InterruptedException {
        HttpRequest req =
                HttpRequest.newBuilder()
                        .uri(URI.create(url))
                        .timeout(Duration.ofSeconds(5))
                        .method("POST", HttpRequest.BodyPublishers.noBody())
                        .build();
        return CLIENT.send(req, HttpResponse.BodyHandlers.ofString());
    }

    @Test
    void healthReturnsJsonOk() throws Exception {
        Javalin app = TodoServer.createApp("staging", tinySpec(), tinyHtml());
        try {
            int port = startPort(app);
            var res = get("http://127.0.0.1:" + port + "/health");
            assertEquals(200, res.statusCode());
            assertTrue(
                    res.headers().firstValue("Content-Type").orElse("").startsWith("application/json"));
            assertArrayEquals("{\"status\":\"ok\"}".getBytes(), res.body().getBytes());
        } finally {
            app.stop();
        }
    }

    @Test
    void productionHidesDocs() throws Exception {
        Javalin app = TodoServer.createApp("production", tinySpec(), tinyHtml());
        try {
            int port = startPort(app);
            for (String path : new String[] {"/api-spec.yaml", "/api-docs/", "/api-docs"}) {
                var res = get("http://127.0.0.1:" + port + path);
                assertEquals(404, res.statusCode(), "path=" + path);
            }
        } finally {
            app.stop();
        }
    }

    @Test
    void stagingServesSpecAndDocs() throws Exception {
        byte[] spec = tinySpec();
        byte[] html = tinyHtml();
        Javalin app = TodoServer.createApp("staging", spec, html);
        try {
            int port = startPort(app);
            var specRes = get("http://127.0.0.1:" + port + "/api-spec.yaml");
            assertEquals(200, specRes.statusCode());
            assertArrayEquals(spec, specRes.body().getBytes());

            var docsRes = get("http://127.0.0.1:" + port + "/api-docs/");
            assertEquals(200, docsRes.statusCode());
            assertTrue(
                    docsRes.headers()
                            .firstValue("Content-Type")
                            .orElse("")
                            .toLowerCase()
                            .replace(" ", "")
                            .contains("text/html;charset=utf-8"));
        } finally {
            app.stop();
        }
    }

    @Test
    void apiDocsRedirectsToSlash() throws Exception {
        Javalin app = TodoServer.createApp("staging", tinySpec(), tinyHtml());
        try {
            int port = startPort(app);
            var res = get("http://127.0.0.1:" + port + "/api-docs");
            assertEquals(301, res.statusCode());
            String location = res.headers().firstValue("Location").orElse("");
            assertTrue(
                    location.endsWith("/api-docs/"),
                    "Location was: " + location);
        } finally {
            app.stop();
        }
    }

    @Test
    void postReturns405() throws Exception {
        Javalin app = TodoServer.createApp("staging", tinySpec(), tinyHtml());
        try {
            int port = startPort(app);
            var res = post("http://127.0.0.1:" + port + "/api-docs/");
            assertEquals(405, res.statusCode());
        } finally {
            app.stop();
        }
    }

    @Test
    void developmentEnablesDocs() throws Exception {
        Javalin app = TodoServer.createApp("development", tinySpec(), tinyHtml());
        try {
            int port = startPort(app);
            var res = get("http://127.0.0.1:" + port + "/api-spec.yaml");
            assertEquals(200, res.statusCode());
        } finally {
            app.stop();
        }
    }

    @Test
    void createAppLoadsRealAssetsFromWorkspace() throws IOException {
        Path root = Path.of(System.getProperty("user.dir")).toAbsolutePath().normalize();
        byte[][] loaded = TodoServer.loadAssets(root);
        byte[] fromDiskSpec = Files.readAllBytes(root.resolve("api-spec.yaml"));
        byte[] fromDiskHtml = Files.readAllBytes(root.resolve("swagger/index.html"));
        assertArrayEquals(fromDiskSpec, loaded[0]);
        assertArrayEquals(fromDiskHtml, loaded[1]);
    }
}
