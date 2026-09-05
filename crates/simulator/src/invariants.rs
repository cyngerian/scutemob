//! Invariant checks run after every state transition during fuzzing.
//!
//! **Twelve checks exist; ten of them fire from [`check_all`]**, one of those ten being a
//! deliberate no-op, and two are END-OF-GAME checks that run once per game instead.
//!
//! From [`check_all`]: zone integrity, ID uniqueness, stack consistency, player
//! consistency, turn order, object-zone agreement, attachment validity, **attachment
//! symmetry** (PB-DX56), game progression, orphaned tokens — plus
//! `check_mana_non_negative`, which cannot fail because `ManaPool` is `u32`.
//!
//! Not from [`check_all`], deliberately: [`check_no_leaked_tokens`] (PB-DX32 Stage 4) and
//! [`check_no_dangling_attachment_at_rest`] (PB-DX56). Both are END-OF-GAME checks — they
//! run at `LocalGame::result_snapshot`, the one site both real terminal paths go through,
//! and each is the strictly stronger property that keeps one transient split honest.
//! `t_every_end_state_check_is_called_from_result_snapshot` is what stops either call from
//! being deleted in silence, which it could be until PB-DX56 (`OOS-DX56-5`).
//!
//! **This header said "Ten checks exist; nine of them fire from `check_all`" until
//! PB-DX56, and PB-DX56's own first draft left it saying so** — a count in a module doc
//! is a claim like any other and it rots the moment a check is added. `check_all`'s call
//! count is the ground truth; count it there before trusting this paragraph.
//!
//! This header used to say "12 checks", and `docs/mtg-engine-simulator.md` still
//! lists twelve. Two of those twelve (legal-action soundness, SBA idempotency)
//! have never been written; SIM-3 (`scutemob-177`) re-derived the list from
//! [`check_all`] and marked them there. Filed as `OOS-SIM3-2`. PB-DX32 Stage 2 serves
//! legal-action soundness at RUN scope (`report.rs::MAX_BOT_REJECTION_PER_MILLE` and
//! `print_sr38_summary`) rather than as a `check_all` function — see
//! `docs/mtg-engine-simulator.md`'s checklist for the current disposition. SBA
//! idempotency (the module's own #11) is still unwritten.

use mtg_engine::{GameState, ObjectId, ZoneId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};

/// The `check` name of the CR 704.3 / `OOS-M11-7` token class: a token in a
/// non-battlefield zone at a checkpoint, cleared by the next SBA sweep. Split out of the
/// hard bucket by PB-DX32 Stage 4 and answered by [`check_no_leaked_tokens`].
pub const TRANSIENT_ORPHANED_TOKENS: &str = "no_orphaned_tokens";

/// The `check` name of the CR 800.4j class (PB-DX56, `OOS-DX32-1`): `turn.active_player`
/// naming a player who has left the game.
///
/// **Transient because CR 800.4j says the turn "continues to its completion without an
/// active player" and this engine cannot say that** — see [`check_player_consistency`].
/// It is BOUNDED to the remainder of that turn, and the bound is CR 800.4k: *"If a player
/// who has left the game would begin a turn, that turn doesn't begin."*
/// `rules/turn_structure.rs::next_player_in_turn_order` skips `has_lost || has_conceded`,
/// and PB-DX56's F2 closed `advance_turn`'s extra-turn branch, which applied no liveness
/// filter at all and was the one route by which this condition was UNBOUNDED.
///
/// **The strictly stronger property that keeps this split honest is therefore a
/// TURN-BOUNDARY rule, not an end-state one**, and it is enforced by
/// `LocalGame::record_violations`: the same departed player reported as active at a
/// STRICTLY GREATER turn number than the one the condition was first seen at is promoted
/// to the HARD bucket as [`HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN`]. An end-state check
/// would be the wrong shape here and would fire spuriously: when the game ends, the last
/// player to die may legitimately still be `active_player`.
pub const TRANSIENT_DEPARTED_ACTIVE_PLAYER: &str = "departed_active_player";

/// The promotion of [`TRANSIENT_DEPARTED_ACTIVE_PLAYER`] that is NOT transient: the
/// condition survived a turn boundary, which CR 800.4k forbids.
pub const HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN: &str =
    "departed_active_player_crossed_a_turn";

/// The `check` name of the CR 800.4a class (PB-DX56, `OOS-DX32-1`): `priority_holder`
/// naming a player who has left the game. **Hard** — CR 800.4a's last sentence is
/// unconditional. Measured at ZERO on the standard invocation, which is the evidence that
/// keeping it hard costs nothing.
pub const HARD_DEPARTED_PRIORITY_HOLDER: &str = "departed_priority_holder";

/// The `check` name of the CR 704.5m / 704.5n class (`OOS-DX22-8`): a battlefield object
/// whose `attached_to` names an `ObjectId` that is not a key of `state.objects`.
///
/// **Transient, and PB-DX56 established the exact iff.** The two SBA arms
/// (`rules/sba.rs::check_aura_sbas`, `::check_equipment_sbas`) clear this unless the
/// attacher is phased out or its LAYER-RESOLVED subtypes contain none of
/// `Aura`/`Equipment`/`Fortification`. What makes it survive a checkpoint is the same
/// CR 704.3 timing deviation as the token class (`OOS-M11-7`): the engine sweeps SBAs at
/// nine sites and `rules/{abilities,casting,combat,mana,turn_actions}.rs` contain **zero**,
/// so a permanent that leaves the battlefield while paying a cost dangles across the
/// invariant check and heals at the next step entry or resolution.
///
/// Answered by TWO strictly stronger properties, because one would not be enough:
/// [`check_no_dangling_attachment_at_rest`] (the end state must be clean) and
/// [`check_attachment_symmetry`] (the direction that NEVER heals, which had no check at
/// all until PB-DX56 — see that function).
pub const TRANSIENT_ATTACHMENT_VALIDITY: &str = "attachment_validity";

/// The `check` name of the direction of the attachment relation that never heals — see
/// [`check_attachment_symmetry`]. **Hard and per-command**: no SBA is supposed to clean this
/// up, so there is no CR 704.3 window to excuse a report.
pub const HARD_ATTACHMENT_SYMMETRY: &str = "attachment_symmetry";

/// The `check` name of [`check_no_dangling_attachment_at_rest`], the END-STATE answer to
/// [`TRANSIENT_ATTACHMENT_VALIDITY`].
pub const HARD_DANGLING_ATTACHMENT_AT_REST: &str = "dangling_attachment_at_rest";

/// The seat an [`InvariantViolation`]'s evidence names as the ARM's own subject, if it has
/// one (PB-DX56).
///
/// **The key is `arm_player=`, not `player=`, and the difference is the whole of
/// `OOS-DX56-1`.** [`check_all`] PREPENDS [`state_context`], which emits one
/// `player=PlayerId(n) life=… has_lost=…` line PER SEAT, so a `player=` lookup returns the
/// first state-context line — a value identical for every violation in the game. Reading
/// the seat is factored here rather than open-coded at the consumer so there is exactly
/// one place that knows which key means what.
pub fn arm_player_of(v: &InvariantViolation) -> Option<&str> {
    v.evidence
        .iter()
        .find_map(|e| e.strip_prefix("arm_player="))
}

/// The CR 800.4k decision, as a PURE function so it can be tested without building a game
/// (`OOS-DX56-1`'s companion finding: the promotion had no test of any kind, and a plant
/// that made it never fire left the whole workspace green).
///
/// `first_seen` is the turn number at which this seat's
/// [`TRANSIENT_DEPARTED_ACTIVE_PLAYER`] condition was first observed. CR 800.4j lets the
/// condition hold for the remainder of THAT turn; CR 800.4k — *"If a player who has left
/// the game would begin a turn, that turn doesn't begin."* — is what forbids it holding
/// into a later one. So a report at a **strictly greater** turn number is the promotion,
/// and anything at or before it is the bounded window.
pub fn crosses_a_turn_boundary(check: &str, turn_number: u32, first_seen: u32) -> bool {
    check == TRANSIENT_DEPARTED_ACTIVE_PLAYER && turn_number > first_seen
}

/// Is this `check` name a known-transient class — reported, but not counted toward the hard
/// bucket and not halting `--stop-on-error`?
///
/// **One arithmetic, deliberately.** Before PB-DX56 the classification was a bare literal
/// `if v.check == "no_orphaned_tokens"` inside `LocalGame::record_violations`, and adding a
/// second and third transient class by editing that literal is how a class ends up
/// transient in one consumer and hard in another. Every consumer calls this instead.
///
/// **Stated precisely, because the obvious stronger claim is false at HEAD**:
/// `tests/local_game_playthrough.rs` does NOT hold a second copy — PB-DX32's own `/review`
/// (finding M1) already found and deleted the duplicate branch there, and that file now
/// reads the split from `LocalGame`'s two accessors. This function exists to keep it that
/// way as the transient set GROWS, not to repair a drift that currently exists.
pub fn is_transient_check(check: &str) -> bool {
    matches!(
        check,
        TRANSIENT_ORPHANED_TOKENS
            | TRANSIENT_DEPARTED_ACTIVE_PLAYER
            | TRANSIENT_ATTACHMENT_VALIDITY
    )
}

/// An invariant violation found during fuzzing.
///
/// # `evidence` (PB-DX56 / OOS-FB1-1)
///
/// `check` and `description` name WHAT is wrong; `evidence` is the check's own
/// account of the state it was found in -- structured facts a human (or a later
/// pass of tooling) can use to decide what to do about it, rather than two
/// `ObjectId`s and a prose sentence. [`check_all`] prepends a common state
/// snapshot (turn/phase/step/active player/priority holder/every player's
/// life-and-status, see [`state_context`]) to every violation's `evidence` after
/// collection; individual checks may append their OWN facts on top of that (see
/// [`check_player_consistency`] and [`check_attachment_validity`] for the two that
/// do, as of this batch).
///
/// **Deliberately NOT part of [`distinct`]'s dedupe key.** `evidence` carries
/// PER-INSTANCE facts (in particular the turn number the state snapshot was taken
/// at), so the same underlying condition observed at two different checkpoints
/// produces two different `evidence` vectors even though it is still one defect.
/// Folding `evidence` into the key would turn ONE defect-shaped condition into N
/// "distinct" conditions -- one per checkpoint it happened to be observed at --
/// which is exactly the defect-shaped-number property `OOS-SIM3-3` exists to
/// preserve (see [`distinct`]'s own doc).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InvariantViolation {
    pub check: String,
    pub description: String,
    pub turn_number: u32,
    /// The check's own evidence for this instance -- see this struct's doc.
    /// `#[serde(default)]` so a pre-PB-DX56 crash-report JSON (which has no
    /// `evidence` field at all) still deserializes.
    #[serde(default)]
    pub evidence: Vec<String>,
}

/// Common state facts prepended (by [`check_all`]) to every violation's
/// `evidence` -- the "dumps the violating state" half of `OOS-FB1-1`. One
/// arithmetic used by every checkpoint rather than each check reaching for it
/// independently: a check that forgets to call a shared helper cannot happen if
/// there is only one call site, in `check_all` itself, after collection.
fn state_context(state: &GameState) -> Vec<String> {
    let turn = state.turn();
    let mut ctx = vec![
        format!("turn={}", turn.turn_number),
        format!("phase={:?}", turn.phase),
        format!("step={:?}", turn.step),
        format!("active_player={:?}", turn.active_player),
        format!("priority_holder={:?}", turn.priority_holder),
    ];
    for (id, p) in state.players().iter() {
        ctx.push(format!(
            "player={:?} life={} has_lost={} has_conceded={}",
            id, p.life_total, p.has_lost, p.has_conceded
        ));
    }
    ctx.push(format!("turn_order={:?}", turn.turn_order));
    ctx
}

