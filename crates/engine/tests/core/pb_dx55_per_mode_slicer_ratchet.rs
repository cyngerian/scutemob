//! PB-DX55 Half 3 (`OOS-SIM5-5`) — the mechanism gate for the class this batch's fix
//! belongs to: *"a per-mode target slice is computed exactly once, by
//! `casting::per_mode_target_requirements`, and never re-derived a second way."*
//!
//! # Why a source scan, and why it is keyed on TWO independent axes
//!
//! This queue has a long, named record of source gates defeated by a differently-shaped
//! but behaviourally-identical needle (`OOS-DX51-6`, `OOS-DX54-6`, and the PB-DX26 ->
//! PB-DX43 -> PB-DX45 -> PB-DX47 chain the `unordered_iteration_ratchet.rs` module doc
//! itself narrates): *a gate written for one variant measures that variant.* A single
//! literal-string needle (e.g. `.flat_map(|&idx| mt.get(idx)`) would be defeated by a
//! `for` loop doing the identical thing, or by a renamed binding.
//!
//! So this file checks the SURFACE the fix actually changed — every CODE-level (not
//! comment, not string-literal) field access of `ModeSelection.mode_targets` anywhere in
//! `crates/engine/src` — rather than the SHAPE of any one implementation. Any new site is
//! a compile-clean, test-suite-green change that this gate still catches, because it
//! doesn't ask "does this look like the banned idiom" — it asks "does anything touch this
//! field that isn't already on the reviewed list".
//!
//! **That question was NOT independent of HOW the new code touches it, and the sentence
//! that said so was false.** The first draft required the name to be immediately preceded
//! by `.`, so it saw field accesses and nothing else; the `/review` planted a compiling
//! sixth slicer reached through `let ModeSelection { mode_targets, .. } = ms;` and every
//! test in this file stayed GREEN. Struct-pattern bindings and struct literals are now
//! sites too, and the claim above is true as rewritten rather than as originally
//! asserted. `OOS-DX55-9`.
//!
//! # Executed defeat, before this gate shipped
//!
//! A rogue site was PLANTED (`queries.rs`, a throwaway `fn _rogue_per_mode_probe` reading
//! `ms.mode_targets.as_ref().map(|mt| mt.iter().flatten().cloned().collect::<Vec<_>>())`,
//! deliberately spelled DIFFERENTLY from the shared slicer's `flat_map`/`.get(idx)` shape)
//! and this gate's `r_sites` assertion went RED, naming the exact new line. A second
//! plant used a `for` loop instead of any iterator adapter at all
//! (`for m in mt { out.extend(m.clone()); }`) and was caught the same way, for the same
//! reason: neither plant's SHAPE matters to a gate that counts SITES, not shapes. Both
//! plants were then removed and the gate reproduced GREEN. Neither survives in this
//! file; this paragraph is the record of the experiment.
//!
//! # Known limitation, shared with every sibling source-scan gate in this tree
//! (`unordered_iteration_ratchet.rs`, `bare_lookup_ratchet.rs`, SR-5's keyword registry):
//! only `//`-prefixed line comments are stripped, not `/* */` block comments, and a type
//! alias for `ModeSelection` used elsewhere would not be seen. Zero occurrences of either
//! exist in this crate today (checked at time of writing).
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/engine -> crates
    p.pop(); // crates -> workspace root
    p
}

fn walk_rs(dir: &Path, acc: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk_rs(&p, acc);
        } else if p.extension().is_some_and(|x| x == "rs") {
            acc.push(p);
        }
    }
}

/// One CODE-level (non-comment, non-string-literal) `<value>.mode_targets` field access.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Site {
    file: String,
    line: usize,
    owner: String,
}

