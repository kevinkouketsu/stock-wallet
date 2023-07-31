use super::{AssetType, OnlineWallet, Ticker};
use crate::wallet::{Code, Event};
use futures::future::BoxFuture;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Trade {
    ticker_type: String,
    user_wallet_id: i32,

    #[serde(rename = "type")]
    trade_type: String,
    source: String,
    _token: String,
    date: String,
    qty: i32,
    ticker: i32,
    #[serde(with = "custom_f64")]
    price: f64,
    cost: f32,
}

mod custom_f64 {
    use serde::Serializer;
    pub fn serialize<S>(f: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:.2}", f);
        let s = format!("{}000000", s.replace('.', ","));
        serializer.serialize_str(&s)
    }
}

pub struct Investidor10Api {
    headers: HeaderMap,
    wallet_id: i32,
}
impl Investidor10Api {
    pub fn new(session: &str, wallet_id: i32) -> Self {
        let laravel_session = format!("laravel_session={}", session);
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "reqwest".parse().unwrap());
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Cookie", HeaderValue::from_str(&laravel_session).unwrap());

        Investidor10Api { headers, wallet_id }
    }

    async fn create_trade_request(&self, event: &Event) -> Result<Trade, Investidor10Error> {
        let ticker_info = self.get_ticker_id(event.code()).await?;
        let r#type = match event {
            Event::Sell(_, _) => "SELL",
            Event::Buy(_, _) => "BUY",
        };

        let ticker_type = match ticker_info.r#type {
            AssetType::Fii => "fii",
            AssetType::Ticker => "Ticker",
        };

        let trade = Trade {
            _token: String::new(),
            cost: 0f32,
            date: event.date().format("%d/%m/%Y").to_string(),
            price: event.price(),
            qty: event.amount(),
            source: "Manual".to_string(),
            ticker: ticker_info.id,
            ticker_type: ticker_type.to_string(),
            trade_type: r#type.to_string(),
            user_wallet_id: self.wallet_id,
        };

        Ok(trade)
    }

    async fn get_as_ticker(&self, ticker: &Code) -> Result<Ticker, Investidor10Error> {
        let url_ticker = format!(
            "https://investidor10.com.br/api/buscar/ticker/?_type=query&q={}",
            ticker
        );

        let client = reqwest::Client::new();
        let response = client
            .get(url_ticker)
            .headers(self.headers.clone())
            .send()
            .await?;

        let trade: Vec<TickerInfo> = serde_json::from_str(&response.text().await?)?;

        trade
            .first()
            .map(|x| Ticker {
                id: x.id,
                _name: x.name.clone(),
                r#type: AssetType::Ticker,
            })
            .ok_or(Investidor10Error::TickerNotFound(ticker.clone()))
    }

    async fn get_as_fii(&self, ticker: &Code) -> Result<Ticker, Investidor10Error> {
        let url_ticker = format!(
            "https://investidor10.com.br/api/buscar/fii/?_type=query&q={}",
            ticker
        );

        let client = reqwest::Client::new();
        let response = client
            .get(url_ticker)
            .headers(self.headers.clone())
            .send()
            .await?;

        serde_json::from_str::<Vec<TickerInfo>>(&response.text().await?)?
            .first()
            .map(|x| Ticker {
                id: x.id,
                _name: x.name.clone(),
                r#type: AssetType::Fii,
            })
            .ok_or(Investidor10Error::TickerNotFound(ticker.clone()))
    }
}
impl OnlineWallet for Investidor10Api {
    type Error = Investidor10Error;

    fn add_asset(&self, event: Event) -> BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            let trade = self.create_trade_request(&event).await?;
            let url = format!(
                "https://investidor10.com.br/api/minhas-carteiras/lancamentos/{}/",
                self.wallet_id
            );

            let client = reqwest::Client::new();
            let json = serde_json::to_string(&trade)?;
            let response = client
                .post(url)
                .headers(self.headers.clone())
                .body(json)
                .send()
                .await?;

            response.error_for_status()?;
            Ok(())
        })
    }

    fn get_ticker_id(&self, ticker: &Code) -> BoxFuture<'_, Result<Ticker, Self::Error>> {
        let ticker = ticker.clone();
        Box::pin(async move {
            self.get_as_ticker(&ticker)
                .await
                .or(self.get_as_fii(&ticker).await)
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Investidor10Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("Ticker {0} not found")]
    TickerNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerInfo {
    id: i32,
    name: String,
}