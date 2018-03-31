/// Information about a shard on a node.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Shard {
    pub id: String,
    pub role: ShardRole,
    pub lag: Option<i64>,
    pub last_op: i64,
}

impl Shard {
    pub fn new<S>(id: S, role: ShardRole, lag: Option<i64>, last_op: i64) -> Shard
        where S: Into<String>,
    {
        Shard {
            id: id.into(),
            role, lag, last_op,
        }
    }
}


/// Possible shard roles.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum ShardRole {
    Primary,
    Secondary,
    Unknown(String)
}


#[cfg(test)]
mod tests {
    use serde_json;
    use super::Shard;
    use super::ShardRole;

    #[test]
    fn primary_from_json() {
        let payload = r#"{"id":"shard-1","role":"Primary","lag":0,"last_op":12345}"#;
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new("shard-1", ShardRole::Primary, Some(0), 12345);
        assert_eq!(shard, expected);
    }

    #[test]
    fn primary_to_json() {
        let shard = Shard::new("shard-1", ShardRole::Primary, Some(0), 12345);
        let payload = serde_json::to_string(&shard).unwrap();
        let expected = r#"{"id":"shard-1","role":"Primary","lag":0,"last_op":12345}"#;
        assert_eq!(payload, expected);
    }

    #[test]
    fn unkown_from_json() {
        let payload = r#"{"id":"shard-1","role":{"Unknown":"Test"},"lag":0,"last_op":12345}"#;
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new(
            "shard-1", ShardRole::Unknown(String::from("Test")), Some(0), 12345
        );
        assert_eq!(shard, expected);
    }

    #[test]
    fn unkown_to_json() {
        let shard = Shard::new(
            "shard-1", ShardRole::Unknown(String::from("Test")), Some(0), 12345
        );
        let payload = serde_json::to_string(&shard).unwrap();
        let expected = r#"{"id":"shard-1","role":{"Unknown":"Test"},"lag":0,"last_op":12345}"#;
        assert_eq!(payload, expected);
    }

    #[test]
    fn missing_lag_from_json() {
        let payload = r#"{"id":"shard-1","role":"Secondary","last_op":12345}"#;
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new("shard-1", ShardRole::Secondary, None, 12345);
        assert_eq!(shard, expected);
    }

    #[test]
    fn no_lag_from_json() {
        let payload = r#"{"id":"shard-1","role":"Secondary","lag":null,"last_op":12345}"#;
        let shard: Shard = serde_json::from_str(payload).unwrap();
        let expected = Shard::new("shard-1", ShardRole::Secondary, None, 12345);
        assert_eq!(shard, expected);
    }

    #[test]
    fn no_lag_to_json() {
        let shard = Shard::new("shard-1", ShardRole::Primary, None, 12345);
        let payload = serde_json::to_string(&shard).unwrap();
        let expected = r#"{"id":"shard-1","role":"Primary","lag":null,"last_op":12345}"#;
        assert_eq!(payload, expected);
    }
}
