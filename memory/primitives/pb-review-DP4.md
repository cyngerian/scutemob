# Primitive Batch Review: PB-DP4 — Costs checked but never collected (DP-10 + DP-11)

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules verified independently via MCP**: 508.1 (all of 508.1a-m), 508.1c, 508.1d, 508.1f,
508.1g, 508.1h, 508.1i, 508.1j, 106.6/106.6a, 107.4e/107.4f (via `debug_assert_flattened`),
118.12/118.12a/118.12b, 119.4/119.4a/119.4b, 608.2d, 608.2g, 702.24/702.24a/702.24b, 702.30a,
702.59a, 400.7, 101.4, 117.3b/117.3c/117.4, 500.4, 514.3a, 603.3
**Engine files reviewed**: `crates/engine/src/rules/combat.rs`,
`crates/engine/src/rules/engine.rs`, `crates/engine/src/rules/resolution.rs`,
`crates/engine/src/state/mod.rs`, `crates/card-types/src/state/player.rs`,
`crates/engine/src/rules/casting.rs` (read-only, to verify `can_pay_cost`/`pay_cost`),
`crates/engine/src/rules/abilities.rs` (`apnap_order`), `crates/engine/src/rules/replacement.rs`
(`StaticRestriction` registration), `crates/engine/src/rules/turn_structure.rs` (CR 508.8 skip)
**Non-engine files reviewed**: `crates/simulator/src/legal_actions.rs`,
`crates/simulator/src/random_bot.rs`, `crates/simulator/src/heuristic_bot.rs`,
`crates/simulator/src/local_game.rs` (`decision_kind_for`), `crates/simulator/Cargo.toml`
**Tests reviewed**: `crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs`
(22), `crates/simulator/src/legal_actions.rs` `mod tests` (8 new),
`crates/engine/tests/rules/restrictions.rs` (4 existing, 1 strengthened),
`crates/engine/tests/core/bare_lookup_ratchet.rs`, `crates/engine/tests/primitives/main.rs`,
`crates/engine/tests/mechanics_a_d/cumulative_upkeep.rs` (spot-traced)
**Card defs reviewed**: 3 — `propaganda.rs`, `ghostly_prison.rs` (oracle-verified, 0 edits),
`goblin_rabblemaster.rs` (comment-only edit). Oracle-verified `mogg_war_marshal`, `grim_harvest`
as the `Complete` echo/recover cards made right with 0 edits.

## Verdict: **ship after fixes**

**0 HIGH. 5 MEDIUM. 13 LOW.** Both halves of the PB are CR-correct in substance and the
load-bearing design judgement — the DP-11 deadline — is, in my independent assessment, **right**,
and it is the *smallest* deviation reachable under the no-wire-change constraint. I verified every
claim the coordinator asked me to trace rather than taking the plan's word:

- **`resolve_top_of_stack` has exactly one caller** (`handle_all_passed:1849`), and
  `handle_all_passed`'s two branches are a genuine `if/else` with an early `return` in the
  non-empty arm. **The sweep provably cannot fire in the call that created the entry.** ✅
- **`resolve_top_of_stack` unconditionally ends with `players_passed = OrdSet::new()` +
  `priority_holder = Some(active)` + `PriorityGiven`** (`resolution.rs:7782-7785`), and its
  early-return paths do the same (`:112-116`). So *every* player — including a non-active recover
  controller — is guaranteed a priority window between entry creation and the deadline, and the
  CR 608.2g mana-ability window is real (pinned by probe 20). ✅
- **Termination holds.** The guard is `!payment_events.is_empty()`, the sweep drains every entry
  on every call, and re-population strictly requires a *new* stack resolution, which requires the
  **other** branch of `handle_all_passed`. I walked the worst realistic chain (echo sacrifice →
  creature into graveyard → recover trigger → new entry → second sweep → exile → done) and it is
  bounded. No deadlock, no infinite retry: a seat that never sends `Pay*` is simply auto-declined,
  and a bot's `PassPriority` always succeeds. The fuzzer smoke run (4 wins / avg 193.6 turns)
  corroborates. ✅
