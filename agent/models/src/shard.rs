/// Information about the current commit offset of a shard or replication lag.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct CommitOffset {
    pub unit: CommitUnit,
    pub value: i64,
}

impl CommitOffset {
    pub fn new(value: i64, unit: CommitUnit) -> CommitOffset {
        CommitOffset { unit, value }
    }

    pub fn seconds(value: i64) -> CommitOffset {
        CommitOffset::new(value, CommitUnit::seconds())
    }

    pub fn unit<S: Into<String>>(value: i64, unit: S) -> CommitOffset {
        CommitOffset::new(value, CommitUnit::unit(unit))
    }
}

/// Unit of commit offsets or replica lags.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum CommitUnit {
    #[serde(rename = "seconds")]
    Seconds,

    #[serde(rename = "unit")]
    Unit(String),
}

impl CommitUnit {
    pub fn seconds() -> CommitUnit {
        CommitUnit::Seconds
    }

    pub fn unit<S: Into<String>>(unit: S) -> CommitUnit {
        CommitUnit::Unit(unit.into())
    }
}

/// Information about a shard on a node.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Shard {
    pub commit_offset: Option<CommitOffset>,
    pub id: String,
    pub lag: Option<CommitOffset>,
    pub role: ShardRole,
}

impl Shard {
    pub fn new<S>(
        id: S,
        role: ShardRole,
        commit_offset: Option<CommitOffset>,
        lag: Option<CommitOffset>,
    ) -> Shard
    where
        S: Into<String>,
    {
        Shard {
            id: id.into(),
            commit_offset,
            lag,
            role,
        }
    }
}

/// Information about shards on a node.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Shards {
    pub shards: Vec<Shard>,
}

impl Shards {
    pub fn new(shards: Vec<Shard>) -> Shards {
        Shards { shards }
    }
}

/// Possible shard roles.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ShardRole {
    #[serde(rename = "primary")]
    Primary,

    #[serde(rename = "secondary")]
    Secondary,

    #[serde(rename = "unknown")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::CommitOffset;
    use super::Shard;
    use super::ShardRole;

    #[test]
    fn primary_from_json() {
        let payload = concat!(
            r#"{"commit_offset":{"unit":"seconds","value":12345},"id":"shard-1","#,
            r#""lag":{"unit":"seconds","value":0},"role":"primary"}"#
        );
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new(
            "shard-1",
            ShardRole::Primary,
            Some(CommitOffset::seconds(12345)),
            Some(CommitOffset::seconds(0)),
        );
        assert_eq!(shard, expected);
    }

    #[test]
    fn primary_to_json() {
        let shard = Shard::new(
            "shard-1",
            ShardRole::Primary,
            Some(CommitOffset::seconds(12345)),
            Some(CommitOffset::seconds(0)),
        );
        let payload = serde_json::to_string(&shard).unwrap();
        let expected = concat!(
            r#"{"commit_offset":{"unit":"seconds","value":12345},"id":"shard-1","#,
            r#""lag":{"unit":"seconds","value":0},"role":"primary"}"#
        );
        assert_eq!(payload, expected);
    }

    #[test]
    fn unkown_from_json() {
        let payload = concat!(
            r#"{"commit_offset":{"unit":"seconds","value":12345},"id":"shard-1","#,
            r#""lag":{"unit":"seconds","value":0},"role":{"unknown":"Test"}}"#
        );
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new(
            "shard-1",
            ShardRole::Unknown(String::from("Test")),
            Some(CommitOffset::seconds(12345)),
            Some(CommitOffset::seconds(0)),
        );
        assert_eq!(shard, expected);
    }

    #[test]
    fn unkown_to_json() {
        let shard = Shard::new(
            "shard-1",
            ShardRole::Unknown(String::from("Test")),
            Some(CommitOffset::seconds(12345)),
            Some(CommitOffset::seconds(0)),
        );
        let payload = serde_json::to_string(&shard).unwrap();
        let expected = concat!(
            r#"{"commit_offset":{"unit":"seconds","value":12345},"id":"shard-1","#,
            r#""lag":{"unit":"seconds","value":0},"role":{"unknown":"Test"}}"#
        );
        assert_eq!(payload, expected);
    }

    #[test]
    fn missing_commit_offset_from_json() {
        let payload = concat!(
            r#"{"id":"shard-1","#,
            r#""lag":{"unit":{"unit":"offset"},"value":0},"role":{"unknown":"Test"}}"#
        );
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new(
            "shard-1",
            ShardRole::Unknown(String::from("Test")),
            None,
            Some(CommitOffset::unit(0, "offset")),
        );
        assert_eq!(shard, expected);
    }

    #[test]
    fn missing_lag_from_json() {
        let payload = concat!(
            r#"{"commit_offset":{"unit":"seconds","value":12345},"id":"shard-1","#,
            r#""role":{"unknown":"Test"}}"#
        );
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new(
            "shard-1",
            ShardRole::Unknown(String::from("Test")),
            Some(CommitOffset::seconds(12345)),
            None,
        );
        assert_eq!(shard, expected);
    }

    #[test]
    fn no_commit_offset_from_json() {
        let payload = concat!(
            r#"{"commit_offset":null,"id":"shard-1","#,
            r#""lag":{"unit":"seconds","value":0},"role":{"unknown":"Test"}}"#
        );
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new(
            "shard-1",
            ShardRole::Unknown(String::from("Test")),
            None,
            Some(CommitOffset::seconds(0)),
        );
        assert_eq!(shard, expected);
    }

    #[test]
    fn no_lag_from_json() {
        let payload = concat!(
            r#"{"commit_offset":{"unit":"seconds","value":12345},"id":"shard-1","#,
            r#""lag":null,"role":{"unknown":"Test"}}"#
        );
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new(
            "shard-1",
            ShardRole::Unknown(String::from("Test")),
            Some(CommitOffset::seconds(12345)),
            None,
        );
        assert_eq!(shard, expected);
    }

    #[test]
    fn no_commit_offset_to_json() {
        let shard = Shard::new(
            "shard-1",
            ShardRole::Primary,
            None,
            Some(CommitOffset::seconds(0)),
        );
        let payload = serde_json::to_string(&shard).unwrap();
        let expected = concat!(
            r#"{"commit_offset":null,"id":"shard-1","#,
            r#""lag":{"unit":"seconds","value":0},"role":"primary"}"#
        );
        assert_eq!(payload, expected);
    }

    #[test]
    fn no_lag_to_json() {
        let shard = Shard::new(
            "shard-1",
            ShardRole::Primary,
            Some(CommitOffset::seconds(12345)),
            None,
        );
        let payload = serde_json::to_string(&shard).unwrap();
        let expected = concat!(
            r#"{"commit_offset":{"unit":"seconds","value":12345},"id":"shard-1","#,
            r#""lag":null,"role":"primary"}"#
        );
        assert_eq!(payload, expected);
    }
}
