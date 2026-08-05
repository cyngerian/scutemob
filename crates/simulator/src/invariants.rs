//! Invariant checks run after every state transition during fuzzing.
//!
//! **Ten checks exist; nine of them fire from [`check_all`]**, plus one deliberate
//! no-op: zone integrity, ID uniqueness, stack consistency, player consistency, turn
//! order, object-zone agreement, attachment validity, game progression, orphaned
//! tokens — and `check_mana_non_negative`, which cannot fail because `ManaPool` is
//! `u32`. The tenth, [`check_no_leaked_tokens`] (PB-DX32 Stage 4), is an END-OF-GAME
//! check and is deliberately NOT in [`check_all`] — it runs once per game, at both
//! real `LocalGame` terminal paths, not per command.
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

/// An invariant violation found during fuzzing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantViolation {
    pub check: String,
    pub description: String,
    pub turn_number: u32,
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
    if let Some(prev) = prev_turn {
        check_game_progression(state, prev, &mut violations);
    }
    check_no_orphaned_tokens(state, &mut violations);

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
        });
    }
}

/// 5. Player consistency: active player and priority holder are alive
fn check_player_consistency(state: &GameState, violations: &mut Vec<InvariantViolation>) {
    let active = state.turn().active_player;
    if let Ok(p) = state.player(active) {
        if p.has_lost || p.has_conceded {
            violations.push(InvariantViolation {
                check: "player_consistency".into(),
                description: format!("Active player {:?} has lost or conceded", active),
                turn_number: state.turn().turn_number,
            });
        }
    }

    if let Some(priority) = state.turn().priority_holder {
        if let Ok(p) = state.player(priority) {
            if p.has_lost || p.has_conceded {
                violations.push(InvariantViolation {
                    check: "player_consistency".into(),
                    description: format!("Priority holder {:?} has lost or conceded", priority),
                    turn_number: state.turn().turn_number,
                });
            }
        }
    }
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
                    check: "attachment_validity".into(),
                    description: format!(
                        "Object {:?} attached to {:?} which doesn't exist",
                        obj.id, target_id
                    ),
                    turn_number: state.turn().turn_number,
                });
            }
        }
    }
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
                check: "no_orphaned_tokens".into(),
                description: format!(
                    "Token {:?} '{}' found in zone {:?}",
                    obj_id, obj.characteristics.name, obj.zone
                ),
                turn_number: state.turn().turn_number,
            });
        }
    }
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
    use mtg_engine::{GameStateBuilder, ObjectSpec, PlayerId, StackObject, StackObjectKind};

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
}
