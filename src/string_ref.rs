use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};
use std::{borrow::Borrow, ffi::OsStr, path::Path};

use crate::{
    backend::{BoxedStr, HeapStr},
    KString, KStringCow,
};

/// A reference to a UTF-8 encoded, immutable string.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct KStringRef<'s> {
    pub(crate) inner: KStringRefInner<'s>,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum KStringRefInner<'s> {
    Borrowed(&'s str),
    Singleton(&'static str),
}

impl<'s> KStringRef<'s> {
    /// Create a new empty `KString`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self::from_static("")
    }

    /// Create a reference to a `'static` data.
    #[inline]
    #[must_use]
    pub const fn from_static(other: &'static str) -> Self {
        Self {
            inner: KStringRefInner::Singleton(other),
        }
    }

    /// Create a reference to a borrowed data.
    #[inline]
    #[must_use]
    pub fn from_ref(other: &'s str) -> Self {
        Self {
            inner: KStringRefInner::Borrowed(other),
        }
    }

    /// Clone the data into an owned-type.
    #[inline]
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_owned<B: HeapStr>(&self) -> KString<B> {
        self.inner.to_owned()
    }

    /// Extracts a string slice containing the entire `KStringRef`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    #[inline]
    #[must_use]
    pub fn into_mut(self) -> String {
        self.inner.into_mut()
    }
}

impl KStringRefInner<'_> {
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn to_owned<B: HeapStr>(&self) -> KString<B> {
        match self {
            Self::Borrowed(s) => KString::from_ref(s),
            Self::Singleton(s) => KString::from_static(s),
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Singleton(s) => s,
        }
    }

    #[inline]
    fn into_mut(self) -> String {
        self.as_str().to_owned()
    }
}

impl Deref for KStringRef<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Eq for KStringRef<'_> {}

impl<'s> PartialEq<KStringRef<'s>> for KStringRef<'s> {
    #[inline]
    fn eq(&self, other: &KStringRef<'s>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<str> for KStringRef<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'s> PartialEq<&'s str> for KStringRef<'s> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl PartialEq<String> for KStringRef<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl Ord for KStringRef<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for KStringRef<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for KStringRef<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Debug for KStringRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl fmt::Display for KStringRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for KStringRef<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for KStringRef<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<OsStr> for KStringRef<'_> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (**self).as_ref()
    }
}

impl AsRef<Path> for KStringRef<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl Borrow<str> for KStringRef<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Default for KStringRef<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'s, B: HeapStr> From<&'s KString<B>> for KStringRef<'s> {
    #[inline]
    fn from(other: &'s KString<B>) -> Self {
        other.as_ref()
    }
}

impl<'s, B: HeapStr> From<&'s KStringCow<'s, B>> for KStringRef<'s> {
    #[inline]
    fn from(other: &'s KStringCow<'s, B>) -> Self {
        other.as_ref()
    }
}

impl<'s> From<&'s String> for KStringRef<'s> {
    #[inline]
    fn from(other: &'s String) -> Self {
        KStringRef::from_ref(other.as_str())
    }
}

impl<'s> From<&'s BoxedStr> for KStringRef<'s> {
    #[inline]
    fn from(other: &'s BoxedStr) -> Self {
        Self::from_ref(other)
    }
}

impl<'s> From<&'s str> for KStringRef<'s> {
    #[inline]
    fn from(other: &'s str) -> Self {
        KStringRef::from_ref(other)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for KStringRef<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de: 's, 's> serde::Deserialize<'de> for KStringRef<'s> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &'s str = serde::Deserialize::deserialize(deserializer)?;
        let s = KStringRef::from_ref(s);
        Ok(s)
    }
}

#[cfg(feature = "diesel")]
#[derive(diesel::expression::AsExpression)]
#[diesel(foreign_derive)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[allow(dead_code)]
struct KStringRefProxy<'s>(KStringRef<'s>);

#[cfg(feature = "diesel")]
impl<DB> diesel::serialize::ToSql<diesel::sql_types::Text, DB> for KStringRef<'_>
where
    DB: diesel::backend::Backend,
    str: diesel::serialize::ToSql<diesel::sql_types::Text, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        self.as_str().to_sql(out)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn test_size() {
        println!("KStringRef: {}", size_of::<KStringRef<'static>>());
    }
}
