pub mod duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(value: &Duration, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // NOTE: this can technically serialise a duration >= 2^64, while the deserialiser can only
        // deserialise up to 2^64 - 1, but I'd be quite concerned if we have a duration that is longer
        // than 585 million years
        value.as_millis().serialize(ser)
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Duration::from_millis(u64::deserialize(de)?))
    }
}
