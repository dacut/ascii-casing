// The test harness always runs on a host with std available, so std and alloc are usable
// here regardless of crate features. Feature gates below reflect what the *crate* exports,
// not what the tests may link against.
extern crate alloc;
extern crate std;

use {
    super::{AsciiStr, CaseInsensitive, CaseSensitive},
    alloc::{format, string::ToString, vec},
    core::cmp::Ordering,
    pretty_assertions::{assert_eq, assert_ne},
};

#[cfg(feature = "alloc")]
use {
    super::AsciiString,
    alloc::{borrow::ToOwned, collections::BTreeMap, string::String},
    std::collections::HashMap,
};

type SensitiveStr = AsciiStr<CaseSensitive>;
type InsensitiveStr = AsciiStr<CaseInsensitive>;
#[cfg(feature = "alloc")]
type SensitiveString = AsciiString<CaseSensitive>;
#[cfg(feature = "alloc")]
type InsensitiveString = AsciiString<CaseInsensitive>;

#[test]
fn test_ascii_str_from_ascii() {
    let s = SensitiveStr::from_ascii(b"Hello, World!").unwrap();
    assert_eq!(s.as_str(), "Hello, World!");
    assert_eq!(s.as_bytes(), b"Hello, World!");
    assert_eq!(s.len(), 13);
    assert!(!s.is_empty());

    let e = SensitiveStr::from_ascii("héllo".as_bytes()).unwrap_err();
    #[cfg(feature = "alloc")]
    assert_eq!(e.to_string(), format!("Invalid ASCII bytes: {:?}", "héllo".as_bytes()));
    #[cfg(not(feature = "alloc"))]
    assert_eq!(e.to_string(), "Invalid ASCII bytes");

    let s: &SensitiveStr = "Hello".try_into().unwrap();
    assert_eq!(s.as_str(), "Hello");
    let s: &SensitiveStr = b"Hello".as_slice().try_into().unwrap();
    assert_eq!(s.as_str(), "Hello");
    assert!(<&SensitiveStr>::try_from("héllo").is_err());

    let empty = <&SensitiveStr>::default();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_case_sensitive_eq() {
    let a = SensitiveStr::from_ascii(b"Hello").unwrap();
    let b = SensitiveStr::from_ascii(b"hELLO").unwrap();
    let c = SensitiveStr::from_ascii(b"Hello").unwrap();
    assert_ne!(a, b);
    assert_eq!(a, c);

    assert!(a == "Hello");
    assert!(a != "hELLO");
    assert!("Hello" == a);
    assert!("hELLO" != a);

    #[cfg(feature = "alloc")]
    {
        let owned = SensitiveString::from_ascii(b"Hello".to_vec()).unwrap();
        assert!(*a == owned);
        assert!(owned == *a);
        assert!(owned == a);
        assert!(a == owned);
        assert!(owned == "Hello");
        assert!(owned != "hELLO");
        assert!("Hello" == owned);
        // Owned strings are constructed on purpose: this exercises the `PartialEq<String>`
        // impls.
        #[allow(clippy::cmp_owned)]
        {
            assert!(owned == String::from("Hello"));
            assert!(String::from("hELLO") != owned);
        }
        let other = SensitiveString::from_ascii(b"hELLO".to_vec()).unwrap();
        assert!(*a != other);
    }
}

#[test]
fn test_case_insensitive_eq() {
    let a = InsensitiveStr::from_ascii(b"Hello").unwrap();
    let b = InsensitiveStr::from_ascii(b"hELLO").unwrap();
    let c = InsensitiveStr::from_ascii(b"goodbye").unwrap();
    assert_eq!(a, b);
    assert_ne!(a, c);

    assert!(a == "HELLO");
    assert!("hello" == a);
    assert!(a != "goodbye");
    // Non-ASCII strings can never compare equal: the left side is always pure ASCII.
    assert!(a != "héllo");

    #[cfg(feature = "alloc")]
    {
        let owned = InsensitiveString::from_ascii(b"HELLO".to_vec()).unwrap();
        assert!(*a == owned);
        assert!(owned == *a);
        assert!(owned == a);
        assert!(a == owned);
        assert!(owned == "hello");
        assert!("HeLLo" == owned);
        // Owned strings are constructed on purpose: this exercises the `PartialEq<String>`
        // impls.
        #[allow(clippy::cmp_owned)]
        {
            assert!(owned == String::from("hello"));
            assert!(String::from("HELLO") == owned);
        }
    }
}

#[test]
fn test_case_sensitive_ord() {
    let apple = SensitiveStr::from_ascii(b"APPLE").unwrap();
    let banana = SensitiveStr::from_ascii(b"banana").unwrap();
    let zebra = SensitiveStr::from_ascii(b"Zebra").unwrap();

    // Byte-wise ordering: all uppercase letters sort before all lowercase letters.
    assert!(apple < banana);
    assert!(zebra < banana);
    assert_eq!(apple.cmp(SensitiveStr::from_ascii(b"apple").unwrap()), Ordering::Less);

    let mut values = vec![banana, zebra, apple];
    values.sort();
    assert_eq!(values, vec![apple, zebra, banana]);

    #[cfg(feature = "alloc")]
    {
        let mut map: BTreeMap<SensitiveString, u32> = BTreeMap::new();
        map.insert(SensitiveString::from_ascii(b"Content-Type".to_vec()).unwrap(), 42);
        assert_eq!(map.get(SensitiveStr::from_ascii(b"Content-Type").unwrap()), Some(&42));
        assert_eq!(map.get(SensitiveStr::from_ascii(b"CONTENT-TYPE").unwrap()), None);
    }
}

#[test]
fn test_case_insensitive_ord() {
    let apple = InsensitiveStr::from_ascii(b"APPLE").unwrap();
    let banana = InsensitiveStr::from_ascii(b"banana").unwrap();
    let zebra = InsensitiveStr::from_ascii(b"Zebra").unwrap();

    assert!(apple < banana);
    assert!(zebra > banana);
    assert_eq!(apple.cmp(InsensitiveStr::from_ascii(b"apple").unwrap()), Ordering::Equal);

    // A common prefix means the shorter string sorts first.
    assert!(InsensitiveStr::from_ascii(b"App").unwrap() < apple);

    let mut values = vec![zebra, apple, banana];
    values.sort();
    assert_eq!(values, vec![apple, banana, zebra]);

    #[cfg(feature = "alloc")]
    {
        let mut map: BTreeMap<InsensitiveString, u32> = BTreeMap::new();
        map.insert(InsensitiveString::from_ascii(b"Content-Type".to_vec()).unwrap(), 42);
        assert_eq!(map.get(InsensitiveStr::from_ascii(b"CONTENT-TYPE").unwrap()), Some(&42));

        let a = InsensitiveString::from_ascii(b"APPLE".to_vec()).unwrap();
        let b = InsensitiveString::from_ascii(b"banana".to_vec()).unwrap();
        assert!(a < b);
        assert_eq!(a.cmp(&a), Ordering::Equal);
    }
}

#[test]
fn test_ascii_str_index() {
    let s = InsensitiveStr::from_ascii(b"Hello, World!").unwrap();
    assert_eq!(s[..].as_str(), "Hello, World!");
    assert_eq!(s[7..12].as_str(), "World");
    assert!(&s[..5] == "hello");

    let s = SensitiveStr::from_ascii(b"Hello, World!").unwrap();
    assert!(&s[..5] != "hello");
    assert!(&s[..5] == "Hello");
}

#[test]
fn test_display_debug() {
    let s = SensitiveStr::from_ascii(b"Hello").unwrap();
    assert_eq!(format!("{s}"), "Hello");
    assert_eq!(format!("{s:?}"), "\"Hello\"");

    #[cfg(feature = "alloc")]
    {
        let owned = SensitiveString::from_ascii(b"Hello".to_vec()).unwrap();
        assert_eq!(format!("{owned}"), "Hello");
        assert_eq!(format!("{owned:?}"), "\"Hello\"");
    }
}

#[cfg(feature = "alloc")]
#[test]
fn test_ascii_string_push() {
    let mut s = SensitiveString::new();
    s.push(b'H').unwrap();
    s.push(b'i').unwrap();
    assert_eq!(s.as_str(), "Hi");

    let e = s.push(0x80).unwrap_err();
    assert_eq!(e.to_string(), "Invalid ASCII bytes: [128]");
    assert_eq!(s.as_str(), "Hi");
}

#[cfg(feature = "alloc")]
#[test]
fn test_ascii_string_deref_and_borrow() {
    let owned = InsensitiveString::from_ascii(b"Content-Type".to_vec()).unwrap();
    let slice: &InsensitiveStr = &owned;
    assert_eq!(slice.as_str(), "Content-Type");
    assert_eq!(owned.as_ascii_str(), slice);
    assert_eq!(owned[8..].as_str(), "Type");

    let roundtrip: InsensitiveString = slice.to_owned();
    assert_eq!(roundtrip, owned);
    let converted = InsensitiveString::from(slice);
    assert_eq!(converted, owned);

    let cloned = owned.clone();
    assert_eq!(cloned, owned);
    let mut target = InsensitiveString::new();
    target.clone_from(&owned);
    assert_eq!(target, owned);
}

#[cfg(feature = "alloc")]
#[test]
fn test_hash_map() {
    let mut map: HashMap<InsensitiveString, u32> = HashMap::new();
    map.insert(InsensitiveString::from_ascii(b"Content-Type".to_vec()).unwrap(), 42);
    assert_eq!(map.get(InsensitiveStr::from_ascii(b"CONTENT-TYPE").unwrap()), Some(&42));
    assert_eq!(map.get(InsensitiveStr::from_ascii(b"Content-Length").unwrap()), None);

    let mut map: HashMap<SensitiveString, u32> = HashMap::new();
    map.insert(SensitiveString::from_ascii(b"Content-Type".to_vec()).unwrap(), 42);
    assert_eq!(map.get(SensitiveStr::from_ascii(b"Content-Type").unwrap()), Some(&42));
    assert_eq!(map.get(SensitiveStr::from_ascii(b"CONTENT-TYPE").unwrap()), None);
}

#[cfg(feature = "serde")]
#[test]
fn test_serde() {
    let s = InsensitiveStr::from_ascii(b"Hello").unwrap();
    let json = serde_json::to_string(s).unwrap();
    assert_eq!(json, "\"Hello\"");

    let deserialized: &InsensitiveStr = serde_json::from_str("\"hELLO\"").unwrap();
    assert_eq!(deserialized, s);
    assert!(serde_json::from_str::<&InsensitiveStr>("\"héllo\"").is_err());

    #[cfg(feature = "alloc")]
    {
        let owned: InsensitiveString = serde_json::from_str("\"Hello\"").unwrap();
        assert!(owned == *s);
        assert_eq!(serde_json::to_string(&owned).unwrap(), "\"Hello\"");
    }
}