/// Run all invariant checks on a game state. Returns violations found.
pub fn check_all(state: &GameState, prev_turn: Option<u32>) -> Vec<InvariantViolation> {
    let mut violations = Vec::new();

    check_zone_integrity(state, &mut violations);
    check_id_uniqueness(state, &mut violations);
    check_mana_non_negative(state, &mut violations);
    check_stack_consistency(state, &mut violations);
    check_player_consistency(state, &mut violations);
    check_turn_order(state, &mut violations);
    check_object_zone_agreement(state, &mut violations);
    check_attachment_validity(state, &mut violations);
    check_attachment_symmetry(state, &mut violations);
    if let Some(prev) = prev_turn {
        check_game_progression(state, prev, &mut violations);
    }
    check_no_orphaned_tokens(state, &mut violations);

    // PB-DX56 / OOS-FB1-1: prepend the common state snapshot to EVERY violation
    // found this checkpoint, in ONE place, after collection -- not inside each
    // check. A check-local call could be forgotten by a future check; a single
    // call site here cannot be, because every violation any check produces flows
    // through this vec before `check_all` returns it. Per-check evidence (see
    // `check_player_consistency` / `check_attachment_validity`) is appended AFTER
    // this common context, so it reads general-first, specific-second.
    if !violations.is_empty() {
        let ctx = state_context(state);
        for v in &mut violations {
            let mut evidence = ctx.clone();
            evidence.append(&mut v.evidence);
            v.evidence = evidence;
        }
    }

    violations
}

/// 1. Zone integrity: every object in exactly one zone
fn check_zone_integrity(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let mut object_zones: HashMap<ObjectId, Vec<ZoneId>> = HashMap::new();

    for (zone_id, zone) in state.zones().iter() {
        for obj_id in zone.object_ids() {
            object_zones.entry(obj_id).or_default().push(*zone_id);
        }
    }

    // Check for objects in multiple zones
    for (obj_id, zones) in &object_zones {
        if zones.len() > 1 {
            violations.push(InvariantViolation {
                check: "zone_integrity".into(),
                description: format!(
                    "Object {:?} found in {} zones: {:?}",
                    obj_id,
                    zones.len(),
                    zones
                ),
                turn_number: state.turn().turn_number,
                evidence: Vec::new(),
            });
        }
    }

    // Check for objects not in any zone
    for (obj_id, _obj) in state.objects().iter() {
        if !object_zones.contains_key(obj_id) {
            violations.push(InvariantViolation {
                check: "zone_integrity".into(),
                description: format!("Object {:?} not found in any zone", obj_id),
                turn_number: state.turn().turn_number,
                evidence: Vec::new(),
            });
        }
    }
}

/// 2. ID uniqueness: no duplicate ObjectIds across all zones
fn check_id_uniqueness(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let mut seen: HashSet<ObjectId> = HashSet::new();
    for (_zone_id, zone) in state.zones().iter() {
        for obj_id in zone.object_ids() {
            if !seen.insert(obj_id) {
                violations.push(InvariantViolation {
                    check: "id_uniqueness".into(),
                    description: format!("Duplicate ObjectId {:?} across zones", obj_id),
                    turn_number: state.turn().turn_number,
                    evidence: Vec::new(),
                });
            }
        }
    }
}

/// 3. Mana non-negative: all mana pool values >= 0
fn check_mana_non_negative(_state: &GameState, _violations: &mut Vec<InvariantViolation>) {
    // ManaPool uses u32 fields, so they can't go negative.
    // This check is a no-op but kept for documentation and future-proofing.
}

/// The card in `ZoneId::Stack` that this stack object owns, if it owns one.
///
/// # Why this is an exhaustive match and not a two-arm `if let`
///
/// Only some `StackObjectKind`s put a *card* on the stack. Which ones is not a
/// property of the kind's name, and it has already been got wrong once by
/// enumerating the kinds that "obviously" do (see
/// [`check_stack_consistency`]'s history note): `MutatingCreatureSpell` is a
/// cast spell whose card sits in `ZoneId::Stack` exactly like `Spell`'s does —
/// `casting.rs::handle_cast_spell` performs the same
/// `move_object_to_zone(card, ZoneId::Stack)` and then chooses between the two
/// kinds afterwards, on `cast_with_mutate` alone.
///
/// So the classification is made once, here, over **every** variant. Adding a
/// `StackObjectKind` is a compile error in this function until someone decides
/// which side the new variant is on — the same forcing function SR-5 applies to
/// `KeywordAbility`. Guessing from the variant name is what produced the defect
/// this check exists to have fixed.
///
/// `None` means "this stack object moved no card": every ability and trigger
/// kind, whose `source_object` (where it has one) "remains in whatever zone it
/// is in" per `StackObjectKind`'s own docs.
///
/// **Deliberately duplicated**, not delegated to,
/// `mtg_engine::state::stack_registry::card_in_stack_zone` (added PB-DX25,
/// `OOS-SIM3-5`). That function is the ENGINE's own classification, consumed by
/// `Effect::CounterSpell` and `resolution::counter_stack_object` to decide
/// which card to move when a spell is countered — the exact defect this check
/// exists to catch. If this verifier simply read the engine's answer back, a
/// wrong `Some`/`None` there would make the check agree with the defect and go
/// silent, in precisely the case it was written for. What keeps the two
/// answers honest without coupling them: both are exhaustive with no wildcard
/// arm (a new `StackObjectKind` variant is a compile error in BOTH crates
/// independently), and a behavioural cross-check
/// (`crates/simulator/tests/pb_dx25_counter_on_mutate_is_consistent.rs`) proves
/// the two agree on the case that matters by running a real counter-on-mutate
/// game and asserting zero `stack_consistency` violations, rather than by
/// sharing code. See `stack_registry`'s own doc comment for the mirror image
/// of this note.
fn stack_card_of(kind: &mtg_engine::StackObjectKind) -> Option<ObjectId> {
    use mtg_engine::StackObjectKind as K;
    match kind {
        // CR 601.2c — the card is moved to `ZoneId::Stack` as part of casting.
        // `MutatingCreatureSpell` is the same cast down the same code path
        // (`casting.rs`: one `move_object_to_zone`, then a `cast_with_mutate`
        // branch that picks the kind), CR 702.140a / CR 729.2.
        K::Spell { source_object } | K::MutatingCreatureSpell { source_object, .. } => {
            Some(*source_object)
        }
        // Everything below puts an ability or a trigger on the stack and moves
        // no card there. Several of them *do* move a card somewhere — Madness
        // and Miracle name a card in exile and in hand respectively, Ninjutsu
        // and the graveyard-recursion abilities move their source at
        // resolution — but none of those destinations is `ZoneId::Stack`, which
        // is the only thing this classification is about.
        K::ActivatedAbility { .. }
        | K::LoyaltyAbility { .. }
        | K::TriggeredAbility { .. }
        | K::MadnessTrigger { .. }
        | K::MiracleTrigger { .. }
        | K::UnearthAbility { .. }
        | K::SuspendCounterTrigger { .. }
        | K::SuspendCastTrigger { .. }
        | K::NinjutsuAbility { .. }
        | K::EmbalmAbility { .. }
        | K::EternalizeAbility { .. }
        | K::EncoreAbility { .. }
        | K::ForecastAbility { .. }
        | K::ScavengeAbility { .. }
        | K::BloodrushAbility { .. }
        | K::SaddleAbility { .. }
        | K::TransformTrigger { .. }
        | K::CraftAbility { .. }
        | K::DayboundTransformTrigger { .. }
        | K::TurnFaceUpTrigger { .. }
        | K::KeywordTrigger { .. }
        | K::RoomAbility { .. }
        | K::RingAbility { .. }
        | K::ClassLevelAbility { .. }
        | K::DelayedActionTrigger { .. } => None,
    }
}

