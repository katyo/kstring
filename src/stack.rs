use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    str,
};
use std::{borrow::Borrow, ffi::OsStr, path::Path};

pub(crate) type Len = u8;

/// Fixed-size stack-allocated string
#[derive(Copy, Clone)]
pub struct StackString<const CAPACITY: usize> {
    len: Len,
    buffer: StrBuffer<CAPACITY>,
}

impl<const CAPACITY: usize> StackString<CAPACITY> {
    pub const CAPACITY: usize = CAPACITY;
    pub const EMPTY: Self = Self::empty();

    const fn empty() -> Self {
        Self {
            len: 0,
            buffer: StrBuffer::empty(),
        }
    }

    /// Create a `StackString` from a `&str`, if it'll fit within `Self::CAPACITY`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let s = StackString::<3>::try_new("foo");
    /// assert_eq!(s.as_deref(), Some("foo"));
    /// let s = StackString::<3>::try_new("foobar");
    /// assert_eq!(s, None);
    /// ```
    #[inline]
    #[must_use]
    pub fn try_new(s: &str) -> Option<Self> {
        if s.len() <= Self::CAPACITY {
            #[cfg(feature = "unsafe")]
            let stack = {
                unsafe {
                    // SAFETY: We've confirmed `len` is within size
                    Self::new_unchecked(s)
                }
            };
            #[cfg(not(feature = "unsafe"))]
            let stack = { Self::new(s) };
            Some(stack)
        } else {
            None
        }
    }

    /// Create a `StackString` from a `&str`
    ///
    /// # Panic
    ///
    /// Calling this function with a string larger than `Self::CAPACITY` will panic
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let s = StackString::<3>::new("foo");
    /// assert_eq!(s, "foo");
    /// ```
    #[inline]
    #[must_use]
    pub fn new(s: &str) -> Self {
        let len = s.len() as u8;
        debug_assert!(Self::CAPACITY <= Len::MAX.into());
        let buffer = StrBuffer::new(s);
        Self { len, buffer }
    }

    /// Create a `StackString` from a `&str`
    ///
    /// # Safety
    ///
    /// Calling this function with a string larger than `Self::CAPACITY` is undefined behavior.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let s = unsafe {
    ///     // SAFETY: Literal is short-enough
    ///     StackString::<3>::new_unchecked("foo")
    /// };
    /// assert_eq!(s, "foo");
    /// ```
    #[inline]
    #[must_use]
    #[cfg(feature = "unsafe")]
    pub unsafe fn new_unchecked(s: &str) -> Self {
        let len = s.len() as u8;
        debug_assert!(Self::CAPACITY <= Len::MAX.into());
        let buffer = StrBuffer::new_unchecked(s);
        Self { len, buffer }
    }

