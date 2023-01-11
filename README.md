# Stock Wallet

A command line tool written in Rust that allows you to track your stock portfolio by consuming a CSV file and calculating the average price, current position of a given stock.

## Installation

Clone the repository and navigate to the project directory.

```
git clone https://github.com/kevinkouketsu/stock-wallet.git
cd stock-wallet
cargo run < your_portfolio.csv
```

Make sure you have Rust and Cargo installed. Then, use cargo to install the dependencies and build the project

## Usage

The program can be run by passing the csv contents as an argument. The CSV file should have the following columns: "date", "ticket", "action", "shares", "price"

Here is an example of an input:

|31/10/2018 00:00:00|PETR4|B|10|28.20|
|31/01/2019 00:00:00|VALE3|B|4|46.95|
|04/04/2019 00:00:00|VALE3|S|4|52.35|
|13/05/2019 00:00:00|PETR4|B|9|26.07|
|10/06/2019 00:00:00|PETR4|S|19|26.85|
|17/06/2019 00:00:00|GRND3|B|71|7.37|
|10/07/2019 00:00:00|BIDI4|B|100|15.00|
|05/08/2019 00:00:00|GRND3|B|21|7.45|

There is no commandline available to iterate with. The only output possible is the current position for all the wallet.

# Purpose 

This is a project to help me calculate the average price for the Imposto de Renda, to notify the government about my current position, and also to practice using Rust
