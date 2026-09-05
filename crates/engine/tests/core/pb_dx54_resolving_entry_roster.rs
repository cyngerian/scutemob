//! PB-DX54 (`OOS-DX25c-6` CLOSED + rider `OOS-DX25-4` CLOSED): the structural gates and the
//! census for CR 608.2n's departure point.
//!
//! # What shipped
//!
//! `rules::resolution::resolve_top_of_stack_inner` used to open with
//! `state.stack_objects.pop_back()`, so for the whole of a resolution the resolving object's
//! stack ENTRY did not exist. `state::stack_registry::stack_index_for_announced_target`
//! returned `None` for it, and both `TargetSpellWithSingleTarget` and
//! `TargetSpellOrAbilityWithSingleTarget` resolve their candidate through that function — so a
//! victim spell could never be redirected onto the Misdirection/Bolt Bend redirecting it,
//! contrary to Misdirection's 2004-10-04 ruling (*"This spell is still on the stack when new
//! targets are selected for the spell"*).
//!
//! It now PEEKS, and the entry departs through `depart_resolving_stack_entry` at the two
//! CR-ordered points plus an idempotent backstop.
//!
//! # CR 608.2n, not CR 608.2m
//!
//! The seed row, the v4 memo row and this task's own acceptance criterion all cite **CR
//! 608.2m**. That rule is *"if an instant spell … **leaves the stack once it starts to
//! resolve**, it will continue to resolve fully"* — an object removed by SOMETHING ELSE
//! mid-resolution. The rule that puts the departure LAST is **CR 608.2n**, reinforced by
//! CR 608.2's own preamble (*"The steps described in rule 608.2n and 608.2p are followed
//! last"*). Every cite in this file is 608.2n, and `r2`'s failure message says so, because a
//! wrong cite is the half the next batch reuses (`OOS-DX49`).
//!
//! # Revert method
//!
//! Every structural gate below is written as a pure function over a source STRING, run once
//! against the real file (must pass) and once against a hand-built violation string that
//! reproduces the exact failure shape (must fail). That gives the same evidence a
//! file-mutation revert would, without this file's owning task ever editing
//! `crates/engine/src/` — the PB-DX52 precedent, and necessary here because another agent held
//! `resolution.rs` while this file was written.
//!
//! The two RIDER rows (`r4`, `r4b`) are deliberately **behavioural, not source** gates.
//! `OOS-DX52-2`'s lesson: a row that reddens only a source gate is telling you the behaviour
//! has no probe.

use std::collections::BTreeSet;
use std::path::PathBuf;

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::{
    all_cards, process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId,
    CardRegistry, CardType, Command, Completeness, CounterType, Effect, GameEvent, GameState,
    GameStateBuilder, LoyaltyCost, ObjectId, ObjectSpec, PlayerId, SpellTarget, Step, Target,
    TargetRequirement, TypeLine, ZoneId,
};

use crate::decision_site_walk::def_contains_variant;

// ── shared helpers ───────────────────────────────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

fn read_source(relative: &str) -> String {
    let path = workspace_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// Strip `//` line comments **and** `/* */` block comments, replacing each stripped byte with a
/// space so byte offsets are preserved. `OOS-DX32-6`: a gate that narrows to `//` alone is
/// silently defeated by the byte-identical sentence written as a block comment.
fn strip_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let mut depth = 1usize;
            out.push_str("  ");
            i += 2;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                } else {
                    out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
        } else {
            let ch = src[i..].chars().next().expect("char boundary");
            out.push_str(&src[i..i + ch.len_utf8()]);
            i += ch.len_utf8();
        }
    }
    out
}