    /// Extracts a string slice containing the entire `StackString`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let s = StackString::<3>::try_new("foo").unwrap();
    ///
    /// assert_eq!("foo", s.as_str());
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        let len = self.len as usize;
        #[cfg(feature = "unsafe")]
        unsafe {
            // SAFETY: Constructors guarantee that `buffer[..len]` is a `str`,
            // and we don't mutate the data afterwards.
            self.buffer.as_str_unchecked(len)
        }
        #[cfg(not(feature = "unsafe"))]
        self.buffer.as_str(len)
    }

    /// Converts a `StackString` into a mutable string slice.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = StackString::<6>::try_new("foobar").unwrap();
    /// let s_mut_str = s.as_mut_str();
    ///
    /// s_mut_str.make_ascii_uppercase();
    ///
    /// assert_eq!("FOOBAR", s_mut_str);
    /// ```
    #[inline]
    #[must_use]
    pub fn as_mut_str(&mut self) -> &mut str {
        let len = self.len as usize;
        #[cfg(feature = "unsafe")]
        unsafe {
            // SAFETY: Constructors guarantee that `buffer[..len]` is a `str`,
            // and we don't mutate the data afterwards.
            self.buffer.as_mut_str_unchecked(len)
        }
        #[cfg(not(feature = "unsafe"))]
        self.buffer.as_mut_str(len)
    }

    /// Returns the length of this `StasckString`, in bytes, not [`char`]s or
    /// graphemes. In other words, it might not be what a human considers the
    /// length of the string.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let a = StackString::<3>::try_new("foo").unwrap();
    /// assert_eq!(a.len(), 3);
    ///
    /// let fancy_f = StackString::<4>::try_new("ƒoo").unwrap();
    /// assert_eq!(fancy_f.len(), 4);
    /// assert_eq!(fancy_f.chars().count(), 3);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if this `StackString` has a length of zero, and `false` otherwise.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut v = StackString::<20>::EMPTY;
    /// assert!(v.is_empty());
    ///
    /// let a = StackString::<3>::try_new("foo").unwrap();
    /// assert!(!a.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Truncates this `StackString`, removing all contents.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = StackString::<3>::try_new("foo").unwrap();
    ///
    /// s.clear();
    ///
    /// assert!(s.is_empty());
    /// assert_eq!(0, s.len());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Shortens this `StackString` to the specified length.
    ///
    /// If `new_len` is greater than the string's current length, this has no
    /// effect.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the string
    ///
    /// # Panics
    ///
    /// Panics if `new_len` does not lie on a [`char`] boundary.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = StackString::<5>::try_new("hello").unwrap();
    ///
    /// s.truncate(2);
    ///
    /// assert_eq!(s, "he");
    /// ```
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert!(self.is_char_boundary(new_len));
            self.len = new_len as u8;
        }
    }

    /// Tries to append a `&str` to the existing `StackString`, if it'll fit within `Self::CAPACITY`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = StackString::<6>::try_new("foo").unwrap();
    /// assert_eq!(s, "foo");
    ///
    /// s.try_push("bar");
    /// assert_eq!(s, "foobar");
    ///
    /// assert!(!s.try_push("baz"));
    /// ```
    #[inline]
    pub fn try_push(&mut self, s: &str) -> bool {
        let new_len = self.len as usize + s.len();
        if new_len <= Self::CAPACITY {
            #[cfg(feature = "unsafe")]
            unsafe {
                // SAFETY: We've confirmed `new_len` is within size
                self.push_unchecked(s)
            };
            #[cfg(not(feature = "unsafe"))]
            self.push(s);
            true
        } else {
            false
        }
    }

    /// Appends a `&str` to the existing `StackString`.
    ///
    /// # Panic
    ///
    /// Calling this function with a string that would make the total length larger than `Self::CAPACITY` will panic.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = StackString::<6>::try_new("foo").unwrap();
    /// assert_eq!(s, "foo");
    ///
    /// s.push("bar");
    /// assert_eq!(s, "foobar");
    /// ```
    #[inline]
    pub fn push(&mut self, s: &str) {
        let new_len = self.len as usize + s.len();
        debug_assert!(new_len <= Self::CAPACITY);
        if let Some(slice) = self.buffer.0.get_mut(self.len as usize..new_len) {
            slice.copy_from_slice(s.as_bytes());
        } else {
            panic!("Appending `{}` would exceed capacity {}", s, Self::CAPACITY);
        }
        self.len = new_len as u8;
    }

    /// Appends a `&str` to the existing `StackString`.
    ///
    /// # Safety
    ///
    /// Calling this function with a string that would make the total length larger than `Self::CAPACITY` is undefined behavior.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = unsafe {
    ///     StackString::<6>::new_unchecked("foo")
    /// };
    /// assert_eq!(s, "foo");
    ///
    /// unsafe {
    ///     // SAFETY: Literal is short-enough
    ///     s.push_unchecked("bar");
    /// }
    /// assert_eq!(s, "foobar");
    /// ```
    #[inline]
    #[cfg(feature = "unsafe")]
    pub unsafe fn push_unchecked(&mut self, s: &str) {
        let new_len = self.len as usize + s.len();
        debug_assert!(new_len <= Self::CAPACITY);
        self.buffer
            .0
            .get_unchecked_mut(self.len as usize..new_len)
            .copy_from_slice(s.as_bytes());
        self.len = new_len as u8;
    }

    /// Tries to append a `char` to the existing `StackString`, if it'll fit within `Self::CAPACITY`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = StackString::<5>::try_new("Hell").unwrap();
    /// assert_eq!(s, "Hell");
    ///
    /// s.try_push_char('o');
    /// assert_eq!(s, "Hello");
    ///
    /// assert!(!s.try_push_char('b'));
    /// ```
    #[inline]
    pub fn try_push_char(&mut self, c: char) -> bool {
        let new_len = self.len as usize + c.len_utf8();
        if new_len <= Self::CAPACITY {
            #[cfg(feature = "unsafe")]
            unsafe {
                // SAFETY: We've confirmed `new_len` is within size
                self.push_char_unchecked(c)
            };
            #[cfg(not(feature = "unsafe"))]
            self.push_char(c);
            true
        } else {
            false
        }
    }

    /// Appends a `char` to the existing `StackString`.
    ///
    /// # Panic
    ///
    /// Calling this function with a character that would make the total length larger than `Self::CAPACITY` will panic.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = StackString::<5>::try_new("Hell").unwrap();
    /// assert_eq!(s, "Hell");
    ///
    /// s.push_char('o');
    /// assert_eq!(s, "Hello");
    /// ```
    #[inline]
    pub fn push_char(&mut self, c: char) {
        let new_len = self.len as usize + c.len_utf8();
        debug_assert!(new_len <= Self::CAPACITY);
        let mut buf = [0; 4];
        let s = c.encode_utf8(&mut buf);
        if let Some(slice) = self.buffer.0.get_mut(self.len as usize..new_len) {
            slice.copy_from_slice(s.as_bytes());
        } else {
            panic!("Appending `{}` would exceed capacity {}", c, Self::CAPACITY);
        }
        self.len = new_len as u8;
    }

    /// Appends a `char` to the existing `StackString`.
    ///
    /// # Safety
    ///
    /// Calling this function with a character that would make the total length larger than `Self::CAPACITY` is undefined behavior.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use lstring::StackString;
    /// let mut s = unsafe {
    ///     StackString::<5>::new_unchecked("Hell")
    /// };
    /// assert_eq!(s, "Hell");
    ///
    /// unsafe {
    ///     // SAFETY: Literal is short-enough
    ///     s.push_char_unchecked('o');
    /// }
    /// assert_eq!(s, "Hello");
    /// ```
    #[inline]
    #[cfg(feature = "unsafe")]
    pub unsafe fn push_char_unchecked(&mut self, c: char) {
        let new_len = self.len as usize + c.len_utf8();
        debug_assert!(new_len <= Self::CAPACITY);
        let mut buf = [0; 4];
        let s = c.encode_utf8(&mut buf);
        self.buffer
            .0
            .get_unchecked_mut(self.len as usize..new_len)
            .copy_from_slice(s.as_bytes());
        self.len = new_len as u8;
    }
}

