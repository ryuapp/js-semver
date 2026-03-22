#![allow(clippy::restriction)]
//! Compatibility tests with semver on npm.

use js_semver::{Range, ReleaseType, Version};

fn v(s: &str) -> Version {
    s.parse().unwrap()
}
fn r(s: &str) -> Range {
    s.parse().unwrap()
}

#[test]
fn std_semver_compat_parse() {
    assert!("v1.2.3".parse::<Version>().is_ok());
    assert!("01.2.3".parse::<Version>().is_err());
    // partial versions are rejected
    assert!("1.2".parse::<Version>().is_err());
    assert!("1".parse::<Version>().is_err());
}

#[test]
fn std_semver_compat_satisfies() {
    assert!(r(">=1.0.0").satisfies(&v("1.5.0")));
    assert!(r("^1.2.3").satisfies(&v("1.2.4")));
    assert!(!r("^1.2.3").satisfies(&v("2.0.0")));
    assert!(r("~1.2.3").satisfies(&v("1.2.9")));
    assert!(!r("~1.2.3").satisfies(&v("1.3.0")));
    assert!(r("1.2.3 - 2.3.4").satisfies(&v("1.5.0")));
    assert!(r("1.x").satisfies(&v("1.5.0")));
    assert!(!r("1.x").satisfies(&v("2.0.0")));
    assert!(r("*").satisfies(&v("1.0.0")));
    assert!(r("1.0.0 || 2.0.0").satisfies(&v("1.0.0")));
    assert!(!r("1.0.0 || 2.0.0").satisfies(&v("3.0.0")));
}

#[test]
fn std_semver_compat_prerelease() {
    assert!(!r("^1.0.0").satisfies(&v("1.0.0-alpha")));
    assert!(r(">=1.0.0-alpha").satisfies(&v("1.0.0-alpha.1")));
    assert!(r(">=4.0.0-rc.0").satisfies(&v("4.0.0-rc.2")));
    assert!(!r(">=4.0.0-rc.0").satisfies(&v("4.2.0-rc.1")));
    // beta.N: N is compared numerically (9 < 18)
    assert!(r(">=8.0.0-beta.18").satisfies(&v("8.0.0-beta.18")));
    assert!(r(">=8.0.0-beta.18").satisfies(&v("8.0.0-beta.19")));
    assert!(!r(">=8.0.0-beta.18").satisfies(&v("8.0.0-beta.9")));
    assert!(r(">=8.0.0-beta.18").satisfies(&v("8.0.0-rc.1")));
    assert!(r(">=8.0.0-beta.18").satisfies(&v("8.0.0")));
    assert!(!r(">=8.0.0-beta.18").satisfies(&v("8.1.0-beta.1")));
}

#[test]
fn std_semver_compat_increment() {
    assert_eq!(
        v("1.2.3")
            .increment(ReleaseType::Major)
            .unwrap()
            .to_string(),
        "2.0.0"
    );
    assert_eq!(
        v("1.2.3")
            .increment(ReleaseType::Minor)
            .unwrap()
            .to_string(),
        "1.3.0"
    );
    assert_eq!(
        v("1.2.3")
            .increment(ReleaseType::Patch)
            .unwrap()
            .to_string(),
        "1.2.4"
    );
    // premajor/preminor/prepatch with no identifier → -0
    assert_eq!(
        v("1.2.3")
            .increment(ReleaseType::PreMajor(None))
            .unwrap()
            .to_string(),
        "2.0.0-0"
    );
    assert_eq!(
        v("1.2.3")
            .increment(ReleaseType::PreMinor(None))
            .unwrap()
            .to_string(),
        "1.3.0-0"
    );
    assert_eq!(
        v("1.2.3")
            .increment(ReleaseType::PrePatch(None))
            .unwrap()
            .to_string(),
        "1.2.4-0"
    );
    assert_eq!(
        v("1.2.3")
            .increment(ReleaseType::PreRelease(None))
            .unwrap()
            .to_string(),
        "1.2.4-0"
    );
    assert_eq!(
        v("1.2.3-alpha.1")
            .increment(ReleaseType::PreRelease(None))
            .unwrap()
            .to_string(),
        "1.2.3-alpha.2"
    );
    assert_eq!(
        v("1.2.3-0")
            .increment(ReleaseType::PreRelease(None))
            .unwrap()
            .to_string(),
        "1.2.3-1"
    );
}

