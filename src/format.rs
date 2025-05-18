use crate::{backend::HeapStr, KString, StackString};
use core::fmt;

/// A macro for formatting arguments into a `KString`.
///
/// This macro is similar to Rust's standard `format!` macro but outputs a `KString`
/// instead of a `String`. It uses the `fmt::Write` trait internally to append formatted
/// text efficiently.
///
/// # Examples
///
/// ```
/// # use kstring2::{kformat, KString};
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
/// use kstring2::{KString, KStringWriter};
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
        Self::Inline(StackString::default())
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
        if let Self::Inline(inline) = self {
            if inline.try_push(s) {
                return Ok(());
            }
            *self = Self::Owned(String::from(&*inline));
        }
        if let Self::Owned(owned) = self {
            owned.push_str(s);
            Ok(())
        } else {
            unreachable!();
        }
    }
}