/// 4. Stack consistency: `ZoneId::Stack` holds exactly the cards named by the stack
///    objects that put a card there — one apiece, in the same order.
///
/// # This check used to compare two different id spaces (M11-local S8)
///
/// It previously asserted `zone(Stack).object_ids() == stack_objects().map(|so| so.id)`,
/// as sets. Those two sets are **never** equal in a healthy game, and the reason is
/// structural rather than incidental:
///
/// * `casting.rs::handle_cast_spell` first does
///   `state.move_object_to_zone(card, ZoneId::Stack)`, which mints a fresh `ObjectId`
///   for the card under CR 400.7 — call it *n*;
/// * it then does `let stack_entry_id = state.next_object_id()` — *n+1* — and that is
///   the `StackObject::id`.
///
/// So every ordinary spell cast produced **two** violations at once ("*n* in Stack zone
/// but not in stack_objects" and "*n+1* in stack_objects but not in Stack zone"), and
/// every activated or triggered ability produced **one** (an ability puts a
/// `StackObject` on the stack but moves no card, so its id has no Stack-zone
/// counterpart and never could).
///
/// This is the same id-space confusion `tools/play-server/src/view.rs`'s `NameIndex`
/// documents from the other side — `StackObject::id` and the Stack-zone `ObjectId` are
/// different namespaces that both count from small integers, so a comparison between
/// them type-checks and means nothing.
///
/// Measured on this branch (SIM-3), the shipped check against the reverted one, same
/// builds, same seeds:
///
/// | run | old check | this check |
/// |---|---|---|
/// | `local_game_playthrough`, seed 1 | 720 (638 + 82) | 0 |
/// | `mtg-fuzzer --games 5 --seed 1 --max-turns 200` | 8,781 (7,575 + 1,206) | 0 |
///
/// Every *other* check's count is identical on both sides of that A/B (929
/// `no_orphaned_tokens` and 9 `player_consistency` in the fuzz run), so the measurement
/// moves this check and nothing else. 8,781 of that run's 9,719 total violations —
/// **90.3%** — were this one check being wrong, which is the concrete size of the noise
/// floor `OOS-DP3-9` and `OOS-M11-3` were reading their "70,719 violations" baseline
/// through.
///
/// **What the clean side of that A/B was not evidence of** — SIM-3's own qualification,
/// kept because it dates the table above. Both rows were measured on the *unshuffled*
/// instrument: `bin/fuzzer.rs` never shuffled a library, so the first spell in those games
/// was cast around **turn 143** and the whole 5-game run contained ~150 of them, none
/// involving a counter, a copy, a mutate or a suspend; the playthrough is a fixed human
/// script that casts only untargeted spells. So that run proved the check quiet on
/// ordinary casts, and the properties below rested on the structural argument, not on it.
///
/// **Re-measured post-shuffle by PB-DX22 (`scutemob-196`), which is this check's first
/// real test.** `bin/fuzzer.rs` now shuffles (CR 103.3) and registers the commander
/// (CR 903.6), so the first cast lands on game turn **3-29** instead of 143-154 and a
/// 20-game run casts **670** spells. Stated like-for-like, because the two sides of
/// this A/B were measured at different game counts and quoting them against each other
/// is the sampling error this very block exists to warn about: the pre-fix instrument
/// cast **121** spells in **5** games (`pb-dx22-measurement-head.txt`) and **1,519**
/// violations in **20**; the post-fix one casts **670** in 20. Per game that is
/// 24.2 -> 33.5 casts, and the real change is not the count but the DEPTH — 143-154
/// -> 3-29 — since every pre-fix cast happened after the basics ran out. Same command
/// as the table above,
/// widened to `--games 20 --seed 1 --max-turns 200 --threads 1 --profile fuzz`:
///
/// | check | violations | games |
/// |---|---|---|
/// | `no_orphaned_tokens` | 301 | 15 of 20 |
/// | `player_consistency` | 114 | 5 of 20 |
/// | `attachment_validity` | 11 | 3 of 20 (seeds 5, 9, 15) |
/// | **`stack_consistency`** | **0** | **0** |
/// | total | **426** | 16 of 20 |
///
/// **That is the COMPLETE tally over all 20 games, and it was not when this paragraph was
/// first written.** As shipped, PB-DX22 asserted "426 total violations, and not one of
/// them is `stack_consistency`" from the **94** lines the binary prints — it prints
/// per-violation detail for the first five offending games only
/// (`bin/fuzzer.rs`, `if violation_seeds.len() <= 5`) — i.e. a universal negative over 426
/// from a 22% sample, in the very block whose sampling caveat it was correcting. Its fix
/// cycle made the binary print a by-`check` histogram over **every** game
/// (`print_violation_histogram`), and the table above is that histogram's output, recorded
/// verbatim in `memory/primitives/pb-dx22-measurement-after-fixcycle.txt`.
///
/// The conclusion survived; the sample did not represent the population. The 94 printed
/// lines were 90 `no_orphaned_tokens` + 3 `attachment_validity` + 1 `player_consistency`,
/// which projects `player_consistency` at ~1% of the run; it is **27%**. Read the
/// histogram, never the detail loop, for any claim about what did or did not fire.
///
/// So the clean side now IS evidence about games with real spells in them — for the first
/// time — though still not about counters, copies, mutates or suspends specifically.
/// Recorded as `OOS-DX22-3`.
///
/// Two live engine defects that legitimately trip this check are filed as `OOS-SIM3-5`
/// precisely because the earlier evidence could not have caught them — read a
/// `stack_consistency` violation as a real finding, which is the point of both batches.
///
/// **Correction (PB-DX25 review fix cycle, kept here rather than silently edited
/// away — the paragraph below already corrects it in substance, this note flags
/// the sentence itself as known-wrong so a reader stops at THIS line, not just
/// the next one):** "two live... trip THIS check" is false as originally written.
/// Of `OOS-SIM3-5`'s three shapes, only shape (c) was live, and shape (c) produces
/// NO `stack_consistency` divergence (the card and its stack entry both survive
/// consistently — the countered spell just resolves anyway, silently). Shapes
/// (a) and (b) were both UNREACHABLE on the corpus at the time this sentence was
/// written (plan §2.2 / §2.3). Zero of the three ever tripped this specific check.
///
/// **PB-DX25 closes `OOS-SIM3-5`** (not deleted from the history above, since the
/// finding is what motivated the fix): `Effect::CounterSpell`'s zone-move used to
/// decide "does this stack object own a card" by matching the `StackObjectKind`
/// variant NAME rather than asking the question this file already asks correctly —
/// so countering a `MutatingCreatureSpell` was a silent no-op (the countered spell
/// resolved anyway) rather than a `stack_consistency`-visible card-in-Stack leak.
/// The fix lives in the engine (`mtg_engine::state::stack_registry::
/// card_in_stack_zone`, deliberately duplicated here rather than delegated to —
/// see [`stack_card_of`]'s doc comment), not in this check; this check was never
/// the thing that was wrong.
///
/// # What is actually invariant
///
/// [`stack_card_of`] decides, per `StackObjectKind`, whether a stack object owns a card
/// in `ZoneId::Stack`. Given that:
///
/// 1. every non-copy card-owning stack object's card is in `ZoneId::Stack`;
/// 2. every object in `ZoneId::Stack` is the card of some card-owning stack object;
/// 3. no two non-copy stack objects claim the **same** card (CR 400.7 mints a fresh
///    `ObjectId` on every move onto the stack, so a shared claim is impossible —
///    `MR-M11-14`, closed here);
/// 4. the two sequences agree in **order**. Both are appended to together and removed
///    from together (`casting.rs` pushes the zone entry then the stack object;
///    countering and resolution take out the pair), so the Stack zone's contents are
///    the card-owning stack objects' cards read in stack order, with the ability and
///    trigger entries — which own no card — skipped over. Checked only when 1–3 all
///    hold, so a set disagreement is reported once as itself rather than twice with an
///    order complaint on top.
///
/// **Copies are excluded from (1), (3) and (4), and only from those** (CR 707.10):
/// `copy.rs` clones the original's `kind` wholesale, so a copy's `source_object` names
/// the *original's* card — correct while the original is still on the stack, and
/// dangling the moment the original is countered, without anything being wrong. A copy
/// adds no Stack-zone object, so it cannot make (2) fail either way.
fn check_stack_consistency(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let before = violations.len();

    // Ordered: `ZoneId::Stack` is built by `Zone::new_ordered()` (`builder.rs`).
    let stack_zone_ids: Vec<ObjectId> = match state.zone(&ZoneId::Stack) {
        Ok(zone) => zone.object_ids(),
        Err(_) => Vec::new(),
    };
    let zone_set: HashSet<ObjectId> = stack_zone_ids.iter().copied().collect();

    // The cards claimed by the non-copy stack objects that own one, in stack order.
    let mut claimed_order: Vec<ObjectId> = Vec::new();
    let mut claim_counts: HashMap<ObjectId, usize> = HashMap::new();
    for so in state.stack_objects().iter() {
        let Some(card) = stack_card_of(&so.kind) else {
            continue;
        };
        if so.is_copy {
            continue;
        }
        claimed_order.push(card);
        *claim_counts.entry(card).or_insert(0) += 1;
        // (1)
        if !zone_set.contains(&card) {
            violations.push(InvariantViolation {
                check: "stack_consistency".into(),
                description: format!(
                    "Stack object {:?} names card {:?}, which is not in the Stack zone",
                    so.id, card
                ),
                turn_number: state.turn().turn_number,
                evidence: Vec::new(),
            });
        }
    }

    // (2)
    for id in &stack_zone_ids {
        if !claim_counts.contains_key(id) {
            violations.push(InvariantViolation {
                check: "stack_consistency".into(),
                description: format!(
                    "Object {:?} is in the Stack zone but no stack object names it as its card",
                    id
                ),
                turn_number: state.turn().turn_number,
                evidence: Vec::new(),
            });
        }
    }

    // (3) MR-M11-14 / CR 400.7.
    for (card, count) in &claim_counts {
        if *count > 1 {
            violations.push(InvariantViolation {
                check: "stack_consistency".into(),
                description: format!(
                    "Card {:?} in the Stack zone is claimed by {} non-copy stack objects; CR \
                     400.7 mints a fresh ObjectId per move onto the stack, so at most one can \
                     name it",
                    card, count
                ),
                turn_number: state.turn().turn_number,
                evidence: Vec::new(),
            });
        }
    }

    // (4) Only meaningful once 1-3 hold — see the doc comment.
    if violations.len() == before && claimed_order != stack_zone_ids {
        violations.push(InvariantViolation {
            check: "stack_consistency".into(),
            description: format!(
                "Stack zone order {:?} does not match the order the stack objects claim their \
                 cards in ({:?})",
                stack_zone_ids, claimed_order
            ),
            turn_number: state.turn().turn_number,
            evidence: Vec::new(),
        });
    }
}

/// 5. Player consistency: active player and priority holder are alive.
///
/// # This check reports TWO conditions and the CR gives them OPPOSITE dispositions
///
/// PB-DX56 (`OOS-DX32-1`) measured 189 of 189 reports on the ACTIVE-PLAYER arm and
/// **zero** on the priority-holder arm, and then read the two rules:
///
/// * **CR 800.4j**, verbatim: *"If a player leaves the game during their turn, that turn
///   continues to its completion **without an active player**."* `TurnState::active_player`
///   is a bare `PlayerId`, not an `Option`, with exactly ONE production write site
///   (`rules/turn_structure.rs`, inside `advance_turn`), so *"without an active player"* is
///   **inexpressible in this engine's state type** and it necessarily encodes that turn by
///   leaving the departed player's id in the field. Everything CR 800.4j actually requires
///   is discharged elsewhere: the departed seat never receives priority
///   (`rules/priority.rs::grant_priority_to_active_player`, which cites 800.4j by name) and
///   never acts (`rules/engine.rs::validate_player_active`). So this arm asserts a
///   REPRESENTATION CHOICE, not a rules violation — it is [`TRANSIENT_DEPARTED_ACTIVE_PLAYER`].
/// * **CR 800.4a**, last sentence, verbatim: *"If the player who left the game had priority
///   at the time they left, priority passes to the next player in turn order who's still in
///   the game."* Unconditional, with no "continues without" escape. There is no state in
///   which a departed player legitimately holds priority, so this arm is a REAL DEFECT and
///   stays hard — [`HARD_DEPARTED_PRIORITY_HOLDER`].
///
/// **The two arms therefore carry different `check` names.** They used to share
/// `"player_consistency"`, which is why the registry row, the v4 memo cell and the dispatch
/// criterion all treat them as one class; folding a CR-permitted representation and a real
/// CR 800.4a defect into one bucket is what made a quarter of the fuzzer's HARD signal
/// undiagnosable. The transient half is answered by the strictly stronger CR 800.4k
/// turn-boundary property — see [`TRANSIENT_DEPARTED_ACTIVE_PLAYER`].
fn check_player_consistency(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let active = state.turn().active_player;
    if let Ok(p) = state.player(active) {
        if p.has_lost || p.has_conceded {
            violations.push(InvariantViolation {
                check: TRANSIENT_DEPARTED_ACTIVE_PLAYER.into(),
                description: format!("Active player {:?} has lost or conceded", active),
                turn_number: state.turn().turn_number,
                evidence: player_consistency_evidence(state, "active_player", active, p),
            });
        }
    }

    if let Some(priority) = state.turn().priority_holder {
        if let Ok(p) = state.player(priority) {
            if p.has_lost || p.has_conceded {
                violations.push(InvariantViolation {
                    check: HARD_DEPARTED_PRIORITY_HOLDER.into(),
                    description: format!("Priority holder {:?} has lost or conceded", priority),
                    turn_number: state.turn().turn_number,
                    evidence: player_consistency_evidence(state, "priority_holder", priority, p),
                });
            }
        }
    }
}

/// Evidence for [`check_player_consistency`] (PB-DX56 / OOS-FB1-1). The description
/// says "has lost or conceded", but CR 704.5a (loss, an SBA) and CR 104.3a
/// (concession, a special action a player takes) are different CR situations with
/// different likely fixes, so `arm`, `has_lost` and `has_conceded` are all reported
/// SEPARATELY rather than left folded into one prose sentence. Also reports whether
/// ANY OTHER player is still alive: "the whole table is out and the game should have
/// ended" and "one straggler is out but everyone else is fine" are different failure
/// shapes with different fixes.
fn player_consistency_evidence(
    state: &GameState,
    arm: &str,
    id: mtg_engine::PlayerId,
    p: &mtg_engine::PlayerState,
) -> Vec<String> {
    let any_other_player_alive = state
        .players()
        .iter()
        .any(|(other_id, other)| *other_id != id && !other.has_lost && !other.has_conceded);
    vec![
        format!("arm={arm}"),
        // `arm_player=`, deliberately NOT `player=`. `check_all` PREPENDS
        // `state_context`, which emits one `player=PlayerId(n) life=... has_lost=...` line
        // PER SEAT, so a consumer looking for the arm's own subject with a `player=` prefix
        // finds the FIRST state-context line instead -- a value that is identical for every
        // violation in the game and therefore collapses two different departed seats into
        // one. That is not hypothetical: it is exactly what
        // `LocalGame::promote_if_it_crossed_a_turn`'s first draft did, and it manufactured
        // a false CR 800.4k promotion on fuzz seed 5 by keying PlayerId(4)'s turn-154
        // report against PlayerId(1)'s turn-133 one. Pinned by
        // `t_arm_player_key_is_not_shadowed_by_state_context`.
        format!("arm_player={id:?}"),
        format!("has_lost={}", p.has_lost),
        format!("has_conceded={}", p.has_conceded),
        format!("any_other_player_alive={any_other_player_alive}"),
    ]
}

