package com.qci.todo;

import io.javalin.Javalin;

public final class TodoApplication {

    private TodoApplication() {}

    public static void main(String[] args) throws Exception {
        String deployEnv = TodoServer.deployEnvFromProcess();
        Javalin app = TodoServer.createAppFromWorkspace(deployEnv, TodoServer.workspaceRoot());

        int port = TodoServer.listenPortFromProcess();
        System.out.printf(
                "listening on http://localhost:%d (NODE_ENV=%s)%n", port, deployEnv);
        app.start(port);
    }
}
