// SPDX-License-Identifier: CC0-1.0

//! Macros for serde trait implementations, and supporting code.

/// Functions used by serde impls of all hashes.
#[cfg(feature = "serde")]
pub mod serde_details {
    use core::marker::PhantomData;
    use core::str::FromStr;
    use core::{fmt, str};

    use crate::FromSliceError;
    struct HexVisitor<ValueT>(PhantomData<ValueT>);
    use serde::{de, Deserializer, Serializer};

    impl<'de, ValueT> de::Visitor<'de> for HexVisitor<ValueT>
    where
        ValueT: FromStr,
        <ValueT as FromStr>::Err: fmt::Display,
    {
        type Value = ValueT;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an ASCII hex string")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> core::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            if let Ok(hex) = str::from_utf8(v) {
                hex.parse::<Self::Value>().map_err(E::custom)
            } else {
                Err(E::invalid_value(de::Unexpected::Bytes(v), &self))
            }
        }

        fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse::<Self::Value>().map_err(E::custom)
        }
    }

    struct BytesVisitor<ValueT>(PhantomData<ValueT>);

    impl<'de, ValueT> de::Visitor<'de> for BytesVisitor<ValueT>
    where
        ValueT: SerdeHash,
        <ValueT as FromStr>::Err: fmt::Display,
    {
        type Value = ValueT;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a bytestring")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> core::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            SerdeHash::from_slice_delegated(v).map_err(|_| {
                // from_slice only errors on incorrect length
                E::invalid_length(v.len(), &stringify!(N))
            })
        }
    }

    /// Default serialization/deserialization methods.
    pub trait SerdeHash
    where
        Self: Sized
            + FromStr
            + fmt::Display
            + crate::Hash,
        <Self as FromStr>::Err: fmt::Display,
    {
        /// Size, in bits, of the hash.
        const N: usize;

        /// Helper function to turn a deserialized slice into the correct hash type.
        fn from_slice_delegated(sl: &[u8]) -> core::result::Result<Self, FromSliceError>;

        /// Do serde serialization.
        fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
            if s.is_human_readable() {
                s.collect_str(self)
            } else {
                s.serialize_bytes(<Self as crate::Hash>::as_byte_array(self).as_ref())
            }
        }

        /// Do serde deserialization.
        fn deserialize<'de, D: Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
            if d.is_human_readable() {
                d.deserialize_str(HexVisitor::<Self>(PhantomData))
            } else {
                d.deserialize_bytes(BytesVisitor::<Self>(PhantomData))
            }
        }
    }
}

/// Implements `Serialize` and `Deserialize` for a type `$t` which
/// represents a newtype over a byte-slice over length `$len`.
#[macro_export]
#[cfg(feature = "serde")]
macro_rules! serde_impl(
    ($t:ident, $len:expr $(, $gen:ident: $gent:ident)*) => (
        impl<$($gen: $gent),*> $crate::serde_macros::serde_details::SerdeHash for $t<$($gen),*> {
            const N : usize = $len;
            fn from_slice_delegated(sl: &[u8]) -> core::result::Result<Self, $crate::FromSliceError> {
                <$t<$($gen),*> as $crate::Hash>::from_slice(sl)
            }
        }

        impl<$($gen: $gent),*> $crate::serde::Serialize for $t<$($gen),*> {
            fn serialize<S: $crate::serde::Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
                $crate::serde_macros::serde_details::SerdeHash::serialize(self, s)
            }
        }

        impl<'de $(, $gen: $gent)*> $crate::serde::Deserialize<'de> for $t<$($gen),*> {
            fn deserialize<D: $crate::serde::Deserializer<'de>>(d: D) -> core::result::Result<$t<$($gen),*>, D::Error> {
                $crate::serde_macros::serde_details::SerdeHash::deserialize(d)
            }
        }
));

/// Does an "empty" serde implementation for the configuration without serde feature.
#[macro_export]
#[cfg(not(feature = "serde"))]
macro_rules! serde_impl(
        ($t:ident, $len:expr $(, $gen:ident: $gent:ident)*) => ()
);
