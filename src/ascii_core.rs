#[cfg(feature = "alloc")]
extern crate alloc;

use {
    crate::{CaseSensitive, Comparator, FromAsciiError},
    core::{
        cmp::{Eq, Ordering, PartialEq},
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
        marker::PhantomData,
        ops::Index,
        slice::SliceIndex,
    },
};

/// ASCII string slice type with pluggable case sensitivity.
///
/// This is the borrowed counterpart to [`AsciiString`](crate::AsciiString), analogous to
/// how [`str`] relates to [`String`](alloc::string::String).
///
/// Comparisons — including comparisons with [`str`] and `String` — follow the comparator
/// `C`. For example, an `AsciiStr<CaseInsensitive>` holding `"hello"` compares equal to
/// both `"hello"` and `"HELLO"`; this is *not* literal string equality. Use
/// [`as_str`][Self::as_str] when exact equality is required.
#[repr(transparent)]
pub struct AsciiStr<C = CaseSensitive>(PhantomData<C>, [u8]);

impl<C> AsciiStr<C> {
    /// Converts a slice of bytes to an `AsciiStr`, validating that the bytes are valid
    /// ASCII.
    pub fn from_ascii(bytes: &[u8]) -> Result<&Self, FromAsciiError> {
        if bytes.is_ascii() {
            // SAFETY: The bytes have been validated as ASCII.
            Ok(unsafe { Self::from_ascii_unchecked(bytes) })
        } else {
            Err(FromAsciiError {
                #[cfg(feature = "alloc")]
                bytes: bytes.to_vec(),
            })
        }
    }

    /// Converts a slice of bytes to an `AsciiStr` without checking that the bytes are
    /// valid ASCII.
    ///
    /// # Safety
    /// The bytes passed in must be valid ASCII.
    #[inline(always)]
    pub const unsafe fn from_ascii_unchecked(bytes: &[u8]) -> &Self {
        // SAFETY: `AsciiStr` is `#[repr(transparent)]` over `[u8]`, so the cast preserves
        // layout, lifetime, and provenance.
        unsafe { &*(bytes as *const [u8] as *const AsciiStr<C>) }
    }

    /// Returns this `AsciiStr` as a byte slice.
    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.1
    }

    /// Returns this `AsciiStr` as a string slice.
    #[inline(always)]
    pub const fn as_str(&self) -> &str {
        // SAFETY: We guarantee that the bytes are valid ASCII, which is a subset of UTF-8, so this
        // conversion is always valid.
        unsafe { core::str::from_utf8_unchecked(&self.1) }
    }

    /// Returns `true` if this `AsciiStr` has a length of zero, and `false` otherwise.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.1.is_empty()
    }

    /// Returns the length of this `AsciiStr` in bytes.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.1.len()
    }
}

impl<C> AsRef<[u8]> for AsciiStr<C> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<C> AsRef<str> for AsciiStr<C> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<C> Debug for AsciiStr<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // SAFETY: We guarantee that the bytes are valid ASCII, which is a subset of UTF-8, so this
        // conversion is always valid.
        Debug::fmt(unsafe { str::from_utf8_unchecked(&self.1) }, f)
    }
}

impl<C> Default for &AsciiStr<C> {
    fn default() -> Self {
        // SAFETY: An empty slice is trivially valid ASCII.
        unsafe { AsciiStr::from_ascii_unchecked(&[]) }
    }
}

#[cfg(feature = "serde")]
impl<'de: 'a, 'a, C> serde::Deserialize<'de> for &'a AsciiStr<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        AsciiStr::from_ascii(s.as_bytes()).map_err(serde::de::Error::custom)
    }
}

impl<C> Display for AsciiStr<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(self.as_str(), f)
    }
}

impl<C, I> Index<I> for AsciiStr<C>
where
    I: SliceIndex<[u8], Output = [u8]>,
{
    type Output = AsciiStr<C>;

    fn index(&self, index: I) -> &Self::Output {
        // SAFETY: Any subslice of valid ASCII is valid ASCII.
        unsafe { Self::from_ascii_unchecked(&self.1[index]) }
    }
}

#[cfg(feature = "serde")]
impl<C> serde::Serialize for AsciiStr<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

#[cfg(feature = "alloc")]
impl<C> alloc::borrow::ToOwned for AsciiStr<C> {
    type Owned = crate::AsciiString<C>;

    fn to_owned(&self) -> crate::AsciiString<C> {
        crate::AsciiString(PhantomData, self.1.to_vec())
    }
}

impl<'a, C> TryFrom<&'a [u8]> for &'a AsciiStr<C> {
    type Error = FromAsciiError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        AsciiStr::from_ascii(value)
    }
}

impl<'a, C> TryFrom<&'a str> for &'a AsciiStr<C> {
    type Error = FromAsciiError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        AsciiStr::from_ascii(value.as_bytes())
    }
}

impl<C: Comparator> Eq for AsciiStr<C> {}

impl<C: Comparator> Hash for AsciiStr<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Delegate to AsciiStr so the Borrow<AsciiStr> contract (equal values hash
        // equally) holds by construction.
        C::hash(self.as_bytes(), state);
    }
}

impl<C: Comparator> Ord for AsciiStr<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        C::cmp(self.as_bytes(), other.as_bytes())
    }
}

impl<C: Comparator> PartialEq for AsciiStr<C> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

#[cfg(feature = "alloc")]
impl<C: Comparator> PartialEq<crate::AsciiString<C>> for AsciiStr<C> {
    fn eq(&self, other: &crate::AsciiString<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

/// Equality with `str` follows the comparator `C`: for
/// [`CaseInsensitive`](crate::CaseInsensitive) this is case-insensitive, not literal
/// string equality.
impl<C: Comparator> PartialEq<str> for AsciiStr<C> {
    fn eq(&self, other: &str) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

/// Equality with `str` follows the comparator `C`: for
/// [`CaseInsensitive`](crate::CaseInsensitive) this is case-insensitive, not literal
/// string equality.
impl<C: Comparator> PartialEq<AsciiStr<C>> for str {
    fn eq(&self, other: &AsciiStr<C>) -> bool {
        C::cmp(self.as_bytes(), other.as_bytes()) == Ordering::Equal
    }
}

impl<C: Comparator> PartialOrd for AsciiStr<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
