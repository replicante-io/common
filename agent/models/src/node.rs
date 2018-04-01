use super::agent::AgentInfo;
use super::datastore::DatastoreInfo;
use super::shard::Shard;


/// Agent and datastore information.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub agent: AgentInfo,
    pub datastore: DatastoreInfo,
}

impl NodeInfo {
    pub fn new(agent: AgentInfo, datastore: DatastoreInfo) -> NodeInfo {
        NodeInfo { agent, datastore }
    }
}


/// Datastore status information.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct NodeStatus {
    pub shards: Vec<Shard>,
}

impl NodeStatus {
    pub fn new(shards: Vec<Shard>) -> NodeStatus {
        NodeStatus { shards }
    }
}


#[cfg(test)]
mod tests {
    mod info {
        use serde_json;
        use super::super::super::AgentInfo;
        use super::super::super::AgentVersion;
        use super::super::super::datastore::DatastoreInfo;
        use super::super::NodeInfo;

        #[test]
        fn from_json() {
            let payload = r#"{"agent":{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}},"datastore":{"cluster":"cluster","kind":"DB","name":"Name","version":"1.2.3"}}"#;
            let info: NodeInfo = serde_json::from_str(payload).unwrap();
            let agent = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            let datastore = DatastoreInfo::new("cluster", "DB", "Name", "1.2.3");
            let expected = NodeInfo::new(agent, datastore);
            assert_eq!(info, expected);
        }

        #[test]
        fn to_json() {
            let agent = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            let datastore = DatastoreInfo::new("cluster", "DB", "Name", "1.2.3");
            let info = NodeInfo::new(agent, datastore);
            let payload = serde_json::to_string(&info).unwrap();
            let expected = r#"{"agent":{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}},"datastore":{"cluster":"cluster","kind":"DB","name":"Name","version":"1.2.3"}}"#;
            assert_eq!(payload, expected);
        }
    }

    mod status {
        use serde_json;
        use super::super::super::Shard;
        use super::super::super::ShardRole;
        use super::super::NodeStatus;

        #[test]
        fn from_json() {
            let shard = Shard::new("id", ShardRole::Secondary, Some(2), 1234);
            let expected = NodeStatus::new(vec![shard]);
            let payload = r#"{"shards":[{"id":"id","role":"Secondary","lag":2,"last_op":1234}]}"#;
            let status: NodeStatus = serde_json::from_str(payload).unwrap();
            assert_eq!(status, expected);
        }

        #[test]
        fn to_json() {
            let shard = Shard::new("id", ShardRole::Secondary, Some(2), 1234); 
            let status = NodeStatus::new(vec![shard]);
            let payload = serde_json::to_string(&status).unwrap();
            let expected = r#"{"shards":[{"id":"id","role":"Secondary","lag":2,"last_op":1234}]}"#;
            assert_eq!(payload, expected);
        }
    }
}
