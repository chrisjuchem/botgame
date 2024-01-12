use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct Uuid(bevy::utils::Uuid);
impl Uuid {
    pub fn new() -> Self {
        Self(bevy::utils::Uuid::new_v4())
    }
}

struct UuidVisitor;
impl<'de> serde::de::Visitor<'de> for UuidVisitor {
    type Value = Uuid;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a UUID")
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let bytes: [u8; 16] =
            v.try_into().map_err(|_| serde::de::Error::invalid_length(16, &self))?;

        Ok(Uuid(bevy::utils::Uuid::from_bytes(bytes)))
    }
}

impl serde::Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0.as_bytes()[..])
    }
}
impl<'de> serde::Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(UuidVisitor)
    }
}
