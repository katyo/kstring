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
    string::FromUtf8Error,
};

use crate::{
    backend::{BoxedStr, DefaultStr, HeapStr},
    stack::StackString,
    KStringCow, KStringRef,
};

/// A UTF-8 encoded, immutable string.
#[derive(Clone)]
#[repr(transparent)]
pub struct KString<B = DefaultStr> {
    inner: KStringInner<B>,
}

impl<B> KString<B> {
    pub const EMPTY: Self = KString::from_static("");

    /// Create a new empty `KString`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::EMPTY
    }

    /// Create a reference to a `'static` data.
    #[inline]
    #[must_use]
    pub const fn from_static(other: &'static str) -> Self {
        Self {
            inner: KStringInner::from_static(other),
        }
    }

    /// Create an inline string, if possible
    #[inline]
    #[must_use]
    pub fn try_inline(other: &str) -> Option<Self> {
        KStringInner::try_inline(other).map(|inner| Self { inner })
    }
}

impl KString {
    /// Maximum length of string which can be created without alloc in heap
    pub const MAX_INLINE_LEN: usize = inner::CAPACITY;
}

impl<B: HeapStr> KString<B> {
    /// Create an owned `KString`.
    #[inline]
    #[must_use]
    pub fn from_boxed(other: BoxedStr) -> Self {
        Self {
            inner: KStringInner::from_boxed(other),
        }
    }

    /// Create an owned `KString`.
    #[inline]
    #[must_use]
    pub fn from_string(other: String) -> Self {
        Self {
            inner: KStringInner::from_string(other),
        }
    }

    /// Create an owned `KString` optimally from a reference.
    #[inline]
    #[must_use]
    pub fn from_ref(other: &str) -> Self {
        Self {
            inner: KStringInner::from_ref(other),
        }
    }

    /// Creates a new `KString` from a UTF-8 byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use kstring2::KString;
    ///
    /// let sparkle_heart = vec![240, 159, 146, 150];
    /// let kstr = <KString>::from_utf8(sparkle_heart).unwrap();
    /// assert_eq!(&kstr, "💖");
    /// ```
    #[inline]
    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error> {
        String::from_utf8(vec).map(Self::from_string)
    }

    /// Creates a new `KString` from a UTF-8 byte slice without checking that the bytes are valid UTF-8.
    ///
    /// # Safety
    ///
    /// The bytes passed in must be valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use kstring2::KString;
    ///
    /// let sparkle_heart = vec![240, 159, 146, 150];
    /// let kstr = unsafe { <KString>::from_utf8_unchecked(sparkle_heart) };
    /// assert_eq!(&kstr, "💖");
    /// ```
    #[cfg(feature = "unsafe")]
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(bytes: Vec<u8>) -> Self {
        Self::from_string(String::from_utf8_unchecked(bytes))
    }

    /// Creates a new `KString` from a UTF-16 byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use kstring2::KString;
    ///
    /// let sparkle_heart = [0xD83D, 0xDC96];
    /// let kstr = <KString>::from_utf16(&sparkle_heart).unwrap();
    /// assert_eq!(&kstr, "💖");
    /// ```
    #[inline]
    pub fn from_utf16(other: &[u16]) -> Result<Self, std::string::FromUtf16Error> {
        String::from_utf16(other).map(Self::from_string)
    }

    /// Get a reference to the `KString`.
    #[inline]
    #[must_use]
    pub fn get_ref(&self) -> KStringRef<'_> {
        self.inner.get_ref()
    }

    /// Extracts a string slice containing the entire `KString`.
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
    pub fn into_cow_str(self) -> Cow<'static, str> {
        self.inner.into_cow_str()
    }
}

impl<B: HeapStr> Deref for KString<B> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> Eq for KString<B> {}

