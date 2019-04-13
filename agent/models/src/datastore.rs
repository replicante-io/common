/// Datastore version details.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct DatastoreInfo {
    pub cluster_display_name: Option<String>,
    pub cluster_id: String,
    pub kind: String,
    pub node_id: String,
    pub version: String,
}

impl DatastoreInfo {
    pub fn new<S1, S2, S3, S4>(
        cluster_id: S1,
        kind: S2,
        node_id: S3,
        version: S4,
        cluster_display_name: Option<String>,
    ) -> DatastoreInfo
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        DatastoreInfo {
            cluster_display_name,
            cluster_id: cluster_id.into(),
            kind: kind.into(),
            node_id: node_id.into(),
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
        let payload = concat!(
            r#"{"cluster_display_name":null,"cluster_id":"id","#,
            r#""kind":"DB","node_id":"Name","version":"1.2.3"}"#
        );
        let info: DatastoreInfo = serde_json::from_str(payload).unwrap();
        let expected = DatastoreInfo::new("id", "DB", "Name", "1.2.3", None);
        assert_eq!(info, expected);
    }

    #[test]
    fn from_json_with_display_name() {
        let payload = concat!(
            r#"{"cluster_display_name":"display name","cluster_id":"id","kind":"DB","#,
            r#""node_id":"Name","version":"1.2.3"}"#
        );
        let info: DatastoreInfo = serde_json::from_str(payload).unwrap();
        let expected = DatastoreInfo::new("id", "DB", "Name", "1.2.3", Some("display name".into()));
        assert_eq!(info, expected);
    }

    #[test]
    fn to_json() {
        let info = DatastoreInfo::new("id", "DB", "Name", "1.2.3", None);
        let payload = serde_json::to_string(&info).unwrap();
        let expected = concat!(
            r#"{"cluster_display_name":null,"cluster_id":"id","#,
            r#""kind":"DB","node_id":"Name","version":"1.2.3"}"#
        );
        assert_eq!(payload, expected);
    }

    #[test]
    fn to_json_with_display() {
        let info = DatastoreInfo::new("id", "DB", "Name", "1.2.3", Some("display name".into()));
        let payload = serde_json::to_string(&info).unwrap();
        let expected = concat!(
            r#"{"cluster_display_name":"display name","cluster_id":"id","#,
            r#""kind":"DB","node_id":"Name","version":"1.2.3"}"#
        );
        assert_eq!(payload, expected);
    }
}