/// Every real `.mode_targets` field-access site in `crates/engine/src`.
///
/// A match is REAL when: (1) the line survives `//` comment stripping (covers `//`,
/// `///` and `//!`, which all begin with the same two characters); (2) the token
/// `mode_targets` is a whole token (not a substring of a longer identifier, e.g.
/// `mode_targets_active`); (3) it is either immediately preceded by `.` (a genuine field
/// access) OR is a struct-pattern binding / struct-literal field inside a `{ .. }` whose
/// brace is reached scanning backwards over binding syntax only -- the SECOND half was
/// added after the `/review` defeated this gate with a `let ModeSelection {
/// mode_targets, .. } = ms;` slicer, and a bare local binding that merely shares the
/// name (e.g. `if let Some(mode_targets) = ...`) still does not qualify, because a `(`
/// stops the backward scan before any `{`; and (4) the identifier immediately before that
/// `.` is not the literal type name `ModeSelection` -- every string-literal error message
/// in this codebase that mentions the field spells it `ModeSelection.mode_targets`
/// (capitalised type name, invalid as real Rust field-access syntax), so filtering that
/// exact owner removes exactly the doc/string mentions and nothing else.
fn mode_targets_sites() -> Vec<Site> {
    let root = workspace_root().join("crates/engine/src");
    let mut files = Vec::new();
    walk_rs(&root, &mut files);
    files.sort();

    // Built at runtime, split across two literals, so this gate's own source cannot
    // match its own needle if this file is ever brought under the scan root.
    let needle = format!("mode{}", "_targets");

    let mut sites = Vec::new();
    for f in files {
        let src = std::fs::read_to_string(&f).expect("readable engine source");
        let rel = f
            .strip_prefix(&root)
            .expect("under crates/engine/src")
            .to_string_lossy()
            .replace('\\', "/");
        for (i, raw_line) in src.lines().enumerate() {
            let line = match raw_line.find("//") {
                Some(at) => &raw_line[..at],
                None => raw_line,
            };
            let b = line.as_bytes();
            let mut from = 0usize;
            while let Some(rel_at) = line[from..].find(&needle) {
                let at = from + rel_at;
                let after = at + needle.len();
                let ok_before =
                    at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_');
                let ok_after =
                    after >= b.len() || !(b[after].is_ascii_alphanumeric() || b[after] == b'_');
                from = at + 1;
                if !(ok_before && ok_after) {
                    continue;
                }
                // A site is either a FIELD ACCESS (`x.mode_targets`) or a STRUCT-PATTERN
                // BINDING (`let ModeSelection { mode_targets, .. } = ms;`, or the same
                // inside a `match`/`if let` arm).
                //
                // **The binding form was added after the `/review` DEFEATED this gate by
                // execution**: a compiling SIXTH slicer in `queries.rs` — the very file
                // this module's doc names — computing exactly what
                // `per_mode_target_requirements` computes, reached through
                // `let ModeSelection { mode_targets, .. } = ms;` instead of a `.`
                // access, was invisible to `r_sites`, to `scanner_is_not_vacuous` and to
                // `r_call_sites` alike. Both of the defeats recorded in this module's
                // doc used `ms.mode_targets`, i.e. the author's own spelling —
                // `OOS-DX54-6` again. And the doc's claim that the gate asks a question
                // *"independent of HOW the new code touches it"* was false precisely
                // because of the `b[at - 1] != b'.'` filter this comment replaces.
                // `OOS-DX55-9`.
                let is_field_access = at > 0 && b[at - 1] == b'.';
                let is_pattern_binding = {
                    // Scan backwards over `,`/whitespace/other binding names to a `{`
                    // that is preceded (modulo whitespace) by an identifier — i.e. a
                    // struct pattern or a struct literal. Either is a real touch of the
                    // field; a struct LITERAL that sets `mode_targets` is a site too.
                    let mut k = at;
                    let mut found = false;
                    while k > 0 {
                        k -= 1;
                        let c = b[k];
                        if c == b'{' {
                            found = true;
                            break;
                        }
                        if !(c.is_ascii_alphanumeric()
                            || c == b'_'
                            || c == b','
                            || c == b':'
                            || c == b'.'
                            || (c as char).is_whitespace())
                        {
                            break;
                        }
                    }
                    found
                };
                if !(is_field_access || is_pattern_binding) {
                    continue;
                }
                if is_pattern_binding && !is_field_access {
                    // Recorded under a synthetic owner so the allowlist names it exactly
                    // the way it names a field access and every count assertion below
                    // still applies.
                    sites.push(Site {
                        file: rel.clone(),
                        line: i + 1,
                        owner: "<destructured>".to_string(),
                    });
                    continue;
                }
                // Walk backward from the `.` to collect the owner identifier.
                let dot_at = at - 1;
                let mut start = dot_at;
                while start > 0 && (b[start - 1].is_ascii_alphanumeric() || b[start - 1] == b'_') {
                    start -= 1;
                }
                let owner = line[start..dot_at].to_string();
                if owner == "ModeSelection" {
                    // The type name in prose/string form (e.g. the error message
                    // "ModeSelection.mode_targets may not contain UpToN"), not a real
                    // field access on a value.
                    continue;
                }
                sites.push(Site {
                    file: rel.clone(),
                    line: i + 1,
                    owner,
                });
            }
        }
    }
    sites
}