impl<B: HeapStr> PartialEq<KString<B>> for KString<B> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<B: HeapStr> PartialEq<str> for KString<B> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<B: HeapStr> PartialEq<&str> for KString<B> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<B: HeapStr> PartialEq<String> for KString<B> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<B: HeapStr> Ord for KString<B> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<B: HeapStr> PartialOrd for KString<B> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: HeapStr> Hash for KString<B> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<B: HeapStr> fmt::Debug for KString<B> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<B: HeapStr> fmt::Display for KString<B> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<B: HeapStr> AsRef<str> for KString<B> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> AsRef<[u8]> for KString<B> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<B: HeapStr> AsRef<OsStr> for KString<B> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (**self).as_ref()
    }
}

impl<B: HeapStr> AsRef<Path> for KString<B> {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl<B: HeapStr> Borrow<str> for KString<B> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<B: HeapStr> Default for KString<B> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<B: HeapStr> From<StackString<{ KString::MAX_INLINE_LEN }>> for KString<B> {
    fn from(other: StackString<{ KString::MAX_INLINE_LEN }>) -> Self {
        Self {
            inner: KStringInner::from_inline(other),
        }
    }
}

impl<'s, B: HeapStr> From<KStringRef<'s>> for KString<B> {
    #[inline]
    fn from(other: KStringRef<'s>) -> Self {
        other.to_owned()
    }
}

impl<'s, B: HeapStr> From<&'s KStringRef<'s>> for KString<B> {
    #[inline]
    fn from(other: &'s KStringRef<'s>) -> Self {
        other.to_owned()
    }
}

impl<'s, B: HeapStr> From<KStringCow<'s, B>> for KString<B> {
    #[inline]
    fn from(other: KStringCow<'s, B>) -> Self {
        other.into_owned()
    }
}

impl<'s, B: HeapStr> From<&'s KStringCow<'s, B>> for KString<B> {
    #[inline]
    fn from(other: &'s KStringCow<'s, B>) -> Self {
        other.clone().into_owned()
    }
}

impl<B: HeapStr> From<String> for KString<B> {
    #[inline]
    fn from(other: String) -> Self {
        Self::from_string(other)
    }
}

impl<'s, B: HeapStr> From<&'s String> for KString<B> {
    #[inline]
    fn from(other: &'s String) -> Self {
        Self::from_ref(other)
    }
}

impl<B: HeapStr> From<BoxedStr> for KString<B> {
    #[inline]
    fn from(other: BoxedStr) -> Self {
        Self::from_boxed(other)
    }
}

impl<'s, B: HeapStr> From<&'s BoxedStr> for KString<B> {
    #[inline]
    fn from(other: &'s BoxedStr) -> Self {
        Self::from_ref(other)
    }
}

impl<B: HeapStr> From<&str> for KString<B> {
    #[inline]
    fn from(other: &str) -> Self {
        Self::from_ref(other)
    }
}

impl<B: HeapStr> FromStr for KString<B> {
    type Err = Infallible;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_ref(s))
    }
}

#[cfg(feature = "serde")]
impl<B: HeapStr> serde::Serialize for KString<B> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de, B: HeapStr> serde::Deserialize<'de> for KString<B> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(StringVisitor(core::marker::PhantomData))
    }
}

#[cfg(feature = "serde")]
struct StringVisitor<B>(core::marker::PhantomData<B>);

#[cfg(feature = "serde")]
impl<B: HeapStr> serde::de::Visitor<'_> for StringVisitor<B> {
    type Value = KString<B>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::from_ref(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::from_string(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match core::str::from_utf8(v) {
            Ok(s) => Ok(Self::Value::from_ref(s)),
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Bytes(v),
                &self,
            )),
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match String::from_utf8(v) {
            Ok(s) => Ok(Self::Value::from_string(s)),
            Err(e) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Bytes(&e.into_bytes()),
                &self,
            )),
        }
    }
}

#[cfg(feature = "quote")]
impl<B: HeapStr> quote::ToTokens for KString<B> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.as_str().to_tokens(tokens);
    }
}

#[cfg(feature = "diesel")]
#[derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)]
#[diesel(foreign_derive)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[allow(dead_code)]
struct KStringProxy<B>(KString<B>);

