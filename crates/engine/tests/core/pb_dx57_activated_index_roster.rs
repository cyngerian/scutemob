//! PB-DX57 (`scutemob-236`) — `OOS-DX26-3`: **an ability index is positional, so
//! authoring an ability into a def silently renumbers that def's other
//! abilities, and nothing gated the class.**
//!
//! `Command::ActivateAbility { ability_index }` and `Command::TapForMana {
//! ability_index }` are bare `usize`s into two runtime vectors. PB-DX26 hit
//! this live: its first pass inserted Umezawa's Jitte's new Equip {2} beside
//! the `KeywordAbility::Equip` marker at the HEAD of the `abilities` vec, which
//! moved the PB-EF7 modal counter-removal ability from activated index **0 to
//! 1** — the index golden scripts and `pb_os10_singleton_cleanup.rs` already
//! name. It was caught only because that batch happened to write a probe
//! pinning the order explicitly. The safe rule (*append, never insert*) was a
//! convention held in one def's comment; this file makes it a checked property.
//!
//! # The index space is the LOWERED list, not `AbilityDefinition::Activated`
//!
//! `OOS-DX26-3`'s own prescription — *"pin the order of a def's
//! `AbilityDefinition::Activated` entries"* — names the wrong list, and
//! building it that way would have produced a gate that reports success while
//! checking something other than what it claims. Three facts, each verified at
//! its site:
//!
//! 1. `rules::abilities::handle_activate_ability` resolves
//!    `expect_characteristics(state, source).activated_abilities.get(ability_index)`
//!    — the **layer-resolved `Characteristics::activated_abilities`** vector.
//! 2. `testing::replay_harness::build_face_ability_vectors` builds that vector,
//!    and it **skips every ability for which `mana_ability_lowering` returns
//!    `Some`** (CR 605.1a; those go to `mana_abilities`, indexed by
//!    `Command::TapForMana`). Its own comment says so: *"Including it here too
//!    would shift ability_index for every non-mana activated ability that
//!    follows it (SF-6)."* So for any def carrying a mana ability, declaration
//!    position and activated index are DIFFERENT numbers.
//! 3. There are **two** index spaces per def, not one:
//!    `enrich_spec_from_def` lowers the FRONT face (`def.abilities`), while
//!    `rules::face::apply_face_change` and `rules::resolution`'s
//!    enters-transformed path both re-lower `def.effective_abilities(true)` —
//!    the **back** face. A front-face-only gate is blind to four corpus defs
//!    (see `r1`'s printed census).
//!
//! So this file pins the output of the **production lowering**, by CALLING it,
//! for every face. It does not re-implement the mana/non-mana split: four
//! batches in this queue have been burned by a second hand-rolled copy of an
//! engine predicate (`OOS-DX55-3` is the sharpest — nobody hand-rolls their way
//! to the exception list).
//!
//! # Why the assertion is a PREFIX and not an equality
//!
//! The convention this gate exists to enforce is *append, never insert*. An
//! equality pin cannot express it: it reddens on the SAFE operation (append)
//! exactly as loudly as on the unsafe one (insert), and a gate that cannot tell
//! the two apart is a gate the next author regenerates reflexively — at which
//! point it has stopped measuring. `r2` therefore asserts that each face's
//! CURRENT lowered list **starts with** its pinned list. Under that rule:
//!
//! | edit                                   | verdict | why                                   |
//! |----------------------------------------|---------|---------------------------------------|
//! | append at tail (the safe operation)    | GREEN   | pinned list is still a prefix         |
//! | insert at head or middle               | **RED** | prefix diverges at the insert point   |
//! | reorder two abilities                  | **RED** | prefix diverges at the first of them  |
//! | delete an ability                      | **RED** | prefix diverges, or current is short  |
//! | cross the mana/non-mana boundary       | **RED** | the ability leaves or joins a list    |
//! | a def gains its FIRST ability          | GREEN   | 0 → N cannot renumber anything        |
//!
//! A row is pinned for every face with **≥1** lowered ability of either kind,
//! not the ≥2 the criterion asks for. ≥2 alone cannot see a 1 → 2 INSERT, which
//! renumbers exactly as much as a 2 → 3 one; and today **all four** back faces
//! carrying an activated ability carry exactly one, so a ≥2 roster would have
//! covered zero of them.
//!
//! # The descriptor: SHAPE, not payload
//!
//! Each ability is reduced to a short descriptor derived from **typed lowered
//! values** — never from source text and never from a `format!("{def:#?}")`
//! render, so `OOS-DX53-2`'s hazard (a `Completeness::partial("... Effect::Foo
//! ...")` note is a compiled string literal, not a comment, and a Debug walk
//! counts it as a declaration) cannot reach this file at all.
//!
//! Sensitive to, by construction:
//! * **insert** / **reorder** / **delete** — the four rows above, each proven by
//!   an executed plant (see `memory/primitives/pb-DX57-*`);
//! * **mana ↔ non-mana boundary crossings** — the descriptor is computed from
//!   whichever list the lowering actually put the ability in.
//!
//! Insensitive to, deliberately:
//! * **comments and formatting** in a card def — nothing here reads source;
//! * **an effect's PAYLOAD** at an unchanged position: only the top-level
//!   `Effect` VARIANT NAME is taken, so `DealDamage { amount: 2 }` →
//!   `{ amount: 3 }` is green. The line is *shape, not payload*.
//!
//! One deliberate over-sensitivity, stated rather than hidden: an ability's
//! **cost** is part of its shape, so editing a cost in place reddens. That is
//! the price of being able to tell two abilities on one permanent apart —
//! cost is their primary discriminator — and `r3` is what makes that
//! discrimination a measured property rather than a hope.
//!
//! # Residuals this file does NOT cover
//!
//! * A swap of two abilities whose descriptors are EQUAL is invisible. `r3`
//!   measures that population and pins it at **zero**, so the residual is a
//!   ratchet rather than an unknown.
//! * Granted abilities. `handle_activate_ability` reads LAYER-RESOLVED
//!   characteristics, so a continuous effect adding an activated ability shifts
//!   nothing here (grants append) but is outside a static def walk entirely.
//! * `adventure_face` is **not** an index space: `effective_abilities` chooses
//!   between front and back only, and `r1` measures that **0** adventure faces
//!   carry an `AbilityDefinition::Activated` at all. If that ever becomes
//!   nonzero those abilities are unreachable, which `r1` will say out loud.
//! * The pin cannot distinguish an author who inserted from an author who
//!   inserted AND regenerated the pin. It makes the second one a visible diff.

use std::collections::{BTreeMap, BTreeSet};

use mtg_card_types::cards::card_definition::{AbilityDefinition, CardDefinition};
use mtg_card_types::state::game_object::{ActivatedAbility, ManaAbility};
use mtg_engine::all_cards;
use mtg_engine::testing::replay_harness::build_face_ability_vectors;

// ─────────────────────────────────────────────────────────────────────────────
// Descriptors
// ─────────────────────────────────────────────────────────────────────────────