/// Byte offset of the `{` matching the one at `open`, string-literal-aware.
fn matching_brace(src: &str, open: usize) -> Option<usize> {
    let b = src.as_bytes();
    if b.get(open) != Some(&b'{') {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open;
    let mut in_str = false;
    let mut in_char = false;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
        } else if in_char {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'\'' {
                in_char = false;
            }
        } else {
            match c {
                b'"' => in_str = true,
                b'\'' => in_char = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// The comment-stripped body of `fn <name>`, brace-matched. Fail-closed: `None` if the
/// signature is not found or the braces do not balance, so a rename cannot make a gate pass by
/// scanning an empty string.
fn fn_body(stripped: &str, name: &str) -> Option<String> {
    let sig = format!("fn {name}(");
    let at = stripped.find(&sig)?;
    let open = stripped[at..].find('{')? + at;
    let close = matching_brace(stripped, open)?;
    Some(stripped[open..=close].to_string())
}

fn count(hay: &str, needle: &str) -> usize {
    hay.matches(needle).count()
}

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const RESOLUTION_RS: &str = "crates/engine/src/rules/resolution.rs";
const SBA_RS: &str = "crates/engine/src/rules/sba.rs";

// ════════════════════════════════════════════════════════════════════════════════════════
// §A — structural gates
// ════════════════════════════════════════════════════════════════════════════════════════

/// r1: `resolve_top_of_stack_inner` must not pop the entry off the front of its own
/// resolution (CR 608.2n).
///
/// Keyed on the MECHANISM — any mutable route that removes the top entry — rather than on the
/// literal `pop_back()` spelling alone, because `OOS-DX51-6` proved four separate times that a
/// gate keyed on one spelling is defeated by the next one (`let v = &mut state.stack_objects;
/// v.pop_back()`).
///
/// **What this does NOT catch, stated rather than left to look total**: it is scoped to this
/// one function. A future removal added to a helper this function CALLS is invisible here; `r2`
/// is the ordering half and `r3` the routing half, and none of the three is a proof that some
/// third file does not remove the entry.
#[test]
fn r1_the_resolving_entry_is_not_popped_before_its_effect_runs() {
    let stripped = strip_comments(&read_source(RESOLUTION_RS));
    let body = fn_body(&stripped, "resolve_top_of_stack_inner").expect(
        "r1 fail-closed: fn resolve_top_of_stack_inner not found or its braces do not \
         balance -- re-derive this gate against the current source, do not delete it",
    );
    let offenders = pop_receivers(&body);
    assert!(
        offenders.is_empty(),
        "r1: CR 608.2n -- the object leaves the stack as the FINAL part of its resolution, so \
         `resolve_top_of_stack_inner` must PEEK its entry, never pop it. Popping first is \
         `OOS-DX25c-6`: `stack_registry::stack_index_for_announced_target` then returns None \
         for the resolving object and Misdirection's 2004-10-04 ruling becomes \
         unimplementable. Offending receivers: {offenders:?}"
    );
    // Non-vacuity: the function really was found and really does read the stack.
    assert!(
        body.contains("stack_objects"),
        "r1 non-vacuity: the extracted body must mention stack_objects -- an empty or wrong \
         extraction would make this gate pass forever"
    );
    assert!(
        body.contains(".back()"),
        "r1 non-vacuity: the extracted body must contain the `.back()` PEEK that replaced the \
         pop"
    );
}

/// Every `X.pop_back(` / `X.remove(` / `X.pop_front(` whose receiver textually ends in
/// `stack_objects` (field access OR a `let`-bound alias one statement earlier), over
/// comment-stripped source. Over-collects on purpose: a false positive can only make a gate
/// redder.
fn pop_receivers(stripped: &str) -> Vec<String> {
    let normalised = stripped.replace("stack_objects()", "stack_objects");
    let mut hits = Vec::new();
    for m in [".pop_back(", ".pop_front("] {
        let mut from = 0usize;
        while let Some(rel) = normalised[from..].find(m) {
            let at = from + rel;
            let start = at.saturating_sub(120);
            let window: String = normalised[start..at]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if window.replace(' ', "").ends_with("stack_objects")
                || window.contains("stack_objects")
            {
                hits.push(format!("{window}{m}"));
            }
            from = at + m.len();
        }
    }
    hits
}

#[test]
fn r1b_pop_detector_fires_on_synthetic_violations() {
    // Direct.
    let direct =
        strip_comments("fn f(state: &mut GameState) { let x = state.stack_objects.pop_back(); }");
    assert!(
        !pop_receivers(&direct).is_empty(),
        "r1b: the detector must fire on the DIRECT `state.stack_objects.pop_back()` -- the \
         exact pre-PB-DX54 shape"
    );
    // Aliased -- PB-DX51's `OOS-DX51-6` defeat shape, one statement of indirection.
    let aliased = strip_comments(
        "fn f(state: &mut GameState) { let v = &mut state.stack_objects; let x = v.pop_back(); }",
    );
    assert!(
        !pop_receivers(&aliased).is_empty(),
        "r1b: the detector must fire on the ALIASED pop (`let v = &mut state.stack_objects; \
         v.pop_back()`) -- `OOS-DX51-6` defeated three successive drafts of a same-shape gate \
         with exactly this"
    );
    // And it must not fire on an unrelated pop.
    let unrelated = strip_comments("fn f(v: &mut Vec<u8>) { let _ = v.pop_back(); }");
    assert!(
        pop_receivers(&unrelated).is_empty(),
        "r1b: the detector must NOT fire on a pop of something that is not the stack -- \
         otherwise r1 is `any pop anywhere` and measures nothing"
    );
}

/// r2: the departure happens BEFORE the CR 704.3 SBA check and the CR 117.3b priority grant,
/// at EVERY site inside `resolve_top_of_stack_inner` that runs one.
///
/// **This is the gate that pins the design decision the whole suite could not refute.** The
/// stage-0 measurement moved the departure to the FUNCTION BOUNDARY and every behavioural test
/// in the workspace stayed green — while two SBAs read `state.stack_objects` and would have
/// answered differently:
///
/// * **CR 714.4** (`sba.rs`, Saga sacrifice): *"…and it isn't the source of a chapter ability
///   that has triggered but not yet left the stack"*. CR 704.3 checks SBAs when a player would
///   receive priority, i.e. AFTER CR 608.2n — so a resolving FINAL chapter ability that had not
///   departed would postpone its own Saga's sacrifice by a whole SBA round.
/// * **CR 309.6** (`sba.rs`, dungeon removal): the same shape for a `RoomAbility`.
///
/// `r5` pins that those two are still the only two such readers.
///
/// # DISCLOSURE: this gate is currently the ONLY thing that catches the wrong design
///
/// Revert row **R2** (delete the two inner departure calls, keep the wrapper backstop — i.e.
/// exactly the function-boundary design) reddens **this test and nothing else in the
/// workspace**. No behavioural probe moves. That is `OOS-DX52-2`'s shape stated out loud: *a
/// row that reddens only a source gate is telling you the behaviour has no probe*, and a later
/// batch that "simplifies" the two calls into one at the function boundary would satisfy every
/// other test in this tree.
///
/// **The behavioural probe is not merely missing, it is currently unbuildable, and that has its
/// own filed cause.** The property R2 breaks is CR 714.4's *"…hasn't yet left the stack"*
/// exemption for a FINAL chapter — and `OOS-DX54-4` is that the engine never reaches that
/// exemption correctly anyway: `enter_step` queues the chapter trigger, runs SBAs, and only
/// then flushes, while `sba.rs`'s guard scans `state.stack_objects` alone. So a Saga is already
/// sacrificed one mechanism early for an unrelated reason, and no fixture can isolate the
/// departure-point property until that is fixed. Four alternative constructions were considered
/// and rejected; they are recorded in the module doc of
/// `crates/engine/tests/primitives/pb_dx54_resolving_entry_target_space.rs`.
///
/// Revert row **R3** (delete the wrapper backstop, keep both inner calls) likewise reddens only
/// this test's exact-count assertion. The backstop's necessity rests on code reading rather
/// than on a probe: four paths return from `resolve_top_of_stack_inner` before either
/// CR-ordered departure — three ability-fizzle / intervening-if returns (Evolve's "no target
/// recorded", Offspring's CR 603.4 re-check, Gift's CR 603.4 re-check) and the CR 608.2d
/// suspension, which the wrapper's own state restore covers. The three ability paths would
/// leave the entry on the stack with no priority granted, which is a stuck game — and **no test
/// in this workspace drives any of them**, which is filed as `OOS-DX54-5`.
#[test]
fn r2_departure_precedes_every_sba_and_priority_site_in_the_resolution() {
    let stripped = strip_comments(&read_source(RESOLUTION_RS));
    let body = fn_body(&stripped, "resolve_top_of_stack_inner")
        .expect("r2 fail-closed: fn resolve_top_of_stack_inner not found");
    let unordered = unordered_tail_sites(&body);
    assert!(
        unordered.is_empty(),
        "r2: CR 608.2n then CR 608.2p then CR 704.3 then CR 117.3b, in that order. Every \
         `check_and_apply_sbas(` / `grant_priority_to_active_player(` inside \
         resolve_top_of_stack_inner must be preceded by a `depart_resolving_stack_entry(` \
         call. (The warrant is CR 608.2n -- NOT CR 608.2m, which is about an object removed \
         by something ELSE mid-resolution.) Sites with no departure before them: {unordered:?}"
    );
    assert_eq!(
        count(&body, "depart_resolving_stack_entry("),
        2,
        "r2 non-vacuity: exactly TWO departure calls are expected inside \
         resolve_top_of_stack_inner (the CR 608.2b fizzle tail and the main CR 608.2p/704.3 \
         tail). A third or a missing one means the tail structure changed -- re-derive this \
         count, do not adjust it to whatever the source says"
    );
    let outer = fn_body(&stripped, "resolve_top_of_stack")
        .expect("r2 fail-closed: fn resolve_top_of_stack not found");
    // `fn_body` finds the FIRST `fn resolve_top_of_stack(` -- the wrapper, since
    // `resolve_top_of_stack_inner` has a different signature prefix. The backstop lives there.
    assert_eq!(
        count(&outer, "depart_resolving_stack_entry("),
        1,
        "r2: the wrapper must carry exactly ONE backstop call, which is what discharges the \
         obligation for the four paths that return from the inner function before either \
         CR-ordered site (PB-DP8: a guard that returns early inherits the obligation of the \
         statements it skipped)"
    );
}

/// Offsets of `check_and_apply_sbas(` / `grant_priority_to_active_player(` in `body` that have
/// NO `depart_resolving_stack_entry(` occurrence at a smaller offset within the preceding
/// `WINDOW` bytes.
fn unordered_tail_sites(body: &str) -> Vec<String> {
    // MEASURED, not guessed, and the margin is measured too. In comment-stripped source the
    // four real sites sit 169 / 466 / 520 / **1,605** bytes after their own tail's departure
    // call, so 4,000 covers the widest by 2.5x. And it cannot let one tail vouch for the
    // other's site: the two departure calls are **464,693 bytes apart** (the fizzle tail at
    // +4,923, the main tail at +469,616), two orders of magnitude beyond this window. A
    // window chosen by taste rather than by that second measurement is the "over-wide gate
    // fails open" shape (`OOS-DX39-8`), so both numbers are recorded here rather than in a
    // memo.
    const WINDOW: usize = 4_000;
    let mut bad = Vec::new();
    for needle in ["check_and_apply_sbas(", "grant_priority_to_active_player("] {
        let mut from = 0usize;
        while let Some(rel) = body[from..].find(needle) {
            let at = from + rel;
            let start = at.saturating_sub(WINDOW);
            if !body[start..at].contains("depart_resolving_stack_entry(") {
                let ctx: String = body[start.max(at.saturating_sub(80))..at]
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                bad.push(format!("{needle} at +{at}, preceded by: ...{ctx}"));
            }
            from = at + needle.len();
        }
    }
    bad
}

#[test]
fn r2b_ordering_detector_fires_on_a_synthetic_boundary_departure() {
    // The stage-0 scaffold's shape: the departure moved OUT of the tail, so the SBA check runs
    // with the entry still on the stack.
    let boundary = strip_comments(
        "fn f() { let x = 1; let sba_events = sba::check_and_apply_sbas(state); \
         depart_resolving_stack_entry(state, id); }",
    );
    assert!(
        !unordered_tail_sites(&boundary).is_empty(),
        "r2b: the detector must fire when the departure comes AFTER the SBA check -- that is \
         the function-boundary design, which breaks CR 714.4 and CR 309.6"
    );
    let correct = strip_comments(
        "fn f() { depart_resolving_stack_entry(state, id); \
         let sba_events = sba::check_and_apply_sbas(state); }",
    );
    assert!(
        unordered_tail_sites(&correct).is_empty(),
        "r2b: the detector must NOT fire on the correct order -- otherwise r2 is unsatisfiable \
         and measures nothing"
    );
}

/// r3: the departure resolves its entry through the ONE shared arithmetic
/// (`stack_registry::stack_index_for_announced_target`) and re-open-codes no scan of its own.
///
/// This is `pb_dx52_stack_target_roster::r1a`'s rule, obeyed rather than allowlisted around.
/// Respelling the removal as `retain(|so| so.id != id)` would have satisfied `r1a`'s needle
/// while re-opening exactly the drift `OOS-DX25-3`/`OOS-SIM3-5` were — *a gate you edit prose
/// to satisfy has stopped measuring* (PB-DX52).
#[test]
fn r3_departure_routes_through_the_shared_lookup() {
    let stripped = strip_comments(&read_source(RESOLUTION_RS));
    let body = fn_body(&stripped, "depart_resolving_stack_entry")
        .expect("r3 fail-closed: fn depart_resolving_stack_entry not found");
    assert!(
        body.contains("stack_index_for_announced_target"),
        "r3: the departure must resolve its entry through \
         state::stack_registry::stack_index_for_announced_target -- the one shared resolution \
         of \"which stack entry does this id name\", also consumed by Effect::CounterSpell, \
         Effect::ChangeTargets, Effect::CopySpellOnStack and casting.rs's two single-target \
         validators. Body was: {body}"
    );
    for forbidden in [".position(", ".find(", ".retain(", ".iter()"] {
        assert!(
            !body.contains(forbidden),
            "r3: `{forbidden}` re-open-codes the lookup inside \
             depart_resolving_stack_entry. Route through \
             stack_index_for_announced_target instead. Body was: {body}"
        );
    }
}

/// r5: the CONSUMER ROSTER behind `r2`'s ordering argument.
///
/// `r2` pins the departure BEFORE the SBA check. The reason that matters is that
/// `check_and_apply_sbas` reads `state.stack_objects` at exactly two decision sites — CR 714.4
/// (Saga sacrifice) and CR 309.6 (dungeon removal). If a THIRD appears, the ordering argument
/// in `depart_resolving_stack_entry`'s doc is no longer a complete account of what the
/// ordering buys, and must be re-derived rather than assumed to still hold.
///
/// Pinned by COUNT **and** by the two CR cites, so a third site cannot be smuggled in by
/// deleting one of the two.
#[test]
fn r5_sba_reads_the_stack_at_exactly_the_two_sites_the_ordering_argument_names() {
    let raw = read_source(SBA_RS);
    let stripped = strip_comments(&raw);
    let normalised = stripped.replace("stack_objects()", "stack_objects");
    // Counted on the RECEIVER, not on one call shape. `stack_objects.iter()` would miss
    // `for so in &state.stack_objects`, `.get(..)`, `.len()` and every other way a third
    // reader could arrive -- and a gate that counts one spelling measures that spelling
    // (`OOS-DX26` -> `OOS-DX43` -> `OOS-DX45` -> `OOS-DX47` -> `OOS-DX51`, five batches).
    let reads = count(&normalised, "stack_objects");
    assert_eq!(
        reads, 2,
        "r5: `sba.rs` must MENTION state.stack_objects at exactly 2 places, both decision \
         sites (CR 714.4 \
         Saga sacrifice, CR 309.6 dungeon removal). Found {reads}. These are the ONLY two \
         readers that make PB-DX54's departure ORDER observable; a third means \
         `depart_resolving_stack_entry`'s doc no longer accounts for what the ordering buys \
         and must be re-derived -- do not simply update this number"
    );
    for cite in ["CR 714.4", "CR 309.6"] {
        assert!(
            raw.contains(cite),
            "r5: `sba.rs` must still carry the {cite} cite naming one of the two stack \
             readers -- if it is gone, the count above may be 2 for a different reason"
        );
    }
}

// ════════════════════════════════════════════════════════════════════════════════════════
// §B — the rider (`OOS-DX25-4`), BEHAVIOURALLY
// ════════════════════════════════════════════════════════════════════════════════════════

/// A synthetic planeswalker with one targeted `-2` loyalty ability, so a real
/// `Command::ActivateLoyaltyAbility` can put a genuine `StackObjectKind::LoyaltyAbility` entry
/// on the stack. **No hand-built `StackObject` anywhere in this file** — `LoyaltyAbility` is
/// one of the 23 kinds the old two-arm `match` fell through, and building it by hand would
/// have proved the classification on a shape no production path can produce (the
/// `ObjectSpec::card()`-is-naked gotcha, `OOS-DX47-4`).
fn rider_walker_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("pb-dx54-rider-walker".to_string()),
        name: "PB-DX54 Rider Walker".to_string(),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Planeswalker],
            ..Default::default()
        },
        oracle_text: "PB-DX54 Rider Walker: -2: Destroy target creature.".to_string(),
        abilities: vec![AbilityDefinition::LoyaltyAbility {
            cost: LoyaltyCost::Minus(2),
            effect: Effect::DestroyPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
        }],
        ..Default::default()
    }
}

