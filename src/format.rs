use crate::{StackString, String, StringCow, StringRef, backend::HeapStr};
use core::fmt;
use std::{borrow::Cow, string::String as StdString};

/// A macro for formatting arguments into a `String`.
///
/// This macro is similar to Rust's standard `format!` macro but outputs a `String`
/// instead of a `String`. It uses the `fmt::Write` trait internally to append formatted
/// text efficiently.
///
/// # Examples
///
/// ```
/// # use lstring::{format, String};
/// let s: String = format!("Hello, {}!", "world");
/// assert_eq!(s, "Hello, world!");
/// ```
#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut ks = $crate::StringWriter::default();
        write!(ks, $($arg)*).unwrap();
        ks.into()
    }};
}

/// A writer that can append formatted text to either an inline `StackString` or a heap-allocated `String`.
///
/// The `StringWriter` starts writing into a stack-allocated `StackString` for efficiency. If the capacity
/// of the inline storage is exceeded, it switches to using a heap-allocated `String`. This allows for
/// efficient formatting without unnecessary allocations in cases where the formatted text fits within the
/// inline buffer.
///
/// # Examples
///
/// ```
/// use lstring::{String, StringWriter};
/// use core::fmt::Write;
///
/// let mut writer = StringWriter::default();
/// write!(writer, "Hello, {}!", "world").unwrap();
/// let s: String = writer.into();
/// assert_eq!(s, "Hello, world!");
/// ```
#[derive(Clone)]
pub enum StringWriter {
    Inline(StackString<{ String::MAX_INLINE_LEN }>),
    Owned(StdString),
}

impl StringWriter {
    pub fn new() -> Self {
        Self::Inline(StackString::default())
    }

    /// Reserves additional capacity for the `StringWriter`.
    ///
    /// If the writer is currently using an inline `StackString` and the requested capacity exceeds
    /// its maximum inline length, it will switch to a heap-allocated `String`. This method ensures
    /// that subsequent writes do not require reallocations if the specified capacity is available.
    pub fn reserve(&mut self, additional: usize) {
        match self {
            StringWriter::Inline(inline) => {
                let capacity = inline.len() + additional;
                if capacity > String::MAX_INLINE_LEN {
                    // Turn to owned
                    let mut owned = StdString::with_capacity(capacity);
                    owned.push_str(inline);
                    *self = StringWriter::Owned(owned);
                }
            }
            StringWriter::Owned(owned) => {
                owned.reserve(additional);
            }
        }
    }

    /// Appends a single character to the `StringWriter`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::StringWriter;
    ///
    /// let mut writer = StringWriter::default();
    /// writer.push('a');
    /// assert_eq!(writer.as_ref(), "a");
    /// ```
    pub fn push(&mut self, ch: char) {
        if let Self::Inline(inline) = self {
            if inline.try_push_char(ch) {
                return;
            }
            *self = Self::Owned(StdString::from(&*inline));
        }
        if let Self::Owned(owned) = self {
            owned.push(ch);
        } else {
            unreachable!();
        }
    }

    /// Appends a string slice to the `StringWriter`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lstring::StringWriter;
    ///
    /// let mut writer = StringWriter::default();
    /// writer.push_str("abc");
    /// assert_eq!(writer.as_ref(), "abc");
    /// ```
    pub fn push_str(&mut self, s: &str) {
        if let Self::Inline(inline) = self {
            if inline.try_push(s) {
                return;
            }
            *self = Self::Owned(StdString::from(&*inline));
        }
        if let Self::Owned(owned) = self {
            owned.push_str(s);
        } else {
            unreachable!();
        }
    }
}

impl AsRef<str> for StringWriter {
    fn as_ref(&self) -> &str {
        match self {
            Self::Inline(inline) => inline.as_ref(),
            Self::Owned(owned) => owned.as_ref(),
        }
    }
}

impl Default for StringWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: HeapStr> From<StringWriter> for String<B> {
    fn from(writer: StringWriter) -> Self {
        match writer {
            StringWriter::Inline(inline) => inline.into(),
            StringWriter::Owned(owned) => owned.into(),
        }
    }
}

impl fmt::Write for StringWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl Extend<char> for StringWriter {
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        for c in iterator {
            self.push(c);
        }
    }
}

impl<'a> Extend<&'a char> for StringWriter {
    fn extend<I: IntoIterator<Item = &'a char>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<'a> Extend<&'a str> for StringWriter {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(s);
        }
    }
}

impl Extend<Box<str>> for StringWriter {
    fn extend<I: IntoIterator<Item = Box<str>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl Extend<StdString> for StringWriter {
    fn extend<I: IntoIterator<Item = StdString>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<'a> Extend<Cow<'a, str>> for StringWriter {
    fn extend<I: IntoIterator<Item = Cow<'a, str>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<'a, B: HeapStr> Extend<&'a String<B>> for StringWriter {
    fn extend<I: IntoIterator<Item = &'a String<B>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(s);
        }
    }
}

impl<B: HeapStr> Extend<String<B>> for StringWriter {
    fn extend<I: IntoIterator<Item = String<B>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<'a, 's, B: HeapStr> Extend<&'a StringCow<'s, B>> for StringWriter {
    fn extend<I: IntoIterator<Item = &'a StringCow<'s, B>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(s);
        }
    }
}

impl<'s, B: HeapStr> Extend<StringCow<'s, B>> for StringWriter {
    fn extend<I: IntoIterator<Item = StringCow<'s, B>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<'a, 's> Extend<&'a StringRef<'s>> for StringWriter {
    fn extend<I: IntoIterator<Item = &'a StringRef<'s>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(s);
        }
    }
}

