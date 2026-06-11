extern crate alloc;
use {
    crate::{AsciiStr, CaseSensitive, Comparator, FromAsciiError},
    alloc::{
        borrow::{Borrow, ToOwned},
        collections::TryReserveError,
        string::{String, ToString as _},
        vec::Vec,
    },
    core::{
        cmp::{Eq, Ordering, PartialEq},
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
        marker::PhantomData,
        ops::Deref,
    },
};

/// ASCII string type with pluggable case sensitivity.
///
/// Comparisons — including comparisons with [`str`] and [`String`] — follow the comparator
/// `C`. For example, an `AsciiString<CaseInsensitive>` holding `"hello"` compares equal to
/// both `"hello"` and `"HELLO"`; this is *not* literal string equality. Use
/// [`as_str`][Self::as_str] when exact equality is required.
pub struct AsciiString<C = CaseSensitive>(pub(crate) PhantomData<C>, pub(crate) Vec<u8>);

impl<C> AsciiString<C> {
    /// Creates a new empty `AsciiString`.
    #[inline(always)]
    pub fn new() -> Self {
        Self(PhantomData, Vec::new())
    }

    /// Creates a new empty `AsciiString` with at least the specified capacity.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(PhantomData, Vec::with_capacity(capacity))
    }

    /// Converts a vector of bytes to an `AsciiString`, validating that the bytes are valid ASCII.
    pub fn from_ascii(bytes: Vec<u8>) -> Result<Self, FromAsciiError> {
        if bytes.is_ascii() {
            Ok(Self(PhantomData, bytes))
        } else {
            Err(FromAsciiError {
                bytes,
            })
        }
    }

    /// Returns this `AsciiString` as a byte slice.
    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8] {
        self.1.as_slice()
    }

    /// Returns this `AsciiString` as a string slice.
    #[inline(always)]
    pub const fn as_str(&self) -> &str {
        // SAFETY: We guarantee that the bytes are valid ASCII, which is a subset of UTF-8, so this
        // conversion is always valid.
        unsafe { core::str::from_utf8_unchecked(self.1.as_slice()) }
    }

    /// Returns this `AsciiString` as an [`AsciiStr`] slice.
    #[inline(always)]
    pub const fn as_ascii_str(&self) -> &AsciiStr<C> {
        // SAFETY: We guarantee that the bytes are valid ASCII.
        unsafe { AsciiStr::from_ascii_unchecked(self.1.as_slice()) }
    }

    /// Returns this `AsciiString`'s capacity in bytes.
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.1.capacity()
    }

    /// Truncates this `AsciiString`, removing all contents.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.1.clear();
    }

    /// Inserts a character into this `AsciiString` at byte position `idx`.
    ///
    /// If the character is not ASCII, a `FromAsciiError` is returned.
    ///
    /// # Panics
    /// Panics if `idx` is larger than the `AsciiString`'s length.
    pub fn insert(&mut self, idx: usize, ch: char) -> Result<(), FromAsciiError> {
        if !ch.is_ascii() {
            return Err(FromAsciiError {
                bytes: ch.to_string().into_bytes(),
            });
        }

        let ch = ch as u8;
        self.1.insert(idx, ch);
        Ok(())
    }

    /// Inserts an ASCII string slice into this `AsciiString` at byte position `idx`.
    ///
    /// # Panics
    /// Panics if `idx` is larger than the `AsciiString`'s length.
    #[inline(always)]
    pub fn insert_ascii(&mut self, idx: usize, ascii: &AsciiString<C>) {
        self.1.splice(idx..idx, ascii.as_bytes().iter().cloned());
    }

    /// Inserts a string slice into this `AsciiString` at byte position `idx`.
    ///
    /// If the string slice contains any non-ASCII characters, a `FromAsciiError` is returned.
    ///
    /// # Panics
    /// Panics if `idx` is larger than the `AsciiString`'s length.
    pub fn insert_str(&mut self, idx: usize, s: &str) -> Result<(), FromAsciiError> {
        if !s.is_ascii() {
            return Err(FromAsciiError {
                bytes: s.as_bytes().to_vec(),
            });
        }

        self.1.splice(idx..idx, s.as_bytes().iter().cloned());
        Ok(())
    }

    /// Converts an `AsciiString` into a byte vector.
    #[inline(always)]
    pub fn into_bytes(self) -> Vec<u8> {
        self.1
    }

    /// Returns `true` if this `AsciiString` has a length of zero, and `false` otherwise.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.1.is_empty()
    }

    /// Returns the length of this `AsciiString` in bytes.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.1.len()
    }

    /// Removes the last ASCII character from this `AsciiString` and returns it.
    ///
    /// Returns `None` if this `AsciiString` is empty.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<u8> {
        self.1.pop()
    }

    /// Appends the given ASCII character to the end of this `AsciiString`.
    ///
    /// If the character is not ASCII, a `FromAsciiError` is returned.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds [`isize::MAX`] bytes.
    #[inline(always)]
    pub fn push(&mut self, c: u8) -> Result<(), FromAsciiError> {
        if !c.is_ascii() {
            return Err(FromAsciiError {
                bytes: alloc::vec![c],
            });
        }

        self.1.push(c);
        Ok(())
    }

    /// Appends a given `AsciiStr` onto the end of this `AsciiString`.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds [`isize::MAX`] bytes.
    #[inline(always)]
    pub fn push_ascii(&mut self, other: &AsciiStr<C>) {
        self.1.extend_from_slice(other.as_bytes());
    }

    /// Removes an ASCII character from this `AsciiString` at byte position `idx` and
    /// returns it.
    ///
    /// Copies all bytes after the removed character to new positions.
    ///
    /// Note that calling this in a loop can result in quadratic behavior.
    ///
    /// # Panics
    /// Panics if `idx` is larger than or equal to the `AsciiString`'s length.
    #[inline(always)]
    pub fn remove(&mut self, idx: usize) -> u8 {
        self.1.remove(idx)
    }

    /// Reserves capacity for at least `additional` bytes more than the current length. The
    /// allocator may reserve more space to speculatively avoid frequent allocations. After calling
    /// `reserve`, capacity will be greater than or equal to `self.len() + additional`. Does nothing
    /// if capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        self.1.reserve(additional);
    }

    /// Reserves the minimum capacity for at least `additional` bytes more than the current length.
    /// Unlike [`reserve`][Self::reserve], this will not deliberately over-allocate to speculatively
    /// avoid frequent allocations. After calling `reserve_exact`, capacity will be greater than or
    /// equal to `self.len() + additional`. Does nothing if the capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    #[inline(always)]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.1.reserve_exact(additional);
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// In other words, remove all characters `c` such that `f(c)` returns `false`. This method
    /// operates in place, visiting each character exactly once in the original order, and
    /// preserves the order of the retained characters.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(u8) -> bool,
    {
        self.1.retain(|&c| f(c));
    }

    /// Shrinks the capacity of this `AsciiString` with a lower bound.
    ///
    /// The capacity will remain at least as large as both the length and the supplied value.
    ///
    /// If the current capacity is less than the lower limit, this is a no-op.
    #[inline(always)]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.1.shrink_to(min_capacity);
    }

    /// Shrinks the capacity of this `AsciiString` to match its length.
    #[inline(always)]
    pub fn shrink_to_fit(&mut self) {
        self.1.shrink_to_fit();
    }

    /// Splits the string into two at the given byte index.
    ///
    /// Returns a newly allocated `AsciiString`. `self` contains bytes `[0, at)`, and
    /// the returned `AsciiString` contains bytes `[at, len)`.
    ///
    /// Note that the capacity of `self` does not change.
    ///
    /// # Panics
    ///
    /// Panics if `at` is beyond the last byte of the string.
    #[inline(always)]
    pub fn split_off(&mut self, at: usize) -> Self {
        Self(PhantomData, self.1.split_off(at))
    }

    /// Shortens this `AsciiString` to the specified length.
    ///
    /// If `new_len` is greater than or equal to the string's current length, this has no effect.
    ///
    /// Note that this method has no effect on the allocated capacity of the string.
    #[inline(always)]
    pub fn truncate(&mut self, new_len: usize) {
        self.1.truncate(new_len);
    }

    /// Tries to reserve capacity for at least `additional` bytes more than the current length. The
    /// allocator may reserve more space to speculatively avoid frequent allocations. After calling
    /// `try_reserve`, capacity will be greater than or equal to `self.len() + additional` if it
    /// returns `Ok(())`. Does nothing if capacity is already sufficient. This method preserves the
    /// contents even if an error occurs.
    ///
    /// # Errors
    ///
    /// If the capacity overflows, or the allocator reports a failure, then an error is returned.
    #[inline(always)]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.1.try_reserve(additional)
    }

    /// Tries to reserve the minimum capacity for at least `additional` bytes more than the current
    /// length. Unlike [`try_reserve`][Self::try_reserve], this will not deliberately over-allocate
    /// to speculatively avoid frequent allocations. After calling `try_reserve_exact`, capacity
    /// will be greater than or equal to `self.len() + additional` if it returns `Ok(())`. Does
    /// nothing if the capacity is already sufficient.
    #[inline(always)]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.1.try_reserve_exact(additional)
    }
}

