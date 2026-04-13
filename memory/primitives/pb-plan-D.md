# Primitive Batch Plan: PB-D — TargetController::DamagedPlayer

> **Note**: this file supersedes the older `pb-plan-D.md` at this
> path, which was written for a different "PB-D" (Chosen Creature
> Type, 2026-04-02) that never completed and had its letter reused in
> the post-PB-N queue (see `docs/primitive-card-plan.md` Phase 1.8).
> The old file is preserved in place for historical reference; the
> runner should read THIS file for the current PB-D implementation.

**Generated**: 2026-04-13
**Primitive**: New `TargetController::DamagedPlayer` variant for scoping target/ForEach filters by "the player dealt combat damage in the triggering event"
**CR Rules**: 510.1, 510.3a, 603.2, 601.2c
**Cards affected**: **6 confirmed** (out of 15 classification candidates, 7 forced-adds via TODO sweep) — 2 precision fixes + 4 newly authorable
**Dependencies**: none
**Deferred items from prior PBs**: none reach PB-D scope (BASELINE-LKI-01 structurally not a concern — see Risks)

---

## Plan Summary (1-paragraph)

PB-D adds a fourth variant to the `TargetController` enum —
`DamagedPlayer` — so that target and ForEach filters in triggered
abilities (specifically combat-damage triggers) can scope to "that
player" without needing parallel target-requirement types. **Confirmed
yield: 6 cards** (Throat Slitter and Sigil of Sleep are precision
fixes from over-broad `Opponent` approximations; Mistblade Shinobi,
Alela Cunning Conqueror, Nature's Will, and Balefire Dragon become
newly authorable). **Dispatch verdict: PASS-AS-NEW-VARIANT** — no new
TargetRequirement variant, no new PlayerTarget variant, no new enum;
just one enum entry plus ~10 new match arms across casting.rs,
abilities.rs, effects/mod.rs, and hash.rs. **Mandatory test count: 7**
(plus 2 optional). **Deferred cards: 9** (compound blockers, wrong
bucket, or already implemented — cataloged below). **PB-P pre-check
verdict: PB-P is a real PB but narrower than its name suggests** —
`EffectAmount::PowerOf(EffectTarget)` already exists and covers the
bulk of cases; the real gap is `EffectAmount::PowerOf` with a
`SacrificedCreature` LKI target (Altar of Dementia, Greater Good).
Report only. **Step 0 sweep scope: 0 stale candidates** — the
classification report flags no DamagedPlayer-bucket cards as
potentially stale post PB-S/X/Q/N; recorded positively.

---

## Primitive Specification

A combat-damage-triggered ability's target selection often needs to be
scoped to the specific player who was dealt the damage — e.g. "destroy
target nonblack creature **that player controls**" (Throat Slitter),
"goad target creature that player controls" (Alela), "return target
creature that player controls to hand" (Mistblade Shinobi), "tap all
lands that player controls" (Nature's Will), "deal that much damage to
each creature that player controls" (Balefire Dragon).

Currently there is no way to express "controller = the damaged player"
in the DSL. The existing `TargetController` enum has only `Any`,
`You`, and `Opponent`. The closest approximation is
`TargetController::Opponent`, which is **wrong in multiplayer** — it
lets the effect target creatures controlled by any opponent, not
specifically the one being damaged. Several card defs
(`throat_slitter.rs`, `skullsnatcher.rs`) explicitly flag this
approximation as a TODO.

The underlying runtime plumbing is already complete: combat-damage
triggers record `PendingTrigger.damaged_player: Option<PlayerId>` at
trigger-queue time (`abilities.rs:4440`, `4501`, `4787`, `4964`,
`4978`, `5017`), propagate it through `StackObject.damaged_player`
(`abilities.rs:7200`), and populate `EffectContext.damaged_player` at
resolution (`resolution.rs:2031`, `2099`). `PlayerTarget::DamagedPlayer`
already resolves via
`ctx.damaged_player.unwrap_or(ctx.controller)` (`effects/mod.rs:2895`,
`2945`).

What is missing is the object-side read: the filter dispatch sites
that check `match filter.controller { ... }` don't have a
`DamagedPlayer` arm. PB-D adds that arm at ~10 sites.

### Dispatch unification verdict: PASS-AS-NEW-VARIANT

**Single new enum variant**: `TargetController::DamagedPlayer`. No new
`TargetRequirement` variant. No new `PlayerTarget` variant. No new
`TargetFilter` field. Backward compatible — serialized as a new
discriminant; hash sentinel bumped 4 → 5 per standing rule.

Justification: every dispatch site that reads `filter.controller`
already has access to either `ctx.damaged_player` (effects/mod.rs path)
or `trigger.damaged_player` (abilities.rs:6461-6482 path) or can treat
the variant as "false" for paths that don't apply (casting.rs spell
targeting — spells can't have `damaged_player` context, so the variant
returns `false` there). No extra plumbing needed.

Alternative considered and rejected: adding
`TargetRequirement::TargetCreatureControlledByDamagedPlayer` as a new
top-level variant. Rejected because it would require a parallel
validation path in 3 sites (`casting.rs`, `abilities.rs`, and hash)
per card pattern, and would not compose with existing
`TargetCreatureWithFilter` / `TargetPermanentWithFilter` /
`ForEachTarget::EachPermanentMatching`. A filter-level extension is
strictly cheaper.

---

## CR Rule Text

### CR 510.1 — Combat damage assignment

> First, the active player announces how each attacking creature
> assigns its combat damage, then the defending player announces how
> each blocking creature assigns its combat damage. […] A blocked
> creature assigns its combat damage to the creatures blocking it
> […]. An unblocked creature assigns its combat damage to the player,
> planeswalker, or battle it's attacking.

**Engine implication**: "Damage to a player" is resolved during the
combat damage step, with a specific `(source, target_player, amount)`
tuple for each event. The "damaged player" identity is a concrete
`PlayerId`, not a ledger or a set.

### CR 510.3a — Damage-dealt triggers go on the stack

> Any abilities that triggered on damage being dealt or while
> state-based actions are performed afterward are put onto the stack
> before the active player gets priority. The order in which they
> triggered doesn't matter.