/// 6. Turn order: all players in turn_order
fn check_turn_order(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let active_players = state.active_players();
    for p in &active_players {
        if !state.turn().turn_order.contains(p) {
            violations.push(InvariantViolation {
                check: "turn_order".into(),
                description: format!("Active player {:?} not in turn_order", p),
                turn_number: state.turn().turn_number,
                evidence: Vec::new(),
            });
        }
    }
}

/// 7. Object-zone agreement: object's zone field matches containing zone
fn check_object_zone_agreement(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    for (zone_id, zone) in state.zones().iter() {
        for obj_id in zone.object_ids() {
            if let Ok(obj) = state.object(obj_id) {
                if obj.zone != *zone_id {
                    violations.push(InvariantViolation {
                        check: "object_zone_agreement".into(),
                        description: format!(
                            "Object {:?} has zone {:?} but found in zone {:?}",
                            obj_id, obj.zone, zone_id
                        ),
                        turn_number: state.turn().turn_number,
                        evidence: Vec::new(),
                    });
                }
            }
        }
    }
}

/// 8. Attachment validity: attached_to references existing battlefield objects
fn check_attachment_validity(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    for obj in state.objects_in_zone(&ZoneId::Battlefield) {
        if let Some(target_id) = obj.attached_to {
            if state.object(target_id).is_err() {
                violations.push(InvariantViolation {
                    check: TRANSIENT_ATTACHMENT_VALIDITY.into(),
                    description: format!(
                        "Object {:?} attached to {:?} which doesn't exist",
                        obj.id, target_id
                    ),
                    turn_number: state.turn().turn_number,
                    evidence: attachment_validity_evidence(state, obj, target_id),
                });
            }
        }
    }
}

/// Evidence for [`check_attachment_validity`] (PB-DX56 / OOS-FB1-1, `OOS-DX22-8`).
/// The whole point of this seed is that `Object A attached to B which doesn't exist`
/// names two `ObjectId`s and nothing that lets anyone decide between CR 704.5m (an
/// Aura with an illegal attachment goes to the graveyard) and CR 704.5n (an
/// Equipment/Fortification with an illegal attachment stays on the battlefield,
/// merely unattached) -- and the ATTACHER's card type is the first thing that
/// decides between them, so it is the first thing reported here.
///
/// The target's own last-known information is included when the engine has one:
/// `GameState::lki_objects()` is a `pub` read-only accessor onto the CR 113.7a /
/// 608.2h LKI store (`state/mod.rs`) -- contrary to this task's brief, which
/// expected that accessor might not be public, it already is, so no engine change
/// was needed to reach it. A target this check has never seen an LKI snapshot for
/// (never left the battlefield with the stack non-empty, or the snapshot was
/// already cleared) reports that explicitly rather than silently omitting the
/// fact.
fn attachment_validity_evidence(
    state: &GameState,
    obj: &mtg_engine::GameObject,
    target_id: ObjectId,
) -> Vec<String> {
    let mut evidence = vec![
        format!("attacher={:?}", obj.id),
        format!("attacher_name={:?}", obj.characteristics.name),
        format!("attacher_card_types={:?}", obj.characteristics.card_types),
        format!("attacher_subtypes={:?}", obj.characteristics.subtypes),
        format!("attacher_controller={:?}", obj.controller),
        format!("attacher_owner={:?}", obj.owner),
        format!("attacher_is_token={}", obj.is_token),
        format!("attacher_phased_out={}", obj.status.phased_out),
        format!("attacher_zone={:?}", obj.zone),
        format!("attacher_attachments={:?}", obj.attachments),
        format!("target={target_id:?}"),
        "target_present_in_state_objects=false".to_string(),
    ];
    match state.lki_objects().get(&target_id) {
        Some(lki) => {
            evidence.push(format!("target_lki_name={:?}", lki.characteristics.name));
            evidence.push(format!(
                "target_lki_card_types={:?}",
                lki.characteristics.card_types
            ));
        }
        None => evidence.push(
            "target_lki=<no snapshot: target never left the battlefield with the stack \
             non-empty, or the snapshot was already cleared>"
                .to_string(),
        ),
    }
    evidence
}

/// 9. Game progression: turn number never decreases
fn check_game_progression(
    state: &GameState,
    prev_turn: u32,
    violations: &mut Vec<InvariantViolation>,
) {
    if state.turn().turn_number < prev_turn {
        violations.push(InvariantViolation {
            check: "game_progression".into(),
            description: format!(
                "Turn number decreased from {} to {}",
                prev_turn,
                state.turn().turn_number
            ),
            turn_number: state.turn().turn_number,
            evidence: Vec::new(),
        });
    }
}

/// 10. No orphaned tokens: no tokens in non-battlefield zones after SBAs.
///
/// Tokens in graveyard/exile are cleaned up by SBAs. A report here is a CHECKPOINT
/// ARTEFACT, not a defect, in deviation from CR 704.3 (`OOS-M11-7`):
/// `LocalGame::record_violations` (PB-DX32 Stage 4) splits every report from this
/// function out of the hard `violations` bucket into `transient_violations`, and
/// [`check_no_leaked_tokens`] answers the split with the strictly stronger end-state
/// property at both real terminal paths — a token still off the battlefield when the
/// game is OVER *is* a hard violation.
fn check_no_orphaned_tokens(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    for (obj_id, obj) in state.objects().iter() {
        if obj.is_token && obj.zone != ZoneId::Battlefield && obj.zone != ZoneId::Stack {
            // Tokens can briefly exist on the stack (e.g., copy of a spell).
            // But in graveyard/exile/hand they should be cleaned up by SBAs.
            violations.push(InvariantViolation {
                check: TRANSIENT_ORPHANED_TOKENS.into(),
                description: format!(
                    "Token {:?} '{}' found in zone {:?}",
                    obj_id, obj.characteristics.name, obj.zone
                ),
                turn_number: state.turn().turn_number,
                evidence: Vec::new(),
            });
        }
    }
}

/// The direction of the attachment relation that **never heals**, and that nothing in this
/// workspace looked at before PB-DX56 (`OOS-DX22-8`).
///
/// [`check_attachment_validity`] watches one side of a two-sided relation: a HOST leaves,
/// and the attacher's `attached_to` dangles. That side is cleared by CR 704.5m /
/// CR 704.5n. **The other side has no state-based action at all.** When an ATTACHER leaves
/// the battlefield by any route other than the six sites that clean up (`rules/sba.rs`'s
/// two arms and `effects/mod.rs`'s four equip/unequip paths) — destroyed by an effect,
/// bounced, exiled — `GameState::move_object_to_zone` retires its id while performing only
/// two cross-object fix-ups (CR 702.95e soulbond and the replacement-effect GC), so the
/// **host keeps the dead `ObjectId` in `attachments` for the rest of the game.**
///
/// That matters because `attachments` is not decorative: it is HASHED
/// (`state/hash.rs`), so a stale entry perturbs `public_state_hash` AND
/// `loop_detection::compute_mandatory_state_hash` — CR 104.4b mandatory-loop detection can
/// fail to recognise a repeated board state; it is read by the CR 510.3a equipped-creature
/// combat-damage trigger family; it is walked by CR 702.26g/h phasing through
/// `expect_object_mut`, an IMPOSSIBLE-class SR-4 lookup that fires a `debug_assert`, so a
/// stale entry is a latent debug-build panic; and it is rendered to the browser.
///
/// PB-DX56's **F1** closes the supply. This function is the run-scale assertion that it
/// stays closed, and it is HARD and per-command rather than end-state: unlike the token
/// class there is no CR 704.3 window here, because no SBA is supposed to be doing this
/// cleanup at all — the pointer is simply garbage the moment the attacher's id is retired.
///
/// Two conditions, both directions of the same relation:
/// 1. every `ObjectId` in a battlefield object's `attachments` must resolve;
/// 2. and it must point back — `attachment.attached_to == Some(host)`.
///
/// (2) is not redundant with (1): an equip that moved an Equipment to a new host without
/// clearing the old host's list leaves a LIVE id in the wrong list, which (1) cannot see.
fn check_attachment_symmetry(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    for host in state.objects_in_zone(&ZoneId::Battlefield) {
        for att_id in host.attachments.iter() {
            match state.object(*att_id) {
                Err(_) => violations.push(InvariantViolation {
                    check: HARD_ATTACHMENT_SYMMETRY.into(),
                    description: format!(
                        "Object {:?} lists {:?} in its attachments, but that object does \
                         not exist",
                        host.id, att_id
                    ),
                    turn_number: state.turn().turn_number,
                    evidence: vec![
                        format!("direction=host_lists_dead_attacher"),
                        format!("host={:?}", host.id),
                        format!("host_name={:?}", host.characteristics.name),
                        format!("host_attachments={:?}", host.attachments),
                        format!("dead_attacher={att_id:?}"),
                    ],
                }),
                Ok(att) => {
                    if att.attached_to != Some(host.id) {
                        violations.push(InvariantViolation {
                            check: HARD_ATTACHMENT_SYMMETRY.into(),
                            description: format!(
                                "Object {:?} lists {:?} in its attachments, but that \
                                 object's attached_to is {:?}",
                                host.id, att_id, att.attached_to
                            ),
                            turn_number: state.turn().turn_number,
                            evidence: vec![
                                format!("direction=host_lists_attacher_that_points_elsewhere"),
                                format!("host={:?}", host.id),
                                format!("host_name={:?}", host.characteristics.name),
                                format!("host_attachments={:?}", host.attachments),
                                format!("attacher={att_id:?}"),
                                format!("attacher_name={:?}", att.characteristics.name),
                                format!("attacher_attached_to={:?}", att.attached_to),
                                format!("attacher_zone={:?}", att.zone),
                            ],
                        });
                    }
                }
            }
        }
    }
}

/// The strictly stronger END-STATE property that keeps the
/// [`TRANSIENT_ATTACHMENT_VALIDITY`] split honest (PB-DX56, `OOS-DX22-8`) — the same shape
/// [`check_no_leaked_tokens`] gives the token class.
///
/// [`check_attachment_validity`] reports a dangling `attached_to` at EVERY checkpoint until
/// the next SBA sweep clears it, which makes the report transient by construction (see that
/// constant). This function asks the question that would be a real defect: is any dangling
/// attachment still there when the game is **OVER**? By then CR 704.5m and CR 704.5n have
/// had every sweep in the game to run, so a survivor is a permanent blind spot — an
/// attacher that is phased out, or whose layer-resolved subtypes contain none of
/// `Aura`/`Equipment`/`Fortification` — and not a checkpoint artefact.
///
/// Run once per game at both real terminal paths via `LocalGame::result_snapshot`, and
/// folded into the HARD bucket.
pub fn check_no_dangling_attachment_at_rest(state: &GameState) -> Vec<InvariantViolation> {
    let mut violations = Vec::new();
    for obj in state.objects_in_zone(&ZoneId::Battlefield) {
        if let Some(target_id) = obj.attached_to {
            if state.object(target_id).is_err() {
                violations.push(InvariantViolation {
                    check: HARD_DANGLING_ATTACHMENT_AT_REST.into(),
                    description: format!(
                        "Object {:?} is still attached to {:?}, which doesn't exist, in \
                         the FINAL state",
                        obj.id, target_id
                    ),
                    turn_number: state.turn().turn_number,
                    evidence: attachment_validity_evidence(state, obj, target_id),
                });
            }
        }
    }
    for v in violations.iter_mut() {
        let mut ctx = state_context(state);
        ctx.append(&mut v.evidence);
        v.evidence = ctx;
    }
    violations
}