impl<C> AsRef<AsciiStr<C>> for AsciiString<C> {
    #[inline(always)]
    fn as_ref(&self) -> &AsciiStr<C> {
        self.as_ascii_str()
    }
}

impl<C> Borrow<AsciiStr<C>> for AsciiString<C> {
    #[inline(always)]
    fn borrow(&self) -> &AsciiStr<C> {
        self.as_ascii_str()
    }
}

impl<C> Default for AsciiString<C> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Debug for AsciiString<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self.as_str(), f)
    }
}

impl<C> Deref for AsciiString<C> {
    type Target = AsciiStr<C>;

    #[inline(always)]
    fn deref(&self) -> &AsciiStr<C> {
        self.as_ascii_str()
    }
}

#[cfg(feature = "serde")]
impl<'de, C> serde::Deserialize<'de> for AsciiString<C>
where
    C: Comparator,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_ascii(s.into_bytes()).map_err(serde::de::Error::custom)
    }
}

impl<C> Display for AsciiString<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(self.as_str(), f)
    }
}

impl<C> From<&AsciiStr<C>> for AsciiString<C> {
    fn from(value: &AsciiStr<C>) -> Self {
        value.to_owned()
    }
}

#[cfg(feature = "serde")]
impl<C> serde::Serialize for AsciiString<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<C: Comparator> Eq for AsciiString<C> {}

