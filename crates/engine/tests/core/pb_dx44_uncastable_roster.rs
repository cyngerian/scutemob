//! PB-DX44 census — the four populations "the casts you cannot make" is scoped on,
//! re-derived at HEAD from `all_cards()` (SR-36: enumerate the corpus, never grep source)
//! and **PRINTED rather than transcribed**.
//!
//! # Why this file prints instead of asserting a remembered number
//!
//! PB-DX8's `/review` deleted a calibration table whose published figures did not
//! reproduce against the shipped code, and PB-DX27 repeated the correction: every
//! population a batch reasons about must be printed by the test that measures it, so the
//! next reader re-derives rather than trusts. Every `t_*_report` below runs with
//! `--nocapture` and prints its own membership; the `r*` gates then pin the figure the
//! batch's scope decisions actually rest on.
//!
//! # Why each population is measured on TWO axes
//!
//! Dispatch hygiene 6, and three consecutive batches that learned it the hard way
//! (PB-DX25/25b/25c each found the filed scope short; PB-DX26's inverse census found two
//! defs neither the seed's grep nor the forward walk could see; PB-DX43's inverse axis
//! found a fourth live-wrong def and a third double-grant risk). **A roster derived from
//! one declaration construct measures that construct.** So each population here is taken
//! forward (from the DSL construct the engine reads) and inverse (from the PRINTED text
//! or the type line, which is what the player sees), and where the two disagree the
//! disagreement is the finding.
//!
//! # What membership asserts, and does NOT
//!
//! Membership means only that this def declares this specific shape (PB-DX4's `BASELINE`
//! lesson, same wording). It says nothing about whether the def is otherwise
//! oracle-correct.

use mtg_engine::{AbilityDefinition, AltCostKind, CardDefinition, CardType, KeywordAbility};
use std::collections::BTreeSet;

/// `Completeness::is_complete()` is the deck-legality predicate `validate_deck` enforces
/// (Architecture Invariant 9 / SR-2). Mirrored here rather than re-implemented, exactly
/// as `pb_dx29_additional_cost_roster::deck_legal` does.
fn deck_legal(defs: &[CardDefinition], names: &BTreeSet<String>) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| names.contains(&d.name) && d.completeness.is_complete())
        .map(|d| d.name.clone())
        .collect()
}

fn names_of(defs: &[CardDefinition], pred: impl Fn(&CardDefinition) -> bool) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| pred(d))
        .map(|d| d.name.clone())
        .collect()
}

/// Every face's oracle text, lowercased and concatenated.
///
/// **The multi-face read is load-bearing and PB-DX8 is why**: that batch's first draft
/// read `def.oracle_text` alone while `CardFace` carries its own, which made the axis
/// blind to every transformed face and Adventure half. A split card is precisely a
/// multi-clause printed object, so an inverse axis over printed text that reads one field
/// would be measuring the wrong thing here of all places.
fn printed_text(def: &CardDefinition) -> String {
    let mut s = def.oracle_text.to_lowercase();
    if let Some(face) = &def.back_face {
        s.push('\n');
        s.push_str(&face.oracle_text.to_lowercase());
    }
    if let Some(face) = &def.adventure_face {
        s.push('\n');
        s.push_str(&face.oracle_text.to_lowercase());
    }
    s
}

// ── Axis definitions ─────────────────────────────────────────────────────────────

/// FORWARD axis, pitch: the def declares the construct `casting.rs` reads
/// (`AbilityDefinition::AltCastAbility { kind: AltCostKind::Pitch, .. }`).
fn pitch_forward(def: &CardDefinition) -> bool {
    def.abilities.iter().any(|a| {
        matches!(
            a,
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Pitch,
                ..
            }
        )
    })
}

/// INVERSE axis, pitch: the def PRINTS an alternative cost in CR 118.9's own wording.
///
/// Deliberately broader than the forward axis — it accepts any "rather than pay this
/// spell's mana cost" clause, which is the phrase every pitch card prints and also the
/// phrase several NON-pitch alternative costs print. The gap between the two axes is the
/// finding, not a bug in the needle.
fn pitch_inverse(def: &CardDefinition) -> bool {
    let t = printed_text(def);
    t.contains("rather than pay this spell's mana cost")
}

