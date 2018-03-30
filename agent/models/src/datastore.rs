/// Datastore version details.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct DatastoreInfo {
    kind: String,
    name: String,
    version: String,
}

impl DatastoreInfo {
    pub fn new(kind: &str, name: &str, version: &str) -> DatastoreInfo {
        DatastoreInfo {
            kind: String::from(kind),
            name: String::from(name),
            version: String::from(version),
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;
    use super::DatastoreInfo;

    #[test]
    fn from_json() {
        let payload = r#"{"kind":"DB","name":"Name","version":"1.2.3"}"#;
        let info: DatastoreInfo = serde_json::from_str(payload).unwrap();
        let expected = DatastoreInfo::new("DB", "Name", "1.2.3");
        assert_eq!(info, expected);
    }

    #[test]
    fn to_json() {
        let info = DatastoreInfo::new("DB", "Name", "1.2.3");
        let payload = serde_json::to_string(&info).unwrap();
        let expected = r#"{"kind":"DB","name":"Name","version":"1.2.3"}"#;
        assert_eq!(payload, expected);
    }
}