- **Auto-decline is correct per CR 118.12a** (verbatim: "unless" ≡ "may …; if that player
  doesn't"). Auto-pay would be CR-wrong *and* would spend unelected resources. ✅
- **Deleting the three `priority_holder = Some(active_player)` bodges is correct** and closes
  OOS-DP1-1. For recover it is an unambiguous fix (non-active controller). For echo/CU the
  "identity write" claim is *nearly* right but slightly overstated (see LOW E4) — it is a fix
  there too whenever a responder took a priority-granting action first. Nothing else depends on
  the writes; the existing echo/CU/recover suites and golden script 153 pass unedited because
  they all answer immediately after the resolving pass round.
- **DP-10 is CR-correct.** I re-derived CR 508.1a-m: the audit's `508.1g` cite **is** imprecise
  (508.1g is *optional* "as it attacks" costs); Propaganda is a 508.1c restriction paid via
  508.1h/i/j. The plan's correction stands. Colour preservation, per-defender × attacker-count
  summation (Propaganda ruling 2004-10-04: "the cost is cumulative", "for each attack"),
  planeswalker exclusion, and the CR 508.1f→508.1j ordering are all right. The restricted-mana
  behaviour flip is **correct**: I read all six `ManaRestriction` variants in
  `card-types/src/cards/card_definition.rs` / `restriction_matches` and every one requires a
  `SpellContext`, so no restricted mana in this engine can pay a non-spell cost.
  `can_pay_cost`/`pay_cost` are `can_spend(cost, None)`/`spend(cost, None)` — check and payment
  are now the same predicate, as intended.
- **Change 1c belongs in scope.** CR 508.1d is verbatim on point and the 2014-07-18 Rabblemaster
  ruling says it in as many words; I confirmed the ruling text via MCP. The two guards (probes 9
  and 10) genuinely pin that must-attack is not blanket-disabled, and the helper reads
  `layers::expect_characteristics`, not `obj.characteristics` (W3-LC / CR 613.1f) — verified at
  `combat.rs:772`. It is scope creep, and I accept it: you cannot turn the tax into a real debit
  without making the pre-existing deadlock cheaper to reach (the same floating `{2}` no longer
  funds a second combat phase), so shipping 1a without 1c would have been incoherent.
- **Wire neutrality is real.** `PROTOCOL_VERSION == 27` (`protocol.rs:260`),
  `HASH_SCHEMA_VERSION == 63` (`hash.rs:578`). No `Command`/`GameEvent`/`GameState` field or
  variant added; the three `pending_*` fields keep their `pub(crate)` seal and their existing
  shapes, so nothing *should* have moved either constant. SR-3 not widened — `state/mod.rs` shows
  doc-comment edits only.
- **Registry gates**: neither `force_resolve_overdue_payments` nor
  `StubProvider::legal_actions` names `KeywordAbility::Echo`/`::CumulativeUpkeep`/`::Recover` in
  executable code; both read the pending vectors. `CumulativeUpkeepCost::Mana/Life` matching is
  registry-free. ✅

What blocks a clean ship: **two test-validity findings that `memory/conventions.md` escalates to
fix-phase HIGH** (T1 — the APNAP test cannot discriminate APNAP from the sweep's kind-grouped
iteration; T2 — the pass-set half of the OOS-DP1-1 probe is vacuous), **one un-tested guard that
the plan itself named as the highest-consequence failure shape** (T3 — the all-no-op sweep must
fall through, not spin), an **unscoped hybrid/Phyrexian rejection** that rejects declarations
against untaxed defenders and even empty declarations (E1), and a **bare-lookup ratchet ceiling
raised where the vocabulary this same PB uses elsewhere would have avoided it** (E2). None of
these implicate the design; all are mechanical.

---

## Rulings the coordinator asked for

### Ruling 1 — the DP-11 deadline design: **UPHELD**

(a) The CR 608.2d deviation is stated honestly, in three places (plan §3 2.0, the
`force_resolve_overdue_payments` doc block, and the three producer arms in `resolution.rs`), and it
enumerates the observable consequences. (b) It **is** the smallest deviation available: the sweep
fires at the first boundary at which every player has demonstrably held priority since the entry
was created, which is the earliest point a no-new-`Command` design can reach. I considered a
"one extra grace round" variant and it is strictly larger. (c) traced and confirmed above.
(d) traced and confirmed above, including the multi-player (probe 16) and multi-permanent
(probe 19) cases. (e) confirmed: no deadlock, no retry loop, and `driver.rs`'s silent-`PassPriority`
fallback cannot spin because `PassPriority` is never rejected on account of a pending payment.

Auto-decline over auto-pay is right per CR 118.12a and I would have rejected auto-pay.

One accuracy note, not a design objection: the deadline is described as "the end of the priority
round in which the ability resolved," but it is actually the end of the first *subsequent* round
that terminates with an empty stack. A player who keeps the stack non-empty postpones the sacrifice
indefinitely (bounded only by their resources). Logged as LOW E9.

### Ruling 2 — APNAP and multi-entry ordering: **CORRECT, but under-tested**

`apnap_order` is a deterministic rotation of `state.turn.turn_order` (`abilities.rs:8477-8491`);
the pending vectors are `imbl::Vector` (insertion-ordered); `combat.rs`'s two maps were switched to
`BTreeMap`/`BTreeSet`; the two `.any()` scans in `has_uncosted_attack_target` are order-independent
booleans over `OrdMap`s. **SR-9b determinism holds.** The entry snapshot is taken before any
handler runs (`engine.rs:1198-1203`, `:1214-1219`, `:1231-1236`) — correct, because each handler
`remove`s from the vector it is iterating. The CR 400.7 no-op decline is handled by the handlers'
own `Ok(vec![])` early returns and is pinned by probe 19's `died_count == 1`.

Two caveats: the *test* cannot distinguish APNAP from kind-grouping (T1), and grouping by player
before kind reorders closeouts relative to their actual resolution order (LOW E13).

### Ruling 3 — deleting the three priority writes: **CORRECT, closes OOS-DP1-1**

Both halves verified as described in the verdict. Nothing else read those writes.

### Ruling 4 — the DP-10 debit: **CR-CORRECT**

Summation, ordering, restricted-mana exclusion, and hybrid/Phyrexian rejection are all right in
substance. The *scoping* of the rejection is wrong (E1) and the failure message is over-broad and
partly an implementation-detail assertion (E5).

### Ruling 5 — Change 1c scope and must-attack strength: **ACCEPTED**

See verdict. The guards do pin it. Layer-resolved read confirmed.

### Ruling 6 — the new CR 119.4 life gate: **REAL BUG, RIGHT FIX**

There genuinely was an unguarded subtraction: `engine.rs:887-890` subtracts `total_life` with no
prior check, and the sibling `Mana` arm at `:849-860` does check. Pre-fix a player at 5 life paying
a 6-life cumulative upkeep went to **-1 and then lost the game to SBA 704.5a** — which is worse
than "silent", since CR 702.24a's correct outcome is sacrificing the permanent. `Err` (rather than
an engine-side decline) is the right response: the command is illegal, the entry stays pending, and
the boundary sweep then applies the CR-mandated sacrifice. `GameStateError::InsufficientLife` is
the existing SR-36 idiom; no new variant. The `?` on the player lookup is safe because the sweep
never enters the `pay: true` arm.

### Ruling 7 — the three un-enumerated sites

