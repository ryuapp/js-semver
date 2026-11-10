#![no_main]

use core::fmt::Write as _;
use std::hint::black_box;

use js_semver::{Range, Version};
use libfuzzer_sys::fuzz_target;

fn check_range(input: String) {
    let _ = black_box(Range::parse(black_box(&input)));
}

fn check_version(input: String) {
    let _ = black_box(Version::parse(black_box(&input)));
}

fuzz_target!(|_data: &[u8]| {
    check_range(">=1.0.0 ".repeat(20_000));

    let mut distinct_comparators = String::new();
    for major in 0..20_000 {
        let _ = write!(distinct_comparators, ">={major}.0.0 ");
    }
    check_range(distinct_comparators);

    check_range(">=1.0.0 <2.0.0 ".repeat(10_000));
    check_range("1.0.0||".repeat(20_000));
    check_range("*||".repeat(50_000));
    check_range("1.0.0 - 2.0.0 ".repeat(10_000));
    check_range(">".repeat(250_000));

    // Exercise whitespace normalization and metadata scanning with inputs far
    // larger than the ordinary version limit. The fuzz runner enforces a timeout.
    for whitespace in ["\t", "\x0b", "\u{feff}", "\u{2003}"] {
        let padding = whitespace.repeat(125_000);
        check_range(format!("{padding}^{padding}+build{padding}1{padding}"));
        check_range(format!("{padding}>=1.0.0{padding}<2.0.0{padding}"));
    }
    let long_build = "a".repeat(250_000);
    check_range(format!("^ +{long_build} 1"));
    check_range(format!("^ +{long_build}! 1"));
    check_range(format!("1.2.3 +{long_build} = 1.2.3"));
    check_range(format!("^ {}1", "+build ".repeat(50_000)));
    check_range(format!("^ {}", "+build ".repeat(50_000)));
    check_range(format!("> {}invalid", "v= ".repeat(100_000)));
    check_range(format!(">= {}1", "v".repeat(250_000)));
    check_range(format!(">= {}.1.x", "0".repeat(250_000)));

    check_version("9".repeat(250_000));
    check_version("0".repeat(250_000));
    check_version(format!("1.2.3-{}", "a".repeat(250_000)));
    check_version(format!("1.2.3+{}", "a".repeat(250_000)));
    check_version(format!("1.2.3-{}", "a.".repeat(120_000)));
});
