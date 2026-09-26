use crate::SemverError;
use crate::error::{Position, SemverErrorKind};

/// JavaScript's `Number.MAX_SAFE_INTEGER` (2^53 − 1).
pub(crate) const MAX_SAFE_INTEGER: u64 = 0x1F_FFFF_FFFF_FFFF;
pub(crate) const MAX_SAFE_INTEGER_DIGITS: usize = 16;

pub(crate) fn parse_nr(s: &str, position: Position) -> Result<u64, SemverError> {
    let b = s.as_bytes();
    if b.is_empty() {
        return Err(SemverErrorKind::MissingVersionSegment(position).into());
    }
    if b[0] == b'0' && b.get(1).is_some_and(u8::is_ascii_digit) {
        return Err(SemverErrorKind::LeadingZero(position).into());
    }
    let mut n: u64 = 0;
    for (digit_count, ch) in s.chars().enumerate() {
        if !ch.is_ascii_digit() {
            let kind = if digit_count == 0 {
                SemverErrorKind::UnexpectedCharacterWhileParsing(ch, position)
            } else {
                SemverErrorKind::UnexpectedCharacterAfter(ch, position)
            };
            return Err(kind.into());
        }
        if digit_count == MAX_SAFE_INTEGER_DIGITS {
            return Err(SemverErrorKind::MaxSafeIntegerExceeded(position).into());
        }
        n = n * 10 + u64::from(u32::from(ch) - u32::from('0'));
    }
    if n > MAX_SAFE_INTEGER {
        return Err(SemverErrorKind::MaxSafeIntegerExceeded(position).into());
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_nr_api() {
        assert_eq!(parse_nr("0", Position::Major).unwrap(), 0);
        assert_eq!(
            parse_nr("9007199254740991", Position::Major).unwrap(),
            MAX_SAFE_INTEGER
        );
        assert!(parse_nr("", Position::Major).is_err());
        assert!(parse_nr("01", Position::Major).is_err());
        assert!(parse_nr("1a", Position::Major).is_err());
        assert!(parse_nr("9007199254740992", Position::Major).is_err());
        assert!(parse_nr("12345678901234567", Position::Major).is_err());
    }
}
