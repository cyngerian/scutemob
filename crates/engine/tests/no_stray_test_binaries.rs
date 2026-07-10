//! SR-9a gate: the integration-test tree stays consolidated.
//!
//! Cargo compiles and links every top-level `crates/engine/tests/*.rs` file as its
//! own test binary. There were 297 of them; linking dominated test-build wall time.
//! They now live as modules under `tests/<group>/`, each group having a `main.rs`
//! module root that Cargo picks up as a single target.
//!
//! Nothing about that arrangement is self-enforcing. Dropping a new `tests/foo.rs`
//! next to the group dirs re-fragments the tree one file at a time and nothing
//! complains; and — worse — moving a file *into* a group dir without adding its
//! `mod` line to that group's `main.rs` compiles clean while silently deleting
//! every test in it. Both are caught here.
//!
//! Layout and the rule for where a new test file goes:
//! `docs/sr-9a-test-consolidation.md`.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// The only top-level `tests/*.rs` file permitted: this gate itself.
///
/// If you are about to add a name here, you are adding a link to every test
/// build. Put the file in a group instead.
const ALLOWED_TOP_LEVEL: &[&str] = &["no_stray_test_binaries.rs"];

/// The consolidated targets. A new group is a deliberate act — adding one means
/// editing this list, which means the `docs/` layout table gets updated too.
const EXPECTED_GROUPS: &[&str] = &[
    "casting",
    "combat",
    "core",
    "mechanics_a_d",
    "mechanics_e_l",
    "mechanics_m_z",
    "primitives",
    "rules",
    "scripts",
];

fn tests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// Every `.rs` file directly inside `tests/<group>/`, excluding `main.rs`.
fn module_files(group: &str) -> BTreeSet<String> {
    fs::read_dir(tests_dir().join(group))
        .unwrap_or_else(|e| panic!("group dir `{group}` is missing: {e}"))
        .map(|e| e.expect("readable dir entry").file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".rs") && n != "main.rs")
        .map(|n| n.trim_end_matches(".rs").to_string())
        .collect()
}

/// `mod foo;` declarations in `tests/<group>/main.rs`.
fn declared_modules(group: &str) -> BTreeSet<String> {
    let main = fs::read_to_string(tests_dir().join(group).join("main.rs"))
        .unwrap_or_else(|e| panic!("group `{group}` has no main.rs: {e}"));
    main.lines()
        .filter_map(|l| l.trim().strip_prefix("mod "))
        .filter_map(|l| l.strip_suffix(';'))
        .map(|n| n.trim().to_string())
        .collect()
}

/// A stray `tests/foo.rs` is a new test binary — and a new link on every build.
#[test]
fn no_top_level_test_binaries() {
    let stray: Vec<String> = fs::read_dir(tests_dir())
        .expect("tests/ is readable")
        .map(|e| e.expect("readable dir entry").file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".rs"))
        .filter(|n| !ALLOWED_TOP_LEVEL.contains(&n.as_str()))
        .collect();

    assert!(
        stray.is_empty(),
        "these files are top-level integration-test binaries: {stray:?}\n\
         Each one adds a link to every `cargo test` build. Move the file into one of\n\
         {EXPECTED_GROUPS:?} and add a `mod` line to that group's main.rs.\n\
         See docs/sr-9a-test-consolidation.md."
    );
}

/// Guards the group list itself: a group dir Cargo would ignore, or a listed
/// group that no longer exists, both mean the layout has drifted from the doc.
#[test]
fn every_expected_group_exists_and_has_a_module_root() {
    for group in EXPECTED_GROUPS {
        let main = tests_dir().join(group).join("main.rs");
        assert!(
            main.is_file(),
            "group `{group}` has no main.rs — Cargo will not build it as a test target"
        );
    }

    let on_disk: BTreeSet<String> = fs::read_dir(tests_dir())
        .expect("tests/ is readable")
        .map(|e| e.expect("readable dir entry"))
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let expected: BTreeSet<String> = EXPECTED_GROUPS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        on_disk, expected,
        "the group dirs on disk do not match EXPECTED_GROUPS; update this list and \
         docs/sr-9a-test-consolidation.md together"
    );
}

/// The one that matters. A `.rs` file inside a group dir with no `mod` line in
/// that group's `main.rs` is not compiled at all: it does not fail, it does not
/// warn, its tests simply cease to exist. That is precisely the silent-coverage
/// loss this consolidation could introduce, so it is machine-checked.
#[test]
fn every_module_file_is_declared_in_its_group() {
    for group in EXPECTED_GROUPS {
        let files = module_files(group);
        let declared = declared_modules(group);

        let undeclared: Vec<_> = files.difference(&declared).collect();
        assert!(
            undeclared.is_empty(),
            "in tests/{group}/: these files are not `mod`-declared in main.rs, so \
             none of their tests run: {undeclared:?}"
        );

        let phantom: Vec<_> = declared.difference(&files).collect();
        assert!(
            phantom.is_empty(),
            "tests/{group}/main.rs declares modules with no file: {phantom:?}"
        );
    }
}
