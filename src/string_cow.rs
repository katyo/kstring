use core::{
    cmp::Ordering,
    convert::Infallible,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    str::FromStr,
};
use std::{
    borrow::{Borrow, Cow},
    ffi::OsStr,
    path::Path,
};

use crate::{
    backend::{BoxedStr, DefaultStr, HeapStr},
    KString, KStringRef, KStringRefInner,
};

/// A reference to a UTF-8 encoded, immutable string.
#[derive(Clone)]
#[repr(transparent)]
pub struct KStringCow<'s, B = DefaultStr> {
    pub(crate) inner: KStringCowInner<'s, B>,
}

#[derive(Clone)]
pub(crate) enum KStringCowInner<'s, B> {
    Borrowed(&'s str),
    Owned(KString<B>),
}

impl<B> KStringCow<'_, B> {
    /// Create a new empty `KStringCow`.
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
            inner: KStringCowInner::Owned(KString::from_static(other)),
        }
    }
}

impl<'s, B: HeapStr> KStringCow<'s, B> {
    /// Create an owned `KStringCow`.
    #[inline]
    #[must_use]
    pub fn from_boxed(other: BoxedStr) -> Self {
        Self {
            inner: KStringCowInner::Owned(KString::from_boxed(other)),
        }
    }

    /// Create an owned `KStringCow`.
    #[inline]
    #[must_use]
    pub fn from_string(other: String) -> Self {
        Self {
            inner: KStringCowInner::Owned(KString::from_string(other)),
        }
    }

    /// Create a reference to a borrowed data.
    #[inline]
    #[must_use]
    pub fn from_ref(other: &'s str) -> Self {
        Self {
            inner: KStringCowInner::Borrowed(other),
        }
    }

    /// Get a reference to the `KString`.
    #[inline]
    #[must_use]
    pub fn get_ref(&self) -> KStringRef<'_> {
        self.inner.get_ref()
    }

    /// Clone the data into an owned-type.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> KString<B> {
        self.inner.into_owned()
    }

    /// Extracts a string slice containing the entire `KStringCow`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        String::from(self.into_boxed_str())
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    #[inline]
    #[must_use]
    pub fn into_boxed_str(self) -> BoxedStr {
        self.inner.into_boxed_str()
    }

    /// Convert to a Cow str
    #[inline]
    #[must_use]
    pub fn into_cow_str(self) -> Cow<'s, str> {
        self.inner.into_cow_str()
    }
}

impl<'s, B: HeapStr> KStringCowInner<'s, B> {
    #[inline]
    fn get_ref(&self) -> KStringRef<'_> {
        match self {
            Self::Borrowed(s) => KStringRef::from_ref(s),
            Self::Owned(s) => s.get_ref(),
        }
    }

    #[inline]
    fn into_owned(self) -> KString<B> {
        match self {
            Self::Borrowed(s) => KString::from_ref(s),
            Self::Owned(s) => s,
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s.as_str(),
        }
    }

    #[inline]
    fn into_boxed_str(self) -> BoxedStr {
        match self {
            Self::Borrowed(s) => BoxedStr::from(s),
            Self::Owned(s) => s.into_boxed_str(),
        }
    }

    /// Convert to a Cow str
    #[inline]
    fn into_cow_str(self) -> Cow<'s, str> {
        match self {
            Self::Borrowed(s) => Cow::Borrowed(s),
            Self::Owned(s) => s.into_cow_str(),
        }
    }
}

impl<B: HeapStr> Deref for KStringCow<'_, B> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> Eq for KStringCow<'_, B> {}

impl<'s, B: HeapStr> PartialEq<KStringCow<'s, B>> for KStringCow<'s, B> {
    #[inline]
    fn eq(&self, other: &KStringCow<'s, B>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<B: HeapStr> PartialEq<str> for KStringCow<'_, B> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'s, B: HeapStr> PartialEq<&'s str> for KStringCow<'s, B> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<B: HeapStr> PartialEq<String> for KStringCow<'_, B> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<B: HeapStr> Ord for KStringCow<'_, B> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<B: HeapStr> PartialOrd for KStringCow<'_, B> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: HeapStr> Hash for KStringCow<'_, B> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<B: HeapStr> fmt::Debug for KStringCow<'_, B> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<B: HeapStr> fmt::Display for KStringCow<'_, B> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<B: HeapStr> AsRef<str> for KStringCow<'_, B> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> AsRef<[u8]> for KStringCow<'_, B> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<B: HeapStr> AsRef<OsStr> for KStringCow<'_, B> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (**self).as_ref()
    }
}

