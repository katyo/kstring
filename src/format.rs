use crate::{backend::HeapStr, KString, KStringCow, KStringRef, StackString};
use core::fmt;
use std::{borrow::Cow, string::String as StdString};

/// A macro for formatting arguments into a `KString`.
///
/// This macro is similar to Rust's standard `format!` macro but outputs a `KString`
/// instead of a `String`. It uses the `fmt::Write` trait internally to append formatted
/// text efficiently.
///
/// # Examples
///
/// ```
/// # use lstring::{kformat, KString};
/// let s: KString = kformat!("Hello, {}!", "world");
/// assert_eq!(s, "Hello, world!");
/// ```
#[macro_export]
macro_rules! kformat {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut ks = $crate::KStringWriter::default();
        write!(ks, $($arg)*).unwrap();
        ks.into()
    }};
}

/// A writer that can append formatted text to either an inline `StackString` or a heap-allocated `String`.
///
/// The `KStringWriter` starts writing into a stack-allocated `StackString` for efficiency. If the capacity
/// of the inline storage is exceeded, it switches to using a heap-allocated `String`. This allows for
/// efficient formatting without unnecessary allocations in cases where the formatted text fits within the
/// inline buffer.
///
/// # Examples
///
/// ```
/// use lstring::{KString, KStringWriter};
/// use core::fmt::Write;
///
/// let mut writer = KStringWriter::default();
/// write!(writer, "Hello, {}!", "world").unwrap();
/// let s: KString = writer.into();
/// assert_eq!(s, "Hello, world!");
/// ```
#[derive(Clone)]
pub enum KStringWriter {
    Inline(StackString<{ KString::MAX_INLINE_LEN }>),
    Owned(String),
}

impl KStringWriter {
    pub fn new() -> Self {
        Self::Inline(StackString::default())
    }

    /// Reserves additional capacity for the `KStringWriter`.
    ///
    /// If the writer is currently using an inline `StackString` and the requested capacity exceeds
    /// its maximum inline length, it will switch to a heap-allocated `String`. This method ensures
    /// that subsequent writes do not require reallocations if the specified capacity is available.
    pub fn reserve(&mut self, additional: usize) {
        match self {
            KStringWriter::Inline(inline) => {
                let capacity = inline.len() + additional;
                if capacity > KString::MAX_INLINE_LEN {
                    // Turn to owned
                    let mut owned = String::with_capacity(capacity);
                    owned.push_str(inline);
                    *self = KStringWriter::Owned(owned);
                }
            }
            KStringWriter::Owned(owned) => {
                owned.reserve(additional);
            }
        }
    }

    /// Appends a single character to the `KStringWriter`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::KStringWriter;
    ///
    /// let mut writer = KStringWriter::default();
    /// writer.push('a');
    /// assert_eq!(writer.as_ref(), "a");
    /// ```
    pub fn push(&mut self, ch: char) {
        if let Self::Inline(inline) = self {
            if inline.try_push_char(ch) {
                return;
            }
            *self = Self::Owned(String::from(&*inline));
        }
        if let Self::Owned(owned) = self {
            owned.push(ch);
        } else {
            unreachable!();
        }
    }

    /// Appends a string slice to the `KStringWriter`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::KStringWriter;
    ///
    /// let mut writer = KStringWriter::default();
    /// writer.push_str("abc");
    /// assert_eq!(writer.as_ref(), "abc");
    /// ```
    pub fn push_str(&mut self, s: &str) {
        if let Self::Inline(inline) = self {
            if inline.try_push(s) {
                return;
            }
            *self = Self::Owned(String::from(&*inline));
        }
        if let Self::Owned(owned) = self {
            owned.push_str(s);
        } else {
            unreachable!();
        }
    }
}

impl AsRef<str> for KStringWriter {
    fn as_ref(&self) -> &str {
        match self {
            Self::Inline(inline) => inline.as_ref(),
            Self::Owned(owned) => owned.as_ref(),
        }
    }
}

impl Default for KStringWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: HeapStr> From<KStringWriter> for KString<B> {
    fn from(writer: KStringWriter) -> Self {
        match writer {
            KStringWriter::Inline(inline) => inline.into(),
            KStringWriter::Owned(owned) => owned.into(),
        }
    }
}

impl fmt::Write for KStringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl Extend<char> for KStringWriter {
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        for c in iterator {
            self.push(c);
        }
    }
}

impl<'a> Extend<&'a char> for KStringWriter {
    fn extend<I: IntoIterator<Item = &'a char>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<'a> Extend<&'a str> for KStringWriter {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(s);
        }
    }
}

impl Extend<Box<str>> for KStringWriter {
    fn extend<I: IntoIterator<Item = Box<str>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl Extend<StdString> for KStringWriter {
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<'a> Extend<Cow<'a, str>> for KStringWriter {
    fn extend<I: IntoIterator<Item = Cow<'a, str>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<'a, B: HeapStr> Extend<&'a KString<B>> for KStringWriter {
    fn extend<I: IntoIterator<Item = &'a KString<B>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(s);
        }
    }
}

impl<B: HeapStr> Extend<KString<B>> for KStringWriter {
    fn extend<I: IntoIterator<Item = KString<B>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<'a, 's, B: HeapStr> Extend<&'a KStringCow<'s, B>> for KStringWriter {
    fn extend<I: IntoIterator<Item = &'a KStringCow<'s, B>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(s);
        }
    }
}

impl<'s, B: HeapStr> Extend<KStringCow<'s, B>> for KStringWriter {
    fn extend<I: IntoIterator<Item = KStringCow<'s, B>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<'a, 's> Extend<&'a KStringRef<'s>> for KStringWriter {
    fn extend<I: IntoIterator<Item = &'a KStringRef<'s>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(s);
        }
    }
}

