use std::net::SocketAddr;

use todo_list_application::{
    app_for_env, deploy_env_from_process, listen_port_from_process, load_assets, workspace_root,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let deploy_env = deploy_env_from_process();
    let root = workspace_root();
    let (openapi_spec, swagger_index_html) = load_assets(&root)?;
    let app = app_for_env(&deploy_env, openapi_spec, swagger_index_html);

    let port = listen_port_from_process();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on http://localhost:{port} (NODE_ENV={deploy_env})");

    axum::serve(listener, app).await?;
    Ok(())
}
