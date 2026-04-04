use crate::SemverError;
use crate::error::SemverErrorKind;

/// JavaScript's `Number.MAX_SAFE_INTEGER` (2^53 − 1).
pub(crate) const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

pub(crate) fn parse_core_number_digits(digits: &[u8]) -> Result<u64, SemverError> {
    let mut value = 0u64;
    for &digit in digits {
        if !digit.is_ascii_digit() {
            return Err(SemverErrorKind::InvalidNumber.into());
        }
        value = value * 10 + u64::from(digit - b'0');
    }
    Ok(value)
}

pub(crate) fn parse_nr(s: &str) -> Result<u64, SemverError> {
    let b = s.as_bytes();
    if b.is_empty() {
        return Err(SemverErrorKind::Empty.into());
    }
    if b.len() > 1 && b[0] == b'0' {
        return Err(SemverErrorKind::LeadingZero.into());
    }
    if b.len() > 16 {
        return Err(SemverErrorKind::MaxSafeIntegerExceeded.into());
    }
    let mut n: u64 = 0;
    for &byte in b {
        if !byte.is_ascii_digit() {
            return Err(SemverErrorKind::InvalidNumber.into());
        }
        n = n * 10 + u64::from(byte - b'0');
    }
    if n > MAX_SAFE_INTEGER {
        return Err(SemverErrorKind::MaxSafeIntegerExceeded.into());
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_nr_api() {
        assert_eq!(parse_nr("0").unwrap(), 0);
        assert_eq!(parse_nr("9007199254740991").unwrap(), MAX_SAFE_INTEGER);
        assert!(parse_nr("").is_err());
        assert!(parse_nr("01").is_err());
        assert!(parse_nr("1a").is_err());
        assert!(parse_nr("9007199254740992").is_err());
        assert!(parse_nr("12345678901234567").is_err());
    }

    #[test]
    fn parse_core_number_digits_rejects_non_digits() {
        assert!(parse_core_number_digits(b"1a").is_err());
    }
}
