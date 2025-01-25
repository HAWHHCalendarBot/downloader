use std::path::Path;
use std::process::{Command, ExitStatus};

pub const FOLDER: &str = "events";

fn raw(args: &[&str]) -> ExitStatus {
    Command::new("git")
        .args(args)
        .current_dir(FOLDER)
        .status()
        .expect("git process should execute")
}

fn successful(args: &[&str]) {
    let status = raw(args);
    assert!(
        status.success(),
        "failed git command. Status code {status}. git {args:?}"
    );
}

pub fn pull() {
    if Path::new(FOLDER).join(".git").exists() {
        successful(&["pull", "--ff-only"]);
    } else {
        let status = Command::new("git")
            .args([
                "clone",
                "-q",
                "--depth",
                "1",
                "git@github.com:HAWHHCalendarBot/eventfiles.git",
                FOLDER,
            ])
            .status()
            .expect("git process should execute");
        assert!(status.success(), "git clone status code {status}");
    }
}

pub fn add(arg: &str) {
    successful(&["add", arg]);
}

fn commit() {
    let _status = raw(&[
        "commit",
        "--no-gpg-sign",
        "--author",
        "downloader <calendarbot-downloader@hawhh.de>",
        "--message",
        "update",
    ]);
    // ignore status as "nothing to commit" is also a non success. Maybe handle in a better way my checking stdout.
}

pub fn commit_and_push() {
    commit();

    #[cfg(not(debug_assertions))]
    successful(&["push"]);
}

/// Roll back to the versions from the repo.
///
/// This is needed when some base url failed during the download but others went fine.
pub fn checkout() {
    successful(&["checkout", "."]);
}