/// FORWARD axis, split halves: the def carries `AbilityDefinition::Fuse`, which is how
/// the DSL stores a split card's RIGHT half — for fuse cards and, as
/// `pb_dx29_additional_cost_roster::FUSE_DATA_CARRIERS` records, for at least one split
/// card that is a deliberate data carrier with no `KeywordAbility::Fuse` marker.
fn right_half_forward(def: &CardDefinition) -> bool {
    def.abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Fuse { .. }))
}

/// FORWARD axis, fusable: the def carries the `Fuse` KEYWORD, i.e. CR 702.102a actually
/// permits casting both halves together.
fn fuse_marker(def: &CardDefinition) -> bool {
    def.abilities.iter().any(|a| {
        matches!(a, AbilityDefinition::Keyword(KeywordAbility::Fuse))
            || matches!(a, AbilityDefinition::Keyword(k) if *k == KeywordAbility::Fuse)
    })
}

/// INVERSE axis, split cards: the PRINTED name carries the `//` split marker.
///
/// This axis is deliberately over-broad — it catches modal double-faced cards, Adventure
/// creatures, Rooms and Aftermath split cards as well as fuse split cards, because all of
/// them print a `//` name. That over-breadth is the POINT: the question this batch has to
/// answer is "which printed two-halved cards can a player not cast a half of", and a
/// needle narrowed to the construct the engine already handles could never surface a
/// two-halved card the engine models some other way.
fn split_name_inverse(def: &CardDefinition) -> bool {
    def.name.contains(" // ")
}

/// FORWARD axis, Spree: the `KeywordAbility::Spree` marker `casting.rs` gates on.
fn spree_marker(def: &CardDefinition) -> bool {
    def.abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Spree)))
}

/// INVERSE axis, Spree: the def declares `ModeSelection.mode_costs`, i.e. the per-mode
/// cost data `casting.rs` charges and `effective_cast_cost_with_additional` now mirrors —
/// regardless of whether the marker is present.
fn mode_costs_declared(def: &CardDefinition) -> bool {
    def.abilities.iter().any(|a| {
        matches!(
            a,
            AbilityDefinition::Spell {
                modes: Some(m),
                ..
            } if m.mode_costs.is_some()
        )
    })
}

// ── Reports ──────────────────────────────────────────────────────────────────────

/// Prints every population this batch reasons about. Run with `--nocapture`.
///
/// This is the test that makes the execution notes' figures re-derivable rather than
/// remembered; the numbers in `memory/primitives/pb-DX44-execution-notes.md` are copied
/// FROM this output, and if they ever disagree this output wins.
#[test]
fn t_census_report() {
    let defs = mtg_engine::all_cards();
    println!("\n=== PB-DX44 census, {} defs in corpus ===\n", defs.len());

    for (label, fwd, inv) in [
        (
            "PITCH (CR 118.9)",
            names_of(&defs, pitch_forward),
            names_of(&defs, pitch_inverse),
        ),
        (
            "SPLIT RIGHT HALF (CR 702.102a/709.4)",
            names_of(&defs, right_half_forward),
            names_of(&defs, split_name_inverse),
        ),
        (
            "SPREE (CR 702.172a)",
            names_of(&defs, spree_marker),
            names_of(&defs, mode_costs_declared),
        ),
    ] {
        let fwd_legal = deck_legal(&defs, &fwd);
        let inv_legal = deck_legal(&defs, &inv);
        println!("--- {label} ---");
        println!(
            "  forward axis: {} defs, {} deck-legal",
            fwd.len(),
            fwd_legal.len()
        );
        println!("    all:        {fwd:?}");
        println!("    deck-legal: {fwd_legal:?}");
        println!(
            "  inverse axis: {} defs, {} deck-legal",
            inv.len(),
            inv_legal.len()
        );
        println!("    deck-legal: {inv_legal:?}");
        let only_fwd: Vec<_> = fwd.difference(&inv).collect();
        let only_inv: Vec<_> = inv.difference(&fwd).collect();
        println!("  forward-only: {only_fwd:?}");
        println!("  inverse-only: {only_inv:?}");
        println!();
    }

    let fuse_kw = names_of(&defs, fuse_marker);
    println!("--- FUSE KEYWORD (CR 702.102a, both halves together) ---");
    println!("  all: {fuse_kw:?}");
    println!("  deck-legal: {:?}\n", deck_legal(&defs, &fuse_kw));
}