impl<const CAPACITY: usize> Default for StackString<CAPACITY> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const CAPACITY: usize> Deref for StackString<CAPACITY> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAPACITY: usize> From<&StackString<CAPACITY>> for String {
    fn from(other: &StackString<CAPACITY>) -> Self {
        Self::from(other.as_str())
    }
}

impl<const CAPACITY: usize> From<StackString<CAPACITY>> for String {
    fn from(other: StackString<CAPACITY>) -> Self {
        Self::from(other.as_str())
    }
}

impl<const CAPACITY: usize> TryFrom<&str> for StackString<CAPACITY> {
    type Error = ();
    fn try_from(other: &str) -> Result<Self, Self::Error> {
        Self::try_new(other).ok_or(())
    }
}

impl<const CAPACITY: usize> TryFrom<&String> for StackString<CAPACITY> {
    type Error = ();
    fn try_from(other: &String) -> Result<Self, Self::Error> {
        Self::try_new(other).ok_or(())
    }
}

impl<const CAPACITY: usize> TryFrom<String> for StackString<CAPACITY> {
    type Error = ();
    fn try_from(other: String) -> Result<Self, Self::Error> {
        Self::try_new(&other).ok_or(())
    }
}

impl<const CAPACITY: usize> Eq for StackString<CAPACITY> {}

