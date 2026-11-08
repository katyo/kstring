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
    string::{FromUtf8Error, FromUtf16Error},
};

use crate::{
    HeapStr, StdString, StringCow, StringInner, StringRef,
    backend::{BoxedStr, DefaultStr},
    stack::StackString,
};

/// A UTF-8 encoded, immutable string.
#[derive(Clone)]
#[repr(transparent)]
pub struct String<B = DefaultStr> {
    pub(crate) inner: StringInner<B>,
}

impl<B> String<B> {
    pub const EMPTY: Self = Self::from_static("");

    /// Create a new empty `String`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Create a reference to a `'static` data.
    #[inline]
    #[must_use]
    pub const fn from_static(other: &'static str) -> Self {
        Self {
            inner: StringInner::from_static(other),
        }
    }

    /// Create an inline string, if possible
    #[inline]
    #[must_use]
    pub fn try_inline(other: &str) -> Option<Self> {
        StringInner::try_inline(other).map(|inner| Self { inner })
    }
}

impl String {
    /// Maximum length of string which can be created without alloc in heap
    pub const MAX_INLINE_LEN: usize = crate::inner::CAPACITY;
}

impl<B: HeapStr> String<B> {
    /// Create an owned `String`.
    #[inline]
    #[must_use]
    pub fn from_boxed(other: BoxedStr) -> Self {
        Self {
            inner: StringInner::from_boxed(other),
        }
    }

    /// Create an owned `String`.
    #[inline]
    #[must_use]
    pub fn from_string(other: StdString) -> Self {
        Self {
            inner: StringInner::from_string(other),
        }
    }

    /// Create an owned `String` optimally from a reference.
    #[inline]
    #[must_use]
    pub fn from_ref(other: &str) -> Self {
        Self {
            inner: StringInner::from_ref(other),
        }
    }

    /// Creates a new `String` from a UTF-8 byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::String;
    ///
    /// let sparkle_heart = vec![240, 159, 146, 150];
    /// let string = <String>::from_utf8(sparkle_heart).unwrap();
    /// assert_eq!(&string, "💖");
    /// ```
    #[inline]
    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error> {
        StdString::from_utf8(vec).map(Self::from_string)
    }

    /// Creates a new `String` from a UTF-8 byte slice without checking that the bytes are valid UTF-8.
    ///
    /// # Safety
    ///
    /// The bytes passed in must be valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::String;
    ///
    /// let sparkle_heart = vec![240, 159, 146, 150];
    /// let string = unsafe { <String>::from_utf8_unchecked(sparkle_heart) };
    /// assert_eq!(&string, "💖");
    /// ```
    #[cfg(feature = "unsafe")]
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(bytes: Vec<u8>) -> Self {
        Self::from_string(unsafe { StdString::from_utf8_unchecked(bytes) })
    }

    /// Creates a new `String` from a UTF-16 byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::String;
    ///
    /// let sparkle_heart = [0xD83D, 0xDC96];
    /// let string = <String>::from_utf16(&sparkle_heart).unwrap();
    /// assert_eq!(&string, "💖");
    /// ```
    #[inline]
    pub fn from_utf16(other: &[u16]) -> Result<Self, FromUtf16Error> {
        StdString::from_utf16(other).map(Self::from_string)
    }

    /// Get a reference to the `String`.
    #[inline]
    #[must_use]
    pub fn get_ref(&self) -> StringRef<'_> {
        self.inner.get_ref()
    }

    /// Extracts a string slice containing the entire `String`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Extracts a mutable string slice containing the entire `String`.
    #[inline]
    #[must_use]
    pub fn get_mut_str(&mut self) -> Option<&mut str> {
        self.inner.get_mut_str()
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    #[inline]
    #[must_use]
    pub fn into_string(self) -> StdString {
        StdString::from(self.into_boxed_str())
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
    pub fn into_cow_str(self) -> Cow<'static, str> {
        self.inner.into_cow_str()
    }
}

impl<B: HeapStr> Deref for String<B> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> Eq for String<B> {}

impl<B: HeapStr> PartialEq<String<B>> for String<B> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<B: HeapStr> PartialEq<str> for String<B> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<B: HeapStr> PartialEq<&str> for String<B> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<B: HeapStr> PartialEq<StdString> for String<B> {
    #[inline]
    fn eq(&self, other: &StdString) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<B: HeapStr> Ord for String<B> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<B: HeapStr> PartialOrd for String<B> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: HeapStr> Hash for String<B> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<B: HeapStr> fmt::Debug for String<B> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<B: HeapStr> fmt::Display for String<B> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<B: HeapStr> AsRef<str> for String<B> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> AsRef<[u8]> for String<B> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<B: HeapStr> AsRef<OsStr> for String<B> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (**self).as_ref()
    }
}

impl<B: HeapStr> AsRef<Path> for String<B> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl<B: HeapStr> Borrow<str> for String<B> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> Default for String<B> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<B: HeapStr> From<StackString<{ String::MAX_INLINE_LEN }>> for String<B> {
    fn from(other: StackString<{ String::MAX_INLINE_LEN }>) -> Self {
        Self {
            inner: StringInner::from_inline(other),
        }
    }
}

impl<'s, B: HeapStr> From<StringRef<'s>> for String<B> {
    #[inline]
    fn from(other: StringRef<'s>) -> Self {
        other.to_owned()
    }
}

impl<'s, B: HeapStr> From<&'s StringRef<'s>> for String<B> {
    #[inline]
    fn from(other: &'s StringRef<'s>) -> Self {
        other.to_owned()
    }
}

impl<'s, B: HeapStr> From<StringCow<'s, B>> for String<B> {
    #[inline]
    fn from(other: StringCow<'s, B>) -> Self {
        other.into_owned()
    }
}

impl<'s, B: HeapStr> From<&'s StringCow<'s, B>> for String<B> {
    #[inline]
    fn from(other: &'s StringCow<'s, B>) -> Self {
        other.clone().into_owned()
    }
}

impl<B: HeapStr> From<StdString> for String<B> {
    #[inline]
    fn from(other: StdString) -> Self {
        Self::from_string(other)
    }
}

impl<'s, B: HeapStr> From<&'s StdString> for String<B> {
    #[inline]
    fn from(other: &'s StdString) -> Self {
        Self::from_ref(other)
    }
}

impl<B: HeapStr> From<BoxedStr> for String<B> {
    #[inline]
    fn from(other: BoxedStr) -> Self {
        Self::from_boxed(other)
    }
}

impl<'s, B: HeapStr> From<&'s BoxedStr> for String<B> {
    #[inline]
    fn from(other: &'s BoxedStr) -> Self {
        Self::from_ref(other)
    }
}

impl<B: HeapStr> From<&str> for String<B> {
    #[inline]
    fn from(other: &str) -> Self {
        Self::from_ref(other)
    }
}

impl<B: HeapStr> FromStr for String<B> {
    type Err = Infallible;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_ref(s))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn test_size() {
        println!("String: {}", size_of::<String>());
    }
}