#[test]
fn std_semver_compat_caret_zero() {
    // ^0.0.1 only matches exactly
    assert!(!r("^0.0.1").satisfies(&v("0.0.2")));
    assert!(r("^0.0.1").satisfies(&v("0.0.1")));
    // ^0.1.0 locks the minor version
    assert!(r("^0.1.0").satisfies(&v("0.1.5")));
    assert!(!r("^0.1.0").satisfies(&v("0.2.0")));
    // ~0 matches all 0.x.x
    assert!(r("~0").satisfies(&v("0.5.0")));
}

#[test]
fn std_semver_compat_difference() {
    assert_eq!(v("1.0.0").difference(&v("2.0.0")), Some(ReleaseType::Major));
    assert_eq!(v("1.0.0").difference(&v("1.1.0")), Some(ReleaseType::Minor));
    assert_eq!(v("1.0.0").difference(&v("1.0.1")), Some(ReleaseType::Patch));
    // 1.0.0 vs 1.0.0-alpha → "prerelease"
    assert_eq!(
        v("1.0.0").difference(&v("1.0.0-alpha")),
        Some(ReleaseType::PreRelease(None))
    );
    // alpha vs beta → "prerelease"
    assert_eq!(
        v("1.0.0-alpha").difference(&v("1.0.0-beta")),
        Some(ReleaseType::PreRelease(None))
    );
}

// --- invalid version strings ---

#[test]
fn std_semver_compat_version_invalid() {
    // empty / whitespace
    assert!("".parse::<Version>().is_err());
    assert!("   ".parse::<Version>().is_err());
    // partial (no minor/patch)
    assert!("1".parse::<Version>().is_err());
    assert!("1.2".parse::<Version>().is_err());
    assert!("1.".parse::<Version>().is_err());
    assert!("1.2.".parse::<Version>().is_err());
    // double dot / too many components
    assert!("1..2.3".parse::<Version>().is_err());
    assert!("1.2.3.4".parse::<Version>().is_err());
    // leading zero
    assert!("01.2.3".parse::<Version>().is_err());
    assert!("1.02.3".parse::<Version>().is_err());
    assert!("1.2.03".parse::<Version>().is_err());
    // leading zero in pre-release
    assert!("1.2.3-01".parse::<Version>().is_err());
    assert!("1.2.3-0.01".parse::<Version>().is_err());
    // exceeds MAX_SAFE_INTEGER in major/minor/patch
    assert!("9007199254740992.0.0".parse::<Version>().is_err());
    assert!("1.9007199254740992.0".parse::<Version>().is_err());
    assert!("1.0.9007199254740992".parse::<Version>().is_err());
    // pre-release numeric identifiers are not bounded by MAX_SAFE_INTEGER
    assert!("1.2.3-9007199254740992".parse::<Version>().is_ok());
    // empty pre-release / build
    assert!("1.2.3-".parse::<Version>().is_err());
    assert!("1.2.3+".parse::<Version>().is_err());
    // double dot in pre-release / build
    assert!("1.2.3-a..b".parse::<Version>().is_err());
    assert!("1.2.3+a..b".parse::<Version>().is_err());
    // invalid chars
    assert!("1.2.3-!".parse::<Version>().is_err());
    assert!("1.2.3+!".parse::<Version>().is_err());
    assert!("a.b.c".parse::<Version>().is_err());
    // starts with dot / dash
    assert!("-1.2.3".parse::<Version>().is_err());
    assert!(".1.2.3".parse::<Version>().is_err());
}

