macro_rules! convert_impls {
    ($($b:ident)*) => {
        $(
            static_assertions::const_assert_eq!(core::mem::size_of::<kstring::KStringBase<kstring::backend::$b>>(), core::mem::size_of::<crate::KString<crate::backend::$b>>());

            impl From<kstring::KStringBase<kstring::backend::$b>> for crate::KString<crate::backend::$b> {
                fn from(s: kstring::KStringBase<kstring::backend::$b>) -> Self {
                    unsafe { core::mem::transmute(s) }
                }
            }

            impl From<crate::KString<crate::backend::$b>> for kstring::KStringBase<kstring::backend::$b> {
                fn from(s: crate::KString<kstring::backend::$b>) -> Self {
                    unsafe { core::mem::transmute(s) }
                }
            }

            static_assertions::const_assert_eq!(core::mem::size_of::<kstring::KStringCowBase<'static, kstring::backend::$b>>(), core::mem::size_of::<crate::KStringCow<'static, crate::backend::$b>>());

            impl<'s> From<kstring::KStringCowBase<'s, kstring::backend::$b>> for crate::KStringCow<'s, crate::backend::$b> {
                fn from(s: kstring::KStringCowBase<'s, kstring::backend::$b>) -> Self {
                    unsafe { core::mem::transmute(s) }
                }
            }

            impl<'s> From<crate::KStringCow<'s, crate::backend::$b>> for kstring::KStringCowBase<'s, kstring::backend::$b> {
                fn from(s: crate::KStringCow<'s, kstring::backend::$b>) -> Self {
                    unsafe { core::mem::transmute(s) }
                }
            }
        )*

        static_assertions::const_assert_eq!(core::mem::size_of::<kstring::KStringRef<'static>>(), core::mem::size_of::<crate::KStringRef<'static>>());

        impl<'s> From<kstring::KStringRef<'s>> for crate::KStringRef<'s> {
            fn from(s: kstring::KStringRef<'s>) -> Self {
                unsafe { core::mem::transmute(s) }
            }
        }

        impl<'s> From<crate::KStringRef<'s>> for kstring::KStringRef<'s> {
            fn from(s: crate::KStringRef<'s>) -> Self {
                unsafe { core::mem::transmute(s) }
            }
        }
    };
}

convert_impls!(BoxedStr ArcStr RcStr);