impl<const C1: usize, const C2: usize> PartialEq<StackString<C1>> for StackString<C2> {
    #[inline]
    fn eq(&self, other: &StackString<C1>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<const CAPACITY: usize> PartialEq<str> for StackString<CAPACITY> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<const CAPACITY: usize> PartialEq<&str> for StackString<CAPACITY> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<const CAPACITY: usize> PartialEq<String> for StackString<CAPACITY> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<const CAPACITY: usize> Ord for StackString<CAPACITY> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<const C1: usize, const C2: usize> PartialOrd<StackString<C1>> for StackString<C2> {
    #[inline]
    fn partial_cmp(&self, other: &StackString<C1>) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<const CAPACITY: usize> PartialOrd<str> for StackString<CAPACITY> {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl<const CAPACITY: usize> PartialOrd<&str> for StackString<CAPACITY> {
    #[inline]
    fn partial_cmp(&self, other: &&str) -> Option<Ordering> {
        self.as_str().partial_cmp(*other)
    }
}

impl<const CAPACITY: usize> PartialOrd<String> for StackString<CAPACITY> {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<const CAPACITY: usize> Hash for StackString<CAPACITY> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<const CAPACITY: usize> fmt::Debug for StackString<CAPACITY> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const CAPACITY: usize> fmt::Display for StackString<CAPACITY> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<const CAPACITY: usize> AsRef<str> for StackString<CAPACITY> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAPACITY: usize> AsRef<[u8]> for StackString<CAPACITY> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<const CAPACITY: usize> AsRef<OsStr> for StackString<CAPACITY> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (**self).as_ref()
    }
}

impl<const CAPACITY: usize> AsRef<Path> for StackString<CAPACITY> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl<const CAPACITY: usize> Borrow<str> for StackString<CAPACITY> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct StrBuffer<const CAPACITY: usize>([u8; CAPACITY]);

impl<const CAPACITY: usize> StrBuffer<CAPACITY> {
    pub(crate) const fn empty() -> Self {
        let array = [0; CAPACITY];
        StrBuffer(array)
    }

    #[inline]
    pub(crate) fn new(s: &str) -> Self {
        let len = s.len();
        debug_assert!(len <= CAPACITY);
        let mut buffer = Self::default();
        if let Some(buffer) = buffer.0.get_mut(..len) {
            buffer.copy_from_slice(s.as_bytes());
        } else {
            panic!("`{s}` is larger than capacity {CAPACITY}");
        }
        buffer
    }
}

#[cfg(not(feature = "unsafe"))]
impl<const CAPACITY: usize> StrBuffer<CAPACITY> {
    #[inline]
    pub(crate) fn as_str(&self, len: usize) -> &str {
        let slice = self.0.get(..len).unwrap();
        str::from_utf8(slice).unwrap()
    }

    #[inline]
    pub(crate) fn as_mut_str(&mut self, len: usize) -> &mut str {
        let slice = self.0.get_mut(..len).unwrap();
        str::from_utf8_mut(slice).unwrap()
    }
}

#[cfg(feature = "unsafe")]
impl<const CAPACITY: usize> StrBuffer<CAPACITY> {
    #[inline]
    pub(crate) unsafe fn new_unchecked(s: &str) -> Self {
        let len = s.len();
        debug_assert!(len <= CAPACITY);
        let mut buffer = Self::default();
        buffer
            .0
            .get_unchecked_mut(..len)
            .copy_from_slice(s.as_bytes());
        buffer
    }

    #[inline]
    pub(crate) unsafe fn as_str_unchecked(&self, len: usize) -> &str {
        let slice = self.0.get_unchecked(..len);
        str::from_utf8_unchecked(slice)
    }

    #[inline]
    pub(crate) unsafe fn as_mut_str_unchecked(&mut self, len: usize) -> &mut str {
        let slice = self.0.get_unchecked_mut(..len);
        str::from_utf8_unchecked_mut(slice)
    }
}

impl<const CAPACITY: usize> Default for StrBuffer<CAPACITY> {
    fn default() -> Self {
        Self::empty()
    }
}
