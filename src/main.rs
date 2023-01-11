use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::error::Error;
use wallet::{Currency, Event, TransactionInfo, Wallet};

pub mod stock;
pub mod wallet;

#[derive(Debug, Deserialize)]
enum ActionEntry {
    #[serde(rename = "S")]
    Sell,
    #[serde(rename = "B")]
    Buy,
}

#[derive(Debug, Deserialize)]
struct CsvEntry {
    #[serde(with = "custom_date_time")]
    date: DateTime<Utc>,
    code: String,
    action: ActionEntry,
    amount: i32,
    price: Currency,
}

mod custom_date_time {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Deserializer};
    const FORMAT: &str = "%d/%m/%Y %H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

impl From<CsvEntry> for Event {
    fn from(val: CsvEntry) -> Self {
        match val.action {
            ActionEntry::Sell => Event::Sell(
                val.code,
                TransactionInfo::new(val.date, val.amount, val.price),
            ),
            ActionEntry::Buy => Event::Buy(
                val.code,
                TransactionInfo::new(val.date, val.amount, val.price),
            ),
        }
    }
}

fn import_csv_to_entries<R: std::io::Read>(reader: R) -> Result<Vec<Event>, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .double_quote(false)
        .flexible(true)
        .from_reader(reader);

    let mut csv_entries = vec![];
    for result in rdr.deserialize() {
        let record: CsvEntry = result?;
        csv_entries.push(record.into());
    }

    Ok(csv_entries)
}

fn main() {
    let entries = import_csv_to_entries(std::io::stdin()).unwrap();
    let position = Wallet::from_transactions(entries);

    for ticker in position.wealth() {
        println!(
            "{} {}, {:?}",
            ticker.name(),
            ticker.average_price(),
            ticker.position()
        );
    }
}