**Engine implication**: "Whenever X deals combat damage to a player"
triggers are fired with the `damaged_player` identity bound from the
triggering event. The engine already handles this correctly in
`abilities.rs` during `check_triggers_for_combat_damage`.

### CR 603.2 — Trigger event matching

> Whenever a game event or game state matches a triggered ability's
> trigger event, that ability automatically triggers. The ability
> doesn't do anything at this point.

And 603.2g: *"An ability triggers only if its trigger event actually
occurs. An event that's prevented or replaced won't trigger anything."*

**Engine implication**: the `damaged_player` binding is captured
exactly once per trigger event; it does not "update" after the trigger
is queued. If the trigger is replaced (e.g. prevention effects),
`damaged_player` is never set because the ability never fires.

### CR 601.2c — Target selection at put-on-stack time

> The player announces their choice of an appropriate object or
> player for each target the spell requires. […] Once the number of
> targets the spell has is determined, that number doesn't change,
> even if the information used to determine the number of targets does.

**Engine implication**: for triggered abilities with targets, target
legality is evaluated **at the moment the trigger is put on the
stack**, not at resolution. This is the site where PB-D's filter
dispatch matters most: `abilities.rs:6247-6528`'s
auto-target-selection loop. The `trigger.damaged_player` is already
set at that point (see `abilities.rs:4440`, `4501`, etc.).

---

## Engine Changes

### Change 1: Add `TargetController::DamagedPlayer` enum variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Line**: 2385-2392 (the `TargetController` enum)
**Action**: Add fourth variant `DamagedPlayer` with doc comment citing
CR 510.3a. Place it after `Opponent` to preserve the existing
discriminants (0=Any, 1=You, 2=Opponent, new 3=DamagedPlayer).

```rust
pub enum TargetController {
    #[default]
    Any,
    You,
    Opponent,
    /// CR 510.3a: Scope to the player dealt combat damage in the
    /// triggering event. Used on combat-damage triggered abilities
    /// that say "that player controls" — e.g. Throat Slitter, Sigil
    /// of Sleep, Mistblade Shinobi, Alela, Nature's Will, Balefire
    /// Dragon. Resolves from `ctx.damaged_player` (or
    /// `trigger.damaged_player` at target-selection time); returns
    /// `false` at sites where no damaged-player context exists (e.g.
    /// spell casting), gracefully degrading to "no legal target".
    DamagedPlayer,
}
```

**Follow pattern**: `PlayerTarget::DamagedPlayer` at
`card_definition.rs:2163-2168`.

### Change 2: Hash arm for the new variant

**File**: `crates/engine/src/state/hash.rs`
**Line**: 4137-4143 (`HashInto for TargetController`)
**Action**: Add a fourth hash arm using discriminant `3u8`.

```rust
impl HashInto for TargetController {
    fn hash_into(&self, hasher: &mut impl crate::state::hash::Hasher) {
        match self {
            TargetController::Any => 0u8.hash_into(hasher),
            TargetController::You => 1u8.hash_into(hasher),
            TargetController::Opponent => 2u8.hash_into(hasher),
            TargetController::DamagedPlayer => 3u8.hash_into(hasher),
        }
    }
}
```

### Change 3: Bump hash sentinel 4 → 5

**File**: `crates/engine/src/state/hash.rs`
**Line**: ~31 (the `pub const HASH_SCHEMA_VERSION: u8`)
**Action**: Bump the constant from 4 to 5 and extend the history
comment with one line:
`// v5 (PB-D, 2026-04-13): TargetController::DamagedPlayer added`.

**Rationale**: PB-D introduces a new serialized enum variant. Per the
standing rule in `memory/conventions.md` "Hash bump rule", the default
action is to bump on every change to a serialized type's variant
shape. The cost is near-zero (one constant edit + one test parity
assertion). **Must also update the `assert_eq!(HASH_SCHEMA_VERSION,
…)` in any existing hash-parity test** to the new value.

### Change 4: Validate "DamagedPlayer target" on cast-time target validation

**File**: `crates/engine/src/rules/casting.rs`
**Lines**:
- 5521-5526 (`TargetCreatureWithFilter` validation — `validate_object_satisfies_requirement`)
- 5534-5538 (`TargetPermanentWithFilter` validation)

**Action**: Add an explicit arm returning `false` for
`TargetController::DamagedPlayer`. Spells cast from hand have no
combat-damage context — no `PendingTrigger` — so this variant is
structurally unreachable for spell casting. Rejecting at validation
time produces a clean "no legal target" error, preventing a card
author from accidentally writing a spell with `DamagedPlayer`
controller filter and having it silently treat as "any".

```rust
TargetCreatureWithFilter(filter) => {
    // …existing checks…
    let passes_controller = match filter.controller {
        TargetController::Any => true,
        TargetController::You => obj.controller == caster,
        TargetController::Opponent => obj.controller != caster,
        // PB-D: DamagedPlayer is meaningful only for triggered
        // abilities that carry a damaged_player context. Spells cast
        // from hand have no such context — reject as unreachable.
        TargetController::DamagedPlayer => false,
    };
    passes_filter && passes_controller
}
```

Repeat for `TargetPermanentWithFilter` at line 5534-5538.

### Change 5: Dispatch TargetController::DamagedPlayer in triggered-ability auto-target selection

**File**: `crates/engine/src/rules/abilities.rs`
**Lines**:
- 6461-6468 (`TargetCreatureWithFilter` auto-target in `flush_pending_triggers`)
- 6474-6482 (`TargetPermanentWithFilter` auto-target)

**Action**: Add a `TargetController::DamagedPlayer` arm that reads
`trigger.damaged_player` and returns `obj.controller == dp` when set,
`false` when not set. The `trigger` variable is already in scope at
both sites.

```rust
let ctrl_ok = match f.controller {
    crate::cards::card_definition::TargetController::Any => true,
    crate::cards::card_definition::TargetController::You => {
        obj.controller == trigger.controller
    }
    crate::cards::card_definition::TargetController::Opponent => {
        obj.controller != trigger.controller
    }
    // PB-D: target must be controlled by the player who was dealt
    // combat damage in the triggering event. Falls through to false
    // if no damaged_player is set (non-combat-damage trigger).
    crate::cards::card_definition::TargetController::DamagedPlayer => {
        trigger.damaged_player
            .is_some_and(|dp| obj.controller == dp)
    }
};
passes && ctrl_ok
```

