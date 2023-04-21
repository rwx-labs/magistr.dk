use sqlx::FromRow;

pub struct NewQuote {
    pub date: String,
    pub text: String,
}

#[derive(Debug, FromRow, Clone)]
pub struct Quote {
    pub id: i32,
    pub date: String,
    pub text: String,
}
