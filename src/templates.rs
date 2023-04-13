//! Template rendering

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub struct HtmlTemplate<T>(pub T);

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
pub struct QuoteTemplate {
    pub name: String,
}

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<'a> {
    pub title: &'a str,
}

#[derive(Template)]
#[template(path = "create_quote.html")]
pub struct NewQuoteTemplate {}

mod utils {
    use chrono::Local;
    use rand::prelude::*;

    pub fn random_statement() -> String {
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

    pub fn current_date_formatted(s: &str) -> String {
        let dt = Local::now();

        dt.format(s).to_string()
    }

    pub fn random_number(min: usize, max: usize) -> String {
        let mut rng = rand::thread_rng();

        rng.gen_range(min..=max).to_string()
    }
}
