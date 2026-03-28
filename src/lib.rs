#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, allow(clippy::restriction))]
//! A parser and evaluator for semantic versioning in JavaScript, including Node.js and Deno.
//!
//! # Examples
//!
//! ```rust
//! use js_semver::{BuildMetadata, PreRelease, Range, Version};
//!
//! let range: Range = ">=1.2.3 <1.8.0".parse().unwrap();
//!
//! let version = Version {
//!     major: 1,
//!     minor: 2,
//!     patch: 3,
//!     pre_release: PreRelease::new("alpha.1").unwrap(),
//!     build: BuildMetadata::default(),
//! };
//! assert!(!range.satisfies(&version));
//!
//! let version: Version = "1.3.0".parse().unwrap();
//! assert!(range.satisfies(&version));
//! ```
//!
//! ```rust
//! use core::cmp::Ordering;
//! use js_semver::Version;
//!
//! let stable: Version = "1.2.3+build.1".parse().unwrap();
//! let rebuilt: Version = "1.2.3+build.2".parse().unwrap();
//!
//! // Total ordering includes build metadata.
//! assert!(stable < rebuilt);
//! assert_ne!(stable, rebuilt);
//!
//! // SemVer precedence ignores build metadata.
//! assert_eq!(stable.cmp_precedence(&rebuilt), Ordering::Equal);
//! ```

#[cfg(not(feature = "std"))]
extern crate alloc;

// --------------------------------------------------------------------------
// Constants
// --------------------------------------------------------------------------

/// JavaScript's `Number.MAX_SAFE_INTEGER` (2^53 − 1).
pub(crate) const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
/// Maximum accepted length for any version or range string.
pub(crate) const MAX_LENGTH: usize = 256;

mod error;
mod range;
#[cfg(feature = "serde")]
mod serde;
mod version;

pub use error::SemverError;
pub use range::Range;
pub use version::{BuildMetadata, PreRelease, ReleaseType, Version};