/// Drive a REAL loyalty activation and return `(state, walker_id, entry_id)`.
fn state_with_a_real_loyalty_ability_on_the_stack() -> (GameState, ObjectId, ObjectId) {
    let (p1, p2) = (p(1), p(2));
    let registry = CardRegistry::new(vec![rider_walker_def()]);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "PB-DX54 Rider Walker")
                .with_card_id(CardId("pb-dx54-rider-walker".to_string()))
                .with_types(vec![CardType::Planeswalker])
                .with_counter(CounterType::Loyalty, 6)
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::creature(p2, "PB-DX54 Rider Victim", 2, 2))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let walker = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "PB-DX54 Rider Walker")
        .expect("the walker fixture must exist")
        .id;
    let victim = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "PB-DX54 Rider Victim")
        .expect("the victim fixture must exist")
        .id;

    let (state, _events) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: walker,
            ability_index: 0,
            targets: vec![Target::Object(victim)],
            x_value: None,
        },
    )
    .expect("activating the -2 with a legal target must succeed (CR 606.3)");

    let entry = state
        .stack_objects()
        .iter()
        .find(|so| matches!(so.kind, StackObjectKind::LoyaltyAbility { .. }))
        .expect("the activation must have pushed a LoyaltyAbility entry")
        .id;
    (state, walker, entry)
}

