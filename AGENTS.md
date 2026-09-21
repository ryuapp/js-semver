# Development

This repository is a Cargo workspace containing the main `js-semver` crate, fuzz targets, benchmarks, and the website WASM crate.

Run the workspace checks from the repository root:

```console
cargo test
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --benches
```

Feature-specific checks must target the main crate because workspace members enable its default features:

```console
cargo test -p js-semver --no-default-features
cargo test -p js-semver --features serde
cargo test -p js-semver --no-default-features --features serde
```

# Website

Run website commands from `website`:

```console
deno task build
deno task test
deno task bench:crate
```

The website WASM build uses `website/wasm/profile.toml`; keep the `website` Cargo profile there instead of the workspace root.

# Packaging

Always target the publishable crate explicitly and inspect the package before release:

```console
cargo package -p js-semver --list
cargo publish -p js-semver --dry-run
```

# Fuzzing

Run fuzzing in a container so that the same Linux environment is used on Windows, Linux, and macOS.

On Windows, use WSL Containers from PowerShell in the repository root:

```powershell
wslc build --tag js-semver-fuzz --file fuzz/Dockerfile fuzz
wslc run --rm --volume "${PWD}:/workspace" js-semver-fuzz fuzz run --target x86_64-unknown-linux-gnu range -- -max_len=65536 -timeout=2
wslc run --rm --volume "${PWD}:/workspace" js-semver-fuzz fuzz run --target x86_64-unknown-linux-gnu version -- -max_len=65536 -timeout=2
wslc run --rm --volume "${PWD}:/workspace" js-semver-fuzz fuzz run --target x86_64-unknown-linux-gnu pathological -- -runs=1 -timeout=10
```

For a time-bounded local run, add `-max_total_time=30`. Keep `-timeout=2` so pathologically slow inputs are reported as artifacts.
