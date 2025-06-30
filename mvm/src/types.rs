use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::ops::Deref;

macro_rules! define_byte_array {
    ($struct_name:ident, $len:expr) => {
        #[derive(Debug)]
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

define_byte_array!(ActionHash, 32);
define_byte_array!(Signature, 64);
define_byte_array!(AccountKey, 33);