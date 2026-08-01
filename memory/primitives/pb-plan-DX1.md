# Primitive Batch Plan: PB-DX1 — the intervening-if dropped in the runtime lowering

**Generated**: 2026-07-31
**Task**: `scutemob-160` · branch `feat/pb-dx1-the-intervening-if-dropped-in-the-runtime-lowering-oo`
**Seed**: **OOS-DP6-1** (rank 1 of PB-DX1..DX18, `memory/primitives/seed-rerank-2026-07-27.md` §4)
**Riders**: **OOS-DP6-5** (TurnFaceUp resolution re-check), **OOS-DP6-9** (haunt, both ends)
**Class**: **CORRECTNESS** — live-wrong on a `Complete`, deck-legal def; unbounded loop
**CR**: **603.4** (primary), 603.2, 603.2c/h, 603.3, 603.10a, 508.3a, 700.4, 708.8, 702.55c, 113.7a
**Baseline**: tests **3,928 / 0**; PROTOCOL **31** / HASH **68**; branch head `3d73763d`
**Predicted wire**: **PROTOCOL 31 → 32 AND HASH 68 → 69** — see §7. *(This corrects the audit
row and the dispatch brief, which both predicted HASH only. The correction is falsifiable and
the gates arbitrate; do not hand-bump either.)*
**Predicted card yield**: **1 marker flip** (`karlach_fury_of_avernus`, `known_wrong` → `Complete`,
oracle-gated), **4 behaviour repairs on `Complete` defs** (aurelia + 3 from the `once_per_turn`
rider), **0 new defs**. Discounted per `feedback_pb_yield_calibration`; see §6.

---

## 0. Reading order for the runner

1. §1 — premise (extends `memory/primitive-wip.md`, does not repeat it).
2. §2 — CR text. Read 603.4 **and** 603.10a together; the second is why §4 has a
   three-valued moment enum instead of a boolean.
3. §3 — **the (a)/(b)/(c) decision.** Load-bearing. Do not start coding before reading it.
4. §4 — the fix shape, with the 14-call-site classification table.
5. §5 — the 34 push sites.
6. §6 — roster derivation (**from `all_cards()`, never grep**) + card-def dispositions.
7. §7 — the wire/hash re-pin protocol. §8 — tests + the mandatory fail-before probe.
8. §9 — riders. §10 — the `once_per_turn` rider (a NEW finding from planning). §11 — seeds.
9. §12 — ordered step list. §13 — "done" checklist. §14 — risks.

---

## 1. Premise (extends the WIP re-verification; not a repeat)

`memory/primitive-wip.md` already confirmed the four legs of OOS-DP6-1 on `3d73763d`. Planning
added six facts the WIP file does not carry. All six are load-bearing.

**P1 — the drop is total, at both ends, and both ends are single sites.**
- Queue end: `rules/abilities.rs:6878-6882`, inside `collect_triggers_for_event`. It reads
  `trigger_def.intervening_if` — the **runtime** `Option<InterveningIf>`, hardcoded `None` by the
  lowering — and calls `check_intervening_if`.