#[cfg(feature = "diesel")]
impl<B, ST, DB> diesel::deserialize::FromSql<ST, DB> for KString<B>
where
    B: HeapStr,
    DB: diesel::backend::Backend,
    *const str: diesel::deserialize::FromSql<ST, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let str_ptr = <*const str as diesel::deserialize::FromSql<ST, DB>>::from_sql(bytes)?;
        if !str_ptr.is_null() {
            // SAFETY: We just checked that `str_ptr` is not null, and `from_sql()` should return
            // a valid pointer to an `str`.
            let string = unsafe { &*str_ptr };
            Ok(Self::from_ref(string))
        } else {
            Ok(Default::default())
        }
    }
}

#[cfg(feature = "diesel")]
impl<B, DB> diesel::serialize::ToSql<diesel::sql_types::Text, DB> for KString<B>
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

use inner::KStringInner;

#[cfg(not(feature = "unsafe"))]
mod inner {
    use super::*;
    use core::mem::size_of;

    pub(super) enum KStringInner<B> {
        Singleton(&'static str),
        Inline(StackString<CAPACITY>),
        Owned(B),
    }

    impl<B> KStringInner<B> {
        /// Create a reference to a `'static` data.
        #[inline]
        pub const fn from_static(other: &'static str) -> Self {
            Self::Singleton(other)
        }

        #[inline]
        pub fn try_inline(other: &str) -> Option<Self> {
            StackString::try_new(other).map(Self::from_inline)
        }

        #[inline]
        pub fn from_inline(other: StackString<CAPACITY>) -> Self {
            Self::Inline(other)
        }
    }

    impl<B: HeapStr> KStringInner<B> {
        #[inline]
        pub(super) fn from_boxed(other: BoxedStr) -> Self {
            #[allow(clippy::useless_conversion)]
            Self::Owned(B::from_boxed_str(other))
        }

        #[inline]
        pub(super) fn from_string(other: String) -> Self {
            Self::try_inline(&other).unwrap_or_else(|| Self::from_boxed(other.into_boxed_str()))
        }

        #[inline]
        pub(super) fn from_ref(other: &str) -> Self {
            Self::try_inline(other).unwrap_or_else(|| Self::Owned(B::from_str(other)))
        }

        #[inline]
        pub(super) fn get_ref(&self) -> KStringRef<'_> {
            match self {
                Self::Singleton(s) => KStringRef::from_static(s),
                Self::Inline(s) => KStringRef::from_ref(s.as_str()),
                Self::Owned(s) => KStringRef::from_ref(s.as_str()),
            }
        }

        #[inline]
        pub(super) fn as_str(&self) -> &str {
            match self {
                Self::Singleton(s) => s,
                Self::Inline(s) => s.as_str(),
                Self::Owned(s) => s.as_str(),
            }
        }

        #[inline]
        pub(super) fn into_boxed_str(self) -> BoxedStr {
            match self {
                Self::Singleton(s) => BoxedStr::from(s),
                Self::Inline(s) => BoxedStr::from(s.as_str()),
                Self::Owned(s) => BoxedStr::from(s.as_str()),
            }
        }

