use std::net::SocketAddr;

use axum::Router;
use tracing::{debug, info};

pub async fn run(router: Router) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let server = axum::Server::bind(&addr).serve(router.into_make_service());

    info!("server listening {:?}", addr);

    server
        .with_graceful_shutdown(async {
            use tokio::signal::{
                ctrl_c,
                unix::{signal, SignalKind},
            };

            let mut sig_int = signal(SignalKind::interrupt()).unwrap();
            let mut sig_term = signal(SignalKind::terminate()).unwrap();
            tokio::select! {
                _ = sig_int.recv() => debug!("SIGINT received"),
                _ = sig_term.recv() => debug!("SIGTERM received"),
                _ = ctrl_c() => debug!("'Ctrl C' received"),
            }
            debug!("gracefully shutting down");
        })
        .await?;

    info!("server shutdown");

    #[cfg(feature = "with-sentry")]
    if let Some(client) = sentry::Hub::current().client() {
        client.close(Some(std::time::Duration::from_secs(2)));
    }

    #[cfg(feature = "with-opentelemetry")]
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
