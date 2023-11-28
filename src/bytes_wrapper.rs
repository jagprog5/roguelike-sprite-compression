use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::cmp::{PartialEq, PartialOrd, Ord, Ordering};
use std::borrow::{Borrow, BorrowMut};

// BytesWrapper implements traits required for num_traits::ops::bytes::NumBytes

pub struct BytesWrapper<'a> {
    pub bytes: &'a [u8],
}

impl<'a> Debug for BytesWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.bytes).finish()
    }
}

impl<'a> AsRef<[u8]> for BytesWrapper<'a> {
    fn as_ref(&self) -> &[u8] {
        self.bytes
    }
}

impl<'a> AsMut<[u8]> for BytesWrapper<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        panic!("https://github.com/rust-num/num-traits/pull/301");
    }
}

impl<'a> PartialEq for BytesWrapper<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<'a> Eq for BytesWrapper<'a> {}

impl<'a> PartialOrd for BytesWrapper<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl<'a> Ord for BytesWrapper<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<'a> Hash for BytesWrapper<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<'a> Borrow<[u8]> for BytesWrapper<'a> {
    fn borrow(&self) -> &[u8] {
        self.bytes
    }
}

impl<'a> BorrowMut<[u8]> for BytesWrapper<'a> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        panic!("https://github.com/rust-num/num-traits/pull/301");
    }
}
