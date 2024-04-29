use std::fmt::Formatter;

use bevy::{
    ecs::{
        schedule::{Chain, SystemConfigs},
        system::BoxedSystem,
    },
    prelude::{IntoSystem, IntoSystemConfigs},
};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Uuid(bevy::utils::Uuid);
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

pub trait IterExt: Iterator {
    type IterItem;

    /// Asserts that an iterator has a single item and returns it
    fn single(self) -> Self::IterItem;
}
impl<I> IterExt for I
where
    I: Iterator,
{
    type IterItem = I::Item;

    fn single(mut self) -> Self::IterItem {
        match (self.next(), self.next()) {
            (Some(item), None) => return item,
            (None, _) => panic!("Expected 1 item in iterator, found 0."),
            (Some(_), Some(_)) => panic!("Expected 1 item in iterator, found multiple."),
        }
    }
}

pub trait StrJoin {
    fn join(self, sep: impl AsRef<str>) -> String;
}
impl<T, I> StrJoin for T
where
    T: Iterator<Item = I>,
    I: AsRef<str>,
{
    fn join(mut self, sep: impl AsRef<str>) -> String {
        let mut joined = String::new();
        if let Some(first) = self.next() {
            joined.push_str(first.as_ref());
        } else {
            return joined;
        }

        for part in self {
            joined.push_str(sep.as_ref());
            joined.push_str(part.as_ref());
        }
        joined
    }
}

pub struct OrderedSystemList(Vec<BoxedSystem>);
impl OrderedSystemList {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push<M>(&mut self, sys: impl IntoSystem<(), (), M>) {
        self.0.push(Box::new(IntoSystem::into_system(sys)));
    }
}

impl IntoSystemConfigs<()> for OrderedSystemList {
    fn into_configs(self) -> SystemConfigs {
        SystemConfigs::Configs {
            configs: self.0.into_iter().map(IntoSystemConfigs::into_configs).collect(),
            collective_conditions: vec![],
            chained: Chain::Yes,
        }
    }
}