impl<B: HeapStr> AsRef<Path> for KStringCow<'_, B> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl<B: HeapStr> Borrow<str> for KStringCow<'_, B> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<B> Default for KStringCow<'_, B> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<B: HeapStr> From<KString<B>> for KStringCow<'_, B> {
    #[inline]
    fn from(other: KString<B>) -> Self {
        let inner = KStringCowInner::Owned(other);
        Self { inner }
    }
}

impl<'s, B: HeapStr> From<&'s KString<B>> for KStringCow<'s, B> {
    #[inline]
    fn from(other: &'s KString<B>) -> Self {
        let other = other.get_ref();
        other.into()
    }
}

impl<'s, B: HeapStr> From<KStringRef<'s>> for KStringCow<'s, B> {
    #[inline]
    fn from(other: KStringRef<'s>) -> Self {
        match other.inner {
            KStringRefInner::Borrowed(s) => Self::from_ref(s),
            KStringRefInner::Singleton(s) => Self::from_static(s),
        }
    }
}

impl<'s, B: HeapStr> From<&'s KStringRef<'s>> for KStringCow<'s, B> {
    #[inline]
    fn from(other: &'s KStringRef<'s>) -> Self {
        match other.inner {
            KStringRefInner::Borrowed(s) => Self::from_ref(s),
            KStringRefInner::Singleton(s) => Self::from_static(s),
        }
    }
}

impl<B: HeapStr> From<String> for KStringCow<'_, B> {
    #[inline]
    fn from(other: String) -> Self {
        Self::from_string(other)
    }
}

impl<'s, B: HeapStr> From<&'s String> for KStringCow<'s, B> {
    #[inline]
    fn from(other: &'s String) -> Self {
        Self::from_ref(other.as_str())
    }
}

impl<B: HeapStr> From<BoxedStr> for KStringCow<'_, B> {
    #[inline]
    fn from(other: BoxedStr) -> Self {
        // Since the memory is already allocated, don't bother moving it into a FixedString
        Self::from_boxed(other)
    }
}

impl<'s, B: HeapStr> From<&'s BoxedStr> for KStringCow<'s, B> {
    #[inline]
    fn from(other: &'s BoxedStr) -> Self {
        Self::from_ref(other)
    }
}

impl<'s, B: HeapStr> From<&'s str> for KStringCow<'s, B> {
    #[inline]
    fn from(other: &'s str) -> Self {
        Self::from_ref(other)
    }
}

impl<B: HeapStr> FromStr for KStringCow<'_, B> {
    type Err = Infallible;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_string(s.into()))
    }
}

#[cfg(feature = "serde")]
impl<B: HeapStr> serde::Serialize for KStringCow<'_, B> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de, B: HeapStr> serde::Deserialize<'de> for KStringCow<'_, B> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        KString::deserialize(deserializer).map(|s| s.into())
    }
}

#[cfg(feature = "diesel")]
#[derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)]
#[diesel(foreign_derive)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[allow(dead_code)]
struct KStringCowProxy<'s, B>(KStringCow<'s, B>);

#[cfg(feature = "diesel")]
impl<B, ST, DB> diesel::deserialize::FromSql<ST, DB> for KStringCow<'_, B>
where
    B: HeapStr,
    DB: diesel::backend::Backend,
    *const str: diesel::deserialize::FromSql<ST, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        KString::from_sql(bytes).map(From::from)
    }
}

#[cfg(feature = "diesel")]
impl<B, DB> diesel::serialize::ToSql<diesel::sql_types::Text, DB> for KStringCow<'_, B>
where
    B: HeapStr,
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
        println!("KStringCow: {}", size_of::<KStringCow<'static>>());
    }
}
