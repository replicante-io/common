use chrono::DateTime;
use chrono::Utc;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use uuid::Uuid;

use super::ActionHistoryItem;
use super::ActionModel;
use super::ActionRequester;

/// Action information returned by the API.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionInfoResponse {
    pub action: ActionModel,
    pub history: Vec<ActionHistoryItem>,
}

/// Parameters passed to the action scheduling API.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionScheduleRequest {
    /// Optional pre-generated action ID.
    ///
    /// A new UUID will be generated if one is not provided.
    #[serde(default)]
    pub action_id: Option<Uuid>,

    /// Additional arguments for use by the action.
    #[serde(default = "ActionScheduleRequest::default_args")]
    pub args: Value,

    /// Optional time at which the action was created (by replicore or other system).
    #[serde(default)]
    pub created_ts: Option<DateTime<Utc>>,

    /// Optional action requester to propagate.
    #[serde(default)]
    pub requester: Option<ActionRequester>,
}

impl ActionScheduleRequest {
    fn default_args() -> Value {
        Value::Null
    }
}

impl Default for ActionScheduleRequest {
    fn default() -> Self {
        Self {
            action_id: None,
            args: Self::default_args(),
            created_ts: None,
            requester: None,
        }
    }
}
