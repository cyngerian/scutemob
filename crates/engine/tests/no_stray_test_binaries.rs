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
//! every test in it. `cargo test --test combat` will cheerfully print
//! `ok. 69 passed; 0 failed` with six tests missing. All of that is caught here.
//!
//! The declaration check is textual, so three of these tests exist only to keep
//! the text honest: a `main.rs` may hold nothing but `//!` docs and bare `mod x;`
//! lines, group dirs are flat, and the group set on disk equals `EXPECTED_GROUPS`.
//! Each closes a way to satisfy the check while still deleting coverage.
//!
//! Layout, the rule for where a new test file goes, and the eight attacks these
//! five tests were validated against: `docs/sr-9a-test-consolidation.md`.

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

/// Directories under `tests/` that Cargo does not treat as test targets (no
/// `main.rs`), and that this gate must therefore not treat as groups.
///
/// `proptest-regressions/` is written by `proptest` the first time a property
/// test fails, to persist the failing seed. Cargo ignores it. Without this
/// exemption, one property-test failure produces *two* red tests — the real one
/// and `every_expected_group_exists_and_has_a_module_root` — and the second
/// buries the first. `tests/core/` has carried four proptest files since before
/// this gate existed (SR-9b).
const NON_GROUP_DIRS: &[&str] = &["proptest-regressions"];

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

/// Lines of `tests/<group>/main.rs` that are neither blank nor `//!` doc comment.
fn main_rs_code_lines(group: &str) -> Vec<String> {
    let main = fs::read_to_string(tests_dir().join(group).join("main.rs"))
        .unwrap_or_else(|e| panic!("group `{group}` has no main.rs: {e}"));
    main.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with("//!"))
        .collect()
}

/// `mod foo;` declarations in `tests/<group>/main.rs`.
///
/// Only matches the bare form. That is not laziness — `group_main_rs_declares_modules_and
/// _nothing_else` separately forbids every other form, so anything this function fails to
/// see is already a hard error. See that test for why.
fn declared_modules(group: &str) -> BTreeSet<String> {
    main_rs_code_lines(group)
        .iter()
        .filter_map(|l| l.strip_prefix("mod "))
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
        .filter(|name| !NON_GROUP_DIRS.contains(&name.as_str()))
        .collect();
    let expected: BTreeSet<String> = EXPECTED_GROUPS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        on_disk, expected,
        "the group dirs on disk do not match EXPECTED_GROUPS; update this list and \
         docs/sr-9a-test-consolidation.md together"
    );
}

/// A group's `main.rs` is a module list and nothing else.
///
/// Without this, `every_module_file_is_declared_in_its_group` is a *textual* check
/// and several ways to satisfy it while still deleting coverage survive:
///
/// - `#[cfg(feature = "never")] mod foo;` — declared to this file's parser, compiled
///   out by rustc. The tests vanish and the gate says "all declared".
/// - `#[path = "elsewhere.rs"] mod foo;` — `foo.rs` is declared *and* never compiled.
/// - `mod foo { … }` — an inline module named after a file that is not compiled.
/// - `pub mod foo;` — parses as undeclared here, so it fails *the wrong test* with a
///   confusing message.
///
/// Rather than teach the parser each attack, forbid everything that is not a bare
/// `mod x;`. The grammar of these files is small on purpose.
#[test]
fn group_main_rs_declares_modules_and_nothing_else() {
    for group in EXPECTED_GROUPS {
        for line in main_rs_code_lines(group) {
            let is_bare_mod_decl = line
                .strip_prefix("mod ")
                .and_then(|rest| rest.strip_suffix(';'))
                .is_some_and(|name| {
                    !name.is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                });
            assert!(
                is_bare_mod_decl,
                "tests/{group}/main.rs must contain only `//!` docs and bare `mod x;` lines, \
                 but has: `{line}`\n\
                 Attributes, `pub mod`, `#[path]`, and inline `mod x {{ … }}` are all ways to \
                 look declared while not being compiled. See docs/sr-9a-test-consolidation.md."
            );
        }
    }
}

/// Group dirs are flat. `module_files` reads one level, so a `tests/<group>/sub/foo.rs`
/// would be invisible to the declaration check below — undeclared, uncompiled, unnoticed.
#[test]
fn group_dirs_are_flat() {
    for group in EXPECTED_GROUPS {
        let nested: Vec<String> = fs::read_dir(tests_dir().join(group))
            .expect("group dir is readable")
            .map(|e| e.expect("readable dir entry"))
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            nested.is_empty(),
            "tests/{group}/ has subdirectories {nested:?}; the declaration check only sees \
             the top level of a group, so files under them would silently not be compiled"
        );
    }
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
