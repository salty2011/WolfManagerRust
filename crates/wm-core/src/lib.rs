use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, ToSchema)]
pub struct UserId(#[schema(value_type = String)] pub Uuid);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, ToSchema)]
pub struct ClientId(#[schema(value_type = String)] pub Uuid);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, ToSchema)]
pub struct PairingId(#[schema(value_type = String)] pub Uuid);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, ToSchema)]
pub struct SessionId(#[schema(value_type = String)] pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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