/// The scanner finds a real tree and a real needle -- deliberately below the live 5, so a
/// harmless refactor that removes one legitimate reader does not, by itself, fail this
/// floor (it would still be caught by `r_sites`' exact-set assertion below).
const MIN_SITES_FOUND: usize = 3;

#[test]
fn scanner_is_not_vacuous() {
    let sites = mode_targets_sites();
    assert!(
        sites.len() >= MIN_SITES_FOUND,
        "only {} real `.mode_targets` field-access sites found; the scanner is broken \
         (expected >= {MIN_SITES_FOUND})",
        sites.len()
    );
}

/// The load-bearing assertion: the EXACT set of files/owners that may read
/// `ModeSelection.mode_targets` directly, each with the reason it is allowed to. A new
/// site anywhere -- including inside `crates/engine/src/rules/queries.rs`, the very file
/// PB-DX55 Half 3 edited -- fails this test by NAME, forcing it onto this list rather
/// than letting it through silently.
///
/// Five sites, none of them the batch's own new code:
/// 1. `rules/casting.rs`, owner `ms` -- `per_mode_target_requirements`'s OWN body. This
///    is the one and only place that slices `mode_targets` down to a `Vec<TargetRequirement>`
///    for a set of chosen mode indices; every other consumer (including PB-DX55's new
///    `queries::ability_target_requirements` and the rewritten
///    `abilities.rs::handle_activate_ability`) calls this function rather than reading
///    the field itself.
/// 2. `rules/abilities.rs`, owner `modes` -- `trigger_modal_plan`'s `mode_targets.is_none()`
///    boolean check (PB-DX35). A precondition read, immediately followed by a call to
///    `per_mode_target_requirements` for the actual slice -- not a second slicer.
/// 3. `rules/resolution.rs`, owner `modes` -- the SAME boolean-check shape, on the spell
///    resolution path (deciding whether a chosen mode's effect comes from
///    `modes.modes.get(idx)` directly or from the per-mode-target branch below it).
/// 4. `rules/resolution.rs`, owner `modes_ref` -- a DIFFERENT computation from
///    `per_mode_target_requirements`: it reads each chosen mode's `mode_targets[idx].len()`
///    to compute `stack_obj.targets` slice OFFSETS during EFFECT EXECUTION (CR 700.2c/700.2f,
///    PB-AC4), because targets for multiple modes are already announced as one flat,
///    concatenated `Vec<SpellTarget>` on the stack object and each mode's effect must see
///    only its own slice. This runs strictly AFTER the offer/cast-time slice
///    (`per_mode_target_requirements`) has already validated and announced the targets --
///    it is bookkeeping over an already-resolved announcement, not a second legality
///    derivation, and it has no activated-ability counterpart because
///    `handle_activate_ability` bakes its chosen mode's effect at ACTIVATION time
///    (`embedded_effect`), never at resolution.
/// 5. `state/hash.rs`, owner `self` -- the `HashInto` implementation for `ModeSelection`,
///    hashing the field's bytes. Unrelated to target-requirement derivation.
#[test]
fn r_sites_every_mode_targets_reader_is_on_this_reviewed_list() {
    let mut sites = mode_targets_sites();
    sites.sort();
    for s in &sites {
        eprintln!(
            "mode_targets site: {}:{} (owner `{}`)",
            s.file, s.line, s.owner
        );
    }

    let allowed: Vec<(&str, &str)> = vec![
        ("rules/casting.rs", "ms"),
        ("rules/abilities.rs", "modes"),
        ("rules/resolution.rs", "modes"),
        ("rules/resolution.rs", "modes_ref"),
        ("state/hash.rs", "self"),
    ];

    let mut unexpected: Vec<&Site> = Vec::new();
    for s in &sites {
        if !allowed.iter().any(|(f, o)| *f == s.file && *o == s.owner) {
            unexpected.push(s);
        }
    }
    assert!(
        unexpected.is_empty(),
        "a NEW `.mode_targets` field-access site was found and is not on the reviewed \
         allowlist -- this is exactly the shape of re-derivation PB-DX55 removed from \
         `abilities.rs::handle_activate_ability` (a FIFTH hand-rolled copy of the \
         per-mode slice). Add it to `casting::per_mode_target_requirements`'s callers \
         instead of reading the field directly, or add it to this test's `allowed` list \
         with a stated reason if it is genuinely a new legitimate purpose: {unexpected:#?}"
    );

    assert_eq!(
        sites.len(),
        allowed.len(),
        "exactly {} real `.mode_targets` sites are expected; got {} -- either a site \
         disappeared (tighten `allowed` above) or one was added without reddening the \
         `unexpected` check above (which would itself be a bug in this gate)",
        allowed.len(),
        sites.len()
    );
}

