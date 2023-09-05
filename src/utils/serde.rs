// modified https://docs.rs/serde_with/latest/serde_with/rust/double_option/
pub mod double_option_err {
    use serde::{Deserialize, Deserializer};

    /// Deserialize potentially non-existing optional value
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Ok(match Deserialize::deserialize(deserializer).map(Some) {
            Ok(d) => d,
            Err(_) => Some(None)
        })
    }

    /*/// Serialize optional value
    pub fn serialize<S, T>(values: &Option<Option<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match values {
            None => serializer.serialize_unit(),
            Some(None) => serializer.serialize_none(),
            Some(Some(v)) => serializer.serialize_some(&v),
        }
    }*/
}