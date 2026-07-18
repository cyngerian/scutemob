# Primitive Batch Review: PB-EF2 — `TokenSpec.recipient` player-scoped CreateToken recipient

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: scutemob-102
**CR Rules**: 111.1, 608.2h, 614.1, 701.5 / 701.5g; An Offer ruling 2022-04-29; Swan Song ruling 2013-09-15
**Engine files reviewed**: `crates/card-types/src/cards/card_definition.rs` (PlayerTarget +2 variants, TokenSpec.recipient, Defaults), `crates/engine/src/effects/mod.rs` (EffectContext.countered_spell_controller, CounterSpell capture, CreateToken executor rewrite, resolve_player_target_list + Manifest/Cloak arms), `crates/engine/src/state/hash.rs` (HashInto + version 45 + history), `crates/engine/src/rules/protocol.rs` (PROTOCOL_VERSION 7 + fingerprint + history), `crates/engine/tests/core/bare_lookup_ratchet.rs` (ceiling 100→105)
**Card defs reviewed**: `swan_song.rs` (flip to Complete), `an_offer_you_cant_refuse.rs` (new, Complete) — 2 defs
**Tests reviewed**: `crates/engine/tests/primitives/pb_ef2_create_token_recipient.rs` (8 tests), `test-data/generated-scripts/tokens/001_swan_song_creates_bird.json` (un-retired), `test-data/generated-scripts/stack/045_swan_song_counters_damnation.json` (bug fix)

## Verdict: needs-fix

