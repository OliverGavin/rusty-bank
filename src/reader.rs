use std::{fs::File, path::Path};

use anyhow::{Error, Result};
use csv::{ReaderBuilder, Trim};

use crate::TransactionRecord;

/// A trait for any transaction reader implementation.
#[cfg_attr(test, mockall::automock)]
pub trait TransactionReader {
    /// Returns an iterator over [`Transaction`] records.
    fn read<'a>(&'a mut self) -> Box<dyn Iterator<Item = Result<TransactionRecord>> + 'a>;
}

/// Transaction reader for CSV files.
pub struct CsvTransactionReader {
    reader: csv::Reader<File>,
}

impl CsvTransactionReader {
    /// Create a new CSV reader for the given file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path: &Path = path.as_ref();
        let reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;
        Ok(CsvTransactionReader { reader })
    }
}

impl TransactionReader for CsvTransactionReader {
    /// Returns an iterator over deserialized [`Transaction`] records.
    fn read<'a>(&'a mut self) -> Box<dyn Iterator<Item = Result<TransactionRecord>> + 'a> {
        Box::new(
            self.reader
                .deserialize()
                .map(|result| result.map_err(Error::from))
        )
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;
    use test_case::test_case;

    use crate::ClientId;
    use crate::TransactionId;
    use crate::TransactionType;

    use super::*;

    #[test]
    fn test_read() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "type,client,tx,amount\n")?;
        writeln!(file, "deposit,1,1,10\n")?;
        writeln!(file, "withdrawal,1,2,5\n")?;

        let path = NamedTempFile::into_temp_path(file);
        let mut rdr = CsvTransactionReader::from_path(path)?;

        let mut transactions = Vec::new();
        for res in rdr.read() {
            let transaction = res?;
            transactions.push(transaction);
        }

        assert_eq!(
            vec![
                TransactionRecord::new(
                    TransactionType::Deposit,
                    ClientId(1),
                    TransactionId(1),
                    Some(10.into())
                ),
                TransactionRecord::new(
                    TransactionType::Withdrawal,
                    ClientId(1),
                    TransactionId(2),
                    Some(5.into())
                ),
            ],
            transactions
        );

        Ok(())
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_from_path_when_no_such_file() {
        CsvTransactionReader::from_path("some_file_path").unwrap();
    }

    #[test_case("invalid,client,tx,amount", "deposit,1,1,10"; "when invalid header")]
    #[test_case("type,client,tx,amount",    "borrow,1,1,10";  "when invalid type")]
    #[should_panic(expected = "CSV deserialize error")]
    fn test_read_failure_when_invalid_record(header: &str, line: &str) {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}\n{}\n", header, line).unwrap();

        let path = NamedTempFile::into_temp_path(file);
        let mut rdr = CsvTransactionReader::from_path(path).unwrap();

        for res in rdr.read() {
            res.unwrap();
        }
    }
}
