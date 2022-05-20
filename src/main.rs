extern crate rusty_bank;

use std::env;

use anyhow::Result;
use rusty_bank::{Config, CsvAccountWriter, CsvTransactionReader, TransactionProcessor, InMemoryAccountStore};

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
        let store = InMemoryAccountStore::new();
        let reader = CsvTransactionReader::from_path(&self.config.filename)?;
        let writer = CsvAccountWriter::from_writer(std::io::stdout());
        let mut processor = TransactionProcessor::new(store);
        processor.process(reader);
        processor.export(writer)?;
        Ok(())
    }
}
