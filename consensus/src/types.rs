use alloy_rlp::{Decodable, Encodable, Error as RlpError, RlpDecodable, RlpEncodable, bytes};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;

macro_rules! define_byte_array {
    ($struct_name:ident, $len:expr) => {
        #[derive(RlpEncodable, RlpDecodable, Clone, PartialEq, Eq, Copy, Ord, PartialOrd, Hash)]
        pub struct $struct_name(pub [u8; $len]);

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", hex::encode(self.0))
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                $struct_name([0u8; $len])
            }
        }

        impl $struct_name {
            pub fn new(arr: [u8; $len]) -> Self {
                $struct_name(arr)
            }
        }

        impl Deref for $struct_name {
            type Target = [u8];
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Serialize for $struct_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&hex::encode(self.0))
            }
        }

        impl<'de> Deserialize<'de> for $struct_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                let key = hex::decode(s).map_err(serde::de::Error::custom)?;
                let mut arr = [0u8; $len];
                arr.copy_from_slice(&key);
                Ok($struct_name(arr))
            }
        }
    };
}

macro_rules! impl_u256_ops {
    ($type_name:ident) => {
        #[derive(Clone, PartialEq, Eq, Copy, Ord, PartialOrd, Hash)]
        pub struct $type_name(pub crypto_bigint::U256);

        impl $type_name {
            pub const MAX_VAL: $type_name = $type_name(crypto_bigint::U256::MAX);
        }

        impl Serialize for $type_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let hex_str = hex::encode(self.0.to_be_bytes());
                serializer.serialize_str(&hex_str)
            }
        }

        impl<'de> Deserialize<'de> for $type_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
                Ok($type_name(crypto_bigint::U256::from_be_slice(&bytes)))
            }
        }

        impl std::fmt::Debug for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }

        impl Encodable for $type_name {
            fn encode(&self, out: &mut dyn bytes::BufMut) {
                let bytes = self.0.to_be_bytes();
                bytes.encode(out);
            }

            fn length(&self) -> usize {
                let bytes = self.0.to_be_bytes();
                bytes.length()
            }
        }

        impl Decodable for $type_name {
            fn decode(buf: &mut &[u8]) -> Result<Self, RlpError> {
                let bytes: [u8; 32] = Decodable::decode(buf)?;
                Ok($type_name(crypto_bigint::U256::from_be_slice(&bytes)))
            }
        }

        impl From<crypto_bigint::U256> for $type_name {
            fn from(value: crypto_bigint::U256) -> Self {
                $type_name(value)
            }
        }

        impl From<$type_name> for crypto_bigint::U256 {
            fn from(value: $type_name) -> crypto_bigint::U256 {
                value.0
            }
        }

        impl std::fmt::Display for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let hex_str = hex::encode(self.0.to_be_bytes());
                write!(f, "{}", hex_str)
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                $type_name(crypto_bigint::U256::ZERO)
            }
        }
    };
}

pub const ACCOUNT_KEY_LEN: usize = 33;

define_byte_array!(Hash, 32);
define_byte_array!(Signature, 64);
define_byte_array!(AccountKey, ACCOUNT_KEY_LEN);

impl_u256_ops!(BlockKey);
impl_u256_ops!(DagWork);