- Resolution end: `rules/resolution.rs:2356-2391`, the `else` branch of the
  `StackObjectKind::TriggeredAbility` arm. The `if` branch (`:2136-2216`) explicitly returns
  `(None, None)` for the runtime path (`:2144-2149`, comment: *"Characteristics path —
  intervening_if handled below via original code"*), so a lowered trigger always falls to the
  `else`, where `:2370-2385` again reads the runtime field. Both see `None`.

**P2 — thirteen queue sites, not one, read the runtime field.** `check_intervening_if` has
**13** call sites in `rules/abilities.rs` (`:4559`, `:4622`, `:4872`, `:4917`, `:5014`, `:5475`,
`:5785`, `:5843`, `:5901`, `:6366`, `:6550`, `:6879`, `:6935`) and **1** in `rules/resolution.rs`
(`:2378`). Every lowered trigger event is dispatched through one of them. This is the single
strongest argument for the fix shape chosen in §3: putting the card-def condition **inside**
`InterveningIf` repairs all 14 at once, with no per-site work and no chance of missing one.

**P3 — `TriggeredAbilityDef` is in the PROTOCOL wire closure, not only the hash closure.**
`crates/engine/tests/core/protocol_schema.rs:98` lists `Characteristics` in `CLOSURE_MUST_CONTAIN`;
`Characteristics.triggered_abilities: Vec<TriggeredAbilityDef>` (`game_object.rs:964`); and the
closure walk (`protocol_schema.rs:576-614`) is a naive capitalized-identifier scan of each
declaration body, so it descends into `TriggeredAbilityDef` and from there into `InterveningIf`.
**Therefore this batch bumps PROTOCOL as well as HASH.** Corollary finding: the parenthetical
notes on `PROTOCOL_VERSION` history rows **25** and **26** (`rules/protocol.rs:229-232`, `:245-247`)
assert that `TriggerEvent`/`TriggerCondition` are "not in the wire closure". For `TriggerEvent`
that is almost certainly **false** — it is `TriggeredAbilityDef.trigger_on`. Both bumps
co-occurred with an independent digest move, so the claim was never tested. See §7.4.

**P4 — the in-repo tombstone for alternative (c) already exists.** `replay_harness.rs:2642-2646`
and `:2781-2787` both carry an *"Index-namespace fix (2026-07-09)"* comment recording that the
index-correspondence trick was tried for a *different* dropped field on this exact lowering, and
that it shipped as a bug: *"…instead of leaving the post-filter in rules/abilities.rs to re-look
the ability up via CardDefinition::abilities (a different, non-dense index space … this was the
root cause of the Monastery Mentor / Leaf-Crowned Visionary filter bypass bug)."* §3.3 uses this.

**P5 — CR 603.10a bites.** Eight of the 14 `check_intervening_if` call sites hand in a source
that is **already off the battlefield** (graveyard / exile / hand / any-zone), because they are
look-back-in-time trigger sites. `crate::effects::check_condition` evaluates against the
**current** state, so a source-scoped `Condition` (`SourceOnBattlefield`, `SourceHasCounters`,
`SourceIsUntapped`, …) would read false there and **suppress a trigger CR 603.4 says must fire**.
This is the exact failure mode PB-DP6 hard constraint 3 forbids, and it is why §4's moment
parameter is three-valued. Zero corpus exposure today (§6), so the carve-out costs 0 flips.

**P6 — a SECOND field is dropped by the same lowering, and it is live-wrong on three `Complete`
defs.** `once_per_turn` is hardcoded `false` at **31 of the 34** push sites (propagated at only
`:3070`, `:3106`, `:3158`). `flush_pending_triggers`' gate (`abilities.rs:7957-7993`) reads the
runtime value **first** and only falls back to the registry *when the runtime lookup misses* —
for a lowered trigger the lookup **hits** and returns `false`, so the fallback never runs.
`welcoming_vampire`, `elvish_warmaster` and `whispering_wizard` all print *"This ability triggers
only once each turn"* (MCP-verified, §10) and all carry no `completeness` field, i.e. they are
`Complete` and deck-legal, and all three over-fire today. Wire-neutral to fix. See §10.

**Premise verdict: holds, and is stronger than filed.**

---

## 2. CR text (MCP, verbatim)

**CR 603.4** — *"A triggered ability may read 'When/Whenever/At [trigger event], if [condition],
[effect].' When the trigger event occurs, the ability checks whether the stated condition is true.
The ability triggers only if it is; otherwise it does nothing. If the ability triggers, it checks
the stated condition again as it resolves. If the condition isn't true at that time, the ability
is removed from the stack and does nothing. Note that this mirrors the check for legal targets.
This rule is referred to as the 'intervening "if" clause' rule. (The word 'if' has only its normal
English meaning anywhere else in the text of a card; this rule only applies to an 'if' that
immediately follows a trigger condition.)"*

Two obligations, not one. A fix that gates only the queue end is a queue-then-fizzle no-op
(PB-DP6's own review found precisely this shape at `resolution.rs:2299` — the queue gate let the
trigger through and the resolution re-check killed it every time). **Both ends, or nothing.**

**CR 603.2** — *"Whenever a game event or game state matches a triggered ability's trigger event,
that ability automatically triggers. The ability doesn't do anything at this point."*

**CR 603.2c** — *"An ability triggers only once each time its trigger event occurs. However, it
can trigger repeatedly if one event contains multiple occurrences."*
**CR 603.2h** — *"A triggered ability may have an instruction followed by 'Do this only once each
turn.' This ability triggers only if its source's controller has not yet taken the indicated
action that turn."* *(The engine's `once_per_turn` flag serves the adjacent "This ability triggers
only once each turn" templating; §10.)*

**CR 508.3a** — *"An ability that reads 'Whenever [a creature] attacks, . . .' triggers if that
creature is declared as an attacker. … Such abilities won't trigger if a creature is put onto the
battlefield attacking."*

**CR 700.4** — *"The term dies means 'is put into a graveyard from the battlefield.'"*

**CR 603.10a** *(paraphrase — the look-back rule)*: leave-the-battlefield abilities are checked
against the game state as it existed immediately before the event. The engine implements the
*trigger-matching* half of this (eight sites read the graveyard/exile/hand object and thread
`pre_death_characteristics` / `lki_counters` / `lki_power`), but has **no LKI-aware
`check_condition`**. See §4.3 and seed OOS-DX1-1.

---

## 3. THE DECISION: (a), (b) or (c)

### 3.1 Recommendation

**Take (a) — the runtime type learns to carry the card-def condition — realised as a new
*variant on `InterveningIf`*, not as a second *field on `TriggeredAbilityDef`*.**

```rust
// crates/card-types/src/state/game_object.rs, in `enum InterveningIf` (:817)
    /// PB-DX1 (CR 603.4): a **card-definition** intervening-if
    /// (`AbilityDefinition::Triggered.intervening_if: Option<Condition>`) carried
    /// through `build_face_ability_vectors`' lowering. Before PB-DX1 the lowering
    /// hardcoded `intervening_if: None` at all 34 push sites because `Condition` and
    /// `InterveningIf` were unrelated types, so the condition was checked at NEITHER
    /// end of CR 603.4 (OOS-DP6-1: Aurelia, the Warleader granted herself unbounded
    /// extra combats on a `Complete`, deck-legal def).
    /// Boxed per `clippy::large_enum_variant` — `Condition` is far larger than the
    /// two legacy variants (mirrors `TriggerEvent::PermanentBecomesTarget.scope`).
    CardDef(Box<crate::cards::card_definition::Condition>),
```

Call this **(a′)**. It is (a) in every respect the brief cares about — the runtime type gains the
ability to carry an `Option<Condition>`, and it forces a wire bump — but it is strictly cheaper
and strictly safer than the literal "add a field" form. Both sub-forms are argued in 3.2.

### 3.2 (a) as a field vs (a′) as a variant

| | (a) `carddef_intervening_if: Option<Condition>` field on `TriggeredAbilityDef` | **(a′) `InterveningIf::CardDef(Box<Condition>)` variant** |
|---|---|---|
| Construction-site churn | **~140 struct literals** must add `carddef_intervening_if: None` — 61 in `crates/engine/src`, ~130 in `crates/engine/tests`, 2 in `crates/card-defs`, 1 in `crates/simulator/tests` (`rg -c 'TriggeredAbilityDef \{'` → 238 hits / 84 files, incl. docs) | **zero** — every existing `intervening_if: None` stays valid |
| Read-site safety | 4 existing readers of `.intervening_if` (`abilities.rs:6549/6878/6934`, `resolution.rs:2370`) must each be *remembered*; nothing forces it. A future reader of one field silently ignores the other — a permanent structural footgun | **compile-forced**: `check_intervening_if`'s match becomes non-exhaustive until the new variant is classified (the SR-5 idiom this codebase already prizes) |
| Sites repaired | must be wired per dispatch path | **all 13 queue sites + the 1 resolution site at once** (P2) |
| Wire cost | PROTOCOL + HASH | **identical** — PROTOCOL + HASH |
| Signature churn | none | `check_intervening_if` gains `source: ObjectId` + `moment` → 14 call sites |
| Net diff | ~140 mechanical edits + 5 logic edits | ~14 signature edits + 34 lowering lines + 3 logic edits |

(a′) trades 140 mechanical edits a Sonnet runner can fumble for 14 typed edits the compiler
verifies, and converts the "did you remember to read the other field?" hazard into a compile
error. It is also *conceptually* the honest move: `InterveningIf` and `Condition` are two
representations of the same CR 603.4 clause, and this unifies them behind one evaluator.

**If the runner finds a blocker in (a′)** — e.g. an orphan reader of `TriggeredAbilityDef.
intervening_if` that must distinguish the two legacy variants from a card-def condition and
cannot — **stop and report**; do not silently fall back to (a). There are exactly 4 readers and
all 4 route through `check_intervening_if`; a fifth appearing is news.

### 3.3 Why NOT (b)

(b) is "re-route these conditions onto a `CardDefETB`-style dispatch that re-reads the registry",
which is what the adjacent `WhenExertedAsAttacks` block does (`abilities.rs:4081-4141`, queue;
`resolution.rs:2182-2212`, the `is_carddef_etb` branch). It is wire-neutral. It is also wrong here,
for four independent reasons — each of which alone is disqualifying:

1. **It discards layer resolution.** `collect_triggers_for_event` reads
   `layers::expect_characteristics(state, obj_id)` (`abilities.rs:6609`) *specifically* so CR
   613.1f ability-removal (Humility, Dress Down) suppresses triggers. `def.abilities` is the
   *printed* card. Rerouting 34 trigger events onto the registry would make Humility stop
   suppressing all of them — a CR 613.1f regression far larger than the bug being fixed.
2. **It discards every runtime filter the lowering exists to carry.** `etb_filter`,
   `death_filter`, `combat_damage_filter`, `triggering_creature_filter`, `counter_filter`,
   `counter_on_self` and the CR 708.3 face-down suppression are all evaluated against the
   *runtime* `TriggeredAbilityDef` inside `collect_triggers_for_event` (`:6614-6875`). A registry
   dispatch has none of them, and re-deriving each is the multi-session refactor PB-DP6 §1 named.
3. **It breaks tokens and copies.** A token or a Clone-copied permanent has characteristics
   produced by the token spec / Layer 1, not by a `card_id` lookup. `def.abilities` is the wrong
   source of truth for them; `calculate_characteristics` is the right one.
4. **The model itself carries a live index-space asymmetry.** The exemplar queue site indexes
   `def.abilities` (`abilities.rs:4102`) while its resolution counterpart indexes
   `def.effective_abilities(obj.is_transformed)` (`resolution.rs:2192`). For a DFC with the
   ability on the back face these disagree. Copying that shape 34 times is copying a latent bug
   34 times. *(Filed as OOS-DX1-4; not fixed here.)*

### 3.4 Why NOT (c) — with the OOS-DP6-2 analysis the brief demands

(c) is the index-correspondence trick: `build_face_ability_vectors` appends one runtime entry per
matching card-def ability in `def.abilities` order, per condition, so the *k*-th runtime
`SelfAttacks` entry is the *k*-th `WhenAttacks` card-def ability, and the queue site could
re-derive the condition by counting. **Rejected.** The record:

- **OOS-DP6-2 is a live instance of exactly this assumption being wrong.**
  `rules/abilities.rs:6252`'s `WheneverYouSacrifice` `retain` post-filter looks its ability up with
  `def.effective_abilities(is_transformed).get(t.ability_index)` — but `t.ability_index` was
  written by `collect_triggers_for_event`, whose index space is
  `characteristics.triggered_abilities` (the runtime vec). Those two lists are populated by
  different rules and are not in bijection: the runtime vec omits every `AbilityDefinition::
  Keyword`, `::Activated`, `::Static`, `::Spell` and `::SagaChapter`, and omits every `Triggered`
  whose condition is not one of the 34 lowered ones; the card-def list contains all of them.
  PB-DP6 declined to gate that site *for this reason* and recorded it as its one Category-C site.
- **The same trick already shipped as a bug in this very function.** P4: `replay_harness.rs:2642`
  and `:2781` record that `spell_type_filter` / `noncreature_only` / `spell_subtype_filter` were
  once left to a post-filter that re-derived the card-def ability by index, and that this was
  *"the root cause of the Monastery Mentor / Leaf-Crowned Visionary filter bypass bug."* The fix
  applied then was **carry the datum on the runtime `TriggeredAbilityDef` itself** — i.e. (a).
  (a′) is the *same fix, again, for the next dropped field*, which is the strongest possible
  precedent argument.
- **Face changes re-derive nothing.** `apply_face_change` (`rules/face.rs:104`) calls
  `build_face_ability_vectors` on the *other* face's ability list. Any correspondence computed
  against the front face's `def.abilities` is invalid the moment the permanent transforms, and
  nothing in the codebase recomputes it. A correspondence that must be mirrored at resolution
  *and* re-derived on every face change is not a shortcut; it is three coupled invariants with no
  enforcement.
- **It would still not reach resolution.** `resolution.rs:2370` has only `ability_index` and the
  runtime vec. Re-deriving there means re-implementing the same counting a second time, against a
  possibly-transformed object. Two hand-maintained mirrors of an unenforced invariant.

**Verdict: (c) is rejected, and the rejection is recorded in this plan so the next reader does
not re-derive it.**

### 3.5 The falsifiable wire prediction that follows

> Taking (a′), `enum InterveningIf` gains a variant. `InterveningIf` is reachable from
> `Characteristics` (a `CLOSURE_MUST_CONTAIN` entry) via
> `triggered_abilities: Vec<TriggeredAbilityDef>` → `intervening_if: Option<InterveningIf>`.
> **Therefore `tests/core/protocol_schema.rs` MUST redden and `PROTOCOL_VERSION` MUST go 31 → 32,
> and `tests/core/hash_schema.rs` MUST redden on both `decl_fingerprint` and `stream_fingerprint`
> and `HASH_SCHEMA_VERSION` MUST go 68 → 69.**
>
> **Falsifier**: if `protocol_schema.rs` stays green after the variant lands, then
> `TriggeredAbilityDef` is *not* in the wire closure, the audit row's HASH-only prediction was
> right, and my P3 reasoning is wrong. **Stop, report it, and correct `docs/audits/
> decision-point-audit.md` §8.1's OOS-DP6-1 row** — do not paper over it.
>
> Both fingerprints are **gate-computed**. Never hand-write one. Read the new value out of the
> failure text.

---

## 4. Fix shape

### 4.1 The three edits that do the work

**Edit 1 — the variant** (`crates/card-types/src/state/game_object.rs`, in `enum InterveningIf`
at `:817-827`): add `CardDef(Box<Condition>)` exactly as written in §3.1. Update the enum's own
doc comment (`:812-815`) — it currently says the condition "is checked at trigger time … and again
at resolution", which was aspirationally true and is about to become actually true; note the
PB-DX1 addition per `memory/conventions.md`'s aspirationally-wrong-comment rule.

**Edit 2 — the evaluator** (`crates/engine/src/rules/abilities.rs`, `check_intervening_if` at
`:10171`):

```rust
/// When a CR 603.4 intervening-if is being evaluated. Not serialized, not hashed,
/// not on the wire — a pure call-site classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterveningIfMoment {
    /// CR 603.4 sentence 1, source still in the zone its ability functions in.
    TriggerTime,
    /// CR 603.4 sentence 1 for a **leave-the-battlefield** trigger (CR 603.10a).
    /// The source has already moved; the game must "look back in time" and the
    /// engine has no LKI-aware `check_condition`. A card-def condition is treated
    /// as HOLDING here (hard constraint 3: never suppress on a state we cannot
    /// query faithfully). Seeded as OOS-DX1-1.
    TriggerTimeLookBack,
    /// CR 603.4 sentence 2 — re-check as the ability resolves.
    Resolution,
}

pub fn check_intervening_if(
    state: &GameState,
    cond: &InterveningIf,
    controller: PlayerId,
    source: ObjectId,
    pre_death_counters: Option<&imbl::OrdMap<CounterType, u32>>,
    moment: InterveningIfMoment,
) -> bool {
    match cond {
        // UNCHANGED — byte-for-byte. Pinned by T7.
        InterveningIf::ControllerLifeAtLeast(n) => { /* as today */ }
        InterveningIf::SourceHadNoCounterOfType(ct) => { /* as today */ }
        // PB-DX1 (CR 603.4, OOS-DP6-1): the card-def condition, carried through the
        // lowering by `build_face_ability_vectors`.
        InterveningIf::CardDef(c) => match moment {
            InterveningIfMoment::TriggerTimeLookBack => true, // CR 603.10a — see above
            InterveningIfMoment::TriggerTime =>
                carddef_intervening_if_holds_at_queue_time(state, Some(c), controller, source),
            InterveningIfMoment::Resolution => {
                // CR 603.4 sentence 2. The SAME evaluability guard as the queue end:
                // of `condition_is_queue_time_evaluable`'s seven `false` variants, six
                // (WasOverloaded/WasBargained/WasCleaved/EvidenceWasCollected/
                // GiftWasGiven/SacrificeFired) are ALSO unpropagated into a trigger's
                // resolution context (OOS-DP6-6), so gating on them here would be the
                // same false negative one step later. The seventh, TargetIsLegal, IS
                // answerable at resolution and is therefore over-conservative here —
                // deliberately, because CR 608.2b's all-targets-illegal fizzle at
                // `resolution.rs:2274` already removes exactly that ability, so nothing
                // is lost. Split seeded as OOS-DX1-2.
                if !crate::effects::condition_is_queue_time_evaluable(c) {
                    return true;
                }
                let (kicker_times_paid, x_value) = state
                    .fizzle_object(source)
                    .map(|o| (o.kicker_times_paid, o.x_value))
                    .unwrap_or((0, 0));
                let mut ctx = crate::effects::EffectContext::new_with_kicker(
                    controller, source, vec![], kicker_times_paid,
                );
                ctx.x_value = x_value;
                crate::effects::check_condition(state, c, &ctx)
            }
        },
    }
}
```

Notes for the runner:
- `carddef_intervening_if_holds_at_queue_time` (`abilities.rs:10138`) is **reused verbatim** —
  it already applies the evaluability guard, already uses `fizzle_object` (SR-25 safe) and
  already builds the kicker/x context. Do not duplicate it.
- 6 parameters is under clippy's `too_many_arguments` threshold (7). Do not restructure.
- `pre_death_counters` is unused by the new arm; that is correct, and the arm should say so.

**Edit 3 — the lowering** (`crates/engine/src/testing/replay_harness.rs`,
`build_face_ability_vectors`, `:2382-3660`): at each of the **34** push sites, add
`intervening_if` to the destructure (it is currently swallowed by `..`) and replace
`intervening_if: None` with

```rust
intervening_if: intervening_if
    .clone()
    .map(|c| InterveningIf::CardDef(Box::new(c))),
```

Delete the now-false confession comment at `:2563-2564` (*"intervening_if is None here: Condition
and InterveningIf are separate types; conditional combat-damage triggers are rare and deferred"*)
and replace it with a one-line PB-DX1 note. Full site table in §5.

### 4.2 The 14 call sites and their moment — VERIFY, do not trust

The classification below was derived by reading the source that supplies `source` at each site.
Every `TriggerTimeLookBack` row is a site whose own existing comment already says CR 603.10a /
LKI, so the classification is mechanically checkable. **The runner must re-verify each row and
say so in the review.**

| # | site | trigger event(s) | source handed in | source's zone at gate time | moment |
|---|---|---|---|---|---|
| 1 | `abilities.rs:4559` | `SelfDies` | `*new_grave_id` | Graveyard | **LookBack** |
| 2 | `abilities.rs:4622` | `SelfLeavesBattlefield` | `*new_grave_id` | Graveyard | **LookBack** |
| 3 | `abilities.rs:4872` | `AnyCreatureDies` | `obj.id` (the **observer**) | Battlefield | **TriggerTime** |
| 4 | `abilities.rs:4917` | `SelfDies` / `SelfLeavesBattlefield` (Aura LTB) | `*new_grave_id` | Graveyard | **LookBack** |
| 5 | `abilities.rs:5014` | `SourceConnives` | `*object_id`, **any zone** (CR 701.50b) | any | **LookBack** |
| 6 | `abilities.rs:5475` | `AnyCreatureYouControlBatchCombatDamage` | `obj_id` | Battlefield | **TriggerTime** |
| 7 | `abilities.rs:5785` | `SelfLeavesBattlefield` (champion, to graveyard) | `*new_grave_id` | Graveyard | **LookBack** |
| 8 | `abilities.rs:5843` | `SelfLeavesBattlefield` (to exile) | `*new_exile_id` | Exile | **LookBack** |
| 9 | `abilities.rs:5901` | `SelfLeavesBattlefield` (bounce) | `*new_hand_id` | Hand | **LookBack** |
| 10 | `abilities.rs:6366` | `SelfLeavesBattlefield` (sacrifice) | `*new_id` | Graveyard/Exile | **LookBack** |
| 11 | `abilities.rs:6550` | `PermanentBecomesTarget` | `src.id` | Battlefield *(verify the enclosing scan)* | **TriggerTime** |
| 12 | `abilities.rs:6879` | **all 34 lowered events** (`collect_triggers_for_event`) | `obj_id`, guarded `zone == Battlefield` at `:6593` | Battlefield | **TriggerTime** |
| 13 | `abilities.rs:6935` | emblem sweep | `obj_id` | **Command zone** — CR 113.6p; `SourceOnBattlefield` correctly reads false | **TriggerTime** |
| 14 | `resolution.rs:2378` | resolution re-check | `source_object` (may be LKI) | any | **Resolution** |

8 LookBack / 5 TriggerTime / 1 Resolution. **Site 12 is the one that matters for the headline
bug**; the other 13 gain the classification for free and change nothing (§4.4).

### 4.3 The CR 603.10a carve-out is deliberate, and it is a stated deviation

`TriggerTimeLookBack` returns `true` for a card-def condition without evaluating it. That is a
**known, bounded deviation from CR 603.4 sentence 1**, not an oversight:

- Evaluating would be *wrong*: `check_condition` reads the current state, in which the source is
  gone, so `SourceOnBattlefield` / `SourceHasCounters` / `SourceIsUntapped` all read false and the
  trigger is destroyed. CR 603.10a requires the pre-event state, which the engine cannot supply.
- Not evaluating is *safe*: the trigger still goes on the stack, and the **Resolution** re-check
  still runs (site 14). So the deviation degrades to "over-fires in exactly the case where the
  alternative was to silently under-fire", which is hard constraint 3's stated preference.
- **Corpus exposure today is zero** — no def in `crates/card-defs/src/defs/` pairs `WhenDies`,
  `WhenLeavesBattlefield` or `WhenConnives` with an `intervening_if` (§6). 0 flips either way.
- Record it in the `InterveningIfMoment::TriggerTimeLookBack` doc comment and seed it
  (**OOS-DX1-1**). Do **not** leave a comment claiming CR 603.4 holds unconditionally.

### 4.4 What must NOT change

- The two legacy `InterveningIf` variants must behave identically at all three moments. Pinned
  by T7 (regression) and by the untouched `evolve` / `graft` / persist / undying tests.
- `carddef_intervening_if_holds_at_queue_time`'s body must not change. PB-DP6's 14
  Category-A registry sites keep calling it directly with their own sources; nothing about them
  moves. (`abilities.rs:6252`'s `WheneverYouSacrifice` retain stays **ungated** — OOS-DP6-2.)
- `resolution.rs:2136-2216`'s registry branch keeps its own `check_condition` call at `:2304`,
  with one harmonising addition (§9.3).
- **SR-25 `bare_lookup_ratchet` must not move.** Pinned ceilings on the touched files:
  `src/rules/abilities.rs` **75**, `src/rules/resolution.rs` **100**, `src/effects/mod.rs` **110**
  (`tests/core/bare_lookup_ratchet.rs:98/112/129`). Use `fizzle_object`, never `.objects.get(`.
  The ratchet fails in **both** directions.
- **SR-7**: every `PendingTrigger` still goes through `PendingTrigger::blank`. This PB adds none.

---

## 5. The 34 push sites in `build_face_ability_vectors`

All 34 are `if let AbilityDefinition::Triggered { … , .. }` matches, so **every one of them can
carry an `intervening_if`** — the field is on the variant, not on the condition. The table below
is the complete enumeration; `once_per_turn` column is for §10.

| # | `intervening_if: None` line | card-def `TriggerCondition` | runtime `TriggerEvent` | `once_per_turn` today |
|---|---|---|---|---|
| 1 | 2488 | `WhenDies` | `SelfDies` | dropped |
| 2 | **2526** | **`WhenAttacks`** | **`SelfAttacks`** | dropped |
| 3 | 2553 | `WhenBlocks` | `SelfBlocks` | dropped |
| 4 | 2589 | `WhenDealsCombatDamageToPlayer` | `SelfDealsCombatDamageToPlayer` | dropped |
| 5 | 2617 | `WhenDealtDamage` | `SelfIsDealtDamage` | dropped |
| 6 | 2665 | `WheneverOpponentCastsSpell{..}` | `OpponentCastsSpell` | dropped |
| 7 | 2692 | `WheneverYouSurveil` | `ControllerSurveils` | dropped |
| 8 | 2720 | `WhenConnives` | `SourceConnives` | dropped |
| 9 | 2747 | `WheneverYouInvestigate` | `ControllerInvestigates` | dropped |
| 10 | 2810 | `WheneverYouCastSpell{..}` | `ControllerCastsSpell` | **dropped — §10** |
| 11 | 2874 | `WheneverCreatureEntersBattlefield{..}` | `AnyPermanentEntersBattlefield` | **dropped — §10** |
| 12 | 2987 | `WheneverPermanentEntersBattlefield{..}` | `AnyPermanentEntersBattlefield` | dropped |
| 13 | 3019 | `WhenMutates` | `SelfMutates` | dropped |
| 14 | 3046 | `WhenSelfBecomesTapped` | `SelfBecomesTapped` | dropped |
| 15 | 3076 | `WheneverPermanentUntaps{filter}` | `AnyPermanentUntaps` | *propagated* (`:3070`) |
| 16 | 3112 | `WhenCounterPlaced{..}` | `CounterPlaced` | *propagated* (`:3106`) |
| 17 | 3160 | `WheneverCreatureDies{..}` | `AnyCreatureDies` | *propagated* (`:3158`) |
| 18 | 3192 | `WheneverCreatureYouControlAttacks{filter}` | `AnyCreatureYouControlAttacks` | dropped |
| 19 | 3226 | `WheneverCreatureYouControlDealsCombatDamageToPlayer{filter}` | `AnyCreatureYouControlDealsCombatDamageToPlayer` | dropped |
| 20 | 3255 | `WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer{filter}` | `AnyCreatureYouControlBatchCombatDamage` | dropped |
| 21 | 3282 | `WhenEquippedCreatureDealsCombatDamageToPlayer` | `EquippedCreatureDealsCombatDamageToPlayer` | dropped |
| 22 | 3309 | `WhenEquippedCreatureDealsCombatDamage` | `EquippedCreatureDealsCombatDamage` | dropped |
| 23 | 3335 | `WhenEnchantedCreatureDealsDamageToPlayer{..}` | `EnchantedCreatureDealsDamageToPlayer` | dropped |
| 24 | 3362 | `WhenAnyCreatureDealsCombatDamageToOpponent` | `AnyCreatureDealsCombatDamageToOpponent` | dropped |
| 25 | 3389 | `WheneverYouDiscard` | `ControllerDiscards` | dropped |
| 26 | 3414 | `WheneverOpponentDiscards` | `OpponentDiscards` | dropped |
| 27 | 3439 | `WheneverOpponentPlaysLand` | `OpponentPlaysLand` | dropped |
| 28 | 3466 | `WheneverYouSacrifice{..}` | `ControllerSacrifices` | dropped |
| 29 | **3495** | **`WheneverYouAttack{filter}`** | **`ControllerAttacks`** | dropped |
| 30 | 3520 | `WhenLeavesBattlefield` | `SelfLeavesBattlefield` | dropped |
| 31 | 3546 | `WheneverYouDrawACard` | `ControllerDrawsCard` | dropped |
| 32 | 3583 | `WheneverPlayerDrawsCard{player_filter}` | (per `player_filter`) | dropped |
| 33 | 3609 | `WheneverYouGainLife` | `ControllerGainsLife` | dropped |
| 34 | 3651 | `WhenBecomesTarget{..}` | `PermanentBecomesTarget{..}` | dropped |

Rows **2** and **29** are the two that carry corpus exposure today (§6). Row 12 carries Tatyova.

`TriggeredAbilityDef` is also constructed at 10 non-lowering sites — `replay_harness.rs:3812/3829/
3866`, `state/builder.rs` (21 literals incl. the two `InterveningIf::SourceHadNoCounterOfType`
pushes at `:610`/`:651`), `state/ability_definition_registry.rs:435/445`, `resolution.rs:892` —
all keyword-derived, none with a card-def condition in hand. **(a′) leaves every one untouched.**

### 5.1 Fields the lowering drops — the complete audit

Record this table in the source (a module-level comment on `build_face_ability_vectors`), because
the batch's real lesson is that the lowering is lossy and nothing said so:

| `AbilityDefinition::Triggered` field | status after PB-DX1 |
|---|---|
| `trigger_condition` | mapped to `trigger_on` (by construction) |
| `effect` | propagated |
| `targets` | propagated |
| `intervening_if` | **propagated (this batch)** |
| `once_per_turn` | **propagated (this batch, §10)** — was dropped at 31/34 |
| `modes` | collapsed to mode 0 as a bot fallback — deliberate, OOS-DP8-7 / **PB-DX10** |
| `trigger_zone` | **no runtime home** — dropped; `collect_triggers_for_event` scans the battlefield only and the graveyard sweep is a separate registry path. Seeded **OOS-DX1-3** |

---

## 6. Roster — derive from `all_cards()`, never from grep (SR-36)

### 6.1 The mandated derivation

Grep is a planning aid only. The runner **must** produce the roster by enumeration, and must ship
the enumeration as a permanent test (`T9`, §8):

```
for def in all_cards():
    for (face_is_transformed in [false, true]):
        for (idx, ability) in def.effective_abilities(face_is_transformed):
            if let AbilityDefinition::Triggered { trigger_condition, intervening_if, once_per_turn, .. }:
                classify trigger_condition as LOWERED (one of §5's 34) or REGISTRY
                record (def.name, def.completeness, idx, LOWERED?, intervening_if.is_some(), once_per_turn)
```

Walk `effective_abilities` for **both** faces, not `def.abilities` — a back-face-only trigger is
invisible to `def.abilities` and PB-OS4b/PB-RS4 made the back face live. PB-DP9's roster was
69/16/7 against the audit's 74/16/8 precisely because a flat scan undercounts; assume this one is
wrong until it is computed.

**Planning-time expectation, to be confirmed or falsified**: exactly **24** files carry
`intervening_if: Some(..)` (`rg -l '^\s*intervening_if: Some' crates/card-defs/src/defs/` → 24,
matching PB-DP6 §5.1), and of those exactly **3** sit on a lowered condition. If the enumeration
returns a different number, **the enumeration wins** and the plan's §6.2 must be re-derived.

### 6.2 The three defs on a lowered condition (expected)

| def | marker | condition | lowered site | oracle (MCP) | disposition |
|---|---|---|---|---|---|
| `aurelia_the_warleader.rs:33` | **`Complete`** (no `completeness` field) | `WhenAttacks` + `IsFirstCombatPhase` | row 2 | *"Whenever Aurelia attacks for the first time each turn, untap all creatures you control. After this phase, there is an additional combat phase."* | **REPAIRED BY THE ENGINE FIX. No def edit. No marker flip** (it is already `Complete`). See 6.3. |
| `karlach_fury_of_avernus.rs:42` | `known_wrong` | `WhenAttacks` + `IsFirstCombatPhase` | row 2 | *"Whenever you attack, if it's the first combat phase of the turn, untap all attacking creatures. They gain first strike until end of turn. After this phase, there is an additional combat phase."* | **CANDIDATE FLIP** → `Complete`, see 6.4 |
| `tatyova_steward_of_tides.rs:89` | `partial` | `WheneverPermanentEntersBattlefield` + `ControlAtLeastNOtherLands(6)` | row 12 | *"Land creatures you control have flying. Whenever a land you control enters, if you control seven or more lands, up to one target land you control becomes a 3/3 Elemental creature with haste. It's still a land."* | **NO FLIP.** Its `partial` note names two blockers this batch does not touch (no `EffectFilter` intersecting card types → the flying grant is unimplemented; `targets` is bare `TargetLand`, should be `UpToN{1, TargetLandWithFilter{controller: You}}`). Behaviour improves; marker stays. |

**The brief's "unblocks karlach and tatyova" is therefore over-stated by one.** Tatyova is not
unblocked. Expected honest yield: **1 flip, oracle-gated**.

### 6.3 Aurelia: the fix is right, and the residual divergence must be seeded not hidden

With the engine fix, `Condition::IsFirstCombatPhase` (`effects/mod.rs:9872`,
`!state.turn.in_extra_combat`) gates at both ends: combat 1 → queue-time true, resolution-time
true (still not in an extra combat) → untap + `AdditionalCombatPhase`; extra combat → queue-time
false → **no second trigger**. Exactly one extra combat. The loop is closed.

But the printed text is *"attacks **for the first time each turn**"*, and `IsFirstCombatPhase` is
a **proxy**, not a translation. They diverge when Aurelia attacks only in a *later* combat phase
(e.g. she is blinked in during combat 1 and attacks in an extra combat granted by another source):
the real card triggers, the def does not. The faithful authoring is `once_per_turn: true` with no
`intervening_if` — which §10 makes expressible for the first time. **Do not re-author her in this
batch**: the change would alter which mechanism the headline probe exercises, and it must be
argued on its own oracle merits. **Seed it (OOS-DX1-5)** with this analysis, and leave a source
comment on `aurelia_the_warleader.rs:28-29` recording that `IsFirstCombatPhase` is a proxy and why.

### 6.4 Karlach: the flip is earned only if the runner verifies it

`karlach_fury_of_avernus.rs:77-81`'s `known_wrong` note names exactly one defect and asserts its
fix already exists: *"'whenever you attack' modelled as WhenAttacks on Karlach — she must attack
personally; `TriggerCondition::WheneverYouAttack` now exists and is wired … and should be used."*
With PB-DX1, row 29 (`WheneverYouAttack` → `ControllerAttacks`) carries the intervening-if, so the
combination the note asks for becomes expressible for the first time.

**Required before flipping** (a wrong flip on a `known_wrong` def ships a legal-but-wrong card —
`validate_deck` stops rejecting it):
1. Re-read the oracle text via MCP and check **every** clause, not just the named one:
   `ForEachTarget::EachAttackingCreature` × untap; the first-strike grant via
   `ApplyContinuousEffect` on `EffectFilter::DeclaredTarget`; `Effect::AdditionalCombatPhase
   { followed_by_main: false }`; and `KeywordAbility::ChooseABackground`.
2. Confirm `TriggerCondition::WheneverYouAttack { filter: None }` fires **once per combat for the
   controller**, not once per attacker (`abilities.rs:6752-6790` says batch; PB-OS11 shipped it).
3. Ship a fail-before probe: Karlach attacks in combat 1 → one extra combat; attacks again in the
   extra combat → **no** further trigger.
4. If any clause fails, **keep `known_wrong`, narrow the note to the surviving defect, and say so
   in the review.** Default is file, not demote (PB-DP10 §5.3 precedent).

### 6.5 Golden scripts

`rg -l 'Aurelia|Karlach|Tatyova|Welcoming Vampire|Elvish Warmaster|Whispering Wizard'
test-data/generated-scripts/` → **0 files** at planning time. Re-run before starting; SR-9c
forbids silent skips, so a broken script surfaces. Any changed expectation needs a one-line
CR 603.4 (or 603.2h) justification in the diff. **Do not adjust a test to fit.**

---

## 7. Wire + hash re-pin protocol (SR-8, SR-17, SR-27)

Read `docs/mtg-engine-protocol-versioning.md` first. PB-DP9 (`scutemob-157`) is the worked model.

### 7.1 Sequence — one commit

1. Land the variant. Run `cargo test -p mtg-engine --test core` and let
   `protocol_schema.rs` / `hash_schema.rs` **fail**. Read the recomputed digests out of the
   failure text. **Never compute or guess a fingerprint by hand.**
2. `rules/protocol.rs`: `PROTOCOL_VERSION` `31` → **`32`**; add a `- 32:` line to the `# History`
   doc block naming the type, the field/variant, the CR and the reason the closure moved
   (type **count** is unchanged — `Condition` is already in the closure via `Effect::Conditional`,
   `InterveningIf`'s declared shape moved); **append** a `ProtocolEpoch { version: 32, fingerprint }`
   row to `PROTOCOL_HISTORY` (**never edit an existing row**); set
   `PROTOCOL_SCHEMA_FINGERPRINT` to the same value.
3. `tests/core/protocol_schema.rs`: update the `protocol_version_sentinel` (`:870`) and the FROZEN
   prefix digest.
4. `state/hash.rs`: `HASH_SCHEMA_VERSION` `68` → **`69`**; add a `- 69:` History line stating that
   **both** `decl_fingerprint` (a new enum variant) and `stream_fingerprint` (a new `HashInto` arm
   + the v40 mechanism) move; **append** a `HashSchemaEpoch { version: 69, decl_, stream_ }` row to
   `HASH_SCHEMA_HISTORY` (`:722-1011`).
5. `impl HashInto for InterveningIf` (`hash.rs:3526-3539`): add
   `InterveningIf::CardDef(c) => { 2u8.hash_into(hasher); c.hash_into(hasher); }`.
   Discriminant **2** is free (0 = `ControllerLifeAtLeast`, 1 = `SourceHadNoCounterOfType`).
   `impl HashInto for Condition` and `impl<T: HashInto> HashInto for Box<T>` (`hash.rs:1155`)
   both already exist — no new impl.
6. Re-pin **every** sentinel. Use a **symbol** grep, not a literal-`68` grep:
   ```
   rg -n 'HASH_SCHEMA_VERSION' crates/ tools/
   rg -n 'PROTOCOL_VERSION'    crates/ tools/
   ```
   At planning time that is **53** `HASH_SCHEMA_VERSION` assertion sites across ~48 files and
   **9** `PROTOCOL_VERSION` sites — nearly all in `crates/engine/tests/primitives/*.rs`, plus
   `tests/rules/loyalty_target_validation.rs`, `tests/casting/optional_cost_and_counter_tax.rs`,
   `tests/mechanics_e_l/effect_sacrifice_permanents_filter.rs`, `tests/core/hash_schema.rs:1200`,
   `tests/core/protocol_schema.rs:870`. Forms vary (`68`, `68u8`, inline and multi-line
   `assert_eq!`) — which is exactly why the literal grep is banned.
7. `docs/mtg-engine-protocol-versioning.md`: record the bump.

### 7.2 SR-19 field-coverage gate

`tests/core/hash_schema.rs:1207+` parses each hashed **struct** and asserts every declared field
reaches `HashInto`. (a′) adds no struct field, so `NOT_HASHED` (`:1254`, empty) stays empty and
the gate is a no-op. The new **enum** variant is covered by the SR-17 `decl_fingerprint` instead.
*(If the runner deviates to (a), the SR-19 gate will demand `carddef_intervening_if` be hashed —
hash it, do not allowlist it.)*

### 7.3 Bench check

`Characteristics` is cloned on every `calculate_characteristics`. `Box<Condition>` keeps
`InterveningIf` at 16 bytes, so no growth is expected — but run
`cargo bench -p mtg-engine` against the merge base in a throwaway worktree (PB-DP9's method) and
report `priority_cycle_4p` / `sba_check` / `full_turn_4p`. Baseline at PB-DP9:
`full_turn_4p` ≈ **229 µs**. A >5 % regression is a stop-and-report.

### 7.4 The two stale protocol-history parentheticals

If §3.5's prediction holds (protocol digest moves), then `rules/protocol.rs`'s `- 25:` note
(`:229-232`) and `- 26:` note (`:245-247`) — both asserting `TriggerEvent`/`TriggerCondition` are
"not in the wire closure" — are **false for `TriggerEvent`**, which is
`TriggeredAbilityDef.trigger_on`. Correct the prose in place (History lines are doc comments, not
`PROTOCOL_HISTORY` rows; the append-only rule does not cover them) per `memory/conventions.md`'s
aspirationally-wrong-comment rule, and note the correction in the `- 32:` line. If the runner can
establish `TriggerCondition`'s status too (it is reached via `TargetFilter`/`AbilityDefinition`,
not via `Characteristics`), say so; if not, say "unverified" rather than guessing.

---

## 8. Tests

**New file**: `crates/engine/tests/primitives/pb_dx1_lowered_intervening_if.rs`
**Registration**: `crates/engine/tests/primitives/main.rs` — insert `mod
pb_dx1_lowered_intervening_if;` **after line 29** (`mod pb_dp9_effect_choice;`), keeping the list
sorted. SR-9a: never add a top-level `tests/*.rs`.
**Patterns to copy**: `tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs:16-120` (the
`all_cards()` + `enrich_spec_from_def` + real-registry + engine-driven idiom — this is the model
for the probe); `tests/combat/additional_combat.rs:29-46` (`pass_until_step_advance`);
`tests/primitives/pb_os11_batch_filtered_attack_trigger.rs:161-169` (the
`Command::DeclareAttackers` shape).

Every test cites its CR section (Architecture Invariant 8).

### 8.1 T1 — THE MANDATORY FAIL-BEFORE PROBE

`test_dx1_aurelia_attack_trigger_fires_exactly_once_per_turn`
**CR 603.4 / 508.3a / 500.8.**

Spec, non-negotiable in these three respects:

1. **The real def.** `all_cards()` → `CardRegistry`; the Aurelia object built with
   `enrich_spec_from_def(ObjectSpec::card(p1, "Aurelia, the Warleader")
   .with_card_id(cid("aurelia-the-warleader")).in_zone(ZoneId::Battlefield), &defs)`.
   No synthetic ability, no hand-built `TriggeredAbilityDef`.
2. **Engine-driven, no state pokes.** The extra combat must be produced by the *card*, via
   `Effect::AdditionalCombatPhase` resolving off the stack. **Do not** write
   `state.turn_mut().additional_phases` or `in_extra_combat`. `tests/combat/additional_combat.rs`
   pokes `additional_phases` directly — that is the *control-group* idiom, not this one. PB-DP6's
   close-out probe (throwaway, never committed; recorded only in `docs/audits/
   decision-point-audit.md` §8.1's OOS-DP6-1 row) established by execution that a paired
   poke/no-poke pair gives byte-identical results here; this test is its committed successor.
3. **The assertion is a count, over the whole turn.** Not "the second combat has no trigger" —
   count `GameEvent`s attributable to the ability across both combats.

Shape:
- 2 players, p1 active, `Step::DeclareAttackers`, Aurelia + one other untapped creature p1
  controls (so the untap is observable), p2 with a library.
- `Command::DeclareAttackers { player: p1, attackers: [(aurelia, AttackTarget::Player(p2))],
  enlist_choices: vec![], exert_choices: vec![] }`.
- Drain the stack by passing priority. Assert the trigger resolved **once** and
  `state.turn().additional_phases.len() == 1`.
- Advance through `EndOfCombat` with `pass_until_step_advance` until `Step::BeginningOfCombat`
  with `state.turn().in_extra_combat == true`.
- Declare Aurelia as an attacker again (she has vigilance, so she is untapped).
- Drain. **Assert the ability did NOT trigger a second time**: total resolutions over the turn
  `== 1`, and `additional_phases` is empty (no third combat granted).

**Fail-before prediction**: on unmodified code this test **FAILS** — the second declaration
queues the trigger again and `additional_phases` goes back to 1, granting a third combat.
Run it against `git stash`-ed engine changes and paste the failure into the review. *A probe that
passes before the fix is a test-validity HIGH (`memory/conventions.md`), not a LOW.*

### 8.2 The rest

| # | test | asserts | fail-before |
|---|---|---|---|
| T2 | `test_dx1_lowered_condition_gates_at_queue_time` | Synthetic def, `WhenAttacks` + a false `Condition`, real lowering: `pending_triggers` is **empty** after `DeclareAttackers`. CR 603.4 s1. | **FAILS** (queued today) |
| T3 | `test_dx1_lowered_condition_rechecked_at_resolution` | Condition **true at declaration, false at resolution** (flip the state between the two, e.g. via an intervening spell): the ability reaches the stack and resolves with **no effect**. CR 603.4 s2. **This is the test that proves the fix is not queue-only.** | **FAILS** (resolves with full effect today) |
| T4 | `test_dx1_lowered_condition_true_still_fires` | The non-regression twin of T2. | passes before and after |
| T5 | `test_dx1_unevaluable_condition_does_not_suppress` | Lowered trigger whose condition is `Condition::TargetIsLegal { index: 0 }` (or `And(YouAttackedThisTurn, WasCleaved)`): **still queued, and still resolves**. Hard constraint 3, both ends. | passes before; its value is entirely post-fix |
| T6 | `test_dx1_lookback_dies_trigger_not_suppressed` | `WhenDies` + `Condition::SourceOnBattlefield` on a lowered def: the trigger **still fires** after the creature dies. CR 603.10a + hard constraint 3. Pins §4.3's deviation *in the direction of over-firing* so a later "simplification" to two-valued moments reddens. | passes before (nothing gates); must pass after |
| T7 | `test_dx1_legacy_intervening_if_variants_unchanged` | `ControllerLifeAtLeast` and `SourceHadNoCounterOfType` answer identically at `TriggerTime`, `TriggerTimeLookBack` and `Resolution`. The regression pin for Edit 2. | passes before and after |
| T8 | `test_dx1_face_change_carries_back_face_condition` | A DFC whose **back**-face `WhenAttacks` carries an `intervening_if`: after `apply_face_change` (`rules/face.rs:104`) the back face's condition gates. Pins the PB-OS4b/PB-RS4 contract through the new lowering. | passes before *vacuously*; must pass after |
| T9 | `test_dx1_corpus_roster_is_enumerated_not_grepped` | §6.1's `all_cards()` walk. Asserts the LOWERED×`intervening_if.is_some()` set **and** the LOWERED×`once_per_turn` set by name, with a non-vacuity floor on the denominator. Becomes the permanent gate that a new such def cannot land unnoticed. | new |
| T10 | `test_dx1_turn_face_up_intervening_if_rechecked_at_resolution` | Rider OOS-DP6-5, §9.1. Synthetic morph def. | **FAILS** |
| T11 | `test_dx1_haunt_intervening_if_gated_at_both_ends` | Rider OOS-DP6-9, §9.2. Synthetic haunt def, both queue and resolution. | **FAILS** |
| T12 | `test_dx1_karlach_extra_combat_once_per_turn` | §6.4 probe. **Only if the flip is taken.** | **FAILS** |
| T13-T15 | `test_dx1_once_per_turn_*` (welcoming vampire / elvish warmaster / whispering wizard) | §10. Real defs, two qualifying events in one turn, assert **one** resolution. CR 603.2c/h. | **FAIL** (each fires twice today) |

### 8.3 Existing tests: predicted impact

| test / corpus | prediction | reasoning |
|---|---|---|
| `mechanics_e_l/evolve.rs:1002`, `mechanics_e_l/graft.rs:857` | **UNCHANGED** | keyword-specific resolution checks; neither mechanism |
| `primitives/pb_ac8_restrictions_and_wingame.rs:332` | **UNCHANGED** | pushes `PendingTrigger::blank` directly, bypassing the sweep |
| `primitives/pb_dp6_intervening_if_queue_time.rs` (all 12) | **UNCHANGED** | all exercise registry-path Category-A sites, which this batch does not touch |
| `tests/rules/abilities.rs:854/:928`, `tests/mechanics_*` persist/undying | **UNCHANGED** | legacy `InterveningIf` variants, pinned by T7 |
| `tests/core/decision_gate.rs` `BASELINE` (97 defs) | **UNCHANGED unless karlach flips** | the frozen list is name-keyed and none of §6.2/§10's defs are in it (verified: `rg 'Karlach\|Aurelia\|Welcoming\|Warmaster\|Whispering' crates/engine/tests/core/` → 0). If karlach's def is edited, re-run `decision_site_walk.rs` and reconcile |
| 211 golden scripts | **UNCHANGED** (§6.5: 0 name hits) | re-verify before starting |

---

## 9. Riders — dispositions

### 9.1 OOS-DP6-5 — `TurnFaceUp` resolution re-check: **FIX**

The audit's cite `resolution.rs:7369` is **stale**; the site is **`resolution.rs:7513-7550`**.
It destructures `AbilityDefinition::Triggered { effect, .. }` from `def.abilities.get(ability_index)`
at `:7530` and executes unconditionally. Its queue-time counterpart (PB-DP6's A12) is at
`abilities.rs:5971-5997` and **is** gated via `carddef_intervening_if_holds_at_queue_time`.

Fix: change `:7530`'s destructure to `{ effect, intervening_if, .. }` and gate `execute_effect`
on the same evaluability-guarded `check_condition` shape as `resolution.rs:2299-2316`.
**Index spaces match** (both ends use `def.abilities`), so there is no OOS-DP6-2 hazard here —
say so in the code comment. *Neither end uses `effective_abilities(is_transformed)`; that
face-awareness gap is consistent across both ends and out of scope — seed **OOS-DX1-4** (shared
with §3.3 point 4).*

Latent: no corpus `WhenTurnedFaceUp` def carries an `intervening_if` (§6.1's enumeration must
confirm). **0 flips.** Probe T10 uses a synthetic def.

### 9.2 OOS-DP6-9 — haunt, both ends: **FIX**

The audit's cite `resolution.rs:5351` is stale; the site is **`resolution.rs:5500-5526`**, the
`find_map` over `def.abilities` looking for `TriggerCondition::HauntedCreatureDies`. It returns
`effect.clone()` and never reads `intervening_if`. The queue site
(`abilities.rs:4745-4780`) pushes purely off `haunting_target` and never looks at the ability at
all, so it has no gate either.

Fix, both ends:
- Resolution: widen the `find_map` to return `(effect, intervening_if)` and gate.
- Queue: mirror the same `find_map` in `abilities.rs:4766`'s loop and gate with
  `carddef_intervening_if_holds_at_queue_time`. The haunt source is in **Exile** (CR 702.55c), so
  a `SourceOnBattlefield` condition correctly reads false — comment it rather than special-casing.

Latent, **0 flips**. Probe T11 uses a synthetic haunt def.

Both riders are strictly additive on paths with zero corpus exposure, which is exactly why they
belong here: they close CR 603.4's second sentence at the last two places it is open, at zero
flip risk, in the one batch whose whole subject is that asymmetry.

### 9.3 In-scope harmonisation (not a rider, but do it)

`resolution.rs:2304-2316` (the registry path's re-check) evaluates `check_condition`
**unconditionally**. After Edit 2, the runtime path guards with
`condition_is_queue_time_evaluable`. Leaving them different means the same condition behaves
differently depending on which lowering path the trigger took — the precise asymmetry this batch
exists to delete. Add the identical guard at `:2304`. Corpus impact: **zero** (no def uses one of
the seven unevaluable variants as an `intervening_if` — confirm via T9). Same for
`resolution.rs:7530` and `:5500` once §9.1/§9.2 land: one guard shape, four sites.

---

## 10. The `once_per_turn` rider — NEW finding, in scope, separately committed

**Not in the brief.** Found in planning (P6). Same root cause, same 34 sites, **no new type, no
new field, no wire movement.**

`once_per_turn` is hardcoded `false` at 31 of 34 push sites. `flush_pending_triggers`
(`abilities.rs:7957-7993`) reads the runtime value first and only consults the card registry when
the **runtime lookup misses**; for a lowered trigger it hits and returns `false`. Three
`Complete`, deck-legal defs over-fire today (oracle text via MCP):

| def | trigger | lowering row | oracle |
|---|---|---|---|
| `welcoming_vampire.rs:27` | `WheneverCreatureEntersBattlefield` | 11 | *"Whenever one or more other creatures you control with power 2 or less enter, draw a card. **This ability triggers only once each turn.**"* |
| `elvish_warmaster.rs:30` | `WheneverCreatureEntersBattlefield` | 11 | *"Whenever one or more other Elves you control enter, create a 1/1 green Elf Warrior creature token. **This ability triggers only once each turn.**"* |
| `whispering_wizard.rs:25` | `WheneverYouCastSpell` | 10 | *"Whenever you cast a noncreature spell, create a 1/1 white Spirit creature token with flying. **This ability triggers only once each turn.**"* |

(`morbid_opportunist`, `spiteful_banditry`, `dusk_legion_duelist` already work — their conditions
lower at rows 15/16/17, the three sites that already propagate.)

**Fix**: at each of the 31 dropping sites, add `once_per_turn` to the destructure and write
`once_per_turn: *once_per_turn` instead of `false`. Rows 15/16/17 already do exactly this — copy
them. **Zero marker flips** (all three are already `Complete`; they simply stop over-firing).

**Scope discipline**: commit this **separately** from the `intervening_if` work, after it, with
its own probes (T13-T15) and its own roster row in T9. If it destabilises anything, **drop it and
seed it** rather than blocking the correctness fix — the headline bug is the priority. Say which
you did in the review.

---

## 11. Seeds to file (`docs/audits/decision-point-audit.md` §8.1)

| seed | finding | class |
|---|---|---|
| **OOS-DX1-1** | **A card-def intervening-if on a leave-the-battlefield trigger is never evaluated.** PB-DX1's `InterveningIfMoment::TriggerTimeLookBack` returns `true` unconditionally at 8 of 14 queue sites, because `check_condition` reads the *current* state and CR 603.10a requires the pre-event state. Closing it needs an LKI-aware `check_condition` fed by the `pre_death_characteristics` / `lki_counters` / `lki_power` snapshots those sites already thread. Zero corpus exposure today. | correctness, stated deviation |
| **OOS-DX1-2** | **`condition_is_queue_time_evaluable` is reused at resolution time and is over-conservative there by exactly one variant.** `Condition::TargetIsLegal` *is* answerable at resolution (`stack_obj.targets` exists) but is treated as unanswerable. Harmless today because CR 608.2b's fizzle at `resolution.rs:2274` covers the same case. A `condition_is_resolution_time_evaluable` split would be strictly more correct. | correctness, latent |
| **OOS-DX1-3** | **`AbilityDefinition::Triggered.trigger_zone` has no runtime home and is dropped by the lowering** at all 34 sites. `collect_triggers_for_event` scans the battlefield only; the graveyard sweep is a separate registry path (PB-DP6 A14). A def pairing a lowered condition with `trigger_zone: Some(Graveyard)` would function from the battlefield instead. | DSL gap / correctness, latent |
| **OOS-DX1-4** | **The `CardDefETB`-style registry dispatch indexes two different ability lists at its two ends.** Queue: `def.abilities` (`abilities.rs:4102`). Resolution: `def.effective_abilities(obj.is_transformed)` (`resolution.rs:2192`). For a DFC carrying the ability on the back face these disagree. Same shape at `resolution.rs:7513` (TurnFaceUp) and `:5500` (haunt), where **both** ends use `def.abilities` and so are self-consistent but face-blind. | correctness, latent |
| **OOS-DX1-5** | **Aurelia's `IsFirstCombatPhase` is a proxy for "for the first time each turn" and diverges.** If she attacks only in a later combat phase the printed card triggers and the def does not. `once_per_turn: true` (expressible after PB-DX1 §10) is the faithful authoring. Deliberately not re-authored in PB-DX1 so the headline probe keeps exercising the intervening-if mechanism. | card correctness, narrow |
| **OOS-DX1-6** | *(only if §10 is dropped)* the `once_per_turn` half of the lowering drop, §10 verbatim. | correctness, live |
| **OOS-DX1-7** | *(only if §7.4 confirms)* `rules/protocol.rs`'s `- 25:`/`- 26:` History parentheticals mis-state the wire closure. If corrected in this batch, record as closed-on-arrival instead. | documentation |

Close in `docs/audits/decision-point-audit.md` §8.1: **OOS-DP6-1**, **OOS-DP6-5**, **OOS-DP6-9**.
Correct the stale cites in the OOS-DP6-5 and OOS-DP6-9 rows (`7369` → `7513`, `5351` → `5500`)
while closing them — the class of documentation rot PB-DP6 filed as OOS-DP6-8.

---

## 12. Ordered step list for the runner

**Phase 0 — probe first, no production code.**
1. Write T1 (§8.1) alone. Run it. **It must fail.** Paste the failure text into the review. If it
   passes, STOP and report — the premise is falsified.

**Phase 1 — the type.**
2. Add `InterveningIf::CardDef(Box<Condition>)` (Edit 1). `cargo check -p mtg-card-types`.
3. `cargo check -p mtg-engine` — the only breakage should be `check_intervening_if`'s
   non-exhaustive match and `impl HashInto for InterveningIf`. **If anything else breaks, there is
   a fifth reader of `.intervening_if` — stop and report it** (§3.2).

**Phase 2 — the evaluator.**
4. Add `InterveningIfMoment` and the new signature + `CardDef` arm (Edit 2).
5. Update all 14 call sites with the moment from §4.2's table, **re-verifying each source's zone**.
6. `resolution.rs:2378` gets `InterveningIfMoment::Resolution` and `source_object`.
7. Apply §9.3's harmonisation at `resolution.rs:2304`.
8. `cargo check -p mtg-engine`. T1 still fails (the lowering has not landed).

**Phase 3 — the lowering.**
9. All 34 sites (Edit 3), plus the comment cleanup at `:2563-2564` and the §5.1 table as a module
   comment on `build_face_ability_vectors`.
10. Run T1. **It must now pass.** `cargo build --workspace` (simulator / TUI / replay-viewer
    exhaustive matches — the runner-miss rate here is ~50 %).

**Phase 4 — wire + hash.** §7.1 steps 1-7, in one commit with Phase 1-3.

**Phase 5 — tests.** T2-T9 + the roster enumeration. `cargo test --all`.

**Phase 6 — riders.** §9.1, §9.2 + T10, T11. Separate commit.

**Phase 7 — `once_per_turn`.** §10 + T13-T15. Separate commit. Droppable (§10).

**Phase 8 — card defs.** §6.4's karlach verification + T12, or an honest "not earned" with the
narrowed note. §6.3's Aurelia comment. `tools/check-defs-fmt.sh`.

**Phase 9 — bookkeeping.** §11 seeds; audit rows closed with corrected cites; §7.4 prose fix;
`docs/mtg-engine-protocol-versioning.md`; benches (§7.3); `memory/primitive-wip.md` → review phase.

---

## 13. What "done" looks like — falsifiable

- [ ] T1 fails on unmodified code and passes after, with the pre-fix failure text quoted in the review.
- [ ] T3 exists and fails pre-fix — i.e. the **resolution** end is demonstrably fixed, not just the queue end.
- [ ] All **34** push sites carry `intervening_if`; `rg -c 'intervening_if: None' crates/engine/src/testing/replay_harness.rs` → **0**.
- [ ] All **14** `check_intervening_if` call sites carry an explicit `InterveningIfMoment`, each verified against its source's zone, tabulated in the review.
- [ ] `PROTOCOL_VERSION == 32`, `HASH_SCHEMA_VERSION == 69`; both fingerprints **gate-computed**; both histories **appended**, never edited; all ~62 sentinel sites re-pinned via the **symbol** grep.
- [ ] `cargo build --workspace`, `cargo test --all`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` — all green.
- [ ] `bare_lookup_ratchet` green with **no** ceiling edits.
- [ ] Roster produced by `all_cards()` enumeration (T9), not grep; the count is stated even if it equals the planning-time 24/3.
- [ ] Riders OOS-DP6-5 and OOS-DP6-9 **fixed** (not deferred), each with a fail-before probe.
- [ ] `once_per_turn` (§10) either fixed with T13-T15 green, or explicitly dropped and seeded as OOS-DX1-6 — stated either way.
- [ ] Karlach either flipped with the §6.4 four-point verification recorded, or left `known_wrong` with a narrowed note. **Tatyova stays `partial`** — if the review claims otherwise, it is wrong.
- [ ] 211 golden scripts green, 0 new skips; any changed expectation carries a CR citation.
- [ ] Benches reported against the merge base; `full_turn_4p` within 5 % of ~229 µs.
- [ ] Seeds OOS-DX1-1..5 (+6/7 as applicable) filed; OOS-DP6-1/5/9 closed with corrected cites.
- [ ] Test count reported; expected ≈ **3,928 → 3,940-3,945**.

---

## 14. Risks

1. **Suppressing a trigger that should fire.** The only way this batch makes the engine worse.
   Mitigated by: the `TriggerTimeLookBack` carve-out (§4.3), the evaluability guard at *both* ends
   (§4.1), T5, T6. **If in doubt at any site, queue the trigger.**
2. **Fixing only the queue end.** PB-DP6's review found exactly that shape at `resolution.rs:2299`.
   T3 is the specific antidote; it is not optional.
3. **The wire prediction being wrong in the *other* direction** — the protocol digest moves for a
   reason other than the one predicted (e.g. `Box` changing the declaration text of something
   unexpected). Read the failure text; it names the drifted types. Never re-pin without reading it.
4. **Sentinel re-pin misses.** ~62 sites, three syntactic forms. Symbol grep, not literal grep.
   A missed site is a red test, not a silent bug — but it costs a cycle.
5. **`cargo build --workspace` skipped after Phase 3.** Historic ~50 % miss rate on the
   replay-viewer / TUI exhaustive matches. `InterveningIf` has no external matcher today
   (verified), but the build is the gate, not the grep.
6. **Scope creep into OOS-DP6-2.** `abilities.rs:6252`'s `WheneverYouSacrifice` retain will look
   fixable. It is not — its index-space mismatch is the precondition, not the consequence. Leave it.
7. **Karlach flipped without full verification.** A `known_wrong` → `Complete` flip removes a
   `validate_deck` rejection. Four-point check or no flip (§6.4).
8. **The `once_per_turn` rider expanding.** It is 31 one-word edits. If it starts requiring
   changes to `flush_pending_triggers`, stop: that is a different batch (§10).
