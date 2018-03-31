/// Datastore version details.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct DatastoreInfo {
    pub kind: String,
    pub name: String,
    pub version: String,
}

impl DatastoreInfo {
    pub fn new<S1, S2, S3>(kind: S1, name: S2, version: S3) -> DatastoreInfo
        where S1: Into<String>,
              S2: Into<String>,
              S3: Into<String>,
    {
        DatastoreInfo {
            kind: kind.into(),
            name: name.into(),
            version: version.into(),
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