The PB-EF2 core deliverable — a per-recipient `Effect::CreateToken` executor keyed to
`TokenSpec.recipient`, with CR 614.1-correct per-recipient token doubling and the An Offer
uncounterable-but-legal capture — is **correct and well-tested**. Default `recipient =
Controller` resolves to `[ctx.controller]`, preserving pre-PB-EF2 behaviour byte-for-byte;
the CounterSpell capture is placed correctly before the `cant_be_countered` continue and
does not leak; version bumps (HASH 44→45, PROTOCOL 6→7) are consistent with zero stale
sentinels. One HIGH finding: `swan_song.rs` is flipped to **Complete** while still declaring
an over-broad target (`TargetSpell`, any spell) that contradicts its oracle ("enchantment,
instant, or sorcery spell"), so the engine would permit Swan Song to illegally counter a
creature/artifact/planeswalker spell. Plus one LOW (a cosmetic stale note in the fixed
stack/045 script).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| — | — | — | No engine-change findings. CreateToken rewrite, CounterSpell capture, resolver arms, hash/protocol bumps, and ratchet raise all verified correct against CR. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | `swan_song.rs` | **Over-broad target flipped to Complete.** `targets: vec![TargetRequirement::TargetSpell]` accepts any spell; oracle restricts to enchantment/instant/sorcery. Bare `TargetSpell` does only a zone==Stack check (`casting.rs:6108`), no type filter, so the engine permits Swan Song to counter a creature/artifact/etc. spell — wrong game state on a Complete card. **Fix:** below. |
| 2 | LOW | `stack/045_swan_song_counters_damnation.json` | **Stale note contradicts corrected assertion.** Line 196 note still reads "p2 (Swan Song's controller) creates a 2/2 blue Bird token" while `target.player` and the load-bearing `zones.battlefield.p1` assertion are now correctly p1. Note is cosmetic (unchecked). **Fix:** reword to "p1 (countered spell's controller) creates the Bird." |

### Finding Details

#### Finding 1: Swan Song ships Complete with an over-broad `TargetSpell`

**Severity**: HIGH
**File**: `crates/card-defs/src/defs/swan_song.rs:44`
**Oracle**: "Counter target enchantment, instant, or sorcery spell. Its controller creates a
2/2 blue Bird creature token with flying."
**Issue**: PB-EF2 correctly fixes the token recipient and deletes the `known_wrong` marker,
asserting the card is now Complete (produces correct game state in all lines). But the target
requirement is bare `TargetRequirement::TargetSpell`. Verified in
`crates/engine/src/rules/casting.rs:6107-6131`: bare `TargetSpell` validates only that the
object is in the stack zone — it applies **no** card-type filter. So the engine will accept
Swan Song targeting an opponent's creature (or artifact, planeswalker, battle) spell and
counter it, an illegal play the paper card cannot make. This is a legal-but-wrong game state,
the project's declared #1 pre-alpha risk, on a card being certified Complete.

This is independent of, and not fixed by, the recipient work. It is a pre-existing
target-breadth gap that the flip to Complete now exposes.

**Fix**: Restrict the target to enchantment/instant/sorcery. The `has_card_types` (OR-semantics)
filter field exists and is honored by `matches_filter` (`effects/mod.rs:8144-8151`), which the
`TargetSpellWithFilter` validation path calls (`casting.rs:6119-6128`). Change to:
```rust
targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
    has_card_types: vec![CardType::Enchantment, CardType::Instant, CardType::Sorcery],
    ..Default::default()
})],
```
Add a test that a creature spell is a rejected target (mirrors An Offer's `non_creature`
enforcement). An Offer's own `non_creature: true` filter is correct and honored — no change there.

#### Finding 2: stale note in the fixed stack/045 script

**Severity**: LOW
**File**: `test-data/generated-scripts/stack/045_swan_song_counters_damnation.json:196`
**Issue**: The script's `battlefield.p1 includes Bird` assertion (line 215) is the correct
fix. But the expected-events note at line 196 still says "p2 (Swan Song's controller) creates
a 2/2 blue Bird token," which contradicts both the corrected `target.player: p1` and the
assertion. The note is not evaluated by the harness, so this is cosmetic only.
**Fix**: Reword the note to attribute the Bird to p1 (the countered spell's controller).

## Focus-Area Verification (against brief)

1. **CreateToken executor rewrite** (`effects/mod.rs:675-733`) — CORRECT. Default
   `recipient = Controller` → `resolve_player_target_list` returns `[ctx.controller]` →
   single recipient, same `apply_token_creation_replacement`, same `make_token`, same
   `TokenCreated`/`PermanentEnteredBattlefield` events → byte-identical to pre-PB-EF2. No
   double-application: `resolve_amount` computes the base count once outside the loop, then
   replacement is applied once per recipient (CR 614.1: replacement is the recipient-player's,
   e.g. their own Doubling Season). `attack_target` is resolved once from `ctx.source`
   (recipient-independent, correct). `ctx.last_created_permanent` updated in-loop (last token
   wins; identical for the single-recipient default). `CreateTokenAndAttachSource` correctly
   left on `ctx.controller` with an explanatory comment (Living Weapon never sets a non-default
   recipient; create+attach atomicity is source-relative).

2. **CounterSpell capture** (`effects/mod.rs:1960-1971`) — CORRECT per both An Offer rulings.
   `ctx.countered_spell_controller = Some(state.stack_objects[pos].controller)` is set the
   moment a valid target `pos` is found, **before** the `cant_be_countered` continue → an
   uncounterable-but-legal target's controller still gets tokens (ruling 2022-04-29 #1). If the
   target is not found on the stack (`pos == None`), the field stays `None` → empty recipients
   → no tokens (ruling 2022-04-29 #2 / whole-spell fizzle). No stale-leak risk: the field is
   read only by `PlayerTarget::ControllerOfCounteredSpell`; a later default-recipient
   CreateToken in the same Sequence ignores it. `EffectContext` is fresh per resolution, so no
   cross-spell leak.

3. **New PlayerTarget variants resolution** — CONSISTENT across all sites.
   `resolve_player_target_list` (6474-6495): `ControllerOfCounteredSpell` filters `has_lost`
   and returns `[]` when `None` (fizzle → no tokens, correct); `ControllerOfTriggeringObject`
   resolves triggering creature's controller → `triggering_player` → `ctx.controller`, with a
   `has_lost` guard. Manifest (3654-3661) and Cloak (3714-3721) single-player match arms mirror
   the same logic with `.unwrap_or(ctx.controller)` (harmless — no card sets these recipients
   on Manifest/Cloak; arms exist only for exhaustiveness).

4. **Card defs vs oracle** — An Offer: CORRECT (counters noncreature spell via
   `TargetSpellWithFilter { non_creature: true }`, honored; two Treasures to
   `ControllerOfCounteredSpell`; Treasure mana ability preserved). Swan Song: recipient
   CORRECT, but target over-broad — Finding 1 (HIGH).

5. **Tests** (`pb_ef2_create_token_recipient.rs`) — NON-VACUOUS, execution-probed. Real spells
   cast/resolved through `process_command` or `execute_effect` on production entry points;
   board state read back. Each decoy fails if recipient defaults to Controller (runner also
   manually verified all 7 recipient-sensitive tests fail under a hardcoded
   `vec![ctx.controller]` revert). Doubling tests are real: `register_permanent_replacement_abilities`
   on an actual Doubling Season, with a reverse decoy (doubler on caller's side ⇒ recipient
   still gets exactly 1) proving the replacement is keyed to recipient, not `ctx.controller`.

6. **Version bumps** — CONSISTENT. HASH_SCHEMA_VERSION 45 (`hash.rs:414`), history epoch 45
   appended (`:539-548`) + `- 45:` History line (`:406-413`); `TokenSpec.recipient` hashed
   last (`:5028`); PlayerTarget discriminants 8/9 fresh (`:5079-5081`). PROTOCOL_VERSION 7
   (`protocol.rs:99`), fingerprint `c5931e61…` + v7 history row (`:203-207`) + `- 7:` line.
   No stale sentinels: grep for `HASH_SCHEMA_VERSION, 4[0-4]` / `PROTOCOL_VERSION, [0-6]`
   returns only doc/plan hits, zero source hits.

7. **stack/045 script fix** — CORRECT (Bird asserted on `battlefield.p1`, the countered
   spell's controller). Damnation is a sorcery, a legal Swan Song target. One cosmetic stale
   note remains — Finding 2 (LOW).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 111.1 / 608.2h (which player creates/controls token) | Yes | Yes | recipient loop; swan_song/an_offer happy-path + decoy |
| 614.1 (token-creation replacements are the recipient's) | Yes | Yes | `apply_token_creation_replacement` keyed per-recipient; both doubling tests + reverse decoy |
| 701.5 / 701.5g (counter + "its controller creates") | Yes | Yes | CounterSpell capture; SpellCountered event assertions |
| An Offer 2022-04-29 #1 (uncounterable-but-legal → tokens) | Yes | Indirect | capture before `cant_be_countered`; not exercised by a `cant_be_countered` victim (see note) |
| An Offer 2022-04-29 #2 (illegal target → no tokens) | Yes | No | `None` capture → empty recipients; no dedicated test |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| swan_song | Partial | 0 | No (target over-broad) | Recipient fixed; `TargetSpell` should be `TargetSpellWithFilter` — Finding 1 HIGH |
| an_offer_you_cant_refuse | Yes | 0 | Yes | `non_creature` filter honored; two Treasures to countered controller; mana ability preserved |

## Observations (informational, out of scope — no action required)

- `acererak_the_archlich.rs` correctly stays `known_wrong`: its wrongness (1) needs a
  **per-ForEach-iteration** recipient (`EffectTarget::CurrentIterationPlayer`), which PB-EF2
  did **not** add (it added `ControllerOfCounteredSpell`/`ControllerOfTriggeringObject`), and
  wrongness (2) (MayPayOrElse) is unaddressed. Its note now cites shifted line numbers
  (`effects/mod.rs:677/3113/3196`) after the PB-EF2 edits — inherent note drift, not a blocker.
- Consider adding a test for An Offer ruling #1 using a genuinely uncounterable victim spell
  (`cant_be_countered: true`) to lock in the "capture-before-continue" behaviour, and #2 with
  an illegal target ⇒ zero tokens. Both paths are correct in code but not directly exercised
  (MEDIUM-adjacent test gap; not raised as a formal finding since the recipient decoys already
  fail loudly on any capture regression).

## Previous Findings

N/a — first review of PB-EF2.

---

## Resolution (worker, 2026-07-18)

- **HIGH (swan_song bare `TargetSpell`)** — FIXED. `swan_song.rs` now uses
  `TargetSpellWithFilter(TargetFilter { has_card_types: vec![Instant, Sorcery, Enchantment], .. })`
  (OR-semantics per `matches_filter` `effects/mod.rs:8144`, confirmed against `flusterstorm.rs`).
  New non-vacuous test `test_swan_song_cannot_target_a_creature_spell` asserts the CastSpell is
  rejected against a creature spell (would pass/succeed under bare `TargetSpell`).
- **LOW (stale note in stack/045 line 196)** — FIXED. Note now reads "p1 (Damnation's
  controller — the countered spell's controller) creates …".
- Gates re-run after fixes: build --workspace clean; `cargo test --all` **3355 passed, 0
  failed**; clippy -D warnings clean; fmt --check clean; check-defs-fmt.sh clean (1783 defs).
