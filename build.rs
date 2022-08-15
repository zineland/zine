use std::io::{Error, ErrorKind};
use std::process::Command;

fn main() {
    // Obtain cargo pkg version as app version.
    let package_version: String = option_env!("CARGO_PKG_VERSION")
        .unwrap_or("(Unknown Cargo package version)")
        .to_string();

    // Obtain build infomation from Git.
    let build_info: String =
        from_git().unwrap_or_else(|_| "(Build info from Git not present)".into());

    println!("cargo:rustc-env=ZINE_VERSION={}", package_version);
    println!("cargo:rustc-env=BUILD_INFO={}", build_info);
}

fn run(args: &[&str]) -> Result<String, std::io::Error> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    match out.status.success() {
        true => Ok(String::from_utf8(out.stdout).unwrap().trim().to_string()),
        false => Err(Error::new(ErrorKind::Other, "Command not successful.")),
    }
}

fn from_git() -> Result<String, std::io::Error> {
    // Read the current git commit hash
    let rev = run(&["git", "rev-parse", "--verify", "--short", "HEAD"])?;
    println!("cargo:rustc-env=GIT_REV={}", rev);

    // Read the current branch name.
    let branch = run(&["git", "rev-parse", "--abbrev-ref", "HEAD"])?;
    println!("cargo:rustc-env=GIT_BRANCH={}", branch);

    // Read date from current build branch.
    let date_binding = run(&["git", "show", &branch, "--pretty=format:\"%ci %cr\""]).unwrap();
    let mut date = date_binding.split_once(' ').unwrap().0.chars();
    date.next();
    println!("cargo:rustc-env=LAST_COMMIT_DATE={}", date.as_str());

    // Combined
    Ok(format!(
        "build branch in \"{}\", lasted commit id: {}",
        branch, rev
    ))
}
