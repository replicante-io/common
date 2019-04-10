/// Datastore version details.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct DatastoreInfo {
    pub cluster_display_name: Option<String>,
    pub cluster_id: String,
    pub kind: String,
    pub name: String,
    pub version: String,
}

impl DatastoreInfo {
    pub fn new<S1, S2, S3, S4>(
        cluster_display_name: Option<String>,
        cluster_id: S1,
        kind: S2,
        name: S3,
        version: S4,
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
        let payload = concat!(
            r#"{"cluster_display_name":null,"cluster_id":"id","kind":"DB","#,
            r#""name":"Name","version":"1.2.3"}"#
        );
        let info: DatastoreInfo = serde_json::from_str(payload).unwrap();
        let expected = DatastoreInfo::new(None, "id", "DB", "Name", "1.2.3");
        assert_eq!(info, expected);
    }

    #[test]
    fn from_json_with_display_name() {
        let payload = concat!(
            r#"{"cluster_display_name":"display name","cluster_id":"id","kind":"DB","#,
            r#""name":"Name","version":"1.2.3"}"#
        );
        let info: DatastoreInfo = serde_json::from_str(payload).unwrap();
        let expected = DatastoreInfo::new(Some("display name".into()), "id", "DB", "Name", "1.2.3");
        assert_eq!(info, expected);
    }

    #[test]
    fn to_json() {
        let info = DatastoreInfo::new(None, "id", "DB", "Name", "1.2.3");
        let payload = serde_json::to_string(&info).unwrap();
        let expected = concat!(
            r#"{"cluster_display_name":null,"cluster_id":"id","kind":"DB","#,
            r#""name":"Name","version":"1.2.3"}"#
        );
        assert_eq!(payload, expected);
    }

    #[test]
    fn to_json_with_display() {
        let info = DatastoreInfo::new(Some("display name".into()), "id", "DB", "Name", "1.2.3");
        let payload = serde_json::to_string(&info).unwrap();
        let expected = concat!(
            r#"{"cluster_display_name":"display name","cluster_id":"id","kind":"DB","#,
            r#""name":"Name","version":"1.2.3"}"#
        );
        assert_eq!(payload, expected);
    }
}
