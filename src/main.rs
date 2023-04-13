use std::net::SocketAddr;

use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use phf::phf_map;
use serde::Deserialize;
use tokio::signal;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static ASSETS: phf::Map<&'static str, &'static [u8]> = phf_map! {
    "static/fartscroll.js" => include_bytes!("../static/fartscroll.js"),
    "static/robots.txt" => include_bytes!("../static/robots.txt")

};

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "quote.html")]
struct QuoteTemplate {
    name: String,
}

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate<'a> {
    title: &'a str,
}

mod utils {
    pub fn random_statement() -> String {
        use rand::prelude::*;

        const STATEMENTS: &[&str; 11] = &[
            "Treds",
            "Hold kæft!",
            "Goddag.",
            "Grønne svin på et skod beat",
            "dddddd",
            "<robutler> Goddag og farvel.",
            "ja",
            "60",
            ":D",
            "Godt.",
            "100",
        ];

        let mut rng = rand::thread_rng();

        STATEMENTS.choose(&mut rng).unwrap().to_string()
    }
}

#[derive(Deserialize)]
struct QuoteId {
    id: Option<usize>,
}

async fn quote(Query(params): Query<QuoteId>) -> impl IntoResponse {
    if let Some(id) = params.id {
        let name = format!("{}", id);

        HtmlTemplate(QuoteTemplate { name }).into_response()
    } else {
        HtmlTemplate(BaseTemplate { title: "goddag" }).into_response()
    }
}

async fn fartscroll<'a>() -> &'a [u8] {
    ASSETS.get("static/fartscroll.js").unwrap()
}

async fn robots<'a>() -> &'a [u8] {
    ASSETS.get("static/robots.txt").unwrap()
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

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "magistr=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(quote))
        .route("/static/fartscroll.js", get(fartscroll))
        .route("/robots.txt", get(robots))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}
