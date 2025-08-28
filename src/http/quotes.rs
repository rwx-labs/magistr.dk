use axum::{
    Form,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::{debug, instrument, trace};

use crate::{
    database,
    http::AppState,
    models,
    templates::{self, HtmlTemplate},
};

#[derive(Deserialize)]
pub struct QuoteId {
    id: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuoteRequest {
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

#[instrument(skip(state))]
pub(crate) async fn create(
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

        // Flush the quotes from
        if database::get_quotes_prime_cache(&db).await.is_some() {
            debug!("primed quotes cache");
        }

        match result {
            Ok(()) => "oki",
            Err(_) => "pis",
        }
    } else {
        "4kert"
    }
}

#[instrument(skip_all)]
pub(crate) async fn index(
    Query(params): Query<QuoteId>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let db = state.database;

    if let Some(id) = params.id {
        trace!("fetching single quote");
        let quote = database::get_quote(&db, id as i32).await;
        trace!("fetched single quote");

        if let Some(quote) = quote {
            trace!("quote found");
            (
                StatusCode::OK,
                HtmlTemplate(templates::QuoteTemplate { quote }).into_response(),
            )
        } else {
            trace!("quote not found");
            (
                StatusCode::NOT_FOUND,
                HtmlTemplate(templates::NotFoundTemplate).into_response(),
            )
        }
    } else {
        trace!("fetching all quotes");
        let quotes = database::get_quotes(&db).await;
        trace!("fetched all quotes");

        match quotes {
            Some(quotes) => (
                StatusCode::OK,
                HtmlTemplate(templates::QuotesTemplate { quotes }).into_response(),
            ),
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HtmlTemplate(templates::BaseTemplate {}).into_response(),
            ),
        }
    }
}

#[instrument]
pub(crate) async fn new() -> impl IntoResponse {
    HtmlTemplate(templates::NewQuoteTemplate {}).into_response()
}
