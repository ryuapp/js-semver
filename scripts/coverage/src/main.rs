use std::collections::BTreeSet;
use std::error::Error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let coverage_dir = root
        .join("target/coverage")
        .join(std::process::id().to_string());
    fs::create_dir_all(&coverage_dir)?;
    let raw_profile = coverage_dir.join("%m-%p.profraw");

    let mut build = cargo_test(&root, &raw_profile);
    build.args(["--no-run", "--message-format=json"]);
    let artifacts = build.stderr(Stdio::inherit()).output()?;
    if !artifacts.status.success() {
        return Err("coverage build failed".into());
    }

    let mut binaries = BTreeSet::new();
    for line in String::from_utf8(artifacts.stdout)?.lines() {
        if !line.starts_with('{') {
            continue;
        }
        let artifact: Value = serde_json::from_str(line)?;
        if artifact["reason"] != "compiler-artifact" || artifact["profile"]["test"] != true {
            continue;
        }
        if let Some(executable) = artifact["executable"].as_str() {
            binaries.insert(PathBuf::from(executable));
        }
    }
    if binaries.is_empty() {
        return Err("cargo did not produce any test binaries".into());
    }

    run(&mut cargo_test(&root, &raw_profile))?;

    let mut profiles = fs::read_dir(&coverage_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    profiles.retain(|path| {
        path.extension().and_then(|extension| extension.to_str()) == Some("profraw")
    });
    profiles.sort();
    if profiles.is_empty() {
        return Err("tests did not produce any raw coverage profiles".into());
    }

    let rustc_info = capture(Command::new("rustc").arg("-vV"))?;
    let host = rustc_info
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or("rustc did not report its host triple")?;
    let sysroot = capture(Command::new("rustc").arg("--print").arg("sysroot"))?;
    let llvm_tools = Path::new(sysroot.trim())
        .join("lib/rustlib")
        .join(host)
        .join("bin");
    let profdata = coverage_dir.join("merged.profdata");
    let mut merge =
        Command::new(llvm_tools.join(format!("llvm-profdata{}", std::env::consts::EXE_SUFFIX)));
    merge
        .arg("merge")
        .arg("-sparse")
        .args(&profiles)
        .arg("-o")
        .arg(&profdata);
    run(&mut merge)?;

    let mut export =
        Command::new(llvm_tools.join(format!("llvm-cov{}", std::env::consts::EXE_SUFFIX)));
    export
        .arg("export")
        .arg("-format=lcov")
        .arg("-ignore-filename-regex=/.cargo/registry/|/rustc/")
        .arg(format!("-instr-profile={}", profdata.display()));
    for binary in binaries {
        export.arg("-object").arg(binary);
    }
    export.stdout(File::create(root.join("lcov.info"))?);
    run(&mut export)
}

fn cargo_test(root: &Path, raw_profile: &Path) -> Command {
    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .args([
            "test",
            "-p",
            "js-semver",
            "--all-features",
            "--lib",
            "--tests",
        ])
        .env("RUSTFLAGS", "-C instrument-coverage")
        .env("LLVM_PROFILE_FILE", raw_profile);
    command
}

fn run(command: &mut Command) -> Result<(), Box<dyn Error>> {
    if command.status()?.success() {
        Ok(())
    } else {
        Err(format!("command failed: {command:?}").into())
    }
}

fn capture(command: &mut Command) -> Result<String, Box<dyn Error>> {
    let output = command.output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(format!("command failed: {command:?}").into())
    }
}