/// r4 (rider `OOS-DX25-4`): `resolution::counter_stack_object` names a source for a kind
/// OUTSIDE the two the old two-arm `match` handled.
///
/// Before this batch both counter paths carried a byte-identical
/// `ActivatedAbility | TriggeredAbility => Some(..), _ => None` — so countering any of the
/// other 23 kinds removed the entry and reported NOTHING to the event log. PB-DX48 made four
/// of them (`ForecastAbility`, `ScavengeAbility`, `LoyaltyAbility`, `KeywordTrigger`)
/// reachable from a real Ward trigger, so the silence was live rather than theoretical.
///
/// **Behavioural, not a source gate, on purpose** (`OOS-DX52-2`: a row that reddens only a
/// source gate is telling you the behaviour has no probe).
#[test]
fn r4_counter_stack_object_names_a_source_for_a_loyalty_ability() {
    let (mut state, walker, entry) = state_with_a_real_loyalty_ability_on_the_stack();

    let events = mtg_engine::rules::resolution::counter_stack_object(&mut state, entry)
        .expect("countering a live stack entry must succeed");

    let named: Vec<ObjectId> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::SpellCountered {
                stack_object_id,
                source_object_id,
                ..
            } if *stack_object_id == entry => Some(*source_object_id),
            _ => None,
        })
        .collect();
    assert_eq!(
        named,
        vec![walker],
        "rider OOS-DX25-4 / CR 113.7: countering a LoyaltyAbility must emit exactly one \
         SpellCountered naming the ability's SOURCE (the planeswalker). Before this batch the \
         two-arm match fell through `_ => None` and emitted nothing at all -- a player watched \
         their loyalty ability vanish with no line in the feed. Events: {events:?}"
    );
    // Non-vacuity floor: the entry really was removed, so this is not "nothing happened".
    assert!(
        !state.stack_objects().iter().any(|so| so.id == entry),
        "r4 non-vacuity: the countered entry must actually be off the stack"
    );
}