// --- invalid range strings ---

#[test]
fn std_semver_compat_range_invalid() {
    // operator with no version (standalone)
    assert!(">".parse::<Range>().is_err());
    assert!(">=".parse::<Range>().is_err());
    assert!("<".parse::<Range>().is_err());
    assert!("<=".parse::<Range>().is_err());
    assert!("=".parse::<Range>().is_err());
    assert!("^".parse::<Range>().is_err());
    assert!("~".parse::<Range>().is_err());
    // incomplete hyphen range
    assert!("1.0.0 -".parse::<Range>().is_err());
    assert!("- 2.0.0".parse::<Range>().is_err());
    // too many hyphen range parts
    assert!("1.0.0 - 2.0.0 - 3.0.0".parse::<Range>().is_err());
    // leading zero
    assert!("^01.0.0".parse::<Range>().is_err());
    assert!("~01.0.0".parse::<Range>().is_err());
    assert!(">01.0.0".parse::<Range>().is_err());
    assert!(">=01.0.0".parse::<Range>().is_err());
    // leading zero in pre-release
    assert!("^1.0.0-0.01".parse::<Range>().is_err());
    assert!(">=1.0.0-01".parse::<Range>().is_err());
    // too many version components
    assert!("^1.2.3.4".parse::<Range>().is_err());
    // garbage
    assert!("!!".parse::<Range>().is_err());
    assert!("1.0.0!".parse::<Range>().is_err());
    // operator after version
    assert!("1.0.0>".parse::<Range>().is_err());
}

// --- valid strings that must be accepted ---

#[test]
fn std_semver_compat_version_valid() {
    // v prefix and whitespace tolerance
    assert!("v1.2.3".parse::<Version>().is_ok());
    assert!("V1.2.3".parse::<Version>().is_err()); // npm/semver: uppercase V rejected
    // double-dash pre-release (valid per semver spec)
    assert!("1.2.3--pre".parse::<Version>().is_ok());
    // pre-release with build metadata
    assert!("1.2.3-a+b".parse::<Version>().is_ok());
    // 0.0.0 is valid
    assert!("0.0.0".parse::<Version>().is_ok());
    // MAX_SAFE_INTEGER itself is valid
    assert!("9007199254740991.0.0".parse::<Version>().is_ok());
}

#[test]
fn std_semver_compat_range_valid() {
    // whitespace around operator
    assert!("  >=1.0.0  ".parse::<Range>().is_ok());
    assert!("^ 1.0.0".parse::<Range>().is_ok());
    assert!("~ 1.0.0".parse::<Range>().is_ok());
    // OR edge cases accepted
    assert!("||".parse::<Range>().is_ok());
    assert!("1.0.0 ||".parse::<Range>().is_ok());
    assert!("|| 1.0.0".parse::<Range>().is_ok());
    // double-dash pre-release in range
    assert!("1.0.0--pre".parse::<Range>().is_ok());
    // wildcard with pre-release
    assert!("*.0.0-alpha".parse::<Range>().is_ok());
    // AND of two exact versions
    assert!("1.0.0 2.0.0".parse::<Range>().is_ok());
    // hyphen range with x-ranges
    assert!("1.x - 2.x".parse::<Range>().is_ok());
    assert!("* - 2.0.0".parse::<Range>().is_ok());
}

