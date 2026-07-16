//! SR-23 anti-rot gate for the LKI-vs-fizzle diagnostics vocabulary.
//!
//! Background: `state::diagnostics` classifies an absent `ObjectId` as either an
//! engine bug (`expect_object`, which `debug_assert!`s) or a rules-correct fizzle
//! (`fizzle_object`, a quiet live `self.objects.get`). Separately, SR-13 added a
//! genuine *last-known-information* store, `GameState::lki_object_snapshot`, keyed on
//! a retired battlefield id and holding layer-resolved characteristics.
//!
//! The fizzle getter used to be named `lki_object`, and the `expect_object` assert
//! told authors "If this id can be last-known-information (CR 400.7), use
//! `GameState::lki_object` instead." That text *reads* like it points at the LKI
//! store, but `lki_object` was a live getter that returns `None` for a departed
//! object — so an author at a CR 608.2h damage site (where the departed source's
//! last-known keywords *are* required) would be steered to a getter with no
//! information while a valid snapshot sat unused. SR-23 renamed the family to
//! `fizzle_*` and rewrote every assert/doc string.
//!
//! This test pins the corrected text so the misdirection cannot silently return.
//! It scans the diagnostics source rather than a runtime value because the guidance
//! lives only in comments and `debug_assert!` format strings, neither of which
//! survives into a compiled artifact — the same source-scan technique SR-5's keyword
//! registry, SR-8's protocol fingerprint, and SR-12's deviation scan all use. The
//! forbidden needles are held *here*, not in `diagnostics.rs`, so the scan never
//! matches its own source (the self-reference trap).

use std::fs;
use std::path::PathBuf;

/// `crates/engine/src/state/diagnostics.rs`, the file that owns the vocabulary.
fn diagnostics_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/state/diagnostics.rs");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Collapse Rust multi-line string continuations (`\` + newline + indent) and all
/// runs of whitespace to single spaces, so an assert message split across source
/// lines matches as one logical string.
fn normalize(src: &str) -> String {
    src.replace('\\', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The corrected guidance must be present — every LKI-semantics mention points at the
/// real snapshot store, and `fizzle_object` is explicitly labelled a live lookup.
#[test]
fn corrected_lki_guidance_is_pinned() {
    let norm = normalize(&diagnostics_source());

    let required = [
        // expect_object assert: LKI sites are sent to the snapshot store, not the getter.
        "read GameState::lki_object_snapshot instead",
        "fizzle_object is a live lookup and has none",
        // expect_object_mut assert: the LKI store is read-only, so an LKI site is a read.
        "last-known-information store (GameState::lki_object_snapshot) is read-only",
        // module docs: fizzle_object is explicitly disqualified at LKI-semantics sites.
        "do not reach for it at an LKI-semantics site",
    ];
    for needle in required {
        assert!(
            norm.contains(needle),
            "SR-23: diagnostics.rs lost the corrected LKI guidance {needle:?}. \
             If you reworded it, update this gate too — but the invariant stands: \
             no assert/doc may steer a last-known-information site at the live getter."
        );
    }
}

/// The misdirection must not return: no assert may name a *live* getter as the answer
/// for a last-known-information site, and the old `lki_object`/`lki_object_mut` method
/// names must not reappear in this module (they now mean the snapshot store elsewhere).
#[test]
fn the_misdirection_cannot_return() {
    let src = diagnostics_source();
    let norm = normalize(&src);

    let forbidden = [
        // The exact pre-SR-23 assert text (live getter presented as the LKI answer).
        "use GameState::lki_object instead",
        "use GameState::lki_object_mut instead",
        // The intermediate post-rename misdirection: an LKI mention steered at the
        // live fizzle getter rather than the snapshot store.
        "last-known-information (CR 400.7), use GameState::fizzle_object instead",
        "last-known-information (CR 400.7), use GameState::fizzle_object_mut instead",
    ];
    for needle in forbidden {
        assert!(
            !norm.contains(needle),
            "SR-23: diagnostics.rs steers a last-known-information site at a live getter \
             ({needle:?}). Point CR 608.2h sites at GameState::lki_object_snapshot instead."
        );
    }

    // The fizzle family was renamed off the `lki_` prefix so `lki_` means exactly one
    // thing (the snapshot store). The old method definitions must be gone from here.
    assert!(
        !src.contains("fn lki_object("),
        "SR-23: `fn lki_object` reappeared — the fizzle getter must stay `fizzle_object`"
    );
    assert!(
        !src.contains("fn lki_object_mut("),
        "SR-23: `fn lki_object_mut` reappeared — it must stay `fizzle_object_mut`"
    );
}

/// Non-vacuity: prove the scan read the intended file and that the anchors it depends
/// on actually exist, so the two gates above are not passing against an empty or
/// mis-pathed read.
#[test]
fn scan_denominator_is_non_vacuous() {
    let src = diagnostics_source();
    assert!(
        src.len() > 2_000,
        "diagnostics.rs is suspiciously small ({} bytes) — wrong file?",
        src.len()
    );
    for anchor in [
        "fn expect_object(",
        "fn expect_object_mut(",
        "fn fizzle_object(",
    ] {
        assert!(
            src.contains(anchor),
            "expected anchor {anchor:?} in diagnostics.rs"
        );
    }
    let snapshot_refs = src.matches("lki_object_snapshot").count();
    assert!(
        snapshot_refs >= 4,
        "expected the corrected guidance to name lki_object_snapshot several times, \
         found {snapshot_refs} — the LKI redirect may be missing"
    );
}
