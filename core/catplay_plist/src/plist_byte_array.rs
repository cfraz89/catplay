use bytes::Bytes;
use serde::{de, ser};
use std::{fmt, ops::Deref};

#[derive(Clone, PartialEq, Eq)]
pub struct PlistByteArray(pub(crate) Bytes);

impl From<Vec<u8>> for PlistByteArray {
    fn from(from: Vec<u8>) -> Self {
        PlistByteArray(from.into())
    }
}

impl<const N: usize> From<[u8; N]> for PlistByteArray {
    fn from(from: [u8; N]) -> Self {
        PlistByteArray(from.to_vec().into())
    }
}

impl From<PlistByteArray> for Vec<u8> {
    fn from(from: PlistByteArray) -> Self {
        from.0.into()
    }
}

impl AsRef<[u8]> for PlistByteArray {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Debug for PlistByteArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(debug_assertions)]
        {
            use catplay_tracing::hexdump::pretty_hexdump_limited;

            const MAX_TRACE: usize = 256;
            return f.write_str(&pretty_hexdump_limited(self.as_ref(), MAX_TRACE));
        }

        #[allow(unused)]
        f.write_fmt(format_args!("<plist binary {}b>", self.as_ref().len()))
    }
}

impl Default for PlistByteArray {
    fn default() -> Self {
        Self(Bytes::new())
    }
}

impl Deref for PlistByteArray {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl ser::Serialize for PlistByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(self.as_ref())
    }
}

struct DataVisitor;

impl de::Visitor<'_> for DataVisitor {
    type Value = PlistByteArray;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_byte_buf(v.to_owned())
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.into())
    }
}

impl<'de> de::Deserialize<'de> for PlistByteArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(DataVisitor)
    }
}
