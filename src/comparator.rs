use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

/// Comparator trait for ASCII strings, which determines how equality and ordering are defined.
pub trait Comparator {
    /// Generate a hash for the underlying bytes of an ASCII string, according to the rules
    /// of this comparator.
    fn hash<H: Hasher>(bytes: &[u8], state: &mut H);

    /// Compare two ASCII strings, according to the rules of this comparator.
    fn cmp(a: &[u8], b: &[u8]) -> Ordering;
}

/// Case-sensitive comparator for ASCII strings.
pub struct CaseSensitive;

/// Case-insensitive comparator for ASCII strings.
pub struct CaseInsensitive;

impl Comparator for CaseSensitive {
    fn hash<H: Hasher>(bytes: &[u8], state: &mut H) {
        bytes.hash(state);
    }

    fn cmp(a: &[u8], b: &[u8]) -> Ordering {
        a.cmp(b)
    }
}

impl Comparator for CaseInsensitive {
    fn hash<H: Hasher>(bytes: &[u8], state: &mut H) {
        for &b in bytes {
            state.write_u8(b.to_ascii_lowercase());
        }
    }

    fn cmp(a: &[u8], b: &[u8]) -> Ordering {
        // Compare byte-by-byte, converting to lowercase for each byte. This is more efficient than
        // allocating temporary lowercase strings.
        for (a_byte, b_byte) in a.iter().zip(b.iter()) {
            let a_lower = a_byte.to_ascii_lowercase();
            let b_lower = b_byte.to_ascii_lowercase();
            match a_lower.cmp(&b_lower) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            }
        }

        a.len().cmp(&b.len())
    }
}