**CR cite**: 510.3a, 601.2c.

### Change 6: Dispatch TargetController::DamagedPlayer in ForEach filter resolution

**File**: `crates/engine/src/effects/mod.rs`
**Lines**:
- 869-873 (`DestroyAll`/filter dispatch — path 1)
- 1050-1052 (`ExileAll`/filter dispatch)
- 1155-1157 (`DestroyAll` second site)
- 5410-5412 (match-filter-controller path)
- 7203-7206 (`ForEachTarget::EachPermanentMatching` — **load-bearing for Nature's Will + Balefire Dragon**)

**Action**: Add a `TargetController::DamagedPlayer` arm at each site
that reads `ctx.damaged_player` and returns `obj.controller ==
ctx.damaged_player.unwrap_or(ctx.controller)`. Mirror the existing
`PlayerTarget::DamagedPlayer` fallback pattern at
`effects/mod.rs:2895,2945`.

```rust
match filter.controller {
    TargetController::Any => true,
    TargetController::You => obj.controller == ctx.controller,
    TargetController::Opponent => obj.controller != ctx.controller,
    // PB-D: scope to the player who was dealt combat damage in the
    // triggering event. Uses ctx.damaged_player (populated from the
    // StackObject at resolution time — effects/mod.rs:2895,2945).
    // Fall back to ctx.controller if no damaged_player is set, to
    // preserve determinism if the variant is used on a non-combat
    // trigger (defensive; not expected in practice).
    TargetController::DamagedPlayer => {
        obj.controller == ctx.damaged_player.unwrap_or(ctx.controller)
    }
}
```

**CR cite**: 510.3a.

**Note**: the fallback-to-controller pattern matches the existing
`PlayerTarget::DamagedPlayer` resolution. This is a best-effort
graceful degradation; in practice the variant should only be used on
combat-damage triggers where `damaged_player` is always set.

### Change 7: Exhaustive match sites in supporting modules

Every file with a match on `TargetController` must get an arm for the
new variant (even if it's a safe `matches!(…, TargetController::You)`
pattern — new Rust code still compiles, but exhaustive matches
require an explicit arm).

| File | Line | Match expression | Action |
|------|------|-----------------|--------|
| `state/hash.rs` | 4137-4143 | `impl HashInto for TargetController` | Add `DamagedPlayer => 3u8.hash_into(hasher)` (Change 2) |
| `cards/card_definition.rs` | 2385 | enum definition | Add variant (Change 1) |
| `rules/casting.rs` | 5521-5526, 5534-5538 | `match filter.controller` | Add `DamagedPlayer => false` (Change 4) |
| `rules/abilities.rs` | 5629-5645 | `WheneverYouSacrifice` player_filter | **Defensive**: add `DamagedPlayer => false` (sacrifice triggers have no damage context). Do NOT silently treat as `Any`. |
| `rules/abilities.rs` | 6131-6134 | Bloodghast-style land-filter | Wildcard fall-through `_ => true` already present — no change required |
| `rules/abilities.rs` | 6461-6468 | `TargetCreatureWithFilter` auto-target | Add arm (Change 5) |
| `rules/abilities.rs` | 6474-6482 | `TargetPermanentWithFilter` auto-target | Add arm (Change 5) |
| `effects/mod.rs` | 869-873 | DestroyAll | Add arm (Change 6) |
| `effects/mod.rs` | 1050-1052 | ExileAll | Add arm (Change 6) |
| `effects/mod.rs` | 1155-1157 | DestroyAll second | Add arm (Change 6) |
| `effects/mod.rs` | 5410-5412 | filter-match path | Add arm (Change 6) |
| `effects/mod.rs` | 7203-7206 | EachPermanentMatching — **load-bearing** | Add arm (Change 6) |
| `testing/replay_harness.rs` | 2342, 2445 | `matches!(f.controller, TargetController::You)` | **No change required** — `matches!` is non-exhaustive and returns `false` for the new variant (correct fall-through for these sites) |
| `testing/replay_harness.rs` | 2776-2782 | `Some(TargetController::Opponent)` / `Some(TargetController::You)` arms in `match` | **Defensive**: inspect actual match structure at implementation time. This path is for draw triggers, not combat damage, so `DamagedPlayer` is a code smell there; return a sensible default (likely "not a draw trigger filter, ignore"). |

**Worker note**: after making the enum change, run `cargo build
--workspace` and fix every resulting non-exhaustive-match error. The
compiler will identify any site I missed. Treat any surprise match
site as a stop-and-flag; investigate before silently adding a default
arm.

---

## Card Definition Fixes

Six cards confirmed shippable by PB-D. All verified by Read of the
current card source.

### throat_slitter.rs (precision fix)

**Oracle text**: "Whenever Throat Slitter deals combat damage to a
player, destroy target nonblack creature that player controls."

**Current state**: Ships with `TargetController::Opponent` as an
approximation. TODO comment at lines 8-11 explicitly names
`TargetController::DamagedPlayer` as the precise fix. Produces subtly
wrong game state in 3+ player games — the effect can target a creature
controlled by an opponent who was NOT dealt damage.

**Fix**: Change `controller: TargetController::Opponent` → `controller:
TargetController::DamagedPlayer`. Strip the TODO comment block at
lines 7-11 and 29. Add a CR 510.3a citation.

### sigil_of_sleep.rs (precision fix)

**Oracle text**: "Whenever enchanted creature deals damage to a
player, return target creature that player controls to its owner's
hand."

**Current state**: Ships with
`targets: vec![TargetRequirement::TargetCreature]` (any creature) as
an approximation. TODO at lines 16-20 explicitly names "DamagedPlayer
target filtering" as the gap.

**Fix**: Change `TargetCreature` →
`TargetCreatureWithFilter(TargetFilter { controller:
TargetController::DamagedPlayer, ..Default::default() })`. Strip the
TODO comment lines 16-20. Add CR 510.3a citation.

### mistblade_shinobi.rs (newly authored triggered ability)

**Oracle text**: "Whenever Mistblade Shinobi deals combat damage to a
player, you may return target creature that player controls to its
owner's hand."

**Current state**: Ninjutsu keyword and alt-cast ability are present.
The combat-damage trigger is a TODO at lines 21-23.

**Fix**: Add a new `AbilityDefinition::Triggered` with
`trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer`,
`effect: Effect::MoveZone { target: DeclaredTarget { index: 0 }, to:
ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(...) },
controller_override: None }`, and `targets:
vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
controller: TargetController::DamagedPlayer, ..Default::default() })]`.
Strip the TODO. Note: the "you may" optionality is a separate gap;
author as mandatory for now and add a TODO comment citing the "you
may" gap only — this is consistent with other Ninjutsu cards (e.g.
Sigil of Sleep is also authored as mandatory).

