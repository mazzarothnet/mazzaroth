use alloy_rlp::{Decodable, Encodable, Error as RlpError, RlpDecodable, RlpEncodable, bytes};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;
use std::ops::{Add, Div, Mul, Sub, AddAssign};

macro_rules! define_byte_array {
    ($struct_name:ident, $len:expr) => {
        #[derive(Debug, RlpEncodable, RlpDecodable, Clone, PartialEq, Eq, Copy)]
        pub struct $struct_name(pub [u8; $len]);

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
                serializer.serialize_bytes(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $struct_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let vec: Vec<u8> = Deserialize::deserialize(deserializer)?;
                if vec.len() != $len {
                    return Err(serde::de::Error::invalid_length(
                        vec.len(),
                        &stringify!($len),
                    ));
                }
                let mut arr = [0u8; $len];
                arr.copy_from_slice(&vec);
                Ok($struct_name(arr))
            }
        }
    };
}

macro_rules! impl_u256_ops {
    ($type_name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Ord, PartialOrd)]
        pub struct $type_name(pub crypto_bigint::U256);

        impl Add for $type_name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                $type_name(self.0 + rhs.0)
            }
        }

        impl Sub for $type_name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                $type_name(self.0 - rhs.0)
            }
        }

        impl Mul for $type_name {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                $type_name(self.0 * rhs.0)
            }
        }

        impl Div for $type_name {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                $type_name(self.0 / rhs.0)
            }
        }

        impl AddAssign for $type_name {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
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

        impl Into<crypto_bigint::U256> for $type_name {
            fn into(self) -> crypto_bigint::U256 {
                self.0
            }
        }

        impl ToString for $type_name {
            fn to_string(&self) -> String {
                self.0.to_string()
            }
        }
    };
}

define_byte_array!(ActionHash, 32);
define_byte_array!(Signature, 64);
define_byte_array!(AccountKey, 33);

impl_u256_ops!(BlockKey);
impl_u256_ops!(DagWork);


