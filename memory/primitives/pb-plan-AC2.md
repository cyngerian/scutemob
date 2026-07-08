# Primitive Batch Plan: PB-AC2 — Optional-cost (beneficial-pay) wrapper & counter-tax

**Generated**: 2026-07-07
**Primitive**: Two new `Effect` variants — `Effect::MayPayThenEffect` (beneficial
optional-cost wrapper on resolving spells/abilities) and `Effect::CounterUnlessPays`
(caster-side counter-tax, the Mana Leak pattern).
**CR Rules**: **118.12** (primary — governs BOTH), **118.12a** (counter-tax = "unless pays"),
118.8 / 118.8b (optional additional costs), 701.5 (cast/counter), 603.2 (triggered abilities),
119.4 (paying life).
**Cards affected**: **13 clean** (7 beneficial-pay + 6 counter-tax) + ~7 improved-but-still-PARTIAL.
**Dependencies**: none new. (Reuses `Effect::CounterSpell`, `Effect::Proliferate`,
`Effect::SearchLibrary`, `can_pay_cost`/`pay_cost`, `WheneverYouDrawACard`,
`WheneverCreatureEntersBattlefield{controller_you, exclude_self}`, `ControllerProliferates`
trigger — all already present.)
**Deferred items from prior PBs**: none targeted; PB-AC1's sole HIGH finding (missed hash
field) is baked into this plan as a mandatory checklist gate.

---

## CR correction (IMPORTANT — read first)

The brief cites 118.8 / 603.2 / 701.5. Those are *supporting*. The **precise governing
rule for both primitives is CR 118.12**, verified via MCP:

> 118.12: "[Do something]. If [a player] [does, doesn't, or can't], [effect]." Or
> "[A player] may [do something]. If [that player] [does, doesn't, or can't], [effect]."
> **The action [do something] is a cost, paid when the spell or ability resolves.**
>
> 118.12a: "[Do something] unless [a player does something else]" means the same thing as
> "[A player may do something else]. If [that player doesn't], [do something]."

Consequences baked into the design:
1. The optional cost is **paid at resolution** (not on the stack) — both primitives execute
   inside `execute_effect_inner`, not at trigger-queue time. Correct.
2. CR 118.12a formally equates "**Counter target spell unless its controller pays {X}**"
   with "controller **may** pay {X}; **if they don't**, counter." So `CounterUnlessPays`
   is the *tax* family (same shape as the existing `Effect::MayPayOrElse`), and
   `MayPayThenEffect` is the *beneficial* family (sub-effect fires on **pay**, not on decline).
   This opposite-firing-condition is the whole reason two distinct variants are needed —
   `MayPayOrElse` runs its sub-effect when the player DOESN'T pay and therefore cannot
   express "if you do, draw."

118.8/701.5/603.2/119.4 remain cited where relevant (below).

---

## Primitive Specification

### 1. `Effect::MayPayThenEffect` (beneficial-pay)

The general "**You may pay/sacrifice/discard X. If you do, `<effect>`.**" pattern.
Distinct from `Effect::MayPayOrElse { cost, payer, or_else }` (card_definition.rs
L1596-1601), which has **tax** semantics: `or_else` runs when the payer DECLINES.
`MayPayThenEffect` runs `then` only when the payer PAYS.

**Deterministic non-interactive semantics** (M10+ replaces with an interactive
`Command`): the payer **pays when able**, then `then` runs; if the payer cannot pay,
nothing happens. "Able" per cost kind:
- `Cost::Mana(mc)` — payer's mana pool `can_spend(mc)` (CR 118.8). Note: mana pools empty
  between steps (CR 500.4), so during trigger/ability resolution the pool is normally
  empty → the `then` fires only if the payer has **floating** mana. Tests for mana-cost
  cards must pre-float mana (`player.mana_pool.add(color, n)`).
- `Cost::PayLife(n)` — payer's life `>= n` (CR 119.4).
- `Cost::DiscardCard` — payer has ≥1 card in hand.
- `Cost::Sacrifice(filter)` — payer controls ≥1 battlefield permanent whose layer-resolved
  characteristics match `filter` (CR 613.1d).
