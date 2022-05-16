//! Serdes for clients

use serde::{Deserialize, Serialize};

/// Represents a client ID as it's own type
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClientId(pub u16);
