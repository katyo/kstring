use super::{HeapStr, String, StringCow, StringRef};
use proc_macro2::TokenStream;
use quote::ToTokens;

impl<B: HeapStr> ToTokens for String<B> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_str().to_tokens(tokens);
    }
}

impl<B: HeapStr> ToTokens for StringCow<'_, B> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_str().to_tokens(tokens);
    }
}

impl ToTokens for StringRef<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_str().to_tokens(tokens);
    }
}