// ── Gates ────────────────────────────────────────────────────────────────────────

/// **R1** — the pitch population, pinned on BOTH axes.
///
/// The `OOS-DX29-3` row names four defs. That is the deck-legal figure, it is ALSO the
/// whole forward population, and it **reproduces exactly** — worth saying out loud,
/// because "the filed site list is a floor" has now held for four consecutive batches and
/// this is a counterexample, the same way `OOS-ENG2-1`'s five-site census was exact.
///
/// **This gate's first draft asserted five, and it was wrong for the reason SR-36
/// exists.** `grep -l "AltCostKind::Pitch" crates/card-defs/src/defs/*.rs` returns five
/// files, so the batch's own hand census recorded a fifth member (`force_of_despair`,
/// `inert`). The def declares no such ability — `force_of_despair.rs:5` merely *mentions*
/// `AltCostKind::Pitch` in a COMMENT explaining what PB-AC5 shipped. A source grep counts
/// the token; `all_cards()` counts the declaration. The census was corrected by executing
/// this test, which is the only reason the notes do not publish a phantom member.
///
/// The INVERSE axis is 14 defs, of which 10 print CR 118.9's phrase and declare no pitch
/// construct — every one of them non-deck-legal, so none is a live-wrong card today. Two
/// (`Gush`, `Mindbreak Trap`) were missed by the batch's own hand grep as well.
#[test]
fn r1_pitch_population_is_pinned_on_both_axes() {
    let defs = mtg_engine::all_cards();
    let fwd = names_of(&defs, pitch_forward);
    let fwd_legal = deck_legal(&defs, &fwd);

    assert_eq!(
        fwd_legal.iter().cloned().collect::<Vec<_>>(),
        vec![
            "Force of Negation".to_string(),
            "Force of Vigor".to_string(),
            "Force of Will".to_string(),
            "Misdirection".to_string(),
        ],
        "the DECK-LEGAL pitch population is the four defs `OOS-DX29-3` names. A member \
         joining or leaving means a human must re-read PB-DX44's scope."
    );
    assert_eq!(
        fwd.len(),
        4,
        "the FORWARD pitch population is exactly the deck-legal four -- no `inert` or \
         `partial` def declares the construct. Measured by walking `all_cards()`, NOT by \
         grepping the defs directory, which returns a fifth file on a comment; got {fwd:?}"
    );

    // The inverse axis is broader by construction and must stay a superset ON THE
    // DECK-LEGAL SET -- if a def ever prints CR 118.9's phrase, is deck-legal, and does
    // NOT declare the construct, it is a live-wrong card and this gate is how it surfaces.
    let inv_legal = deck_legal(&defs, &names_of(&defs, pitch_inverse));
    let printed_but_unimplemented: Vec<_> = inv_legal.difference(&fwd_legal).collect();
    assert!(
        printed_but_unimplemented.is_empty(),
        "these defs are deck-legal and PRINT CR 118.9's alternative-cost phrase while \
         declaring no `AltCastAbility {{ kind: Pitch }}` -- each is a card whose printed \
         alternative cost no client can pay: {printed_but_unimplemented:?}"
    );
}