/// Leading identifier of a `Debug` render — i.e. the enum VARIANT NAME, with the
/// payload discarded. This is the whole of the "shape, not payload" rule.
fn variant_name(dbg: &str) -> String {
    dbg.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Six hex characters of a blake3 digest. Used only to fold a `Debug`-rendered
/// COST into the descriptor without making every row 300 bytes wide; the human
/// half of the descriptor carries the flags a reader actually wants, and a
/// failure message prints the full `{:#?}` of the offending ability anyway.
fn h6(s: &str) -> String {
    blake3::hash(s.as_bytes()).to_hex()[..6].to_string()
}

fn activated_descriptor(ab: &ActivatedAbility) -> String {
    let c = &ab.cost;
    let mut cost_sig = String::new();
    if c.requires_tap {
        cost_sig.push('T');
    }
    if c.sacrifice_self {
        cost_sig.push('S');
    }
    if c.mana_cost.is_some() {
        cost_sig.push('M');
    }
    // The digest covers EVERY `ActivationCost` field, including ones added
    // later: it is taken over the struct's own `Debug`, not over a hand-listed
    // field set that would silently stop covering a new field (`OOS-DX20b-2`).
    cost_sig.push('#');
    cost_sig.push_str(&h6(&format!("{c:?}")));

    let eff = ab
        .effect
        .as_ref()
        .map(|e| variant_name(&format!("{e:?}")))
        .unwrap_or_else(|| "NoEffect".to_string());

    let mut flags = String::new();
    if ab.sorcery_speed {
        flags.push('s');
    }
    if ab.once_per_turn {
        flags.push('o');
    }
    if ab.modes.is_some() {
        flags.push('m');
    }
    if ab.activation_condition.is_some() {
        flags.push('c');
    }
    if let Some(z) = &ab.activation_zone {
        flags.push('z');
        flags.push_str(&variant_name(&format!("{z:?}")));
    }
    format!("A:{cost_sig}|{eff}|t{}|{flags}", ab.targets.len())
}

fn mana_descriptor(ma: &ManaAbility) -> String {
    let mut flags = String::new();
    if ma.requires_tap {
        flags.push('T');
    }
    if ma.sacrifice_self {
        flags.push('S');
    }
    if ma.any_color {
        flags.push('*');
    }
    if ma.mana_cost.is_some() {
        flags.push('M');
    }
    // Same reason as the activated cost digest: the whole struct, not a field
    // list this file would have to remember to extend.
    format!("M:{flags}#{}", h6(&format!("{ma:?}")))
}

/// The two faces that `effective_abilities` can select, in the order
/// `enrich_spec_from_def` / `apply_face_change` reach them. `adventure_face` is
/// deliberately absent — see the module doc.
fn faces(def: &CardDefinition) -> Vec<(&'static str, &[AbilityDefinition])> {
    let mut v: Vec<(&'static str, &[AbilityDefinition])> = vec![("F", &def.abilities)];
    if let Some(b) = def.back_face.as_ref() {
        v.push(("B", &b.abilities));
    }
    v
}

/// `(name, face, mana descriptors, activated descriptors)` for every face
/// carrying at least one lowered ability of either kind, ordered by name then
/// face. This is the shape `PINNED_ORDER` stores.
type Row = (String, &'static str, Vec<String>, Vec<String>);

/// A row indexed for lookup: `(name, face) -> (mana descriptors, activated
/// descriptors)`.
type RowIndex = BTreeMap<(String, &'static str), (Vec<String>, Vec<String>)>;

fn current_rows() -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    for def in all_cards() {
        for (tag, abs) in faces(&def) {
            let (mana, activated, _) = build_face_ability_vectors(abs);
            if mana.is_empty() && activated.is_empty() {
                continue;
            }
            rows.push((
                def.name.clone(),
                tag,
                mana.iter().map(mana_descriptor).collect(),
                activated.iter().map(activated_descriptor).collect(),
            ));
        }
    }
    rows.sort_by(|a, b| (&a.0, a.1).cmp(&(&b.0, b.1)));
    rows
}

// ─────────────────────────────────────────────────────────────────────────────
// R1 — the census, PRINTED (PB-DX8's rule: publish the figure, never transcribe)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// R1: the measured population this gate covers, printed by the test itself.
/// Carries the non-vacuity floors for everything below: an `all_cards()` that
/// returned nothing, or a lowering that produced nothing, must FAIL here rather
/// than let `r2` pass over an empty roster.
fn r1_index_space_census() {
    let cards = all_cards();
    assert!(
        cards.len() >= 1000,
        "PB-DX57 r1 non-vacuity: all_cards() returned {} defs",
        cards.len()
    );

    let mut act_hist: BTreeMap<usize, usize> = BTreeMap::new();
    let mut mana_hist: BTreeMap<usize, usize> = BTreeMap::new();
    let mut back_faces = 0usize;
    let mut back_faces_with_activated: Vec<String> = Vec::new();
    let mut activated_ge2: Vec<String> = Vec::new();
    let mut adventure_faces_with_activated: Vec<String> = Vec::new();

    for def in &cards {
        if let Some(adv) = def.adventure_face.as_ref() {
            if adv
                .abilities
                .iter()
                .any(|a| matches!(a, AbilityDefinition::Activated { .. }))
            {
                adventure_faces_with_activated.push(def.name.clone());
            }
        }
        if def.back_face.is_some() {
            back_faces += 1;
        }
        for (tag, abs) in faces(def) {
            let (mana, activated, _) = build_face_ability_vectors(abs);
            *act_hist.entry(activated.len()).or_default() += 1;
            *mana_hist.entry(mana.len()).or_default() += 1;
            if tag == "B" && !activated.is_empty() {
                back_faces_with_activated.push(format!("{} (act {})", def.name, activated.len()));
            }
            if activated.len() >= 2 {
                activated_ge2.push(format!("{} [{}] x{}", def.name, tag, activated.len()));
            }
        }
    }

    let rows = current_rows();
    println!("PB-DX57 R1 — index-space census (OOS-DX26-3)");
    println!(
        "  defs                                     : {}",
        cards.len()
    );
    println!("  defs with a back face                    : {back_faces}");
    println!(
        "  faces walked                             : {}",
        cards.len() + back_faces
    );
    println!("  activated-list length histogram          : {act_hist:?}");
    println!("  mana-list length histogram               : {mana_hist:?}");
    println!(
        "  PINNED rows (>=1 lowered ability)        : {}",
        rows.len()
    );
    println!(
        "  faces with >=2 activated (reorderable)   : {}",
        activated_ge2.len()
    );
    for m in &activated_ge2 {
        println!("      {m}");
    }
    println!(
        "  BACK faces with >=1 activated            : {}",
        back_faces_with_activated.len()
    );
    for m in &back_faces_with_activated {
        println!("      {m}");
    }
    println!(
        "  adventure faces carrying an Activated    : {} {:?}",
        adventure_faces_with_activated.len(),
        adventure_faces_with_activated
    );

    // Floors. Each is a FLOOR, not a pin: the populations may legitimately grow.
    assert!(
        rows.len() >= 400,
        "PB-DX57 r1 non-vacuity: only {} pinned rows -- the lowering produced almost \
         nothing, so r2 would be asserting over an empty roster",
        rows.len()
    );
    assert!(
        activated_ge2.len() >= 15,
        "PB-DX57 r1 non-vacuity: only {} faces carry >=2 activated abilities. r2's \
         REORDER sensitivity is unexercised below 2, so this floor is what keeps the \
         gate from going quietly vacuous",
        activated_ge2.len()
    );
    assert!(
        !back_faces_with_activated.is_empty(),
        "PB-DX57 r1 non-vacuity: no BACK face carries an activated ability, so the \
         second index space (rules::face::apply_face_change) is unexercised"
    );
    assert!(
        adventure_faces_with_activated.is_empty(),
        "PB-DX57 r1: {:?} carry an AbilityDefinition::Activated on their adventure_face. \
         `CardDefinition::effective_abilities` selects between the FRONT and BACK faces \
         only, so nothing ever lowers an adventure face -- those abilities are \
         unreachable from Command::ActivateAbility, and this file's roster does not \
         cover them. Widen `faces()` (and re-pin) or move the ability",
        adventure_faces_with_activated
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R2 — the gate: every pinned face's lowered order is an append-only extension
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// R2 (`OOS-DX26-3`): for every pinned face, the CURRENT lowered mana and
/// activated lists must each START WITH the pinned list. Appending at the tail
/// is green; inserting, reordering, deleting, or moving an ability across the
/// CR 605.1a mana boundary is red, and the message names the def.
///
/// Regenerate after a legitimate change with:
/// ```text
/// PB_DX57_REGEN=1 ~/.cargo/bin/cargo test -p mtg-engine --test core \
///     pb_dx57_activated_index_roster::r2 -- --nocapture --ignored
/// ```
/// (the `r2_regenerate_pin` sibling below prints the table in source form).
/// **Read the diff before pasting it.** A regeneration that moves an existing
/// position is the defect this gate exists to make visible.
fn r2_lowered_ability_order_is_append_only() {
    let rows = current_rows();
    let current: RowIndex = rows
        .into_iter()
        .map(|(n, f, m, a)| ((n, f), (m, a)))
        .collect();

    assert!(
        !PINNED_ORDER.is_empty(),
        "PB-DX57 r2 non-vacuity: PINNED_ORDER is empty"
    );
    assert!(
        PINNED_ORDER
            .iter()
            .filter(|(_, _, _, a)| a.len() >= 2)
            .count()
            >= 15,
        "PB-DX57 r2 non-vacuity: fewer than 15 pinned faces carry >=2 activated \
         abilities, so the pin has stopped constraining any ORDER"
    );

    let mut failures: Vec<String> = Vec::new();

    for (name, face, pin_mana, pin_act) in PINNED_ORDER {
        let Some((cur_mana, cur_act)) = current.get(&(name.to_string(), *face)) else {
            failures.push(format!(
                "`{name}` [{face}]: pinned face has NO lowered ability at all now. Either \
                 the def was renamed/removed, or every ability it declares stopped \
                 lowering. Both invalidate every `ability_index` naming this card."
            ));
            continue;
        };
        for (kind, pinned, cur) in [
            ("mana (Command::TapForMana)", *pin_mana, cur_mana),
            ("activated (Command::ActivateAbility)", *pin_act, cur_act),
        ] {
            if cur.len() < pinned.len() {
                failures.push(format!(
                    "`{name}` [{face}] {kind}: list SHRANK {} -> {}. Deleting a lowered \
                     ability renumbers every later one.\n     pinned : {pinned:?}\n     \
                     now    : {cur:?}",
                    pinned.len(),
                    cur.len()
                ));
                continue;
            }
            if let Some(i) = (0..pinned.len()).find(|i| pinned[*i] != cur[*i]) {
                failures.push(format!(
                    "`{name}` [{face}] {kind}: index {i} CHANGED, so every ability_index \
                     >= {i} on this card now names a different ability.\n     pinned[{i}] \
                     : {}\n     now[{i}]    : {}\n     pinned : {pinned:?}\n     now    : \
                     {cur:?}\n     If you APPENDED at the tail this test would be green -- \
                     it is red because something moved. Insert-at-head is the PB-DX26 \
                     defect (`OOS-DX26-3`).",
                    pinned[i], cur[i]
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "PB-DX57 r2 (OOS-DX26-3): {} pinned face(s) had a lowered ability list that is no \
         longer an append-only extension of its pin.\n\n{}\n\nAn `ability_index` is \
         positional over the LOWERED list \
         (`testing::replay_harness::build_face_ability_vectors`), which is not the same \
         list as the def's `AbilityDefinition::Activated` entries -- a mana ability is \
         lowered into `mana_abilities` and skipped here (CR 605.1a / SF-6). Golden \
         scripts, `pb_os10_singleton_cleanup` and any stored command trace name these \
         indices.",
        failures.len(),
        failures.join("\n   - ")
    );
}

#[test]
#[ignore = "regeneration helper, not a check: run with PB_DX57_REGEN=1 and --ignored"]
/// Prints `PINNED_ORDER` in source form. Refuses to run without the env var so
/// a bare `--include-ignored` sweep cannot mistake it for a passing check.
fn r2_regenerate_pin() {
    assert!(
        std::env::var("PB_DX57_REGEN").is_ok(),
        "set PB_DX57_REGEN=1 to regenerate"
    );
    println!("const PINNED_ORDER: &[(&str, &str, &[&str], &[&str])] = &[");
    for (name, face, mana, act) in current_rows() {
        let f = |v: &Vec<String>| {
            if v.is_empty() {
                "&[]".to_string()
            } else {
                format!(
                    "&[{}]",
                    v.iter()
                        .map(|s| format!("{s:?}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        };
        println!("    ({:?}, {:?}, {}, {}),", name, face, f(&mana), f(&act));
    }
    println!("];");
}

// ─────────────────────────────────────────────────────────────────────────────
// R3 — the reorder-blindness residual, measured and ratcheted at zero
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// R3: `r2` detects a REORDER only when the two swapped abilities have
/// different descriptors. This measures the population where they do not — the
/// faces holding two positions whose descriptors are EQUAL — and pins it at
/// **zero**. Without this row, "r2 catches reorders" would be a claim about the
/// corpus that nothing re-checks (`OOS-DX36-8`: a residual you do not measure is
/// a residual you have assumed away).
fn r3_no_face_holds_two_identical_descriptors() {
    let rows = current_rows();
    assert!(!rows.is_empty(), "PB-DX57 r3 non-vacuity: no rows");

    let mut blind: Vec<String> = Vec::new();
    let mut compared = 0usize;
    for (name, face, mana, act) in &rows {
        for (kind, list) in [("mana", mana), ("activated", act)] {
            if list.len() < 2 {
                continue;
            }
            compared += 1;
            let distinct: BTreeSet<&String> = list.iter().collect();
            if distinct.len() != list.len() {
                blind.push(format!("`{name}` [{face}] {kind}: {list:?}"));
            }
        }
    }
    println!("PB-DX57 R3 — faces with >=2 entries in a list: {compared}");
    println!(
        "             reorder-blind faces            : {}",
        blind.len()
    );
    assert!(
        compared >= 100,
        "PB-DX57 r3 non-vacuity: only {compared} lists have >=2 entries, so this row \
         compared almost nothing"
    );
    assert!(
        blind.is_empty(),
        "PB-DX57 r3: {} face(s) hold two lowered abilities with IDENTICAL descriptors, so \
         `r2` cannot see a swap between them:\n   - {}\n\nEither the two abilities really \
         are interchangeable (say so here and allowlist it) or the descriptor in this file \
         is too coarse and needs another SHAPE field -- not a payload field, which would \
         make every in-place edit red.",
        blind.len(),
        blind.join("\n   - ")
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R4 — the in-corpus consumer of the activated index
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// R4: `CardDefinition::activated_ability_cost_reductions` is keyed by
/// activated-ability index — *"0 = first activated ability in
/// `characteristics.activated_abilities`"*, per its own doc. That makes it a
/// consumer of this index space living INSIDE the card corpus, which neither
/// `OOS-DX26-3` nor any other document names, and it is the one consumer a
/// card-def-only batch is most likely to invalidate without touching a line of
/// engine source. Every key must be in range for its def's FRONT-face lowered
/// list (CR 602.2b / CR 601.2f).
fn r4_activated_cost_reduction_keys_are_in_range() {
    let cards = all_cards();
    let mut bad: Vec<String> = Vec::new();
    let mut checked = 0usize;
    let mut carriers: Vec<String> = Vec::new();
    for def in &cards {
        if def.activated_ability_cost_reductions.is_empty() {
            continue;
        }
        carriers.push(def.name.clone());
        let (_, activated, _) = build_face_ability_vectors(&def.abilities);
        for (idx, _) in &def.activated_ability_cost_reductions {
            checked += 1;
            if *idx >= activated.len() {
                bad.push(format!(
                    "`{}`: cost-reduction key {idx} but the front face lowers only {} \
                     activated ability/ies {:?}",
                    def.name,
                    activated.len(),
                    activated
                        .iter()
                        .map(activated_descriptor)
                        .collect::<Vec<_>>()
                ));
            }
        }
    }
    println!(
        "PB-DX57 R4 — defs carrying activated_ability_cost_reductions: {} {:?}",
        carriers.len(),
        carriers
    );
    println!("             keys checked: {checked}");
    assert!(
        checked >= 1,
        "PB-DX57 r4 non-vacuity: no def carries an activated_ability_cost_reduction, so \
         this row asserted nothing. If the field was removed, delete this test with its \
         reason; do not leave it passing vacuously"
    );
    assert!(
        bad.is_empty(),
        "PB-DX57 r4: {} activated-ability cost-reduction key(s) are out of range for their \
         def's lowered activated list:\n   - {}\n\nThis is the same positional index \
         `Command::ActivateAbility` uses. A key that points past the end is silently \
         inert; a key that points at the WRONG ability discounts the wrong cost.",
        bad.len(),
        bad.join("\n   - ")
    );
}

// The pin. Regenerate with `r2_regenerate_pin` (see its doc); read the diff.
const PINNED_ORDER: &[(&str, &str, &[&str], &[&str])] = &[
    (
        "Abstergo Entertainment",
        "F",
        &["M:T#33c5df", "M:T*M#bd7593"],
        &[],
    ),
    (
        "Access Tunnel",
        "F",
        &["M:T#33c5df"],
        &["A:TM#1c92fd|ApplyContinuousEffect|t1|"],
    ),
    (
        "Accorder's Shield",
        "F",
        &[],
        &["A:M#56e994|AttachEquipment|t1|s"],
    ),
    (
        "Aetherflux Reservoir",
        "F",
        &[],
        &["A:#2fcd95|DealDamage|t1|"],
    ),
    (
        "Aggravated Assault",
        "F",
        &[],
        &["A:M#9dfb17|Sequence|t0|s"],
    ),
    (
        "Akroma, Angel of Fury",
        "F",
        &[],
        &["A:M#d68d56|ApplyContinuousEffect|t0|"],
    ),
    ("Altar of Dementia", "F", &[], &["A:#69bd6d|MillCards|t1|"]),
    ("Ancient Den", "F", &["M:T#988966"], &[]),
    ("Ancient Tomb", "F", &["M:T#5f3434"], &[]),
    ("Arbor Elf", "F", &[], &["A:T#a692a1|UntapPermanent|t1|"]),
    (
        "Arcane Sanctum",
        "F",
        &["M:T#988966", "M:T#a58da2", "M:T#906e31"],
        &[],
    ),
    ("Arcane Signet", "F", &["M:T*#bf1bd3"], &[]),
    (
        "Arcanis the Omnipotent",
        "F",
        &[],
        &["A:T#a692a1|DrawCards|t0|", "A:M#5c6a44|MoveZone|t0|"],
    ),
    (
        "Arch of Orazca",
        "F",
        &["M:T#33c5df"],
        &["A:TM#077d0c|Conditional|t0|"],
    ),
    ("Arena of Glory", "F", &["M:T#bb0d56"], &[]),
    (
        "Argentum Armor",
        "F",
        &[],
        &["A:M#be0952|AttachEquipment|t1|s"],
    ),
    ("Arid Mesa", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Arixmethes, Slumbering Isle", "F", &["M:T#9cc700"], &[]),
    ("Ashnod's Altar", "F", &[], &["A:#69bd6d|AddMana|t0|"]),
    ("Avacyn's Pilgrim", "F", &["M:T#988966"], &[]),
    (
        "Ayara, First of Locthwain",
        "F",
        &[],
        &["A:T#6c8a11|DrawCards|t0|"],
    ),
    ("Azorius Chancery", "F", &["M:T#273a86"], &[]),
    ("Badlands", "F", &["M:T#906e31", "M:T#bb0d56"], &[]),
    (
        "Balthor the Defiled",
        "F",
        &[],
        &["A:M#616f5b|ReturnAllFromGraveyardToBattlefield|t0|"],
    ),
    (
        "Barkchannel Pathway // Tidechannel Pathway",
        "F",
        &["M:T#b733ed"],
        &[],
    ),
    (
        "Baron Bertram Graywater",
        "F",
        &[],
        &["A:M#e9d57b|DrawCards|t0|"],
    ),
    (
        "Bartolomé del Presidio",
        "F",
        &[],
        &["A:#e8f073|AddCounter|t0|"],
    ),
    (
        "Basilisk Collar",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Batterskull",
        "F",
        &[],
        &["A:M#56e994|MoveZone|t0|", "A:M#d5dbaf|AttachEquipment|t1|s"],
    ),
    ("Battle Cry Goblin", "F", &[], &["A:M#2d876a|Sequence|t0|"]),
    (
        "Battlefield Forge",
        "F",
        &["M:T#33c5df", "M:T#0151ff", "M:T#d6a925"],
        &[],
    ),
    ("Bayou", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    ("Birds of Paradise", "F", &["M:T*#bf1bd3"], &[]),
    ("Birthing Pod", "F", &[], &["A:TM#a5ae0d|Sequence|t0|s"]),
    (
        "Blackblade Reforged",
        "F",
        &[],
        &[
            "A:M#fd0996|AttachEquipment|t1|s",
            "A:M#56e994|AttachEquipment|t1|s",
        ],
    ),
    (
        "Blade of the Bloodchief",
        "F",
        &[],
        &["A:M#57eab6|AttachEquipment|t1|s"],
    ),
    (
        "Bladewing the Risen",
        "F",
        &[],
        &["A:M#e9fb51|ApplyContinuousEffect|t0|"],
    ),
    ("Blasting Station", "F", &[], &["A:T#09954a|DealDamage|t1|"]),
    ("Blazemire Verge", "F", &["M:T#906e31", "M:T#da6e7a"], &[]),
    ("Bleachbone Verge", "F", &["M:T#906e31", "M:T#51ec1d"], &[]),
    (
        "Blighted Woodland",
        "F",
        &["M:T#33c5df"],
        &["A:TSM#3ec398|Sequence|t0|"],
    ),
    (
        "Blightstep Pathway // Searstep Pathway",
        "F",
        &["M:T#906e31"],
        &[],
    ),
    (
        "Blinkmoth Nexus",
        "F",
        &["M:T#33c5df"],
        &[
            "A:M#57eab6|Sequence|t0|",
            "A:TM#ee15c2|ApplyContinuousEffect|t1|",
        ],
    ),
    ("Blood Crypt", "F", &["M:T#906e31", "M:T#bb0d56"], &[]),
    ("Bloodfell Caves", "F", &["M:T#906e31", "M:T#bb0d56"], &[]),
    (
        "Bloodline Keeper",
        "B",
        &[],
        &["A:T#a692a1|CreateToken|t0|"],
    ),
    (
        "Bloodline Keeper",
        "F",
        &[],
        &[
            "A:T#a692a1|CreateToken|t0|",
            "A:M#24f64d|TransformSelf|t0|c",
        ],
    ),
    (
        "Bloodsoaked Champion",
        "F",
        &[],
        &["A:M#65626a|MoveZone|t0|czGraveyard"],
    ),
    ("Bloodstained Mire", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Blooming Marsh", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    ("Bojuka Bog", "F", &["M:T#906e31"], &[]),
    ("Bone Saw", "F", &[], &["A:M#57eab6|AttachEquipment|t1|s"]),
    ("Boros Garrison", "F", &["M:T#714526"], &[]),
    ("Boros Signet", "F", &["M:TM#6bb5c8"], &[]),
    (
        "Boseiju, Who Endures",
        "F",
        &["M:T#b733ed"],
        &["A:M#5d7a79|Sequence|t1|"],
    ),
    (
        "Bountiful Landscape",
        "F",
        &["M:T#33c5df"],
        &["A:TS#f8689e|Sequence|t0|"],
    ),
    (
        "Bountiful Promenade",
        "F",
        &["M:T#b733ed", "M:T#988966"],
        &[],
    ),
    (
        "Brallin, Skyshark Rider",
        "F",
        &[],
        &["A:M#d68d56|ApplyContinuousEffect|t1|"],
    ),
    ("Brash Taunter", "F", &[], &["A:TM#bc42fc|Fight|t1|"]),
    ("Breeding Pool", "F", &["M:T#b733ed", "M:T#a58da2"], &[]),
    (
        "Bridgeworks Battle // Tanglespan Bridgeworks",
        "B",
        &["M:T#b733ed"],
        &[],
    ),
    (
        "Brightclimb Pathway // Grimclimb Pathway",
        "F",
        &["M:T#988966"],
        &[],
    ),
    (
        "Buried Ruin",
        "F",
        &["M:T#33c5df"],
        &["A:TSM#f98da3|MoveZone|t1|"],
    ),
    ("Cabal Coffers", "F", &["M:TM#bcc47d"], &[]),
    ("Cabal Stronghold", "F", &["M:T#33c5df", "M:TM#69d7d5"], &[]),
    (
        "Camellia, the Seedmiser",
        "F",
        &[],
        &["A:M#c27aa1|AddCounter|t0|"],
    ),
    ("Cankerbloom", "F", &[], &["A:SM#3d1e52|Sequence|t0|m"]),
    ("Canopy Tactician", "F", &["M:T#0d1cd8"], &[]),
    ("Canopy Vista", "F", &["M:T#b733ed", "M:T#988966"], &[]),
    ("Carnelian Orb of Dragonkind", "F", &["M:T#bb0d56"], &[]),
    ("Carrion Feeder", "F", &[], &["A:#69bd6d|AddCounter|t0|"]),
    ("Cascade Bluffs", "F", &["M:T#33c5df", "M:TM#0afa3e"], &[]),
    (
        "Castle Ardenvale",
        "F",
        &["M:T#988966"],
        &["A:TM#340781|CreateToken|t0|"],
    ),
    (
        "Castle Embereth",
        "F",
        &["M:T#bb0d56"],
        &["A:TM#76232d|ApplyContinuousEffect|t0|"],
    ),
    (
        "Castle Locthwain",
        "F",
        &["M:T#906e31"],
        &["A:TM#bf098a|Sequence|t0|"],
    ),
    (
        "Castle Vantress",
        "F",
        &["M:T#a58da2"],
        &["A:TM#81b756|Scry|t0|"],
    ),
    (
        "Cathar Commando",
        "F",
        &[],
        &["A:SM#3d1e52|DestroyPermanent|t1|"],
    ),
    (
        "Cathar's Shield",
        "F",
        &[],
        &["A:M#56e994|AttachEquipment|t1|s"],
    ),
    (
        "Caustic Caterpillar",
        "F",
        &[],
        &["A:SM#703584|DestroyPermanent|t1|"],
    ),
    (
        "Cavern of Souls",
        "F",
        &["M:T#33c5df"],
        &["A:T#a692a1|AddManaAnyColorRestricted|t0|"],
    ),
    (
        "Caves of Koilos",
        "F",
        &["M:T#33c5df", "M:T#0151ff", "M:T#d560d4"],
        &[],
    ),
    (
        "Chivalric Alliance",
        "F",
        &[],
        &["A:M#0b8729|CreateToken|t0|"],
    ),
    ("Choked Estuary", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    ("Chromatic Lantern", "F", &["M:T*#bf1bd3"], &[]),
    (
        "Chulane, Teller of Tales",
        "F",
        &[],
        &["A:TM#1c92fd|MoveZone|t1|"],
    ),
    ("Cinder Glade", "F", &["M:T#bb0d56", "M:T#b733ed"], &[]),
    ("Circle of Dreams Druid", "F", &["M:T#78f655"], &[]),
    ("City of Brass", "F", &["M:T*#bf1bd3"], &[]),
    ("Claws of Gix", "F", &[], &["A:M#86b1e0|GainLife|t0|"]),
    (
        "Clearwater Pathway // Murkwater Pathway",
        "F",
        &["M:T#a58da2"],
        &[],
    ),
    ("Clifftop Retreat", "F", &["M:T#bb0d56", "M:T#988966"], &[]),
    ("Command Beacon", "F", &["M:T#33c5df"], &[]),
    ("Command Tower", "F", &["M:T*#bf1bd3"], &[]),
    (
        "Commander's Plate",
        "F",
        &[],
        &["A:M#d5dbaf|AttachEquipment|t1|s"],
    ),
    (
        "Commander's Sphere",
        "F",
        &["M:T*#bf1bd3"],
        &["A:S#597633|DrawCards|t0|"],
    ),
    (
        "Commissar Severina Raine",
        "F",
        &[],
        &["A:M#4bd86b|Sequence|t0|"],
    ),
    (
        "Concealed Courtyard",
        "F",
        &["M:T#988966", "M:T#906e31"],
        &[],
    ),
    (
        "Contagion Clasp",
        "F",
        &[],
        &["A:TM#5354d1|Proliferate|t0|"],
    ),
    (
        "Cragcrown Pathway // Timbercrown Pathway",
        "F",
        &["M:T#bb0d56"],
        &[],
    ),
    (
        "Crashing Drawbridge",
        "F",
        &[],
        &["A:T#a692a1|ApplyContinuousEffect|t0|"],
    ),
    (
        "Creeping Tar Pit",
        "F",
        &["M:T#a58da2", "M:T#906e31"],
        &["A:M#364206|Sequence|t0|"],
    ),
    (
        "Crown of Skemfar",
        "F",
        &[],
        &["A:M#3ee25a|MoveZone|t0|zGraveyard"],
    ),
    ("Crucible of the Spirit Dragon", "F", &["M:T#33c5df"], &[]),
    ("Crypt of Agadeem", "F", &["M:T#906e31", "M:TM#fe1587"], &[]),
    ("Cryptic Coat", "F", &[], &["A:M#5f645d|MoveZone|t0|"]),
    (
        "Cult Conscript",
        "F",
        &[],
        &["A:M#65626a|MoveZone|t0|zGraveyard"],
    ),
    (
        "Darkbore Pathway // Slitherbore Pathway",
        "F",
        &["M:T#906e31"],
        &[],
    ),
    ("Darkslick Shores", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    (
        "Darksteel Garrison",
        "F",
        &[],
        &["A:M#56e994|AttachFortification|t1|s"],
    ),
    ("Darksteel Ingot", "F", &["M:T*#bf1bd3"], &[]),
    ("Darkwater Catacombs", "F", &["M:TM#0864c1"], &[]),
    ("Deathcap Glade", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    (
        "Deathrite Shaman",
        "F",
        &[],
        &[
            "A:T#a692a1|Sequence|t1|",
            "A:TM#d67fdc|Sequence|t1|",
            "A:TM#044070|Sequence|t1|",
        ],
    ),
    ("Decanter of Endless Water", "F", &["M:T*#bf1bd3"], &[]),
    ("Delighted Halfling", "F", &["M:T#b733ed"], &[]),
    ("Demolition Field", "F", &["M:T#33c5df"], &[]),
    (
        "Den of the Bugbear",
        "F",
        &["M:T#bb0d56"],
        &["A:M#d59ed4|Sequence|t0|"],
    ),
    ("Desert of the Fervent", "F", &["M:T#bb0d56"], &[]),
    ("Desert of the True", "F", &["M:T#988966"], &[]),
    (
        "Deserted Temple",
        "F",
        &["M:T#33c5df"],
        &["A:TM#ee15c2|UntapPermanent|t1|"],
    ),
    ("Destiny Spinner", "F", &[], &["A:M#881826|Sequence|t1|"]),
    (
        "Diamond Pick-Axe",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    ("Diamond Valley", "F", &[], &["A:T#09954a|GainLife|t0|"]),
    ("Dimir Aqueduct", "F", &["M:T#67503e"], &[]),
    ("Dimir Guildgate", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    (
        "Dimir Infiltrator",
        "F",
        &[],
        &["A:M#919a1f|SearchLibrary|t0|s"],
    ),
    ("Dimir Signet", "F", &["M:TM#0864c1"], &[]),
    ("Disciple of Freyalise", "B", &["M:T#b733ed"], &[]),
    ("Doom Whisperer", "F", &[], &["A:#872c17|Surveil|t0|"]),
    ("Dour Port-Mage", "F", &[], &["A:TM#69c24b|MoveZone|t1|"]),
    (
        "Dragon's Hoard",
        "F",
        &["M:T*#bf1bd3"],
        &["A:T#efb3db|DrawCards|t0|"],
    ),
    (
        "Dragonskull Summit",
        "F",
        &["M:T#906e31", "M:T#bb0d56"],
        &[],
    ),
    ("Dragonstorm Globe", "F", &["M:T*#bf1bd3"], &[]),
    ("Dreamroot Cascade", "F", &["M:T#b733ed", "M:T#a58da2"], &[]),
    (
        "Dreamstone Hedron",
        "F",
        &["M:T#a9f44f"],
        &["A:TSM#dfc107|DrawCards|t0|"],
    ),
    ("Drifting Meadow", "F", &["M:T#988966"], &[]),
    ("Drowned Catacomb", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    ("Drudge Skeletons", "F", &[], &["A:M#24f64d|Regenerate|t0|"]),
    ("Druids' Repository", "F", &["M:*#5a241a"], &[]),
    ("Dryad Arbor", "F", &["M:T#b733ed"], &[]),
    (
        "Earthquake Dragon",
        "F",
        &[],
        &["A:M#849ace|MoveZone|t0|zGraveyard"],
    ),
    (
        "Eiganjo, Seat of the Empire",
        "F",
        &["M:T#988966"],
        &["A:M#e46dfc|DealDamage|t1|"],
    ),
    ("Elves of Deep Shadow", "F", &["M:T#d560d4"], &[]),
    ("Elvish Archdruid", "F", &["M:T#9a5495"], &[]),
    ("Elvish Harbinger", "F", &["M:T*#bf1bd3"], &[]),
    ("Elvish Mystic", "F", &["M:T#b733ed"], &[]),
    ("Elvish Reclaimer", "F", &[], &["A:TM#831b6c|Sequence|t0|"]),
    ("Elvish Spirit Guide", "F", &["M:#6143c8"], &[]),
    ("Elvish Warmaster", "F", &[], &["A:M#47b55b|Sequence|t0|"]),
    ("Emeria, the Sky Ruin", "F", &["M:T#988966"], &[]),
    (
        "Empyrial Plate",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Etchings of the Chosen",
        "F",
        &[],
        &["A:M#708e1e|ApplyContinuousEffect|t1|"],
    ),
    ("Everflowing Chalice", "F", &["M:T#33c5df"], &[]),
    ("Evolving Wilds", "F", &[], &["A:TS#f8689e|Sequence|t0|"]),
    ("Exotic Orchard", "F", &["M:T*#bf1bd3"], &[]),
    (
        "Ezuri, Renegade Leader",
        "F",
        &[],
        &["A:M#7f1294|Regenerate|t1|", "A:M#cd56de|Sequence|t0|"],
    ),
    (
        "Fable of the Mirror-Breaker",
        "B",
        &[],
        &["A:TM#ee15c2|CreateTokenCopy|t1|s"],
    ),
    ("Fabled Passage", "F", &[], &["A:TS#f8689e|Sequence|t0|"]),
    ("Fellwar Stone", "F", &["M:T*#bf1bd3"], &[]),
    ("Fetid Heath", "F", &["M:T#33c5df", "M:TM#c64a00"], &[]),
    ("Field of the Dead", "F", &["M:T#33c5df"], &[]),
    (
        "Fiery Islet",
        "F",
        &["M:T#81bc8f", "M:T#626463"],
        &["A:TSM#d01778|DrawCards|t0|"],
    ),
    ("Fire Diamond", "F", &["M:T#bb0d56"], &[]),
    (
        "Flamekin Village",
        "F",
        &["M:T#bb0d56"],
        &["A:TM#3c5e48|ApplyContinuousEffect|t1|"],
    ),
    ("Flooded Grove", "F", &["M:T#33c5df", "M:TM#2799b3"], &[]),
    ("Flooded Strand", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Food Chain", "F", &[], &["A:S#597633|AddManaAnyColor|t0|"]),
    ("Forbidden Orchard", "F", &["M:T*#bf1bd3"], &[]),
    ("Foreboding Ruins", "F", &["M:T#906e31", "M:T#bb0d56"], &[]),
    (
        "Forerunner of Slaughter",
        "F",
        &[],
        &["A:M#57eab6|ApplyContinuousEffect|t1|"],
    ),
    ("Forest", "F", &["M:T#b733ed"], &[]),
    ("Forger's Foundry", "F", &["M:T#a58da2"], &[]),
    ("Forgotten Cave", "F", &["M:T#bb0d56"], &[]),
    ("Foul Orchard", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    (
        "Frontier Bivouac",
        "F",
        &["M:T#b733ed", "M:T#a58da2", "M:T#bb0d56"],
        &[],
    ),
    ("Frostboil Snarl", "F", &["M:T#a58da2", "M:T#bb0d56"], &[]),
    ("Furycalm Snarl", "F", &["M:T#bb0d56", "M:T#988966"], &[]),
    ("Fyndhorn Elves", "F", &["M:T#b733ed"], &[]),
    ("Gaea's Cradle", "F", &["M:T#78f655"], &[]),
    (
        "Geier Reach Sanitarium",
        "F",
        &["M:T#33c5df"],
        &["A:TM#42f5c5|ForEach|t0|"],
    ),
    (
        "Gemstone Array",
        "F",
        &["M:*#5a241a"],
        &["A:M#7be59f|AddCounter|t0|"],
    ),
    (
        "Ghave, Guru of Spores",
        "F",
        &[],
        &["A:M#13ac4a|CreateToken|t0|", "A:M#86b1e0|AddCounter|t1|"],
    ),
    (
        "Ghost Quarter",
        "F",
        &["M:T#33c5df"],
        &["A:TS#f8689e|Sequence|t1|"],
    ),
    ("Gilt-Leaf Palace", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    (
        "Gingerbrute",
        "F",
        &[],
        &[
            "A:M#57eab6|ApplyContinuousEffect|t0|",
            "A:TSM#f98da3|GainLife|t0|",
        ],
    ),
    ("Glacial Fortress", "F", &["M:T#988966", "M:T#a58da2"], &[]),
    (
        "Glimmer Lens",
        "F",
        &[],
        &["A:M#155e30|AttachEquipment|t1|s"],
    ),
    (
        "Glistening Sphere",
        "F",
        &["M:T*#bf1bd3"],
        &["A:T#a692a1|AddManaChoice|t0|c"],
    ),
    ("Gloomlake Verge", "F", &["M:T#a58da2", "M:T#179f40"], &[]),
    (
        "Gnarlroot Trapper",
        "F",
        &[],
        &["A:T#e84cf5|AddManaRestricted|t0|"],
    ),
    (
        "Goblin Bombardment",
        "F",
        &[],
        &["A:#69bd6d|DealDamage|t1|"],
    ),
    ("Goblin Chirurgeon", "F", &[], &["A:#524f21|Regenerate|t1|"]),
    (
        "Goblin Cratermaker",
        "F",
        &[],
        &["A:SM#3d1e52|Sequence|t0|m"],
    ),
    (
        "Goblin Lookout",
        "F",
        &[],
        &["A:T#a87f16|ApplyContinuousEffect|t0|"],
    ),
    (
        "Goblin Motivator",
        "F",
        &[],
        &["A:T#a692a1|ApplyContinuousEffect|t1|"],
    ),
    (
        "Goblin Sharpshooter",
        "F",
        &[],
        &["A:T#a692a1|DealDamage|t1|"],
    ),
    (
        "Goblin Sledder",
        "F",
        &[],
        &["A:#524f21|ApplyContinuousEffect|t1|"],
    ),
    (
        "Goblin Trashmaster",
        "F",
        &[],
        &["A:#524f21|DestroyPermanent|t1|"],
    ),
    ("Godless Shrine", "F", &["M:T#988966", "M:T#906e31"], &[]),
    ("Goldhound", "F", &["M:TS*#1ab63c"], &[]),
    (
        "Golgari Grave-Troll",
        "F",
        &[],
        &["A:M#13ac4a|Regenerate|t0|"],
    ),
    ("Golgari Guildgate", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    ("Golgari Rot Farm", "F", &["M:T#573bdd"], &[]),
    ("Golgari Signet", "F", &["M:TM#9b420a"], &[]),
    (
        "Goro-Goro, Disciple of Ryusei",
        "F",
        &[],
        &["A:M#d68d56|ApplyContinuousEffect|t0|"],
    ),
    ("Graven Cairns", "F", &["M:T#33c5df", "M:TM#a37a0c"], &[]),
    ("Greater Good", "F", &[], &["A:#69bd6d|Sequence|t0|"]),
    (
        "Grim Backwoods",
        "F",
        &["M:T#33c5df"],
        &["A:TM#59899c|DrawCards|t0|"],
    ),
    (
        "Growing Rites of Itlimoc",
        "B",
        &["M:T#b733ed", "M:T#78f655"],
        &[],
    ),
    ("Gruul Turf", "F", &["M:T#8ec201"], &[]),
    ("Halimar Depths", "F", &["M:T#a58da2"], &[]),
    (
        "Hall of Heliod's Generosity",
        "F",
        &["M:T#33c5df"],
        &["A:TM#e227fe|MoveZone|t1|"],
    ),
    ("Hallowed Fountain", "F", &["M:T#988966", "M:T#a58da2"], &[]),
    (
        "Hammer of Nazahn",
        "F",
        &[],
        &["A:M#0733fd|AttachEquipment|t1|s"],
    ),
    (
        "Hanweir Battlements",
        "F",
        &["M:T#33c5df"],
        &[
            "A:TM#3c5e48|ApplyContinuousEffect|t1|",
            "A:TM#6df500|Meld|t0|",
        ],
    ),
    ("Haunted Mire", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    ("Haunted Ridge", "F", &["M:T#906e31", "M:T#bb0d56"], &[]),
    (
        "Haven of the Spirit Dragon",
        "F",
        &["M:T#33c5df"],
        &[
            "A:T#a692a1|AddManaAnyColorRestricted|t0|",
            "A:TSM#f98da3|MoveZone|t1|",
        ],
    ),
    ("Haywire Mite", "F", &[], &["A:SM#563a3b|ExileObject|t1|"]),
    (
        "Hedron Archive",
        "F",
        &["M:T#953706"],
        &["A:TSM#f98da3|DrawCards|t0|"],
    ),
    (
        "Helm of the Host",
        "F",
        &[],
        &["A:M#d5dbaf|AttachEquipment|t1|s"],
    ),
    (
        "High Market",
        "F",
        &["M:T#33c5df"],
        &["A:T#09954a|GainLife|t0|"],
    ),
    (
        "Higure, the Still Wind",
        "F",
        &[],
        &["A:M#7be59f|ApplyContinuousEffect|t1|"],
    ),
    ("Hinterland Harbor", "F", &["M:T#b733ed", "M:T#a58da2"], &[]),
    ("Honor-Worn Shaku", "F", &["M:T#33c5df"], &[]),
    ("Howlsquad Heavy", "F", &["M:T#1bf785"], &[]),
    (
        "Idol of Oblivion",
        "F",
        &[],
        &["A:T#a692a1|DrawCards|t0|c", "A:TSM#7ca816|CreateToken|t0|"],
    ),
    (
        "Ignoble Hierarch",
        "F",
        &["M:T#906e31", "M:T#bb0d56", "M:T#b733ed"],
        &[],
    ),
    (
        "Illusionist's Bracers",
        "F",
        &[],
        &["A:M#56e994|AttachEquipment|t1|s"],
    ),
    (
        "Immaculate Magistrate",
        "F",
        &[],
        &["A:T#a692a1|AddCounterAmount|t1|"],
    ),
    (
        "Imperious Perfect",
        "F",
        &[],
        &["A:TM#044070|CreateToken|t0|"],
    ),
    (
        "Incubation Druid",
        "F",
        &[],
        &["A:TM#b441e9|Conditional|t0|"],
    ),
    (
        "Indatha Triome",
        "F",
        &["M:T#988966", "M:T#906e31", "M:T#b733ed"],
        &[],
    ),
    (
        "Indulgent Aristocrat",
        "F",
        &[],
        &["A:M#258006|ForEach|t0|"],
    ),
    (
        "Ink-Eyes, Servant of Oni",
        "F",
        &[],
        &["A:M#65626a|Regenerate|t0|"],
    ),
    (
        "Inkmoth Nexus",
        "F",
        &["M:T#33c5df"],
        &["A:M#57eab6|Sequence|t0|"],
    ),
    (
        "Inventors' Fair",
        "F",
        &["M:T#33c5df"],
        &["A:TSM#8349c6|Sequence|t0|c"],
    ),
    ("Island", "F", &["M:T#a58da2"], &[]),
    ("Isolated Chapel", "F", &["M:T#988966", "M:T#906e31"], &[]),
    (
        "Izoni, Thousand-Eyed",
        "F",
        &[],
        &["A:M#ee7285|Sequence|t0|"],
    ),
    ("Izzet Boilerworks", "F", &["M:T#43c10a"], &[]),
    ("Izzet Signet", "F", &["M:TM#75150d"], &[]),
    ("Jade Orb of Dragonkind", "F", &["M:T#b733ed"], &[]),
    (
        "Jagged-Scar Archers",
        "F",
        &[],
        &["A:T#a692a1|DealDamage|t1|"],
    ),
    (
        "Jetmir's Garden",
        "F",
        &["M:T#bb0d56", "M:T#b733ed", "M:T#988966"],
        &[],
    ),
    (
        "Joraga Treespeaker",
        "F",
        &[],
        &["A:M#fba3b8|AddCounter|t0|s"],
    ),
    ("Jungle Hollow", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    (
        "Jungle Shrine",
        "F",
        &["M:T#bb0d56", "M:T#b733ed", "M:T#988966"],
        &[],
    ),
    (
        "Karn's Bastion",
        "F",
        &["M:T#33c5df"],
        &["A:TM#5354d1|Proliferate|t0|"],
    ),
    (
        "Ketria Triome",
        "F",
        &["M:T#b733ed", "M:T#a58da2", "M:T#bb0d56"],
        &[],
    ),
    (
        "Khalni Heart Expedition",
        "F",
        &[],
        &["A:S#0eac0e|Sequence|t0|"],
    ),
    (
        "Kher Keep",
        "F",
        &["M:T#33c5df"],
        &["A:TM#dbb40e|CreateToken|t0|"],
    ),
    (
        "Kiki-Jiki, Mirror Breaker",
        "F",
        &[],
        &["A:T#a692a1|CreateTokenCopy|t1|s"],
    ),
    (
        "Kite Shield",
        "F",
        &[],
        &["A:M#56e994|AttachEquipment|t1|s"],
    ),
    (
        "Knight of the Ebon Legion",
        "F",
        &[],
        &["A:M#b0c340|Sequence|t0|"],
    ),
    (
        "Kogla, the Titan Ape",
        "F",
        &[],
        &["A:M#fba3b8|Sequence|t1|"],
    ),
    (
        "Kor Haven",
        "F",
        &["M:T#33c5df"],
        &["A:TM#e227fe|PreventCombatDamageFromOrTo|t1|"],
    ),
    (
        "Krenko, Mob Boss",
        "F",
        &[],
        &["A:T#a692a1|CreateToken|t0|"],
    ),
    (
        "Lathliss, Dragon Queen",
        "F",
        &[],
        &["A:M#2d876a|ApplyContinuousEffect|t0|"],
    ),
    ("Leaden Myr", "F", &["M:T#906e31"], &[]),
    (
        "Legion's Landing",
        "B",
        &["M:T#988966"],
        &["A:TM#e227fe|CreateToken|t0|"],
    ),
    (
        "Leyline of Abundance",
        "F",
        &[],
        &["A:M#da658c|ForEach|t0|"],
    ),
    (
        "Lightning Greaves",
        "F",
        &[],
        &["A:M#806929|AttachEquipment|t1|s"],
    ),
    ("Llanowar Elves", "F", &["M:T#b733ed"], &[]),
    ("Llanowar Tribe", "F", &["M:T#0d1cd8"], &[]),
    (
        "Llanowar Wastes",
        "F",
        &["M:T#33c5df", "M:T#d560d4", "M:T#e1ec5d"],
        &[],
    ),
    ("Lonely Sandbar", "F", &["M:T#a58da2"], &[]),
    ("Luxury Suite", "F", &["M:T#906e31", "M:T#bb0d56"], &[]),
    (
        "Maelstrom of the Spirit Dragon",
        "F",
        &["M:T#33c5df"],
        &[
            "A:T#a692a1|AddManaAnyColorRestricted|t0|",
            "A:TSM#8349c6|Sequence|t0|",
        ],
    ),
    (
        "Magnifying Glass",
        "F",
        &["M:T#33c5df"],
        &["A:TM#5354d1|Investigate|t0|"],
    ),
    ("Mana Confluence", "F", &["M:T*#95cd0b"], &[]),
    ("Mana Crypt", "F", &["M:T#953706"], &[]),
    ("Marble Diamond", "F", &["M:T#988966"], &[]),
    (
        "Mardu Ascendancy",
        "F",
        &[],
        &["A:S#597633|ApplyContinuousEffect|t0|"],
    ),
    ("Marsh Flats", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Marwyn, the Nurturer", "F", &["M:T#2c6110"], &[]),
    (
        "Mask of Memory",
        "F",
        &[],
        &["A:M#57eab6|AttachEquipment|t1|s"],
    ),
    ("Maskwood Nexus", "F", &[], &["A:TM#1c92fd|CreateToken|t0|"]),
    ("Maze of Ith", "F", &[], &["A:T#a692a1|Sequence|t1|"]),
    (
        "Minamo, School at Water's Edge",
        "F",
        &["M:T#a58da2"],
        &["A:TM#481cec|UntapPermanent|t1|"],
    ),
    (
        "Minas Tirith",
        "F",
        &["M:T#988966"],
        &["A:TM#e227fe|DrawCards|t0|c"],
    ),
    (
        "Mind Stone",
        "F",
        &["M:T#33c5df"],
        &["A:TSM#d01778|DrawCards|t0|"],
    ),
    (
        "Miren, the Moaning Well",
        "F",
        &["M:T#33c5df"],
        &["A:TM#da53ce|GainLife|t0|"],
    ),
    ("Mirror Entity", "F", &[], &["A:M#cbaf37|Sequence|t0|"]),
    ("Mistrise Village", "F", &["M:T#a58da2"], &[]),
    ("Misty Rainforest", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Moggcatcher", "F", &[], &["A:TM#1c92fd|SearchLibrary|t0|"]),
    ("Morphic Pool", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    ("Mortuary Mire", "F", &["M:T#906e31"], &[]),
    ("Mountain", "F", &["M:T#bb0d56"], &[]),
    ("Mox Amber", "F", &["M:T*#bf1bd3"], &[]),
    ("Mox Jasper", "F", &["M:T*#b7db4b"], &[]),
    ("Mox Opal", "F", &["M:T*#fc7110"], &[]),
    ("Myriad Landscape", "F", &["M:T#33c5df"], &[]),
    (
        "Mystic Monastery",
        "F",
        &["M:T#a58da2", "M:T#bb0d56", "M:T#988966"],
        &[],
    ),
    ("Mystic Sanctuary", "F", &["M:T#a58da2"], &[]),
    (
        "Necroblossom Snarl",
        "F",
        &["M:T#906e31", "M:T#b733ed"],
        &[],
    ),
    (
        "Needleverge Pathway // Pillarverge Pathway",
        "F",
        &["M:T#bb0d56"],
        &[],
    ),
    (
        "Nezahal, Primal Tide",
        "F",
        &[],
        &["A:#1652ac|ExileWithDelayedReturn|t0|"],
    ),
    (
        "Niv-Mizzet, the Firemind",
        "F",
        &[],
        &["A:T#a692a1|DrawCards|t0|"],
    ),
    (
        "Noble Hierarch",
        "F",
        &["M:T#b733ed", "M:T#988966", "M:T#a58da2"],
        &[],
    ),
    (
        "Nomad Outpost",
        "F",
        &["M:T#bb0d56", "M:T#988966", "M:T#906e31"],
        &[],
    ),
    (
        "Nurturing Peatland",
        "F",
        &["M:T#67d761", "M:T#4ca7e6"],
        &["A:TSM#d01778|DrawCards|t0|"],
    ),
    ("Nykthos, Shrine to Nyx", "F", &["M:T#33c5df"], &[]),
    (
        "Oboro, Palace in the Clouds",
        "F",
        &["M:T#a58da2"],
        &["A:M#57eab6|MoveZone|t0|"],
    ),
    (
        "Olivia Voldaren",
        "F",
        &[],
        &["A:M#2d876a|Sequence|t1|", "A:M#93b0fe|GainControl|t1|"],
    ),
    ("Ominous Seas", "F", &[], &["A:#7f8148|CreateToken|t0|"]),
    (
        "Opulent Palace",
        "F",
        &["M:T#906e31", "M:T#b733ed", "M:T#a58da2"],
        &[],
    ),
    ("Oran-Rief, the Vastwood", "F", &["M:T#b733ed"], &[]),
    ("Ornithopter of Paradise", "F", &["M:T*#bf1bd3"], &[]),
    ("Orzhov Basilica", "F", &["M:T#eacbc6"], &[]),
    ("Orzhov Signet", "F", &["M:TM#4cc2e0"], &[]),
    (
        "Otawara, Soaring City",
        "F",
        &["M:T#a58da2"],
        &["A:M#771eb7|MoveZone|t1|"],
    ),
    ("Overgrown Tomb", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    (
        "Paradise Mantle",
        "F",
        &[],
        &["A:M#57eab6|AttachEquipment|t1|s"],
    ),
    ("Pashalik Mons", "F", &[], &["A:M#033314|CreateToken|t0|"]),
    ("Patchwork Banner", "F", &["M:T*#bf1bd3"], &[]),
    ("Path of Ancestry", "F", &["M:T*#bf1bd3"], &[]),
    (
        "Patriar's Seal",
        "F",
        &["M:T*#bf1bd3"],
        &["A:TM#ee15c2|UntapPermanent|t1|"],
    ),
    ("Perilous Forays", "F", &[], &["A:M#86b1e0|Sequence|t0|"]),
    (
        "Phyrexian Altar",
        "F",
        &[],
        &["A:#69bd6d|AddManaAnyColor|t0|"],
    ),
    (
        "Phyrexian Tower",
        "F",
        &["M:T#33c5df"],
        &["A:T#09954a|AddMana|t0|"],
    ),
    ("Plague Myr", "F", &["M:T#33c5df"], &[]),
    ("Plains", "F", &["M:T#988966"], &[]),
    ("Plateau", "F", &["M:T#bb0d56", "M:T#988966"], &[]),
    ("Polluted Delta", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Prairie Stream", "F", &["M:T#988966", "M:T#a58da2"], &[]),
    ("Priest of Titania", "F", &["M:T#3f914e"], &[]),
    ("Prismatic Vista", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    (
        "Purphoros, God of the Forge",
        "F",
        &[],
        &["A:M#71d14c|ApplyContinuousEffect|t0|"],
    ),
    (
        "Radha, Heart of Keld",
        "F",
        &[],
        &["A:M#c9973a|ApplyContinuousEffect|t0|"],
    ),
    (
        "Raffine's Tower",
        "F",
        &["M:T#988966", "M:T#a58da2", "M:T#906e31"],
        &[],
    ),
    ("Rakdos Carnarium", "F", &["M:T#b92bb9"], &[]),
    ("Rakdos Signet", "F", &["M:TM#af2956"], &[]),
    ("Ramos, Dragon Engine", "F", &["M:#1ce854"], &[]),
    (
        "Raugrin Triome",
        "F",
        &["M:T#a58da2", "M:T#bb0d56", "M:T#988966"],
        &[],
    ),
    (
        "Razaketh, the Foulblooded",
        "F",
        &[],
        &["A:#4a4c74|Sequence|t0|"],
    ),
    (
        "Reassembling Skeleton",
        "F",
        &[],
        &["A:M#65626a|MoveZone|t0|zGraveyard"],
    ),
    ("Reconnaissance", "F", &[], &["A:M#806929|Sequence|t1|"]),
    ("Reflecting Pool", "F", &["M:T*#bf1bd3"], &[]),
    (
        "Rejuvenating Springs",
        "F",
        &["M:T#b733ed", "M:T#a58da2"],
        &[],
    ),
    ("Reliquary Tower", "F", &["M:T#33c5df"], &[]),
    ("Replicating Ring", "F", &["M:T*#bf1bd3"], &[]),
    ("Revitalizing Repast", "B", &["M:T#b733ed"], &[]),
    ("Rhys the Exiled", "F", &[], &["A:M#2ad9d6|Regenerate|t0|"]),
    (
        "Riverglide Pathway // Lavaglide Pathway",
        "F",
        &["M:T#a58da2"],
        &[],
    ),
    ("Rockfall Vale", "F", &["M:T#bb0d56", "M:T#b733ed"], &[]),
    (
        "Rogue's Passage",
        "F",
        &["M:T#33c5df"],
        &["A:TM#5354d1|ApplyContinuousEffect|t1|"],
    ),
    ("Rootbound Crag", "F", &["M:T#bb0d56", "M:T#b733ed"], &[]),
    ("Rugged Prairie", "F", &["M:T#33c5df", "M:TM#5884db"], &[]),
    ("Rummaging Goblin", "F", &[], &["A:T#d8f038|DrawCards|t0|"]),
    ("Sacred Foundry", "F", &["M:T#bb0d56", "M:T#988966"], &[]),
    ("Sakura-Tribe Elder", "F", &[], &["A:S#597633|Sequence|t0|"]),
    (
        "Sakura-Tribe Scout",
        "F",
        &[],
        &["A:T#a692a1|PutLandFromHandOntoBattlefield|t0|"],
    ),
    (
        "Samut, Voice of Dissent",
        "F",
        &[],
        &["A:TM#cb4066|UntapPermanent|t1|"],
    ),
    (
        "Sandsteppe Citadel",
        "F",
        &["M:T#988966", "M:T#906e31", "M:T#b733ed"],
        &[],
    ),
    (
        "Savage Lands",
        "F",
        &["M:T#906e31", "M:T#bb0d56", "M:T#b733ed"],
        &[],
    ),
    (
        "Savai Triome",
        "F",
        &["M:T#bb0d56", "M:T#988966", "M:T#906e31"],
        &[],
    ),
    ("Savannah", "F", &["M:T#b733ed", "M:T#988966"], &[]),
    ("Scalding Tarn", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Scaled Nurturer", "F", &["M:T#b733ed"], &[]),
    (
        "Scavenger Grounds",
        "F",
        &["M:T#33c5df"],
        &["A:TM#de8f0c|ForEach|t0|"],
    ),
    (
        "Scion of the Ur-Dragon",
        "F",
        &[],
        &["A:M#7be59f|Sequence|t0|"],
    ),
    ("Scoured Barrens", "F", &["M:T#988966", "M:T#906e31"], &[]),
    (
        "Scourge of Valkas",
        "F",
        &[],
        &["A:M#d68d56|ApplyContinuousEffect|t0|"],
    ),
    ("Scrubland", "F", &["M:T#988966", "M:T#906e31"], &[]),
    ("Sea Gate Restoration", "B", &["M:T#a58da2"], &[]),
    ("Sea of Clouds", "F", &["M:T#988966", "M:T#a58da2"], &[]),
    (
        "Seaside Citadel",
        "F",
        &["M:T#b733ed", "M:T#988966", "M:T#a58da2"],
        &[],
    ),
    (
        "Secluded Courtyard",
        "F",
        &["M:T#33c5df"],
        &["A:T#a692a1|AddManaAnyColorRestricted|t0|"],
    ),
    ("Secluded Steppe", "F", &["M:T#988966"], &[]),
    ("Selesnya Sanctuary", "F", &["M:T#f84e74"], &[]),
    (
        "Shadowspear",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Shalai, Voice of Plenty",
        "F",
        &[],
        &["A:M#e33b90|ForEach|t0|"],
    ),
    ("Sharktocrab", "F", &[], &["A:TM#62d4ea|Conditional|t0|"]),
    ("Shattered Sanctum", "F", &["M:T#988966", "M:T#906e31"], &[]),
    (
        "Shifting Woodland",
        "F",
        &["M:T#b733ed"],
        &["A:M#18d6d3|BecomeCopyOf|t1|c"],
    ),
    ("Shineshadow Snarl", "F", &["M:T#988966", "M:T#906e31"], &[]),
    ("Shipwreck Marsh", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    (
        "Shivan Reef",
        "F",
        &["M:T#33c5df", "M:T#a27058", "M:T#d6a925"],
        &[],
    ),
    (
        "Shizo, Death's Storehouse",
        "F",
        &["M:T#906e31"],
        &["A:TM#d67fdc|ApplyContinuousEffect|t1|"],
    ),
    (
        "Siege-Gang Commander",
        "F",
        &[],
        &["A:M#a7568d|DealDamage|t1|"],
    ),
    (
        "Siege-Gang Lieutenant",
        "F",
        &[],
        &["A:M#6a6d03|DealDamage|t1|"],
    ),
    (
        "Silent Clearing",
        "F",
        &["M:T#72fddb", "M:T#67d761"],
        &["A:TSM#d01778|DrawCards|t0|"],
    ),
    ("Simian Spirit Guide", "F", &["M:#b063ec"], &[]),
    ("Simic Ascendancy", "F", &[], &["A:M#257f23|AddCounter|t1|"]),
    ("Simic Growth Chamber", "F", &["M:T#9cc700"], &[]),
    ("Simic Signet", "F", &["M:TM#66ffb0"], &[]),
    (
        "Skemfar Elderhall",
        "F",
        &["M:T#b733ed"],
        &["A:TSM#26f020|Sequence|t1|s"],
    ),
    (
        "Skithiryx, the Blight Dragon",
        "F",
        &[],
        &[
            "A:M#24f64d|ApplyContinuousEffect|t0|",
            "A:M#79ad50|Regenerate|t0|",
        ],
    ),
    ("Skullclamp", "F", &[], &["A:M#57eab6|AttachEquipment|t1|s"]),
    (
        "Skyshroud Poacher",
        "F",
        &[],
        &["A:TM#1c92fd|SearchLibrary|t0|"],
    ),
    (
        "Slayers' Stronghold",
        "F",
        &["M:T#33c5df"],
        &["A:TM#fc6596|Sequence|t1|"],
    ),
    ("Smoldering Crater", "F", &["M:T#bb0d56"], &[]),
    ("Smoldering Marsh", "F", &["M:T#906e31", "M:T#bb0d56"], &[]),
    ("Snow-Covered Island", "F", &["M:T#a58da2"], &[]),
    ("Snow-Covered Swamp", "F", &["M:T#906e31"], &[]),
    (
        "Sokenzan, Crucible of Defiance",
        "F",
        &["M:T#bb0d56"],
        &["A:M#0fa517|CreateToken|t0|"],
    ),
    ("Sol Ring", "F", &["M:T#953706"], &[]),
    (
        "Spara's Headquarters",
        "F",
        &["M:T#b733ed", "M:T#988966", "M:T#a58da2"],
        &[],
    ),
    (
        "Spawning Pit",
        "F",
        &[],
        &["A:#69bd6d|AddCounter|t0|", "A:M#2366bd|CreateToken|t0|"],
    ),
    ("Spectator Seating", "F", &["M:T#bb0d56", "M:T#988966"], &[]),
    ("Spectral Sailor", "F", &[], &["A:M#1880bc|DrawCards|t0|"]),
    (
        "Spidersilk Net",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Spike Weaver",
        "F",
        &[],
        &[
            "A:M#4e37da|AddCounter|t1|",
            "A:M#13ac4a|PreventAllCombatDamage|t0|",
        ],
    ),
    ("Spinerock Knoll", "F", &["M:T#bb0d56"], &[]),
    ("Spire Garden", "F", &["M:T#bb0d56", "M:T#b733ed"], &[]),
    (
        "Spore Frog",
        "F",
        &[],
        &["A:S#597633|PreventAllCombatDamage|t0|"],
    ),
    ("Spymaster's Vault", "F", &["M:T#906e31"], &[]),
    (
        "Staff of Compleation",
        "F",
        &["M:T*#b59410"],
        &[
            "A:T#e84cf5|DestroyPermanent|t1|",
            "A:T#7622f9|Proliferate|t0|",
            "A:T#a7a5f6|DrawCards|t0|",
            "A:M#d5dbaf|UntapPermanent|t0|",
        ],
    ),
    (
        "Staff of Domination",
        "F",
        &[],
        &[
            "A:M#57eab6|UntapPermanent|t0|",
            "A:TM#42f5c5|GainLife|t0|",
            "A:TM#1c92fd|UntapPermanent|t1|",
            "A:TM#5354d1|TapPermanent|t1|",
            "A:TM#077d0c|DrawCards|t0|",
        ],
    ),
    ("Steam Vents", "F", &["M:T#a58da2", "M:T#bb0d56"], &[]),
    (
        "Steel Hellkite",
        "F",
        &[],
        &[
            "A:M#7be59f|ApplyContinuousEffect|t0|",
            "A:M#cbaf37|Nothing|t0|o",
        ],
    ),
    ("Stomping Ground", "F", &["M:T#bb0d56", "M:T#b733ed"], &[]),
    ("Stormcarved Coast", "F", &["M:T#a58da2", "M:T#bb0d56"], &[]),
    (
        "Strip Mine",
        "F",
        &["M:T#33c5df"],
        &["A:TS#f8689e|DestroyPermanent|t1|"],
    ),
    ("Strixhaven Stadium", "F", &[], &["A:T#a692a1|Sequence|t0|"]),
    ("Sulfur Falls", "F", &["M:T#a58da2", "M:T#bb0d56"], &[]),
    (
        "Sulfurous Springs",
        "F",
        &["M:T#33c5df", "M:T#d560d4", "M:T#d6a925"],
        &[],
    ),
    ("Sundown Pass", "F", &["M:T#bb0d56", "M:T#988966"], &[]),
    ("Sunken Hollow", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    ("Sunken Palace", "F", &["M:T#a58da2"], &[]),
    ("Sunken Ruins", "F", &["M:T#33c5df", "M:TM#afc9fb"], &[]),
    ("Sunpetal Grove", "F", &["M:T#b733ed", "M:T#988966"], &[]),
    ("Swamp", "F", &["M:T#906e31"], &[]),
    (
        "Swiftfoot Boots",
        "F",
        &[],
        &["A:M#57eab6|AttachEquipment|t1|s"],
    ),
    ("Swiftwater Cliffs", "F", &["M:T#a58da2", "M:T#bb0d56"], &[]),
    (
        "Sword of Body and Mind",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Sword of Feast and Famine",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Sword of Fire and Ice",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Sword of Light and Shadow",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Sword of Sinew and Steel",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Sword of Truth and Justice",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Sword of Vengeance",
        "F",
        &[],
        &["A:M#56e994|AttachEquipment|t1|s"],
    ),
    (
        "Sword of War and Peace",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Sword of the Animist",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Sword of the Paruns",
        "F",
        &[],
        &["A:M#56e994|AttachEquipment|t1|s"],
    ),
    ("Taiga", "F", &["M:T#bb0d56", "M:T#b733ed"], &[]),
    (
        "Tainted Field",
        "F",
        &["M:T#33c5df", "M:T#92572c", "M:T#39a41a"],
        &[],
    ),
    (
        "Tainted Isle",
        "F",
        &["M:T#33c5df", "M:T#3b19ea", "M:T#39a41a"],
        &[],
    ),
    (
        "Tainted Wood",
        "F",
        &["M:T#33c5df", "M:T#39a41a", "M:T#983636"],
        &[],
    ),
    (
        "Takenuma, Abandoned Mire",
        "F",
        &["M:T#906e31"],
        &["A:M#8eb705|Sequence|t0|"],
    ),
    (
        "Talisman of Conviction",
        "F",
        &["M:T#33c5df", "M:T#d6a925", "M:T#0151ff"],
        &[],
    ),
    (
        "Talisman of Creativity",
        "F",
        &["M:T#33c5df", "M:T#a27058", "M:T#d6a925"],
        &[],
    ),
    (
        "Talisman of Dominance",
        "F",
        &["M:T#33c5df", "M:T#a27058", "M:T#d560d4"],
        &[],
    ),
    (
        "Talisman of Hierarchy",
        "F",
        &["M:T#33c5df", "M:T#0151ff", "M:T#d560d4"],
        &[],
    ),
    (
        "Talisman of Indulgence",
        "F",
        &["M:T#33c5df", "M:T#d560d4", "M:T#d6a925"],
        &[],
    ),
    (
        "Talisman of Resilience",
        "F",
        &["M:T#33c5df", "M:T#d560d4", "M:T#e1ec5d"],
        &[],
    ),
    ("Teferi's Isle", "F", &["M:T#fd48a8"], &[]),
    ("Temple Garden", "F", &["M:T#b733ed", "M:T#988966"], &[]),
    ("Temple of Deceit", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    (
        "Temple of Epiphany",
        "F",
        &["M:T#a58da2", "M:T#bb0d56"],
        &[],
    ),
    ("Temple of Malady", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    ("Temple of Malice", "F", &["M:T#906e31", "M:T#bb0d56"], &[]),
    ("Temple of Silence", "F", &["M:T#988966", "M:T#906e31"], &[]),
    ("Temple of Triumph", "F", &["M:T#bb0d56", "M:T#988966"], &[]),
    (
        "Temple of the Dragon Queen",
        "F",
        &[],
        &["A:T#a692a1|AddManaOfChosenColor|t0|"],
    ),
    ("Temple of the False God", "F", &["M:T#587584"], &[]),
    (
        "Terramorphic Expanse",
        "F",
        &[],
        &["A:TS#f8689e|Sequence|t0|"],
    ),
    (
        "Thaumatic Compass",
        "B",
        &["M:T#33c5df"],
        &["A:T#a692a1|Sequence|t1|"],
    ),
    (
        "Thaumatic Compass",
        "F",
        &[],
        &["A:TM#42f5c5|SearchLibrary|t0|"],
    ),
    (
        "The Fire Crystal",
        "F",
        &[],
        &["A:TM#08b496|CreateTokenCopy|t1|s"],
    ),
    ("The Great Henge", "F", &[], &["A:T#a692a1|Sequence|t0|"]),
    ("The Locust God", "F", &[], &["A:M#bf98b0|DrawCards|t0|"]),
    ("The One Ring", "F", &[], &["A:T#a692a1|Sequence|t0|"]),
    (
        "The Reaver Cleaver",
        "F",
        &[],
        &["A:M#56e994|AttachEquipment|t1|s"],
    ),
    (
        "The Seedcore",
        "F",
        &["M:T#33c5df"],
        &[
            "A:T#a692a1|AddManaAnyColorRestricted|t0|",
            "A:T#a692a1|Sequence|t1|c",
        ],
    ),
    ("The Soul Stone", "F", &["M:T#906e31"], &[]),
    ("The World Tree", "F", &["M:T#b733ed"], &[]),
    (
        "Thespian's Stage",
        "F",
        &["M:T#33c5df"],
        &["A:TM#42f5c5|BecomeCopyOf|t1|"],
    ),
    (
        "Thornbite Staff",
        "F",
        &[],
        &["A:M#0733fd|AttachEquipment|t1|s"],
    ),
    ("Thought Vessel", "F", &["M:T#33c5df"], &[]),
    (
        "Thousand-Year Elixir",
        "F",
        &[],
        &["A:TM#ee15c2|UntapPermanent|t1|"],
    ),
    (
        "Thrasios, Triton Hero",
        "F",
        &[],
        &["A:M#0733fd|Sequence|t0|"],
    ),
    (
        "Three Tree City",
        "F",
        &["M:T#33c5df"],
        &["A:TM#42f5c5|AddManaOfAnyColorAmount|t0|"],
    ),
    ("Thundering Falls", "F", &["M:T#a58da2", "M:T#bb0d56"], &[]),
    (
        "Timberwatch Elf",
        "F",
        &[],
        &["A:T#a692a1|ApplyContinuousEffect|t1|"],
    ),
    (
        "Torch Courier",
        "F",
        &[],
        &["A:S#597633|ApplyContinuousEffect|t1|"],
    ),
    (
        "Touch the Spirit Realm",
        "F",
        &[],
        &["A:M#b86676|ExileWithDelayedReturn|t1|"],
    ),
    ("Training Center", "F", &["M:T#a58da2", "M:T#bb0d56"], &[]),
    (
        "Treasure Vault",
        "F",
        &["M:T#33c5df"],
        &["A:TSM#ed6438|Repeat|t0|"],
    ),
    ("Tropical Island", "F", &["M:T#b733ed", "M:T#a58da2"], &[]),
    ("Tundra", "F", &["M:T#988966", "M:T#a58da2"], &[]),
    ("Twilight Mire", "F", &["M:T#33c5df", "M:TM#800b38"], &[]),
    ("Umbral Collar Zealot", "F", &[], &["A:#e8f073|Surveil|t0|"]),
    (
        "Umbral Mantle",
        "F",
        &[],
        &["A:M#806929|AttachEquipment|t1|s"],
    ),
    (
        "Umezawa's Jitte",
        "F",
        &[],
        &["A:#380fc1|Sequence|t0|m", "A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Unclaimed Territory",
        "F",
        &["M:T#33c5df"],
        &["A:T#a692a1|AddManaAnyColorRestricted|t0|"],
    ),
    ("Undercity Sewers", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    (
        "Underground Mortuary",
        "F",
        &["M:T#906e31", "M:T#b733ed"],
        &[],
    ),
    (
        "Underground River",
        "F",
        &["M:T#33c5df", "M:T#a27058", "M:T#d560d4"],
        &[],
    ),
    ("Underground Sea", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    (
        "Undergrowth Stadium",
        "F",
        &["M:T#906e31", "M:T#b733ed"],
        &[],
    ),
    (
        "Urza's Cave",
        "F",
        &["M:T#33c5df"],
        &["A:TSM#dfc107|Sequence|t0|"],
    ),
    ("Valakut, the Molten Pinnacle", "F", &["M:T#bb0d56"], &[]),
    ("Vampiric Rites", "F", &[], &["A:M#74c51d|Sequence|t0|"]),
    (
        "Vault of Champions",
        "F",
        &["M:T#988966", "M:T#906e31"],
        &[],
    ),
    (
        "Vault of the Archangel",
        "F",
        &["M:T#33c5df"],
        &["A:TM#2b5c3f|Sequence|t0|"],
    ),
    ("Verdant Catacombs", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Viridescent Bog", "F", &["M:TM#9b420a"], &[]),
    ("Viscera Seer", "F", &[], &["A:#69bd6d|Scry|t0|"]),
    (
        "Vito, Thorn of the Dusk Rose",
        "F",
        &[],
        &["A:M#93b0fe|ApplyContinuousEffect|t0|"],
    ),
    ("Volcanic Island", "F", &["M:T#a58da2", "M:T#bb0d56"], &[]),
    (
        "Voldaren Estate",
        "F",
        &["M:T#33c5df"],
        &[
            "A:T#e84cf5|AddManaAnyColorRestricted|t0|",
            "A:TM#077d0c|CreateToken|t0|",
        ],
    ),
    ("War Room", "F", &["M:T#33c5df"], &[]),
    (
        "Warren Soultrader",
        "F",
        &[],
        &["A:#87bf4f|CreateToken|t0|"],
    ),
    (
        "Wasteland",
        "F",
        &["M:T#33c5df"],
        &["A:TS#f8689e|DestroyPermanent|t1|"],
    ),
    ("Wastewood Verge", "F", &["M:T#b733ed", "M:T#87ef9e"], &[]),
    ("Watery Grave", "F", &["M:T#a58da2", "M:T#906e31"], &[]),
    (
        "Wayfarer's Bauble",
        "F",
        &[],
        &["A:TSM#f98da3|Sequence|t0|"],
    ),
    ("Wellwisher", "F", &[], &["A:T#a692a1|GainLife|t0|"]),
    (
        "Whirlpool Warrior",
        "F",
        &[],
        &["A:SM#e08e3b|WheelHand|t0|"],
    ),
    (
        "Whispersilk Cloak",
        "F",
        &[],
        &["A:M#7be59f|AttachEquipment|t1|s"],
    ),
    (
        "Wight of the Reliquary",
        "F",
        &[],
        &["A:T#6c8a11|Sequence|t0|"],
    ),
    ("Wind-Scarred Crag", "F", &["M:T#bb0d56", "M:T#988966"], &[]),
    (
        "Windbrisk Heights",
        "F",
        &["M:T#988966"],
        &["A:TM#cb4066|PlayExiledCard|t0|c"],
    ),
    ("Windswept Heath", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    (
        "Wirewood Lodge",
        "F",
        &["M:T#33c5df"],
        &["A:TM#044070|UntapPermanent|t1|"],
    ),
    ("Witch's Cottage", "F", &["M:T#906e31"], &[]),
    ("Woe Strider", "F", &[], &["A:#e8f073|Scry|t0|"]),
    ("Wooded Foothills", "F", &[], &["A:TS#ecf1d2|Sequence|t0|"]),
    ("Woodland Cemetery", "F", &["M:T#906e31", "M:T#b733ed"], &[]),
    ("Workhorse", "F", &["M:#3bdfec"], &[]),
    (
        "Yahenni, Undying Partisan",
        "F",
        &[],
        &["A:#e8f073|ApplyContinuousEffect|t0|"],
    ),
    (
        "Yavimaya Coast",
        "F",
        &["M:T#33c5df", "M:T#e1ec5d", "M:T#a27058"],
        &[],
    ),
    (
        "Yavimaya Hollow",
        "F",
        &["M:T#33c5df"],
        &["A:TM#044070|Regenerate|t1|"],
    ),
    (
        "Yawgmoth, Thran Physician",
        "F",
        &[],
        &["A:#87bf4f|Sequence|t1|", "A:M#b0dc50|Proliferate|t0|"],
    ),
    (
        "Zagoth Triome",
        "F",
        &["M:T#906e31", "M:T#b733ed", "M:T#a58da2"],
        &[],
    ),
    ("Zhalfirin Void", "F", &["M:T#33c5df"], &[]),
    (
        "Ziatora's Proving Ground",
        "F",
        &["M:T#906e31", "M:T#bb0d56", "M:T#b733ed"],
        &[],
    ),
];
