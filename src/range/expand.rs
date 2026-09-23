#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::SemverError;
use crate::error::{Position, SemverErrorKind};
use crate::identifier::{BuildMetadata, PreRelease};
use crate::number::MAX_SAFE_INTEGER;
use crate::version::{Version, compare_core_and_prerelease};

use super::{Comparator, Operator, Partial};

// --------------------------------------------------------------------------
// Comparator construction helpers (internal)
// --------------------------------------------------------------------------

fn version_with_pre_release(
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: PreRelease,
) -> Version {
    if pre_release.is_empty() {
        return Version::new(major, minor, patch);
    }

    Version {
        major,
        minor,
        patch,
        pre_release,
        build: BuildMetadata::default(),
    }
}

const fn comparator_gte(ver: Version) -> Comparator {
    Comparator {
        op: Operator::GreaterThanOrEqual,
        version: ver,
    }
}
const fn comparator_gt(ver: Version) -> Comparator {
    Comparator {
        op: Operator::GreaterThan,
        version: ver,
    }
}
const fn comparator_lte(ver: Version) -> Comparator {
    Comparator {
        op: Operator::LessThanOrEqual,
        version: ver,
    }
}
const fn comparator_lt(ver: Version) -> Comparator {
    Comparator {
        op: Operator::LessThan,
        version: ver,
    }
}
const fn comparator_eq(ver: Version) -> Comparator {
    Comparator {
        op: Operator::Equal,
        version: ver,
    }
}

fn comparator_lt_upper_bound(major: u64, minor: u64, patch: u64) -> Comparator {
    comparator_lt(version_with_pre_release(
        major,
        minor,
        patch,
        PreRelease::zero(),
    ))
}

fn next_component(value: u64, position: Position) -> Result<u64, SemverError> {
    if value >= MAX_SAFE_INTEGER {
        return Err(SemverErrorKind::MaxSafeIntegerExceeded(position).into());
    }
    Ok(value + 1)
}

// --------------------------------------------------------------------------
// Range expansion helpers
// --------------------------------------------------------------------------

// A major-only tilde, caret, or equality range has the same bounds.
fn expand_major_range_into(
    out: &mut Vec<Comparator>,
    major: Option<u64>,
) -> Result<(), SemverError> {
    match major {
        None => {}
        Some(0) => push_canonical_comparator(out, comparator_lt_upper_bound(1, 0, 0)),
        Some(major) => {
            push_canonical_comparator(out, comparator_gte(Version::new(major, 0, 0)));
            push_canonical_comparator(
                out,
                comparator_lt_upper_bound(next_component(major, Position::Major)?, 0, 0),
            );
        }
    }
    Ok(())
}

/// Expand a tilde range: `~1.2.3` → `>=1.2.3 <1.3.0-0`.
pub(super) fn expand_tilde_into(out: &mut Vec<Comparator>, p: Partial) -> Result<(), SemverError> {
    if p.minor.is_none() {
        return expand_major_range_into(out, p.major);
    }
    let (Some(major), Some(minor)) = (p.major, p.minor) else {
        return Ok(());
    };
    let floor = match p.patch {
        None => Version::new(major, minor, 0),
        Some(patch) => version_with_pre_release(major, minor, patch, p.pre_release),
    };
    push_canonical_comparator(out, comparator_gte(floor));
    push_canonical_comparator(
        out,
        comparator_lt_upper_bound(major, next_component(minor, Position::Minor)?, 0),
    );
    Ok(())
}

#[cfg(test)]
pub(super) fn expand_tilde(p: Partial) -> Result<Vec<Comparator>, SemverError> {
    let mut out = Vec::with_capacity(2);
    expand_tilde_into(&mut out, p)?;
    Ok(out)
}

/// Expand a caret range: `^1.2.3` → `>=1.2.3 <2.0.0-0`.
pub(super) fn expand_caret_into(out: &mut Vec<Comparator>, p: Partial) -> Result<(), SemverError> {
    if p.minor.is_none() {
        return expand_major_range_into(out, p.major);
    }
    let (Some(major), Some(minor)) = (p.major, p.minor) else {
        return Ok(());
    };
    let floor = match p.patch {
        None => Version::new(major, minor, 0),
        Some(patch) => version_with_pre_release(major, minor, patch, p.pre_release),
    };
    push_canonical_comparator(out, comparator_gte(floor));
    let upper = if major > 0 {
        comparator_lt_upper_bound(next_component(major, Position::Major)?, 0, 0)
    } else if minor > 0 {
        comparator_lt_upper_bound(0, next_component(minor, Position::Minor)?, 0)
    } else if let Some(patch) = p.patch {
        comparator_lt_upper_bound(0, 0, next_component(patch, Position::Patch)?)
    } else {
        comparator_lt_upper_bound(0, 1, 0)
    };
    push_canonical_comparator(out, upper);
    Ok(())
}

#[cfg(test)]
pub(super) fn expand_caret(p: Partial) -> Result<Vec<Comparator>, SemverError> {
    let mut out = Vec::with_capacity(2);
    expand_caret_into(&mut out, p)?;
    Ok(out)
}