/// r4b: the SAME property on the OTHER counter path, `Effect::CounterSpell`.
///
/// The two paths carried byte-identical copies of the defect, which is why the row's fix shape
/// says *"consumed by both paths"* — a probe on one proves nothing about the other.
/// `Effect::CounterSpell` is the PRODUCTION path; `counter_stack_object`'s own doc says it is
/// not (its only callers are tests).
#[test]
fn r4b_effect_counterspell_names_a_source_for_a_loyalty_ability() {
    let (mut state, walker, entry) = state_with_a_real_loyalty_ability_on_the_stack();

    let mut ctx = EffectContext::new(
        p(2),
        walker,
        vec![SpellTarget {
            // CR 702.21a: the stack ENTRY's own id -- the id space Ward already passes here,
            // resolved by `stack_index_for_announced_target`'s first clause.
            target: Target::Object(entry),
            zone_at_cast: Some(ZoneId::Stack),
        }],
    );
    let effect = Effect::CounterSpell {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        exile_instead: false,
    };
    let events = execute_effect(&mut state, &effect, &mut ctx);

    let named: Vec<ObjectId> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::SpellCountered {
                stack_object_id,
                source_object_id,
                ..
            } if *stack_object_id == entry => Some(*source_object_id),
            _ => None,
        })
        .collect();
    assert_eq!(
        named,
        vec![walker],
        "rider OOS-DX25-4 / CR 113.7: Effect::CounterSpell must ALSO name the source. Events: \
         {events:?}"
    );
    assert!(
        !state.stack_objects().iter().any(|so| so.id == entry),
        "r4b non-vacuity: the countered entry must actually be off the stack"
    );
}