/// The strictly stronger END-STATE property that keeps the PB-DX32 Stage 4 noise-floor
/// split honest (`OOS-SIM3-3` / `OOS-SIM3-4`).
///
/// [`check_no_orphaned_tokens`] reports a token in a non-battlefield zone at EVERY
/// checkpoint until the next SBA sweep clears it — in deviation from CR 704.3
/// (`OOS-M11-7`), this engine checks SBAs on step entry and at resolution, not
/// whenever a player would get priority as the rule actually requires — which makes
/// that report transient by construction. This function asks the question that would
/// actually be a bug: is any token anywhere but the battlefield when the game is OVER?
/// Measured 0 on all five HEAD seeds (`memory/primitive-wip.md`, Stage 0). Mirrors
/// `crates/simulator/tests/local_game_playthrough.rs:464-472`'s own end-of-playthrough
/// read, generalized from a five-seed script harness to every `LocalGame` terminal path
/// (`LocalGame::result_snapshot`).
///
/// Not called from [`check_all`] — this is an end-of-game check, and `check_all` runs
/// per command.
///
/// **Deliberately stricter than its sibling**: unlike [`check_no_orphaned_tokens`],
/// this function does NOT exempt `ZoneId::Stack` — a token cannot legitimately be on
/// the stack once the game is OVER (contrast a token copy of a spell mid-game, which
/// is the case the sibling's exemption exists for), so a stack-zone token here is a
/// hard `leaked_tokens` violation. Faithful to
/// `crates/simulator/tests/local_game_playthrough.rs:472-476`'s own end-state read, which
/// makes the same choice, and measured 0/20 at Stage 4 (no seed has ever exercised
/// this branch).
pub fn check_no_leaked_tokens(state: &GameState) -> Vec<InvariantViolation> {
    let mut violations = Vec::new();
    for (obj_id, obj) in state.objects().iter() {
        if obj.is_token && obj.zone != ZoneId::Battlefield {
            violations.push(InvariantViolation {
                check: "leaked_tokens".into(),
                description: format!(
                    "Token {:?} '{}' found in zone {:?} at game end",
                    obj_id, obj.characteristics.name, obj.zone
                ),
                turn_number: state.turn().turn_number,
                evidence: Vec::new(),
            });
        }
    }
    violations
}

/// First occurrence per `(check, description)`, order preserved (PB-DX32 Stage 4,
/// `OOS-SIM3-3`'s "report distinct conditions alongside the raw count" prescription).
///
/// Neither `check` nor `description` carries a turn number (that is a separate field),
/// which is why the collapse works: the same underlying condition reported at every
/// checkpoint produces identical `(check, description)` pairs and dedupes to one.
/// `InvariantViolation` derives no `PartialEq`/`Hash` (it is a wire type, not a
/// set-keyed one — see its own doc), so this dedupes on the two `String` fields
/// directly via a `BTreeSet<(String, String)>` rather than deriving anything onto it.
///
/// **`evidence` (PB-DX56) is deliberately NOT part of the key, for the same reason
/// `turn_number` never was.** `check_all` stamps every violation's `evidence` with a
/// state snapshot that includes the turn number, so the SAME underlying condition
/// reported at two different checkpoints carries two DIFFERENT `evidence` vectors
/// even though it is still one defect. Folding `evidence` into the key would turn
/// one defect-shaped condition into N "distinct" conditions — one per checkpoint it
/// happened to be observed at — which is exactly the defect-shaped-number property
/// this function exists to preserve. The FIRST occurrence's evidence is what
/// survives (`out.push(v.clone())` below), which is also the earliest and therefore
/// most useful-for-diagnosis snapshot.
pub fn distinct(violations: &[InvariantViolation]) -> Vec<InvariantViolation> {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut out = Vec::new();
    for v in violations {
        if seen.insert((v.check.clone(), v.description.clone())) {
            out.push(v.clone());
        }
    }
    out
}

