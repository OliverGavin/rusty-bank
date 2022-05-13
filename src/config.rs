//! Argument parsing for Rusty Bank.

use anyhow::{bail, Result};

/// Represents the arguments passed via the command line.
#[derive(Debug, PartialEq)]
pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config> {
        match args.len() {
            // empty args...
            0 => {
                unreachable!();
            }
            // no parameters passed
            1 => {
                bail!("Usage: {} filename", args[0]);
            }
            // one parameter passed
            2 => Ok(Config {
                filename: args[1].clone(),
            }),
            // more than one parameter passed
            _ => {
                bail!("Only one parameter allowed. Got: {:?}", &args[1..]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;

    #[test]
    #[should_panic(expected = "internal error: entered unreachable code")]
    fn test_new_panics_when_empty_args() {
        Config::new(&[]).unwrap_err();
    }

    #[test]
    fn test_new_returns_err_when_no_parameter() {
        let result = Config::new(&["./path/to/executable".to_string()]).unwrap_err();
        let expected = anyhow!("Usage: ./path/to/executable filename");
        assert_eq!(expected.to_string(), result.to_string());
    }

    #[test]
    fn test_new_returns_ok_when_one_parameter() {
        let result: Config =
            Config::new(&["./path/to/executable".to_string(), "some.csv".to_string()]).unwrap();
        let expected: Config = Config {
            filename: "some.csv".to_string(),
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_new_returns_err_when_more_than_one_parameter() {
        let result =
            Config::new(&["executable".to_string(), "a".to_string(), "b".to_string()]).unwrap_err();
        let expected = anyhow!(r#"Only one parameter allowed. Got: ["a", "b"]"#);
        assert_eq!(expected.to_string(), result.to_string());
    }
}