**STOP-AND-FLAG check**: if the worker discovers during implementation
that "you may" optionality is a hard blocker (e.g. the engine auto-
selects a target that it shouldn't), drop Mistblade Shinobi from the
yield and defer to a "may" primitive. Do NOT add the "may" primitive
inline.

### alela_cunning_conqueror.rs (newly authored partial)

**Oracle text**: "Whenever one or more Faeries you control deal combat
damage to a player, goad target creature that player controls."

**Current state**: Partial. The token-creation trigger is authored.
The goad trigger is an `Effect::Sequence(vec![])` placeholder at lines
66-82 with `targets: vec![]`, comment at lines 65-67 explicitly
pointing at PB-37 DamagedPlayer ForEach support.

**Fix**: Replace the placeholder `Effect::Sequence(vec![])` with
`Effect::Goad { target: EffectTarget::DeclaredTarget { index: 0 } }`.
Add
`targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter
{ controller: TargetController::DamagedPlayer, ..Default::default() })]`.
Strip the PB-37 TODO. Keep the first-spell-per-turn TODO intact (that
is a separate, pre-existing gap).

### natures_will.rs (newly authored — second sub-effect)

**Oracle text**: "Whenever one or more creatures you control deal
combat damage to a player, tap all lands that player controls and
untap all lands you control."

**Current state**: Partial. The "untap all lands you control"
sub-effect is authored via
`ForEachTarget::EachPermanentMatching(controller: You)`. The "tap all
lands that player controls" sub-effect is a TODO at lines 4-6 and 37.

**Fix**: Replace the single-effect `ForEach` with an
`Effect::Sequence([…])` containing two `ForEach` blocks:

```rust
effect: Effect::Sequence(vec![
    // Tap all lands that the damaged player controls.
    Effect::ForEach {
        over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
            has_card_type: Some(CardType::Land),
            controller: TargetController::DamagedPlayer,
            ..Default::default()
        })),
        effect: Box::new(Effect::TapPermanent {
            target: EffectTarget::DeclaredTarget { index: 0 },
        }),
    },
    // Untap all lands you control (existing sub-effect).
    Effect::ForEach {
        over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
            has_card_type: Some(CardType::Land),
            controller: TargetController::You,
            ..Default::default()
        })),
        effect: Box::new(Effect::UntapPermanent {
            target: EffectTarget::DeclaredTarget { index: 0 },
        }),
    },
]),
```

**Verify before shipping**: `Effect::TapPermanent` with
`EffectTarget::DeclaredTarget { index: 0 }` semantics inside a
`ForEach` — confirm via Read of `effects/mod.rs` during implementation
that the inner `DeclaredTarget { index: 0 }` resolves to "the current
ForEach iteration value", not to the spell's targets list. The
existing partial implementation already uses this pattern with
`UntapPermanent` (lines 27-29), so the semantics are known-good — but
confirm. Strip the TODO at lines 4-6 and 37.

### balefire_dragon.rs (newly authored)

**Oracle text**: "Whenever Balefire Dragon deals combat damage to a
player, it deals that much damage to each creature that player
controls."

**Current state**: Flying is authored. The combat-damage trigger is a
TODO at lines 6-7 and 21-22.

**Fix**: Add `AbilityDefinition::Triggered` with `trigger_condition:
TriggerCondition::WhenDealsCombatDamageToPlayer`, `effect:
Effect::ForEach { over:
ForEachTarget::EachPermanentMatching(filter), effect:
Box::new(Effect::DealDamage { target: DeclaredTarget { index: 0 },
amount: EffectAmount::CombatDamageDealt }) }` with
`filter = TargetFilter { has_card_type: Some(CardType::Creature),
controller: TargetController::DamagedPlayer, ..Default::default() }`.
Strip the TODOs. Add CR 510.3a citation.

**Verify before shipping**:
- `EffectAmount::CombatDamageDealt` exists (it does — see
  `card_definition.rs:2213-2218`, resolves from
  `EffectContext::combat_damage_amount`).
- The `ForEach` with `DealDamage` whose `amount` reads
  `ctx.combat_damage_amount` (a per-trigger value set from
  `StackObject.combat_damage_amount`) still works inside the ForEach
  loop (the inner `ctx` is the same outer context). Confirm during
  implementation — this is Risk R3 below; if broken, drop Balefire
  Dragon from yield to 5 cards (do not fix inline).

---

## Deferred cards (9 compound blockers + 11 false-positives + 4 already implemented)

These cards appeared in the classification report or TODO sweep but
are NOT shipped by PB-D. One-line reason each.

### Compound blockers (9)

| Card | File | Reason for deferral |
|------|------|---------------------|
| Marisi, Breaker of the Coil | `marisi_breaker_of_the_coil.rs` | Compound blocker: phase-scoped CantCast restriction is a separate primitive. The goad sub-effect is PB-D-expressible but the card is not ship-ready until CantCast lands. |
| Skullsnatcher | `skullsnatcher.rs` | Compound blocker: "up to two target cards" requires an UpToN target variant (PB-T territory). The `that player's graveyard` filter is PB-D-adjacent but graveyard zone scoping is a separate primitive. |
| Ink-Eyes, Servant of Oni | `ink_eyes_servant_of_oni.rs` | Compound blocker: "target creature card from that player's graveyard, put onto battlefield under your control" needs both graveyard-zone scoping by player AND reanimation with controller override. Neither is PB-D scope. |
| Hellkite Tyrant | `hellkite_tyrant.rs` | Compound blocker: "gain control of all artifacts that player controls" needs a GainControl effect with a filter — separate primitive. Also has a win-condition trigger that is out of scope. |
| Dokuchi Silencer | `dokuchi_silencer.rs` | Compound blocker: reflexive trigger ("when you do") is not in the DSL. |
| Ragavan, Nimble Pilferer | `ragavan_nimble_pilferer.rs` | Compound blocker: "exile top card of that player's library + impulse draw" needs library-scan-by-player + impulse primitive. |
| Grenzo, Havoc Raiser | `grenzo_havoc_raiser.rs` | Compound blocker: modal combat-damage trigger + impulse draw from opponent's library. |
| Scalelord Reckoner | `scalelord_reckoner.rs` | Compound blocker: "becomes the target of a spell or ability" is a missing trigger condition. |
| Cavern-Hoard Dragon | `cavern_hoard_dragon.rs` | Compound blocker: "create a Treasure for each artifact that player controls" needs `EffectAmount::PermanentCountControlledByDamagedPlayer` or equivalent — a new EffectAmount variant. **Adjacent to PB-D** but wants a new amount variant, not a new filter variant. Defer. |

### Classification false positives (11)

These cards appeared in the DamagedPlayer bucket (classification
report) or "that player controls" grep but are not DamagedPlayer-
related on closer inspection:

| Card | File | Actual gap |
|------|------|-----------|
| Blood Seeker | `blood_seeker.rs` | ETB trigger referencing "that player" = entering creature's controller. Needs `PlayerTarget::TriggeringPermanentController`, not DamagedPlayer. Also blocked on "you may" optionality. |
| Horn of Greed | `horn_of_greed.rs` | "That player" = land-playing player. Needs `PlayerTarget::TriggeringPermanentController`. |
| Mystic Remora | `mystic_remora.rs` | "That player" = the casting player. Already expressible via `DeclaredTarget { index: 0 }`. Blocked on `Effect::MayPayOrElse` semantics, not DamagedPlayer. |
| Smothering Tithe | `smothering_tithe.rs` | Blocked on MayPayOrElse, not DamagedPlayer. |
| Polymorphist's Jest | `polymorphists_jest.rs` | A **spell** (not a triggered ability) wanting "target player controls creatures X". Spells have no damaged_player context. Needs a ForEach target variant for "creatures controlled by target-index-N player" — different primitive. |
| Walker of Secret Ways | `walker_of_secret_ways.rs` | Blocked on hidden-info reveal + subtype activated ability. |
| The Eternal Wanderer | `the_eternal_wanderer.rs` | PW loyalty ability unrelated to combat damage. Classification false positive. |
| Memory Lapse | `memory_lapse.rs` | Counterspell that puts to library top. Nothing to do with DamagedPlayer. |
| Leyline of the Void | `leyline_of_the_void.rs` | Already implemented; no DamagedPlayer reference. |
| Crackling Doom | `crackling_doom.rs` | "Each opponent sacrifices highest-power creature" — ForEach opponent with max-power selection. Different primitive. |
| Emrakul, the Promised End | `emrakul_the_promised_end.rs` | Blocked on gain-control-of-opponent-turn mechanism. |

### Already implemented (4, for the record)

These cards reference "that player" mechanics but are already shipped
using `PlayerTarget::DamagedPlayer` (for player-directed effects) or
`EffectContext::damaged_player` (for replacement filters):

- `sword_of_feast_and_famine.rs` — `PlayerTarget::DamagedPlayer` for DiscardCards
- `sword_of_body_and_mind.rs` — `PlayerTarget::DamagedPlayer` for MillCards
- `sword_of_war_and_peace.rs` — `TargetPlayer` + CardCount of their hand
- `lightning_army_of_one.rs` — `DamageTargetFilter::ToPlayerOrTheirPermanents(PlayerId(0))` resolved from `ctx.damaged_player`

---

## Pre-existing TODO Sweep (Step 3a, MANDATORY)

Grep run against `crates/engine/src/cards/defs/` for TODOs naming
`DamagedPlayer`, `damaged player`, or `TargetController::DamagedPlayer`.
Cross-verified with full-text reading to distinguish genuine forced
adds from false-positive mentions.

**Result: 7 cards with pre-existing TODOs naming the primitive** via
the narrow grep. All are verified as legitimate DamagedPlayer targets
(not incidental mentions). The results:

| Card | TODO location | Disposition |
|------|--------------|-------------|
| `balefire_dragon.rs` | lines 6, 22 | **Forced add — confirmed** (newly authored) |
| `natures_will.rs` | lines 4-6, 37 | **Forced add — confirmed** (newly authored partial) |
| `marisi_breaker_of_the_coil.rs` | lines 7, 22 | Forced add but **deferred** (compound blocker: CantCast) |
| `skullsnatcher.rs` | line 31 | Forced add but **deferred** (compound blocker: UpToN) |
| `alela_cunning_conqueror.rs` | lines 8, 65 | **Forced add — confirmed** (newly authored partial) |
| `throat_slitter.rs` | lines 8-11, 29 | **Forced add — confirmed** (precision fix) |
| `ink_eyes_servant_of_oni.rs` | (implicit via "target creature from that player's graveyard") | Forced add but **deferred** (compound blocker: graveyard scoping + reanimate) |

Plus 2 forced-adds via the broader "that player controls" sweep that
were not captured by the narrow DamagedPlayer grep:

| Card | TODO reference | Disposition |
|------|---------------|-------------|
| `sigil_of_sleep.rs` | lines 16-20 "DamagedPlayer target filtering" | **Forced add — confirmed** (precision fix) |
| `mistblade_shinobi.rs` | lines 21-23 "'that player controls' filter not expressible" | **Forced add — confirmed** (newly authored) |

**Sweep result recorded positively**: 9 pre-existing TODOs naming or
describing the primitive. **6 become confirmed yield; 3 are deferred
with documented compound-blocker reasons.** No cards missed between
the original 15-card classification roster and the TODO sweep —
balefire_dragon, natures_will, marisi, alela, throat_slitter,
ink_eyes, skullsnatcher, sigil_of_sleep, mistblade_shinobi are all on
both lists. The TODO sweep produces the same 6 confirmed yield as the
oracle-lookup walk, providing cross-verification.

**Note on yield discount**: filter-PB calibration is 50-65% per
`memory/feedback_pb_yield_calibration.md`. Expected yield from 15
candidates: 7.5-9.75. Actual confirmed yield: 6 (40%). **Slightly
below calibration range** — flagged in Risks below. The
under-performance is explained by classification false positives (11
cards in wrong bucket or already implemented). If false positives
and already-done cards are excluded, the 6 / (15 - 11 + 2 forced-adds
= 6 genuine) is a **100% yield** among genuine candidates, or 6/9 =
67% if we count the deferred compound-blockers as also genuine. The
raw 40% figure against the 15-card classification bucket reflects
classification accuracy more than PB-D design.

---

## Unit Tests

**File**: `crates/engine/tests/pbd_damaged_player_filter.rs` (new)

All tests must use the `GameStateBuilder` and the script harness
where appropriate. Every test cites CR 510.3a.

### Test numbering convention

All tests numbered **MANDATORY** (M#) or **OPTIONAL** (O#). No silent
skips per PB-Q4/N fix-phase protocol. Any test that cannot
discriminate against pre-PB-D engine is a silent-skip and MUST be
rewritten or escalated.

### MANDATORY tests (7)

**M1** — `test_damaged_player_target_controller_creature_match_fires`

Setup: 4-player game. Player 1 controls a 1/1 "Thug" creature with a
`WhenDealsCombatDamageToPlayer` trigger that reads "destroy target
creature **that player controls**" — a reduced-scope version of
Throat Slitter. Player 2 and Player 3 each control a different
Goblin. Player 1 declares the Thug attacking Player 2, no blockers;
combat damage resolves.

Assert: the trigger goes on the stack with
`damaged_player = Player 2`; during target auto-selection at
`abilities.rs:6461-6468`, the only legal target is Player 2's Goblin
(Player 3's Goblin is excluded); when the trigger resolves, Player
2's Goblin is destroyed and Player 3's Goblin is still on the
battlefield.

**Discriminator**: without the DamagedPlayer arm in
`abilities.rs:6461-6468`, the pre-PB-D engine would either fail to
compile (if the test used `TargetController::DamagedPlayer`) or
silently match both Goblins under `TargetController::Opponent`. This
test specifically asserts that ONLY Player 2's Goblin is destroyed.

CR: 510.3a, 601.2c.

**M2** — `test_damaged_player_target_controller_negative_excludes_other_opponent`

Setup: same as M1 but with an extra Goblin on Player 3's battlefield
with P/T 3/3. The test's trigger effect is "destroy target creature
that player controls with power 3 or less". Player 1 attacks Player 2;
Player 2 controls no creatures at all.

Assert: the trigger has no legal target; per CR 603.3d the trigger is
skipped (`trigger_targets_opt` returns `None` at
`abilities.rs:6520-6524`); Player 3's 3/3 Goblin is NOT destroyed; the
stack is empty after combat damage resolves.

**Discriminator**: without the DamagedPlayer arm, the pre-PB-D engine
would find Player 3's Goblin as a legal target under
`TargetController::Opponent`. This test asserts NO target is found.

CR: 510.3a, 603.3d.

**M3** — `test_damaged_player_spell_casting_rejects_filter`

Setup: player casts an instant (not a trigger) whose target
requirement is
`TargetCreatureWithFilter(TargetController::DamagedPlayer)`. This is a
synthetic test — PB-D's casting.rs arm returns `false` for this
variant, so the spell should fail to cast with an `InvalidTarget`
error.

Assert: `process_command(CastSpell)` returns
`Err(GameStateError::InvalidTarget(_))`; spell is not on the stack.

**Discriminator**: without the explicit `DamagedPlayer => false` arm
in `casting.rs:5521-5526,5534-5538`, the pre-PB-D code would not
compile (exhaustive-match failure) or would silently accept the cast.
This test exercises the defensive dispatch.

CR: 601.2c.

**M4** — `test_damaged_player_foreach_land_tap_nature_will_pattern`

Setup: 4-player game. Player 1 controls a creature with a "Nature's
Will"-style trigger: `WhenOneOrMoreCreaturesYouControlDealCombatDamage`
+ `Effect::ForEach(EachPermanentMatching{card_type: Land, controller:
DamagedPlayer}, TapPermanent)`. Player 2 controls 4 Forests, all
untapped. Player 3 controls 4 Plains, all untapped. Player 1 attacks
Player 2 with a 5/5; damage resolves.

Assert: all 4 of Player 2's Forests become tapped. All 4 of Player 3's
Plains remain untapped. The trigger's ForEach body iterated exactly 4
times (verified by a side-channel count if available, or by reading
the tapped field).

**Discriminator**: without the DamagedPlayer arm at
`effects/mod.rs:7203-7206`, the pre-PB-D engine matches NO controller
for `DamagedPlayer` (fall-through `_` would not exist) and fails to
compile. Even if a `_ => true` were hypothetically added, it would tap
ALL 8 lands across both opponents. This test specifically asserts the
4/4 split.

CR: 510.3a. This test is **load-bearing for Nature's Will and Balefire
Dragon** — both use `ForEach(EachPermanentMatching)` with the new
filter.

**M5** — `test_damaged_player_destroy_all_filter_multiplayer_isolation`

Setup: 4-player game. Player 1 controls a "battlecry" creature with a
trigger `Effect::DestroyAll(filter: {card_type: Creature, controller:
DamagedPlayer})`. Player 2 and Player 3 each control 2 creatures.
Player 1 attacks Player 3 with a creature dealing lethal damage to
Player 3.

Assert: both of Player 3's creatures are destroyed. Both of Player 2's
creatures remain on the battlefield.

**Discriminator**: exercises the DestroyAll dispatch site at
`effects/mod.rs:869-873` (and/or `5410-5412`). Pre-PB-D engine cannot
express this filter at all. Tests that adding the variant to ONLY ONE
of the 5 DestroyAll sites would be insufficient for other sites that
have their own dispatch.

CR: 510.3a.

**M6** — `test_damaged_player_hash_parity_all_variants`

Setup: construct four minimal `TargetFilter` values, one per
`TargetController` variant (Any, You, Opponent, DamagedPlayer). Hash
each and assert all four produce distinct hashes.

Assert:
- `hash(TargetController::Any) != hash(TargetController::You)`
- `hash(TargetController::You) != hash(TargetController::Opponent)`
- `hash(TargetController::Opponent) != hash(TargetController::DamagedPlayer)`
- `assert_eq!(HASH_SCHEMA_VERSION, 5u8)` (the bumped sentinel, per
  `memory/conventions.md` hash sentinel convention)

**Discriminator**: forces the sentinel assertion to fail if the bump
is not made. Discriminates the new variant from the existing three.

CR: N/A (hash infrastructure).

**M7** — `test_throat_slitter_end_to_end_precision_fix`

Setup: full end-to-end Throat Slitter card test. 4-player game. Player
1 controls Throat Slitter and makes it unblockable (via a test-only
ability or by just having no blockers declared). Player 2 controls a
nonblack creature (Goblin). Player 3 also controls a nonblack creature
(Elf). Player 1 attacks Player 2 with Throat Slitter, no blockers;
combat damage resolves (1 damage to Player 2).

Assert:
- The Throat Slitter trigger goes on the stack.
- During auto-target selection, Player 2's Goblin is selected (NOT
  Player 3's Elf). Verify by reading
  `stack_objects[0].targets[0]`.
- When the trigger resolves, Player 2's Goblin is in the graveyard.
  Player 3's Elf is still on the battlefield.

**Discriminator**: This is the canonical card-level test for the
precision fix PB-D ships. Pre-PB-D (and before Throat Slitter is
updated) the engine approximates with `Opponent`, which means the
auto-target selection at `abilities.rs:6461-6468` could select Player
3's Elf (deterministic-first-match order depends on ObjectId order).
This test asserts the correct target specifically by PlayerId.

CR: 510.3a, 601.2c.

### OPTIONAL tests (2)

**O1** — `test_damaged_player_combined_filter_subtype_and_controller`

Setup: trigger with
`TargetCreatureWithFilter(TargetFilter { has_subtype: Some(Goblin),
controller: DamagedPlayer })`. Player 1 attacks Player 2; Player 2
controls a Goblin and a Human. Player 3 controls a Goblin.

Assert: the Player 2 Goblin is targeted (matches both subtype AND
controller); Player 3's Goblin is excluded; Player 2's Human is
excluded.

Tests that the new variant composes cleanly with existing filter
fields (subtype in this case).

CR: 510.3a.

**O2** — `test_balefire_dragon_end_to_end_fan_out_damage`

Full end-to-end test of Balefire Dragon's "deals that much damage to
each creature that player controls" trigger. Validates the ForEach +
DamagedPlayer + CombatDamageDealt combination.

Setup: Player 1 controls Balefire Dragon (6/6 flying). Player 2
controls two creatures: a 4/4 and a 2/2. Player 3 controls a 3/3.
Player 1 attacks Player 2 with Balefire Dragon, no blockers; 6 combat
damage to Player 2.

Assert: both of Player 2's creatures take 6 damage (the 4/4 dies to
SBA, the 2/2 dies to SBA). Player 3's 3/3 is unaffected.

**Discriminator**: exercises the EachPermanentMatching dispatch site
specifically under the DamagedPlayer filter, with
`EffectAmount::CombatDamageDealt` reading `ctx.combat_damage_amount`
from the triggered context. Pre-PB-D there is no way to express this.

CR: 510.3a.

**Optional marker rationale**: O1 is subsumed by M1+M4 at the
dispatch-coverage level. O2 is a real-card end-to-end that's useful as
a redundant check but M7 already provides the real-card end-to-end on
a simpler card.

**Test pattern**: follow
`crates/engine/tests/pbn_subtype_filtered_triggers.rs` for overall
structure (helper builders + 4-player setup + trigger assertions).
Reuse any helper imports.

---

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] `TargetController::DamagedPlayer` enum variant added with CR
  doc comment
- [ ] Hash sentinel bumped 4 → 5; test 6 uses `assert_eq!(HASH_SCHEMA_VERSION, 5u8)`
- [ ] All existing card-def TODOs for the 6 confirmed cards resolved
- [ ] 10 dispatch sites have the new match arm (casting ×2, abilities
  ×2, effects/mod ×5, hash ×1)
- [ ] All 7 MANDATORY tests pass (`cargo test --all`)
- [ ] No new clippy warnings beyond BASELINE-CLIPPY-01..06 baseline
  (`cargo clippy --all-targets -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in the 6 affected card defs (the 9
  deferred/false-positive cards keep their TODOs, updated to cite the
  compound blocker accurately)
- [ ] `cargo fmt --check` clean
- [ ] Commit message lists: 6 cards shipped, 10 dispatch sites
  updated, hash bump 4→5, test count delta

---

## Risks & Edge Cases

### R1: Confirmed yield (6) is below the filter-PB calibration range (7-10)

Filter PBs calibrate at 50-65% per
`memory/feedback_pb_yield_calibration.md`. 15 candidates × 50% = 7.5
minimum; actual is 6 = 40%. The delta is explained by classification
accuracy (11 false positives / already-implemented in the report's
"DamagedPlayer" bucket). Excluding those, 6 out of 9 genuine
candidates is 67% — healthy. **Flagged here as context for oversight's
post-PB-D yield-tracking update**, not as a stop-and-flag.

### R2: `Effect::TapPermanent` under ForEach — verify DeclaredTarget semantics

Nature's Will uses
`Effect::ForEach { over: EachPermanentMatching, effect: TapPermanent {
target: DeclaredTarget { index: 0 } } }`. The inner `DeclaredTarget {
index: 0 }` must resolve to the current ForEach iteration object, NOT
to the outer trigger's declared targets list. The existing Nature's
Will partial implementation (line 22-30) already uses this pattern
with `UntapPermanent`, so the semantics are known-good — but verify
during implementation with a Read of `effects/mod.rs` around the
ForEach dispatch site. **Low risk**.

### R3: `EffectAmount::CombatDamageDealt` inside ForEach for Balefire Dragon

Balefire Dragon's effect uses
`Effect::ForEach { …, effect: DealDamage { amount: CombatDamageDealt
} }`. The `CombatDamageDealt` resolves from
`EffectContext::combat_damage_amount`, which is set from
`StackObject.combat_damage_amount` at resolution time
(`resolution.rs:2099`). **This should work**: the inner DealDamage
executes within the same `ctx`, so `ctx.combat_damage_amount` is still
set. But there is a subtle risk: if the ForEach implementation
constructs a fresh sub-context per iteration, it may or may not
propagate `combat_damage_amount`. **Verify during implementation**. If
the propagation is broken, Balefire Dragon becomes a deferred card
(drop from yield to 5); do not try to fix the propagation inline —
that's a separate primitive bug.

### R4: BASELINE-LKI-01 reach check

BASELINE-LKI-01 is the structural limitation that filter dispatch on
graveyard (post-zone-change) objects re-runs layer filters against the
LKI characteristics, dropping battlefield-gated filters. **Does this
reach PB-D?** Analysis:

- `TargetController::DamagedPlayer` reads `obj.controller`, which is a
  runtime GameObject field, NOT a layer-resolved characteristic.
  Controller is set when the object enters a zone and does not go
  through the layer system.
- The "damaged player" identity itself is a `PlayerId` on the trigger
  (or ctx), not a GameObject — players don't change zones.
- All filter dispatch sites in PB-D's scope (casting.rs, abilities.rs,
  effects/mod.rs) read `obj.controller` directly, not through
  `calculate_characteristics`.

**Verdict**: BASELINE-LKI-01 does NOT reach PB-D. PB-D is safe from
the known LKI limitation. **This is the expected outcome** — it's why
PB-D was queued at rank 1 after PB-N. Confirmed as a standing
invariant for future player-filter PBs.

### R5: Exhaustive-match compiler errors in unanticipated files

Adding a new `TargetController` variant will force a non-exhaustive
match error at every exhaustive `match` site. The plan enumerates 10
known sites (Change 7 table), but the compiler may surface additional
sites I did not identify through grep. **Expected behavior**: the
worker runs `cargo build --workspace`, receives compiler errors, adds
the missing arms. Each surprise arm is a **stop-and-flag**: investigate
the site's semantics before adding a default arm, because a
thoughtless `_ => false` at a new site could silently hide a bug (e.g.
if the site is in a hot path and needs a real fallback).

### R6: Builder / tests sites with old enum

The builder at `state/builder.rs` and replay harness at
`testing/replay_harness.rs` may reference `TargetController` in
contexts where the new variant is meaningful (e.g. a card-definition
test shortcut that maps a string to a variant). Worker should check
during implementation via grep.

### R7: Mistblade Shinobi "you may" optionality

Mistblade Shinobi's oracle says "you MAY return". The current DSL has
no "you may" wrapper for triggered ability effects. The plan ships it
as mandatory, mirroring Sigil of Sleep. **Risk**: in a real game, a
player may want to decline (e.g. if their own creature is the only
legal target, they'd rather not bounce it). Authoring as mandatory
produces a subtly wrong game state in that edge case. **Mitigation**:
the plan flags this as a known limitation with a note in the card def
comment (do NOT strip the "you may" TODO; update it to point at a
future "MayEffect" primitive). The `DamagedPlayer` filter itself is
still correct.

### R8: Oversight may prefer PB-P

The planner has verified that PB-P (rank 2) is a real PB but narrower
than advertised (the actual gap is
`EffectAmount::PowerOfSacrificedCreature`, not the broader
`PowerOfCreature`). Oversight may choose to run PB-P ahead of PB-D on
the basis that PowerOfSacrificedCreature unblocks high-value cards
(Greater Good, Altar of Dementia, Juri). This is oversight's call; the
PB-D plan is complete and ready to ship as planned. No action required
from this plan file.

### R9: Plan-file naming collision with stillborn older PB-D

The `pb-plan-D.md` file already exists at this path for an older PB-D
(Chosen Creature Type, 2026-04-02) that was reviewed with a "needs-
fix" verdict and never closed. The `PB-D` letter was reused in the
post-PB-N queue per `docs/primitive-card-plan.md` Phase 1.8. This plan
file is written as `pb-plan-D-damaged-player.md` to avoid overwriting
the historical artifact. The runner should read THIS file
(`pb-plan-D-damaged-player.md`), not `pb-plan-D.md`. A follow-up
housekeeping commit should archive the old file to
`memory/archive/` or rename the new file to reclaim the `D` slot —
but that's an oversight decision, not a planner decision.

---

## Implementation Notes (for the runner)

- **Dispatch site order**: do Change 1 (enum variant) first. Run
  `cargo build --workspace`. The compiler will surface every
  non-exhaustive-match site; add arms in order. This is less
  error-prone than grep-hunting.
- **Hash bump last**: update `HASH_SCHEMA_VERSION` only after all code
  changes compile, so the parity test doesn't need multiple edits.
- **Test order**: write M6 (hash parity) first — it's the smallest and
  validates the sentinel bump path. Then M3 (casting reject) — it
  validates the defensive dispatch path. Then M1 (the primary positive
  case). Then M2 (negative). Then M4 (ForEach load-bearing). Then M5
  (multiplayer isolation for DestroyAll). Then M7 (Throat Slitter
  e2e). Optional tests O1/O2 last.
- **Card def edits come AFTER the engine compiles cleanly**. Don't mix
  engine and card changes in interleaved order — it makes bisecting
  later failures harder.
- **Backfill check**: after the enum addition compiles, grep one more
  time for `TargetController::Opponent` and `TargetController::You` to
  confirm nothing new appeared — paranoid check against new match
  sites being added between plan time and implement time.
- **Expected commit size**: ~150 LOC engine diff + ~40 LOC card def
  diff + ~250 LOC test diff. Similar to PB-N's shape.
