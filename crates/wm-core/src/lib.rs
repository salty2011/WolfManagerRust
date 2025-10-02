use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

// Domain ID types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ClientId(pub Uuid);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct PairingId(pub Uuid);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SessionId(pub Uuid);

// Domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    ClientConnected { client_id: ClientId, at: OffsetDateTime },
    ClientDisconnected { client_id: ClientId, at: OffsetDateTime },
    PairingCreated { pairing_id: PairingId, at: OffsetDateTime },
    SessionStarted { session_id: SessionId, at: OffsetDateTime },
    SessionEnded { session_id: SessionId, at: OffsetDateTime },
}

pub trait Normalize {
    fn normalize(self) -> Vec<Event>;
}