use super::datastore::DatastoreInfo;


/// Agent-specific information.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct AgentDetails {
    version: AgentVersion,
}

impl AgentDetails {
    pub fn new(version: AgentVersion) -> AgentDetails {
        AgentDetails { version }
    }
}


/// Agent information returned by the API.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    agent: AgentDetails,
    datastore: DatastoreInfo,
}

impl AgentInfo {
    pub fn new(agent: AgentDetails, datastore: DatastoreInfo) -> AgentInfo {
        AgentInfo { agent, datastore }
    }
}


/// Agent version details.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct AgentVersion {
    checkout: String,
    number: String,
    taint: String,
}

impl AgentVersion {
    pub fn new<S1, S2, S3>(checkout: S1, number: S2, taint: S3) -> AgentVersion
        where S1: Into<String>,
              S2: Into<String>,
              S3: Into<String>,
    {
        AgentVersion {
            checkout: checkout.into(),
            number: number.into(),
            taint: taint.into(),
        }
    }
}


#[cfg(test)]
mod tests {
    mod details {
        use serde_json;
        use super::super::AgentDetails;
        use super::super::AgentVersion;

        #[test]
        fn from_json() {
            let payload = r#"{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}}"#;
            let agent: AgentDetails = serde_json::from_str(payload).unwrap();
            let expected = AgentDetails::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            assert_eq!(agent, expected);
        }

        #[test]
        fn to_json() {
            let agent = AgentDetails::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            let payload = serde_json::to_string(&agent).unwrap();
            let expected = r#"{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}}"#;
            assert_eq!(payload, expected);
        }
    }

    mod info {
        use serde_json;
        use super::super::AgentDetails;
        use super::super::AgentInfo;
        use super::super::AgentVersion;
        use super::super::super::datastore::DatastoreInfo;

        #[test]
        fn from_json() {
            let payload = r#"{"agent":{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}},"datastore":{"kind":"DB","name":"Name","version":"1.2.3"}}"#;
            let info: AgentInfo = serde_json::from_str(payload).unwrap();
            let agent = AgentDetails::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            let datastore = DatastoreInfo::new("DB", "Name", "1.2.3");
            let expected = AgentInfo::new(agent, datastore);
            assert_eq!(info, expected);
        }

        #[test]
        fn to_json() {
            let agent = AgentDetails::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            let datastore = DatastoreInfo::new("DB", "Name", "1.2.3");
            let info = AgentInfo::new(agent, datastore);
            let payload = serde_json::to_string(&info).unwrap();
            let expected = r#"{"agent":{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}},"datastore":{"kind":"DB","name":"Name","version":"1.2.3"}}"#;
            assert_eq!(payload, expected);
        }
    }

    mod version {
        use serde_json;
        use super::super::AgentVersion;

        #[test]
        fn from_json() {
            let payload = r#"{"checkout":"abc123","number":"1.2.3","taint":"tainted"}"#;
            let version: AgentVersion = serde_json::from_str(payload).unwrap();
            let expected = AgentVersion::new("abc123", "1.2.3", "tainted");
            assert_eq!(version, expected);
        }

        #[test]
        fn to_json() {
            let version = AgentVersion::new("abc123", "1.2.3", "tainted");
            let payload = serde_json::to_string(&version).unwrap();
            let expected = r#"{"checkout":"abc123","number":"1.2.3","taint":"tainted"}"#;
            assert_eq!(payload, expected);
        }
    }
}
