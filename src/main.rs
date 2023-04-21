use std::net::SocketAddr;

use axum::{
    body::{boxed, Full},
    error_handling::HandleErrorLayer,
    extract::{Form, Query, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use rust_embed::RustEmbed;
use serde::Deserialize;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{debug, instrument, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod database;
mod error;
mod models;
mod templates;

use templates::HtmlTemplate;

pub use error::Error;

#[derive(Clone)]
struct AppState {
    pub database: database::Database,
}

#[derive(Deserialize)]
struct QuoteId {
    id: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct CreateQuoteRequest {
    #[serde(rename = "tal0")]
    number_1: String,
    #[serde(rename = "tal1")]
    number_2: String,
    #[serde(rename = "inp_dato")]
    date: String,
    #[serde(rename = "inp_tekst")]
    text: String,
    addition: String,
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
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}

#[instrument(skip_all)]
async fn quote(Query(params): Query<QuoteId>, State(state): State<AppState>) -> impl IntoResponse {
    let db = state.database;

    if let Some(id) = params.id {
        let quote = db.get_quote(id as i32).await.ok().flatten();

        if let Some(quote) = quote {
            (
                StatusCode::OK,
                HtmlTemplate(templates::QuoteTemplate { quote }).into_response(),
            )
        } else {
            (
                StatusCode::NOT_FOUND,
                HtmlTemplate(templates::NotFoundTemplate).into_response(),
            )
        }
    } else {
        trace!("fetching all quotes");
        let quotes = db.get_quotes().await;
        trace!("fetched all quotes");

        match quotes {
            Ok(quotes) => (
                StatusCode::OK,
                HtmlTemplate(templates::QuotesTemplate { quotes }).into_response(),
            ),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HtmlTemplate(templates::BaseTemplate { title: "error" }).into_response(),
            ),
        }
    }
}

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        HtmlTemplate(templates::NotFoundTemplate).into_response(),
    )
}

async fn new_quote() -> impl IntoResponse {
    HtmlTemplate(templates::NewQuoteTemplate {}).into_response()
}

#[instrument(skip(state))]
async fn post_quote(
    State(state): State<AppState>,
    Form(quote): Form<CreateQuoteRequest>,
) -> impl IntoResponse {
    let number_1 = quote.number_1.parse::<usize>().unwrap_or(0);
    let number_2 = quote.number_2.parse::<usize>().unwrap_or(0);
    let addition = quote.addition.parse::<usize>().unwrap_or(6080);

    if number_1 + number_2 == addition {
        debug!("adding quote to database");
        let db = state.database;

        let result = db
            .create_quote(models::NewQuote {
                date: quote.date,
                text: quote.text,
            })
            .await;

        match result {
            Ok(_) => "oki",
            Err(err) => "pis",
        }
    } else {
        "4kert"
    }
}

async fn robots<'a>() -> impl IntoResponse {
    StaticFile("robots.txt")
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("static/") {
        path = path.replace("static/", "");
    }

    StaticFile(path)
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

    let opts = cli::Opts::parse();

    debug!("connecting to database");
    let db = database::connect(opts.database_url.as_str()).await?;
    debug!("connected to database");

    debug!("running database migrations");
    database::migrate(db.clone()).await?;
    debug!("database migrations complete");

    let app_state = AppState { database: db };
    let app = Router::new()
        .route("/", get(quote))
        .route("/ny.php", post(post_quote).get(new_quote))
        .route("/static/*file", get(static_handler))
        .route("/robots.txt", get(robots))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}