/// **R2** — the split-card right-half population, pinned, and the deck-legal figure the
/// half-selector's scope rests on.
///
/// `OOS-DX29-9` states the reachable fuse population as **2** and the right-half
/// population as **3**, and both reproduce — but the two figures answer different
/// questions and the row's own text conflates them. `connive_concoct` carries
/// `AbilityDefinition::Fuse` as a deliberate data carrier with NO `Fuse` keyword, so it
/// cannot be fused (2 fusable) while its right half is exactly as uncastable as the other
/// two (3 right halves). **The half-selector therefore serves three defs, not two** —
/// one more than the seed that scoped it claims.
#[test]
fn r2_split_right_half_population_is_pinned() {
    let defs = mtg_engine::all_cards();
    let right = deck_legal(&defs, &names_of(&defs, right_half_forward));
    assert_eq!(
        right.iter().cloned().collect::<Vec<_>>(),
        vec![
            "Connive // Concoct".to_string(),
            "Turn // Burn".to_string(),
            "Wear // Tear".to_string(),
        ],
        "the deck-legal defs carrying a DSL right half. All three gain a right-half cast \
         from PB-DX44's half selector; only the two with the Fuse keyword gain a FUSED \
         cast."
    );

    let fusable = deck_legal(&defs, &names_of(&defs, fuse_marker));
    assert_eq!(
        fusable.iter().cloned().collect::<Vec<_>>(),
        vec!["Turn // Burn".to_string(), "Wear // Tear".to_string()],
        "the deck-legal FUSABLE population (CR 702.102a). `Connive // Concoct` is \
         deliberately absent -- see `pb_dx29_additional_cost_roster::FUSE_DATA_CARRIERS`."
    );
    assert!(
        fusable.is_subset(&right),
        "a def with the Fuse keyword and no `AbilityDefinition::Fuse` would be a \
         marker-only fuse def -- `casting.rs` refuses that cast outright, which is the \
         `galadhrim_brigade` shape one variant over"
    );
}

/// **R3** — every DSL right half's `DeclaredTarget` indices are GLOBALLY OFFSET past the
/// left half's requirement count, which is the contract `resolution.rs` documents and the
/// contract a right-half-ONLY cast has to compensate for.
///
/// This gate exists because the hazard is silent: a right-half-only cast announces its
/// targets at indices `0..n` while the def's effect reads index `left_count + i`, so a
/// mismatch resolves the spell **at nothing** rather than refusing it. Pinning the offset
/// convention is what lets `casting.rs`'s right-half path compensate by a computed amount
/// instead of a remembered one.
/// **Stated residual, so this gate does not overclaim** (PB-DX7's rule: a gate that
/// reports success while checking nothing is worse than no gate). This pins the declared
/// requirement COUNTS, which is what the right-half cast's index compensation is computed
/// from. It does **not** walk each half's `Effect` tree for the `DeclaredTarget { index }`
/// values themselves — that walk must be recursive over every `Effect` nesting site or it
/// measures the shallow ones only (PB-DX26's V6b lesson, and the eleventh nesting site its
/// `/review` found already in the enum). The behavioural half is covered instead by
/// `rules::pb_dx44_split_half_cast`, which casts each right half for real and asserts the
/// effect landed on the announced target.
#[test]
fn r3_right_half_target_counts_are_pinned() {
    let defs = mtg_engine::all_cards();
    let mut measured: Vec<(String, usize, usize)> = Vec::new();
    for def in &defs {
        let Some(right_targets) = def.abilities.iter().find_map(|a| match a {
            AbilityDefinition::Fuse { targets, .. } => Some(targets.len()),
            _ => None,
        }) else {
            continue;
        };
        let left_targets = def
            .abilities
            .iter()
            .find_map(|a| match a {
                AbilityDefinition::Spell { targets, .. } => Some(targets.len()),
                _ => None,
            })
            .unwrap_or(0);
        println!(
            "{}: left declares {left_targets} target(s), right declares {right_targets}",
            def.name
        );
        measured.push((def.name.clone(), left_targets, right_targets));
    }
    measured.sort();
    assert_eq!(
        measured,
        vec![
            // Concoct: "Surveil 3, then you may sacrifice a token..." -- no target.
            ("Connive // Concoct".to_string(), 1, 0),
            // Burn: "deals 2 damage to any target" at global index 1.
            ("Turn // Burn".to_string(), 1, 1),
            // Tear: "destroy target enchantment" at global index 1.
            ("Wear // Tear".to_string(), 1, 1),
        ],
        "the per-half declared target counts every right-half cast's index compensation \
         is computed from. A def changing shape here changes what a right-half-only cast \
         must offset by, and the failure mode of getting it wrong is SILENT (the effect \
         resolves at nothing) rather than a refusal -- which is why this is pinned by \
         value rather than by a floor."
    );
    assert_eq!(
        measured.len(),
        3,
        "non-vacuity floor: expected the three known right-half defs, walked {}",
        measured.len()
    );
}

