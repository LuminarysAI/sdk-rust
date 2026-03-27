//! Luminarys SDK for Rust skills.

pub mod abi;
pub mod archive;
pub mod cluster;
pub mod commands;
pub mod context;
pub mod entrypoint;
pub mod fs;
pub mod http;
pub mod llm;
pub mod shell;
pub mod state;
pub mod system;
pub mod tcp;
pub mod types;
pub mod ws;

pub use archive::*;
pub use cluster::*;
pub use commands::*;
pub use context::Context;
pub use entrypoint::*;
pub use fs::*;
pub use http::*;
pub use llm::*;
pub use shell::*;
pub use state::*;
pub use system::*;
pub use tcp::*;
pub use types::*;
pub use ws::*;

/// Serde helper: deserialise bytes from bin, null, or absent → empty Vec.
pub(crate) mod bytes_or_null {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Vec<u8>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "byte array, null, or absent")
            }
            // msgpack bin format
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Vec<u8>, E> {
                Ok(v.to_vec())
            }
            fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<Vec<u8>, E> {
                Ok(v)
            }
            // msgpack array of u8 (rare but possible)
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self, mut seq: A,
            ) -> Result<Vec<u8>, A::Error> {
                let mut v = Vec::new();
                while let Some(b) = seq.next_element::<u8>()? {
                    v.push(b);
                }
                Ok(v)
            }
            // msgpack nil → empty Vec
            fn visit_unit<E: serde::de::Error>(self) -> Result<Vec<u8>, E> {
                Ok(vec![])
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Vec<u8>, E> {
                Ok(vec![])
            }
            fn visit_some<D2: Deserializer<'de>>(
                self, d: D2,
            ) -> Result<Vec<u8>, D2::Error> {
                d.deserialize_any(self)
            }
        }
        d.deserialize_any(Visitor)
    }
}

/// Serde helper: deserialise int or float → i64.
/// The host sometimes sends integer parameters as
/// float64 (e.g. 3.0 instead of 3). This module accepts both forms.
#[allow(dead_code)]
pub(crate) mod int_or_float {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &i64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_i64(*v)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = i64;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "integer or float")
            }
            fn visit_i8<E: serde::de::Error>(self, v: i8)   -> Result<i64, E> { Ok(v as i64) }
            fn visit_i16<E: serde::de::Error>(self, v: i16) -> Result<i64, E> { Ok(v as i64) }
            fn visit_i32<E: serde::de::Error>(self, v: i32) -> Result<i64, E> { Ok(v as i64) }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<i64, E> { Ok(v) }
            fn visit_u8<E: serde::de::Error>(self, v: u8)   -> Result<i64, E> { Ok(v as i64) }
            fn visit_u16<E: serde::de::Error>(self, v: u16) -> Result<i64, E> { Ok(v as i64) }
            fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<i64, E> { Ok(v as i64) }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<i64, E> { Ok(v as i64) }
            fn visit_f32<E: serde::de::Error>(self, v: f32) -> Result<i64, E> { Ok(v as i64) }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<i64, E> { Ok(v as i64) }
        }
        d.deserialize_any(Visitor)
    }
}


/// Convenience re-export of the most-used items.
pub mod prelude {
    pub use crate::archive::*;
    pub use crate::cluster::*;
    pub use crate::commands::*;
    pub use crate::context::Context;
    pub use crate::entrypoint::*;
    pub use crate::fs::*;
    pub use crate::http::*;
    pub use crate::llm::*;
    pub use crate::shell::*;
    pub use crate::state::*;
    pub use crate::system::*;
    pub use crate::tcp::*;
    pub use crate::types::*;
    pub use crate::ws::*;
}

/// Serde helper: deserialise int or float → Option<i64>.
/// Used for optional integer parameters in generated skill handlers.
#[allow(dead_code)]
pub(crate) mod int_or_float_opt {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Option<i64>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(n) => s.serialize_i64(*n),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Option<i64>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "integer, float, or null")
            }
            fn visit_i8<E: serde::de::Error>(self, v: i8)   -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_i16<E: serde::de::Error>(self, v: i16) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_i32<E: serde::de::Error>(self, v: i32) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Option<i64>, E> { Ok(Some(v)) }
            fn visit_u8<E: serde::de::Error>(self, v: u8)   -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_u16<E: serde::de::Error>(self, v: u16) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_f32<E: serde::de::Error>(self, v: f32) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Option<i64>, E> { Ok(Some(v as i64)) }
            fn visit_unit<E: serde::de::Error>(self)         -> Result<Option<i64>, E> { Ok(None) }
            fn visit_none<E: serde::de::Error>(self)         -> Result<Option<i64>, E> { Ok(None) }
            fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<Option<i64>, D2::Error> {
                d.deserialize_any(Visitor)
            }
        }
        d.deserialize_any(Visitor)
    }
}

/// Serde helper: deserialise bool or int → Option<bool>.
/// Some hosts send boolean params as 0/1 integers instead of true/false.
#[allow(dead_code)]
pub(crate) mod bool_or_int_opt {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Option<bool>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(b) => s.serialize_bool(*b),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<bool>, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Option<bool>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "bool, int, or null")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Option<bool>, E> { Ok(Some(v)) }
            fn visit_i64<E: serde::de::Error>(self, v: i64)   -> Result<Option<bool>, E> { Ok(Some(v != 0)) }
            fn visit_u64<E: serde::de::Error>(self, v: u64)   -> Result<Option<bool>, E> { Ok(Some(v != 0)) }
            fn visit_f64<E: serde::de::Error>(self, v: f64)   -> Result<Option<bool>, E> { Ok(Some(v != 0.0)) }
            fn visit_unit<E: serde::de::Error>(self)           -> Result<Option<bool>, E> { Ok(None) }
            fn visit_none<E: serde::de::Error>(self)           -> Result<Option<bool>, E> { Ok(None) }
            fn visit_some<D2: Deserializer<'de>>(self, d: D2)  -> Result<Option<bool>, D2::Error> {
                d.deserialize_any(Visitor)
            }
        }
        d.deserialize_any(Visitor)
    }
}

/// Serde helper: deserialise array or null/absent → Vec<T>.
/// The host may send nil instead of an empty array for fields like `matches`.
pub(crate) mod vec_or_null {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(v: &Vec<T>, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer, T: serde::Serialize {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(v.len()))?;
        for item in v { seq.serialize_element(item)?; }
        seq.end()
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
    where D: Deserializer<'de>, T: Deserialize<'de> {
        struct Visitor<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
            type Value = Vec<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "array or null")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<T>, A::Error> {
                let mut v = Vec::new();
                while let Some(item) = seq.next_element()? { v.push(item); }
                Ok(v)
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Vec<T>, E> { Ok(vec![]) }
            fn visit_none<E: serde::de::Error>(self) -> Result<Vec<T>, E> { Ok(vec![]) }
            fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<Vec<T>, D2::Error> {
                d.deserialize_any(self)
            }
        }
        d.deserialize_any(Visitor(std::marker::PhantomData))
    }
}
