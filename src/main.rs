extern crate rusty_bank;

use std::env;

use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    log::debug!("args: {:?}", args);

    let bank = RustyBank::new();
    bank.run()
}

pub struct RustyBank {}

impl RustyBank {
    fn new() -> Self {
        RustyBank {}
    }

    fn run(&self) -> Result<()> {
        log::info!("processing...");
        Ok(())
    }
}
