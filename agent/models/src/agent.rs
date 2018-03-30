/// Agent information.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    version: AgentVersion,
}

impl AgentInfo {
    pub fn new(version: AgentVersion) -> AgentInfo {
        AgentInfo { version }
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
    use serde_json;
    use super::AgentInfo;
    use super::AgentVersion;

    #[test]
    fn info_to_json() {
        let info = AgentInfo::new(AgentVersion::new("abc123", "1.2.3", "tainted"));
        let payload = serde_json::to_string(&info).unwrap();
        let expected = r#"{"version":{"checkout":"abc123","number":"1.2.3","taint":"tainted"}}"#;
        assert_eq!(payload, expected);
    }

    #[test]
    fn version_to_json() {
        let version = AgentVersion::new("abc123", "1.2.3", "tainted");
        let payload = serde_json::to_string(&version).unwrap();
        let expected = r#"{"checkout":"abc123","number":"1.2.3","taint":"tainted"}"#;
        assert_eq!(payload, expected);
    }
}
