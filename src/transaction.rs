//! Serdes for transactions

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::client::ClientId;

/// Supported transaction types
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Represents a transaction ID as it's own type
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionId(pub u32);

/// Record of a transaction
//  Ideally `Transaction` would be an enum and `TransactionType` would not need to exist
//    and `Transaction::amount` would only exist for deposit/withdrawal variants.
//  However, in rust-csv internally-tagged enums are not supported:
//    https://github.com/BurntSushi/rust-csv/issues/211
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Option<Decimal>,
}

impl Transaction {
    // Create a transaction
    pub fn new(
        transaction_type: TransactionType,
        client: ClientId,
        tx: TransactionId,
        amount: Option<Decimal>,
    ) -> Self {
        Transaction {
            transaction_type,
            client,
            tx,
            amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;
    use csv::{Reader, ReaderBuilder, Trim, Writer};
    use test_case::test_case;

    #[test]
    fn test_serde_when_valid_csv() -> Result<()> {
        let expected = "\
            type,client,tx,amount\n\
            deposit,1,1,10\n\
            withdrawal,1,2,1.9999\n\
            dispute,1,2,\n\
            resolve,1,2,\n\
            withdrawal,1,3,1\n\
            dispute,1,3,\n\
            chargeback,1,3,\n\
        ";

        // Prepare an in-memory reader/writer
        let mut rdr = Reader::from_reader(expected.as_bytes());
        let mut wtr = Writer::from_writer(vec![]);

        // Deserialize and re-serialize each transaction record
        for res in rdr.deserialize() {
            let transaction: Transaction = res?;
            wtr.serialize(transaction)?;
        }

        // Compare the input and output
        let result = String::from_utf8(wtr.into_inner()?)?;
        assert_eq!(expected.to_string(), result);

        Ok(())
    }

    #[test_case(",         1,  1, 10"; "when missing transaction type")]
    #[test_case("borrow,   1,  1, 10"; "when unknown transaction type")]
    #[test_case("deposit,   ,  1, 10"; "when missing client ID")]
    #[test_case("deposit, -1,  1, 10"; "when negative client ID")]
    #[test_case("deposit,  1,   , 10"; "when missing transaction ID")]
    #[test_case("deposit,  1, -1, 10"; "when negative transaction ID")]
    #[should_panic]
    fn test_serde_when_invalid_csv(expected: &str) {
        let expected = format!(
            "\
            type,client,tx,amount\n\
            {}\n\
        ",
            expected
        );

        // Prepare an in-memory reader
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(expected.as_bytes());

        // Deserialize each transaction record
        for res in rdr.deserialize() {
            let _: Transaction = res.unwrap();
        }
    }
}