/// **R6** — `OOS-DX29-13`, TAKEN as a rider: every def's `card_id` is the id
/// `card_name_to_id` mints from its own `name`.
///
/// The seed's own words: *"a wrong `CardId` on a card object is not a build error, not an
/// offer suppression, and not a diagnostic — it is a silently rider-less offer."*
/// `legal_actions::build_additional_cost_plan` opens with
/// `let Some(def) = obj.card_id.and_then(|cid| registry.get(cid)) else { return
/// AdditionalCostPlan::default(); }`, while `enrich_spec_from_def` fills the object's
/// characteristics from the def by **NAME**. So a def whose two keyings disagree produces
/// a well-formed object with a working cast and **no additional-cost riders at all** —
/// and nothing in the tree compares the keyings.
///
/// PB-DX29 found it the hard way on `turn.rs` (`cid("turn")` vs
/// `card_name_to_id("Turn // Burn") == CardId("turn-burn")`), which is exactly the def
/// this batch builds its fuse and right-half fixtures against, so the rider is taken here
/// rather than deferred to PB-DX57: this batch would have paid the cost of the missing
/// gate a fourth time.
///
/// # The seed's proposed fix cannot hold, and executing it is what shows that
///
/// `OOS-DX29-13` prescribes exactly this gate as its "cheap durable fix", phrased as an
/// equality: *"a corpus gate asserting `card_name_to_id(def.name) == def.card_id` for
/// every def"*. Run against the corpus, that equality fails on **50 defs**, not on the one
/// the row names. The prescription is therefore **corrected here**: the population is a
/// pinned floor, not an assertion of emptiness, and the row is updated to say so.
///
/// The 50 fall into four classes, and only the last two are defects:
///
/// 1. **Split / MDFC / Room defs keyed on the FRONT face alone** (~30, e.g.
///    `Turn // Burn` → `cid("turn")`). A deliberate authoring convention.
/// 2. **Transform defs keyed on BOTH faces while the `name` carries only the front**
///    (`Legion's Landing` → `legions-landing-adanto-the-first-fort`,
///    `Growing Rites of Itlimoc`, `Thaumatic Compass`). Class 1's mirror image — the same
///    convention applied in the other direction, which is itself worth noticing: the
///    corpus has two opposite conventions for the same question.
/// 3. **Diacritics** (`Bartolomé del Presidio`, `Éomer, King of Rohan`,
///    `Clavileño, First of the Blessed`). `card_name_to_id` does not fold accents, so the
///    helper — not the def — is what is wrong for these.
/// 4. **Genuine typos with no convention behind them**: `Skrelv's Hive` declares
///    `skrevls-hive` (the `l` and `v` transposed) and `Lae'zel, Vlaakith's Champion`
///    declares `laez-el-...`. Filed as **`OOS-DX44-2`**.
///
/// **Taken as a GATE with a pinned floor, not as a corpus rewrite.** Renaming a shipped
/// `card_id` moves every fixture and golden script that names it, and this batch has a
/// wire bump of its own to keep legible. What the gate buys is that the population cannot
/// grow **silently** — a new def with a mismatched id is a red test, which is the failure
/// `OOS-DX29-13` actually describes.
#[test]
fn r6_card_id_matches_the_id_minted_from_the_name() {
    let defs = mtg_engine::all_cards();
    let mismatched: Vec<(String, String, String)> = defs
        .iter()
        .filter_map(|d| {
            let minted = mtg_engine::testing::replay_harness::card_name_to_id(&d.name);
            (minted != d.card_id).then(|| (d.name.clone(), d.card_id.0.clone(), minted.0.clone()))
        })
        .collect();

    for (name, declared, minted) in &mismatched {
        println!("MISMATCH {name}: declared `{declared}`, name mints `{minted}`");
    }

    let names: Vec<&str> = mismatched.iter().map(|(n, _, _)| n.as_str()).collect();
    assert_eq!(
        names, KNOWN_ID_NAME_MISMATCHES,
        "`OOS-DX29-13`: a def whose `card_id` is not what `card_name_to_id` mints from its \
         `name` builds objects that cast fine and carry NO additional-cost riders, because \
         `enrich_spec_from_def` keys on the name and `build_additional_cost_plan` keys on \
         the id. The list is PINNED rather than empty (renaming a shipped `card_id` moves \
         every fixture and golden script that names it); what this gate forbids is the \
         population GROWING. A new def here must either be renamed or added to \
         `KNOWN_ID_NAME_MISMATCHES` by a human who has read this doc."
    );
}

