#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, allow(clippy::restriction))]
//! # js-semver
//!
//! <p>
//!   <a href="https://github.com/ryuapp/js-semver/blob/main/LICENSE">
//!     <img alt="License" src="https://img.shields.io/github/license/ryuapp/js-semver?labelColor=171717&color=39b54a&label=License">
//!   </a>
//!   <a href="https://crates.io/crates/js-semver">
//!     <img alt="crates" src="https://img.shields.io/crates/v/js-semver?labelColor=171717&color=39b54a">
//!   </a>
//!   <a href="https://github.com/ryuapp/js-semver">
//!     <img alt="github repo" src="https://img.shields.io/badge/GitHub-ryuapp/js--semver-171717?labelColor=171717&color=39b54a">
//!   </a>
//!   <a href="https://codecov.io/gh/ryuapp/js-semver">
//!     <img alt="codecov" src="https://codecov.io/gh/ryuapp/js-semver/graph/badge.svg?token=P7NMEB4IP7">
//!   </a>
//! </p>
//!
//! A parser and evaluator for npm's flavor of Semantic Versioning.
//!
//! This crate is a Rust implementation of [node-semver](https://github.com/npm/node-semver) (the one npm uses).
//! It maintains high compatibility and performance, and has zero dependencies by default.
//!
//!
//! # Examples
//!
//! ```rust
//! use js_semver::{BuildMetadata, PreRelease, Range, Version};
//!
//! fn main() {
//!     let range: Range = ">=4.1.0 <5.0.0".parse().unwrap();
//!
//!     // Pre-release versions are not included in the range unless explicitly specified.
//!     let version = Version {
//!         major: 4,
//!         minor: 1,
//!         patch: 0,
//!         pre_release: PreRelease::new("rc.1").unwrap(),
//!         build: BuildMetadata::default(),
//!     };
//!     assert!(!range.satisfies(&version));
//!
//!     // Stable version is included in the range.
//!     let version: Version = "4.1.0".parse().unwrap();
//!     assert!(range.satisfies(&version));
//! }
//! ```

#[cfg(not(feature = "std"))]
extern crate alloc;

// --------------------------------------------------------------------------
// Constants
// --------------------------------------------------------------------------

/// Maximum accepted length for any version or range string.
pub(crate) const MAX_LENGTH: usize = 256;

mod error;
mod identifier;
mod number;
mod range;
#[cfg(feature = "serde")]
mod serde;
mod version;

pub use error::SemverError;
pub use identifier::{BuildMetadata, PreRelease};
pub use range::Range;
pub use version::Version;
