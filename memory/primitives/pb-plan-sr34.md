# Primitive Batch Plan: SR-34 — Composite-cost mana abilities (CR 605.1a)

**Generated**: 2026-07-17
**Task**: `scutemob-90` | **Branch**: `feat/sr-34-mana-abilities-with-additional-costs-are-never-registe`
**Primitive**: `ManaAbility` gains a mana component and a life component to its activation
cost; `handle_tap_for_mana` gains a cost-payment step; `enrich_spec_from_def`'s
`matches!(cost, Cost::Tap)` gate widens to any cost whose components are all payable
without a `Command` channel.
**CR Rules**: 605.1a, 605.3a, 605.3b, 605.3c, 602.2, 602.2b, 601.2f, 601.2h, 118.3,
118.3a/b/c, 119.4, 119.4b, 106.12, 732
**Cards affected**: roster arrives separately in `memory/primitives/sr34-affected-defs.md`
(enumerated from the compiled registry, never source text). This plan does not enumerate cards.
**Dependencies**: SR-33 (`953cc5a6`) — the `tainted_field` one-ability-per-colour pattern and
`tests/core/effect_choose_gate.rs` both land first and this task builds on them.
**Origin**: SF-1 in `memory/card-authoring/sr33-engine-findings-2026-07-17.md` (empirically
proven). SF-3 and SF-6 are entangled; see §9.

---

## 0. Read this first — two research findings that change the shape of the task

Both were found by walking the dispatch chain (per `feedback_verify_full_chain`), not by
reading names. Both are load-bearing for §3 and §8.

### Finding A — widening the gate naively **breaks Cabal Coffers**

`try_as_tap_mana_ability` (`replay_harness.rs:3669`) has an arm for `Effect::AddManaScaled`
that registers `produces = {color: 1}` and calls it, in its own comment, *"a marker; actual
production is dynamic"*. **Nothing makes it dynamic.** `handle_tap_for_mana` step 8
(`rules/mana.rs:256-282`) reads only `ability.any_color` and `ability.produces`; there is no
`AddManaScaled` branch anywhere in the handler. The dynamic evaluation lives exclusively in
`effects/mod.rs:2162`, which is reachable **only** through stack resolution — i.e. only via
`ActivateAbility`.

Consequence for this task: `Cabal Coffers` (`{2}, {T}: Add {B} for each Swamp you control`,
`Cost::Sequence([Mana, Tap])` + `AddManaScaled`) is **correct today only because the
`Cost::Tap` gate excludes it** and routes it through the stack. Widening the gate captures it
and demotes it from *correct* to *exactly one black mana*. So `AddManaScaled` must be
**actively excluded** from the widened gate by a named predicate — not left out by omission,
because omission is not how the gate works.

### Finding B — `Gaea's Cradle` already taps for exactly 1 green (new finding, **SF-8**)

The same chain, on the `Cost::Tap` side of the gate, is a live HIGH bug today. `gaeas_cradle.rs`
is `Cost::Tap` + `AddManaScaled{ color: Green, count: PermanentCount{Creature} }`. It is
registered as `ManaAbility { produces: {Green: 1} }`, excluded from `activated_abilities` by
`is_tap_mana_ability`, and `TapForMana` therefore adds **one** green mana regardless of board.
Six defs are in this class (`Elvish Archdruid`, `Priest of Titania`, `Marwyn, the Nurturer`,
`Circle of Dreams Druid`, `Gaea's Cradle`, `Howlsquad Heavy`; plus `Crypt of Agadeem`,
`Black Market`, `Everflowing Chalice`, `Elvish Guidance`, `Brightstone Ritual`,
`Battle Hymn` to be re-checked against the registry).

It is pinned by two tests that cannot see it — `tests/casting/mana_filter.rs:292` and `:338`
assert only `!mana_abilities.is_empty()` and that the marker's key is Green. Neither ever
activates the ability. **This is SF-5's anti-pattern verbatim** ("a data-model test can pin a
defect as a requirement"), and it is the tenth-plus consecutive finding whose sharpest result
is a hole in a checker. The SR-33 colour gate cannot see it either: `printed_tap_mana_colors`
extracts colours, not amounts, so `{T}: Add {G} for each creature` reports `printed={Green}`,
`registered={Green}`, pass.

**SF-8 is OUT OF SCOPE here** (see §8) — it is a different primitive (evaluating an
`EffectAmount` inside the stackless `TapForMana` path, which needs a resolution context
`handle_tap_for_mana` does not have). File it; do not fix it inline. But it is why the
`AddManaScaled` exclusion in §3 is a documented, revisitable seam rather than a silent skip:
when SF-8 lands, deleting the exclusion is what re-includes Cabal Coffers.

---

## 1. CR basis

### CR 605.1a — cost is irrelevant to the classification

> **605.1.** Some activated abilities and some triggered abilities are mana abilities, which
> are subject to special rules. Only abilities that meet either of the following two sets of
> criteria are mana abilities, regardless of what other effects they may generate or what
> timing restrictions (such as "Activate only as an instant") they may have.
>
> **605.1a** An activated ability is a mana ability if it meets all of the following criteria:
> it doesn't require a target (see rule 115.6), it could add mana to a player's mana pool when
> it resolves, and it's not a loyalty ability. (See rule 606, "Loyalty Abilities.")

Three criteria: **no target**, **could add mana**, **not loyalty**. Cost is not among them.
Confirmed — the engine's `matches!(cost, Cost::Tap)` gate has no CR basis.

Note also what 605.1 says about timing restrictions: an activation restriction does **not**
disqualify a mana ability. This matters for §7 risk 3.

### CR 605.3a / 605.3b — what the fix buys

> **605.3.** Activating an activated mana ability follows the rules for activating any other
> activated ability (see rule 602.2), with the following exceptions:
>
> **605.3a** A player may activate an activated mana ability whenever they have priority,
> whenever they are casting a spell or activating an ability that requires a mana payment, or
> whenever a rule or effect asks for a mana payment, even if it's in the middle of casting or
> resolving a spell or activating or resolving an ability.
>
> **605.3b** An activated mana ability doesn't go on the stack, so it can't be targeted,
> countered, or otherwise responded to. Rather, it resolves immediately after it is activated.
> (See rule 405.6c.)
>
> **605.3c** Once a player begins to activate a mana ability, that ability can't be activated
> again until it has resolved.

### CR 119.4 — life payment legality (the brief asked; do not assume)