- **`heuristic_bot.rs`** — legitimate un-enumerated site (§4.6 said "delegates to
  `action_to_command`, no change", missing the *scoring* match). Reported and fixed minimally.
  `pay: true` = 45, `pay: false` = 2, `PassPriority` = 1, `TapForMana` = 5: coherent (the bot taps
  before it declines, and always answers rather than passing).
- **`crates/simulator/Cargo.toml` dev-dependency** — **acceptable, no cycle, no build hazard.**
  `mtg-engine`'s `test-util` feature is `[]`, purely additive gating on
  `#[cfg(any(test, feature = "test-util"))]` accessor blocks. Dev-dependencies are not compiled for
  `cargo build`/`cargo build --release --bin mtg-fuzzer`, so the library and fuzzer builds are
  untouched; under `cargo test --workspace` the engine was already built with `test-util` via its
  own self-dev-dependency, so feature unification changes nothing. Exact precedent
  (`crates/engine/Cargo.toml:21`). Logged only as LOW E12 for the weaker test shape it enables.
- **`bare_lookup_ratchet.rs`** — **the `combat.rs` 16→15 lowering is justified** (I re-derived it:
  the two copy-pasted `state.objects.get(&r.source)` reads in the goad and `MustAttackEachCombat`
  blocks collapse into one inside the new helper; the `handle_declare_attackers` CantAttackOwner
  read at `:130-138` is untouched). **The `engine.rs` 22→24 raise is not justified** — see E2. The
  file has precedent for raising with justification (PB-EF2 100→105, PB-EF3 105→107, PB-OS11 7→8),
  but its own module doc says "**A count may only decrease**", and in this case both new sites had
  non-bare equivalents that this very PB uses ten files away.

### Ruling 8 — SR-4 classification of Change 2c: **CORRECT classification, wrong rationale**

Engine-bug side is right: `card_info` proves the card is in a graveyard ~35 lines above with no
intervening zone change, and zones are never removed. But the justifying comment claims a
propagated `Err` would deadlock via `handle_all_passed → handle_pass_priority → process_command`.
`force_resolve_overdue_payments` swallows every `Err` into a `debug_assert!(false, …)` and never
propagates, so that mechanism cannot obtain. LOW E3.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **MEDIUM** | `rules/combat.rs:228-239` | **Hybrid/Phyrexian/X attack-tax rejection is unscoped to the declared attack.** It fires for any live `CantAttackYouUnlessPay` source anywhere, including when nobody attacks that defender and including `attackers: vec![]`. **Fix:** move the pip check into the per-defender total loop at `:265-269` (or gate it on `attackers_per_player.contains_key(&restriction.controller)`), so only a defender actually being attacked can reject. |
| E2 | **MEDIUM** | `tests/core/bare_lookup_ratchet.rs:155` | **`engine.rs` ratchet ceiling raised 22→24 where the diagnostics vocabulary was available.** **Fix:** rewrite `engine.rs:874-878` as `state.player(player)?.life_total` (the `state.object(id)?` primitive-accessor idiom, which the needle does not match) and `engine.rs:1893-1897` as `state.expect_player(active).map(\|p\| !p.has_lost && !p.has_conceded).unwrap_or(false)` (verbatim the shape this PB's own `combat.rs:756-759` uses), then restore the ceiling to `("src/rules/engine.rs", 22)`. |
| E3 | LOW | `rules/engine.rs:1109-1113` | **Change 2c's justification comment describes an impossible mechanism.** The sweep swallows `Err` into a `debug_assert!`; nothing propagates. **Fix:** reword to the real consequence — a propagated `Err` would leave the entry removed and the card un-exiled, a silent rules failure — and drop the deadlock claim. |
| E4 | LOW | `rules/engine.rs:762-766`, `:996-1000` | **"Removing the write is a behaviour no-op" is overstated for echo/CU.** It is an identity write only if no priority-granting action intervened between resolution and the `Pay*` command. Post-PB-DP1, a *responder's* spell grants priority to the responder, and the deleted bodge would have yanked it to the active player. **Fix:** soften to "an identity write in the common case, and a fix in the same way as recover whenever a responder acted first". |
| E5 | LOW | `rules/combat.rs:278-284` | **Attack-tax insufficiency message is over-broad and asserts an implementation detail.** The restricted-mana sentence prints on *every* failure, including the ordinary not-enough-mana case, and "no `ManaRestriction` in this engine matches a non-spell cost" silently becomes false the day a non-spell-scoped variant is added (conventions.md's aspirational-comment hazard, in a user-facing string). `{:?}` of `ManaCost` is also not readable as `{2}`. **Fix:** state the total and the shortfall; move the CR 106.6 rationale into a code comment. |
| E6 | LOW | `rules/combat.rs:633-643` | **`GameEvent::ManaCostPaid` is pushed outside the `if let Some(ps)`,** so a missing player yields a payment event with no payment (Architecture Invariant 4 inverted). **Fix:** push the event inside the `if let`, or use `?`. |
| E7 | LOW | `rules/combat.rs:222-243`, `:260` | **A `cost_per_creature` of `ManaCost::default()` (`{0}`) still enters `taxed_defenders`,** so a must-attack/goaded creature is wrongly *not* forced to attack a defender whose "tax" is free (CR 118.5: 0 is always payable). Latent. **Fix:** skip `{0}` restrictions when building `tax_per_creature`. |
| E8 | LOW | `rules/combat.rs:769-775` | **Planeswalker fallback runs a full battlefield `expect_characteristics` scan per forced/goaded creature** whenever every live opponent is taxed — the common two-player-plus-Propaganda case, once per goaded creature per declaration. Perf only. **Fix:** hoist the "any opponent planeswalker exists" scan out of the helper and compute it once per `handle_declare_attackers`. |
| E9 | LOW | `rules/engine.rs:1162-1167`, `state/mod.rs:257-259`, `resolution.rs:2805-2810` | **The doc says "end of the priority round in which the ability resolved"; the actual boundary is the first subsequent round that ends with an empty stack.** A player can postpone the sacrifice indefinitely by keeping the stack non-empty. **Fix:** correct the wording and note the stalling vector. |
| E10 | LOW | `rules/replacement.rs:2179-2183` | **`ActiveRestriction.controller` is captured at ETB and never recomputed,** so a Propaganda whose control changes taxes attacks on the wrong player. Pre-existing (the affordability rejection already used it) but PB-DP4 makes it charge real mana. **Fix:** out of scope — file as a seed (OOS-DP4-10) alongside the §9 list. |
| E11 | LOW | `rules/engine.rs:1879-1911` | **No machine guard on the extra-round loop outside Cleanup.** The termination argument is sound (I verified it), but `cleanup_sba_rounds`/`check_for_mandatory_loop` only guard the Cleanup step, and plan risk 4 names "an infinite priority round with no error" as the failure shape. **Fix:** either call `loop_detection::check_for_mandatory_loop` in the extra-round branch, or add a `debug_assert!` bounding consecutive sweep rounds per step. |
| E12 | LOW | `rules/engine.rs:728-748` (echo), `:963-982` (CU) | **A forced decline that hits `ZoneChangeAction::ChoiceRequired` consumes the pending entry, emits `ReplacementChoiceRequired`, and leaves the permanent in place** awaiting a `ChooseReplacement` command for which no `LegalAction` exists — so a bot or M11-local seat cannot answer. Exotic (needs 2+ applicable zone-change replacements on the echo/CU permanent) and pre-existing on the interactive path, but the sweep now reaches it automatically. **Fix:** file as a seed. |
| E13 | LOW | `rules/engine.rs:1196-1248` | **Grouping by player (APNAP) before kind reorders closeouts relative to the order the abilities actually resolved.** Since the CR would have decided each at its own resolution, insertion order across all three vectors is arguably more faithful than APNAP grouping. Deterministic either way, so this is a judgement note, not a defect. **Fix:** none required; record the reasoning in the audit row so the choice is not re-litigated. |

## Test Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| T1 | **MEDIUM** (fix-phase **HIGH** per `memory/conventions.md`) | `pb_dp4_…:1051-1115` | **`test_101_4_multiple_outstanding_payments_resolve_in_apnap_order` cannot discriminate APNAP.** p1 (first in APNAP) owes the *echo* (first kind swept) and p3 owes the *recover* (last kind), so kind-grouped iteration predicts the identical event order. The test would pass against a sweep with no `apnap_order` loop at all. **Fix:** invert — p3 owes the echo, p1 owes the recover — and assert `RecoverDeclined` precedes `CreatureDied`. |
| T2 | **MEDIUM** (fix-phase **HIGH** per `memory/conventions.md`) | `pb_dp4_…:1144`, `:1162-1165` | **The pass-set half of the OOS-DP1-1 probe is vacuous.** The test seeds `players_passed = OrdSet::new()` and then asserts `players_passed.is_empty()`; the deleted bodge also wrote `OrdSet::new()`, so restoring it keeps the test green. Only the `priority_holder` half discriminates. **Fix:** seed `players_passed` with `{p1}` (or `{p1, p2}`) and assert the exact set survives the `PayRecover`. |
| T3 | **MEDIUM** | `pb_dp4_…` (missing) | **No test pins the guard that makes the extra round terminate.** Plan risk 4: "the guard is `!payment_events.is_empty()`, not 'consumed > 0' — a CR 400.7 no-op decline consumes an entry and produces nothing, and must fall through to the advance. Getting this backwards produces an infinite priority round with no error." Probe 19 exercises a no-op *second* entry but the *first* produces events, so the fall-through path is never taken. **Fix:** add a test that seeds a pending echo entry whose permanent is already in a graveyard (or a recover entry whose card is in exile) and asserts that a single `pass_all` advances the step *and* drains the vector. |
| T4 | LOW | `pb_dp4_…:793-846`, `:1000-1046` | **Nothing asserts a dies-trigger from the forced sacrifice actually lands on the stack in the same step** (plan risk 3, CR 603.3). Probes 11/15 only pin `turn().step`. **Fix:** add a permanent with a dies-trigger and assert a stack object exists in the same `Upkeep` step after the sweep. |
| T5 | LOW | `pb_dp4_…:655-662` | **`test_508_1d_must_attack_creature_…`'s second half asserts only `is_err()`,** so it would pass on any unrelated rejection. **Fix:** assert the message contains `"attack tax"`. |
| T6 | LOW | `pb_dp4_…:1051`, `:1120` | **No full-chain test of a non-active controller's recover reaching the deadline.** Both non-active-controller tests seed `pending_recover_payments` directly; the guarantee they rest on (that `resolve_top_of_stack` gives a non-active owing player a window) is never exercised end-to-end. **Fix:** extend probe 13 to a 4-player state with the recover card owned by a non-active player. |
| T7 | LOW | `crates/simulator/src/legal_actions.rs:294-297` | **The provider's CU life gate does not short-circuit on `total == 0` the way `engine.rs:873` does,** so at a negative life total a `Life(0)` cost is accepted by the engine but withheld by the provider — a CR 119.4b divergence from the engine it claims to mirror. Conservative direction (no illegal command). The same file already documents this exact trap at `:1605`. **Fix:** guard on `amount * age_count == 0 \|\| life_total >= …`. |
| T8 | LOW | `memory/primitive-wip.md:311-313` | **Close-out guard/probe accounting is internally inconsistent:** "5 guards + 15 fail-before probes" = 20, contradicting the same section's "16 of 22 failed, 6 passed". Actual split is **16 probes + 6 guards** (the 5 named plus `test_119_4b_cumulative_upkeep_zero_life_cost_is_always_payable`). **Fix:** correct before the audit rows are written, since §10 will quote these numbers. |
| T9 | LOW | `resolution.rs:2831`, `:2943` | **"pause for player choice" survives** in two inline comments even though the plan's checklist item was to remove the pause framing. The doc blocks above them are correct, so this is cosmetic. **Fix:** reword to "queue the choice; the deadline is applied by `force_resolve_overdue_payments`". |

### Finding details worth expanding

#### E1 — unscoped hybrid/Phyrexian/X rejection

**Severity**: MEDIUM · **File**: `crates/engine/src/rules/combat.rs:228-239`
**CR**: 508.1c/508.1h; 107.4e/107.4f

The check sits in the loop over `state.restrictions`, before `attackers_per_player` is built. So a
single hybrid-tax `CantAttackYouUnlessPay` source anywhere on the battlefield rejects:

- a declaration that attacks only *other*, normally-taxed or untaxed defenders;
- a declaration that attacks only planeswalkers (which CR 508.1c exempts from the tax entirely);
- `DeclareAttackers { attackers: vec![] }`.

It is not a hard hang — the active player can `PassPriority` and CR 508.8's no-attacker skip
(`turn_structure.rs:50-51`) advances to `EndOfCombat` — but it means that while such a card is in
play the declare-attackers step becomes unusable and every `MustAttackEachCombat`/goad requirement
becomes silently unenforceable. Fully latent (both corpus restriction defs are pure `{2}` generic),
and the plan prescribed exactly this placement, so the runner is not at fault. Correct scoping is
one line: reject only when that defender appears in `attackers_per_player`. The test
(`test_107_4e_hybrid_attack_tax_is_rejected_not_paid_free`) declares against the hybrid-tax
defender, so it stays valid under the narrower rule.

#### E2 — the ratchet raise was avoidable

**Severity**: MEDIUM · **File**: `crates/engine/tests/core/bare_lookup_ratchet.rs:145-155`
**Invariant**: SR-25 (`docs/engine-invariants.md`), module doc: "A file's count may only ever go
*down*."

Both new sites had a non-counting equivalent that this same PB uses:

- `engine.rs:874-878` (`state.players.get(&player).ok_or(PlayerNotFound)?.life_total`) →
  `state.player(player)?.life_total`. `player()` is one of the primitive accessors the ratchet
  explicitly exempts *by construction* (it is not matched by the `.players.get(` needle), and the
  `state.object(id)?` form of it is used throughout `resolution.rs`.
- `engine.rs:1893-1897` (`state.players.get(&active).map(…).unwrap_or(false)`) →
  `state.expect_player(active).map(…).unwrap_or(false)`, which is byte-for-byte the shape the new
  `combat.rs:756-759` uses in the same commit.

The runner's justification (both match `enter_step`'s existing residue) is honest and the sites are
genuinely NONSWALLOW/propagating, so this is not a silent-failure regression. But the gate exists
to ratchet the vocabulary in, and this PB demonstrably knew the vocabulary. Converting both and
restoring `22` is a two-line change.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 508.1c (restriction, planeswalker exemption) | Yes | Yes | `test_508_1c_planeswalker_attack_is_not_taxed` (guard) |
| 508.1d (never required to pay) | Yes | Yes | probes 7, 8 + guards 9, 10; closes OOS-RS3-4 |
| 508.1f→508.1j ordering | Yes | Partly | debit is after both tapping loops; ordering not directly asserted (event order in probe 1 would pin it) |
| 508.1h (total cost, colour, cumulative) | Yes | Yes | `…colour_is_not_flattened…`, `…sums_per_defender_and_per_attacker` |
| 508.1i (mana-ability window before payment) | **No** — pre-existing deviation | n/a | documented in-code + OOS-DP4-2 |
| 508.1j (real payment, no partial) | Yes | Yes | probes 1, 2b, 4 + strengthened `restrictions.rs` test |
| 106.6 (restricted mana) | Yes | Yes | `test_106_6_restricted_mana_cannot_pay_an_attack_tax`; behaviour flip, correct |
| 107.4e/107.4f | Reject, not pay | Yes | `test_107_4e_hybrid_…`; scoping defect E1 |
| 118.12a (didn't answer ≡ didn't pay) | Yes | Yes | probes 11, 12, 13 |
| 119.4 / 119.4b | Yes | Yes | probes 18, 18b + provider gate (divergence T7) |
| 608.2d | **Deliberate deviation**, documented 3× | n/a | boundary instead of resolution |
| 608.2g (mana ability before paying) | Yes | Yes | probe 20 (guard) |
| 702.24a | Yes | Yes | probe 12 |
| 702.24b (all age counters; multiple instances) | Yes | Yes | probe 19 + provider test |
| 702.30a | Yes | Yes | probes 11, 14 |
| 702.59a | Yes | Yes | probe 13 |
| 400.7 (no-op decline) | Yes | Partly | probe 19's second entry only; **all-no-op fall-through untested — T3** |
| 101.4 (APNAP) | Yes | **Not discriminating — T1** | see T1 |
| 117.3c / 117.4 (OOS-DP1-1) | Yes | Partly (**T2**) | priority half pinned, pass-set half vacuous |
| 603.3 (dies-triggers in this step) | Yes | Partly (**T4**) | step pinned, stack contents not |
| 500.4 (pool emptying) | Preserved | No | sweep returns before `empty_all_mana_pools`; pools still empty exactly once per step |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `propaganda.rs` | Yes | 0 | **Yes (was wrong)** | Oracle + all 4 rulings verified via MCP; `{2}` generic, per-creature, cumulative. 0 edits, now actually charged. |
| `ghostly_prison.rs` | Yes | 0 | **Yes (was wrong)** | Identical oracle text to Propaganda; 0 edits. |
| `mogg_war_marshal.rs` | Yes | 0 | **Yes (was wrong)** | `Echo {1}{R}`, `Complete`; unpaid echo now sacrifices. Not edited (correct). |
| `avalanche_riders.rs` | Yes (not re-read; roster-confirmed) | 0 | **Yes (was wrong)** | Same class as Mogg War Marshal. |
| `grim_harvest.rs` | Yes | 0 | **Yes (was wrong)** | `Recover {2}{B}`, `Complete`; unpaid recover now exiles. Not edited (correct). |
| `mystic_remora.rs` | n/a | 0 | Marker unchanged (`known_wrong`, `MayPayOrElse`/DP-12) | Its note's "cumulative upkeep {1} … are correct" clause becomes true with 0 edits, as predicted. |
| `tombstone_stairwell.rs` | n/a | 0 | CU now correct; marker stays `partial` | Unrelated token-provenance blockers survive. |
| `goblin_rabblemaster.rs` | Yes | 0 | Yes | **Only card-def edit in the PB**, comment-only: the accepted-limitation paragraph replaced by a one-line "OOS-RS3-4 CLOSED by PB-DP4, CR 508.1d" note. Accurate; oracle + 2014-07-18 ruling re-verified via MCP. |

## Constraint Verification

| Constraint | Status | Evidence |
|---|---|---|
| No new `Command` / `GameEvent` / `GameState` variant or field | ✅ | `ManaCostPaid`, the three `Pay*` commands and the three `*PaymentRequired` events all pre-existed and are reused verbatim; the three `pending_*` fields keep their shapes |
| `PROTOCOL_VERSION == 27` | ✅ | `rules/protocol.rs:260`; nothing in the diff touches the `Command`/`GameEvent` type closure, so nothing *should* have moved it |
| `HASH_SCHEMA_VERSION == 63` | ✅ | `state/hash.rs:578`; no serialized field/variant shape changed. Values change (behaviour flip) but the convention bumps on *shape*, so holding at 63 is correct |
| SR-3 seal on `state/mod.rs` not widened | ✅ | `pub(crate)` on all three fields; only doc comments edited; the sweep uses the private fields from inside the crate exactly as the handlers do; no `_mut()` hatch used in engine code |
| `KeywordAbility::Echo/CumulativeUpkeep/Recover` not named in executable code in the sweep or the provider | ✅ | Both read the pending vectors; only doc comments name the keywords (stripped by the site-scan) |
| SR-4: new silent-failure sites pick a side | ✅ | Change 2c → `expect_move_object_to_zone` (engine-bug side, correct); sweep `Err` → `debug_assert!` engine-bug. Rationale wording is wrong (E3), classification is right |
| SR-9a: test registered | ✅ | `tests/primitives/main.rs:24` |
| W3-LC / CR 613.1f: layer-resolved reads | ✅ | `combat.rs:772` `expect_characteristics`, not `obj.characteristics` |
| SR-38: provider offers only what the engine accepts | ✅ (one conservative divergence, T7) | `can_pay_cost` mirrored; `multiply_mana_cost` is an exact copy of `engine.rs:1009-1033` incl. hybrid/phyrexian/x_count |
| SR-9b determinism | ✅ | `BTreeMap`/`BTreeSet`, `apnap_order` rotation, `imbl::Vector` insertion order, order-independent `.any()` |
| No new build hazard / dependency cycle | ✅ | `test-util = []`; dev-dep only; precedent `crates/engine/Cargo.toml:21` |

## Recommended Fix Order

1. **T1**, **T2** (test-validity → fix-phase HIGH per `memory/conventions.md`) — both are ≤5-line
   edits to existing tests.
2. **T3** — add the all-no-op fall-through probe; it is the only untested guard on the design's
   one catastrophic failure mode.
3. **E1** — scope the hybrid rejection to attacked defenders.
4. **E2** — convert the two `engine.rs` lookups and restore the ratchet ceiling to 22.
5. **E3, E4, E5, E9, T7, T8, T9** — wording/accuracy sweep, one pass.
6. **E6, E7, T5, T6, T4** — small correctness/test hardening, opportunistic.
7. **E8, E10, E11, E12, E13** — file as seeds (E10 and E12 as new `OOS-DP4-10`/`OOS-DP4-11` rows
   alongside §9's nine); do not expand scope.

---

## Fix cycle (runner)

Applied 2026-07-26. Every finding below is dispositioned; none were silently skipped. All
five MEDIUMs are **fixed**. Note: this verdict's own banner says "13 LOW", but the Engine
Change Findings + Test Findings tables above actually list **17** LOW rows (E3–E13 = 11,
T4–T9 = 6) — all 17 are dispositioned below, not just 13. Of the 17: **9 fixed cleanly**
(E3, E4, E5, E6, E7, T5, T7, T8, T9), **1 fixed and also folded into a new seed** (E9), **6
declined with a stated reason** (E8, E11, T4, T6, plus E10/E12 which are declined-and-folded
into new seeds), and **1 needs no fix** (E13, per this review's own "Fix: none required").
Gates after the cycle: `cargo test
--all` 3,781 passed / 0 failed (3,777 at implement close + 4 new tests: T3's fall-through
probe, E1's two scoping tests, T7's provider parity test — T1/T2/T5 strengthened existing
tests in place, no count change), `cargo clippy --workspace --all-targets -- -D warnings`
clean, `cargo build --workspace` clean, `cargo fmt --check` clean, `tools/check-defs-fmt.sh`
clean (1,804 defs). `PROTOCOL_VERSION == 27` (`rules/protocol.rs:260`), `HASH_SCHEMA_VERSION
== 63` (`state/hash.rs:578`) — both read directly from source post-fix, unmoved. No new
`Command`/`GameEvent`/`GameState` variant or field. SR-3 seal not widened.

### MEDIUM findings

**T1 — APNAP test could not discriminate. FIXED.**
`test_101_4_multiple_outstanding_payments_resolve_in_apnap_order`
(`crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs`) inverted: p3
(not p1) now owes the echo, p1 (not p3) now owes the recover, and the assertion checks
`RecoverDeclined` index `<` `CreatureDied` index (the reviewer's prescribed discriminating
direction). **Evidence the strengthened test discriminates**: temporarily rewrote
`force_resolve_overdue_payments` (`rules/engine.rs`) into three separate `for owing in
apnap_order` loops — one for echoes, one for cumulative upkeeps, one for recovers (i.e.
"kind-grouped-globally" instead of "per-player-APNAP-outer-loop") — swallowing handler
errors with `Err(_) => {}` to keep it compiling standalone for the experiment. Ran only this
one test against that patched engine: **it failed**
(`crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs:1270`, the
`recover_declined_idx < creature_died_idx` assertion). Restored `rules/engine.rs` from a
pre-experiment backup (`diff` confirmed byte-identical to the fix-cycle version), reran the
full `pb_dp4` suite (25/25 green) and the full workspace suite (3,781/3,781 green) to confirm
the revert was clean. The test now provably distinguishes the real implementation from the
hypothesis it exists to rule out.

**T2 — OOS-DP1-1 probe's pass-set assertion was vacuous. FIXED.**
`test_dp11_answering_a_payment_does_not_reassign_priority` now seeds
`players_passed = imbl::OrdSet::unit(p1)` (was `OrdSet::new()`) and asserts
`state.turn().players_passed == imbl::OrdSet::unit(p1)` (was `.is_empty()`) after the
`PayRecover` call. The deleted bodge wrote `players_passed = OrdSet::new()` unconditionally,
which is indistinguishable from an already-empty set — seeding a non-empty set and asserting
it survives unchanged is the only form that would have caught the bodge's reintroduction.

**T3 — no test pinned the all-no-op fall-through guard. FIXED.**
Added `test_dp11_all_no_op_sweep_falls_through_and_advances`: seeds a pending echo entry
whose permanent is in the graveyard (never on the battlefield at all), so
`handle_pay_echo`'s `source_info` guard short-circuits to `Ok(vec![])` — the entry is
consumed but produces zero events. Asserts the pending vector empties AND the step advances
from `Upkeep` to `Draw` in a single `pass_all` (not staying in `Upkeep` for another round).
This is the first probe in the file where the sweep produces no events at all; every other
probe (including #19/`test_702_24b_...`) has at least one entry that DOES produce events
riding along with it.

**E1 — hybrid/Phyrexian/X rejection unscoped to the declared attack. FIXED.**
`rules/combat.rs`: the rejection no longer fires inside the `state.restrictions` scan. It now
records defenders with an unflattenable tax into a new `unpayable_tax_defenders:
BTreeSet<PlayerId>` and only returns `Err` when building `attackers_per_player` — i.e. only
if a declared attacker's `AttackTarget::Player` actually names one of those defenders. Added
two tests: `test_107_4e_hybrid_tax_does_not_block_attacks_on_other_defenders` (4-player,
attack p3 while p1 carries an unpayable Propaganda-style restriction — must succeed) and
`test_107_4e_hybrid_tax_does_not_block_an_empty_declaration` (same restriction, `attackers:
vec![]` — must succeed). The pre-existing
`test_107_4e_hybrid_attack_tax_is_rejected_not_paid_free` (which DOES attack the unpayably-
taxed defender) stays valid and green under the narrower rule, confirming the rejection still
fires when it should.

**E2 — bare-lookup ratchet ceiling raised avoidably. FIXED.**
`rules/engine.rs`: converted `state.players.get(&player).ok_or(GameStateError::
PlayerNotFound(player))?.life_total` → `state.player(player)?.life_total` (the CR 119.4 life
gate), and `state.players.get(&active).map(|p| !p.has_lost && !p.has_conceded)
.unwrap_or(false)` → `state.expect_player(active).map(...)` (the boundary-sweep priority
hook). `crates/engine/tests/core/bare_lookup_ratchet.rs`: `SWEPT_FILES` entry for
`src/rules/engine.rs` restored from `24` to `22`, changelog comment rewritten to record both
the raise and the fix-cycle reversal. `cargo test -p mtg-engine --test core bare_lookup`
passes (3/3, including the pinned-count assertion) at the restored ceiling.

### LOW findings

| # | Disposition | Detail |
|---|---|---|
| E3 | **Fixed** | `engine.rs`'s Change 2c comment (recover decline, `expect_move_object_to_zone`) reworded: dropped the false "would deadlock via handle_all_passed → handle_pass_priority → process_command" claim (impossible — `force_resolve_overdue_payments` swallows every handler `Err` into a `debug_assert!` and never propagates it) and replaced it with the real consequence — a release-build `None` here would silently abandon the exile, leaving the card un-exiled in the graveyard with the pending entry already gone (a silent rules failure, not a deadlock). |
| E4 | **Fixed** | Both echo and cumulative-upkeep Change 2d comments (`engine.rs`, the two `priority_holder` deletion sites) softened from "removing the write is a behaviour no-op" to "an identity write in the common case where no priority-granting action has intervened since resolution", with the post-PB-DP1 responder-priority scenario spelled out (same class of fix as recover's, just less commonly triggered since echo/CU only fire on the controller's own upkeep). |
| E5 | **Fixed** | `combat.rs`'s attack-tax insufficiency message no longer prints "no ManaRestriction in this engine matches a non-spell cost" (an internal implementation-detail assertion that would silently go stale) on every failure. The CR 106.6 rationale moved into a code comment at the rejection site. The message now states the required `ManaCost` (`{:?}`, preserving colour) and the available unrestricted total, without asserting a specific cause (quantity vs. colour mismatch), since the same total-vs-total case can be either. The `test_106_6_restricted_mana_cannot_pay_an_attack_tax` assertion was updated to match (now checks for `"0 unrestricted mana"` instead of `"106.6"` in the message — the CR cite deliberately no longer appears in the player-facing string). |
| E6 | **Fixed** | `combat.rs`: `events.push(GameEvent::ManaCostPaid { .. })` moved inside the `if let Some(ps) = state.expect_player_mut(player)` block, so a missing player (should be unreachable, but the ratchet's own convention is defense-in-depth) can no longer produce a payment event describing a payment that didn't happen. |
| E7 | **Fixed** | `combat.rs`: a `{0}` (all-fields-zero) `CantAttackYouUnlessPay` restriction is now skipped entirely when building `tax_per_creature` (CR 118.5: 0 is always payable), so it no longer marks its defender as taxed. Folded into the same block as the E1 restructure since both touch the restriction-scan loop; no new test added (the existing corpus has no `{0}` restriction and the plan already characterized this as latent — the fix closes the gap without adding surface area for something no card currently exercises). |
| E8 | **Declined, with reason** | Perf-only (planeswalker fallback scan per forced/goaded creature per declaration). No correctness impact. Fixing it (hoisting the "any opponent planeswalker" scan out of `has_uncosted_attack_target`) is a reasonable future optimization but is out of scope for a fix cycle whose mandate is correctness findings; the reviewer's own "Recommended Fix Order" placed it in the "file as seed" bucket, not the fix bucket. No new seed drafted — already fully described in the review's own E8 entry, which the coordinator can file verbatim. |
| E9 | **Fixed (wording) + folded into seed OOS-DP4-12** | Corrected "the end of the priority round in which the ability resolved" → "the first subsequent priority round that terminates with an empty stack" everywhere the boundary is described: `force_resolve_overdue_payments`'s doc comment and the inline comment in `handle_all_passed` (`engine.rs`), all three producer comment blocks (echo/CU/recover trigger resolution, `resolution.rs`), and all three `pending_*_payments` field doc comments (`state/mod.rs`). Each site now also states the postponability consequence and cites a new seed, drafted below as **OOS-DP4-12**, for the coordinator to file (per instruction, not filed directly into `docs/audits/decision-point-audit.md` by this runner). |
| E10 | **Declined, out of scope; folded into seed OOS-DP4-10** | `replacement.rs:2179-2183`'s `ActiveRestriction.controller` capture-at-ETB is pre-existing PB-18 infrastructure, not something PB-DP4 introduced or touched beyond reading it. Recomputing it on every attack-tax check would mean re-deriving "current controller of the restriction's source" per restriction per declaration — a real design surface (does it read layer-resolved control-change effects? what if the source itself changed control mid-restriction-lifetime?) that deserves its own plan, not a fix-cycle patch. Drafted as **OOS-DP4-10** below. |
| E11 | **Declined, with reason** | The reviewer's own text confirms the termination argument is independently sound ("I verified it"); this finding asks for an additional *machine* guard (a `debug_assert!` bound or a `loop_detection::check_for_mandatory_loop` call) on top of an already-correct invariant. `check_for_mandatory_loop`'s existing call sites are scoped to specific board-state-recurrence detection (CR 104.4b) and repurposing it for a round-count bound is a different mechanism, not a call-site addition. A round-counter approach would need either a new `GameState` field (forbidden by this PB's hard constraints) or reuse of `turn.cleanup_sba_rounds`, which is Cleanup-step-scoped by name and by the code that resets it — repurposing it outside Cleanup would conflate two different bounded-loop invariants under one counter, a correctness-adjacent change disproportionate to a LOW in a fix cycle that is supposed to keep changes minimal. Not drafted as a new seed since the review's own E11 entry already describes the gap precisely; the coordinator can file it as-is if desired. |
| E12 | **Declined, out of scope; folded into seed OOS-DP4-11** | A forced decline that hits `ZoneChangeAction::ChoiceRequired` (`engine.rs:728-748` echo, `:971-985` CU — line numbers shifted slightly post-fix-cycle edits) needing 2+ applicable zone-change replacements on the same echo/CU permanent is exotic and pre-existing on the interactive path; PB-DP4's sweep reaching it automatically is a real widening of exposure but fixing it (e.g. giving `LegalAction` coverage to `ChooseReplacement` for a forced-decline-originated choice) is a multi-file interactive-choice design question, not a fix-cycle patch. Drafted as **OOS-DP4-11** below. |
| T4 | **Declined, with reason** | Adding a dies-trigger-lands-on-the-stack-in-the-same-step test for the forced-sacrifice path is legitimate opportunistic hardening (plan risk 3, CR 603.3), but the mechanism it would exercise (`check_and_flush_triggers(state, &mut payment_events)`, called unconditionally on the sweep's output at `handle_all_passed:1917` per the current line numbers) is the same trigger-check-and-flush helper exercised by dozens of existing dies-trigger tests elsewhere in the suite (the "Command Handler Pattern Gotchas" convention in `memory/gotchas-infra.md`), and the review's own Ruling 2 already traced this call site by reading the code, not by inference. Building a dedicated dies-trigger card def and a full-chain test for this one composition is a reasonable next increment but was deprioritized in this cycle in favor of the 5 mandatory MEDIUMs and the other 12 LOWs, several of which (E1, E2) had test-validity or gate implications this one does not. |
| T5 | **Fixed** | `test_508_1d_must_attack_creature_is_not_forced_to_pay_an_attack_tax`'s second half now captures the `Err`, formats it, and asserts the message contains `"attack tax"` — it would previously have passed against any unrelated rejection reason. |
| T6 | **Declined, with reason** | Extending probe 13 (`test_702_59a_unanswered_recover_card_is_exiled_at_the_round_boundary`) into a full 4-player trigger-production chain (cast a creature, let it die, let the recover trigger produce the pending entry itself, THEN cross the boundary) for a non-active recover controller is real additional coverage, but the underlying guarantee (a non-active player is guaranteed a priority window before the deadline) is independently verified two ways already: by direct code reading in the review's own Ruling 1(b)/(d) (`resolve_top_of_stack` unconditionally re-grants priority regardless of who is active), and by `test_dp11_answering_a_payment_does_not_reassign_priority`, which exercises a non-active-controller (`p3`) recover payment end-to-end through `process_command` (just seeded rather than trigger-produced). Building the full chain is a reasonable next hardening step, deprioritized here for the same reason as T4. |
| T7 | **Fixed** | `crates/simulator/src/legal_actions.rs`: the `CumulativeUpkeepCost::Life` provider arm now short-circuits `total == 0` before comparing against `life_total`, matching `engine.rs`'s `if total_life > 0 { check }` guard exactly (which never runs the affordability check at all for a zero-cost payment). Added `provider_offers_cumulative_upkeep_zero_life_cost_even_at_negative_life_total`, which pins the previously-withheld case (`life_total = -3`, `Life(0)` cost) now offering `pay: true`. |
| T8 | **Fixed** | `memory/primitive-wip.md`'s guard/probe accounting corrected in place: "5 guards + 15 fail-before probes" (= 20, inconsistent with the same section's "16 of 22 failed, 6 passed") → "6 guards + 16 fail-before probes" (= 22, consistent), with `test_119_4b_cumulative_upkeep_zero_life_cost_is_always_payable` added to the named guard list (it passed pre-fix because the pre-fix `Life` arm had no affordability check at all, so a zero-cost payment already succeeded unconditionally — nothing for Change 2e to fix in that specific case). |
| T9 | **Fixed** | Both `resolution.rs` "Emit the payment required event and pause for player choice." comments (echo and recover trigger-resolution sites) reworded to "Queue the choice; the deadline is applied by `rules/engine.rs::force_resolve_overdue_payments`" with a one-line note that nothing here actually pauses anything. |
| E13 | **No fix required — judgement stands** | The review's own verdict already classifies this as "a judgement note, not a defect" with an explicit "Fix: none required; record the reasoning in the audit row so the choice is not re-litigated" — that recording is audit-bookkeeping (§10 close-out), not a fix-cycle code or test change, and is left to the coordinator's close-out pass per the same division of labor as the OOS-DP4-1..9 seeds. |

### New seed text drafted for the coordinator to file (not written into `docs/audits/decision-point-audit.md` by this runner)

**OOS-DP4-10** (folds E10): **`ActiveRestriction.controller` is captured at ETB and never
recomputed for `CantAttackYouUnlessPay`.** `rules/replacement.rs:2179-2183` sets `controller`
once, at the point the ability registers its restriction. If control of the restriction's
source (e.g. Propaganda) changes hands afterward, the attack tax continues to apply against
the *original* controller rather than the current one. Pre-existing since PB-18 (the
affordability-only check already read this field); PB-DP4 makes it charge real mana, raising
the stakes of a wrong answer from "an incorrect rejection" to "an incorrect debit or an
incorrectly-skipped debit." Fix needs either a per-check recompute of current control (reading
`state.objects.get(&source).map(|o| o.controller)` at declare-attackers time instead of the
stored field) or a broader audit of whether other `ActiveRestriction` variants have the same
staleness. No wire change. | correctness, narrow, pre-existing | to be filed by the
coordinator against PB-DP4 (`scutemob-152`) fix cycle.

**OOS-DP4-11** (folds E12, per the review's own "Recommended Fix Order" numbering; confirmed
free, no collision with existing OOS-DP4-1..9): **A forced decline can strand the
game on an unreachable `ChooseReplacement` wait.** If an echo or cumulative-upkeep
permanent has 2+ applicable zone-change replacement effects when `force_resolve_overdue_
payments` (or a direct `Pay*{pay:false}`) declines it, `handle_pay_echo` /
`handle_pay_cumulative_upkeep`'s `ZoneChangeAction::ChoiceRequired` arm (`engine.rs:728-748`
echo, `:971-985` CU) pushes a `PendingZoneChange` and emits `ReplacementChoiceRequired`,
but no `LegalAction` exists for `Command::ChooseReplacement` in `crates/simulator`, so a bot
or an M11-local seat can never answer it — the permanent sits in limbo indefinitely. Exotic
(needs 2+ registered zone-change replacements on the same permanent) and pre-existing on the
manual `Command::PayEcho{pay:false}` path; PB-DP4's automatic boundary sweep reaches it
without any player action, widening exposure. Fix needs `LegalAction` coverage for
`ChooseReplacement`, a simulator-only change (no wire impact). | correctness / M11-local
gap, exotic | to be filed by the coordinator against PB-DP4 (`scutemob-152`) fix cycle.

**OOS-DP4-12** (new, from E9): **The DP-11 deadline can be postponed indefinitely by keeping
the stack non-empty.** `force_resolve_overdue_payments` fires only in `handle_all_passed`'s
stack-EMPTY branch. If any player (not necessarily the one who owes the payment) casts a
spell or activates a non-mana ability before the round would otherwise end, the stack is
non-empty and the deadline does not fire that round. This can repeat: as long as *something*
keeps landing on the stack before all players pass with it empty, the pending payment's
permanent/card survives in its pre-consequence state, observably tappable, sacrificable to
another cost, targetable, or (for recover) millable/exile-able by something else, and
re-triggerable. Bounded only by the postponing player's resources (mana, cards, or willingness
to keep the stack busy), not by a fixed number of rounds. The eventual outcome at whatever
boundary is finally reached is still CR-correct (CR 118.12a's "didn't answer" default);
only the *timing* is looser than "one extra round" might suggest, and this is documented
precisely now in both the engine doc comments (`engine.rs`, `resolution.rs`, `state/mod.rs`)
and this review. No fix prescribed — this is the CR 608.2d deviation's own shape, already
accepted as the smallest available deviation under the no-new-`Command` constraint (see
Ruling 1 above); a fix would need either a new `Command` (a stack-empty check independent of
priority) or a more aggressive deadline design, both out of scope. | design-deviation
consequence, documented | to be filed by the coordinator against PB-DP4 (`scutemob-152`) fix
cycle.
