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
    pub fn new(checkout: &str, number: &str, taint: &str) -> AgentVersion {
        AgentVersion {
            checkout: String::from(checkout),
            number: String::from(number),
            taint: String::from(taint)
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
