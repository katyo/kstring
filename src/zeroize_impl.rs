use super::{HeapStr, String};
use zeroize::{TryZeroize, Zeroize};

impl<B: HeapStr> TryZeroize for String<B> {
    fn try_zeroize(&mut self) -> bool {
        if let Some(s) = self.get_mut_str() {
            s.zeroize();
            true
        } else {
            false
        }
    }
}