/// The whole-tree call-site census for `per_mode_target_requirements(` itself -- the
/// SHARED function, not the field it reads. Six real callers plus its own definition,
/// enumerated by name so a caller cannot silently disappear (which would mean a consumer
/// went back to reading the field directly -- caught independently by `r_sites` above)
/// or silently multiply beyond what this test names.
#[test]
fn r_call_sites_per_mode_target_requirements_has_exactly_six_callers() {
    let root = workspace_root().join("crates/engine/src");
    let mut files = Vec::new();
    walk_rs(&root, &mut files);
    files.sort();

    let needle = format!("per_mode{}", "_target_requirements(");
    let def_needle = format!("fn per_mode{}", "_target_requirements(");

    let mut call_sites: Vec<(String, usize)> = Vec::new();
    let mut def_sites: Vec<(String, usize)> = Vec::new();
    for f in &files {
        let src = std::fs::read_to_string(f).expect("readable engine source");
        let rel = f
            .strip_prefix(&root)
            .expect("under crates/engine/src")
            .to_string_lossy()
            .replace('\\', "/");
        for (i, raw_line) in src.lines().enumerate() {
            let line = match raw_line.find("//") {
                Some(at) => &raw_line[..at],
                None => raw_line,
            };
            if !line.contains(&needle) {
                continue;
            }
            if line.contains(&def_needle) {
                def_sites.push((rel.clone(), i + 1));
            } else {
                call_sites.push((rel.clone(), i + 1));
            }
        }
    }

    assert_eq!(
        def_sites.len(),
        1,
        "exactly one `fn per_mode_target_requirements` definition is expected, in \
         `rules/casting.rs` -- got {def_sites:?}"
    );
    assert_eq!(
        def_sites[0].0, "rules/casting.rs",
        "the shared slicer's definition moved out of `rules/casting.rs`: {def_sites:?}"
    );

    let expected_callers: Vec<&str> = vec![
        // `handle_cast_spell`'s own per-mode slice (CR 700.2a cast path).
        "rules/casting.rs",
        // `rules::queries::spell_target_requirements` (offer/query path for spells).
        "rules/queries.rs",
        // `rules::queries::ability_target_requirements` (PB-DX55, NEW).
        "rules/queries.rs",
        // `handle_activate_ability`'s `mode_targets_active` (PB-DX55, replaces the
        // FIFTH hand-rolled copy this batch deleted).
        "rules/abilities.rs",
        // `trigger_modal_plan`'s legality scan (PB-DX35) -- two call sites in one
        // function (the per-mode legality check, and the final chosen-mode slice).
        "rules/abilities.rs",
        "rules/abilities.rs",
    ];
    let mut got: Vec<&str> = call_sites.iter().map(|(f, _)| f.as_str()).collect();
    got.sort();
    let mut expected_sorted = expected_callers.clone();
    expected_sorted.sort();
    assert_eq!(
        got, expected_sorted,
        "call-site census for `per_mode_target_requirements(` disagrees with the pinned \
         set -- either a caller was added/removed for real, or this gate needs \
         re-deriving rather than the pin being trusted. Raw sites: {call_sites:?}"
    );
}
