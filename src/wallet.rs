use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::{hash_map::Entry, HashMap};

pub type Code = String;
pub type Currency = f64;

#[derive(Default, Debug, Deserialize)]
pub struct TransactionInfo {
    date: DateTime<Utc>,
    amount: i32,
    price: Currency,
}
impl TransactionInfo {
    pub fn new(date: DateTime<Utc>, amount: i32, price: Currency) -> Self {
        Self {
            date,
            amount,
            price,
        }
    }
    pub fn date(&self) -> DateTime<Utc> {
        self.date
    }
    pub fn amount(&self) -> i32 {
        self.amount
    }
    pub fn price(&self) -> Currency {
        self.price
    }
}

#[derive(Debug, Deserialize)]
pub enum Event {
    Sell(Code, TransactionInfo),
    Buy(Code, TransactionInfo),
}

impl Event {
    pub fn code(&self) -> &str {
        match self {
            Event::Sell(code, _) => code,
            Event::Buy(code, _) => code,
        }
    }

    pub fn amount(&self) -> i32 {
        match self {
            Event::Sell(_, transaction) => transaction.amount,
            Event::Buy(_, transaction) => transaction.amount,
        }
    }
    pub fn price(&self) -> f64 {
        match self {
            Event::Sell(_, transaction) => transaction.price,
            Event::Buy(_, transaction) => transaction.price,
        }
    }
}

pub type Transactions = HashMap<Code, Vec<Event>>;
pub struct Wallet {
    transactions: Transactions,
}
impl Wallet {
    pub fn from_transactions<I: IntoIterator<Item = Event>>(transactions: I) -> Wallet {
        let mut hashmap: Transactions = HashMap::new();

        transactions.into_iter().for_each(|x| {
            match hashmap.entry(x.code().to_owned()) {
                Entry::Occupied(mut entry) => entry.get_mut().push(x),
                Entry::Vacant(vacant) => {
                    vacant.insert(vec![x]);
                }
            };
        });

        Self {
            transactions: hashmap,
        }
    }

    pub fn ticker<'a>(&'a self, key: &str) -> Option<WalletTicker<'a>> {
        self.transactions
            .get_key_value(key)
            .map(|value| WalletTicker::new(value.0, value.1))
    }

    pub fn wealth(&self) -> impl Iterator<Item = WalletTicker<'_>> {
        self.transactions
            .iter()
            .map(|x| WalletTicker::new(x.0, x.1))
            .filter(|x| x.position().is_some())
    }
}

#[derive(Debug)]
pub struct WalletTicker<'a> {
    name: &'a str,
    events: &'a [Event],
}

impl<'a> WalletTicker<'a> {
    pub fn new(name: &'a str, events: &'a [Event]) -> Self {
        Self { name, events }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn average_price(&self) -> f64 {
        let calculated = self
            .events
            .iter()
            .filter(|x| !matches!(x, Event::Sell(_, _)))
            .fold((0.0, 0.0), |accumulated, item| {
                (
                    accumulated.0 + item.amount() as f64,
                    accumulated.1 + item.amount() as f64 * item.price(),
                )
            });

        calculated.1 / calculated.0
    }

    pub fn position(&self) -> Option<Position> {
        let amount = self.events.iter().fold(0, |accumulated, item| match item {
            Event::Sell(_, transaction_info) => accumulated - transaction_info.amount,
            Event::Buy(_, transaction_info) => accumulated + transaction_info.amount,
        });

        match amount {
            1.. => Some(Position {
                _code: self.name(),
                _average_price: self.average_price(),
                _current_amount: amount,
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Position<'a> {
    _code: &'a str,
    _current_amount: i32,
    _average_price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATE: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;

    pub fn create_actions() -> Vec<Event> {
        vec![
            Event::Buy("PETR4".to_owned(), TransactionInfo::new(DATE, 200, 14.0)),
            Event::Buy("PETR4".to_owned(), TransactionInfo::new(DATE, 300, 15.0)),
            Event::Buy("PETR4".to_owned(), TransactionInfo::new(DATE, 400, 16.0)),
        ]
    }

    #[test]
    fn it_is_able_to_convert_into_a_position() {
        let position = Wallet::from_transactions(create_actions());

        assert_eq!(position.transactions.values().len(), 1);
        assert_eq!(position.transactions.get("PETR4").unwrap().len(), 3);
    }

    #[test]
    fn it_can_calculate_the_average_price_for_a_ticker() {
        let position = Wallet::from_transactions(create_actions());
        assert!((position.ticker("PETR4").unwrap().average_price().abs() - 15.22).abs() < 0.1);
    }

    #[test]
    fn it_will_return_none_if_there_is_no_transaction_regarding_a_key() {
        let position = Wallet::from_transactions(create_actions());
        matches!(position.ticker("nonexistent"), None);
    }

    #[test]
    fn it_should_ignore_sells_when_calculating_the_avg_price() {
        let actions = vec![
            Event::Buy("BBAS3".to_owned(), TransactionInfo::new(DATE, 100, 20.0)),
            Event::Buy("BBAS3".to_owned(), TransactionInfo::new(DATE, 100, 25.0)),
            Event::Sell("BBAS3".to_owned(), TransactionInfo::new(DATE, 50, 20.0)),
        ];

        let ticker_position = WalletTicker::new("BBAS3", &actions);
        assert!((ticker_position.average_price().abs() - 22.5).abs() < 0.1);
    }

    #[test]
    fn it_will_return_the_current_position_according_to_the_buys_and_sells() {
        let actions = vec![
            Event::Buy("BBAS3".to_owned(), TransactionInfo::new(DATE, 100, 20.0)),
            Event::Buy("BBAS3".to_owned(), TransactionInfo::new(DATE, 100, 25.0)),
            Event::Sell("BBAS3".to_owned(), TransactionInfo::new(DATE, 100, 20.0)),
        ];

        let ticker_position = WalletTicker::new("BBAS3", &actions);
        let position = ticker_position.position();

        assert!(position.is_some());
        assert_eq!(position.unwrap()._current_amount, 100);
    }

    #[test]
    fn it_will_return_none_if_all_the_stocks_were_sold() {
        let actions = vec![
            Event::Buy("BBAS3".to_owned(), TransactionInfo::new(DATE, 100, 20.0)),
            Event::Buy("BBAS3".to_owned(), TransactionInfo::new(DATE, 100, 25.0)),
            Event::Sell("BBAS3".to_owned(), TransactionInfo::new(DATE, 200, 20.0)),
        ];

        let ticker_position = WalletTicker::new("BBAS3", &actions);
        let position = ticker_position.position();

        assert!(position.is_none());
    }
}
