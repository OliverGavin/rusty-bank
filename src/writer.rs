use anyhow::{Error, Result};
use csv::{Writer, WriterBuilder};

use crate::AccountSummary;

/// A trait for any account writer implementation.
#[cfg_attr(test, mockall::automock)]
pub trait AccountWriter {
    // Writes an account
    fn write(&mut self, account: &AccountSummary) -> Result<()>;
}

/// Account writer for CSV files
//  anyhow::Error requires Send + Sync + 'static
pub struct CsvAccountWriter<W>
where
    W: std::io::Write + Send + Sync + 'static,
{
    writer: Option<Writer<W>>,
}

impl<W> CsvAccountWriter<W>
where
    W: std::io::Write + Send + Sync + 'static,
{
    /// Returns an account CSV writer that writes data to wtr.
    pub fn from_writer(wtr: W) -> Self {
        let writer = WriterBuilder::new().has_headers(true).from_writer(wtr);
        CsvAccountWriter {
            writer: Some(writer),
        }
    }

    /// Flush the contents of the internal buffer and return the underlying writer.
    pub fn into_inner(mut self) -> Result<W> {
        self.writer
            .take()
            .unwrap()
            .into_inner()
            .map_err(Error::from)
    }
}

impl<W> AccountWriter for CsvAccountWriter<W>
where
    W: std::io::Write + Send + Sync + 'static,
{
    /// Serializes and writes an account
    fn write(&mut self, account: &AccountSummary) -> Result<()> {
        match self.writer.as_mut() {
            Some(wtr) => wtr.serialize(account).map_err(Error::from),
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ClientId;

    use super::*;

    #[test]
    fn test_write() -> Result<()> {
        let mut wtr = CsvAccountWriter::from_writer(vec![]);

        let accounts = vec![
            AccountSummary::new(ClientId(1), 0.into(), 50.into(), false),
            AccountSummary::new(ClientId(2), 10.into(), 40.into(), false),
        ];

        for account in accounts {
            wtr.write(&account)?;
        }

        // Compare the input and output
        let result = String::from_utf8(wtr.into_inner()?)?;
        let expected = "\
            client,available,held,total,locked\n\
            1,50,0,50,false\n\
            2,30,10,40,false\n\
        ";
        assert_eq!(expected.to_string(), result);

        Ok(())
    }
}
