// Derived from `pinocchio-token-interface` – commit 75116550519a9ee3fdfa6c819aca91e383fffa39, Apache-2.0.
// Modifications by DASMAC, 2025. See: https://github.com/solana-program/token

use crate::error::DropsetError;

/// Marker trait for a zero-copy view of bytes as `&Self` via an unchecked cast, aka a transmute.
///
/// # Safety
/// **Implementor guarantees:**
/// - Use a stable layout (`#[repr(C)]` or `#[repr(transparent)]`) and ensure any
/// - `LEN` bytes form a valid `Self`. Prefer `[u8; N]` and/or transparent byte wrappers.
/// - `size_of::<Self> == LEN`
/// - `align_of::<Self> == 1`
///
/// **Caller guarantees:**
/// - The bytes represent a valid `Self`.
pub unsafe trait Transmutable: Sized {
    /// The cumulative size in bytes of all fields in the struct.
    const LEN: usize;

    /// Returns a reference to a `T: Transmutable` from the given bytes after checking the byte length.
    ///
    /// # Safety
    /// - Caller must guarantee `bytes` is a valid representation of `T`.
    #[inline(always)]
    unsafe fn load(bytes: &[u8]) -> Result<&Self, DropsetError> {
        if bytes.len() != Self::LEN {
            return Err(DropsetError::InsufficientByteLength);
        }
        Ok(&*(bytes.as_ptr() as *const Self))
    }

    /// Returns a reference to a `T: Transmutable` from the given bytes.
    ///
    /// # Safety
    /// - Caller must guarantee `bytes` is a valid representation of `T`.
    /// - Caller must guarantee `bytes.len()` is equal to `T::LEN`.
    #[inline(always)]
    unsafe fn load_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Self)
    }

    /// Returns a mutable reference to a `T: Transmutable` from the given bytes after checking the byte
    /// length.
    ///
    /// # Safety
    /// - Caller must guarantee `bytes` is a valid representation of `T`.
    #[inline(always)]
    unsafe fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, DropsetError> {
        if bytes.len() != Self::LEN {
            return Err(DropsetError::InsufficientByteLength);
        }
        Ok(&mut *(bytes.as_ptr() as *mut Self))
    }

    /// Returns a mutable reference to a `T: Transmutable` from the given bytes.
    ///
    /// # Safety
    /// - Caller must guarantee `bytes` is a valid representation of `T`.
    /// - Caller must guarantee `bytes.len()` is equal to `T::LEN`.
    #[inline(always)]
    unsafe fn load_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_ptr() as *mut Self)
    }
}
