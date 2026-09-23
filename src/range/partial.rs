use crate::SemverError;
use crate::error::{Position, SemverErrorKind};
use crate::identifier::{PreRelease, validate_build_metadata};
use crate::number::{MAX_SAFE_INTEGER, MAX_SAFE_INTEGER_DIGITS, parse_nr};

use super::Partial;

// --------------------------------------------------------------------------
// Partial version (for range parsing)
// --------------------------------------------------------------------------

struct ParsedPartialCore {
    major: Option<u64>,
    minor: Option<u64>,
    patch: Option<u64>,
    component_count: usize,
    has_wildcard: bool,
}

pub(super) fn parse_partial(s: &str) -> Result<Partial, SemverError> {
    let s = s.trim();
    let s = if has_fully_qualified_numeric_core_after_full_strip(s) {
        if s.starts_with('=') {
            return Err(
                SemverErrorKind::UnexpectedCharacterWhileParsing('=', Position::Major).into(),
            );
        }
        s.strip_prefix(['v', '=']).unwrap_or(s)
    } else {
        s.trim_start_matches(['v', '='])
    };
    if let Some(partial) = parse_simple_partial(s) {
        return Ok(partial);
    }
    let original_len = s.len();
    let (s, pre_separator) = strip_build_metadata_and_find_prerelease(s)?;
    if s.len() != original_len {
        if let Some(partial) = parse_simple_partial(s) {
            return Ok(partial);
        }
    }
    let version_end = pre_separator.unwrap_or(s.len());
    let version_core = &s[..version_end];
    let terminator = pre_separator
        .map(|_| '-')
        .or_else(|| (s.len() != original_len).then_some('+'));
    let core = parse_partial_core(version_core, terminator)?;

    let pre_release = parse_partial_pre_release(s, pre_separator, &core)?;

    Ok(Partial {
        major: core.major,
        minor: core.minor,
        patch: core.patch,
        pre_release,
    })
}

fn parse_partial_pre_release(
    s: &str,
    pre_separator: Option<usize>,
    core: &ParsedPartialCore,
) -> Result<PreRelease, SemverError> {
    let mut pre_release = PreRelease::default();
    if let Some(pre_separator) = pre_separator {
        let pre_part = &s[pre_separator + 1..];
        let parsed = PreRelease::new(pre_part)?;
        if core.has_wildcard {
            if core.component_count < 3 {
                return Err(SemverErrorKind::UnexpectedCharacterAfterWildcard.into());
            }
            pre_release = PreRelease::default();
        } else {
            if core.minor.is_none() || core.patch.is_none() {
                let position = if core.minor.is_none() {
                    Position::Minor
                } else {
                    Position::Patch
                };
                return Err(SemverErrorKind::MissingVersionSegment(position).into());
            }
            pre_release = parsed;
        }
    }
    Ok(pre_release)
}

pub(super) fn strip_build_metadata(s: &str) -> Result<&str, SemverError> {
    let Some(plus) = s.find('+') else {
        return Ok(s);
    };
    let build = &s[plus + 1..];
    if build.is_empty() {
        return Err(SemverErrorKind::EmptyIdentifierSegment(Position::BuildMetadata).into());
    }
    validate_build_metadata(build)?;
    Ok(&s[..plus])
}

pub(super) fn strip_build_metadata_and_find_prerelease(
    s: &str,
) -> Result<(&str, Option<usize>), SemverError> {
    let mut pre_separator = None;
    for (pos, byte) in s.bytes().enumerate() {
        match byte {
            b'-' if pre_separator.is_none() => pre_separator = Some(pos),
            b'+' => {
                let build = &s[pos + 1..];
                if build.is_empty() {
                    return Err(
                        SemverErrorKind::EmptyIdentifierSegment(Position::BuildMetadata).into(),
                    );
                }
                validate_build_metadata(build)?;
                return Ok((&s[..pos], pre_separator));
            }
            _ => {}
        }
    }
    Ok((s, pre_separator))
}

fn parse_simple_partial(s: &str) -> Option<Partial> {
    let bytes = s.as_bytes();
    let (major, mut pos) = parse_simple_component(bytes, 0)?;
    let mut minor = None;
    let mut patch = None;

    if pos < bytes.len() {
        if bytes[pos] != b'.' {
            return None;
        }
        (minor, pos) = parse_simple_component(bytes, pos + 1)?;
    }

    if pos < bytes.len() {
        if bytes[pos] != b'.' {
            return None;
        }
        (patch, pos) = parse_simple_component(bytes, pos + 1)?;
    }

    if pos != bytes.len() {
        return None;
    }

    Some(Partial {
        major,
        minor,
        patch,
        pre_release: PreRelease::default(),
    })
}