/// The 50 `card_id`/`name` disagreements that exist at HEAD, pinned by `r6`.
///
/// Each entry is a def that ships today with the `OOS-DX29-13` shape. **This is not an
/// approval list** — it is a floor. Emptying it means picking ONE of the corpus's two
/// opposite front-face/both-faces conventions, teaching `card_name_to_id` to fold
/// diacritics, and fixing the two typos in class 4; that is `PB-DX57`'s work, not this
/// batch's.
const KNOWN_ID_NAME_MISMATCHES: &[&str] = &[
    "Agadeem's Awakening // Agadeem, the Undercrypt",
    "Bala Ged Recovery // Bala Ged Sanctuary",
    "Barkchannel Pathway // Tidechannel Pathway",
    "Bartolomé del Presidio",
    "Beloved Beggar",
    "Blightstep Pathway // Searstep Pathway",
    "Bloodline Keeper",
    "Bloomvine Regent // Claim Territory",
    "Boggart Trawler // Boggart Bog",
    "Bottomless Pool // Locker Room",
    "Braided Net",
    "Bridgeworks Battle // Tanglespan Bridgeworks",
    "Brightclimb Pathway // Grimclimb Pathway",
    "Brutal Cathar",
    "Clavileño, First of the Blessed",
    "Clearwater Pathway // Murkwater Pathway",
    "Commit // Memory",
    "Consign // Oblivion",
    "Cragcrown Pathway // Timbercrown Pathway",
    "Darkbore Pathway // Slitherbore Pathway",
    "Decadent Dragon // Expensive Taste",
    "Delver of Secrets",
    "Docent of Perfection",
    "Dwynen's Elite",
    "Éomer, King of Rohan",
    "Fell the Profane // Fell Mire",
    "Funeral Room // Awakening Hall",
    "Growing Rites of Itlimoc",
    "Hydroelectric Specimen // Hydroelectric Laboratory",
    "Kabira Takedown // Kabira Plateau",
    "Lae'zel, Vlaakith's Champion",
    "Legion's Landing",
    "Malakir Rebirth // Malakir Mire",
    "Marang River Regent // Coil and Catch",
    "Monster Manual // Zoological Study",
    "Needleverge Pathway // Pillarverge Pathway",
    "Riverglide Pathway // Lavaglide Pathway",
    "Rune-Tail, Kitsune Ascendant // Rune-Tail's Essence",
    "Scavenger Regent // Exude Toxin",
    "Sejiri Shelter // Sejiri Glacier",
    "Sink into Stupor // Soporific Springs",
    "Skrelv's Hive",
    "Sundering Eruption // Volcanic Fissure",
    "Suq'Ata Lancer",
    "Thaumatic Compass",
    "Turn // Burn",
    "Turntimber Symbiosis // Turntimber, Serpentine Wood",
    "Valakut Awakening // Valakut Stoneforge",
    "Walk-In Closet // Forgotten Cellar",
    "Witch Enchanter // Witch-Blessed Meadow",
];

