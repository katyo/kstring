use core::{
    cmp::Ordering,
    convert::Infallible,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    str,
    str::{FromStr, Utf8Error},
};
use std::{
    borrow::{Borrow, Cow},
    ffi::OsStr,
    path::Path,
};

use crate::{
    StdString, String, StringRef, StringRefInner,
    backend::{BoxedStr, DefaultStr, HeapStr},
};

/// A reference to a UTF-8 encoded, immutable string.
#[derive(Clone)]
#[repr(transparent)]
pub struct StringCow<'s, B = DefaultStr> {
    pub(crate) inner: StringCowInner<'s, B>,
}

#[derive(Clone)]
pub(crate) enum StringCowInner<'s, B> {
    Borrowed(&'s str),
    Owned(String<B>),
}

impl<B> StringCow<'_, B> {
    /// Create a new empty `StringCow`.
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
            inner: StringCowInner::Owned(String::from_static(other)),
        }
    }
}

impl<'s, B: HeapStr> StringCow<'s, B> {
    /// Create an owned `StringCow`.
    #[inline]
    #[must_use]
    pub fn from_boxed(other: BoxedStr) -> Self {
        Self {
            inner: StringCowInner::Owned(String::from_boxed(other)),
        }
    }

    /// Create an owned `StringCow`.
    #[inline]
    #[must_use]
    pub fn from_string(other: StdString) -> Self {
        Self {
            inner: StringCowInner::Owned(String::from_string(other)),
        }
    }

    /// Create a reference to a borrowed data.
    #[inline]
    #[must_use]
    pub fn from_ref(other: &'s str) -> Self {
        Self {
            inner: StringCowInner::Borrowed(other),
        }
    }

    /// Creates a new `StringCow` from a UTF-8 byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::StringCow;
    ///
    /// let bytes = [240, 159, 146, 150];
    /// let str_cow = <StringCow>::from_utf8(&bytes).unwrap();
    /// assert_eq!(&str_cow, "💖");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the slice is not valid UTF-8.
    #[inline]
    pub fn from_utf8(other: &'s [u8]) -> Result<Self, Utf8Error> {
        str::from_utf8(other).map(Self::from_ref)
    }

    /// Creates a new `StringCow` from a UTF-8 byte slice without checking.
    ///
    /// # Safety
    ///
    /// The bytes must be valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::StringCow;
    ///
    /// let bytes = [240, 159, 146, 150];
    /// let str_cow = unsafe { <StringCow>::from_utf8_unchecked(&bytes) };
    /// assert_eq!(&str_cow, "💖");
    /// ```
    #[cfg(feature = "unsafe")]
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(other: &'s [u8]) -> Self {
        Self::from_ref(unsafe { str::from_utf8_unchecked(other) })
    }

    /// Get a reference to the `String`.
    #[inline]
    #[must_use]
    pub fn get_ref(&self) -> StringRef<'_> {
        self.inner.get_ref()
    }

    /// Clone the data into an owned-type.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> String<B> {
        self.inner.into_owned()
    }

    /// Extracts a string slice containing the entire `StringCow`.
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

impl<'s, B: HeapStr> StringCowInner<'s, B> {
    #[inline]
    fn get_ref(&self) -> StringRef<'_> {
        match self {
            Self::Borrowed(s) => StringRef::from_ref(s),
            Self::Owned(s) => s.get_ref(),
        }
    }

    #[inline]
    fn into_owned(self) -> String<B> {
        match self {
            Self::Borrowed(s) => String::from_ref(s),
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

impl<B: HeapStr> Deref for StringCow<'_, B> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> Eq for StringCow<'_, B> {}

impl<'s, B: HeapStr> PartialEq<StringCow<'s, B>> for StringCow<'s, B> {
    #[inline]
    fn eq(&self, other: &StringCow<'s, B>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<B: HeapStr> PartialEq<str> for StringCow<'_, B> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'s, B: HeapStr> PartialEq<&'s str> for StringCow<'s, B> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<B: HeapStr> PartialEq<String> for StringCow<'_, B> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<B: HeapStr> Ord for StringCow<'_, B> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<B: HeapStr> PartialOrd for StringCow<'_, B> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: HeapStr> Hash for StringCow<'_, B> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<B: HeapStr> fmt::Debug for StringCow<'_, B> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<B: HeapStr> fmt::Display for StringCow<'_, B> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<B: HeapStr> AsRef<str> for StringCow<'_, B> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> AsRef<[u8]> for StringCow<'_, B> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<B: HeapStr> AsRef<OsStr> for StringCow<'_, B> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (**self).as_ref()
    }
}

impl<B: HeapStr> AsRef<Path> for StringCow<'_, B> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl<B: HeapStr> Borrow<str> for StringCow<'_, B> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<B> Default for StringCow<'_, B> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<B: HeapStr> From<String<B>> for StringCow<'_, B> {
    #[inline]
    fn from(other: String<B>) -> Self {
        let inner = StringCowInner::Owned(other);
        Self { inner }
    }
}

impl<'s, B: HeapStr> From<&'s String<B>> for StringCow<'s, B> {
    #[inline]
    fn from(other: &'s String<B>) -> Self {
        let other = other.get_ref();
        other.into()
    }
}

impl<'s, B: HeapStr> From<StringRef<'s>> for StringCow<'s, B> {
    #[inline]
    fn from(other: StringRef<'s>) -> Self {
        match other.inner {
            StringRefInner::Borrowed(s) => Self::from_ref(s),
            StringRefInner::Singleton(s) => Self::from_static(s),
        }
    }
}

impl<'s, B: HeapStr> From<&'s StringRef<'s>> for StringCow<'s, B> {
    #[inline]
    fn from(other: &'s StringRef<'s>) -> Self {
        match other.inner {
            StringRefInner::Borrowed(s) => Self::from_ref(s),
            StringRefInner::Singleton(s) => Self::from_static(s),
        }
    }
}

impl<B: HeapStr> From<StdString> for StringCow<'_, B> {
    #[inline]
    fn from(other: StdString) -> Self {
        Self::from_string(other)
    }
}

impl<'s, B: HeapStr> From<&'s StdString> for StringCow<'s, B> {
    #[inline]
    fn from(other: &'s StdString) -> Self {
        Self::from_ref(other.as_str())
    }
}

impl<B: HeapStr> From<BoxedStr> for StringCow<'_, B> {
    #[inline]
    fn from(other: BoxedStr) -> Self {
        // Since the memory is already allocated, don't bother moving it into a FixedString
        Self::from_boxed(other)
    }
}

impl<'s, B: HeapStr> From<&'s BoxedStr> for StringCow<'s, B> {
    #[inline]
    fn from(other: &'s BoxedStr) -> Self {
        Self::from_ref(other)
    }
}

impl<'s, B: HeapStr> From<&'s str> for StringCow<'s, B> {
    #[inline]
    fn from(other: &'s str) -> Self {
        Self::from_ref(other)
    }
}

impl<B: HeapStr> FromStr for StringCow<'_, B> {
    type Err = Infallible;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_string(s.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn test_size() {
        println!("StringCow: {}", size_of::<StringCow<'static>>());
    }
}
