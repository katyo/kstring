use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    str::Utf8Error,
};
use std::{borrow::Borrow, ffi::OsStr, path::Path, string::String as StdString};

use crate::{
    String, StringCow,
    backend::{BoxedStr, HeapStr},
};

/// A reference to a UTF-8 encoded, immutable string.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct StringRef<'s> {
    pub(crate) inner: StringRefInner<'s>,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum StringRefInner<'s> {
    Borrowed(&'s str),
    Singleton(&'static str),
}

impl<'s> StringRef<'s> {
    /// Create a new empty `String`.
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
            inner: StringRefInner::Singleton(other),
        }
    }

    /// Create a reference to a borrowed data.
    #[inline]
    #[must_use]
    pub fn from_ref(other: &'s str) -> Self {
        Self {
            inner: StringRefInner::Borrowed(other),
        }
    }

    /// Creates a new `StringRef` from a UTF-8 byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::StringRef;
    ///
    /// let bytes = [240, 159, 146, 150];
    /// let str_ref = StringRef::from_utf8(&bytes).unwrap();
    /// assert_eq!(str_ref.as_str(), "💖");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the slice is not valid UTF-8.
    #[inline]
    pub fn from_utf8(v: &'s [u8]) -> Result<Self, Utf8Error> {
        core::str::from_utf8(v).map(Self::from_ref)
    }

    /// Creates a new `StringRef` from a UTF-8 byte slice without checking.
    ///
    /// # Safety
    ///
    /// The bytes must be valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::StringRef;
    ///
    /// let bytes = [240, 159, 146, 150];
    /// let str_ref = unsafe { StringRef::from_utf8_unchecked(&bytes) };
    /// assert_eq!(str_ref.as_str(), "💖");
    /// ```
    #[cfg(feature = "unsafe")]
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(v: &'s [u8]) -> Self {
        Self::from_ref(unsafe { core::str::from_utf8_unchecked(v) })
    }

    /// Clone the data into an owned-type.
    #[inline]
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_owned<B: HeapStr>(&self) -> String<B> {
        self.inner.to_owned()
    }

    /// Extracts a string slice containing the entire `StringRef`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    #[inline]
    #[must_use]
    pub fn into_string(self) -> StdString {
        self.inner.into_string()
    }
}

impl StringRefInner<'_> {
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn to_owned<B: HeapStr>(&self) -> String<B> {
        match self {
            Self::Borrowed(s) => String::from_ref(s),
            Self::Singleton(s) => String::from_static(s),
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
    fn into_string(self) -> StdString {
        self.as_str().to_owned()
    }
}

impl Deref for StringRef<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Eq for StringRef<'_> {}

impl<'s> PartialEq<StringRef<'s>> for StringRef<'s> {
    #[inline]
    fn eq(&self, other: &StringRef<'s>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<str> for StringRef<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'s> PartialEq<&'s str> for StringRef<'s> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl PartialEq<String> for StringRef<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl Ord for StringRef<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for StringRef<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for StringRef<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Debug for StringRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl fmt::Display for StringRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for StringRef<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for StringRef<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<OsStr> for StringRef<'_> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (**self).as_ref()
    }
}

impl AsRef<Path> for StringRef<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl Borrow<str> for StringRef<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Default for StringRef<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'s, B: HeapStr> From<&'s String<B>> for StringRef<'s> {
    #[inline]
    fn from(other: &'s String<B>) -> Self {
        other.get_ref()
    }
}

impl<'s, B: HeapStr> From<&'s StringCow<'s, B>> for StringRef<'s> {
    #[inline]
    fn from(other: &'s StringCow<'s, B>) -> Self {
        other.get_ref()
    }
}

impl<'s> From<&'s StdString> for StringRef<'s> {
    #[inline]
    fn from(other: &'s StdString) -> Self {
        StringRef::from_ref(other.as_str())
    }
}

impl<'s> From<&'s BoxedStr> for StringRef<'s> {
    #[inline]
    fn from(other: &'s BoxedStr) -> Self {
        Self::from_ref(other)
    }
}

impl<'s> From<&'s str> for StringRef<'s> {
    #[inline]
    fn from(other: &'s str) -> Self {
        StringRef::from_ref(other)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::StringRef;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
}

#[cfg(feature = "quote")]
impl quote::ToTokens for StringRef<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.as_str().to_tokens(tokens);
    }
}

#[cfg(feature = "diesel")]
mod diesel_impl {
    use super::StringRef;
    use diesel::{
        backend::Backend,
        expression::AsExpression,
        serialize::{Output, Result as SerResult, ToSql},
        sql_types::Text,
    };

    #[derive(AsExpression)]
    #[diesel(foreign_derive)]
    #[diesel(sql_type = Text)]
    #[allow(dead_code)]
    struct StringRefProxy<'s>(StringRef<'s>);

    impl<DB> ToSql<Text, DB> for StringRef<'_>
    where
        DB: Backend,
        str: ToSql<Text, DB>,
    {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> SerResult {
            self.as_str().to_sql(out)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn test_size() {
        println!("StringRef: {}", size_of::<StringRef<'static>>());
    }
}