/// **R4** — the Spree population, and the marker/cost disagreement the inverse axis found.
///
/// `OOS-DX29-14` states the deck-legal Spree population as exactly **1**
/// (`insatiable_avarice`) and it reproduces. The inverse axis then finds something the
/// row does not mention: **`smugglers_surprise` carries the Spree keyword and declares no
/// `mode_costs` at all**, so `casting.rs` would refuse every cast of it with "spree spell
/// has no per-mode costs defined in ModeSelection". That is `galadhrim_brigade`'s
/// marker-without-cost shape and `nocturnal_hunger`'s cost-without-marker shape — the
/// class `pb_dx29_additional_cost_roster::R2` generalised for the eight
/// `AdditionalCost`-carried kinds — recurring on the one mode-cost mechanic that lives
/// under a different enum and was therefore outside that gate's table. Latent (the def is
/// `partial`), pinned wrong-way-round here, and filed as `OOS-DX44-1`.
#[test]
fn r4_spree_population_and_the_marker_cost_disagreement() {
    let defs = mtg_engine::all_cards();
    let marked = names_of(&defs, spree_marker);
    let costed = names_of(&defs, mode_costs_declared);

    assert_eq!(
        deck_legal(&defs, &marked)
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec!["Insatiable Avarice".to_string()],
        "the deck-legal Spree population is exactly one def, and it was totally \
         uncastable until PB-DX44"
    );

    let marker_without_costs: Vec<_> = marked.difference(&costed).cloned().collect();
    assert_eq!(
        marker_without_costs,
        vec!["Smuggler's Surprise".to_string()],
        "pinned WRONG-WAY-ROUND (`OOS-DX44-1`): a Spree def with no `mode_costs` is \
         refused outright by `casting.rs`. `Smuggler's Surprise` is `partial` so the \
         defect is latent; if this list ever grows a deck-legal member, that member is \
         uncastable and this gate is the only thing that says so."
    );

    let costs_without_marker: Vec<_> = costed.difference(&marked).cloned().collect();
    assert!(
        costs_without_marker.is_empty(),
        "a def declaring `mode_costs` without the Spree marker would have its per-mode \
         costs silently NOT charged -- `casting.rs` reads `mode_costs` only inside its \
         `KeywordAbility::Spree` branch. Offenders: {costs_without_marker:?}"
    );
}

/// **R7** — CR 709.4 timing residual, pinned wrong-way-round on purpose.
///
/// `casting.rs`'s `is_instant_speed` derivation for a right-half-only cast (stage 2a,
/// PB-DX44 `OOS-DX29-9`) deliberately does NOT build a REPLACE-not-OR override the way
/// `casting_with_aftermath` does two blocks up in the same function -- there is no corpus
/// member to prove that machinery on. Instead it reads `chars.card_types` alone, which is
/// CR 709.4-correct only because every right half's own `card_type` happens to match the
/// card's own printed type. This gate is what makes that "happens to" a checked claim
/// rather than an assumption: if a future split card's right half prints a DIFFERENT card
/// type than its left half (the way a real Aftermath sorcery/instant pair can), this test
/// goes red and the residual `casting.rs` documents beside `is_instant_speed` becomes
/// live, not latent.
#[test]
fn r7_right_half_card_type_matches_the_card_printed_type() {
    let defs = mtg_engine::all_cards();
    let mut measured: Vec<(String, CardType, bool)> = Vec::new();
    for def in &defs {
        let Some(right_card_type) = def.abilities.iter().find_map(|a| match a {
            AbilityDefinition::Fuse { card_type, .. } => Some(*card_type),
            _ => None,
        }) else {
            continue;
        };
        let matches = def.types.card_types.contains(&right_card_type);
        println!(
            "{}: right half card_type {right_card_type:?}, def.types.card_types \
             {:?}, matches printed type: {matches}",
            def.name, def.types.card_types
        );
        measured.push((def.name.clone(), right_card_type, matches));
    }
    measured.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        measured.len(),
        3,
        "non-vacuity floor: expected the three known right-half defs, walked {}",
        measured.len()
    );
    let mismatched: Vec<_> = measured
        .iter()
        .filter(|(_, _, m)| !m)
        .map(|(n, _, _)| n.clone())
        .collect();
    assert!(
        mismatched.is_empty(),
        "CR 709.4: these defs' right half prints a card TYPE different from the card's \
         own printed type. `casting.rs`'s right-half-only `is_instant_speed` derivation \
         reads `chars.card_types` alone (never the right half's own `card_type`), which \
         is CR-WRONG for any member of this list -- the timing override documented \
         beside `is_instant_speed` in `casting.rs` must be built: {mismatched:?}"
    );
}

