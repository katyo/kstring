use super::{HeapStr, StdString, String, StringCow, StringRef};
use core::{fmt, marker::PhantomData, str::from_utf8};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Unexpected, Visitor},
};

impl<B: HeapStr> Serialize for String<B> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de, B: HeapStr> Deserialize<'de> for String<B> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(StringVisitor(PhantomData))
    }
}

struct StringVisitor<B>(PhantomData<B>);

impl<B: HeapStr> Visitor<'_> for StringVisitor<B> {
    type Value = String<B>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::Value::from_ref(v))
    }

    fn visit_string<E>(self, v: StdString) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::Value::from_string(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match from_utf8(v) {
            Ok(s) => Ok(Self::Value::from_ref(s)),
            Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match StdString::from_utf8(v) {
            Ok(s) => Ok(Self::Value::from_string(s)),
            Err(e) => Err(Error::invalid_value(
                Unexpected::Bytes(&e.into_bytes()),
                &self,
            )),
        }
    }
}

impl<B: HeapStr> Serialize for StringCow<'_, B> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de, B: HeapStr> Deserialize<'de> for StringCow<'_, B> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|s| s.into())
    }
}

impl Serialize for StringRef<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de: 's, 's> Deserialize<'de> for StringRef<'s> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &'s str = Deserialize::deserialize(deserializer)?;
        let s = StringRef::from_ref(s);
        Ok(s)
    }
}