> **119.4.** If a cost or effect allows a player to pay an amount of life **greater than 0**,
> the player may do so only if their life total is greater than or equal to the amount of the
> payment. If a player pays life, the payment is subtracted from their life total; in other
> words, the player loses that much life.
>
> **119.4b** Players can always pay 0 life, no matter what their (or their team's) life total
> is, and even if an effect says players can't pay life.

Two exact requirements, both easy to get wrong:
1. The predicate is `life_total >= amount`, **not** `life_total > amount`. A player at exactly
   1 life **can** pay 1 life and go to 0. (The SBA then kills them — CR 704.5a — which is
   correct and is a separate event.)
2. 119.4b makes `amount == 0` **unconditionally legal**. A naive `life_total >= 0` check is
   wrong at a negative life total (reachable transiently before SBAs run). The check must
   short-circuit on `amount == 0`. See §5 failure mode 2.

**CR 118.4 is not the rule the brief named** — 118.4 is *"Some costs include an {X} or an X.
See rule 107.3."* The "can't pay what you can't pay" rule is **CR 118.3**:

> **118.3.** A player can't pay a cost without having the necessary resources to pay it fully.
> For example, a player with only 1 life can't pay a cost of 2 life, and a permanent that's
> already tapped can't be tapped to pay a cost.
>
> **118.3a** Paying mana is done by removing the indicated mana from a player's mana pool.
> (Players can always pay 0 mana.)
> **118.3b** Paying life is done by subtracting the indicated amount of life from a player's
> life total. (Players can always pay 0 life.)
> **118.3c** Activating mana abilities is not mandatory, even if paying a cost is.

Cite **118.3**, not 118.4, throughout. (Note `events.rs:366` already miscites 118.4 on
`GameEvent::LifeLost`; do not propagate, do not fix inline.)

### CR 602.2 / 601.2h — cost payment order

> **602.2b** The remainder of the process for activating an ability is identical to the process
> for casting a spell listed in rules 601.2b–i. […] An activated ability's analog to a spell's
> mana cost (as referenced in rule 601.2f) is its activation cost.
>
> **601.2h** The player pays the total cost. First, they pay all costs that don't involve
> random elements or moving objects from the library to a public zone, in any order. Then they
> pay all remaining costs in any order. Partial payments are not allowed. Unpayable costs can't
> be paid.

**"in any order"** — so the relative order of tap / mana / life / sacrifice inside
`handle_tap_for_mana` is a free choice under the CR, and no observable game state depends on
it (none of these components' legality depends on another's completion). §4 chooses on
engineering grounds, not CR grounds, and says so.

> **602.2** […] If, at any point during the activation of an ability, a player is unable to
> comply with any of those steps, the activation is illegal; the game returns to the moment
> before that ability started to be activated (see rule 732 […]).

The engine gets CR 732 **for free**: `process_command` (`rules/engine.rs:67`) takes
`GameState` **by value** and returns it only on `Ok`. An `Err` drops the partially-mutated
state, and the caller's copy is the pre-command state. So a mutation before a later failure is
unobservable. Validate-before-mutate is still the recommended discipline (§4) for
debuggability, but it is not load-bearing for rules correctness — say so rather than claiming
a safety property the code does not need.

### CR 106.12 — does a composite cost change any "tap for mana" step?

> **106.12.** To "tap [a permanent] for mana" is to activate a mana ability of that permanent
> that includes the {T} symbol in its **activation cost**.
>
> **106.12a** An ability that triggers whenever a permanent "is tapped for mana" […] triggers
> whenever such a mana ability resolves and produces mana […].
> **106.12b** A replacement effect that applies if a permanent "is tapped for mana" […]
> modifies the mana production event while such an ability is resolving […].

**No.** 106.12 asks only whether `{T}` is *included in* the activation cost — not whether it
*is* the activation cost. `{1}, {T}: Add {C}{C}` and `{T}, Pay 1 life: Add {B}` both include
`{T}` and are both "tapping for mana". The three `ability.requires_tap`-gated steps in
`handle_tap_for_mana` — 7b (replacements, CR 106.12b), 8 (production), 10
(`fire_mana_triggered_abilities`, CR 106.12a) — are therefore **correct as written** and need
**no change**: `requires_tap` is exactly the 106.12 predicate, and it stays true for every
composite cost in the roster. Do not touch them.

This is a positive finding: the widened gate makes Signets and horizon lands newly *eligible*
for Caged Sun / Nyxbloom Ancient multiplication and for "whenever you tap a land for mana"
triggers — which they should always have been (CR 106.12a/b) and were not, because they never
entered this function at all.

### Recursion: can a mana ability with a mana cost recurse?

**No recursion exists in this architecture, and none is needed.** CR 605.3a permits activating
a mana ability *during* a cost payment, so a Signet's `{1}` may legally come from another mana
ability. But in this engine a mana ability is a **`Command`**, and a mana pool persists between
Commands. The player issues `TapForMana(land)` → pool holds `{G}` → then `TapForMana(signet)`,
which finds `{G}` already in the pool. `handle_tap_for_mana` never calls itself and never needs
to ask for mana mid-execution.

CR 605.3c ("can't be activated again until it has resolved") is satisfied vacuously: the
ability resolves within the same Command that activates it, so there is no window in which it
is activated-but-unresolved.

**No loop risk.** A `{1}, {T}: Add {C}{C}` Signet is net-positive but requires a fresh untapped
source per activation; the tap cost bounds the loop at the number of permanents. There is no
CR 104.4b concern and `handle_tap_for_mana` needs no loop detection.

**Scoping honesty**: full CR 605.3a — activating a mana ability *in the middle of* paying a
cost — is **not expressible** in this Command model, because `CastSpell` pays from the pool
atomically inside one Command. The reachable payload of this fix is: *no stack, no priority
window for opponents, and usable in the same priority window as the cast it funds.* That is
what §6's tests assert. Do not write a test that claims a mid-payment interleave; the model
does not have one, and a test that pretends otherwise is the SF-5 pattern again.

---

## 2. The `ManaAbility` shape

### Current declaration (`crates/card-types/src/state/game_object.rs:164`)

```rust
pub struct ManaAbility {
    pub produces: OrdMap<ManaColor, u32>,
    pub requires_tap: bool,
    pub sacrifice_self: bool,
    pub any_color: bool,
    pub damage_to_controller: u32,
}
```

Derives `Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize`. The `Default` derive is
load-bearing for the migration (§3, step 1).

### CHOSEN: two narrow fields

```rust
    /// CR 602.2b / CR 601.2f: mana component of this mana ability's activation cost.
    /// `{1}, {T}: Add {C}{C}` (Signets, Cluestones) and `{W/B}, {T}: Add ...` (filter lands).
    /// Paid from the controller's pool at activation (CR 118.3a), before mana is produced.
    /// CR 605.3a permits the payer to have filled the pool from another mana ability first.
    #[serde(default)]
    pub mana_cost: Option<ManaCost>,

    /// CR 119.4 / CR 118.3b: life component of this mana ability's activation cost.
    /// `{T}, Pay 1 life: Add {B}` (horizon lands). Legal only if `life_total >= life_cost`;
    /// CR 119.4b makes a cost of 0 always legal, so the check must short-circuit on 0.
    #[serde(default)]
    pub life_cost: u32,
```

`ManaCost` is already imported in `game_object.rs` (it is declared there) and is already in
both the SR-8 protocol closure and the `GameState` serde closure, so this adds **no new type**
to either — only two fields to an existing one.

### REJECTED: `pub cost: ActivationCost` (reuse the existing cost type)

Four reasons, in order of weight.

1. **It mints the exact defect this task exists to remove.** `ActivationCost`
   (`game_object.rs:239`) carries `discard_card`, `sacrifice_filter`, `remove_counter_cost`,
   `discard_self`, `forage`, `exile_self`, `exert`. Three of those need a caller-supplied
   `ObjectId` — `handle_activate_ability` gets them from `Command::ActivateAbility { discard_card,
   sacrifice_target, x_value, .. }`. **`Command::TapForMana { player, source, ability_index }`
   has no such channel.** So embedding `ActivationCost` creates a type most of whose inhabitants
   `handle_tap_for_mana` structurally cannot honour, and it would honour them by *silently
   ignoring them*. That is SF-3 / SR-33 / SF-1's defect class precisely: a field that looks
   like a cost and is never paid, passing every gate. We would be shipping the disease in the
   cure.
2. **Two sources of truth for two fields.** `ActivationCost` already has `requires_tap` and
   `sacrifice_self`, and so does `ManaAbility` — where both are already honoured, by
   `handle_tap_for_mana` steps 6 and 7, and where `requires_tap` is additionally the CR 106.12
   predicate read by steps 7b/8/10. Embedding forces a choice between duplicating them
   (divergence hazard at every read site) or migrating them out (a corpus-wide edit of every
   `ManaAbility` literal, every `LayerModification::AddManaAbility` grant site, and both
   constructors, for no behaviour gain).
3. **`ActivationCost` does not even cover the roster.** It has **no life field** —
   `flatten_cost_into` (`replay_harness.rs:3774`) maps `Cost::PayLife(_) => {}` with the
   comment *"no ActivationCost representation yet"*. Reuse would require extending it anyway
   (and that extension changes non-mana activated abilities corpus-wide — see SF-9 in §8).
4. **Smaller wire and hash delta.** `ManaAbility` is inside `Characteristics`, so it sits in
   **both** the SR-8 `PROTOCOL_SCHEMA_FINGERPRINT` closure (`Characteristics` is a
   `CLOSURE_MUST_CONTAIN` entry, `tests/core/protocol_schema.rs:101`) and the `GameState` serde
   closure behind `HASH_SCHEMA_VERSION`. Both bump either way (§7), but two scalar fields is a
   smaller and more legible delta than a nested 10-field struct, and every field is one the
   `HashInto` impl can feed honestly.

### The falsifier for this choice, stated up front

Two narrow fields is right **because the roster's cost shapes are exactly `{N}` mana and
`Pay N life`.** The moment a def needs a mana ability with a *discard* or *sacrifice-another*
component — the real example is **Krark-Clan Ironworks**, `Sacrifice an artifact: Add {C}{C}`,
which is a mana ability by CR 605.1a and needs an `ObjectId` — narrow fields stop scaling and
the answer is to **widen `Command::TapForMana`**, not to retrofit `ActivationCost`. That card
is out of scope (§8) and this is the seam where it would come back.

### Also add, for symmetry with the existing constructors

`ManaAbility::tap_for` and `ManaAbility::treasure` (`game_object.rs:191`, `:204`) build
all-fields literals and will not compile. Add `..Default::default()` to both rather than
listing the two new fields; that is the pattern that stops the next field addition from
touching them again.

---

## 3. Ordered implementation steps

### Step 1 — extend the struct

**File**: `crates/card-types/src/state/game_object.rs`
**Function/decl**: `struct ManaAbility` (L164-188), `impl ManaAbility` (L189-213)
**Action**: add `mana_cost: Option<ManaCost>` and `life_cost: u32` per §2, each `#[serde(default)]`
(so an older serialized `GameState` still deserializes) and each carrying its CR citation.
Convert `tap_for` and `treasure` to `..Default::default()`.
Also rewrite the struct doc comment — it currently says *"For M3-A, only tap-activated mana
abilities are supported […] Future milestones will add additional cost components (pay life,
sacrifice a permanent, etc.)"*. This task **is** that future milestone for two of the three;
leave an accurate note about which remain (§8).

**SR-6 invariant**: `card-types` may not reference `GameState`. Both new fields are pure data
(`ManaCost` is declared in this same file). No violation. Do not add a method here that takes
a state.

**Migration**: the derive is `Default`, so most literals are unaffected. The all-fields literals
that **will** fail to compile, from an exhaustive grep:
- `crates/engine/src/testing/replay_harness.rs:3642, 3659, 3672, 3721` (4 sites, inside
  `try_as_tap_mana_ability` / `mana_pool_to_ability`)
- `crates/card-types/src/state/game_object.rs:191, 204` (the two constructors)

Card defs and `LayerModification::AddManaAbility` grant sites (`enduring_vitality`,
`paradise_mantle`, `cryptolith_rite`, `chromatic_lantern`, `vraska_betrayals_sting`,
`wrenn_and_realmbreaker`, `awakening_zone`, `sifter_of_skulls`, `pawn_of_ulamog`, plus
`tests/rules/grant_activated_ability.rs` ×4 and `tests/rules/layer_correctness.rs:425`) use
`..Default::default()` — verify with `cargo check --workspace`, do not assume.

### Step 2 — hash the new fields

**File**: `crates/engine/src/state/hash.rs`
**Function**: `impl HashInto for ManaAbility` (L1386-1394)
**Action**: feed `self.mana_cost` and `self.life_cost` after `damage_to_controller`. Two states
differing only in a mana ability's cost must not hash identically.

`tests/core/hash_schema.rs::every_hashed_struct_field_is_hashed_or_allowlisted` (SR-19) will
fail until you do — that is the gate working. Do not allowlist.

### Step 3 — pay the cost in the mana-ability path

**File**: `crates/engine/src/rules/mana.rs`
**Function**: `handle_tap_for_mana` (L29)
**Action**: two new steps. Exact placement and its justification are §4.

- **New step 5b — cost legality check** (after the ability is fetched at step 5 / L131-138,
  before the tap at step 6 / L141). Pure validation, no mutation:
  - if `ability.mana_cost` is `Some(mc)` and `mc.mana_value() > 0`:
    `state.player(player)?.mana_pool.can_spend(mc, None)` → else `Err(InsufficientMana)`.
  - if `ability.life_cost > 0`: `state.player(player)?.life_total >= ability.life_cost as i32`
    → else the new error (§5). **The `> 0` guard is CR 119.4b and is mandatory**, not an
    optimisation — see §5 failure mode 2.
- **New step 6b — pay** (after the tap at L167, before the SR-28 snapshot at L181):
  - mana: `state.player_mut(player)?.mana_pool.spend(mc, None)`; push
    `GameEvent::ManaCostPaid { player, cost: mc.clone() }`.
  - life: `state.player_mut(player)?.life_total -= ability.life_cost as i32`; push
    `GameEvent::LifeLost { player, amount: ability.life_cost }` (CR 119.4: *"the player loses
    that much life"*).

**Reuse, do not reinvent.** These are the exact helpers `handle_activate_ability`
(`rules/abilities.rs:524-573`) uses:
- `ManaPool::can_spend(&ManaCost, Option<&SpellContext>) -> bool` (`card-types/src/state/player.rs:148`)
- `ManaPool::spend(&ManaCost, Option<&SpellContext>)` (`:185`)
- `GameEvent::ManaCostPaid { player, cost }`
- `GameEvent::LifeLost { player, amount }` (`rules/events.rs:367`) — **already exists**, reuse it.

Pass `None` for `SpellContext`: restricted mana (CR 106.12's "spend only on X") is keyed to a
spell, and a mana ability's cost is not a spell. This matches `handle_activate_ability:564`.

**Do NOT** add an `{X}` path (no `x_value` on `Command::TapForMana`), a cost-reduction lookup
(`get_self_activated_reduction` — no card in the roster has one; adding it is unverified
scope), or any `requires_tap`-gated logic (§1, CR 106.12: steps 7b/8/10 are already right).

### Step 4 — widen the lowering gate

**File**: `crates/engine/src/testing/replay_harness.rs`
**Function**: `enrich_spec_from_def`, the mana-ability loop at **L2115-2123**
**Action**: replace `if matches!(cost, Cost::Tap)` with a new named predicate, e.g.

```rust
/// CR 605.1a: an activated ability is a mana ability by what it does, not what it costs.
/// Returns the ability's cost components iff *every* component is one this engine can pay
/// through `Command::TapForMana { player, source, ability_index }` — which carries no
/// ObjectId channel, so a discard / sacrifice-another / remove-counter component is not
/// lowerable and the ability must stay on the stack. Returns None if any component is not.
fn mana_ability_cost_components(cost: &Cost) -> Option<ManaAbilityCost>
```

accepting `Cost::Tap`, `Cost::Mana(_)`, `Cost::PayLife(_)`, `Cost::SacrificeSelf`, and
`Cost::Sequence` of those; returning `None` for anything else. **`Cost::SacrificeSelf` is
already honoured** by `handle_tap_for_mana` step 7 and `ManaAbility::sacrifice_self` — include
it (a bare `Cost::SacrificeSelf` mana ability is a mana ability by CR 605.1a too), but verify
against the roster before claiming yield.

Then merge the components into the `ManaAbility` returned by `try_as_tap_mana_ability`.

**The `AddManaScaled` exclusion (Finding A).** `try_as_tap_mana_ability`'s `AddManaScaled` arm
(L3667-3679) must be refused when the cost is *not* bare `Cost::Tap`. Concretely: keep the
existing `Cost::Tap` + `AddManaScaled` behaviour exactly as-is (it is broken — SF-8 — but it is
broken *today* and un-breaking it is a different primitive), and make the **widened** path
reject `AddManaScaled` with a comment naming SF-8 and stating that deleting this exclusion is
the correct move once SF-8 lands. Cabal Coffers / Cabal Stronghold / Crypt of Agadeem stay on
the stack — a CR 605.1a violation, but "right mana, wrong mechanism", which is strictly better
than the "wrong mana" this task would otherwise ship. **Write a test that pins the exclusion**
(§6, T10) — an exclusion nobody executes is a comment.

### Step 5 — widen the exclusion list in lockstep (SF-6)

**File**: `crates/engine/src/testing/replay_harness.rs`
**Function**: `enrich_spec_from_def`, `is_tap_mana_ability` at **L2141-2149**
**Action**: this must use **the same predicate** as step 4. It currently duplicates the logic
as a `matches!` over effect variants OR'd with `try_as_tap_mana_ability(effect).is_some()`.

**These two lists already disagree**, and the plan should not preserve that:
`Effect::AddManaMatchingType` is in the `matches!` at L2148 but has **no arm** in
`try_as_tap_mana_ability`. A `Cost::Tap` + `AddManaMatchingType` activated ability would
therefore be excluded from `activated_abilities` *and* absent from `mana_abilities` —
**silently gone**. No live victim today (its only users, `zendikar_resurgent.rs` and
`miraris_wake.rs`, are `AbilityDefinition::Triggered`, which this loop never sees), which is
why it has survived. Since the runner is editing both lists anyway: **collapse them into one
call** — `is_tap_mana_ability` should be *exactly* "the mana-ability lowering would have
succeeded", i.e. `mana_ability_cost_components(cost).is_some() && try_as_tap_mana_ability(effect).is_some()`
— so the two can never disagree again. Drop the redundant `matches!` OR-arm; if that changes
any def's registration, that def was relying on the divergence and the change is the fix.
Pin it with T11 (§6).

### Step 6 — find and fix everything that depends on the old `ability_index` (SF-6)

Widening the gate **moves abilities out of `activated_abilities`**, so every non-mana activated
ability that sat *after* a composite-cost mana ability shifts **down**. Worked example —
`Fiery Islet` after its §9 rewrite:

| | before SR-34 | after SR-34 |
|---|---|---|
| `mana_abilities` | `[]` | `[{T},Pay 1: {U}]`, `[{T},Pay 1: {R}]` |
| `activated_abilities` | `[0]={U} arm, [1]={R} arm, [2]=Draw` | `[0]=Draw` |

A `Command::ActivateAbility { ability_index: 2 }` written against the old shape now returns
`InvalidAbilityIndex` — but the dangerous case is the **silent** one: an index that still
exists and now names a *different* ability. That is SF-6's whole warning.

**Do all three, in order** (grep alone is not sufficient — SR-33 used it and it worked, but
only because it also had the roster):
1. Wait for `memory/primitives/sr34-affected-defs.md`. For each name, grep `crates/engine/tests/`
   and `test-data/generated-scripts/` for the **card name** (not the index).
2. Grep for `ActivateAbility` and `activate_ability` across `crates/engine/tests/` and
   `test-data/` and cross-reference against the roster.
3. **Mechanical sweep** — the only exhaustive one. Write a throwaway probe that walks
   `all_cards()`, runs `enrich_spec_from_def`, and prints `(name, mana_abilities.len(),
   activated_abilities.len())`; diff it across the change. Any def whose
   `activated_abilities.len()` drops **and is non-zero after** is an index-shift candidate.
   Delete the probe before commit (a probe left behind is a test nobody wrote).
4. Note `tests/casting/mana_filter.rs:111-118` **hardcodes** Fetid Heath's filter ability at
   `activated_ability index 0` and will move to `mana_abilities`. That file needs rewriting
   regardless — see §9.

### Step 7 — version bumps

Per §7. Do this **last**, in one commit with the code, reading the recomputed digests out of the
gate failure text. Never edit an existing `PROTOCOL_HISTORY` / `HASH_SCHEMA_HISTORY` row.

### Step 8 — card defs

Roster-driven; arrives separately. §9 covers the one class this plan can specify (horizon lands).
Per SF-7: `cargo fmt` reaches **zero** card defs. Run `rustfmt` over each touched def **by name**,
and re-check the file afterwards — rustfmt exits **0** while silently abandoning a macro body
that exceeds `max_width`.

---

## 4. Where exactly the cost step goes, and why

```
  1.  priority check                          (CR 605.3a)
  1b. restrictions (Stony Silence, …)         (CR 605.3)
  2.  clone source
  3.  battlefield check
  4.  controller check
  5.  fetch ability via layer-resolved chars  (CR 613.1f)
+ 5b. COST LEGALITY: mana + life              (CR 118.3, 119.4)   ← new, pure validation
  6.  tap: already-tapped + summoning sickness, then tap   (CR 118.3, 302.6)
+ 6b. PAY: mana, then life                    (CR 601.2h, 118.3a/b)  ← new, mutation
  ─── SR-28 snapshot: source_pre_cost_chars ───────────────  (CR 106.12a/b, 603.10a)
  7.  sacrifice cost                          (CR 602.2c)   ← the zone-changing component
  7b. mana-production replacements            (CR 106.12b)
  8.  add mana                                (CR 605)
  8b. additive mana                           (CR 106.6a)
  9.  pain damage
  10. tapped-for-mana triggers                (CR 106.12a)
  11. retain priority                         (CR 605.5)
```

**Why 5b and 6b are split.** Not for CR 732 — the engine gets rollback free from
`process_command` taking `GameState` by value (§1), so a mutation before a failure is
unobservable. The split is for the *ordering* CR 601.2h licenses but does not require: with
validation hoisted, a Signet activated on an empty pool returns `InsufficientMana` having
touched nothing, so a debugger, a `debug_assert`, or a future in-place-mutation refactor all
see a clean transaction. It costs one extra `state.player()` read.

**Why 6b sits between the tap (6) and the SR-28 snapshot.**

- **Against CR 602.2c / 601.2h**: *"they pay all costs that don't involve random elements or
  moving objects from the library to a public zone, **in any order**"*. Tap, mana, life and
  sacrifice-self are all in that first group. The CR imposes **no** order among them, and none
  of their legality predicates reads another's result (mana-pool contents, life total, tapped
  status and zone are independent). So **any** position before step 8 is CR-correct, and 6b
  vs. 7-then-6b is observationally identical today. **This is a choice made on invariant
  shape, not on rules, and the plan says so rather than manufacturing a CR justification.**
- **Against the SR-28 snapshot comment** (`mana.rs:168-181`), which states its contract
  precisely: the snapshot is *"taken after the {T} tap (step 6) — tapping does not change
  type/subtype/color — and before any zone change"*, because a `{T}`+Sacrifice source is *"a
  dead ObjectId (CR 400.7) by the time"* steps 7b and 10 read it. Paying mana or life changes
  the **controller's** pool and life total; it touches neither the source's characteristics nor
  its zone. So the snapshot is **byte-identical** on either side of 6b, and SR-28's invariant
  is not at risk either way.
- **The tiebreak**: placing 6b *before* the snapshot makes the comment's claim structurally
  true rather than incidentally true — the snapshot then sits exactly at the boundary between
  "cost components that cannot move the source" and "the one that can". The alternative
  (6b after step 7) leaves a cost component below the boundary and makes SR-28's invariant a
  fact about which components happen to exist today, re-argued every time one is added. Given
  Krark-Clan Ironworks (§2) is the named future extension and it *does* move an object, the
  boundary should be drawn where it will still hold.

**Extend the SR-28 comment** to say the boundary is now load-bearing for a third reason, so
the next person does not slide a zone-changing cost above it.

---

## 5. Failure modes

| # | Condition | Result | CR | Where |
|---|---|---|---|---|
| 1 | Insufficient mana in pool | `GameStateError::InsufficientMana` (existing, `error.rs:49`) | 118.3, 601.2h ("Unpayable costs can't be paid") | new step 5b, before any mutation |
| 2 | Insufficient life | **new** `GameStateError::InsufficientLife { player, required, actual }` | 119.4 | new step 5b |
| 3 | Already tapped | `GameStateError::PermanentAlreadyTapped(source)` (existing, `error.rs:39`) | 118.3 ("a permanent that's already tapped can't be tapped to pay a cost") | step 6, **unchanged** |
| 4 | Summoning sickness | `GameStateError::InvalidCommand("object {:?} has summoning sickness and cannot tap for mana (no haste)")` (existing) | 302.6, 702.10 | step 6, **unchanged** |
| 5 | Ability index out of range | `GameStateError::InvalidAbilityIndex { object_id, index }` (existing) | — | step 5, **unchanged** |
| 6 | Not priority holder | `GameStateError::NotPriorityHolder { expected, actual }` (existing) | 605.3a | step 1, **unchanged** |
| 7 | Stony Silence / Grand Abolisher | `GameStateError::InvalidCommand("restriction: …")` (existing) | 605.3 | step 1b, **unchanged** |

Notes on the two that need care:

**#2 — the new error.** Prefer a typed variant over `InvalidCommand(String)`: `InvalidCommand`
is the untyped bucket, and this is a rules-defined illegality a client will want to branch on
(the UI must grey out a horizon land at 0 life). Carry `required` and `actual` so the message
is actionable. **Before adding it, verify `GameStateError` is not on the SR-8 wire** —
`PROTOCOL_ROOTS` is `["Command", "GameEvent", "ReplayLog"]` (`tests/core/protocol_schema.rs:74`)
and an error is returned, not embedded, so it should be outside the closure; but confirm by
running the gate rather than by reading this sentence.

**#2 — the CR 119.4b short-circuit is mandatory.** The check is:

```rust
if ability.life_cost > 0 && state.player(player)?.life_total < ability.life_cost as i32 {
    return Err(GameStateError::InsufficientLife { .. });
}
```

**not** `life_total >= life_cost as i32` unguarded. CR 119.4 constrains only payments *greater
than 0*, and CR 119.4b makes 0 always legal *"no matter what their life total is"*. A player at
**-1 life** (reachable transiently — life totals go negative before the CR 704.5a SBA runs) must
still be able to activate a `life_cost: 0` mana ability. The unguarded form returns
`InsufficientLife` there, which is wrong. `life_cost` is `u32`, so this is not hypothetical
defensive coding — every ability in the corpus with no life component has `life_cost: 0` and
takes this branch on every activation.

**#2 — the boundary is `>=`, not `>`.** A player at exactly 1 life **can** pay 1 life (CR 119.4:
*"greater than or equal to"*). They go to 0 and die to the SBA. That is correct, and T5 pins it.

**#3 vs the new checks.** `PermanentAlreadyTapped` fires at step 6, *after* the 5b cost check.
So a Signet that is both tapped and unaffordable returns `InsufficientMana`, not
`PermanentAlreadyTapped`. Both are correct (CR 118.3 covers both, and CR 601.2h lets costs be
evaluated in any order); the *specific* error on a doubly-illegal activation is not a rules
question. Do not write a test that pins which one wins — it would pin an arbitrary choice as a
requirement (SF-5).

---

## 6. Tests

**File**: `crates/engine/tests/primitives/primitive_sr34_composite_mana_costs.rs`
(new module; add `mod primitive_sr34_composite_mana_costs;` to
`crates/engine/tests/primitives/main.rs` — SR-9a: a file in a group dir with no `mod` line is
**not compiled and its tests silently cease to exist**, and `tests/no_stray_test_binaries.rs`
fails on it. Never add a top-level `tests/*.rs`.)

**Pattern**: follow `tests/core/effect_choose_gate.rs` (SR-33) — it is the closest analogue and
already has the `defs_map()` / `find_by_name` / `pool_amount` helpers and, crucially, the
end-to-end "activate it and assert the mana that comes out" shape that SF-5 demands. Copy that
shape, not `mana_filter.rs`'s.

| # | Test | Asserts | CR |
|---|---|---|---|
| T1 | `signet_registers_a_mana_ability_not_an_activated_ability` | a `{1},{T}: Add {C}{C}` def lowers to `mana_abilities.len()==1`, `activated_abilities` does not contain it | 605.1a |
| T2 | `signet_tap_for_mana_pays_generic_and_produces_two` | pool `{G}` → `TapForMana` → pool has 2 colourless, 0 green; `ManaCostPaid` emitted | 605.1a, 118.3a |
| T3 | `signet_tap_for_mana_does_not_use_the_stack` | `state.stack_objects().is_empty()` after activation | **605.3b** |
| T4 | `signet_with_empty_pool_is_insufficient_mana` | `Err(InsufficientMana)`; source **still untapped** in the caller's pre-command state | 118.3, 601.2h |
| T5 | `horizon_land_pays_life_and_at_exactly_one_life_is_legal` | at 1 life, `TapForMana` succeeds, life → 0, `LifeLost{amount:1}` emitted | **119.4** (`>=`, not `>`) |
| T6 | `horizon_land_at_zero_life_cannot_pay` | at 0 life, `Err(InsufficientLife)`, land untapped | 119.4 |
| T7 | `zero_life_cost_ability_is_legal_at_negative_life` | a `life_cost: 0` mana ability activates at life `-1` — **the CR 119.4b short-circuit; this test fails on the unguarded `>=` form** | **119.4b** |
| T8 | `mana_ability_funds_a_spell_in_the_same_priority_window` | `TapForMana(land)` → `TapForMana(signet)` → `CastSpell`; the stack held **only** the spell, never a mana ability. This is the CR 605.3b payload as this Command model can express it (§1) — and it is the one that shows a Signet doing what a Signet is for | 605.3a, 605.3b |
| T9 | `signet_mana_cost_can_be_paid_from_another_mana_ability` | the `{1}` comes from a land tapped in a prior Command; no recursion, no loop | 605.3a |
| T10 | `composite_cost_add_mana_scaled_stays_on_the_stack` | **pins Finding A's exclusion.** Cabal Coffers registers **0** mana abilities and 1 activated ability. Comment: delete this test when SF-8 lands | 605.1a (knowingly violated; see §8) |
| T11 | `is_tap_mana_ability_agrees_with_the_lowering` | for every `all_cards()` def: an `Activated` ability excluded from `activated_abilities` **is** in `mana_abilities`, and vice versa. Pins step 5's collapse and kills the `AddManaMatchingType` divergence class permanently | 605.1a |
| T12 | `composite_cost_mana_source_is_tapped_for_mana` | a Signet under Caged Sun / Nyxbloom Ancient gets multiplied, and a "whenever you tap a land for mana" trigger fires off a horizon land — proves §1's claim that the `requires_tap`-gated steps 7b/8/10 now correctly apply to composite costs | **106.12, 106.12a, 106.12b** |
| T13 | `sr34_gates_are_not_vacuous` | a synthetic def with a bare `Cost::Tap` still lowers; one with `Cost::DiscardCard`+`Tap` does **not**. Both directions | — |

**T12 is the highest-value test in the list** and the easiest to skip: it is the only one that
proves the widened gate delivers CR 106.12's *consequences* and not just its classification.
SR-33's lesson is that a card can be registered correctly and still do the wrong thing.

**Do not write**: a test that pins which error wins on a doubly-illegal activation (§5); a test
that claims a mid-cost-payment interleave (§1 — the model has none); any test that asserts only
a def's *shape* without activating it (that is SF-5, and it is what let Finding B live).

---

## 7. Version-bump analysis

Read from `crates/engine/src/rules/protocol.rs`, `crates/engine/src/state/hash.rs`, and
`crates/engine/tests/core/protocol_schema.rs`, not guessed.

### `PROTOCOL_SCHEMA_FINGERPRINT` — **MOVES**

`ManaAbility` is in the closure. `PROTOCOL_ROOTS = ["Command", "GameEvent", "ReplayLog"]`
(`protocol_schema.rs:74`); the walk reaches `Characteristics` — which is an explicit
`CLOSURE_MUST_CONTAIN` entry (`:98-101`) — and `Characteristics.mana_abilities: Vec<ManaAbility>`
pulls `ManaAbility` in. Adding two fields is a declaration change, so the blake3 of the
normalized declaration text moves. `tests/core/protocol_schema.rs` fails and prints the new digest.

### `PROTOCOL_VERSION` — **BUMPS 2 → 3**

This is a real wire-shape change (not one of the documented no-bump exceptions: the closure
definition is unchanged — no new scan root, protocol root, or `EXTERNAL_TYPES` entry). Per the
append-only procedure at `protocol.rs:123-133`, in **one commit**:
1. `PROTOCOL_VERSION: u32 = 3` and add a `- 3: SR-34 (2026-07-17) — …` History line at `:52-67`;
2. **append** a `ProtocolEpoch { version: 3, fingerprint: "<recomputed>" }` to
   `PROTOCOL_HISTORY` (`:138`) and set `PROTOCOL_SCHEMA_FINGERPRINT` (`:85`) to the same value;
3. update `BASELINE_*`? **No** — the baseline row is FROZEN. Update the
   `protocol_version_sentinel` and `FROZEN_HISTORY_PREFIX_DIGEST`
   (`protocol_schema.rs:130-148`) — version 2's row becomes frozen prefix, so the prefix digest
   moves.

Note `#[serde(default)]` on both new fields means an old serialized `Characteristics` still
deserializes — but strict lockstep (SR-8) rejects a v2 message regardless, by design. Do not
argue for skipping the bump on compatibility grounds; the policy is exact equality.

### `HASH_SCHEMA_VERSION` — **BUMPS 40 → 41**, and **both** fingerprints move

`ManaAbility` is inside `Characteristics` inside `GameObject` inside `GameState`. Both SR-17
axes move, independently and for different reasons:
- `decl_fingerprint`: the struct declaration gained two serde fields.
- `stream_fingerprint`: step 2 changes what `HashInto for ManaAbility` feeds — **and** the
  bump itself moves it regardless, since `public_state_hash` folds `HASH_SCHEMA_VERSION` in as
  its first byte (`hash.rs:365`).

Procedure: `HASH_SCHEMA_VERSION: u8 = 41` (`hash.rs:372`), a `- 41: SR-34 …` History entry
above it, **append** a `HashSchemaEpoch` row with both recomputed digests, and update the
**30 `assert_eq!(HASH_SCHEMA_VERSION, 40)` sentinels across 29 files** (grep for
`HASH_SCHEMA_VERSION, 40`; `tests/primitives/pbt_up_to_n_targets.rs` has **two**).

Caveat worth writing into the History entry, mirroring the v40 precedent: the canonical fixture
in `tests/core/hash_schema.rs` may not populate a `ManaAbility` with a non-default cost, in
which case the two new *feeds* are not themselves exercised by the digest. The bump is
mandatory regardless, per the checklist — any change to what `HashInto` feeds bumps. Consider
extending the fixture; note it in the entry either way.

### Summary

| Constant | Moves? | New value |
|---|---|---|
| `PROTOCOL_SCHEMA_FINGERPRINT` | yes | recompute from `protocol_schema.rs` failure text |
| `PROTOCOL_VERSION` | yes | `2 → 3` (+ `PROTOCOL_HISTORY` row, + `FROZEN_HISTORY_PREFIX_DIGEST`) |
| `HASH_SCHEMA_VERSION` | yes | `40 → 41` (+ `HASH_SCHEMA_HISTORY` row w/ both digests, + 30 sentinels) |

---

## 8. OUT OF SCOPE — explicit

1. **SF-3 (`Effect::AddManaChoice` needs a colour list) — NOT fixed.** The variant stays a stub
   and stays gated out of `Complete` by `tests/core/effect_choose_gate.rs`. The three horizon
   lands are instead rewritten as one ability per printed colour (§9). Do not reintroduce
   `AddManaChoice`; do not weaken the gate.

2. **`Cabal Coffers` / `Cabal Stronghold` / `Crypt of Agadeem` (`{2},{T}: Add {B} for each …`)
   — OUT, and actively excluded.** Not merely "not addressed": widening the gate *captures*
   them and would demote them from correct-via-stack to **exactly one black mana**, because
   `handle_tap_for_mana` has no `AddManaScaled` branch and reads the `produces: {B: 1}`
   "marker" literally (Finding A, §0). They keep using the stack — a CR 605.1a/605.3b
   violation, but "right mana, wrong mechanism", which is strictly better than the wrong mana
   this task would otherwise ship. Pinned by T10. Re-include by deleting the exclusion once
   SF-8 lands.

3. **SF-8 (new finding, HIGH) — `Cost::Tap` + `AddManaScaled` produces exactly 1 mana.**
   `Gaea's Cradle` taps for one green regardless of creature count; ~6 defs in the class,
   pinned by two shape-only tests (`tests/casting/mana_filter.rs:292, :338`) that never
   activate anything — SF-5's anti-pattern verbatim, and invisible to the SR-33 colour gate,
   which reads colours and not amounts. Fixing it means evaluating an `EffectAmount` inside the
   stackless path, which needs a resolution context `handle_tap_for_mana` does not have: a
   separate primitive. **File it in this task's findings doc; do not fix inline.**

4. **SF-9 (new finding) — `Cost::PayLife` is silently unpaid for *non-mana* activated
   abilities.** `flatten_cost_into` (`replay_harness.rs:3774`) maps `Cost::PayLife(_) => {}`
   ("no ActivationCost representation yet") and `ActivationCost` has no life field, so any
   activated ability with a life cost pays **nothing**. SR-34 fixes this only inside the
   `ManaAbility` path via `life_cost`; the general `handle_activate_ability` path is untouched.
   Cleanly separable (the two paths do not overlap) and needs its own roster. File it.

5. **`Command::TapForMana` payload is unchanged.** No `x_value`, no `sacrifice_target`, no
   `discard_card`. This is what bounds the widened gate (§3 step 4) — a cost component needing
   an `ObjectId` is not lowerable. Named consequence: **Krark-Clan Ironworks**
   (`Sacrifice an artifact: Add {C}{C}`) is a mana ability by CR 605.1a and stays on the stack.

6. **Hybrid mana cost enforcement.** `ManaPool::can_spend` / `spend`
   (`card-types/src/state/player.rs:148, :185`) read only `white/blue/black/red/green/colorless/generic`
   — **`hybrid` and `phyrexian` are ignored entirely**. So a filter land's `{W/B}` component
   costs nothing, before and after this task. Filter lands still improve (they become real mana
   abilities, off the stack, CR 605.3b) but their cost remains unenforced. Say this in
   `mana_filter.rs`'s rewritten note; do **not** claim filter lands are fixed. Pre-existing P4
   item, unchanged by SR-34.

7. **CR 305.6 / SF-2** (basic land subtypes granting intrinsic mana abilities) — untouched.

8. **Cost reductions and `{X}`** on mana abilities (§3 step 3).

9. **Interactive choice generally** — no `MakeChoice` Command, no `Effect::Choose`
   implementation. Per `memory/decisions.md` (2026-07-17), `TapForMana { ability_index }` **is**
   the choice channel for stackless mana abilities (CR 605.3b), and that decision stands.

10. **`ManaAbility` has no `activation_condition`, and this task does not add one (new finding,
    MEDIUM — SF-10).** `tainted_field.rs` authors `activation_condition:
    Some(Condition::ControlLandWithSubtypes([Swamp]))` on its `{W}`/`{B}` arms; the lowering
    loop at `replay_harness.rs:2116` destructures `AbilityDefinition::Activated { cost, effect, .. }`
    and the `..` **swallows it**, and `handle_tap_for_mana` never checks one. So Tainted Field
    taps for `{W}` with no Swamp. CR 605.1 is explicit that a restriction does not stop an
    ability being a mana ability, so the condition should be carried and enforced. **This is
    pre-existing** (Tainted Field is already lowered today, being bare `Cost::Tap`) and SR-34
    does not widen it — but do **not** "fix" it by refusing to lower abilities that carry a
    condition: that would push Tainted Field's arms back into `activated_abilities` and redden
    SR-33's `every_complete_land_registers_each_printed_tap_mana_color`. File it.

---

## 9. Entangled items from the origin findings

### SF-3 / the three horizon lands — rewrite, don't wait for the primitive

`Fiery Islet`, `Nurturing Peatland`, `Silent Clearing`. Oracle (Fiery Islet, MCP-confirmed):

```
{T}, Pay 1 life: Add {U} or {R}.
{1}, {T}, Sacrifice this land: Draw a card.
```

SR-33 demoted all three to `known_wrong`: they were `Complete` while modelling the "or" as
`Effect::AddManaChoice`, which adds **one `{C}`** — a colour they do not print. They were
blocked on SF-1 **and** SF-3. SR-34 removes the SF-1 half; the SF-3 half is removed **by
rewriting, not by fixing the stub** — one activated ability per printed colour, the
`tainted_field.rs` pattern (read it: `crates/card-defs/src/defs/tainted_field.rs`, three
`AbilityDefinition::Activated` arms, one per colour, with the "or" resolved by which
`ability_index` the player taps).

Result per land: two arms, each `cost: Cost::Sequence([Cost::Tap, Cost::PayLife(1)])`,
`effect: Effect::AddMana { player: Controller, mana: mana_pool(...) }` — plus the existing
`{1},{T},Sacrifice: Draw` arm, which stays an activated ability and **shifts to index 0**
(§3 step 6). All three then un-demote from `known_wrong` to `Complete`, which is this task's
headline card yield and reverses part of SR-33's 58.3% → 57.9% honesty adjustment.

Note the `{U} or {R}` arms have no `activation_condition`, so SF-10 (§8 item 10) does not bite
here.

### SF-6 — the index shift

§3 step 6. Non-negotiable; the failure mode is silent.

### `tests/casting/mana_filter.rs` — the note this task falsifies

Its module doc (L10-13) says:

> `//! CR 605.1a — activated mana abilities resolve immediately (no priority window).`
> `//!   (Filter lands use Cost::Sequence and go through ActivateAbility, which puts them`
> `//!   on the stack. Stack resolution yields the same final mana result.)`

**"Yields the same final mana result" was always the wrong bar** — CR 605.3b is not about the
final mana, it is about whether an opponent gets a window and whether the ability can be used
to fund a cast. That is precisely SF-1's indictment, and this note is where the gap was
recorded as acceptable. Rewrite it: filter lands are now mana abilities; the hybrid component
remains unenforced (§8 item 6); the tests that hardcode `ability_index: 0` for the filter
ability (L106-124) must move to `Command::TapForMana`.

**Also rewrite the two PB-34 tests at L292 and L338** — they are Finding B's cover (§0). Either
give them an activation and an assertion about the mana produced (at which point they **fail**,
correctly, and Gaea's Cradle's SF-8 defect surfaces — so instead: `#[ignore]` them with a
comment naming SF-8, or leave them and add the SF-8 note to their doc comment). **Do not
silently leave a test asserting `!mana_abilities.is_empty()` and calling that coverage.**
Decide explicitly and write down why.

### `tests/core/effect_choose_gate.rs` — extend the colour gate to composite costs

`printed_tap_mana_colors` (L249-288) documents exclusion (1) — *"The tap must be the whole
cost"* — justifying it thus:

> `{1}, {T}: Add {R}{W}` (a Signet) and `{T}, Pay 1 life: Add {B} or {G}` (a horizon land)
> are mana abilities by CR 605.1a, but `enrich_spec_from_def` only lowers `Cost::Tap` […] so
> they are *uniformly* unregistered regardless of how their colours are modelled. That is a
> real and separate gap (filed, see the SR-33 follow-ups) — folding it in here would make this
> gate assert a defect it is not about and cannot fix.

**SR-34 is that follow-up, and it falsifies the exclusion's premise.** The parser must widen to
match `Add ` clauses preceded by a cost prefix — `{N}, {T}: Add …`, `{T}, Pay N life: Add …` —
and the gate then covers Signets, Cluestones, horizon lands and filter lands. Update the doc
comment to record what the exclusion became rather than deleting it silently.

Keep exclusion (2) — the granted-ability quote check (`Citanul Hierophants`) — untouched; it is
orthogonal and still correct.

**The `AddManaScaled` blind spot must be written into that gate's doc**: it reads *colours*, not
*amounts*, so `{T}: Add {G} for each creature` passes while producing 1 (Finding B / SF-8).
A gate whose blind spot is undocumented is how SF-8 survived; do not let it survive twice.

---

## 10. Verification checklist

- [ ] `cargo check --workspace` — catches the 6 all-fields `ManaAbility` literals (§3 step 1)
- [ ] `cargo build --workspace` — **the only gate that proves the SR-3 `GameState` seal**;
      `test --all` and `clippy --all-targets` enable `test-util` workspace-wide via feature
      unification and will not catch a seal break
- [ ] `cargo test --all` — baseline **3284**, expect > 3284
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo fmt --check` **and** `rustfmt <each touched def>.rs` **by name**, then re-inspect
      the file (SF-7: `cargo fmt` reaches zero defs; `rustfmt` exits 0 while abandoning an
      over-`max_width` macro body)
- [ ] `tests/core/protocol_schema.rs` green after the v3 append (§7)
- [ ] `tests/core/hash_schema.rs` green after the v41 append + 30 sentinels (§7)
- [ ] `tests/core/effect_choose_gate.rs` green with the **widened** parser (§9)
- [ ] `tests/scripts/run_all_scripts.rs` green — the 210 approved scripts are where an SF-6
      index shift surfaces
- [ ] The mechanical before/after `(name, mana_abilities.len(), activated_abilities.len())`
      diff has been run and every non-zero shrink in `activated_abilities` accounted for
      (§3 step 6) — **and the probe deleted**
- [ ] Every def in `memory/primitives/sr34-affected-defs.md` triaged: fixed, or explicitly
      out-of-scope with a reason
- [ ] The three horizon lands un-demoted to `Complete` (§9)
- [ ] SF-8, SF-9, SF-10 filed in `memory/card-authoring/sr34-engine-findings-2026-07-17.md`
      — **not fixed inline**
- [ ] `memory/primitive-wip.md` updated; findings doc linked from CLAUDE.md's Current State

---

## 11. Risks

1. **The `AddManaScaled` capture (Finding A) is the top risk and it is silent.** A runner who
   widens the gate without the exclusion breaks Cabal Coffers, and **no existing test catches
   it** — `mana_filter.rs`'s scaled tests assert only that a `ManaAbility` exists, which after
   the naive widening becomes *more* true. T10 exists precisely for this.

2. **SF-6 index shift.** A stale `ActivateAbility { ability_index }` that still resolves to a
   *different* ability activates the wrong thing with no error. Mitigated by §3 step 6's three
   passes; the mechanical diff is the only exhaustive one.

3. **`activation_condition` is dropped on lowering (SF-10).** Pre-existing and not widened by
   this task, but the tempting "fix" (refuse to lower conditioned abilities) regresses
   `tainted_field` and reddens SR-33's colour gate. §8 item 10.

4. **Filter lands look fixed and are not.** They become real mana abilities while their `{W/B}`
   cost stays unenforced (hybrid is invisible to `can_spend`). If `mana_filter.rs`'s note is
   rewritten to "filter lands now work", this task ships a new false note in the exact file
   whose false note it was filed to correct.

5. **`{T}` is the CR 106.12 predicate and steps 7b/8/10 depend on it.** They are correct
   (§1) — a "cleanup" that re-gates them on "cost is exactly Tap" would break Caged Sun on a
   Signet. Leave them.

6. **Three version constants and 30+ sentinels in one commit.** Half-done bumps are worse than
   none: SR-27 exists because re-pinning a fingerprint without bumping the version lets two
   incompatible builds claim the same version and mis-decode each other silently. Do §7 last
   and completely, reading digests from failure text.

7. **The engine gets CR 732 from `process_command`'s by-value signature.** If that signature
   ever changes to `&mut GameState`, every partial mutation in this function becomes
   observable. Not a risk today; worth a comment at step 6b.

8. **Scope creep toward SF-8/SF-9.** Both are one-line-looking and neither is. SF-8 needs an
   `EffectAmount` resolution context inside a stackless path; SF-9 changes non-mana activated
   abilities corpus-wide. File, don't fix.
