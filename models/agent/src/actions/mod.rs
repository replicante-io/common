use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value as Json;
use uuid::Uuid;

pub mod api;
mod enums;

pub use self::enums::ActionRequester;
pub use self::enums::ActionState;

/// Transition history records for actions.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionHistoryItem {
    /// ID of the action that transitioned.
    pub action_id: Uuid,

    /// Time the agent transitioned into this state.
    pub timestamp: DateTime<Utc>,

    /// State the action is currently in.
    pub state: ActionState,

    /// Optional payload attached to the current state.
    pub state_payload: Option<Json>,
}

/// Summary info about an action returned in lists.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ActionListItem {
    pub id: Uuid,
    pub kind: String,
    pub state: ActionState,
}

/// Action state and metadata information.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionModel {
    /// Arguments passed to the action when invoked.
    pub args: Json,

    /// Time the agent recorded the action in the DB.
    pub created_ts: DateTime<Utc>,

    /// Time the action entered a finished state.
    pub finished_ts: Option<DateTime<Utc>>,

    /// Additional metadata headers attached to the action.
    pub headers: HashMap<String, String>,

    /// Unique ID of the action.
    pub id: Uuid,

    /// Type ID of the action to run.
    pub kind: String,

    /// Entity (system or user) requesting the execution of the action.
    pub requester: ActionRequester,

    /// State the action is currently in.
    pub state: ActionState,

    /// Optional payload attached to the current state.
    pub state_payload: Option<Json>,
}
