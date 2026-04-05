#![allow(missing_docs, reason = "integration test crate")]
#![allow(
    clippy::tests_outside_test_module,
    reason = "integration tests are the test module"
)]
#![allow(clippy::unwrap_used, reason = "test fixtures use unwrap for brevity")]
#![allow(
    clippy::assertions_on_result_states,
    reason = "result state assertions are explicit in tests"
)]

use core::cmp::Ordering;
use core::fmt::{self, Write as _};

use js_semver::Version;

fn v(s: &str) -> Version {
    s.parse().unwrap()
}

struct FailingWriter {
    fail_on: &'static str,
    fail_any: bool,
}

impl fmt::Write for FailingWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.fail_any || (!self.fail_on.is_empty() && s.contains(self.fail_on)) {
            return Err(fmt::Error);
        }
        Ok(())
    }
}

#[test]
fn parse_valid_and_display_cases() {
    let basic = v("1.2.3");
    assert_eq!((basic.major, basic.minor, basic.patch), (1, 2, 3));
    assert!(basic.pre_release.is_empty());
    assert!(basic.build.is_empty());

    let with_pre = v("1.2.3-alpha.1");
    assert_eq!(with_pre.pre_release.to_string(), "alpha.1");

    let with_build = v("1.2.3+build.42");
    assert_eq!(with_build.build.to_string(), "build.42");

    let cases = [
        ("1.2.3", "1.2.3"),
        ("1.2.3-alpha.1", "1.2.3-alpha.1"),
        ("1.2.3+build.42", "1.2.3+build.42"),
        ("1.2.3-alpha.1+build", "1.2.3-alpha.1+build"),
        ("v1.2.3", "1.2.3"),
        ("1.2.3--pre", "1.2.3--pre"),
        ("1.2.3-a+b", "1.2.3-a+b"),
        ("0.0.0", "0.0.0"),
        ("9007199254740991.0.0", "9007199254740991.0.0"),
        ("1.2.3-9007199254740992", "1.2.3-9007199254740992"),
        ("1.0.0-9007199254740992", "1.0.0-9007199254740992"),
        ("1.0.0-18446744073709551616", "1.0.0-18446744073709551616"),
    ];

    for (input, expected) in cases {
        assert_eq!(input.parse::<Version>().unwrap().to_string(), expected);
    }
}

#[test]
fn build_is_ignored_in_eq_and_ord() {
    assert_eq!(v("1.2.3+a"), v("1.2.3+b"));
    assert_eq!(v("1.2.3+a").cmp(&v("1.2.3+b")), Ordering::Equal);
    assert_eq!(v("1.2.3+9").cmp(&v("1.2.3+a")), Ordering::Equal);
    assert_eq!(
        v("1.2.3+demo.90").cmp(&v("1.2.3+demo.090")),
        Ordering::Equal
    );
}

#[test]
fn cmp_build_uses_build_metadata_as_tiebreaker() {
    assert_eq!(v("1.2.3+a").cmp_build(&v("1.2.3+b")), Ordering::Less);
    assert_eq!(
        v("1.2.3-alpha+meta.1").cmp_build(&v("1.2.3-alpha+meta.2")),
        Ordering::Less
    );
}

#[test]
fn cmp_versions() {
    assert!(v("1.0.0") < v("2.0.0"));
    assert!(v("2.0.0") > v("1.0.0"));
    assert_eq!(v("1.0.0"), v("1.0.0"));
}

#[test]
fn partial_ord_matches_total_order() {
    assert_eq!(v("1.0.0").partial_cmp(&v("2.0.0")), Some(Ordering::Less));
    assert_eq!(
        v("1.2.3+build.1").partial_cmp(&v("1.2.3+build.2")),
        Some(Ordering::Equal)
    );
    assert_eq!(
        v("1.2.3-alpha").partial_cmp(&v("1.2.3-alpha")),
        Some(Ordering::Equal)
    );
}

#[test]
fn pre_lower_than_release() {
    assert!(v("1.0.0-alpha") < v("1.0.0"));
    assert!(v("1.0.0-beta") > v("1.0.0-alpha"));
    assert!(v("1.0.0-1") < v("1.0.0-alpha"));
}

#[test]
fn comparators_gt_gte_lt_lte_eq_neq() {
    assert!(v("2.0.0") > v("1.0.0"));
    assert!(v("1.0.0") <= v("2.0.0"));
    assert!(v("1.0.0") >= v("1.0.0"));
    assert!(v("1.0.0") < v("2.0.0"));
    assert!(v("1.0.0") <= v("1.0.0"));
    assert_eq!(v("1.0.0"), v("1.0.0"));
    assert_ne!(v("1.0.0"), v("2.0.0"));
}

