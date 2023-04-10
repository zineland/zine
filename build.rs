use std::io::{Error, ErrorKind};
use std::process::Command; // A process builder, controlling how a new process should be spawned

fn main() {
    // Obtain build infomation from Git.
    let build_info: String =
        from_git().unwrap_or_else(|_| "Build info from Git not present".into());

    println!("cargo:rustc-env=BUILD_INFO={}", build_info);
}

fn run(args: &[&str]) -> Result<String, std::io::Error> {
    let out = Command::new(args[0]).args(&args[1..]).output()?; // Output() executes command as child process
    match out.status.success() { // Collecting status (child process) and checking for success
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
    // Here is a example output:
    //
    // 2022-8-16
    //
    let date = run(&["git", "show", &branch, "--pretty=format:%ci"])?;
    println!("cargo:rustc-env=LAST_COMMIT_DATE={}", &date[..10]);

    // Combined
    Ok(format!(
        "build branch in \"{}\", last commit id: {}",
        branch, rev
    ))
}
