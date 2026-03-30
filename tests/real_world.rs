#![allow(clippy::restriction)]
//! Compatibility tests with semver on npm.

use js_semver::{Range, Version};

fn v(s: &str) -> Version {
    s.parse().unwrap()
}
fn r(s: &str) -> Range {
    s.parse().unwrap()
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
