//! End-to-end tests that run the ferry binary against temporary directories.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A scratch directory with its own selection file, removed on drop.
struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "ferry-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("dst")).unwrap();
        // Canonical, so paths compare equal to what ferry stores (macOS /tmp is a symlink).
        Sandbox { root: root.canonicalize().unwrap() }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative)).unwrap()
    }

    fn ferry(&self, dir: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ferry"))
            .args(args)
            .current_dir(self.path(dir))
            .env("FERRY_CACHE_DIR", self.path("cache"))
            .output()
            .unwrap()
    }

    /// Runs ferry and asserts it succeeded.
    fn ok(&self, dir: &str, args: &[&str]) -> String {
        let out = self.ferry(dir, args);
        assert!(
            out.status.success(),
            "ferry {:?} failed:\n{}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Runs ferry, asserts it failed, and returns stderr.
    fn fails(&self, dir: &str, args: &[&str]) -> String {
        let out = self.ferry(dir, args);
        assert!(!out.status.success(), "ferry {:?} unexpectedly succeeded", args);
        String::from_utf8_lossy(&out.stderr).into_owned()
    }

    fn selection(&self) -> Vec<PathBuf> {
        self.ok("src", &["list"])
            .lines()
            .skip(1)
            .map(|l| PathBuf::from(l.trim().trim_end_matches("  (missing)")))
            .collect()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn exists(path: &Path) -> bool {
    path.symlink_metadata().is_ok()
}

#[test]
fn relative_selection_is_usable_from_another_directory() {
    let s = Sandbox::new();
    s.write("src/a.txt", "a");
    s.ok("src", &["select", "a.txt"]);
    assert_eq!(s.selection(), vec![s.path("src/a.txt")]);

    s.ok("dst", &["copy"]);
    assert_eq!(s.read("dst/a.txt"), "a");
    assert_eq!(s.read("src/a.txt"), "a");
    assert!(s.selection().is_empty());
}

#[test]
fn move_relocates_files() {
    let s = Sandbox::new();
    s.write("src/a.txt", "a");
    s.ok("src", &["select", "a.txt"]);
    s.ok("dst", &["move"]);
    assert_eq!(s.read("dst/a.txt"), "a");
    assert!(!exists(&s.path("src/a.txt")));
}

#[test]
fn selections_accumulate_without_duplicates() {
    let s = Sandbox::new();
    s.write("src/a", "");
    s.write("src/b", "");
    s.ok("src", &["select", "a"]);
    s.ok("src", &["select", "a", "b"]);
    assert_eq!(s.selection(), vec![s.path("src/a"), s.path("src/b")]);
}

#[test]
fn conflicts_are_all_reported_and_nothing_is_transferred() {
    let s = Sandbox::new();
    for name in ["one", "two", "three"] {
        s.write(&format!("src/{name}"), "new");
    }
    s.write("dst/two", "old");
    s.write("dst/three", "old");
    s.ok("src", &["select", "one", "two", "three"]);

    let err = s.fails("dst", &["move"]);
    assert!(err.contains("two") && err.contains("three"), "{err}");
    assert!(!exists(&s.path("dst/one")), "nothing should be moved");
    assert_eq!(s.selection().len(), 3);

    s.ok("dst", &["move", "--force"]);
    assert_eq!(s.read("dst/two"), "new");
    assert_eq!(s.read("dst/three"), "new");
    // No backup files are left behind.
    assert_eq!(fs::read_dir(s.path("dst")).unwrap().count(), 3);
}

#[test]
fn force_copy_onto_itself_is_refused_and_keeps_content() {
    let s = Sandbox::new();
    s.write("src/a.txt", "precious");
    s.ok("src", &["select", "a.txt"]);
    let err = s.fails("src", &["copy", "--force"]);
    assert!(err.contains("already in the current directory"), "{err}");
    assert_eq!(s.read("src/a.txt"), "precious");
}

#[test]
fn items_with_the_same_name_are_refused() {
    let s = Sandbox::new();
    s.write("src/a/x.txt", "a");
    s.write("src/b/x.txt", "b");
    s.ok("src", &["select", "a/x.txt", "b/x.txt"]);
    let err = s.fails("dst", &["copy"]);
    assert!(err.contains("would both be placed"), "{err}");
    assert!(!exists(&s.path("dst/x.txt")));
}

#[test]
fn nested_items_are_refused() {
    let s = Sandbox::new();
    s.write("src/dir/f", "");
    s.ok("src", &["select", "dir", "dir/f"]);
    let err = s.fails("dst", &["move"]);
    assert!(err.contains("one contains the other"), "{err}");
    assert!(exists(&s.path("src/dir/f")));
}

#[test]
fn missing_items_are_reported_and_can_be_deselected() {
    let s = Sandbox::new();
    s.write("src/a", "");
    s.write("src/b", "");
    s.ok("src", &["select", "a", "b"]);
    fs::remove_file(s.path("src/b")).unwrap();

    assert!(s.ok("src", &["list"]).contains("(missing)"));
    let err = s.fails("dst", &["copy"]);
    assert!(err.contains("no longer exists"), "{err}");

    s.ok("src", &["deselect", "--missing"]);
    assert_eq!(s.selection(), vec![s.path("src/a")]);
}

#[test]
fn deselect_and_clear() {
    let s = Sandbox::new();
    s.write("src/a", "");
    s.write("src/b", "");
    s.ok("src", &["select", "a", "b"]);
    s.ok("dst", &["deselect", "../src/a"]);
    assert_eq!(s.selection(), vec![s.path("src/b")]);
    s.ok("src", &["clear"]);
    assert!(s.selection().is_empty());
}

#[test]
fn directories_are_copied_recursively() {
    let s = Sandbox::new();
    s.write("src/tree/top.txt", "top");
    s.write("src/tree/deep/inner.txt", "inner");
    s.ok("src", &["select", "tree"]);
    s.ok("dst", &["copy"]);
    assert_eq!(s.read("dst/tree/top.txt"), "top");
    assert_eq!(s.read("dst/tree/deep/inner.txt"), "inner");
    assert_eq!(s.read("src/tree/deep/inner.txt"), "inner");
}

#[cfg(unix)]
#[test]
fn symlinks_inside_directories_are_preserved() {
    let s = Sandbox::new();
    s.write("src/tree/target.txt", "t");
    std::os::unix::fs::symlink("target.txt", s.path("src/tree/link")).unwrap();
    s.ok("src", &["select", "tree"]);
    s.ok("dst", &["copy"]);
    assert_eq!(fs::read_link(s.path("dst/tree/link")).unwrap(), PathBuf::from("target.txt"));
}

#[test]
fn regex_matches_paths_relative_to_the_search_root() {
    let s = Sandbox::new();
    s.write("src/b.rs", "");
    s.write("src/sub/c.rs", "");
    s.write("src/sub/notes.txt", "");

    s.ok("src", &["select", "--regex", r"^b\.rs$"]);
    assert_eq!(s.selection(), vec![s.path("src/b.rs")]);
    s.ok("src", &["clear"]);

    // Directories are not selected, only the files beneath them.
    s.ok("dst", &["select", "--regex", "sub", "--path", "../src"]);
    assert_eq!(s.selection(), vec![s.path("src/sub/c.rs"), s.path("src/sub/notes.txt")]);
    s.ok("src", &["clear"]);

    s.ok("src", &["select", "--regex", r"^sub/.*\.rs$"]);
    assert_eq!(s.selection(), vec![s.path("src/sub/c.rs")]);
}

#[test]
fn dry_run_does_not_change_the_selection() {
    let s = Sandbox::new();
    s.write("src/a", "");
    let out = s.ok("src", &["select", "--dry-run", "a"]);
    assert!(out.contains("would select"));
    assert!(s.selection().is_empty());
}

#[cfg(unix)]
#[test]
fn file_names_with_newlines_round_trip() {
    let s = Sandbox::new();
    s.write("src/new\nline", "n");
    s.ok("src", &["select", "new\nline"]);
    s.ok("dst", &["copy"]);
    assert_eq!(s.read("dst/new\nline"), "n");
}

#[test]
fn newline_separated_selection_from_older_versions_is_read() {
    let s = Sandbox::new();
    s.write("src/a", "a");
    s.write("src/b", "b");
    let old = format!("{}\n{}", s.path("src/a").display(), s.path("src/b").display());
    s.write("cache/selection", &old);
    s.ok("dst", &["copy"]);
    assert_eq!(s.read("dst/a"), "a");
    assert_eq!(s.read("dst/b"), "b");
}
