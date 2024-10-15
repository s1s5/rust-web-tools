use axum::Router;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{debug, info};

pub async fn run(router: Router, port: Option<u16>) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port.unwrap_or(8000)));
    info!("server listening {:?}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    info!("server shutdown");

    #[cfg(feature = "with-sentry")]
    if let Some(client) = sentry::Hub::current().client() {
        client.close(Some(std::time::Duration::from_secs(2)));
    }

    #[cfg(feature = "with-opentelemetry")]
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    #[cfg(unix)]
    let sig_int = async {
        signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sig_int = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {debug!("'Ctrl C' received")},
        _ = sig_int => {debug!("SIGINT received")},
        _ = terminate => {debug!("SIGTERM received")},
    }
}