impl<'s> Extend<StringRef<'s>> for StringWriter {
    fn extend<I: IntoIterator<Item = StringRef<'s>>>(&mut self, iter: I) {
        for s in iter.into_iter() {
            self.push_str(&s);
        }
    }
}

impl<B: HeapStr> FromIterator<char> for String<B> {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, B: HeapStr> FromIterator<&'a char> for String<B> {
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> Self {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, B: HeapStr> FromIterator<&'a str> for String<B> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<B: HeapStr> FromIterator<Box<str>> for String<B> {
    fn from_iter<I: IntoIterator<Item = Box<str>>>(iter: I) -> String<B> {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, B: HeapStr> FromIterator<Cow<'a, str>> for String<B> {
    fn from_iter<I: IntoIterator<Item = Cow<'a, str>>>(iter: I) -> String<B> {
        let mut iterator = iter.into_iter();

        match iterator.next() {
            None => String::new(),
            Some(cow) => {
                let mut writer = cow.into_owned();
                writer.extend(iterator);
                writer.into()
            }
        }
    }
}

impl<'a, D: HeapStr, B: HeapStr> FromIterator<&'a String<D>> for String<B> {
    fn from_iter<I: IntoIterator<Item = &'a String<D>>>(iter: I) -> String<B> {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<D: HeapStr, B: HeapStr> FromIterator<String<D>> for String<B> {
    fn from_iter<I: IntoIterator<Item = String<D>>>(iter: I) -> String<B> {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, 's, B: HeapStr, D: HeapStr> FromIterator<&'a StringCow<'s, D>> for String<B> {
    fn from_iter<I: IntoIterator<Item = &'a StringCow<'s, D>>>(iter: I) -> String<B> {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'s, B: HeapStr, D: HeapStr> FromIterator<StringCow<'s, D>> for String<B> {
    fn from_iter<I: IntoIterator<Item = StringCow<'s, D>>>(iter: I) -> String<B> {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'a, 's, B: HeapStr> FromIterator<&'a StringRef<'s>> for String<B> {
    fn from_iter<I: IntoIterator<Item = &'a StringRef<'s>>>(iter: I) -> String<B> {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

impl<'s, B: HeapStr> FromIterator<StringRef<'s>> for String<B> {
    fn from_iter<I: IntoIterator<Item = StringRef<'s>>>(iter: I) -> String<B> {
        let mut writer = StringWriter::new();
        writer.extend(iter);
        writer.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringwriter_new() {
        let writer = StringWriter::new();
        assert!(matches!(writer, StringWriter::Inline(_)));
    }

    #[test]
    fn test_stringwriter_push_char_inline() {
        let mut writer = StringWriter::new();
        writer.push('a');
        assert_eq!(writer.as_ref(), "a");
    }

    #[test]
    fn test_stringwriter_push_str_inline() {
        let mut writer = StringWriter::new();
        writer.push_str("abc");
        assert_eq!(writer.as_ref(), "abc");
    }

    #[test]
    fn test_stringwriter_push_char_to_owned() {
        let mut writer = StringWriter::new();
        for _ in 0..=String::MAX_INLINE_LEN {
            writer.push('a');
        }
        assert!(matches!(writer, StringWriter::Owned(_)));
        assert_eq!(writer.as_ref(), "a".repeat(String::MAX_INLINE_LEN + 1));
    }

    #[test]
    fn test_stringwriter_push_str_to_owned() {
        let mut writer = StringWriter::new();
        for _ in 0..=String::MAX_INLINE_LEN / 3 {
            writer.push_str("abc");
        }
        assert!(matches!(writer, StringWriter::Owned(_)));
        assert_eq!(
            writer.as_ref(),
            "abc".repeat(String::MAX_INLINE_LEN / 3 + 1)
        );
    }

    #[test]
    fn test_stringwriter_reserve_inline() {
        let mut writer = StringWriter::new();
        writer.reserve(String::MAX_INLINE_LEN);
        assert!(matches!(writer, StringWriter::Inline(_)));

        let mut writer = StringWriter::new();
        writer.push('a');
        writer.reserve(String::MAX_INLINE_LEN - 1);
        assert!(matches!(writer, StringWriter::Inline(_)));
    }

    #[test]
    fn test_stringwriter_reserve_to_owned() {
        let mut writer = StringWriter::new();
        writer.reserve(String::MAX_INLINE_LEN + 1);
        assert!(matches!(writer, StringWriter::Owned(_)));

        let mut writer = StringWriter::new();
        writer.push('a');
        writer.reserve(String::MAX_INLINE_LEN);
        assert!(matches!(writer, StringWriter::Owned(_)));
    }

    #[test]
    fn test_stringwriter_from_writer_inline() {
        let mut writer = StringWriter::new();
        writer.push_str("abc");
        let kstring: String = writer.into();
        assert_eq!(&kstring, "abc");
    }

    #[test]
    fn test_stringwriter_from_writer_owned() {
        let mut writer = StringWriter::new();
        for _ in 0..=String::MAX_INLINE_LEN / 3 {
            writer.push_str("abc");
        }
        let kstring: String = writer.into();
        assert_eq!(&kstring, &"abc".repeat(String::MAX_INLINE_LEN / 3 + 1));
    }

    #[test]
    fn test_stringwriter_extend_char() {
        let mut writer = StringWriter::new();
        writer.extend("hello".chars());
        assert_eq!(writer.as_ref(), "hello");
    }

    #[test]
    fn test_stringwriter_extend_str() {
        let mut writer = StringWriter::new();
        writer.extend(["he", "ll", "o"]);
        assert_eq!(writer.as_ref(), "hello");
    }
}
