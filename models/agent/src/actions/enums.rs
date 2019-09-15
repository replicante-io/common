use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Entity (system, user, ...) that requested the action to be performed.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActionRequester {
    Api,
}

/// Current state of an action execution.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActionState {
    /// The action is to be cancelled, but that has not happened yet.
    Cancel,

    /// The action was successfully cancelled.
    Cancelled,

    /// The action was successfully completed.
    Done,

    /// The action ended with an error.
    Failed,

    /// The action has just been sheduled and is not being executed yet.
    New,

    /// The action was started by the agent and is in progress.
    Running,
}

impl ActionState {
    /// True if the action is finished (failed or succeeded).
    pub fn is_finished(&self) -> bool {
        match self {
            Self::Cancelled => true,
            Self::Done => true,
            Self::Failed => true,
            _ => false,
        }
    }
}
