use crate::{
    HeapStr, StackString, StdString, StringRef,
    backend::{BoxedStr, DefaultStr},
};
use core::mem::{ManuallyDrop, MaybeUninit, transmute_copy};
use std::borrow::Cow;

#[repr(C)]
pub(super) union StringInner<B> {
    tag: TagVariant,
    singleton: SingletonVariant,
    owned: ManuallyDrop<OwnedVariant<B>>,
    inline: InlineVariant,
}

impl<B> StringInner<B> {
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

impl<B: HeapStr> StringInner<B> {
    #[inline]
    pub(super) fn from_boxed(other: BoxedStr) -> Self {
        #[allow(clippy::useless_conversion)]
        let payload = B::from_boxed_str(other);
        Self {
            owned: ManuallyDrop::new(OwnedVariant::new(payload)),
        }
    }

    #[inline]
    pub(super) fn from_string(other: StdString) -> Self {
        StringInner::try_inline(&other).unwrap_or_else(|| Self::from_boxed(other.into_boxed_str()))
    }

    #[inline]
    pub(super) fn from_ref(other: &str) -> Self {
        StringInner::try_inline(other).unwrap_or_else(|| {
            #[allow(clippy::useless_conversion)]
            let payload = B::from_str(other);
            Self {
                owned: ManuallyDrop::new(OwnedVariant::new(payload)),
            }
        })
    }

    #[inline]
    pub(super) fn get_ref(&self) -> StringRef<'_> {
        let tag = self.tag();
        unsafe {
            // SAFETY: `tag` ensures access to correct variant
            if tag.is_singleton() {
                StringRef::from_static(self.singleton.payload)
            } else if tag.is_owned() {
                StringRef::from_ref(self.owned.payload.as_str())
            } else {
                debug_assert!(tag.is_inline());
                StringRef::from_ref(self.inline.payload.as_str())
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
    pub(super) fn get_mut_str(&mut self) -> Option<&mut str> {
        let tag = self.tag();
        unsafe {
            // SAFETY: `tag` ensures access to correct variant
            if tag.is_singleton() {
                None
            } else if tag.is_owned() {
                self.owned.payload.get_mut_str()
            } else {
                debug_assert!(tag.is_inline());
                Some(self.inline.payload.as_mut_str())
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
impl<B: Clone> Clone for StringInner<B> {
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

impl<B> Drop for StringInner<B> {
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
type Target = StdString;

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

/// Represents the internal tag used to distinguish between different variants of `StringInner`.
///
/// The `Tag` type is crucial for determining whether a `StringInner` instance holds a singleton,
/// owned, or inline string. This helps in safely accessing the correct variant without undefined behavior.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
struct Tag(u8);

impl Tag {
    /// Represents a `StringInner` variant that holds a reference to `'static` string data.
    const SINGLETON: Tag = Tag(0);
    /// Represents a `StringInner` variant that owns the string data on the heap.
    const OWNED: Tag = Tag(u8::MAX);
    /// Represents a `StringInner` variant that holds the string data inline within the struct.
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
