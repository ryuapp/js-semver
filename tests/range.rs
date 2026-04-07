// Includes tests adapted from node-semver. See NOTICE.md for details.

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
#![allow(
    clippy::missing_assert_message,
    reason = "assert messages would be repetitive in table-style tests"
)]

use core::fmt::{self, Write as _};

use js_semver::{Range, Version};

fn v(s: &str) -> Version {
    s.parse().unwrap()
}

fn r(s: &str) -> Range {
    s.parse().unwrap()
}

fn assert_satisfies_case(range: &str, version: &str, expected: bool) {
    assert_eq!(r(range).satisfies(&v(version)), expected);
}

fn assert_display_case(input: &str, expected: &str) {
    assert_eq!(Range::parse(input).unwrap().to_string(), expected);
}

fn assert_invalid_range(input: &str) {
    assert!(Range::parse(input).is_err());
}

fn assert_non_matching_invalid_version(range: &str, version: &str) {
    let range = r(range);
    let version = Version::parse(version);
    assert!(version.is_err());
    assert!(!version.is_ok_and(|v| range.satisfies(&v)));
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
fn satisfies_cases() {
    assert_satisfies_inclusion_cases();
    assert_satisfies_additional_positive_cases();
    assert_satisfies_negative_cases();
    assert_non_matching_invalid_versions();
}

fn assert_satisfies_inclusion_cases() {
    assert_satisfies_case("1.2.3-pre+asdf - 2.4.3-pre+asdf", "1.2.3", true);
    assert_satisfies_case("1.2.3-pre+asdf - 2.4.3-pre+asdf", "1.2.3-pre.2", true);
    assert_satisfies_case("1.2.3-pre+asdf - 2.4.3-pre+asdf", "2.4.3-alpha", true);
    assert_satisfies_case("1.2.3+asdf - 2.4.3+asdf", "1.2.3", true);
    assert_satisfies_case("0.1.20 || 1.2.4", "1.2.4", true);
    assert_satisfies_case(">=0.2.3 || <0.0.1", "0.0.0", true);
    assert_satisfies_case(">=0.2.3 || <0.0.1", "0.2.3", true);
    assert_satisfies_case(">=0.2.3 || <0.0.1", "0.2.4", true);
    assert_satisfies_case("^1.0.0", "1.2.3", true);
    assert_satisfies_case("^1.0.0", "1.9.9", true);
    assert_satisfies_case("^1.0.0", "2.0.0", false);
    assert_satisfies_case("^1.0.0", "0.9.9", false);
    assert_satisfies_case("~1.2.0", "1.2.3", true);
    assert_satisfies_case("~1.2.0", "1.2.9", true);
    assert_satisfies_case("~1.2.0", "1.3.0", false);
    assert_satisfies_case("~1.2.0", "1.1.9", false);
    assert_satisfies_case("1.0.0 - 2.0.0", "1.5.0", true);
    assert_satisfies_case("1.0.0 - 2.0.0", "1.0.0", true);
    assert_satisfies_case("1.0.0 - 2.0.0", "2.0.0", true);
    assert_satisfies_case("1.0.0 - 2.0.0", "3.0.0", false);
    assert_satisfies_case(">1.0.0", "2.0.0", true);
    assert_satisfies_case(">=1.0.0", "1.0.0", true);
    assert_satisfies_case("<1.0.0", "0.9.9", true);
    assert_satisfies_case("<=1.0.0", "1.0.0", true);
    assert_satisfies_case("=1.0.0", "1.0.0", true);
    assert_satisfies_case("1.0.0", "1.0.0", true);
    assert_satisfies_case("1.x", "1.2.3", true);
    assert_satisfies_case("1", "1.0.0", true);
    assert_satisfies_case("1", "1.9.9", true);
    assert_satisfies_case("1", "2.0.0", false);
    assert_satisfies_case("1.2.x", "1.2.9", true);
    assert_satisfies_case("*", "1.2.3", true);
    assert_satisfies_case("*", "0.0.1", true);
    assert_satisfies_case("1.0.0 || 2.0.0", "1.0.0", true);
    assert_satisfies_case("1.0.0 || 2.0.0", "2.0.0", true);
    assert_satisfies_case("1.0.0 || 2.0.0", "3.0.0", false);
    assert_satisfies_case("2.x.x", "2.1.3", true);
    assert_satisfies_case("1.2.x || 2.x", "2.1.3", true);
    assert_satisfies_case("1.2.x || 2.x", "1.2.3", true);
    assert_satisfies_case("2.*.*", "2.1.3", true);
    assert_satisfies_case("1.2.* || 2.*", "2.1.3", true);
    assert_satisfies_case("1.2.* || 2.*", "1.2.3", true);
    assert_satisfies_case("2", "2.1.2", true);
    assert_satisfies_case("2.3", "2.3.1", true);
    assert_satisfies_case("||", "1.3.4", true);
    assert_satisfies_case("~2.4", "2.4.5", true);
    assert_satisfies_case("~2.4", "2.5.0", false);
    assert_satisfies_case("~>3.2.1", "3.2.2", true);
    assert_satisfies_case("~0.0.1", "0.0.1", true);
    assert_satisfies_case("~0.0.1", "0.0.2", true);
    assert_satisfies_case("~0.0.1", "0.1.0", false);
    assert_satisfies_case("~x", "0.0.9", true);
    assert_satisfies_case("~2", "2.0.9", true);
    assert_satisfies_case("~2", "3.0.0", false);
    assert_satisfies_case("~ 1.0.3", "1.0.12", true);
    assert_satisfies_case("~v0.5.4-pre", "0.5.5", true);
    assert_satisfies_case("~v0.5.4-pre", "0.5.4", true);
    assert_satisfies_case("=0.7.x", "0.7.2", true);
    assert_satisfies_case(">=0.7.x", "0.7.2", true);
    assert_satisfies_case(">=1.2", "1.2.8", true);
    assert_satisfies_case("~1.2.1 >=1.2.3", "1.2.3", true);
    assert_satisfies_case("~1.2.1 =1.2.3", "1.2.3", true);
    assert_satisfies_case("~1.2.1 1.2.3", "1.2.3", true);
    assert_satisfies_case("~1.2.1 >=1.2.3 1.2.3", "1.2.3", true);
    assert_satisfies_case("~1.2.1 1.2.3 >=1.2.3", "1.2.3", true);
    assert_satisfies_case(">=1.2.1 1.2.3", "1.2.3", true);
    assert_satisfies_case("1.2.3 >=1.2.1", "1.2.3", true);
    assert_satisfies_case(">=1.2.3 >=1.2.1", "1.2.3", true);
    assert_satisfies_case(">=1.2.1 >=1.2.3", "1.2.3", true);
    assert_satisfies_case("^1.2.3", "1.8.1", true);
    assert_satisfies_case("^0.1.2", "0.1.2", true);
    assert_satisfies_case("^0.1", "0.1.2", true);
    assert_satisfies_case("^1.2", "1.4.2", true);
    assert_satisfies_case("^1.2 ^1", "1.4.2", true);
    assert_satisfies_case("^0.0.1", "0.0.1", true);
    assert_satisfies_case("^1.2.3-alpha", "1.2.3-pre", true);
    assert_satisfies_case("^1.2.0-alpha", "1.2.0-pre", true);
    assert_satisfies_case("^0.0.1-alpha", "0.0.1-beta", true);
    assert_satisfies_case("^0.0.1-alpha", "0.0.1", true);
    assert_satisfies_case("^0.1.1-alpha", "0.1.1-beta", true);
    assert_satisfies_case("^x", "1.2.3", true);
}

fn assert_satisfies_additional_positive_cases() {
    assert_satisfies_case("^0.0.1", "0.0.2", false);
    assert_satisfies_case("^1.2", "1.1.9", false);
    assert_satisfies_case("x - 1.0.0", "0.9.7", true);
    assert_satisfies_case("x - 1.x", "0.9.7", true);
    assert_satisfies_case("1.0.0 - x", "1.9.7", true);
    assert_satisfies_case("1.x - x", "1.9.7", true);
    assert_satisfies_case("<=7.x", "7.9.9", true);
    assert_satisfies_case("<=7.x", "8.0.0", false);
    assert_satisfies_case(">1", "2.0.0", true);
    assert_satisfies_case(">1", "1.0.0", false);
    assert_satisfies_case("1.0.0", "1.0.1", false);
    assert_satisfies_case(">=1.0.0", "0.0.0", false);
    assert_satisfies_case(">=1.0.0", "0.0.1", false);
    assert_satisfies_case(">=1.0.0", "0.1.0", false);
    assert_satisfies_case(">1.0.0", "0.0.1", false);
    assert_satisfies_case(">1.0.0", "0.1.0", false);
    assert_satisfies_case("<=2.0.0", "3.0.0", false);
    assert_satisfies_case("<2.0.0", "2.2.9", false);
}

fn assert_satisfies_negative_cases() {
    assert_satisfies_case("0.1.20 || 1.2.4", "1.2.3", false);
    assert_satisfies_case(">=0.2.3 || <0.0.1", "0.0.3", false);
    assert_satisfies_case(">=0.2.3 || <0.0.1", "0.2.2", false);
    assert_satisfies_case("2.x.x", "1.1.3", false);
    assert_satisfies_case("2.x.x", "3.1.3", false);
    assert_satisfies_case("1.2.x", "1.3.3", false);
    assert_satisfies_case("2.*.*", "1.1.3", false);
    assert_satisfies_case("2.*.*", "3.1.3", false);
    assert_satisfies_case("2", "1.1.2", false);
    assert_satisfies_case("2.3", "2.4.1", false);
    assert_satisfies_case("~2.4", "2.3.9", false);
    assert_satisfies_case("~>3.2.1", "3.3.2", false);
    assert_satisfies_case("~>3.2.1", "3.2.0", false);
    assert_satisfies_case("~1", "0.2.3", false);
    assert_satisfies_case("~>1", "2.2.3", false);
    assert_satisfies_case("~1.0", "1.1.0", false);
    assert_satisfies_case("<1", "1.0.0", false);
    assert_satisfies_case(">=1.2", "1.1.1", false);
    assert_satisfies_case("~v0.5.4-beta", "0.5.4-alpha", false);
    assert_satisfies_case("=0.7.x", "0.8.2", false);
    assert_satisfies_case(">=0.7.x", "0.6.2", false);
    assert_satisfies_case("<0.7.x", "0.7.2", false);
    assert_satisfies_case(">1.2", "1.2.8", false);
    assert_satisfies_case("^1.2.3", "1.2.2", false);
    assert_satisfies_case("^1.2.3+build", "2.0.0", false);
    assert_satisfies_case("^1.2.3+build", "1.2.0", false);
    assert_satisfies_case("1.2.3+asdf - 2.4.3+asdf", "1.2.3-pre.2", false);
    assert_satisfies_case("1.2.3+asdf - 2.4.3+asdf", "2.4.3-alpha", false);
    assert_satisfies_case("^1.2.3", "1.2.3-pre", false);
    assert_satisfies_case("^1.2", "1.2.0-pre", false);
    assert_satisfies_case(">1.2", "1.3.0-beta", false);
    assert_satisfies_case("<=1.2.3", "1.2.3-beta", false);
    assert_satisfies_case("^1.2.3", "1.2.3-beta", false);
    assert_satisfies_case("=0.7.x", "0.7.0-asdf", false);
    assert_satisfies_case(">=0.7.x", "0.7.0-asdf", false);
    assert_satisfies_case("<=0.7.x", "0.7.0-asdf", false);
    assert_satisfies_case("<1.2.3", "1.2.3-beta", false);
    assert_satisfies_case("=1.2.3", "1.2.3-beta", false);
    assert_satisfies_case("^0.0.1", "0.0.2-alpha", false);
    assert_satisfies_case("^1.2.3", "2.0.0-alpha", false);
}

fn assert_non_matching_invalid_versions() {
    assert_non_matching_invalid_version("*", "not a version");
    assert_non_matching_invalid_version(">=2", "glorp");
}

#[test]
fn prerelease_restriction() {
    assert!(!r("^1.0.0").satisfies(&v("1.0.0-alpha")));
    assert!(r(">=1.0.0-alpha").satisfies(&v("1.0.0-alpha.1")));
    assert!(r(">=1.0.0-alpha <=1.0.0-rc").satisfies(&v("1.0.0-beta")));
    assert!(!r(">=1.0.0-alpha <2.0.0").satisfies(&v("1.2.3-alpha")));
    assert!(r(">=4.0.0-rc.0").satisfies(&v("4.0.0-rc.0")));
    assert!(r(">=4.0.0-rc.0").satisfies(&v("4.0.0-rc.2")));
    assert!(!r(">=4.0.0-rc.0").satisfies(&v("4.2.0-rc.1")));
}

#[test]
fn parse_valid_and_display_cases() {
    assert_display_case("0.1.20 || 1.2.4", "0.1.20||1.2.4");
    assert_display_case(">=0.2.3 || <0.0.1", ">=0.2.3||<0.0.1");
    assert_display_case("||", "*");
    assert_display_case("2.x.x", ">=2.0.0 <3.0.0-0");
    assert_display_case("1.2.x", ">=1.2.0 <1.3.0-0");
    assert_display_case("1 - 2", ">=1.0.0 <3.0.0-0");
    assert_display_case("1.0 - 2.0", ">=1.0.0 <2.1.0-0");
    assert_display_case("^1.0.0", ">=1.0.0 <2.0.0-0");
    assert_display_case("1.0.0", "1.0.0");
    assert_display_case("=1.0.0", "1.0.0");
    assert_display_case("~0.x.0", "<1.0.0-0");
    assert_display_case("~1.x.0", ">=1.0.0 <2.0.0-0");
    assert_display_case("*", "*");
    assert_display_case("* || ^1.2.3", "*");
    assert_display_case(">X", "<0.0.0-0");
    assert_display_case("<X", "<0.0.0-0");
    assert_display_case("<x <* || >* 2.x", "<0.0.0-0");
    assert_display_case(
        ">=1.0.0 <2.0.0 || >=2.0.0 <3.0.0",
        ">=1.0.0 <2.0.0||>=2.0.0 <3.0.0",
    );
    assert_display_case("~> 1", ">=1.0.0 <2.0.0-0");
    assert_display_case("~ 1.0", ">=1.0.0 <1.1.0-0");
    assert_display_case("~v0.5.2-pre", ">=0.5.2-pre <0.6.0-0");
    assert_display_case("1.2.x || 2.x", ">=1.2.0 <1.3.0-0||>=2.0.0 <3.0.0-0");
    assert_display_case("2.*.*", ">=2.0.0 <3.0.0-0");
    assert_display_case("1.2.*", ">=1.2.0 <1.3.0-0");
    assert_display_case("1.2.* || 2.*", ">=1.2.0 <1.3.0-0||>=2.0.0 <3.0.0-0");
    assert_display_case("2", ">=2.0.0 <3.0.0-0");
    assert_display_case("2.3", ">=2.3.0 <2.4.0-0");
    assert_display_case("~0.0.1", ">=0.0.1 <0.1.0-0");
    assert_display_case("~x", "*");
    assert_display_case("~2", ">=2.0.0 <3.0.0-0");
    assert_display_case("~>1", ">=1.0.0 <2.0.0-0");
    assert_display_case("~1.0", ">=1.0.0 <1.1.0-0");
    assert_display_case("~ 1.0.3", ">=1.0.3 <1.1.0-0");
    assert_display_case("~2.4", ">=2.4.0 <2.5.0-0");
    assert_display_case("~>3.2.1", ">=3.2.1 <3.3.0-0");
    assert_display_case("^0.1", ">=0.1.0 <0.2.0-0");
    assert_display_case("^1.0", ">=1.0.0 <2.0.0-0");
    assert_display_case("^1.2", ">=1.2.0 <2.0.0-0");
    assert_display_case("^0.0.1", ">=0.0.1 <0.0.2-0");
    assert_display_case("^0.0.1-beta", ">=0.0.1-beta <0.0.2-0");
    assert_display_case("^x", "*");
    assert_display_case("^ 1.2.3", ">=1.2.3 <2.0.0-0");
    assert_display_case("<=1.2.3", "<=1.2.3");
    assert_display_case("<1.2.3", "<1.2.3");
    assert_display_case("<1", "<1.0.0-0");
    assert_display_case(">=1", ">=1.0.0");
    assert_display_case("<1.2", "<1.2.0-0");
    assert_display_case("^1.2 ^1", ">=1.2.0 <2.0.0-0 >=1.0.0");
    assert_display_case("<=7.x", "<8.0.0-0");
    assert_display_case("1.2 - 3.4", ">=1.2.0 <3.5.0-0");
    assert_display_case("1.2.3 - 3.4", ">=1.2.3 <3.5.0-0");
    assert_display_case(">1", ">=2.0.0");
    assert_display_case(">1.2", ">=1.3.0");
    assert_display_case("x", "*");
    assert_display_case("=x", "*");
}

#[test]
fn tilde_partial() {
    assert!(r("~1").satisfies(&v("1.9.9")));
    assert!(!r("~1").satisfies(&v("2.0.0")));
    assert_eq!(Range::parse("~0.x.0").unwrap().to_string(), "<1.0.0-0");
    assert_eq!(
        Range::parse("~1.x.0").unwrap().to_string(),
        ">=1.0.0 <2.0.0-0"
    );
    assert!(r("~1.2").satisfies(&v("1.2.9")));
    assert!(!r("~1.2").satisfies(&v("1.3.0")));
    assert!(r("~1.2.3-alpha").satisfies(&v("1.2.3-beta")));
    assert!(!r("~1.2.3-alpha").satisfies(&v("1.3.0")));
}

#[test]
fn caret_partial() {
    assert_eq!(Range::parse("^0").unwrap().to_string(), "<1.0.0-0");
    assert!(r("^1").satisfies(&v("1.9.9")));
    assert!(!r("^1").satisfies(&v("2.0.0")));
    assert!(r("^0.2").satisfies(&v("0.2.9")));
    assert!(!r("^0.2").satisfies(&v("0.3.0")));
    assert!(r("^0.0").satisfies(&v("0.0.9")));
    assert!(!r("^0.0").satisfies(&v("0.1.0")));
    assert!(r("^0.2.3").satisfies(&v("0.2.9")));
    assert!(!r("^0.2.3").satisfies(&v("0.3.0")));
    assert!(r("^0.0.3").satisfies(&v("0.0.3")));
    assert!(!r("^0.0.3").satisfies(&v("0.0.4")));
    assert!(r("^1.2.3-alpha").satisfies(&v("1.2.3-beta")));
}

#[test]
fn primitive_partial() {
    assert!(r(">1").satisfies(&v("2.0.0")));
    assert!(!r(">1").satisfies(&v("1.9.9")));
    assert!(r(">1.2").satisfies(&v("1.3.0")));
    assert!(!r(">1.2").satisfies(&v("1.2.9")));
    assert!(r(">=1.2").satisfies(&v("1.2.0")));
    assert!(r("<1").satisfies(&v("0.9.9")));
    assert!(!r("<1").satisfies(&v("1.0.0")));
    assert!(r("<1.2").satisfies(&v("1.1.9")));
    assert!(r("<=1.2").satisfies(&v("1.2.9")));
    assert!(!r("<=1.2").satisfies(&v("1.3.0")));
    assert!(r("<=0.7.x").satisfies(&v("0.7.2")));
    assert!(r("<=0.7.x").satisfies(&v("0.6.2")));
    assert!(!r("<0.7.x").satisfies(&v("0.7.2")));
}

#[test]
fn wildcard_operator_forms() {
    assert!(r("~*").satisfies(&v("1.0.0")));
    assert!(!r("~*").satisfies(&v("1.0.0-alpha")));
    assert!(r("^*").satisfies(&v("1.0.0")));
    assert!(!r("^*").satisfies(&v("1.0.0-alpha")));
    assert!(r(">=*").satisfies(&v("1.0.0")));
    assert!(!r(">=*").satisfies(&v("1.0.0-alpha")));
    assert!(r("<=*").satisfies(&v("99.0.0")));
    assert!(!r("<=*").satisfies(&v("1.0.0-alpha")));
    assert!(!r("*").satisfies(&v("1.0.0-alpha")));
    assert!(!r("<*").satisfies(&v("0.0.0")));
}

#[test]
fn range_display_propagates_formatter_errors() {
    let mut wildcard_writer = FailingWriter {
        fail_on: "*",
        fail_any: false,
    };
    assert!(write!(&mut wildcard_writer, "{}", r("*")).is_err());

    let mut or_writer = FailingWriter {
        fail_on: "||",
        fail_any: false,
    };
    assert!(write!(&mut or_writer, "{}", r("1.0.0 || >=2.0.0")).is_err());

    let mut space_writer = FailingWriter {
        fail_on: " ",
        fail_any: false,
    };
    assert!(write!(&mut space_writer, "{}", r(">=1.0.0 <2.0.0")).is_err());

    let mut comparator_writer = FailingWriter {
        fail_on: "",
        fail_any: true,
    };
    assert!(write!(&mut comparator_writer, "{}", r(">=1.0.0")).is_err());
}

#[test]
fn caret_major_minor() {
    assert!(r("^1.2").satisfies(&v("1.9.9")));
    assert!(!r("^1.2").satisfies(&v("2.0.0")));
}

#[test]
fn primitive_single_major() {
    assert!(r(">=1").satisfies(&v("1.0.0")));
    assert!(!r(">=1").satisfies(&v("0.9.9")));
    assert!(r("<=1").satisfies(&v("1.9.9")));
    assert!(!r("<=1").satisfies(&v("2.0.0")));
}

#[test]
fn eq_with_pre() {
    assert!(r("=1.2.3-alpha").satisfies(&v("1.2.3-alpha")));
    assert!(!r("=1.2.3-alpha").satisfies(&v("1.2.3-beta")));
}

#[test]
fn lt_gt_with_pre() {
    assert!(r("<1.2.3-beta").satisfies(&v("1.2.3-alpha")));
    assert!(!r("<1.2.3-beta").satisfies(&v("1.2.3-beta")));
    assert!(r(">1.2.3-alpha").satisfies(&v("1.2.3-beta")));
}

#[test]
fn hyphen_partial_upper() {
    assert!(r("1.0.0 - *").satisfies(&v("99.0.0")));
    assert!(r("1.0.0 - 2").satisfies(&v("2.9.9")));
    assert!(!r("1.0.0 - 2").satisfies(&v("3.0.0")));
    assert!(r("1.0.0 - 2.5").satisfies(&v("2.5.9")));
    assert!(!r("1.0.0 - 2.5").satisfies(&v("2.6.0")));
    assert!(r("1.0.0 - 2.0.0-alpha").satisfies(&v("2.0.0-alpha")));
    assert!(!r("1.0.0 - 2.0.0-alpha").satisfies(&v("2.0.0")));
}

#[test]
fn range_too_long() {
    assert!(Range::parse(&"^1.0.0 ".repeat(50)).is_err());
}

#[test]
fn parse_token_star_mixed() {
    assert!(r(">=1.0.0 *").satisfies(&v("1.0.0")));
    assert!(!r(">=1.0.0 *").satisfies(&v("0.9.9")));
}

#[test]
fn parse_invalid_cases() {
    assert_invalid_range("01.0.0");
    assert_invalid_range("1a.0.0");
    assert_invalid_range("9007199254740992.0.0");
    assert_invalid_range(">");
    assert_invalid_range(">=");
    assert_invalid_range("> ");
    assert_invalid_range("<");
    assert_invalid_range("<=");
    assert_invalid_range("=");
    assert_invalid_range("^");
    assert_invalid_range("~");
    assert_invalid_range("~=");
    assert_invalid_range("1.0.0 -");
    assert_invalid_range("- 2.0.0");
    assert_invalid_range("1.0.0 - 2.0.0 - 3.0.0");
    assert_invalid_range(">>1.0.0");
    assert_invalid_range("><1.0.0");
    assert_invalid_range(">=<=1.0.0");
    assert_invalid_range("^01.0.0");
    assert_invalid_range("~01.0.0");
    assert_invalid_range(">01.0.0");
    assert_invalid_range(">=01.0.0");
    assert_invalid_range("^1.2.3.4");
    assert_invalid_range(">=a.b.c");
    assert_invalid_range(">1.2.3-0.01");
    assert_invalid_range("!!");
    assert_invalid_range("??");
    assert_invalid_range("1.0.0!");
    assert_invalid_range("1.0.0-");
    assert_invalid_range("-1.0.0");
    assert_invalid_range("^1.0.0-0.01");
    assert_invalid_range(">=1.0.0-01");
    assert_invalid_range("~1.0.0-01");
    assert_invalid_range("1.0.0>");
    assert_invalid_range("1.0.0>=");
    assert_invalid_range("1.0.0^");
    assert_invalid_range("~1.");
    assert_invalid_range("^1.");
    assert_invalid_range("^1.2.");
    assert_invalid_range("~1.2.");
    assert_invalid_range(">=1.");
    assert_invalid_range(">=1.2.");
    assert_invalid_range("1. - 2.0.0");
    assert_invalid_range("1.0.0 - 2.");
    assert_invalid_range("1.0.0- 2.0.0");
    assert_invalid_range("1.0.0 -2.0.0");
    assert_invalid_range("!1.0.0");
    assert_invalid_range("!=1.0.0");
    assert_invalid_range(">1.0.0\x00");
    assert_invalid_range("^00.0.0");
    assert_invalid_range("~0.00.0");
    assert_invalid_range(">=0.0.00");
    assert_invalid_range("^9007199254740991.0.0");
}

#[test]
fn invalid_partial_after_operator_errors() {
    assert!(Range::parse("<=a.b.c").is_err());
    assert!(Range::parse("<a.b.c").is_err());
    assert!(Range::parse("=a.b.c").is_err());
    assert!(Range::parse("> a.b.c").is_err());
    assert!(Range::parse("^1.0.0 || >").is_err());
}

#[test]
fn prerelease_zero_upper_bound_excludes_next_tuple_prereleases() {
    let range = r("^1.2.3");
    assert_eq!(range.to_string(), ">=1.2.3 <2.0.0-0");
    assert!(range.satisfies(&v("1.9.9")));
    assert!(!range.satisfies(&v("2.0.0-0")));
    assert!(!range.satisfies(&v("2.0.0-alpha")));
    assert!(!range.satisfies(&v("2.0.0")));
}

#[test]
fn canonical_comparator_dedup_and_tightening() {
    assert_eq!(r("<1.2.4 <1.2.3").to_string(), "<1.2.4 <1.2.3");
    assert_eq!(r("<=1.2.3 <1.2.3").to_string(), "<1.2.3");
    assert_eq!(r("<1.2.3 <=1.2.3").to_string(), "<1.2.3");
    assert_eq!(r("1.2.3 1.2.3").to_string(), "1.2.3");
}