#[test]
fn range_metadata_and_wildcards() {
    let cases = [
        (">=1.x+experimental <2.x.x+experimental", ">=1.0.0 <2.0.0-0"),
        (
            "1.x.x+experimental || 2.x.x+experimental",
            ">=1.0.0 <2.0.0-0||>=2.0.0 <3.0.0-0",
        ),
        ("1.x.x+experimental <2.x.x+beta", ">=1.0.0 <2.0.0-0"),
        (
            "^1.x+experimental.123 <2.x.x+pre-release",
            ">=1.0.0 <2.0.0-0",
        ),
        (
            ">=1.x.x-alpha+experimental <2.x.x+experimental",
            ">=1.0.0 <2.0.0-0",
        ),
        (">=0.x.x-alpha <1.x.x-alpha", "<1.0.0-0"),
        ("0.x.x-alpha || 1.x.x-alpha", "<1.0.0-0||>=1.0.0 <2.0.0-0"),
        ("0.x.x-alpha <1.x.x-alpha", "<1.0.0-0"),
    ];

    for (input, expected) in cases {
        assert_eq!(r(input).to_string(), expected, "{input}");
    }
}

// --- real-world version strings (Next.js / shadcn / Vite ecosystem) ---

#[test]
fn real_world_canary_versions() {
    // Next.js canary: major.minor.patch-canary-hash-date
    assert!("19.3.0-canary-5e9eedb5-20260312".parse::<Version>().is_ok());
    // Next.js experimental: 0.0.0-experimental-hash-date
    assert!(
        "0.0.0-experimental-5e9eedb5-20260312"
            .parse::<Version>()
            .is_ok()
    );
    // classic RC and canary
    assert!("19.0.0-canary.0".parse::<Version>().is_ok());
    assert!("1.0.0-rc.9".parse::<Version>().is_ok());
    assert!("19.0.0-rc.0".parse::<Version>().is_ok());
    // alpha / beta
    assert!("1.0.0-alpha.1".parse::<Version>().is_ok());
    assert!("1.0.0-beta.2".parse::<Version>().is_ok());

    // pre-release < release
    assert!(v("19.3.0-canary-5e9eedb5-20260312") < v("19.3.0"));
    assert!(v("0.0.0-experimental-5e9eedb5-20260312") < v("0.0.0"));
    assert!(v("19.0.0-canary.0") < v("19.0.0"));
    assert!(v("1.0.0-rc.9") < v("1.0.0"));

    // canary.N: numeric comparison (9 < 10)
    assert!(v("19.0.0-canary.9") < v("19.0.0-canary.10"));
    // lexicographic: alpha < beta < rc
    assert!(v("1.0.0-alpha") < v("1.0.0-beta"));
    assert!(v("1.0.0-beta") < v("1.0.0-rc"));
    // canary < rc (c < r)
    assert!(v("19.0.0-canary.0") < v("19.0.0-rc.0"));
}

