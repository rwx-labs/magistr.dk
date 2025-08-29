use axum::{
    Router,
    body::Body,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use miette::IntoDiagnostic;
use rust_embed::RustEmbed;
use tokio::signal::unix::SignalKind;
use tokio::{net::TcpListener, signal};
use tower_http::{CompressionLevel, compression::CompressionLayer, trace::TraceLayer};
use tracing::{debug, info, instrument};

use crate::{AppState, config};

mod quotes;

#[instrument]
async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "404 page not found")
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = Body::from(content.data);
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404"))
                .unwrap(),
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("shutdown signal received");
}

#[instrument]
async fn robots() -> impl IntoResponse {
    StaticFile("robots.txt")
}

#[instrument]
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("static/") {
        path = path.replace("static/", "");
    }

    StaticFile(path)
}

pub async fn serve(config: &config::Http, app_state: crate::AppState) -> miette::Result<()> {
    debug!("starting http server");

    let mut app = Router::new()
        .route("/", get(quotes::index))
        .route("/ny.php", post(quotes::create).get(quotes::new))
        .route("/static/{*file}", get(static_handler))
        .route("/robots.txt", get(robots))
        .fallback(not_found)
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    if config.compression {
        debug!("enabling http compression");
        let compression_layer = CompressionLayer::new().quality(CompressionLevel::Fastest);
        app = app.layer(compression_layer);
    }

    let addr = config.address;
    debug!("binding to {}", addr);
    let listener = TcpListener::bind(addr).await.into_diagnostic()?;
    debug!("listening on {}", addr);
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .into_diagnostic()?;

    debug!("http server shut down");

    Ok(())
}