        /// Convert to a Cow str
        #[inline]
        pub(super) fn into_cow_str(self) -> Cow<'static, str> {
            match self {
                Self::Singleton(s) => Cow::Borrowed(s),
                Self::Inline(s) => Cow::Owned(s.as_str().into()),
                Self::Owned(s) => Cow::Owned(s.as_str().into()),
            }
        }
    }

    // Explicit to avoid inlining which cuts clone times in half.
    //
    // An automatically derived `clone()` has 10ns overhead while the explicit `Deref`/`as_str` has
    // none of that.  Being explicit and removing the `#[inline]` attribute dropped the overhead to
    // 5ns.
    //
    // My only guess is that the `clone()` calls we delegate to are just that much bigger than
    // `as_str()` that, when combined with a jump table, is blowing the icache, slowing things down.
    impl<B: Clone> Clone for KStringInner<B> {
        fn clone(&self) -> Self {
            match self {
                Self::Singleton(s) => Self::Singleton(s),
                Self::Inline(s) => Self::Inline(*s),
                Self::Owned(s) => Self::Owned(s.clone()),
            }
        }
    }

    #[allow(unused)]
    const LEN_SIZE: usize = size_of::<crate::stack::Len>();

    #[allow(unused)]
    const TAG_SIZE: usize = size_of::<u8>();

    #[allow(unused)]
    const MAX_CAPACITY: usize = size_of::<String>() - TAG_SIZE - LEN_SIZE;

    // Performance seems to slow down when trying to occupy all of the padding left by `String`'s
    // discriminant.  The question is whether faster len=1-16 "allocations" outweighs going to the heap
    // for len=17-22.
    #[allow(unused)]
    const ALIGNED_CAPACITY: usize = size_of::<DefaultStr>() - LEN_SIZE;

    #[cfg(feature = "max_inline")]
    pub(crate) const CAPACITY: usize = MAX_CAPACITY;
    #[cfg(not(feature = "max_inline"))]
    pub(crate) const CAPACITY: usize = ALIGNED_CAPACITY;
}

#[cfg(feature = "unsafe")]
mod inner {
    use super::*;
    use core::mem::{transmute_copy, ManuallyDrop, MaybeUninit};

    #[repr(C)]
    pub(super) union KStringInner<B> {
        tag: TagVariant,
        singleton: SingletonVariant,
        owned: ManuallyDrop<OwnedVariant<B>>,
        inline: InlineVariant,
    }

    impl<B> KStringInner<B> {
        /// Create a reference to a `'static` data.
        #[inline]
        pub const fn from_static(other: &'static str) -> Self {
            Self {
                singleton: SingletonVariant::new(other),
            }
        }

        #[inline]
        pub fn try_inline(other: &str) -> Option<Self> {
            StackString::try_new(other).map(Self::from_inline)
        }

        #[inline]
        pub fn from_inline(inline: StackString<CAPACITY>) -> Self {
            Self {
                inline: InlineVariant::new(inline),
            }
        }

        #[inline]
        const fn tag(&self) -> Tag {
            unsafe {
                // SAFETY: `tag` is in the same spot in each variant
                self.tag.tag
            }
        }
    }

    impl<B: HeapStr> KStringInner<B> {
        #[inline]
        pub(super) fn from_boxed(other: BoxedStr) -> Self {
            #[allow(clippy::useless_conversion)]
            let payload = B::from_boxed_str(other);
            Self {
                owned: ManuallyDrop::new(OwnedVariant::new(payload)),
            }
        }

        #[inline]
        pub(super) fn from_string(other: String) -> Self {
            KStringInner::try_inline(&other)
                .unwrap_or_else(|| Self::from_boxed(other.into_boxed_str()))
        }

        #[inline]
        pub(super) fn from_ref(other: &str) -> Self {
            KStringInner::try_inline(other).unwrap_or_else(|| {
                #[allow(clippy::useless_conversion)]
                let payload = B::from_str(other);
                Self {
                    owned: ManuallyDrop::new(OwnedVariant::new(payload)),
                }
            })
        }

        #[inline]
        pub(super) fn get_ref(&self) -> KStringRef<'_> {
            let tag = self.tag();
            unsafe {
                // SAFETY: `tag` ensures access to correct variant
                if tag.is_singleton() {
                    KStringRef::from_static(self.singleton.payload)
                } else if tag.is_owned() {
                    KStringRef::from_ref(self.owned.payload.as_str())
                } else {
                    debug_assert!(tag.is_inline());
                    KStringRef::from_ref(self.inline.payload.as_str())
                }
            }
        }

