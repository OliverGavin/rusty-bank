extern crate rusty_bank;

use std::env;

use anyhow::Result;
use rusty_bank::{Config, CsvAccountWriter, CsvTransactionReader};

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args)?;
    let bank = RustyBank::new(config);
    bank.run()
}

pub struct RustyBank {
    config: Config,
}

impl RustyBank {
    fn new(config: Config) -> Self {
        RustyBank { config }
    }

    fn run(&self) -> Result<()> {
        log::debug!("config: {:?}", self.config);
        log::info!("processing...");

        // Example read
        let mut reader = CsvTransactionReader::from_path(&self.config.filename)?;
        for result in reader.read() {
            match result {
                Ok(transaction) => log::debug!("Processing `{:?}`", transaction),
                Err(err) => log::error!("Could not read transaction: {}", err),
            }
        }

        let accounts = vec![
            rusty_bank::AccountSummary::new(rusty_bank::ClientId(1), 0.into(), 5.into(), false),
            rusty_bank::AccountSummary::new(rusty_bank::ClientId(2), 0.into(), 20.into(), false),
        ];

        // Example write
        let mut writer = CsvAccountWriter::from_writer(std::io::stdout());
        for account in accounts {
            writer.write(&account)?;
        }
        Ok(())
    }
}
