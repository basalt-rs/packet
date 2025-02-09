use serde::{de::DeserializeOwned, Serialize};

#[cfg_attr(not(feature = "render"), allow(unused))]
pub fn convert<D, S>(value: &S) -> D
where
    D: DeserializeOwned,
    S: Serialize,
{
    let value = serde_json::to_value(value).unwrap();
    serde_json::from_value(value).unwrap()
}