        #[inline]
        pub(super) fn as_str(&self) -> &str {
            let tag = self.tag();
            unsafe {
                // SAFETY: `tag` ensures access to correct variant
                if tag.is_singleton() {
                    self.singleton.payload
                } else if tag.is_owned() {
                    self.owned.payload.as_str()
                } else {
                    debug_assert!(tag.is_inline());
                    self.inline.payload.as_str()
                }
            }
        }

        #[inline]
        pub(super) fn into_boxed_str(self) -> BoxedStr {
            let tag = self.tag();
            unsafe {
                // SAFETY: `tag` ensures access to correct variant
                if tag.is_singleton() {
                    BoxedStr::from(self.singleton.payload)
                } else if tag.is_owned() {
                    BoxedStr::from(self.owned.payload.as_str())
                } else {
                    debug_assert!(tag.is_inline());
                    BoxedStr::from(self.inline.payload.as_ref())
                }
            }
        }

        /// Convert to a Cow str
        #[inline]
        pub(super) fn into_cow_str(self) -> Cow<'static, str> {
            let tag = self.tag();
            unsafe {
                // SAFETY: `tag` ensures access to correct variant
                if tag.is_singleton() {
                    Cow::Borrowed(self.singleton.payload)
                } else if tag.is_owned() {
                    Cow::Owned(self.owned.payload.as_str().into())
                } else {
                    debug_assert!(tag.is_inline());
                    Cow::Owned(self.inline.payload.as_str().into())
                }
            }
        }
    }

    // Explicit to avoid inlining which cuts clone times in half.
    //
    // An automatically derived `clone()` has 10ns overhead while the explicit `Deref`/`as_str` has
    // none of that.  Being explicit and removing the `#[inline]` attribute dropped the overhead to
    // 5ns.
    //
    // My only guess is that the `clone()` calls we delegate to are just that much bigger than
    // `as_str()` that, when combined with a jump table, is blowing the icache, slowing things down.
    impl<B: Clone> Clone for KStringInner<B> {
        fn clone(&self) -> Self {
            let tag = self.tag();
            if tag.is_owned() {
                unsafe {
                    // SAFETY: `tag` ensures access to correct variant
                    Self {
                        owned: ManuallyDrop::new(OwnedVariant::new(self.owned.payload.clone())),
                    }
                }
            } else {
                unsafe {
                    // SAFETY: `tag` ensures access to correct variant
                    // SAFETY: non-owned types are copyable
                    transmute_copy(self)
                }
            }
        }
    }

    impl<B> Drop for KStringInner<B> {
        fn drop(&mut self) {
            let tag = self.tag();
            if tag.is_owned() {
                unsafe {
                    // SAFETY: `tag` ensures we are using the right variant
                    ManuallyDrop::drop(&mut self.owned)
                }
            }
        }
    }

    #[allow(unused)]
    const LEN_SIZE: usize = size_of::<crate::stack::Len>();

    #[allow(unused)]
    const TAG_SIZE: usize = size_of::<Tag>();

    #[allow(unused)]
    const PAYLOAD_SIZE: usize = size_of::<DefaultStr>();
    type Payload = Padding<PAYLOAD_SIZE>;

    #[allow(unused)]
    const TARGET_SIZE: usize = size_of::<Target>();
    type Target = String;

    #[allow(unused)]
    const MAX_CAPACITY: usize = TARGET_SIZE - LEN_SIZE - TAG_SIZE;

    // Performance seems to slow down when trying to occupy all of the padding left by `String`'s
    // discriminant.  The question is whether faster len=1-16 "allocations" outweighs going to the heap
    // for len=17-22.
    #[allow(unused)]
    const ALIGNED_CAPACITY: usize = PAYLOAD_SIZE - LEN_SIZE;

    #[cfg(feature = "max_inline")]
    pub(crate) const CAPACITY: usize = MAX_CAPACITY;
    #[cfg(not(feature = "max_inline"))]
    pub(crate) const CAPACITY: usize = ALIGNED_CAPACITY;

    const PAYLOAD_PAD_SIZE: usize = TARGET_SIZE - PAYLOAD_SIZE - TAG_SIZE;
    const INLINE_PAD_SIZE: usize = TARGET_SIZE - CAPACITY - LEN_SIZE - TAG_SIZE;

    #[derive(Copy, Clone)]
    #[repr(C)]
    struct TagVariant {
        payload: Payload,
        pad: Padding<PAYLOAD_PAD_SIZE>,
        tag: Tag,
    }
    static_assertions::assert_eq_size!(Target, TagVariant);

    #[derive(Copy, Clone)]
    #[repr(C)]
    struct SingletonVariant {
        payload: &'static str,
        pad: Padding<PAYLOAD_PAD_SIZE>,
        tag: Tag,
    }
    static_assertions::assert_eq_size!(Payload, &'static str);
    static_assertions::assert_eq_size!(Target, SingletonVariant);

    impl SingletonVariant {
        #[inline]
        const fn new(payload: &'static str) -> Self {
            Self {
                payload,
                pad: Padding::new(),
                tag: Tag::SINGLETON,
            }
        }
    }

    impl std::fmt::Debug for SingletonVariant {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.payload.fmt(f)
        }
    }

    #[derive(Clone)]
    #[repr(C)]
    struct OwnedVariant<B> {
        payload: B,
        pad: Padding<PAYLOAD_PAD_SIZE>,
        tag: Tag,
    }
    static_assertions::assert_eq_size!(Payload, DefaultStr);
    static_assertions::assert_eq_size!(Target, OwnedVariant<DefaultStr>);

    impl<B> OwnedVariant<B> {
        #[inline]
        const fn new(payload: B) -> Self {
            Self {
                payload,
                pad: Padding::new(),
                tag: Tag::OWNED,
            }
        }
    }

    impl<B: HeapStr> std::fmt::Debug for OwnedVariant<B> {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.payload.fmt(f)
        }
    }

    #[derive(Copy, Clone)]
    #[repr(C)]
    struct InlineVariant {
        payload: StackString<CAPACITY>,
        pad: Padding<INLINE_PAD_SIZE>,
        tag: Tag,
    }
    static_assertions::assert_eq_size!(Target, InlineVariant);

    impl InlineVariant {
        #[inline]
        const fn new(payload: StackString<CAPACITY>) -> Self {
            Self {
                payload,
                pad: Padding::new(),
                tag: Tag::INLINE,
            }
        }
    }

    impl std::fmt::Debug for InlineVariant {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.payload.fmt(f)
        }
    }

    /// Represents the internal tag used to distinguish between different variants of `KStringInner`.
    ///
    /// The `Tag` type is crucial for determining whether a `KStringInner` instance holds a singleton,
    /// owned, or inline string. This helps in safely accessing the correct variant without undefined behavior.
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    struct Tag(u8);

    impl Tag {
        /// Represents a `KStringInner` variant that holds a reference to `'static` string data.
        const SINGLETON: Tag = Tag(0);
        /// Represents a `KStringInner` variant that owns the string data on the heap.
        const OWNED: Tag = Tag(u8::MAX);
        /// Represents a `KStringInner` variant that holds the string data inline within the struct.
        const INLINE: Tag = Tag(1);

        #[inline]
        const fn is_singleton(self) -> bool {
            self.0 == Self::SINGLETON.0
        }

        #[inline]
        const fn is_owned(self) -> bool {
            self.0 == Self::OWNED.0
        }

        #[inline]
        const fn is_inline(self) -> bool {
            !self.is_singleton() && !self.is_owned()
        }
    }

    #[derive(Copy, Clone)]
    #[repr(transparent)]
    struct Padding<const L: usize>([MaybeUninit<u8>; L]);

    impl<const L: usize> Padding<L> {
        const fn new() -> Self {
            let padding = unsafe {
                // SAFETY: Padding, never actually used
                MaybeUninit::uninit().assume_init()
            };
            Self(padding)
        }
    }

    impl<const L: usize> Default for Padding<L> {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn test_size() {
        println!("KString: {}", size_of::<KString>());
    }
}
