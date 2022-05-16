//! Serdes for accounts

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::client::ClientId;

/// State of a client's account
#[derive(Debug, Deserialize, Serialize)]
pub struct AccountSummary {
    client: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl AccountSummary {
    // Create an account
    pub fn new(client: ClientId, held: Decimal, total: Decimal, locked: bool) -> Self {
        AccountSummary {
            client,
            // `available` is derivable from `total` and `held` and as such does not need to exist.
            // for simplicity in serialization it is kept.
            available: total - held,
            held,
            total,
            locked,
        }
    }

    // Create an empty account with a balance of zero
    pub fn empty(client: ClientId) -> Self {
        AccountSummary::new(client, 0.into(), 0.into(), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;
    use rust_decimal_macros::dec;

    #[test]
    fn test_serde_when_valid_csv() -> Result<()> {
        let expected = "\
            client,available,held,total,locked\n\
            1,1.9999,1,0.9999,false\n\
        ";

        // Prepare an in-memory reader/writer
        let mut rdr = csv::Reader::from_reader(expected.as_bytes());
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Deserialize and re-serialize each account record
        for res in rdr.deserialize() {
            let account: AccountSummary = res?;
            wtr.serialize(account)?;
        }

        // Compare the input and output
        let result = String::from_utf8(wtr.into_inner()?)?;
        assert_eq!(expected.to_string(), result);

        Ok(())
    }

    #[test]
    fn test_computes_available_with_correct_precision_when_serialized() -> Result<()> {
        let expected = "\
            client,available,held,total,locked\n\
            1,1.9999,1.0,2.9999,false\n\
        ";

        // Prepare an in-memory writer
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Serialize the account
        let account: AccountSummary = AccountSummary::new(ClientId(1), dec!(1.0), dec!(2.9999), false);
        wtr.serialize(account)?;

        // Compare the result against the expected output
        let result = String::from_utf8(wtr.into_inner()?)?;
        assert_eq!(expected.to_string(), result);

        Ok(())
    }

    #[test]
    fn test_new_returns_account_with_computed_available_funds() {
        let account = AccountSummary::new(ClientId(1), dec!(5), dec!(15), false);
        assert_eq!(ClientId(1), account.client);
        assert_eq!(dec!(10), account.available);
        assert_eq!(dec!(5), account.held);
        assert_eq!(dec!(15), account.total);
        assert_eq!(false, account.locked);
    }

    #[test]
    fn test_empty_returns_unlocked_account_with_no_funds() {
        let account = AccountSummary::empty(ClientId(1));
        assert_eq!(ClientId(1), account.client);
        assert_eq!(dec!(0), account.available);
        assert_eq!(dec!(0), account.held);
        assert_eq!(dec!(0), account.total);
        assert_eq!(false, account.locked);
    }
}
