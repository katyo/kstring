use super::{HeapStr, SecureString, String, inner::StringInner};
use zeroize::{TryZeroize, Zeroize};

impl<B: HeapStr> TryZeroize for StringInner<B> {
    fn try_zeroize(&mut self) -> bool {
        if let Some(s) = self.get_mut_str() {
            s.zeroize();
            true
        } else {
            false
        }
    }
}

impl<B: HeapStr> TryZeroize for String<B> {
    fn try_zeroize(&mut self) -> bool {
        self.inner.try_zeroize()
    }
}

impl<B: HeapStr> TryZeroize for SecureString<B> {
    fn try_zeroize(&mut self) -> bool {
        self.inner.try_zeroize()
    }
}
