//! ASCII string types with pluggable case sensitivity.
#![no_std]
#![warn(clippy::all)]
#![allow(clippy::manual_range_contains)]
#![deny(
    missing_docs,
    rustdoc::bare_urls,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::private_intra_doc_links,
    rustdoc::unescaped_backticks
)]
#![cfg_attr(doc, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod ascii_core;
mod comparator;
pub use {ascii_core::*, comparator::*};

#[cfg(feature = "alloc")]
mod ascii_alloc;

#[cfg(feature = "alloc")]
pub use ascii_alloc::*;

#[cfg(test)]
mod tests;

use core::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

/// A possible error value when converting ASCII bytes to [`AsciiString`] or [`AsciiStr`].
#[derive(Debug)]
pub struct FromAsciiError {
    #[cfg(feature = "alloc")]
    bytes: alloc::vec::Vec<u8>,
}

impl Display for FromAsciiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        #[cfg(feature = "alloc")]
        {
            write!(f, "Invalid ASCII bytes: {:?}", self.bytes)
        }
        #[cfg(not(feature = "alloc"))]
        {
            write!(f, "Invalid ASCII bytes")
        }
    }
}

impl Error for FromAsciiError {}
