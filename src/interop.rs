macro_rules! convert_impls {
    ($($b:ident)*) => {
        $(
            static_assertions::const_assert_eq!(core::mem::size_of::<kstring::KStringBase<kstring::backend::$b>>(), core::mem::size_of::<crate::String<crate::backend::$b>>());

            impl From<kstring::KStringBase<kstring::backend::$b>> for crate::String<crate::backend::$b> {
                fn from(s: kstring::KStringBase<kstring::backend::$b>) -> Self {
                    unsafe { core::mem::transmute(s) }
                }
            }

            impl From<crate::String<crate::backend::$b>> for kstring::KStringBase<kstring::backend::$b> {
                fn from(s: crate::String<kstring::backend::$b>) -> Self {
                    unsafe { core::mem::transmute(s) }
                }
            }

            static_assertions::const_assert_eq!(core::mem::size_of::<kstring::KStringCowBase<'static, kstring::backend::$b>>(), core::mem::size_of::<crate::StringCow<'static, crate::backend::$b>>());

            impl<'s> From<kstring::KStringCowBase<'s, kstring::backend::$b>> for crate::StringCow<'s, crate::backend::$b> {
                fn from(s: kstring::KStringCowBase<'s, kstring::backend::$b>) -> Self {
                    unsafe { core::mem::transmute(s) }
                }
            }

            impl<'s> From<crate::StringCow<'s, crate::backend::$b>> for kstring::KStringCowBase<'s, kstring::backend::$b> {
                fn from(s: crate::StringCow<'s, kstring::backend::$b>) -> Self {
                    unsafe { core::mem::transmute(s) }
                }
            }
        )*

        static_assertions::const_assert_eq!(core::mem::size_of::<kstring::KStringRef<'static>>(), core::mem::size_of::<crate::StringRef<'static>>());

        impl<'s> From<kstring::KStringRef<'s>> for crate::StringRef<'s> {
            fn from(s: kstring::KStringRef<'s>) -> Self {
                unsafe { core::mem::transmute(s) }
            }
        }

        impl<'s> From<crate::StringRef<'s>> for kstring::KStringRef<'s> {
            fn from(s: crate::StringRef<'s>) -> Self {
                unsafe { core::mem::transmute(s) }
            }
        }
    };
}

convert_impls!(BoxedStr ArcStr RcStr);