// ════════════════════════════════════════════════════════════════════════════════════════
// §C — the census (acceptance criterion 7381)
// ════════════════════════════════════════════════════════════════════════════════════════

/// The three `TargetRequirement` variants AC 7381 names. A VICTIM declaring one of these is a
/// spell whose redirect candidate set is what PB-DX54 widened.
const DECLARED_NEEDLES: &[&str] = &[
    "TargetSpellWithSingleTarget",
    "TargetSpellOrAbilityWithSingleTarget",
    "TargetSpellOrAbility",
];

/// The INVERSE ORACLE axis — printed text, lowercased. A declared-axis census structurally
/// cannot see a card whose defect is that its ability is UNAUTHORED (`OOS-DX53`'s
/// `minas_tirith`), so the printed axis is not a nicety.
const PRINTED_NEEDLES: &[&str] = &[
    "change the target of target spell",
    "change the target of target",
    "choose new targets",
];

#[derive(Debug, Clone)]
struct CensusRow {
    name: String,
    declared: Vec<&'static str>,
    printed: Vec<&'static str>,
    completeness: &'static str,
}

fn completeness_tag(c: &Completeness) -> &'static str {
    match c {
        Completeness::Complete => "Complete",
        Completeness::Inert(_) => "Inert",
        Completeness::Partial(_) => "Partial",
        Completeness::KnownWrong(_) => "KnownWrong",
    }
}

