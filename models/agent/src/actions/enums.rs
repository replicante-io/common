use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Entity (system, user, ...) that requested the action to be performed.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ActionRequester {
    /// The action was requested over the API.
    #[serde(rename = "API")]
    Api,
}

/// Current state of an action execution.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ActionState {
    /// The action was successfully completed.
    #[serde(rename = "DONE")]
    Done,

    /// The action ended with an error.
    #[serde(rename = "FAILED")]
    Failed,

    /// The action has just been sheduled and is not being executed yet.
    #[serde(rename = "NEW")]
    New,

    /// The action was started by the agent and is in progress.
    #[serde(rename = "RUNNING")]
    Running,
}

impl ActionState {
    /// True if the action is finished (failed or succeeded).
    pub fn is_finished(&self) -> bool {
        match self {
            ActionState::Done => true,
            ActionState::Failed => true,
            _ => false,
        }
    }
}