- `Cost::Sequence(costs)` — payer can pay **all** sub-costs (check all, then pay all).

This "pay-when-able" default is a **legal game choice** — it does not corrupt state
(architecture invariant #9). It is deterministic and replayable. Interactive choice
(pay vs. decline) is M10+ work, mirroring the existing `Command::ChooseMiracle` /
`Command::ChooseDredge` pause-and-resume pattern.

### 2. `Effect::CounterUnlessPays` (counter-tax)

"**Counter target spell unless its controller pays {cost}.**" (Mana Leak.)

**Deterministic non-interactive semantics**: the target spell's controller does **not**
pay → the spell is **countered** (delegates to `Effect::CounterSpell`). This mirrors the
existing Ward construction in `builder.rs` L396-412 (`MayPayOrElse { payer:
ControllerOf(DeclaredTarget{0}), or_else: CounterSpell{DeclaredTarget{0}} }`), which also
always counters in the deterministic path. Because it reuses `Effect::CounterSpell`, it
inherits the correct flashback-exile-at-counter behavior (gotcha in gotchas-rules.md:
Flashback must exile at ALL four departure points, incl. the CounterSpell effect path).

**Field-shape deviation from the brief**: the brief writes `{ cost }`. This plan uses
`{ target: EffectTarget, cost: Cost }` — the target is required to know which stack object
to counter. All roster cards use `target: EffectTarget::DeclaredTarget { index: 0 }`
(spell_pierce, izzet mode 0, etc.). The `payer` is not stored (it is derivable as
`ControllerOf(target)` when M10+ interactive payment lands); the `cost` field is retained
for M10+ and for hash/display.

---

## Why this batch is unusually low-risk (surface map)

`Effect` is a plain enum executed in exactly **two** exhaustive engine match sites, plus
its constructor uses in card defs:

| Site | File | Action |
|------|------|--------|
| Execution | `crates/engine/src/effects/mod.rs` (`execute_effect_inner`, MayPayOrElse arm at ~L2992) | add 2 arms + 1 helper |
| Hash | `crates/engine/src/state/hash.rs` (`impl HashInto for Effect`, block ends L5954) | add 2 arms + bump version |
| Enum def | `crates/engine/src/cards/card_definition.rs` (`pub enum Effect`, MayPayOrElse at L1596) | add 2 variants |

**No TUI / replay-viewer changes.** `tools/tui/.../stack_view.rs` and
`tools/replay-viewer/.../view_model.rs` match on `StackObjectKind` and `KeywordAbility`,
**not** `Effect`. Verified: grep for `Effect::MayPayOrElse`/`CounterSpell`/`SacrificePermanents`
across the engine returns only `hash.rs`, `builder.rs` (constructor), and `effects/mod.rs`.

**No KW / AbilDef / SOK discriminant-chain changes.** No `KeywordAbility`,
`AbilityDefinition`, or `StackObjectKind` variants are added. The discriminant chain
(KW 158, AbilDef 55, SOK ~20) is untouched. `helpers.rs` prelude already exports `Cost`,
`PlayerTarget`, `EffectTarget`, `Effect` — no prelude additions needed.

**Hash discriminants** (verified current max Effect discriminant = 87, `UntapAll`,
hash.rs L5950; `HASH_SCHEMA_VERSION = 28` at hash.rs L213):
- `MayPayThenEffect` → **88**
- `CounterUnlessPays` → **89**
- Bump `HASH_SCHEMA_VERSION` **28 → 29** and update the parity test
  `assert_eq!(HASH_SCHEMA_VERSION, 29)` (per conventions.md Hash sentinel convention).

---

## CR Rule Text (verified via MCP)

**118.12** — "[Do something]. If [a player] [does, doesn't, or can't], [effect]." Or
"[A player] may [do something]. If [that player] [does, doesn't, or can't], [effect]."
The action [do something] is a cost, paid when the spell or ability resolves. The
"If [a player] [does...]" clause checks whether the player chose to pay an optional cost
or started to pay a mandatory cost, regardless of what events actually occurred.

**118.12a** — "[Do something] unless [a player does something else]" means the same thing
as "[A player may do something else]. If [that player doesn't], [do something]."

**118.8 / 118.8b** — Additional costs; some additional costs are optional.

**701.5a** — To cast a spell is to take it from its zone, put it on the stack, and pay its
costs. (Counter target spell removes it from the stack, CR 701.5 family.)

**119.4** — A player can pay an amount of life greater than 0 only if their life total is
greater than or equal to that amount.

**603.2** — Triggered abilities (the wrapper riders live inside triggered-ability effects).

---

## Engine Changes

### Change 1 — Add two `Effect` variants

**File**: `crates/engine/src/cards/card_definition.rs` (in `pub enum Effect`, adjacent to
`MayPayOrElse` at L1596)
**Action**:

```rust
/// CR 118.12: "[player] may pay [cost]. If they do, [then]." Beneficial optional cost.
/// Counterpart to MayPayOrElse (CR 118.12a tax semantics). `then` runs ONLY IF the cost
/// is paid. Deterministic non-interactive path: the payer pays when able (life >= n, a
/// matching permanent to sacrifice, a card to discard, or floating mana), then `then`
/// runs; otherwise nothing happens. Cost is paid at resolution (CR 118.12). Interactive
/// pay-vs-decline choice deferred to M10+.
MayPayThenEffect {
    cost: Cost,
    payer: PlayerTarget,
    then: Box<Effect>,
},
/// CR 118.12a / CR 701.5: "Counter target spell unless its controller pays [cost]."
/// Equivalent to "controller may pay [cost]; if they don't, counter [target]."
/// Deterministic non-interactive path: controller does NOT pay -> [target] is countered
/// (delegates to Effect::CounterSpell, inheriting flashback-exile-at-counter). Interactive
/// payment deferred to M10+. `cost` retained for M10+ / hashing / display.
CounterUnlessPays {
    target: EffectTarget,
    cost: Cost,
},
```

### Change 2 — Execution arms + payment helper

**File**: `crates/engine/src/effects/mod.rs` (in `execute_effect_inner`, next to the
`Effect::MayPayOrElse` arm at L2992)
**CR**: 118.12 / 118.12a
**Action**:

```rust
// CR 118.12: beneficial optional cost. Deterministic path: pay when able, then run `then`.
Effect::MayPayThenEffect { cost, payer, then } => {
    let payer_ids = resolve_player_target_list(state, payer, ctx);
    for pid in payer_ids {
        if try_pay_optional_cost(state, pid, cost, events) {
            execute_effect_inner(state, then, ctx, events);
        }
    }
}
// CR 118.12a: counter target spell unless its controller pays. Deterministic path:
// controller declines -> counter. Delegates to CounterSpell (flashback-exile safe).
Effect::CounterUnlessPays { target, cost: _ } => {
    execute_effect_inner(
        state,
        &Effect::CounterSpell { target: target.clone() },
        ctx,
        events,
    );
}
```

New private helper in `effects/mod.rs`:

```rust
/// CR 118.12 / 118.8: attempt to pay an optional cost non-interactively (deterministic).
/// Returns true and mutates state iff the cost was fully paid. Reference impls:
///  - Mana:      casting.rs `can_pay_cost` / `pay_cost` against player.mana_pool
///  - PayLife:   CR 119.4 (life >= n); deduct as the LoseLife effect does
///  - DiscardCard: reuse DiscardCards effect machinery; deterministic = lowest ObjectId
///  - Sacrifice: reuse the SacrificePermanents effect zone-move (effects/mod.rs ~L3008),
///               layer-resolved filter (CR 613.1d), deterministic = lowest ObjectId,
///               fires "dies" triggers normally
///  - Sequence:  payable iff ALL sub-costs payable; check all, then pay all
///  - other Cost variants (Tap/SacrificeSelf/ExileSelf/Forage/RemoveCounter/DiscardSelf):
///    return false (out of PB-AC2 scope — none are on the roster)
fn try_pay_optional_cost(
    state: &mut GameState,
    pid: PlayerId,
    cost: &Cost,
    events: &mut Vec<GameEvent>,
) -> bool { /* ... */ }
```

Implementation notes for the runner:
- **Sequence must be all-or-nothing**: do a payability pre-check across every sub-cost
  before mutating any state, then pay. (Miara = `Sequence[Mana{1}, PayLife(1)]`.)
- For `Sacrifice`/`DiscardCard`, factor the selection+move so it reuses the existing
  zone-move code — do NOT hand-roll a second sacrifice path (dies triggers must fire).
- Emit the same events the existing sacrifice/discard/life/mana-spend paths emit so
  downstream triggers (e.g., a separate "whenever you sacrifice" watcher) behave.

### Change 3 — Hash arms + version bump

**File**: `crates/engine/src/state/hash.rs`
**Action**: in `impl HashInto for Effect` (block ends L5954), before the closing brace:

```rust
// PB-AC2: MayPayThenEffect (discriminant 88) — CR 118.12
Effect::MayPayThenEffect { cost, payer, then } => {
    88u8.hash_into(hasher);
    cost.hash_into(hasher);
    payer.hash_into(hasher);
    then.hash_into(hasher);
}
// PB-AC2: CounterUnlessPays (discriminant 89) — CR 118.12a
Effect::CounterUnlessPays { target, cost } => {
    89u8.hash_into(hasher);
    target.hash_into(hasher);
    cost.hash_into(hasher);
}
```

And at hash.rs L213: `pub const HASH_SCHEMA_VERSION: u8 = 29;` (was 28). Update the parity
test assertion to `assert_eq!(HASH_SCHEMA_VERSION, 29)` and note the bump in the impl
commit message + the hash module comment (conventions.md Hash bump rule).

### Change 4 — Exhaustive-match sweep (mandatory verification)

After Changes 1-3, run `cargo build --workspace`. The compiler will flag any additional
exhaustive `match` on `Effect` that this plan did not anticipate (none expected — the grep
found only `effects/mod.rs` and `hash.rs`). **Do not add a `_ =>` catch-all** to silence a
missing arm; add the explicit arm. TUI/replay-viewer should build unchanged.

---

## Card Definition Fixes (backfill)

### Confirmed CLEAN — beneficial-pay (`MayPayThenEffect`), 7 cards

| File | Oracle rider | Cost | `then` | Trigger (already expressible) |
|------|--------------|------|--------|-------------------------------|
| `crossway_troublemakers.rs` | may pay 2 life → draw | `PayLife(2)` | `DrawCards(1)` | `WheneverCreatureDies{Vampire, exclude_self:false}` — currently omitted entirely |
| `miara_thorn_of_the_glade.rs` | may pay {1} & 1 life → draw | `Sequence[Mana{1}, PayLife(1)]` | `DrawCards(1)` | `WheneverCreatureDies{Elf, exclude_self:false}` — currently omitted; keep Partner keyword |
| `hazorets_monument.rs` | may discard → draw | `DiscardCard` | `DrawCards(1)` | `WheneverYouCastSpell{spell_type_filter:[Creature]}` — **currently authored as unconditional draw (WRONG game state); wrap it** |
| `tainted_observer.rs` | may pay {2} → proliferate | `Mana{2}` | `Effect::Proliferate` | `WheneverCreatureEntersBattlefield{controller_you:true, exclude_self:true}` (ETBTriggerFilter, Batch 12) |
| `springbloom_druid.rs` | ETB: may sac a land → search 2 basics tapped, shuffle | `Sacrifice(land filter)` | `SearchLibrary(≤2 basics, tapped, shuffle)` | `WhenEntersBattlefield` — **TODO-sweep forced-add (not in brief)** |
| `nadir_kraken.rs` | on draw, may pay {1} → +1/+1 counter + 1/1 Tentacle | `Mana{1}` | `Sequence[AddCounter(Source,+1/+1), CreateToken(1/1 blue Tentacle)]` | `WheneverYouDrawACard` (exists) — **TODO-sweep forced-add** |
| `ezuri_stalker_of_spheres.rs` | ETB: may pay {3} → proliferate twice | `Mana{3}` | `Sequence[Proliferate, Proliferate]` | `WhenEntersBattlefield`; 2nd ability "whenever you proliferate, draw" uses `ControllerProliferates` trigger (exists) — **TODO-sweep forced-add** |

### Confirmed CLEAN — counter-tax (`CounterUnlessPays`), 6 cards

| File | Oracle | Effect | Target requirement |
|------|--------|--------|--------------------|
| `mana_leak.rs` | counter unless pays {3} | `CounterUnlessPays{target:DeclaredTarget{0}, cost:Mana{3}}` | `TargetSpellWithFilter(default)` (currently `abilities: vec![]` — author the Spell ability) |
| `mana_tithe.rs` | counter unless pays {1} | `...cost:Mana{1}` | `TargetSpellWithFilter(default)` (currently `abilities: vec![]`) |
| `spell_pierce.rs` | counter noncreature unless pays {2} | `...cost:Mana{2}` | `TargetSpellWithFilter{non_creature:true}` — replace existing `CounterSpell` |
| `flusterstorm.rs` | counter instant/sorcery unless pays {1} | `...cost:Mana{1}` | `TargetSpellWithFilter{has_card_types:[Instant,Sorcery]}` — replace `CounterSpell`; keep Storm |
| `make_disappear.rs` | counter unless pays {2} | `...cost:Mana{2}` | `TargetSpellWithFilter(default)` — keep Casualty(1) |
| `izzet_charm.rs` | mode 0: counter noncreature unless pays {2} | `...cost:Mana{2}` at modes[0] | replace mode-0 `CounterSpell`; modes 1/2 already correct |

### Improved but still PARTIAL (author the now-expressible part; leave a precise ENGINE-BLOCKED marker for the rest)

| File | PB-AC2 unblocks | Remaining blocker |
|------|-----------------|-------------------|
| `stubborn_denial.rs` | base `CounterUnlessPays{Mana{1}}` | Ferocious upgrade ("counter instead if you control a power-4+ creature") needs a power-≥N control Condition — verify existence; if absent, wrap base in `Conditional` only when the condition exists, else keep PARTIAL |
| `leaf_crowned_visionary.rs` | `MayPayThenEffect{Mana{G}→Draw}` | **Elf spell-SUBTYPE filter** on `WheneverYouCastSpell` (PB-AC7 gap) — do NOT author the trigger without it (would draw on every spell) |
| `call_of_the_ring.rs` | `MayPayThenEffect{PayLife(2)→Draw}` | `TriggerCondition::WhenRingBearerChosen` does not exist (out of scope) |
| `ruthless_technomancer.rs` | ETB `MayPayThenEffect{Sacrifice(creature)→...}` shape | `then` amount = sacrificed creature's power (needs `sacrificed_creature_powers` ctx wired through the optional-sacrifice path) AND a second activated ability (variable-X sac + graveyard power filter) — leave BLOCKED/PARTIAL |
| `vampire_gourmand.rs` | `MayPayThenEffect{Sacrifice(creature)→Draw+...}` | "this can't be blocked this turn" grant + self-attack trigger — verify; likely PARTIAL |
| `mana_vault.rs` | upkeep `MayPayThenEffect{Mana{4}→UntapPermanent(self)}` | interacts with `DoesNotUntap` (PB-AC1) + the counter/damage clause — verify holistically; likely PARTIAL |
| `temur_sabertooth.rs` | — | **BLOCKED, out of scope**: the optional action is "return another creature to hand" (a bounce), which is not a `Cost` variant. Not a beneficial-*pay*. Leave as-is with a corrected marker. |

### Reflexive / other-family cards found by the TODO sweep — NOT PB-AC2

These matched the sweep grep but are a **different** primitive (reflexive "**When you do**"
triggers, or opponent-pays `MayPayOrElse` which already exists). Record and skip:
`ruthless_lawbringer.rs`, `sorin_imperious_bloodlord.rs`, `caesar_legions_emperor.rs`,
`ziatora_the_incinerator.rs`, `dokuchi_silencer.rs`, `rings_of_brighthearth.rs`
(reflexive/copy-ability); `smothering_tithe.rs`, `mystic_remora.rs`, `esper_sentinel.rs`,
`kazuul_tyrant_of_the_cliffs.rs` (opponent-pays, existing `MayPayOrElse` family).

**TODO sweep result**: run of `grep -E "(TODO|ENGINE-BLOCKED).*(if you do|may pay|may sac|
may discard|beneficial|MayPayThen|optional-pay)"` and `grep "unless .* pays"` over
`crates/engine/src/cards/defs/`. Beyond the brief's 8+6, it forced-added **springbloom_druid,
nadir_kraken, ezuri_stalker_of_spheres** (clean) and **stubborn_denial** (partial) to the
roster. All four are verified via MCP oracle lookup. Not a 0-result sweep — recorded here
per the roster-recall gate.

---

## New Card Definitions

None. All 13 clean cards already have def files (some with `abilities: vec![]` stubs to be
filled). No missing-file authoring in this batch.

---

## Unit Tests

**File**: `crates/engine/tests/optional_cost_and_counter_tax.rs` (new)
**Pattern**: follow the Ward tests (search `tests/` for `ward` / `MayPayOrElse`) and the
counterspell tests (`tests/` for `counter`), using `GameStateBuilder`.

Beneficial-pay (`MayPayThenEffect`):
- `test_may_pay_then_effect_paylife_pays_and_runs` — payer life=20, `PayLife(2)→Draw`;
  assert 1 card drawn AND life == 18. (CR 118.12, 119.4)
- `test_may_pay_then_effect_paylife_insufficient_declines` — payer life=1, `PayLife(2)→Draw`;
  assert 0 cards drawn AND life unchanged. (CR 119.4 negative)
- `test_may_pay_then_effect_discard_pays_and_runs` — hand has 2 cards, `DiscardCard→Draw`;
  assert net hand change reflects one discard + one draw and a card left graveyard.
- `test_may_pay_then_effect_discard_empty_hand_declines` — empty hand; assert `then` skipped.
- `test_may_pay_then_effect_sacrifice_pays_and_runs` — controls a matching creature;
  `Sacrifice(creature)→Draw`; assert creature in graveyard, card drawn, dies-trigger fired.
- `test_may_pay_then_effect_sacrifice_none_declines` — no matching permanent; `then` skipped.
- `test_may_pay_then_effect_mana_requires_floating` — pre-float {2}; `Mana{2}→Proliferate`;
  assert proliferate happened and pool emptied. Companion negative:
  `test_may_pay_then_effect_mana_empty_pool_declines` — pool empty → `then` skipped.
- `test_may_pay_then_effect_sequence_all_or_nothing` — `Sequence[Mana{1},PayLife(1)]`:
  (a) with {1} floating + life≥1 → pays both, `then` runs; (b) with life≥1 but no mana →
  pays neither (no partial life loss), `then` skipped. (CR 118.12 — cost is atomic.)

Counter-tax (`CounterUnlessPays`):
- `test_counter_unless_pays_counters_when_declined` — target spell on stack;
  `CounterUnlessPays{DeclaredTarget{0}, Mana{3}}`; assert target countered (left stack to
  graveyard). (CR 118.12a)
- `test_counter_unless_pays_flashback_exiles` — target is a flashback-cast spell; assert it
  is **exiled**, not put in graveyard, when countered (regression for the CounterSpell reuse
  path — gotchas-rules.md flashback-exile-at-all-4-points).
- `test_counter_unless_pays_noncreature_filter` — spell_pierce-shape: legal noncreature
  target counters; a creature spell is not a legal target (target validation).

Hash:
- `test_hash_schema_version_is_29` — `assert_eq!(HASH_SCHEMA_VERSION, 29)`.
- `test_hash_distinguishes_new_effect_variants` — two `Effect`s differing only by
  `MayPayThenEffect` vs `CounterUnlessPays` (and by cost) hash differently.

Card integration (one per representative cost kind + one counter):
- `test_crossway_troublemakers_vampire_death_may_pay_life_draws` (PayLife on death trigger)
- `test_hazorets_monument_creature_cast_may_discard_draws` (DiscardCard on cast trigger;
  regression: it must NOT draw unconditionally as it does today)
- `test_springbloom_druid_etb_may_sacrifice_land_searches` (Sacrifice on ETB)
- `test_nadir_kraken_on_draw_may_pay_puts_counter_and_token` (Mana with pre-floated {1})
- `test_mana_leak_counters_target_spell` (CounterUnlessPays end-to-end)

---

## Verification Checklist

- [ ] `Effect::MayPayThenEffect` + `Effect::CounterUnlessPays` added to `card_definition.rs`
- [ ] Execution arms + `try_pay_optional_cost` helper added to `effects/mod.rs`
      (Mana/PayLife/DiscardCard/Sacrifice/Sequence supported; Sequence atomic)
- [ ] Hash arms 88/89 added; `HASH_SCHEMA_VERSION` bumped 28→29; parity test updated
- [ ] `cargo check` — engine compiles
- [ ] `cargo build --workspace` — TUI + replay-viewer build unchanged (no Effect match there)
- [ ] 13 clean card defs authored/fixed; hazoret's unconditional-draw wrong-state removed
- [ ] PARTIAL cards left with precise ENGINE-BLOCKED markers (no wrong game state)
- [ ] `cargo test --all` green (incl. new test file)
- [ ] `cargo clippy -- -D warnings` (watch the unused `cost` in CounterUnlessPays arm — bind
      as `cost: _` or `let _ = cost;`)
- [ ] `cargo fmt --check`
- [ ] No remaining `TODO`/`ENGINE-BLOCKED` referencing MayPayThenEffect/CounterUnlessPays in
      the 13 clean defs

---

## Risks & Edge Cases

- **Mana-cost beneficial pays fire only with floating mana.** Because pools empty between
  steps (CR 500.4), tainted_observer/nadir/ezuri/miara's mana portions decline in normal
  deterministic play. This is CR-legal (a player who can't pay doesn't). Tests must
  pre-float mana. Document in each affected def so a future reader doesn't mistake the
  decline for a bug.
- **Auto-pay-when-able can drain life over repeated triggers** (crossway pays 2 life on
  every Vampire death). Legal, deterministic, observable — not wrong state — but note it as
  a known deterministic-modeling artifact until M10+ interactivity.
- **Sequence atomicity**: paying `Mana{1}` then failing `PayLife(1)` must NOT leave the mana
  spent. Pre-check all sub-costs before mutating. (Covered by a test.)
- **Sacrifice-as-cost must fire dies triggers** and use layer-resolved filters. Reuse the
  `SacrificePermanents` zone-move; do not hand-roll.
- **CounterUnlessPays reuses CounterSpell** → must preserve flashback exile (all 4 counter
  departure points). Covered by a regression test.
- **Hash-field omission** was PB-AC1's sole HIGH finding. Both new variants hash **every**
  field (MayPayThenEffect: cost+payer+then; CounterUnlessPays: target+cost). Do not drop the
  `cost` field on CounterUnlessPays from the hash even though execution ignores it.
- **stubborn_denial Ferocious** and **leaf_crowned Elf-spell filter** are second-order gaps
  (PB-AC7 territory); do not extend scope to close them (conventions.md
  implement-phase default-to-defer). Author the PB-AC2-expressible part only.
- **Yield reality**: brief estimated ~20; verified clean roster is **13**. Of the brief's 8
  beneficial cards only 4 are clean (crossway, miara, hazoret, tainted); leaf/call/
  ruthless/temur stay PARTIAL/BLOCKED. All 6 brief counter-tax cards are clean. TODO sweep
  added 3 clean beneficial cards not in the brief.