fn printed_haystack(def: &CardDefinition) -> String {
    let mut s = def.oracle_text.to_lowercase();
    if let Some(f) = &def.back_face {
        s.push('\n');
        s.push_str(&f.oracle_text.to_lowercase());
    }
    if let Some(f) = &def.adventure_face {
        s.push('\n');
        s.push_str(&f.oracle_text.to_lowercase());
    }
    s
}

fn build_census(cards: &[CardDefinition]) -> Vec<CensusRow> {
    let mut rows = Vec::new();
    for def in cards {
        // `def_contains_variant`, NOT `format!("{def:#?}")`. `OOS-DX53-2`: a Debug render also
        // prints PROSE compiled into the def (a `Completeness::partial("... TargetSpell... ")`
        // note is a string LITERAL, not a comment), so a substring scan over it counts a
        // blocker note as a DECLARER. The mechanism that separates them is that
        // `def_contains_variant` matches a unit variant's serialized name EXACTLY -- it is the
        // exactness, not the `PROSE_FIELDS` denylist, which is never even consulted on a
        // sentence-shaped note.
        let declared: Vec<&'static str> = DECLARED_NEEDLES
            .iter()
            .copied()
            .filter(|n| def_contains_variant(def, n))
            .collect();
        let hay = printed_haystack(def);
        let printed: Vec<&'static str> = PRINTED_NEEDLES
            .iter()
            .copied()
            .filter(|n| hay.contains(n))
            .collect();
        if declared.is_empty() && printed.is_empty() {
            continue;
        }
        rows.push(CensusRow {
            name: def.name.clone(),
            declared,
            printed,
            completeness: completeness_tag(&def.completeness),
        });
    }
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    rows
}

