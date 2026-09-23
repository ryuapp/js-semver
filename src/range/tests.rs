#[cfg(not(feature = "std"))]
use alloc::string::ToString;

use super::*;

#[test]
fn parse_errors_include_the_cause_and_position() {
    let cases = [
        ("+", "empty identifier segment in build metadata"),
        ("1.", "missing minor version segment"),
        (
            "1..2",
            "unexpected character '.' while parsing minor version",
        ),
        (
            "1...3",
            "unexpected character '.' while parsing minor version",
        ),
        (
            "1.2..",
            "unexpected character '.' while parsing patch version",
        ),
        ("1.2.3.4", "unexpected character '.' after patch version"),
        (
            "1.2.3.4-alpha",
            "unexpected character '.' after patch version",
        ),
        ("1.2.3.", "unexpected character '.' after patch version"),
        (".1", "unexpected character '.' while parsing major version"),
        ("1-alpha", "missing minor version segment"),
        ("1.2-alpha", "missing patch version segment"),
        (
            "1.2.-alpha",
            "unexpected character '-' while parsing patch version",
        ),
        (
            "1.0.0-",
            "empty identifier segment in pre-release identifier",
        ),
        ("1.0.0+", "empty identifier segment in build metadata"),
        ("01.2.3", "invalid leading zero in major version"),
        ("1.02.3", "invalid leading zero in minor version"),
        ("1.2.03", "invalid leading zero in patch version"),
        (
            "abc",
            "unexpected character 'a' while parsing major version",
        ),
        (
            ">>1.0.0",
            "unexpected character '>' while parsing major version",
        ),
        (
            "1.x.5",
            "unexpected character after wildcard in version range",
        ),
        (
            "x.1.2",
            "unexpected character after wildcard in version range",
        ),
        ("x1", "unexpected character after wildcard in version range"),
        ("1.0.0!", "unexpected character '!' after patch version"),
        (
            "1.0.0-alpha!",
            "unexpected character '!' after pre-release identifier",
        ),
        (
            "1.0.0+build!",
            "unexpected character '!' after build metadata",
        ),
        (
            ">=a.b.c",
            "unexpected character 'a' while parsing major version",
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(
            Range::parse(input).unwrap_err().to_string(),
            expected,
            "input: {input:?}"
        );
    }
}

#[test]
fn max_safe_integer_errors_include_the_position() {
    let cases = [
        (
            "9007199254740992.0.0",
            "number exceeds MAX_SAFE_INTEGER in major version",
        ),
        (
            "~9007199254740991",
            "number exceeds MAX_SAFE_INTEGER in major version",
        ),
        (
            "~1.9007199254740991",
            "number exceeds MAX_SAFE_INTEGER in minor version",
        ),
        (
            "^0.0.9007199254740991",
            "number exceeds MAX_SAFE_INTEGER in patch version",
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(
            Range::parse(input).unwrap_err().to_string(),
            expected,
            "input: {input:?}"
        );
    }
}

#[test]
fn try_hyphen_rejects_non_hyphen_forms() {
    assert!(try_hyphen(">=1.0.0 - 2.0.0").unwrap().is_none());
    assert!(try_hyphen("1.0.0 - <=2.0.0").unwrap().is_none());
    assert!(try_hyphen("1.2.3").unwrap().is_none());
    assert!(try_hyphen("1.2.3 -").unwrap().is_none());
    assert!(try_hyphen("- 1.2.3").unwrap().is_none());
}

// --- Operator Display ---

#[test]
fn operator_display() {
    assert_eq!(Operator::LessThan.to_string(), "<");
    assert_eq!(Operator::LessThanOrEqual.to_string(), "<=");
    assert_eq!(Operator::GreaterThan.to_string(), ">");
    assert_eq!(Operator::GreaterThanOrEqual.to_string(), ">=");
    assert_eq!(Operator::Equal.to_string(), "=");
}

#[test]
fn helper_count_and_expand_tilde_caret_coverage() {
    assert_eq!(parse_partial("1.2").unwrap().minor, Some(2));
    assert_eq!(count_whitespace_tokens(b""), 0);
    assert_eq!(count_whitespace_tokens(b">=1.0.0 <2.0.0"), 2);
    assert_eq!(count_whitespace_tokens(b"  >=1.0.0   <2.0.0  "), 2);

    assert_eq!(expand_tilde(parse_partial("1").unwrap()).unwrap().len(), 2);
    assert_eq!(
        expand_tilde(parse_partial("1.2").unwrap()).unwrap().len(),
        2
    );

    assert_eq!(expand_caret(parse_partial("1").unwrap()).unwrap().len(), 2);
    assert_eq!(
        expand_caret(parse_partial("1.2").unwrap()).unwrap().len(),
        2
    );
    assert_eq!(
        expand_caret(parse_partial("1.2.3").unwrap()).unwrap().len(),
        2
    );
    assert_eq!(
        expand_caret(parse_partial("0.2.3").unwrap()).unwrap().len(),
        2
    );
}

#[test]
fn helper_build_metadata_stripping_and_length_coverage() {
    assert!(has_fully_qualified_numeric_core_after_full_strip("v1.2.3"));
    assert!(!has_fully_qualified_numeric_core_after_full_strip("v1.2.x"));
    assert!(!has_fully_qualified_numeric_core_after_full_strip("v1..3"));
    assert!(!has_fully_qualified_numeric_core_after_full_strip("v1.2"));
    assert_eq!(strip_build_metadata("1.2.3+A0-z.9").unwrap(), "1.2.3");
    assert_eq!(strip_build_metadata("1.2.3").unwrap(), "1.2.3");
    assert!(strip_build_metadata("1.2.3+").is_err());
    assert!(strip_build_metadata("1.2.3+bad!").is_err());
    assert_eq!(
        strip_build_metadata_and_find_prerelease("1.2.3-alpha-1+build").unwrap(),
        ("1.2.3-alpha-1", Some(5))
    );
    assert_eq!(
        strip_build_metadata_and_find_prerelease("1.2.3").unwrap(),
        ("1.2.3", None)
    );
    assert!(strip_build_metadata_and_find_prerelease("1.2.3+").is_err());
    assert!(strip_build_metadata_and_find_prerelease("1.2.3+bad!").is_err());
    assert_eq!(range_len_without_build_metadata("1.2.3+A0-z.9"), 5);
    assert_eq!(range_len_without_build_metadata("1.2.3+!"), 6);
}

#[test]
fn helper_expand_primitive_equal_coverage() {
    assert_eq!(
        expand_primitive(None, parse_partial("1").unwrap())
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        expand_primitive(None, parse_partial("1.2").unwrap())
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        expand_primitive(None, parse_partial("0").unwrap())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        expand_primitive(None, parse_partial("1.2.3-alpha").unwrap())
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn helper_expand_primitive_greater_coverage() {
    assert_eq!(
        expand_primitive(Some(Operator::GreaterThan), parse_partial("1").unwrap())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        expand_primitive(Some(Operator::GreaterThan), parse_partial("1.2").unwrap())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        expand_primitive(Some(Operator::GreaterThan), parse_partial("*").unwrap())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        expand_primitive(
            Some(Operator::GreaterThan),
            parse_partial("1.2.3-alpha").unwrap()
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn helper_expand_primitive_greater_equal_coverage() {
    assert_eq!(
        expand_primitive(
            Some(Operator::GreaterThanOrEqual),
            parse_partial("1").unwrap()
        )
        .unwrap()
        .len(),
        1
    );
    assert_eq!(
        expand_primitive(
            Some(Operator::GreaterThanOrEqual),
            parse_partial("1.2").unwrap()
        )
        .unwrap()
        .len(),
        1
    );
    assert_eq!(
        expand_primitive(
            Some(Operator::GreaterThanOrEqual),
            parse_partial("*").unwrap()
        )
        .unwrap()
        .len(),
        0
    );
    assert_eq!(
        expand_primitive(
            Some(Operator::GreaterThanOrEqual),
            parse_partial("1.2.3-alpha").unwrap()
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn helper_expand_primitive_less_coverage() {
    assert_eq!(
        expand_primitive(Some(Operator::LessThan), parse_partial("1").unwrap())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        expand_primitive(
            Some(Operator::LessThan),
            parse_partial("1.2.3-alpha").unwrap()
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn helper_expand_primitive_less_equal_coverage() {
    assert_eq!(
        expand_primitive(Some(Operator::LessThanOrEqual), parse_partial("1").unwrap())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        expand_primitive(
            Some(Operator::LessThanOrEqual),
            parse_partial("1.2").unwrap()
        )
        .unwrap()
        .len(),
        1
    );
    assert_eq!(
        expand_primitive(
            Some(Operator::LessThanOrEqual),
            parse_partial("1.2.3-alpha").unwrap()
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn helper_expand_hyphen_and_parse_range_coverage() {
    assert_eq!(
        expand_hyphen(parse_partial("1.0.0").unwrap(), parse_partial("2").unwrap())
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        expand_hyphen(
            parse_partial("1.0.0").unwrap(),
            parse_partial("2.5").unwrap()
        )
        .unwrap()
        .len(),
        2
    );
    assert!(try_hyphen("1.0.0 - 2.0.0").unwrap().is_some());
}

#[test]
fn helper_expand_error_paths() {
    let partial = parse_partial("1.2").unwrap();
    assert_eq!(partial.major, Some(1));
    assert_eq!(partial.minor, Some(2));
    assert_eq!(partial.patch, None);
    assert!(expand_tilde(parse_partial("9007199254740991").unwrap()).is_err());
    assert!(expand_tilde(parse_partial("1.9007199254740991").unwrap()).is_err());
    assert!(expand_tilde(parse_partial("1.9007199254740991.0").unwrap()).is_err());

    assert!(expand_caret(parse_partial("9007199254740991").unwrap()).is_err());
    assert!(expand_caret(parse_partial("9007199254740991.1").unwrap()).is_err());
    assert!(expand_caret(parse_partial("0.9007199254740991").unwrap()).is_err());
    assert!(expand_caret(parse_partial("0.9007199254740991.1").unwrap()).is_err());
    assert!(expand_caret(parse_partial("0.0.9007199254740991").unwrap()).is_err());
    assert_eq!(
        expand_caret(parse_partial("1.2").unwrap()).unwrap().len(),
        2
    );
    assert_eq!(
        expand_caret(parse_partial("0.2").unwrap()).unwrap().len(),
        2
    );
    let mut out = Vec::new();
    expand_caret_into(&mut out, parse_partial("0.2").unwrap()).unwrap();
    assert_eq!(out.len(), 2);

    assert!(expand_primitive(None, parse_partial("9007199254740991").unwrap()).is_err());
    assert!(expand_primitive(None, parse_partial("1.9007199254740991").unwrap()).is_err());
    assert!(
        expand_primitive(
            Some(Operator::GreaterThan),
            parse_partial("9007199254740991").unwrap()
        )
        .is_err()
    );
    assert!(
        expand_primitive(
            Some(Operator::GreaterThan),
            parse_partial("1.9007199254740991").unwrap()
        )
        .is_err()
    );
    assert!(
        expand_primitive(
            Some(Operator::LessThanOrEqual),
            parse_partial("9007199254740991").unwrap()
        )
        .is_err()
    );
    assert!(
        expand_primitive(
            Some(Operator::LessThanOrEqual),
            parse_partial("1.9007199254740991").unwrap()
        )
        .is_err()
    );

    assert!(
        expand_hyphen(
            parse_partial("1.0.0").unwrap(),
            parse_partial("9007199254740991").unwrap()
        )
        .is_err()
    );
    assert!(
        expand_hyphen(
            parse_partial("1.0.0").unwrap(),
            parse_partial("1.9007199254740991").unwrap()
        )
        .is_err()
    );
    assert!(parse_partial("1.bad").is_err());
    assert!(parse_partial("1.bad-alpha").is_err());
    assert!(parse_partial("1.bad.3-alpha").is_err());
    assert!(parse_partial("1-alpha").is_err());
    assert!(parse_partial("bad-alpha").is_err());
    assert!(parse_partial("-alpha").is_err());
    assert!(parse_partial("1.-alpha").is_err());
    assert!(parse_partial("1..2-alpha").is_err());
    assert!(parse_partial("1.2.3.4-alpha").is_err());
    assert!(parse_partial("1.2-rc.0").is_err());
    assert!(parse_partial("2.x-rc.0").is_err());
    assert!(parse_partial("1.2.3+").is_err());
    assert!(parse_partial("10000000000000000").is_err());
    assert_eq!(parse_range("^1.0.0").unwrap().set.len(), 1);
    assert_eq!(parse_range("1.0.0 || 2.0.0").unwrap().set.len(), 2);
    assert_eq!(parse_range("1.0.0 || 2.0.0 || 3.0.0").unwrap().set.len(), 3);
    assert!(parse_range(">= || 1.0.0").is_err());
    assert!(parse_range("1.0.0 || >=").is_err());
    assert!(parse_comparator_set(">= ", true).is_err());
    let mut long_bounded_range = "1.0.0 || ".repeat(29);
    long_bounded_range.push_str("1.0.0");
    assert!(parse_range(&long_bounded_range).is_err());
    assert_eq!(try_hyphen("1.0.0 - 2.0.0").unwrap().unwrap().len(), 2);
    assert!(try_hyphen("1.0.0 - 9007199254740991").is_err());
}

#[test]
fn oversized_comparator_sets_skip_quadratic_normalization() {
    let input = ">=1.0.0 ".repeat(20_000);
    assert!(parse_range(&input).is_err());
}

#[test]
fn public_and_comparator_helpers_are_used_in_crate_tests() {
    let version = Version::parse("1.2.3").unwrap();
    let prerelease = Version::parse("1.2.3-alpha.1").unwrap();

    let eq = Comparator {
        op: Operator::Equal,
        version: version.clone(),
    };
    let lt = Comparator {
        op: Operator::LessThan,
        version: Version::parse("2.0.0").unwrap(),
    };
    let set = ComparatorSet {
        comparators: vec![eq.clone(), lt.clone()],
    };
    let range = Range::parse("1.2.3").unwrap();

    assert!(eq.test(&version));
    assert!(
        Comparator {
            op: Operator::GreaterThan,
            version: Version::parse("1.2.2").unwrap(),
        }
        .test(&version)
    );
    assert!(
        Comparator {
            op: Operator::LessThanOrEqual,
            version: version.clone(),
        }
        .test(&version)
    );
    assert_eq!(eq.to_string(), "1.2.3");
    assert!(set.test(&version));
    assert!(!set.test(&prerelease));
    assert_eq!(
        Range::parse("^1.2.3").unwrap().to_string(),
        ">=1.2.3 <2.0.0-0"
    );
    assert!(range.satisfies(&version));
    assert!(!Range::parse("2.x || 3.x").unwrap().satisfies(&version));
    assert_eq!(
        compare_core_and_prerelease(&version, &Version::parse("1.2.4").unwrap()),
        core::cmp::Ordering::Less
    );
}

#[test]
fn comparator_set_test_covers_release_and_prerelease_paths() {
    let release = Version::parse("1.2.3").unwrap();
    let prerelease = Version::parse("1.2.3-alpha.1").unwrap();
    let next_release = Version::parse("1.2.4").unwrap();
    let matching_pre = Version::parse("1.2.3-alpha.0").unwrap();

    let empty = ComparatorSet {
        comparators: Vec::new(),
    };
    assert!(empty.test(&release));
    assert!(!empty.test(&prerelease));

    let release_ok = ComparatorSet {
        comparators: vec![Comparator {
            op: Operator::Equal,
            version: release.clone(),
        }],
    };
    assert!(release_ok.test(&release));
    assert!(!release_ok.test(&next_release));

    let prerelease_without_match = ComparatorSet {
        comparators: vec![Comparator {
            op: Operator::GreaterThanOrEqual,
            version: release.clone(),
        }],
    };
    assert!(!prerelease_without_match.test(&prerelease));

    let prerelease_passes_but_tuple_does_not_match = ComparatorSet {
        comparators: vec![
            Comparator {
                op: Operator::GreaterThan,
                version: Version::parse("1.0.0").unwrap(),
            },
            Comparator {
                op: Operator::LessThanOrEqual,
                version: Version::parse("2.0.0").unwrap(),
            },
        ],
    };
    assert!(!prerelease_passes_but_tuple_does_not_match.test(&prerelease));

    let prerelease_with_match = ComparatorSet {
        comparators: vec![Comparator {
            op: Operator::GreaterThanOrEqual,
            version: matching_pre,
        }],
    };
    assert!(prerelease_with_match.test(&prerelease));
}

#[test]
fn compare_core_and_prerelease_covers_all_major_paths() {
    assert_eq!(
        compare_core_and_prerelease(
            &Version::parse("2.0.0").unwrap(),
            &Version::parse("1.9.9").unwrap()
        ),
        core::cmp::Ordering::Greater
    );
    assert_eq!(
        compare_core_and_prerelease(
            &Version::parse("1.3.0").unwrap(),
            &Version::parse("1.2.9").unwrap()
        ),
        core::cmp::Ordering::Greater
    );
    assert_eq!(
        compare_core_and_prerelease(
            &Version::parse("1.2.4").unwrap(),
            &Version::parse("1.2.3").unwrap()
        ),
        core::cmp::Ordering::Greater
    );
    assert_eq!(
        compare_core_and_prerelease(
            &Version::parse("1.2.3").unwrap(),
            &Version::parse("1.2.3-alpha.1").unwrap()
        ),
        core::cmp::Ordering::Greater
    );
    assert_eq!(
        compare_core_and_prerelease(
            &Version::parse("1.2.3-alpha.1").unwrap(),
            &Version::parse("1.2.3-alpha.2").unwrap()
        ),
        core::cmp::Ordering::Less
    );
    assert_eq!(
        compare_core_and_prerelease(
            &Version::parse("1.2.3-alpha.1").unwrap(),
            &Version::parse("1.2.3-alpha.1").unwrap()
        ),
        core::cmp::Ordering::Equal
    );
}
