use chrono::{DateTime, Utc};
use online::{investidor10::{Investidor10Api}, OnlineWallet};
use serde::Deserialize;
use std::error::Error;
use wallet::{Currency, Event, TransactionInfo, Wallet};

pub mod online;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //["]([\d]+),([\d]+)["]
    let entries = import_csv_to_entries(std::io::stdin()).unwrap();
    let position = Wallet::from_transactions(entries);

    let investidor10 = Investidor10Api::new("eyJpdiI6Im54cDVZekJlYU1BdGppaXMvNjZmV0E9PSIsInZhbHVlIjoiMEFETmwxUFRhMnJyZ2RtK0Y2dU9tZ3hpYjNJekV2SlJJUFhjTHdpZm5wSzJzQW9qOHVsRWdGYnllRUNzM0tSbXEwUnk1V1FRcE4zL0RkNlV5QmFKb2FacVUrRk9EOFk4OUJTQm9hV2JnZUsrR3hLaVBnSHJWQTZSRGlhc2RmdEsiLCJtYWMiOiI0ZjcwMGQwMjgwYTVmOGRkNzQ2NzBkNzNhODE5YmE5Y2JkYzQxOWJmZTgzZTMzZDk2ZTUwZmI5N2RjYTI2OGNjIn0%3D", 194632);
    for ticker in position.wealth() {
        for event in ticker.events() {
            if (investidor10.add_asset(event.clone()).await).is_err() {
                println!("{:?} failed to be added", event);
            } else {
                println!("{:?} added with success", event);
            }
        }
    }

    Ok(())
}