impl<C: Comparator> Hash for AsciiString<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Delegate to AsciiStr so the Borrow<AsciiStr> contract (equal values hash
        // equally) holds by construction.
        C::hash(self.as_bytes(), state);
    }
}

impl<C: Comparator> Ord for AsciiString<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        C::cmp(self.as_bytes(), other.as_bytes())
    }
}

impl<C: Comparator> PartialEq for AsciiString<C> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<AsciiStr<C>> for AsciiString<C> {
    fn eq(&self, other: &AsciiStr<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

/// Equality with `str` follows the comparator `C`: for
/// [`CaseInsensitive`](crate::CaseInsensitive) this is case-insensitive, not literal
/// string equality. The same applies to the other `str`/`String` comparison impls below.
impl<C: Comparator> PartialEq<str> for AsciiString<C> {
    fn eq(&self, other: &str) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<&str> for AsciiString<C> {
    fn eq(&self, other: &&str) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<AsciiString<C>> for str {
    fn eq(&self, other: &AsciiString<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<AsciiString<C>> for &str {
    fn eq(&self, other: &AsciiString<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<String> for AsciiString<C> {
    fn eq(&self, other: &String) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<AsciiString<C>> for String {
    fn eq(&self, other: &AsciiString<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<String> for AsciiStr<C> {
    fn eq(&self, other: &String) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<AsciiStr<C>> for String {
    fn eq(&self, other: &AsciiStr<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<&AsciiStr<C>> for AsciiString<C> {
    fn eq(&self, other: &&AsciiStr<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialEq<AsciiString<C>> for &AsciiStr<C> {
    fn eq(&self, other: &AsciiString<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialOrd for AsciiString<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
