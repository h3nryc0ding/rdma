/// A RDMA device guid
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C, align(8))]
pub struct Guid([u8; 8]);

impl Guid {
    /// Constructs a Guid from network bytes.
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// Returns the bytes of GUID in network byte order.
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }
}
