use bytes::{Bytes, BytesMut};
use serde::{
    Deserialize, Deserializer,
    de::Visitor,
    ser::{Serialize, Serializer},
};
use std::{fmt, ops::Deref, str::Utf8Error};

#[derive(Clone, PartialEq, Eq, Default)]
pub enum RtspString {
    Static(&'static str),
    Buf(Bytes),
    #[default]
    Empty,
}

impl RtspString {
    pub fn from_bytes(buf: impl Into<Bytes>) -> Result<Self, Utf8Error> {
        let buf = buf.into();
        std::str::from_utf8(&buf)?;
        Ok(Self::Buf(buf))
    }

    pub fn from_string(str: String) -> Self {
        Self::Buf(str.into_bytes().into())
    }

    pub const fn from_static(s: &'static str) -> Self {
        Self::Static(s)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Buf(buf) => unsafe { std::str::from_utf8_unchecked(buf) },
            Self::Empty => "",
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn split_to(&mut self, at: usize) -> Self {
        if let Self::Buf(buf) = self {
            return Self::Buf(buf.split_to(at));
        }

        let (head, tail) = {
            let s = self.as_str();
            let (head, tail) = s.split_at(at);
            (head.to_owned(), tail.to_owned())
        };

        *self = Self::from_string(tail);
        Self::from_string(head)
    }
}

impl Serialize for RtspString {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RtspString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RtspStringVisitor;

        impl<'de> Visitor<'de> for RtspStringVisitor {
            type Value = RtspString;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a UTF-8 string")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RtspString::from_string(v.to_owned()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RtspString::from_string(v.to_owned()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RtspString::from_string(v))
            }
        }

        deserializer.deserialize_str(RtspStringVisitor)
    }
}

impl AsRef<str> for RtspString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for RtspString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Debug for RtspString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Display for RtspString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::hash::Hash for RtspString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl From<&'static str> for RtspString {
    fn from(value: &'static str) -> Self {
        Self::from_static(value)
    }
}

impl From<String> for RtspString {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl TryFrom<BytesMut> for RtspString {
    type Error = Utf8Error;

    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        Self::from_bytes(value)
    }
}

impl PartialEq<str> for RtspString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl From<RtspString> for String {
    fn from(val: RtspString) -> Self {
        val.as_str().into()
    }
}
impl PartialEq<&str> for RtspString {
    fn eq(&self, other: &&str) -> bool {
        &self.as_str() == other
    }
}

#[test]
fn split_to_works_for_static_and_alloc_variants() {
    fn run_case(mut s: RtspString) {
        let sep = s.as_str().find(": ").unwrap();
        let left = s.split_to(sep);
        let _ = s.split_to(2); // drop ": "

        assert_eq!(left.as_str(), "left");
        assert_eq!(s.as_str(), "right");
    }

    let input = "left: right";
    run_case(RtspString::from_static(input));
    run_case(RtspString::from_string(input.to_owned()));
    run_case(RtspString::from_bytes(BytesMut::from(input)).unwrap());
}

#[test]
fn serialize_deserialize_cycle_works_for_all_variants() {
    fn run_case(input: RtspString) {
        let serialized = serde_json::to_string(&input).unwrap();
        let out: RtspString = serde_json::from_str(&serialized).unwrap();

        assert_eq!(input.as_str(), out.as_str());
    }

    let input = "rtsp://127.0.0.1:7000/stream";
    run_case(RtspString::from_static(input));
    run_case(RtspString::from_string(input.to_owned()));
    run_case(RtspString::from_bytes(BytesMut::from(input)).unwrap());
}