#[test]
fn sort_versions() {
    let mut vs: Vec<Version> = ["3.0.0", "1.0.0", "2.0.0"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
    vs.sort();
    assert_eq!(vs, [v("1.0.0"), v("2.0.0"), v("3.0.0")]);

    let mut with_build: Vec<Version> = ["3.0.0", "1.0.0", "2.0.0", "2.0.0+demo.9", "2.0.0+demo.10"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
    with_build.sort_by(|a, b| b.cmp_build(a));
    assert_eq!(
        with_build,
        [
            v("3.0.0"),
            v("2.0.0+demo.10"),
            v("2.0.0+demo.9"),
            v("2.0.0"),
            v("1.0.0"),
        ]
    );
}

#[test]
fn prerelease_field() {
    assert!(v("1.2.3").pre_release.is_empty());
    assert!(!v("1.2.3-alpha.1").pre_release.is_empty());
    assert_eq!(v("1.2.3-alpha.1").pre_release.to_string(), "alpha.1");
}

#[test]
fn release_greater_than_prerelease() {
    assert_eq!(v("1.0.0").cmp(&v("1.0.0-alpha")), Ordering::Greater);
}

#[test]
fn semver_error_display() {
    let err = "bad".parse::<Version>().unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[test]
fn version_display_propagates_formatter_errors() {
    let mut core_writer = FailingWriter {
        fail_on: "",
        fail_any: true,
    };
    assert!(write!(&mut core_writer, "{}", v("1.2.3")).is_err());

    let mut pre_writer = FailingWriter {
        fail_on: "-",
        fail_any: false,
    };
    assert!(write!(&mut pre_writer, "{}", v("1.2.3-alpha.1")).is_err());

    let mut build_writer = FailingWriter {
        fail_on: "+",
        fail_any: false,
    };
    assert!(write!(&mut build_writer, "{}", v("1.2.3+build.1")).is_err());
}

#[test]
fn parse_invalid_cases() {
    let cases = [
        "1.2.3".repeat(60),
        "1".into(),
        "1.2".into(),
        "V1.2.3".into(),
        "v 1.2.3".into(),
        "01.2.3".into(),
        "1.02.3".into(),
        "18446744073709551616.0.0".into(),
        "1.2.3-".into(),
        "1.2.3+".into(),
        "1.2.3+a..b".into(),
        "1.2.3+a!b".into(),
        "1.2.3 extra".into(),
        "1.2.3-.0".into(),
        "1.2.3-a!b".into(),
        "1.2.3-01".into(),
        String::new(),
        "   ".into(),
        "1.".into(),
        "1.2.".into(),
        "1..2.3".into(),
        "1.2.3.4".into(),
        "..1".into(),
        ".1.2".into(),
        "-1.2.3".into(),
        "1.2.3-pre..rel".into(),
        "1.2.3-0.1.".into(),
        "1.2.3-0.01".into(),
        "1.2.3-01.0".into(),
        "1.2.3-.".into(),
        "1.2.3++".into(),
        "1.2.3+.".into(),
        "1.9007199254740992.0".into(),
        "1.0.9007199254740992".into(),
        "1.2.03".into(),
        "1.00.3".into(),
        "1.2.3\t4".into(),
        "1.2.3/".into(),
        "1.2.3#1".into(),
        "a.b.c".into(),
        "abc".into(),
        "+1.2.3".into(),
        "1.+2.3".into(),
        "1.2.+3".into(),
        "1 .2.3".into(),
        "1. 2.3".into(),
        "1.2. 3".into(),
        "1.2.3- alpha".into(),
        "1.2.3 -alpha".into(),
        "00.0.0".into(),
        "0.00.0".into(),
        "0.0.00".into(),
        "1.2.3-00".into(),
        "1.2.3-0.00".into(),
        "1.2.3+a b".into(),
        "1.2.3+a/b".into(),
        "1.2.3+a.b.".into(),
        "1.2.3+.a.b".into(),
        "\u{ff11}.0.0".into(),
        "1.\u{ff12}.0".into(),
        "V1.2.3".into(),
        "bad".into(),
    ];

    for input in cases {
        assert!(input.parse::<Version>().is_err());
        assert!(Version::parse(&input).is_err());
    }
}

#[test]
fn max_safe_integer_core_components() {
    assert!("9007199254740991.0.0".parse::<Version>().is_ok());
    assert!("9007199254740992.0.0".parse::<Version>().is_err());
    assert!("1.9007199254740991.0".parse::<Version>().is_ok());
    assert!("1.0.9007199254740991".parse::<Version>().is_ok());
    assert!("1.9007199254740992.0".parse::<Version>().is_err());
    assert!("1.0.9007199254740992".parse::<Version>().is_err());
}