/// **R5** — non-vacuity floors for R1-R4.
///
/// PB-DX26's V4b lesson: a roster gate whose walk finds nothing passes just as green as
/// one whose walk finds everything. Each axis above must actually match something, and
/// the corpus itself must be non-empty.
#[test]
fn r5_non_vacuity_floors() {
    let defs = mtg_engine::all_cards();
    assert!(
        defs.len() > 1_500,
        "the corpus itself looks empty ({} defs) -- `all_cards()` is broken and every \
         gate in this file is vacuous",
        defs.len()
    );
    assert!(
        !names_of(&defs, pitch_forward).is_empty(),
        "pitch forward axis matched nothing"
    );
    assert!(
        !names_of(&defs, pitch_inverse).is_empty(),
        "pitch inverse axis matched nothing"
    );
    assert!(
        !names_of(&defs, right_half_forward).is_empty(),
        "right-half forward axis matched nothing"
    );
    assert!(
        names_of(&defs, split_name_inverse).len() > 10,
        "the `//` inverse axis is supposed to be deliberately over-broad; matching {} \
         defs means the needle has stopped working",
        names_of(&defs, split_name_inverse).len()
    );
    assert!(
        !names_of(&defs, spree_marker).is_empty(),
        "spree forward axis matched nothing"
    );
    assert!(
        !names_of(&defs, mode_costs_declared).is_empty(),
        "mode_costs inverse axis matched nothing"
    );
}

/// **R8** — the deck-legal `Complete` **Escape** population, pinned at ZERO.
///
/// This is the measurement that makes `OOS-DX29-3`'s deferred half safe to
/// defer, and the seed's own row does not mention it.
///
/// The row's argument is that a graveyard cast loop and the `EscapeExile`
/// channel must land together, because `casting.rs:283` auto-detects escape
/// from the zone alone (`casting_from_graveyard && card_has_escape_keyword &&
/// !casting_with_flashback`, no caller opt-in) — so a graveyard loop shipped
/// alone converts "never offered" into a HARD REFUSAL. That argument is
/// correct. What it omits is that **no deck-legal member exists to be refused**:
/// all four corpus Escape defs are `partial` or `known_wrong`.
///
/// The distinction is the difference between a latent defect and an unreachable
/// one, and it is the kind of figure this project has repeatedly found to be
/// load-bearing for a scope decision (PB-DX29's "13 of 15 kinds invisible" was
/// arithmetically right and materially misleading for exactly this reason —
/// four of the thirteen had no deck-legal member at all).
///
/// Pinned WRONG-WAY-ROUND: the day an Escape def is promoted to `Complete`,
/// this gate goes red and says that the graveyard-loop coupling has acquired a
/// real member. Read alongside
/// `pb_dx44_pitch_channel::t7_an_escape_card_in_a_graveyard_is_offered_no_cast_today`,
/// which pins the other half (nothing is offered, so nothing is refused).
#[test]
fn r8_deck_legal_escape_population_is_zero() {
    let defs = mtg_engine::all_cards();
    let escape = names_of(&defs, |d| {
        d.abilities
            .iter()
            .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Escape)))
    });
    println!("Escape defs (any completeness): {escape:?}");
    assert!(
        !escape.is_empty(),
        "non-vacuity floor: the Escape needle matched nothing, so the emptiness \
         asserted below would be meaningless"
    );
    let legal = deck_legal(&defs, &escape);
    assert!(
        legal.is_empty(),
        "`OOS-DX29-3` deferred half: a deck-legal `Complete` Escape def now \
         exists ({legal:?}). The graveyard cast loop's coupling to the \
         `EscapeExile` channel has a REAL member as of this change -- read \
         `t7_an_escape_card_in_a_graveyard_is_offered_no_cast_today` and the \
         `OOS-DX29-3` registry row before adding a graveyard cast loop."
    );
}
