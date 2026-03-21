# js-semver

[![License](https://img.shields.io/github/license/ryuapp/js-semver?labelColor=171717&color=39b54a&label=License)](https://github.com/ryuapp/js-semver/blob/main/LICENSE)
[![crates](https://img.shields.io/crates/v/js-semver?labelColor=171717&color=39b54a)](https://crates.io/crates/js-semver)
[![github repo](https://img.shields.io/badge/GitHub-ryuapp/js--semver-171717?labelColor=171717&color=39b54a)](https://github.com/ryuapp/js-semver)
[![codecov](https://codecov.io/gh/ryuapp/js-semver/graph/badge.svg?token=P7NMEB4IP7)](https://codecov.io/gh/ryuapp/js-semver)

A parser and evaluator for semantic versioning in JavaScript, including Node.js and Deno.

This crate is designed for the JavaScript ecosystem. It maintains high compatibility with the behavior of [node-semver](https://github.com/npm/node-semver) (the one npm uses) and has zero dependencies by default.

## Example

```rs
use js_semver::{BuildMetadata, PreRelease, Range, Version};

fn main() {
    let range = Range::parse(">=1.2.3 <1.8.0").unwrap();

    // Pre-release versions are not included in the range unless explicitly specified.
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        pre_release: PreRelease::parse("alpha.1").unwrap(),
        build: BuildMetadata::default(),
    };
    assert!(!range.satisfies(&version));

    let version = Version::parse("1.3.0").unwrap();
    assert!(range.satisfies(&version));
}
```

## Comparison with other crates

### node-semver

[node-semver](https://crates.io/crates/node-semver) crate has numerous issues, including unnecessary dependencies like `miette`, incompatibilities with npm's [node-semver](https://github.com/npm/node-semver), and the fact that it is no longer actively maintained.

### semver

[semver](https://crates.io/crates/semver) crate is designed for Cargo. Therefore, it is not well-suited for the Node.js ecosystem, such as parsing versions in `package.json`.

## License

MIT-0