/// Expand an x-range or primitive comparator to concrete [`Comparator`]s.
pub(super) fn expand_primitive_into(
    out: &mut Vec<Comparator>,
    op: Option<Operator>,
    p: Partial,
) -> Result<(), SemverError> {
    if (p.major.is_none() && p.minor.is_some()) || (p.minor.is_none() && p.patch.is_some()) {
        return Err(SemverErrorKind::UnexpectedCharacterAfterWildcard.into());
    }
    match op {
        None | Some(Operator::Equal) => expand_equal_primitive(out, p)?,
        Some(Operator::GreaterThan) => expand_greater_than_primitive(out, p)?,
        Some(Operator::GreaterThanOrEqual) => expand_greater_than_or_equal_primitive(out, p),
        Some(Operator::LessThan) => expand_less_than_primitive(out, p),
        Some(Operator::LessThanOrEqual) => expand_partial_upper_bound_into(out, p)?,
    }
    Ok(())
}

fn version_from_partial(p: Partial, major: u64, minor: u64, patch: u64) -> Version {
    version_with_pre_release(major, minor, patch, p.pre_release)
}

fn expand_equal_primitive(out: &mut Vec<Comparator>, p: Partial) -> Result<(), SemverError> {
    if let (Some(major), Some(minor), Some(patch)) = (p.major, p.minor, p.patch) {
        push_canonical_comparator(
            out,
            comparator_eq(version_from_partial(p, major, minor, patch)),
        );
        return Ok(());
    }
    // Partial equality and tilde ranges cover the same interval.
    expand_tilde_into(out, p)
}

fn expand_greater_than_primitive(out: &mut Vec<Comparator>, p: Partial) -> Result<(), SemverError> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => push_canonical_comparator(out, comparator_lt_upper_bound(0, 0, 0)),
        (Some(maj), None, _) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(
                    next_component(maj, Position::Major)?,
                    0,
                    0,
                    PreRelease::default(),
                )),
            );
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(
                    maj,
                    next_component(mnr, Position::Minor)?,
                    0,
                    PreRelease::default(),
                )),
            );
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(out, comparator_gt(version_from_partial(p, maj, mnr, patch)));
        }
    }
    Ok(())
}

fn expand_greater_than_or_equal_primitive(out: &mut Vec<Comparator>, p: Partial) {
    match (p.major, p.minor, p.patch) {
        (None, _, _) | (Some(0), None, _) => {}
        (Some(maj), None, _) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, 0, 0, PreRelease::default())),
            );
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, mnr, 0, PreRelease::default())),
            );
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_from_partial(p, maj, mnr, patch)),
            );
        }
    }
}

fn expand_less_than_primitive(out: &mut Vec<Comparator>, p: Partial) {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => push_canonical_comparator(out, comparator_lt_upper_bound(0, 0, 0)),
        (Some(maj), None, _) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, 0, 0));
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, mnr, 0));
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(out, comparator_lt(version_from_partial(p, maj, mnr, patch)));
        }
    }
}

// Used by both `<=` and the right-hand side of a hyphen range.
fn expand_partial_upper_bound_into(
    out: &mut Vec<Comparator>,
    p: Partial,
) -> Result<(), SemverError> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => {}
        (Some(maj), None, _) => {
            push_canonical_comparator(
                out,
                comparator_lt_upper_bound(next_component(maj, Position::Major)?, 0, 0),
            );
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(
                out,
                comparator_lt_upper_bound(maj, next_component(mnr, Position::Minor)?, 0),
            );
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(
                out,
                comparator_lte(version_from_partial(p, maj, mnr, patch)),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn expand_primitive(
    op: Option<Operator>,
    p: Partial,
) -> Result<Vec<Comparator>, SemverError> {
    let mut out = Vec::with_capacity(2);
    expand_primitive_into(&mut out, op, p)?;
    Ok(out)
}

/// Expand a hyphen range `a - b` to comparators.
fn expand_hyphen_into(
    out: &mut Vec<Comparator>,
    a: Partial,
    b: Partial,
) -> Result<(), SemverError> {
    push_canonical_comparator(out, comparator_gte(a.floor()));
    expand_partial_upper_bound_into(out, b)
}

pub(super) fn expand_hyphen(a: Partial, b: Partial) -> Result<Vec<Comparator>, SemverError> {
    let mut out = Vec::with_capacity(2);
    expand_hyphen_into(&mut out, a, b)?;
    Ok(out)
}

fn push_canonical_comparator(all: &mut Vec<Comparator>, new: Comparator) {
    if is_impossible_comparator(&new) {
        all.clear();
        all.push(new);
        return;
    }
    for existing in &mut *all {
        if is_impossible_comparator(existing) {
            return;
        }
        match (existing.op, new.op) {
            (
                Operator::LessThan | Operator::LessThanOrEqual,
                Operator::LessThan | Operator::LessThanOrEqual,
            ) if existing.version.major == new.version.major
                && existing.version.minor == new.version.minor
                && existing.version.patch == new.version.patch =>
            {
                let ordering = compare_core_and_prerelease(&existing.version, &new.version);
                if ordering == core::cmp::Ordering::Greater
                    || (ordering == core::cmp::Ordering::Equal
                        && existing.op == Operator::LessThanOrEqual
                        && new.op == Operator::LessThan)
                {
                    *existing = new;
                }
                return;
            }
            (Operator::Equal, Operator::Equal)
                if compare_core_and_prerelease(&existing.version, &new.version)
                    == core::cmp::Ordering::Equal =>
            {
                return;
            }
            _ => {}
        }
    }
    all.push(new);
}

fn is_impossible_comparator(comparator: &Comparator) -> bool {
    comparator.op == Operator::LessThan
        && comparator.version.major == 0
        && comparator.version.minor == 0
        && comparator.version.patch == 0
        && comparator.version.pre_release == PreRelease::zero()
}