/// SIM-3 (`scutemob-177`): probes for [`check_stack_consistency`], both directions.
///
/// # Why this module exists at all
///
/// Before SIM-3 this file had **no** test module across 306 lines, and the check it
/// most needed one for was a false positive by construction that shipped for months
/// and flooded every fuzz artefact the project produced. A check with no test is a
/// check nobody can tell apart from a check that always passes, which is exactly what
/// the rewrite risks becoming in the other direction — so every probe below is paired:
/// a healthy state that must be silent, and a deliberately broken one that must not
/// be.
///
/// The states are hand-built rather than played out. That is the point: the properties
/// under test are ones no legal sequence of `Command`s can violate, so the only way to
/// prove the check would *catch* them is to construct them. The healthy-state probes
/// are built to mirror `casting.rs::handle_cast_spell` exactly — the Stack-zone card
/// first, then a stack object whose id is a *different, larger* number — because the
/// consecutive-integer relationship between those two ids is the whole reason the old
/// check looked plausible.
#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::{
        GameStateBuilder, ObjectSpec, PlayerId, StackObject, StackObjectKind, SubType,
    };

    fn p(n: u64) -> PlayerId {
        PlayerId(n)
    }

    /// Just this check's violations, so a hand-built state tripping some *other*
    /// invariant (a bare `GameStateBuilder` state has no library, for instance)
    /// cannot make a probe pass or fail for the wrong reason.
    fn stack_violations(state: &GameState) -> Vec<String> {
        let mut v = Vec::new();
        check_stack_consistency(state, &mut v);
        v.into_iter().map(|x| x.description).collect()
    }

    /// A stack object of `kind`, built the way `resolution.rs`'s suspend free-cast
    /// builds its `Spell` entry — `StackObject::trigger_default` with the cast flags
    /// at their defaults.
    fn stack_obj(id: u64, kind: StackObjectKind) -> StackObject {
        StackObject::trigger_default(ObjectId(id), p(1), kind)
    }

    /// A two-player state with `stack_names` in `ZoneId::Stack` (in that order) and
    /// one card in hand. Returns the Stack-zone ids in zone order, then the hand id —
    /// a real object that is genuinely *not* on the stack, which is what the
    /// "names a card that is not in the Stack zone" probes need.
    fn state_with(stack_names: &[&str]) -> (GameState, Vec<ObjectId>, ObjectId) {
        let mut b = GameStateBuilder::new().add_player(p(1)).add_player(p(2));
        for name in stack_names {
            b = b.object(ObjectSpec::card(p(1), name).in_zone(ZoneId::Stack));
        }
        b = b.object(ObjectSpec::card(p(1), "Card In Hand").in_zone(ZoneId::Hand(p(1))));
        let state = b.build().expect("builder state");

        let ids = state
            .zone(&ZoneId::Stack)
            .expect("Stack zone exists")
            .object_ids();
        assert_eq!(
            ids.len(),
            stack_names.len(),
            "fixture did not put every card on the stack"
        );
        let hand = state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == "Card In Hand")
            .map(|(id, _)| *id)
            .expect("hand card");
        (state, ids, hand)
    }

    /// T1 — a healthy one-spell stack is silent.
    ///
    /// Non-vacuity is asserted, not assumed: the fixture must actually have a card in
    /// the Stack zone and a stack object naming it, or "0 violations" would prove
    /// nothing at all.
    #[test]
    fn t1_healthy_single_spell_stack_is_silent() {
        let (mut state, ids, _hand) = state_with(&["Lightning Bolt"]);
        let card = ids[0];
        state.stack_objects_mut().push_back(stack_obj(
            card.0 + 1,
            StackObjectKind::Spell {
                source_object: card,
            },
        ));

        assert_eq!(state.zone(&ZoneId::Stack).unwrap().object_ids().len(), 1);
        assert_eq!(state.stack_objects().len(), 1);
        assert!(
            stack_violations(&state).is_empty(),
            "healthy stack must be silent: {:?}",
            stack_violations(&state)
        );
    }

    /// T2 — the historical record: on T1's *healthy* state, the pre-S8 check fired
    /// twice.
    ///
    /// This is the fail-before evidence pinned in code rather than left in a commit
    /// message. It re-implements the old comparison verbatim (`zone(Stack).object_ids()`
    /// against `stack_objects().map(|so| so.id)`, as sets, both directions) and asserts
    /// it produces exactly the two-violation pair the doc comment describes — one per
    /// direction, on consecutive integers, in a state with nothing wrong with it.
    ///
    /// If this ever stops producing 2, the id-space relationship this check's whole
    /// design rests on has changed and the doc comment above is stale.
    #[test]
    fn t2_the_pre_s8_check_fired_twice_on_that_same_healthy_state() {
        let (mut state, ids, _hand) = state_with(&["Lightning Bolt"]);
        let card = ids[0];
        state.stack_objects_mut().push_back(stack_obj(
            card.0 + 1,
            StackObjectKind::Spell {
                source_object: card,
            },
        ));

        let zone_ids: HashSet<ObjectId> = state
            .zone(&ZoneId::Stack)
            .unwrap()
            .object_ids()
            .into_iter()
            .collect();
        let entry_ids: HashSet<ObjectId> = state.stack_objects().iter().map(|so| so.id).collect();
        let old_violations =
            zone_ids.difference(&entry_ids).count() + entry_ids.difference(&zone_ids).count();

        assert_eq!(
            old_violations, 2,
            "the pre-S8 check's two-per-spell false positive is the premise of the rewrite"
        );
        assert!(
            stack_violations(&state).is_empty(),
            "…and this check says 0"
        );
    }

    /// T3 — direction (1): a stack object naming a card that is not in the Stack zone.
    ///
    /// The card named is a real object in hand, so this is a genuine
    /// stack-object/zone divergence and not a dangling-id lookup failure.
    #[test]
    fn t3_spell_naming_a_card_outside_the_stack_zone_fires() {
        let (mut state, _ids, hand) = state_with(&[]);
        state.stack_objects_mut().push_back(stack_obj(
            500,
            StackObjectKind::Spell {
                source_object: hand,
            },
        ));

        let v = stack_violations(&state);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("not in the Stack zone"), "{v:?}");
    }

    /// T4 — direction (2): a card sitting in the Stack zone that no stack object claims.
    #[test]
    fn t4_orphaned_stack_zone_card_fires() {
        let (state, ids, _hand) = state_with(&["Lightning Bolt"]);
        assert_eq!(state.stack_objects().len(), 0);

        let v = stack_violations(&state);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("no stack object names it"), "{v:?}");
        assert!(v[0].contains(&format!("{:?}", ids[0])), "{v:?}");
    }

    /// T5 — CR 707.10: a copy naming a card that has left the stack is *not* a
    /// violation, and the exclusion is load-bearing.
    ///
    /// Both halves are asserted. The same stack object with `is_copy = false` must
    /// fire, or "copies are excluded" would be an untested claim about a branch that
    /// might as well not be there.
    #[test]
    fn t5_copy_exclusion_is_real_and_load_bearing() {
        let (mut state, _ids, hand) = state_with(&[]);
        let mut so = stack_obj(
            500,
            StackObjectKind::Spell {
                source_object: hand,
            },
        );
        so.is_copy = true;
        state.stack_objects_mut().push_back(so);
        assert!(
            stack_violations(&state).is_empty(),
            "a copy's dangling source_object is CR 707.10, not a defect"
        );

        state.stack_objects_mut().clear();
        let mut so = stack_obj(
            500,
            StackObjectKind::Spell {
                source_object: hand,
            },
        );
        so.is_copy = false;
        state.stack_objects_mut().push_back(so);
        assert_eq!(
            stack_violations(&state).len(),
            1,
            "the exclusion must be the only thing that silenced the copy"
        );
    }

    /// T6 — property (3), `MR-M11-14`: two non-copy stack objects claiming one card.
    ///
    /// CR 400.7 mints a fresh `ObjectId` every time a card moves onto the stack, so
    /// this state is unreachable — which is precisely why nothing but an invariant
    /// check would ever notice it.
    #[test]
    fn t6_two_non_copies_claiming_one_card_fires() {
        let (mut state, ids, _hand) = state_with(&["Lightning Bolt"]);
        let card = ids[0];
        for entry in [card.0 + 1, card.0 + 2] {
            state.stack_objects_mut().push_back(stack_obj(
                entry,
                StackObjectKind::Spell {
                    source_object: card,
                },
            ));
        }

        let v = stack_violations(&state);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(
            v[0].contains("claimed by 2 non-copy stack objects"),
            "{v:?}"
        );
    }

    /// T7 — property (4): the two sequences disagree in order.
    ///
    /// Set-equal, no duplicates, nothing dangling — the *only* thing wrong is that the
    /// Stack zone holds the two cards bottom-to-top and the stack objects name them
    /// top-to-bottom. Nothing before SIM-3 looked at order at all.
    #[test]
    fn t7_order_disagreement_fires() {
        let (mut state, ids, _hand) = state_with(&["Lightning Bolt", "Counterspell"]);
        for (n, card) in ids.iter().rev().enumerate() {
            state.stack_objects_mut().push_back(stack_obj(
                100 + n as u64,
                StackObjectKind::Spell {
                    source_object: *card,
                },
            ));
        }

        let v = stack_violations(&state);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("does not match the order"), "{v:?}");

        // …and in the right order it is silent, so T7 is testing order and not a
        // second copy of T3/T4.
        state.stack_objects_mut().clear();
        for (n, card) in ids.iter().enumerate() {
            state.stack_objects_mut().push_back(stack_obj(
                100 + n as u64,
                StackObjectKind::Spell {
                    source_object: *card,
                },
            ));
        }
        assert!(
            stack_violations(&state).is_empty(),
            "{:?}",
            stack_violations(&state)
        );
    }

    /// T8 — **the SIM-3 finding**: a mutate cast is silent.
    ///
    /// `casting.rs::handle_cast_spell` moves the card into `ZoneId::Stack` and *then*
    /// branches on `cast_with_mutate` to choose between `Spell` and
    /// `MutatingCreatureSpell` (CR 702.140a / CR 729.2). The S8 rewrite classified on
    /// the `Spell` variant alone, on the stated premise that every Stack-zone move
    /// "ends in that same `Spell` kind" — which this state is the counterexample to:
    /// before [`stack_card_of`], this fired direction (2) on every mutate cast, a
    /// false positive of exactly the shape the rewrite existed to delete.
    ///
    /// Revert `stack_card_of`'s `MutatingCreatureSpell` arm to `None` and this test
    /// fails; that is the discrimination.
    ///
    /// This test's discrimination is over THIS crate's own `stack_card_of` --
    /// `mtg_engine::state::stack_registry::card_in_stack_zone` (PB-DX25) is a
    /// second, independent classification of the same question, consumed by
    /// `Effect::CounterSpell` and `resolution::counter_stack_object`. See
    /// `stack_card_of`'s doc comment for why the two are not shared.
    #[test]
    fn t8_mutating_creature_spell_owns_its_stack_card() {
        let (mut state, ids, _hand) = state_with(&["Gemrazer"]);
        let card = ids[0];
        state.stack_objects_mut().push_back(stack_obj(
            card.0 + 1,
            StackObjectKind::MutatingCreatureSpell {
                source_object: card,
                target: ObjectId(9_999),
            },
        ));

        assert!(
            stack_violations(&state).is_empty(),
            "a mutate cast puts its card in the Stack zone exactly as a plain cast \
             does: {:?}",
            stack_violations(&state)
        );
    }

    /// T9 — the fuzz-shaped half of the old false positive: an ability on the stack
    /// owns no card, and that is not a divergence.
    ///
    /// This is the arm that produced 7,575 of the 8,781 violations the pre-S8 check
    /// emitted over five fuzz games — those games cast no spells at all, so *every*
    /// stack object in them was an ability or a trigger.
    #[test]
    fn t9_an_ability_on_the_stack_owns_no_card() {
        let (mut state, _ids, hand) = state_with(&[]);
        state.stack_objects_mut().push_back(stack_obj(
            500,
            StackObjectKind::ActivatedAbility {
                source_object: hand,
                ability_index: 0,
                embedded_effect: None,
            },
        ));

        assert!(
            stack_violations(&state).is_empty(),
            "an activated ability moves no card to the stack: {:?}",
            stack_violations(&state)
        );
    }

    /// T10 — the check is actually wired into [`check_all`].
    ///
    /// T1–T9 call `check_stack_consistency` directly, which would keep passing if
    /// someone deleted its line from `check_all`. This one goes through the front
    /// door, in both directions.
    #[test]
    fn t10_check_all_dispatches_to_this_check() {
        let (mut state, ids, _hand) = state_with(&["Lightning Bolt"]);
        let named = |s: &GameState| {
            check_all(s, None)
                .into_iter()
                .filter(|v| v.check == "stack_consistency")
                .count()
        };
        assert_eq!(named(&state), 1, "orphaned Stack-zone card, via check_all");

        let card = ids[0];
        state.stack_objects_mut().push_back(stack_obj(
            card.0 + 1,
            StackObjectKind::Spell {
                source_object: card,
            },
        ));
        assert_eq!(named(&state), 0, "…and silent once it is claimed");
    }

    /// PB-DX56 / OOS-FB1-1: two violations differing ONLY in `evidence` still
    /// dedupe to one. This is the load-bearing claim behind `evidence` being
    /// excluded from `distinct`'s key -- see that function's own doc for the
    /// reasoning; this test is the executed proof.
    #[test]
    fn t_distinct_ignores_evidence_only_differences() {
        let a = InvariantViolation {
            check: "no_orphaned_tokens".into(),
            description: "Token ObjectId(1) 'Spirit' found in zone Graveyard(PlayerId(1))".into(),
            turn_number: 3,
            evidence: vec!["turn=3".into()],
        };
        let b = InvariantViolation {
            check: "no_orphaned_tokens".into(),
            description: "Token ObjectId(1) 'Spirit' found in zone Graveyard(PlayerId(1))".into(),
            turn_number: 4,
            evidence: vec!["turn=4".into(), "phase=PreCombatMain".into()],
        };
        let deduped = distinct(&[a.clone(), b]);
        assert_eq!(deduped.len(), 1, "{deduped:?}");
        assert_eq!(
            deduped[0].evidence, a.evidence,
            "the FIRST occurrence's evidence must be kept, not merged or dropped"
        );
    }

    /// PB-DX56 / OOS-FB1-1: `check_player_consistency`'s evidence names the ARM
    /// (active_player vs priority_holder) and reports `has_lost`/`has_conceded`
    /// separately, per this task's brief -- CR 704.5a (loss) and CR 104.3a
    /// (concession) are different CR situations with different likely fixes.
    #[test]
    fn t_player_consistency_evidence_names_the_arm_and_conceded_flag() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .build()
            .expect("builder state");

        // Builder default: priority_holder == active_player == p(1). Move
        // priority to p(2) and mark p(2) conceded, so the ACTIVE player is fine
        // and only the PRIORITY HOLDER arm should fire.
        state.turn_mut().priority_holder = Some(p(2));
        state
            .players_mut()
            .get_mut(&p(2))
            .expect("p2 exists")
            .has_conceded = true;

        let mut v = Vec::new();
        check_player_consistency(&state, &mut v);
        assert_eq!(v.len(), 1, "{v:?}");
        let evidence = &v[0].evidence;
        assert!(
            evidence.contains(&"arm=priority_holder".to_string()),
            "{evidence:?}"
        );
        assert!(
            evidence.contains(&"has_conceded=true".to_string()),
            "{evidence:?}"
        );
        assert!(
            evidence.contains(&"has_lost=false".to_string()),
            "{evidence:?}"
        );
        assert!(
            evidence.contains(&"any_other_player_alive=true".to_string()),
            "p(1) is untouched and must still read alive: {evidence:?}"
        );
    }

    /// …and the ACTIVE PLAYER arm reports itself, not the other arm, when it is
    /// the active player who has lost.
    #[test]
    fn t_player_consistency_evidence_names_the_active_player_arm() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .build()
            .expect("builder state");
        // Builder default: priority_holder == active_player == p(1). Move
        // priority to p(2) (who has NOT lost) so only the ACTIVE PLAYER arm fires
        // -- otherwise both arms would fire off the same lost p(1), since p(1) is
        // the default priority holder too.
        state.turn_mut().priority_holder = Some(p(2));
        state
            .players_mut()
            .get_mut(&p(1))
            .expect("p1 exists")
            .has_lost = true;

        let mut v = Vec::new();
        check_player_consistency(&state, &mut v);
        assert_eq!(v.len(), 1, "{v:?}");
        let evidence = &v[0].evidence;
        assert!(
            evidence.contains(&"arm=active_player".to_string()),
            "{evidence:?}"
        );
        assert!(
            evidence.contains(&"has_lost=true".to_string()),
            "{evidence:?}"
        );
        assert!(
            evidence.contains(&"has_conceded=false".to_string()),
            "{evidence:?}"
        );
    }

    /// PB-DX56 / OOS-FB1-1: `check_attachment_validity`'s evidence names the
    /// attacher's card types -- the fact CR 704.5m (Aura -> graveyard) vs
    /// CR 704.5n (Equipment -> stays, unattached) is decided by first.
    #[test]
    fn t_attachment_validity_evidence_names_the_attachers_card_types() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .object(
                ObjectSpec::enchantment(p(1), "Test Aura")
                    .with_subtypes(vec![SubType("Aura".to_string())])
                    .in_zone(ZoneId::Battlefield),
            )
            .build()
            .expect("builder state");

        let aura_id = state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == "Test Aura")
            .map(|(id, _)| *id)
            .expect("aura object");
        // A dangling id that genuinely names no object in `state.objects()`.
        let dangling = ObjectId(aura_id.0 + 5_000);
        state
            .objects_mut()
            .get_mut(&aura_id)
            .expect("aura object")
            .attached_to = Some(dangling);
        assert!(
            state.object(dangling).is_err(),
            "the fixture's whole premise is that this id is dangling"
        );

        let mut v = Vec::new();
        check_attachment_validity(&state, &mut v);
        assert_eq!(v.len(), 1, "{v:?}");
        let evidence = &v[0].evidence;
        assert!(
            evidence
                .iter()
                .any(|e| e.starts_with("attacher_card_types=") && e.contains("Enchantment")),
            "{evidence:?}"
        );
        assert!(
            evidence
                .iter()
                .any(|e| e.starts_with("attacher_subtypes=") && e.contains("Aura")),
            "{evidence:?}"
        );
        assert!(
            evidence.contains(&"target_present_in_state_objects=false".to_string()),
            "{evidence:?}"
        );
        assert!(
            evidence
                .iter()
                .any(|e| e.starts_with(&format!("target={dangling:?}"))),
            "{evidence:?}"
        );
    }

    /// PB-DX56 / OOS-FB1-1: `check_all` prepends the common state snapshot to
    /// EVERY violation, in front of the check's own evidence -- proven through the
    /// front door (`check_all`, not `check_player_consistency` directly), so a
    /// future change that stops calling `state_context` from `check_all` cannot
    /// pass this test by accident.
    #[test]
    fn t_check_all_prepends_state_context_before_the_checks_own_evidence() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .build()
            .expect("builder state");
        state.turn_mut().priority_holder = Some(p(2));
        state
            .players_mut()
            .get_mut(&p(2))
            .expect("p2 exists")
            .has_conceded = true;

        let violations = check_all(&state, None);
        let v = violations
            .iter()
            .find(|v| v.check == HARD_DEPARTED_PRIORITY_HOLDER)
            .expect("the priority-holder violation must be present");
        assert!(
            v.evidence.iter().any(|e| e.starts_with("turn=")),
            "{:?}",
            v.evidence
        );
        assert!(
            v.evidence.iter().any(|e| e.starts_with("phase=")),
            "{:?}",
            v.evidence
        );
        assert!(
            v.evidence.contains(&"arm=priority_holder".to_string()),
            "the check's OWN evidence must still be present, appended after the \
             common context: {:?}",
            v.evidence
        );
        // General-first, specific-second: the common context's `turn=` line comes
        // before the check-specific `arm=` line.
        let turn_idx = v
            .evidence
            .iter()
            .position(|e| e.starts_with("turn="))
            .unwrap();
        let arm_idx = v
            .evidence
            .iter()
            .position(|e| e == "arm=priority_holder")
            .unwrap();
        assert!(turn_idx < arm_idx, "{:?}", v.evidence);
    }

    // ── PB-DX56 (`OOS-DX32-1` / `OOS-DX22-8` / `OOS-FB1-1`) ────────────────────────

    /// **The evidence key that the state context shadows.**
    ///
    /// `check_all` PREPENDS `state_context`, which emits one `player=PlayerId(n) …` line
    /// per seat. A consumer keying the departed-active arm on `player=` therefore reads the
    /// FIRST state-context line — a value identical for every violation in the game — and
    /// collapses two different departed seats into one CR 800.4k window.
    ///
    /// This is not a hypothetical: `LocalGame::promote_if_it_crossed_a_turn`'s first draft
    /// did exactly that and manufactured a false promotion on fuzz seed 5, keying
    /// `PlayerId(4)`'s turn-154 report against `PlayerId(1)`'s turn-133 one. RED before the
    /// `arm_player=` rename (executed).
    #[test]
    fn t_arm_player_key_is_not_shadowed_by_state_context() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .build()
            .expect("two-player state builds");
        // Seat 2 is the departed ACTIVE player; seat 1 is alive and comes FIRST in the
        // state-context lines, which is precisely what shadowed the lookup.
        state.players_mut().get_mut(&p(2)).expect("seat 2").has_lost = true;
        state.turn_mut().active_player = p(2);

        let vs = check_all(&state, None);
        let departed: Vec<_> = vs
            .iter()
            .filter(|v| v.check == TRANSIENT_DEPARTED_ACTIVE_PLAYER)
            .collect();
        assert_eq!(
            departed.len(),
            1,
            "exactly one departed-active report: {vs:?}"
        );

        let ev = &departed[0].evidence;
        let by_arm_key: Vec<_> = ev
            .iter()
            .filter_map(|e| e.strip_prefix("arm_player="))
            .collect();
        assert_eq!(
            by_arm_key,
            vec!["PlayerId(2)"],
            "`arm_player=` must name the arm's OWN subject, exactly once: {ev:?}"
        );

        // And the shadowing itself is pinned, so the reason for the odd key survives:
        // a `player=` lookup finds a state-context line for a DIFFERENT seat first.
        let first_player_prefixed = ev
            .iter()
            .find_map(|e| e.strip_prefix("player="))
            .expect("state_context emits per-seat `player=` lines");
        assert!(
            first_player_prefixed.starts_with("PlayerId(1)"),
            "the first `player=`-prefixed evidence line must be a state-context line for a \
             DIFFERENT seat -- that is what makes `player=` the wrong key: {ev:?}"
        );
    }

    /// **CR 800.4a vs CR 800.4j: the two arms carry different `check` names, and the split
    /// is the finding.** A departed ACTIVE player is the CR 800.4j class (transient); a
    /// departed PRIORITY HOLDER is the CR 800.4a class (hard). Both directions asserted, so
    /// a future edit that collapses them back reddens here.
    #[test]
    fn t_player_consistency_arms_are_separate_classes() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .add_player(p(3))
            .build()
            .expect("three-player state builds");
        state.players_mut().get_mut(&p(1)).expect("seat 1").has_lost = true;
        state
            .players_mut()
            .get_mut(&p(2))
            .expect("seat 2")
            .has_conceded = true;
        state.turn_mut().active_player = p(1);
        state.turn_mut().priority_holder = Some(p(2));

        let vs = check_all(&state, None);
        let names: Vec<&str> = vs
            .iter()
            .filter(|v| {
                v.check == TRANSIENT_DEPARTED_ACTIVE_PLAYER
                    || v.check == HARD_DEPARTED_PRIORITY_HOLDER
            })
            .map(|v| v.check.as_str())
            .collect();
        assert!(
            names.contains(&TRANSIENT_DEPARTED_ACTIVE_PLAYER),
            "the CR 800.4j arm must report under its own name: {vs:?}"
        );
        assert!(
            names.contains(&HARD_DEPARTED_PRIORITY_HOLDER),
            "the CR 800.4a arm must report under its own name: {vs:?}"
        );
        assert!(
            is_transient_check(TRANSIENT_DEPARTED_ACTIVE_PLAYER),
            "CR 800.4j: the active-player arm is the transient half"
        );
        assert!(
            !is_transient_check(HARD_DEPARTED_PRIORITY_HOLDER),
            "CR 800.4a's last sentence is unconditional -- the priority arm is NOT transient"
        );
        // The concession/loss distinction the old shared description folded away.
        let prio = vs
            .iter()
            .find(|v| v.check == HARD_DEPARTED_PRIORITY_HOLDER)
            .expect("priority arm reported");
        assert!(
            prio.evidence.iter().any(|e| e == "has_conceded=true")
                && prio.evidence.iter().any(|e| e == "has_lost=false"),
            "CR 104.3a concession and CR 704.5a loss are different situations and must be \
             reported separately: {:?}",
            prio.evidence
        );
    }

    /// **The direction of the attachment relation that never heals** — `OOS-DX22-8`'s
    /// at-rest half, planted. A host listing a dead `ObjectId` is a hard violation, and so
    /// is a host listing a live attacher that points somewhere else. Paired with a healthy
    /// state that must be silent.
    #[test]
    fn t_attachment_symmetry_catches_both_asymmetries() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .object(ObjectSpec::creature(p(1), "Bearer", 2, 2))
            .object(ObjectSpec::artifact(p(1), "Jitte"))
            .object(ObjectSpec::creature(p(1), "Other", 1, 1))
            .build()
            .expect("attachment fixture builds");
        let find = |state: &GameState, name: &str| -> ObjectId {
            state
                .objects_in_zone(&ZoneId::Battlefield)
                .into_iter()
                .find(|o| o.characteristics.name == name)
                .expect("fixture object present")
                .id
        };
        let bearer = find(&state, "Bearer");
        let jitte = find(&state, "Jitte");
        let other = find(&state, "Other");
        assert_eq!(
            state.objects_in_zone(&ZoneId::Battlefield).len(),
            3,
            "three battlefield objects in the fixture"
        );

        // Healthy: a symmetric attachment must be silent.
        state
            .objects_mut()
            .get_mut(&jitte)
            .expect("jitte")
            .attached_to = Some(bearer);
        state
            .objects_mut()
            .get_mut(&bearer)
            .expect("bearer")
            .attachments
            .push_back(jitte);
        let mut vs = Vec::new();
        check_attachment_symmetry(&state, &mut vs);
        assert!(
            vs.is_empty(),
            "a symmetric attachment must be silent: {vs:?}"
        );

        // Asymmetry 1: the host lists an attacher that points somewhere ELSE.
        //
        // **A correction to this comment's first draft, which was a false claim about
        // coverage inside a batch whose own findings are false claims about coverage.** It
        // said the dead-id direction was "covered end-to-end by
        // `crates/engine/tests/primitives/pb_dx56_departure_hygiene.rs`". It is not: that
        // file is in another crate, `check_attachment_symmetry` is private to this one, and
        // it never calls it -- that file covers the ENGINE fix (F1), which is a different
        // proposition from "this check catches the condition". An executed plant making the
        // `Err(_)` arm unreachable left the whole workspace GREEN. The dead-id direction is
        // now driven by `t_attachment_symmetry_catches_a_dead_object_id`.
        state
            .objects_mut()
            .get_mut(&jitte)
            .expect("jitte")
            .attached_to = Some(other);
        let mut vs = Vec::new();
        check_attachment_symmetry(&state, &mut vs);
        assert_eq!(
            vs.len(),
            1,
            "a host listing an attacher whose attached_to points elsewhere is one \
             violation: {vs:?}"
        );
        assert_eq!(vs[0].check, HARD_ATTACHMENT_SYMMETRY);
        assert!(
            !is_transient_check(HARD_ATTACHMENT_SYMMETRY),
            "no SBA is supposed to clean this up, so there is no CR 704.3 window to \
             excuse a report -- it must NOT be transient"
        );
    }

    /// **The end-state answer to the `attachment_validity` transient split.** A dangling
    /// `attached_to` at a checkpoint is the CR 704.3 / `OOS-M11-7` window; one still there
    /// when the game is OVER is a permanent CR 704.5m/704.5n blind spot. Both directions.
    #[test]
    fn t_dangling_attachment_at_rest_is_a_hard_violation() {
        let clean = GameStateBuilder::new()
            .add_player(p(1))
            .object(ObjectSpec::creature(p(1), "Bearer", 2, 2))
            .build()
            .expect("clean fixture builds");
        assert!(
            check_no_dangling_attachment_at_rest(&clean).is_empty(),
            "a state with no attachment at all must be silent"
        );

        let mut state = clean.clone();
        let bearer = state
            .objects_in_zone(&ZoneId::Battlefield)
            .into_iter()
            .next()
            .expect("bearer")
            .id;
        // `ObjectId` values are minted monotonically, so an id far above every live one is
        // guaranteed absent from `state.objects` -- which is exactly the CR 400.7 condition
        // this check is about.
        state
            .objects_mut()
            .get_mut(&bearer)
            .expect("bearer")
            .attached_to = Some(ObjectId(999_999));
        let vs = check_no_dangling_attachment_at_rest(&state);
        assert_eq!(vs.len(), 1, "exactly one dangling attachment: {vs:?}");
        assert_eq!(vs[0].check, HARD_DANGLING_ATTACHMENT_AT_REST);
        assert!(
            !is_transient_check(HARD_DANGLING_ATTACHMENT_AT_REST),
            "the END-STATE property must be hard, or the split it answers is a whitewash"
        );
        assert!(
            vs[0]
                .evidence
                .iter()
                .any(|e| e.starts_with("attacher_card_types=")),
            "the evidence must name the attacher's card types -- that is the single fact \
             that decides between CR 704.5m and CR 704.5n: {:?}",
            vs[0].evidence
        );
    }

    // ── PB-DX56 `/review` fix cycle: every one of these closes a bypass that was
    // EXECUTED and came back GREEN. See `memory/primitives/pb-DX56-bypass-attempts.md`.

    /// **Bypass A1, closed.** Deleting `check_attachment_symmetry(..)` from [`check_all`]
    /// left all 74 simulator tests and all 6 engine probes GREEN, because both of that
    /// check's probes call the private function DIRECTLY. `check_stack_consistency` has had
    /// exactly this gate since SIM-3 (`t10_check_all_dispatches_to_this_check`) and the new
    /// check did not get one. Driven through the FRONT DOOR, both directions.
    #[test]
    fn t_check_all_dispatches_to_attachment_symmetry() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .object(ObjectSpec::creature(p(1), "Bearer", 2, 2))
            .object(ObjectSpec::artifact(p(1), "Jitte"))
            .build()
            .expect("fixture builds");
        let find = |st: &GameState, n: &str| -> ObjectId {
            st.objects_in_zone(&ZoneId::Battlefield)
                .into_iter()
                .find(|o| o.characteristics.name == n)
                .expect("present")
                .id
        };
        let bearer = find(&state, "Bearer");
        let jitte = find(&state, "Jitte");
        let named = |st: &GameState| {
            check_all(st, None)
                .into_iter()
                .filter(|v| v.check == HARD_ATTACHMENT_SYMMETRY)
                .count()
        };

        // Symmetric: silent through `check_all`.
        state
            .objects_mut()
            .get_mut(&jitte)
            .expect("jitte")
            .attached_to = Some(bearer);
        state
            .objects_mut()
            .get_mut(&bearer)
            .expect("bearer")
            .attachments
            .push_back(jitte);
        assert_eq!(
            named(&state),
            0,
            "a symmetric attachment must be silent via check_all"
        );

        // Asymmetric: reported through `check_all`, which is what A1 removed.
        state
            .objects_mut()
            .get_mut(&jitte)
            .expect("jitte")
            .attached_to = None;
        assert_eq!(
            named(&state),
            1,
            "check_all must DISPATCH to check_attachment_symmetry -- deleting that one \
             line reddened nothing before this test existed"
        );
    }

    /// **Bypass A2, closed.** The `Err(_)` arm — a host listing an `ObjectId` that does not
    /// resolve — is the whole of `OOS-DX22-8`'s direction B, and **no test drove it**: both
    /// branches of `t_attachment_symmetry_catches_both_asymmetries` exercise `Ok(att)`.
    /// Making the arm unreachable left everything GREEN.
    ///
    /// That test's own comment excused the gap by claiming the dead-id direction was
    /// *"covered end-to-end by `pb_dx56_departure_hygiene.rs`"*. **That claim was FALSE
    /// about this check**: that file is in another crate, this function is private, and it
    /// never calls it — it covers the ENGINE fix (F1), not this check's arm. The comment is
    /// corrected at the site; this is the coverage.
    #[test]
    fn t_attachment_symmetry_catches_a_dead_object_id() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .object(ObjectSpec::creature(p(1), "Bearer", 2, 2))
            .build()
            .expect("fixture builds");
        let bearer = state
            .objects_in_zone(&ZoneId::Battlefield)
            .into_iter()
            .next()
            .expect("bearer")
            .id;
        // `ObjectId`s are minted monotonically, so an id far above every live one cannot be
        // a key of `state.objects` — which is exactly the CR 400.7 condition after a zone
        // change retires an attacher's id.
        state
            .objects_mut()
            .get_mut(&bearer)
            .expect("bearer")
            .attachments
            .push_back(ObjectId(999_999));

        let vs = check_all(&state, None);
        let sym: Vec<_> = vs
            .iter()
            .filter(|v| v.check == HARD_ATTACHMENT_SYMMETRY)
            .collect();
        assert_eq!(sym.len(), 1, "exactly one dead-attacher report: {vs:?}");
        assert!(
            sym[0]
                .evidence
                .contains(&"direction=host_lists_dead_attacher".to_string()),
            "the DEAD-ID arm specifically, not the points-elsewhere one: {:?}",
            sym[0].evidence
        );
    }

    /// **Bypass C1, closed.** Adding `HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN` as a
    /// fourth arm of [`is_transient_check`] routes the CR 800.4k promotion straight back
    /// into the transient bucket — silently voiding the entire justification for calling
    /// the CR 800.4j class transient — and left the workspace GREEN.
    ///
    /// Keyed on the NAMING CONVENTION and parsed from this file's own source, so a class
    /// constant added tomorrow is covered by construction rather than by someone
    /// remembering to extend a hand-written list. That is the repair PB-DX43's
    /// `TOKEN_SPEC_FIELDS` finding prescribes: gate the list against the declaration.
    #[test]
    fn t_every_class_constant_is_classified_by_its_own_name() {
        // Parsed over the WHOLE source with the line break tolerated, not line by line.
        // The single-line first draft found 3 hard constants instead of 4, because
        // `cargo fmt` wraps
        //     pub const HARD_DEPARTED_ACTIVE_PLAYER_CROSSED_A_TURN: &str =
        //         "departed_active_player_crossed_a_turn";
        // across two lines -- the multi-line-spelling blind spot PB-DX45's re-pin and
        // PB-DX50's sentinel census each hit once. **The non-vacuity floor below is what
        // caught it**, which is the entire argument for having one on a parsing gate.
        let src = include_str!("invariants.rs");
        let mut transient = Vec::new();
        let mut hard = Vec::new();
        for chunk in src.split("pub const ").skip(1) {
            let Some((name, tail)) = chunk.split_once(": &str =") else {
                continue;
            };
            if name.contains(char::is_whitespace) {
                continue;
            }
            let Some((value, _)) = tail.split_once(';') else {
                continue;
            };
            let value = value.trim().trim_matches('"').to_string();
            if name.starts_with("TRANSIENT_") {
                transient.push(value);
            } else if name.starts_with("HARD_") {
                hard.push(value);
            }
        }
        assert!(
            transient.len() >= 3 && hard.len() >= 4,
            "non-vacuity: the parse must find the class constants it is gating -- \
             transient {transient:?}, hard {hard:?}"
        );
        for v in &transient {
            assert!(
                is_transient_check(v),
                "`TRANSIENT_*` constant {v:?} must be classified transient"
            );
        }
        for v in &hard {
            assert!(
                !is_transient_check(v),
                "`HARD_*` constant {v:?} must NOT be classified transient -- moving a hard \
                 class into the transient set silently deletes the strictly stronger \
                 property it exists to be"
            );
        }
    }

    /// **Bypass C2, closed.** The CR 800.4k promotion had **no test of any kind** — the
    /// constant occurred in exactly two places workspace-wide, its declaration and its
    /// assignment — so a plant that made it never fire left everything GREEN. The decision
    /// is now a pure function and both directions are asserted, including the boundary.
    #[test]
    fn t_crosses_a_turn_boundary_is_strict_and_class_scoped() {
        assert!(
            !crosses_a_turn_boundary(TRANSIENT_DEPARTED_ACTIVE_PLAYER, 10, 10),
            "CR 800.4j: the SAME turn is the bounded window, not a crossing"
        );
        assert!(
            !crosses_a_turn_boundary(TRANSIENT_DEPARTED_ACTIVE_PLAYER, 9, 10),
            "a report at an EARLIER turn is not a crossing either"
        );
        assert!(
            crosses_a_turn_boundary(TRANSIENT_DEPARTED_ACTIVE_PLAYER, 11, 10),
            "CR 800.4k: a departed player's turn does not begin, so the condition may not \
             survive into a later turn"
        );
        // Class-scoped: the other transient classes have their own strictly stronger
        // properties and must not be promoted by this one.
        assert!(
            !crosses_a_turn_boundary(TRANSIENT_ORPHANED_TOKENS, 11, 10),
            "the token class is answered by check_no_leaked_tokens, not by a turn boundary"
        );
        assert!(
            !crosses_a_turn_boundary(TRANSIENT_ATTACHMENT_VALIDITY, 11, 10),
            "the attachment class is answered by check_no_dangling_attachment_at_rest"
        );
    }

    /// **`OOS-DX56-1`'s seat key, at the consumer.** `arm_player_of` must read the ARM's
    /// subject and not the first `player=` line `state_context` prepends.
    #[test]
    fn t_arm_player_of_reads_the_arm_not_the_state_context() {
        let mut state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .build()
            .expect("state builds");
        state.players_mut().get_mut(&p(2)).expect("seat 2").has_lost = true;
        state.turn_mut().active_player = p(2);
        let v = check_all(&state, None)
            .into_iter()
            .find(|v| v.check == TRANSIENT_DEPARTED_ACTIVE_PLAYER)
            .expect("departed-active report present");
        assert_eq!(
            arm_player_of(&v),
            Some("PlayerId(2)"),
            "must name the departed ACTIVE player, not the first seat in the state \
             context: {:?}",
            v.evidence
        );
    }

    /// **Bypass B1, closed — and the hole was INHERITED, not introduced.** Deleting
    /// `check_no_dangling_attachment_at_rest`'s call from `LocalGame::result_snapshot`
    /// left the whole workspace GREEN. So does deleting `check_no_leaked_tokens`'s, which
    /// has sat at the same call site since PB-DX32 Stage 4.
    ///
    /// These are END-STATE checks: they are the strictly stronger properties that keep the
    /// transient splits honest, they run once per game at the one site both real terminal
    /// paths go through, and **nothing asserted they were still invoked**. The per-command
    /// behavioural probes cannot notice, because they never reach `result_snapshot` — which
    /// is `OOS-DX52-2`'s shape one axis worse: not "a row that reddens only a source gate",
    /// but a property with no gate at all.
    ///
    /// A source gate rather than a behavioural one, deliberately: reaching `result_snapshot`
    /// needs a full driven game, and what is being asserted is a WIRING fact, not a
    /// behaviour. Keyed on the `check_no_` prefix so a THIRD end-state check added tomorrow
    /// is covered by construction. `OOS-DX56-5`.
    #[test]
    fn t_every_end_state_check_is_called_from_result_snapshot() {
        let invariants_src = include_str!("invariants.rs");
        let local_game_src = include_str!("local_game.rs");

        let mut end_state_checks: Vec<String> = Vec::new();
        for chunk in invariants_src.split("pub fn check_no_").skip(1) {
            let Some((name, _)) = chunk.split_once('(') else {
                continue;
            };
            // The name must be a real Rust identifier. Without this the parse matches its
            // OWN source -- `include_str!` pulls in this test module, whose literal
            // `"pub fn check_no_"` yields the "name" `").skip`. A self-referential source
            // gate that scans the file it lives in has to exclude itself, and the cheapest
            // honest way is to insist the capture is spellable as an identifier.
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }
            end_state_checks.push(format!("check_no_{name}"));
        }
        assert!(
            end_state_checks.len() >= 2,
            "non-vacuity: the parse must find the end-state checks it is gating, got \
             {end_state_checks:?}"
        );

        // `result_snapshot`'s body, brace-matched — a byte window would fail OPEN by
        // over-scanning into the next function and vouching for a call that is not there
        // (`OOS-DX49-2`).
        let at = local_game_src
            .find("pub fn result_snapshot(")
            .expect("result_snapshot exists");
        let open = at + local_game_src[at..].find('{').expect("body opens");
        let mut depth = 0usize;
        let bytes = local_game_src.as_bytes();
        let mut end = None;
        for (i, b) in bytes.iter().enumerate().skip(open) {
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let body = &local_game_src[open..=end.expect("result_snapshot's body is balanced")];
        assert!(
            body.len() > 200,
            "non-vacuity: the brace match returned a body too small to be real ({} bytes)",
            body.len()
        );

        for name in &end_state_checks {
            assert!(
                body.contains(name.as_str()),
                "`invariants::{name}` is an END-STATE check and must be called from \
                 `LocalGame::result_snapshot` -- the ONE site both real terminal paths go \
                 through. Deleting such a call reddens no behavioural probe, because none \
                 of them reaches that function (OOS-DX56-5). Body was: {body}"
            );
        }
    }
}
