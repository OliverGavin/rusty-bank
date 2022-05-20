//! Serdes for clients

use serde::{Deserialize, Serialize};

/// Represents a client ID as it's own type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ClientId(pub u16);
