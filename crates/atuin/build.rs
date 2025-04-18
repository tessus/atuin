use std::env;
use std::process::Command;

fn main() {
    // Rerun when these paths are changed.
    // Someone could have checked-out a tag or specific commit, but no other files changed.
    println!("cargo:rerun-if-changed=.git");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/refs/tags");

    // Original atuin info
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output();
    let sha = match output {
        Ok(sha) => String::from_utf8(sha.stdout).unwrap(),
        Err(_) => String::from("NO_GIT"),
    };
    println!("cargo:rustc-env=GIT_HASH={sha}");

    // Read info from git. If it fails, use CARGO_PKG_VERSION.
    let maybe_atuin_version = version_from_git_info().or_else(|_| env::var("CARGO_PKG_VERSION"));

    if let Ok(version) = maybe_atuin_version {
        println!("cargo:rustc-env=ATUIN_VERSION={version}");
        println!("cargo:rustc-env=CARGO_PKG_VERSION={version}");
    }
}

fn run(args: &[&str]) -> Result<String, std::io::Error> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    if !out.status.success() {
        use std::io::Error;
        return Err(Error::other("Command not successful"));
    }
    Ok(String::from_utf8(out.stdout).unwrap().trim().to_string())
}

/// This method reads info from Git, namely tags, branch, and revision
/// To access these values, use:
///    - `env!("GIT_EXACT_TAG")`
///    - `env!("GIT_LAST_TAG")`
///    - `env!("GIT_BRANCH")`
///    - `env!("GIT_REV")`
///    - `env!("ATUIN_VERSION")`
fn version_from_git_info() -> Result<String, std::io::Error> {
    // The exact tag for the current commit, can be empty when
    // the current commit doesn't have an associated tag
    let exact_tag = run(&["git", "describe", "--abbrev=0", "--tags", "--exact-match"]).ok();
    if let Some(ref exact) = exact_tag {
        println!("cargo:rustc-env=GIT_EXACT_TAG={exact}");
    }

    // The last available tag, equal to exact_tag when the current commit is tagged
    // Can be empty in a shallow clone, in which case the version is set to CARGO_PKG_VERSION
    // Remove v from tag, if necessary
    let last_tag = run(&["git", "describe", "--abbrev=0", "--tags"]).ok();
    let version = match last_tag {
        Some(mut tag) => {
            if tag.starts_with("v") {
                tag.remove(0);
            }
            println!("cargo:rustc-env=GIT_LAST_TAG={tag}");
            tag
        }
        None => env::var("CARGO_PKG_VERSION")
            .expect("Cannot retrieve CARGO_PKG_VERSION. Something's off."),
    };

    // The current branch name
    let branch = run(&["git", "rev-parse", "--abbrev-ref", "HEAD"])?;
    println!("cargo:rustc-env=GIT_BRANCH={branch}");

    // The current git commit hash
    let rev = run(&["git", "rev-parse", "HEAD"])?;
    let rev_short = rev.get(..8).unwrap_or_default();
    println!("cargo:rustc-env=GIT_REV={rev_short}");

    // Combined version
    if let Some(exact) = exact_tag {
        Ok(exact)
    } else if &branch != "main" && &branch != "master" && &branch != "HEAD" {
        Ok(format!("{version}-{rev_short} ({branch})"))
    } else {
        Ok(format!("{version}-{rev_short}"))
    }
}
