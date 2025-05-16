//! Key String: Optimized for map keys.
//!
//! # Examples
//!
//! String creation
//! ```rust
//! # use kstring2::{KString, KStringCow};
//! // Explicit
//! let literal = <KString>::from_static("literal");
//! // Implicit
//! let literal = <KString>::from("literal");
//!
//! // Explicit
//! let inline = <KString>::try_inline("stack").unwrap();
//! let inline = <KString>::from_ref("stack");
//!
//! let formatted: KStringCow = format!("Hello {} and {}", literal, inline).into();
//! ```
//!
//! # Background
//!
//! Considerations:
//! - Large maps
//! - Most keys live and drop without being used in any other way
//! - Most keys are relatively small (single to double digit bytes)
//! - Keys are immutable
//! - Allow zero-cost abstractions between structs and maps (e.g. no allocating
//!   when dealing with struct field names)
//!
//! Ramifications:
//! - Inline small strings rather than going to the heap.
//! - Preserve `&'static str` across strings ([`KString`]),
//!   references ([`KStringRef`]), and lifetime abstractions ([`KStringCow`]) to avoid
//!   allocating for struct field names.
//! - Use `Box<str>` rather than `String` to use less memory.
//!
//! Significant changes:
//! - Because `From<&'static str>` is unsound it changed to `From<&str>`. To instantiate //! from static str use `KString::from_static` instead of `From::from`.
//! - Added default generic to types `KStringBase` and `KStringCowBase` which renamed to `KString` and `KStringCow`. Corresponding type aliases is removed. To instantiate types with default backend wrap it with angle brackets (`KString::from_ref("abc")` => `<KString>::from_ref("abc")`).
//!
//! # Feature Flags
//!
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(not(feature = "std"))]
compile_error!("`std` feature is required; reserved for future `no_std` support");

#[cfg(feature = "kstring")]
mod interop;
mod stack;
mod string;
mod string_cow;
mod string_ref;

pub mod backend;

pub use stack::StackString;
pub use string::*;
pub use string_cow::*;
pub use string_ref::*;

#[cfg(test)]
mod test {
    #[test]
    fn test_size() {
        println!("String: {}", std::mem::size_of::<String>());
        println!(
            "Box<str>: {}",
            std::mem::size_of::<crate::backend::DefaultStr>()
        );
        println!(
            "Box<Box<str>>: {}",
            std::mem::size_of::<Box<crate::backend::DefaultStr>>()
        );
        println!("str: {}", std::mem::size_of::<&'static str>());
        println!(
            "Cow: {}",
            std::mem::size_of::<std::borrow::Cow<'static, str>>()
        );
    }
}
