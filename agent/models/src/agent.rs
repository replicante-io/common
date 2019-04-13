/// Agent-specific information.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub version: AgentVersion,
}

impl AgentInfo {
    pub fn new(version: AgentVersion) -> AgentInfo {
        AgentInfo { version }
    }
}

/// Agent version details.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct AgentVersion {
    pub checkout: String,
    pub number: String,
    pub taint: String,
}

impl AgentVersion {
    pub fn new<S1, S2, S3>(checkout: S1, number: S2, taint: S3) -> AgentVersion
    where
        S1: Into<String>,
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
    mod info {
        use serde_json;

        use super::super::AgentInfo;
        use super::super::AgentVersion;

        #[test]
        fn from_json() {
            let payload = r#"{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}}"#;
            let agent: AgentInfo = serde_json::from_str(payload).unwrap();
            let expected = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            assert_eq!(agent, expected);
        }

        #[test]
        fn to_json() {
            let agent = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
            let payload = serde_json::to_string(&agent).unwrap();
            let expected =
                r#"{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}}"#;
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