fn parse_simple_component(bytes: &[u8], start: usize) -> Option<(Option<u64>, usize)> {
    match bytes.get(start).copied()? {
        b'x' | b'X' | b'*' => Some((None, start + 1)),
        first @ b'0'..=b'9' => {
            if first == b'0' && bytes.get(start + 1).is_some_and(u8::is_ascii_digit) {
                return None;
            }

            let mut pos = start;
            let mut value = 0u64;
            while let Some(digit @ b'0'..=b'9') = bytes.get(pos).copied() {
                if pos - start == MAX_SAFE_INTEGER_DIGITS {
                    return None;
                }
                value = value * 10 + u64::from(digit - b'0');
                pos += 1;
            }
            (value <= MAX_SAFE_INTEGER).then_some((Some(value), pos))
        }
        _ => None,
    }
}

fn parse_partial_core(
    core: &str,
    terminator: Option<char>,
) -> Result<ParsedPartialCore, SemverError> {
    let mut segments = core.split('.');
    let major_segment = segments.next().unwrap_or_default();
    let minor_segment = segments.next();
    let patch_segment = segments.next();
    let has_extra_segment = segments.next().is_some();

    let (major, major_wildcard) = parse_partial_component(
        major_segment,
        Position::Major,
        minor_segment.is_none(),
        terminator,
    )?;
    let (minor, minor_wildcard) = if let Some(segment) = minor_segment {
        parse_partial_component(
            segment,
            Position::Minor,
            patch_segment.is_none(),
            terminator,
        )?
    } else {
        (None, false)
    };
    let (patch, patch_wildcard) = if let Some(segment) = patch_segment {
        parse_partial_component(segment, Position::Patch, !has_extra_segment, terminator)?
    } else {
        (None, false)
    };

    if has_extra_segment {
        return Err(SemverErrorKind::UnexpectedCharacterAfter('.', Position::Patch).into());
    }
    if (major_wildcard && (minor.is_some() || patch.is_some()))
        || (minor_wildcard && patch.is_some())
    {
        return Err(SemverErrorKind::UnexpectedCharacterAfterWildcard.into());
    }

    let component_count =
        1 + usize::from(minor_segment.is_some()) + usize::from(patch_segment.is_some());
    Ok(ParsedPartialCore {
        major,
        minor,
        patch,
        component_count,
        has_wildcard: major_wildcard || minor_wildcard || patch_wildcard,
    })
}

fn parse_partial_component(
    segment: &str,
    position: Position,
    is_last: bool,
    terminator: Option<char>,
) -> Result<(Option<u64>, bool), SemverError> {
    if segment.is_empty() {
        if is_last {
            if let Some(terminator) = terminator {
                return Err(
                    SemverErrorKind::UnexpectedCharacterWhileParsing(terminator, position).into(),
                );
            }
            return Err(SemverErrorKind::MissingVersionSegment(position).into());
        }
        return Err(SemverErrorKind::UnexpectedCharacterWhileParsing('.', position).into());
    }

    match segment {
        "*" | "x" | "X" => Ok((None, true)),
        _ if matches!(segment.as_bytes().first(), Some(b'*' | b'x' | b'X')) => {
            Err(SemverErrorKind::UnexpectedCharacterAfterWildcard.into())
        }
        _ => Ok((Some(parse_nr(segment, position)?), false)),
    }
}

pub(super) fn has_fully_qualified_numeric_core_after_full_strip(s: &str) -> bool {
    if s.strip_prefix(['v', '=']).is_none() {
        return false;
    }
    let fully_stripped = s.trim_start_matches(['v', '=']);
    let core_end = fully_stripped
        .find(['-', '+'])
        .unwrap_or(fully_stripped.len());
    let core = &fully_stripped[..core_end];
    let mut parts = core.split('.');
    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    [major, minor, patch]
        .into_iter()
        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::{ParsedPartialCore, parse_partial_pre_release};

    #[test]
    fn no_prerelease_stays_empty() {
        let core = ParsedPartialCore {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            component_count: 3,
            has_wildcard: false,
        };
        assert!(
            parse_partial_pre_release("1.2.3", None, &core)
                .unwrap()
                .is_empty()
        );
    }
}