#[test]
fn real_world_ranges() {
    // shadcn peerDependencies: react ^18 || ^19
    assert!(r("^18 || ^19").satisfies(&v("18.0.0")));
    assert!(r("^18 || ^19").satisfies(&v("18.19.0")));
    assert!(r("^18 || ^19").satisfies(&v("19.0.0")));
    assert!(r("^18 || ^19").satisfies(&v("19.5.0")));
    assert!(!r("^18 || ^19").satisfies(&v("17.9.9")));
    assert!(!r("^18 || ^19").satisfies(&v("20.0.0")));

    // Node.js LTS multi-range
    assert!(r("18.x || 19.x || >=20").satisfies(&v("18.0.0")));
    assert!(r("18.x || 19.x || >=20").satisfies(&v("18.19.1")));
    assert!(r("18.x || 19.x || >=20").satisfies(&v("19.5.0")));
    assert!(r("18.x || 19.x || >=20").satisfies(&v("20.0.0")));
    assert!(r("18.x || 19.x || >=20").satisfies(&v("22.5.0")));
    assert!(!r("18.x || 19.x || >=20").satisfies(&v("17.9.9")));

    // Node.js dual range
    assert!(r(">=18 <20").satisfies(&v("18.0.0")));
    assert!(r(">=18 <20").satisfies(&v("19.9.9")));
    assert!(!r(">=18 <20").satisfies(&v("20.0.0")));
    assert!(!r(">=18 <20").satisfies(&v("17.9.9")));

    // large major version range (^100 || 20 pattern user confirmed)
    // ^100 = >=100.0.0 <101.0.0, 20 = x-range >=20.0.0 <21.0.0
    assert!(r("^100 || 20").satisfies(&v("100.0.0")));
    assert!(r("^100 || 20").satisfies(&v("100.9.1")));
    assert!(r("^100 || 20").satisfies(&v("20.0.0")));
    assert!(r("^100 || 20").satisfies(&v("20.5.1")));
    assert!(!r("^100 || 20").satisfies(&v("101.0.0")));
    assert!(!r("^100 || 20").satisfies(&v("21.0.0")));
    assert!(!r("^100 || 20").satisfies(&v("19.9.9")));

    // TypeScript-style: ^5.0.0
    assert!(r("^5.0.0").satisfies(&v("5.0.0")));
    assert!(r("^5.0.0").satisfies(&v("5.8.3")));
    assert!(!r("^5.0.0").satisfies(&v("6.0.0")));
    assert!(!r("^5.0.0").satisfies(&v("4.9.9")));

    // next.js: >=13.4.0
    assert!(r(">=13.4.0").satisfies(&v("13.4.0")));
    assert!(r(">=13.4.0").satisfies(&v("14.0.0")));
    assert!(r(">=13.4.0").satisfies(&v("15.3.0")));
    assert!(!r(">=13.4.0").satisfies(&v("13.3.9")));

    // Common React peer dependency range with RC support
    let react_peer = r("^16.8 || ^17.0 || ^18.0 || ^19.0 || ^19.0.0-rc");
    assert!(react_peer.satisfies(&v("16.8.0")));
    assert!(react_peer.satisfies(&v("17.0.2")));
    assert!(react_peer.satisfies(&v("18.3.1")));
    assert!(react_peer.satisfies(&v("19.0.0")));
    assert!(react_peer.satisfies(&v("19.0.0-rc.1")));
    assert!(!react_peer.satisfies(&v("16.7.9")));
    assert!(!react_peer.satisfies(&v("20.0.0")));
    assert!(!react_peer.satisfies(&v("19.1.0-rc.0")));
}

#[test]
fn real_world_canary_ranges() {
    // canary range: same tuple + pre-release in comparator → canary versions accepted
    assert!(r(">=18.0.0-canary.0 <19.0.0").satisfies(&v("18.0.0-canary.5")));
    assert!(r(">=18.0.0-canary.0 <19.0.0").satisfies(&v("18.5.0")));
    assert!(!r(">=18.0.0-canary.0 <19.0.0").satisfies(&v("19.0.0")));
    assert!(!r(">=18.0.0-canary.0 <19.0.0").satisfies(&v("17.9.9")));

    // ^18 does NOT match canary (pre-release restriction: no comparator has same tuple + pre)
    assert!(!r("^18").satisfies(&v("18.0.0-canary.0")));
    // but stable versions within range are fine
    assert!(r("^18").satisfies(&v("18.5.0")));

    // Next.js canary hash format: single identifier with dashes
    assert!(r(">=19.3.0-canary.0 <19.4.0").satisfies(&v("19.3.0-canary-5e9eedb5-20260312")));
    // canary of a different tuple is rejected by pre-release restriction
    assert!(!r(">=19.3.0-canary.0 <19.4.0").satisfies(&v("19.4.0-canary.0")));
    // canary version not matching the numeric comparator
    assert!(!r(">=19.3.0-canary.0 <19.4.0").satisfies(&v("19.2.0-canary.0")));

    // experimental range
    assert!(
        r(">=0.0.0-experimental.0 <1.0.0").satisfies(&v("0.0.0-experimental-5e9eedb5-20260312"))
    );
    assert!(!r(">=0.0.0-experimental.0 <1.0.0").satisfies(&v("1.0.0")));
}