/// r6: the census, PRINTED by the test rather than transcribed into a document (PB-DX8's rule,
/// which PB-DX35 then broke and had to correct). `--nocapture` shows the whole table.
#[test]
fn r6_census_report() {
    let cards = all_cards();
    assert!(
        cards.len() >= 1_700,
        "r6 non-vacuity: all_cards() must return at least 1,700 defs, got {} -- a broken \
         enumeration cannot make an empty census look correct",
        cards.len()
    );
    let rows = build_census(&cards);

    println!("═══ PB-DX54 census (AC 7381) ═══");
    println!("DECLARED axis {DECLARED_NEEDLES:?} ∪ PRINTED axis {PRINTED_NEEDLES:?}");
    println!("union = {}", rows.len());
    for r in &rows {
        println!(
            "  {:<26} declared={:?} printed={:?} completeness={}",
            r.name, r.declared, r.printed, r.completeness
        );
    }

    let declared_deck_legal: Vec<&CensusRow> = rows
        .iter()
        .filter(|r| !r.declared.is_empty() && r.completeness == "Complete")
        .collect();
    println!(
        "--- deck-legal `Complete` declarers ({}) ---",
        declared_deck_legal.len()
    );
    for r in &declared_deck_legal {
        println!("  {} {:?}", r.name, r.declared);
    }

    // ── The seed row's "2 deck-legal `Complete`" cell, REPRODUCED and given its reason ──
    //
    // The blind spot was never about which requirement the REDIRECTOR declares -- it is about
    // which requirement the VICTIM declares, because `plan_target_change` validates candidates
    // against `so.target_requirements`, the VICTIM's list. Only the two SINGLE-TARGET
    // requirements consult `state.stack_objects` (through
    // `stack_index_for_announced_target`); `TargetSpell`, `TargetSpellWithFilter` and
    // `TargetSpellOrAbility` all decide the object branch on `obj.zone == ZoneId::Stack`
    // alone, and the resolving spell's CARD never left `ZoneId::Stack`. That is why
    // `TargetSpellOrAbility` is a stated CONTROL in the probe file rather than a third
    // subject.
    let single_target: BTreeSet<&str> = rows
        .iter()
        .filter(|r| {
            r.completeness == "Complete"
                && (r.declared.contains(&"TargetSpellWithSingleTarget")
                    || r.declared.contains(&"TargetSpellOrAbilityWithSingleTarget"))
        })
        .map(|r| r.name.as_str())
        .collect();
    assert_eq!(
        single_target,
        ["Bolt Bend", "Misdirection"]
            .into_iter()
            .collect::<BTreeSet<&str>>(),
        "r6: `OOS-DX25c-6` scopes itself to 2 deck-legal `Complete` defs, and this is the \
         reproduction of that claim. It HOLDS -- unlike the last four batches in this queue, \
         whose yield cells were floors. The third single-target declarer, `Untimely \
         Malfunction`, is `Partial` (its mode-1 half is `TargetSpellOrAbilityWithSingleTarget` \
         behind an `UpToN` gap), so its deck-legal exposure is zero. Found: {single_target:?}"
    );
    assert!(
        rows.iter().any(|r| r.name == "Untimely Malfunction"
            && r.completeness == "Partial"
            && r.declared.contains(&"TargetSpellOrAbilityWithSingleTarget")),
        "r6: `Untimely Malfunction` must still be the `Partial` third single-target declarer \
         -- if it is promoted, the deck-legal class becomes 3 and this batch's own \
         '2 deck-legal' statement needs re-deriving, not re-numbering"
    );
    assert!(
        rows.iter().any(|r| r.name == "Deflecting Swat"
            && r.completeness == "Complete"
            && r.declared.contains(&"TargetSpellOrAbility")),
        "r6: `Deflecting Swat` must still be the deck-legal `Complete` declarer of the \
         UNAFFECTED third requirement -- it is what makes `TargetSpellOrAbility` a real \
         control rather than a hypothetical one, and its `must_change: false` no-op is the \
         still-open `OOS-DX25b-4`"
    );

    // The two axes DO NOT NEST, and saying so is the point (PB-DX26 -> PB-DX43 -> PB-DX35 ->
    // PB-DX53, four batches running). A declared-axis census cannot see a printed redirect
    // whose ability is unauthored, and a printed-text axis cannot see a def that declares one
    // of these requirements without printing the phrase (every counterspell-shaped victim).
    let declared_only = rows.iter().filter(|r| r.printed.is_empty()).count();
    let printed_only = rows.iter().filter(|r| r.declared.is_empty()).count();
    println!("declared-only = {declared_only} | printed-only = {printed_only}");
    assert!(
        declared_only > 0 || printed_only > 0,
        "r6: if neither axis has an exclusive member the two have collapsed into one \
         measurement, and the inverse-oracle axis has stopped earning its keep -- re-derive \
         the needles rather than deleting this assertion"
    );
}