impl<'s> Extend<KStringRef<'s>> for KStringWriter {
    fn extend<I: IntoIterator<Item = KStringRef<'s>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<B: HeapStr> FromIterator<char> for KString<B> {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, B: HeapStr> FromIterator<&'a char> for KString<B> {
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> Self {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, B: HeapStr> FromIterator<&'a str> for KString<B> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<B: HeapStr> FromIterator<Box<str>> for KString<B> {
    fn from_iter<I: IntoIterator<Item = Box<str>>>(iter: I) -> KString<B> {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, B: HeapStr> FromIterator<Cow<'a, str>> for KString<B> {
    fn from_iter<I: IntoIterator<Item = Cow<'a, str>>>(iter: I) -> KString<B> {
        let mut iterator = iter.into_iter();

        match iterator.next() {
            None => KString::new(),
            Some(cow) => {
                let mut writer = cow.into_owned();
                writer.extend(iterator);
                writer.into()
            }
        }
    }
}

impl<'a, D: HeapStr, B: HeapStr> FromIterator<&'a KString<D>> for KString<B> {
    fn from_iter<I: IntoIterator<Item = &'a KString<D>>>(iter: I) -> KString<B> {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<D: HeapStr, B: HeapStr> FromIterator<KString<D>> for KString<B> {
    fn from_iter<I: IntoIterator<Item = KString<D>>>(iter: I) -> KString<B> {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, 's, B: HeapStr, D: HeapStr> FromIterator<&'a KStringCow<'s, D>> for KString<B> {
    fn from_iter<I: IntoIterator<Item = &'a KStringCow<'s, D>>>(iter: I) -> KString<B> {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'s, B: HeapStr, D: HeapStr> FromIterator<KStringCow<'s, D>> for KString<B> {
    fn from_iter<I: IntoIterator<Item = KStringCow<'s, D>>>(iter: I) -> KString<B> {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, 's, B: HeapStr> FromIterator<&'a KStringRef<'s>> for KString<B> {
    fn from_iter<I: IntoIterator<Item = &'a KStringRef<'s>>>(iter: I) -> KString<B> {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'s, B: HeapStr> FromIterator<KStringRef<'s>> for KString<B> {
    fn from_iter<I: IntoIterator<Item = KStringRef<'s>>>(iter: I) -> KString<B> {
        let mut writer = KStringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kstringwriter_new() {
        let writer = KStringWriter::new();
        assert!(matches!(writer, KStringWriter::Inline(_)));
    }

    #[test]
    fn test_kstringwriter_push_char_inline() {
        let mut writer = KStringWriter::new();
        writer.push('a');
        assert_eq!(writer.as_ref(), "a");
    }

    #[test]
    fn test_kstringwriter_push_str_inline() {
        let mut writer = KStringWriter::new();
        writer.push_str("abc");
        assert_eq!(writer.as_ref(), "abc");
    }

    #[test]
    fn test_kstringwriter_push_char_to_owned() {
        let mut writer = KStringWriter::new();
        for _ in 0..=KString::MAX_INLINE_LEN {
            writer.push('a');
        }
        assert!(matches!(writer, KStringWriter::Owned(_)));
        assert_eq!(writer.as_ref(), "a".repeat(KString::MAX_INLINE_LEN + 1));
    }

    #[test]
    fn test_kstringwriter_push_str_to_owned() {
        let mut writer = KStringWriter::new();
        for _ in 0..=KString::MAX_INLINE_LEN / 3 {
            writer.push_str("abc");
        }
        assert!(matches!(writer, KStringWriter::Owned(_)));
        assert_eq!(
            writer.as_ref(),
            "abc".repeat(KString::MAX_INLINE_LEN / 3 + 1)
        );
    }

    #[test]
    fn test_kstringwriter_reserve_inline() {
        let mut writer = KStringWriter::new();
        writer.reserve(KString::MAX_INLINE_LEN);
        assert!(matches!(writer, KStringWriter::Inline(_)));

        let mut writer = KStringWriter::new();
        writer.push('a');
        writer.reserve(KString::MAX_INLINE_LEN - 1);
        assert!(matches!(writer, KStringWriter::Inline(_)));
    }

    #[test]
    fn test_kstringwriter_reserve_to_owned() {
        let mut writer = KStringWriter::new();
        writer.reserve(KString::MAX_INLINE_LEN + 1);
        assert!(matches!(writer, KStringWriter::Owned(_)));

        let mut writer = KStringWriter::new();
        writer.push('a');
        writer.reserve(KString::MAX_INLINE_LEN);
        assert!(matches!(writer, KStringWriter::Owned(_)));
    }

    #[test]
    fn test_kstringwriter_from_writer_inline() {
        let mut writer = KStringWriter::new();
        writer.push_str("abc");
        let kstring: KString = writer.into();
        assert_eq!(&kstring, "abc");
    }

    #[test]
    fn test_kstringwriter_from_writer_owned() {
        let mut writer = KStringWriter::new();
        for _ in 0..=KString::MAX_INLINE_LEN / 3 {
            writer.push_str("abc");
        }
        let kstring: KString = writer.into();
        assert_eq!(&kstring, &"abc".repeat(KString::MAX_INLINE_LEN / 3 + 1));
    }

    #[test]
    fn test_kstringwriter_extend_char() {
        let mut writer = KStringWriter::new();
        writer.extend("hello".chars());
        assert_eq!(writer.as_ref(), "hello");
    }

    #[test]
    fn test_kstringwriter_extend_str() {
        let mut writer = KStringWriter::new();
        writer.extend(["he", "ll", "o"]);
        assert_eq!(writer.as_ref(), "hello");
    }
}
