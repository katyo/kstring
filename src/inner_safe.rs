use crate::{
    HeapStr, StackString, StdString, StringRef,
    backend::{BoxedStr, DefaultStr},
};
use core::mem::size_of;
use std::borrow::Cow;

pub(super) enum StringInner<B> {
    Singleton(&'static str),
    Inline(StackString<CAPACITY>),
    Owned(B),
}

impl<B> StringInner<B> {
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

impl<B: HeapStr> StringInner<B> {
    #[inline]
    pub(super) fn from_boxed(other: BoxedStr) -> Self {
        #[allow(clippy::useless_conversion)]
        Self::Owned(B::from_boxed_str(other))
    }

    #[inline]
    pub(super) fn from_string(other: StdString) -> Self {
        Self::try_inline(&other).unwrap_or_else(|| Self::from_boxed(other.into_boxed_str()))
    }

    #[inline]
    pub(super) fn from_ref(other: &str) -> Self {
        Self::try_inline(other).unwrap_or_else(|| Self::Owned(B::from_str(other)))
    }

    #[inline]
    pub(super) fn get_ref(&self) -> StringRef<'_> {
        match self {
            Self::Singleton(s) => StringRef::from_static(s),
            Self::Inline(s) => StringRef::from_ref(s.as_str()),
            Self::Owned(s) => StringRef::from_ref(s.as_str()),
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
    pub(super) fn get_mut_str(&mut self) -> Option<&mut str> {
        match self {
            Self::Singleton(_) => None,
            Self::Inline(s) => Some(s.as_mut_str()),
            Self::Owned(s) => s.get_mut_str(),
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
impl<B: Clone> Clone for StringInner<B> {
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
