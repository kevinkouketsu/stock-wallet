use crate::wallet::{Code, Event};
use futures::future::BoxFuture;

pub mod investidor10;

#[derive(Debug, Clone)]
pub enum AssetType {
    Fii,
    Ticker,
}

#[derive(Debug, Clone)]
pub struct Ticker {
    id: i32,
    _name: String,
    r#type: AssetType
}

pub trait OnlineWallet {
    type Error: std::error::Error;

    fn add_asset(&self, event: Event) -> BoxFuture<'_, Result<(), Self::Error>>;
    fn get_ticker_id(&self, ticker: &Code) -> BoxFuture<'_, Result<Ticker, Self::Error>>;
}
