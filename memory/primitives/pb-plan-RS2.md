# Primitive Batch Plan: PB-RS2 — Activated-Cost Hybrid/Phyrexian Pip Payment

<!-- last_updated: 2026-07-20 -->

**Generated**: 2026-07-20
**Task**: `scutemob-144`
**Branch**: `feat/pb-rs2-activated-cost-hybridphyrexian-pip-payment-every-such`
**Primitive**: hybrid (`{B/R}`, `{2/W}`) and Phyrexian (`{G/P}`, `{G/W/P}`) pips become *payable* —
and therefore *chargeable* — in **activated-ability** and **mana-ability** activation costs.
Adds payment-choice channels to `Command::ActivateAbility` **and** `Command::TapForMana`.
**CR Rules**: 107.4e, 107.4f, 202.3f, 202.3g, 601.2f, 602.2b, 605.1a, 118.3, 119.4, 119.4b
**Seeds**: OOS-RS-2 (primary) + OOS-OS8-1 (subsumed)
**Class**: CORRECTNESS, LIVE (silent undercharge on 7 shipped `known_wrong` filter lands)
**Cards affected**: 8 (7 repaired-in-place, 1 new flip)
**Honest discounted flip estimate**: **1** (`birthing_pod`). See §10.
**Dependencies**: PB-9 (`ManaCost.hybrid`/`.phyrexian` + `flatten_hybrid_phyrexian`),
PB-EF10/PB-OS8 (`min_cmc_amount`, `ManaValueOfSacrificedCreature`), SR-34 (mana-ability
composite costs), PB-EF12 (`TapForMana.chosen_color` — the exact schema precedent)
**Deferred items from prior PBs**: OOS-OS8-1 closes here. OOS-RS-6 (crucible dynamic-X),
hidden_strings optionality, and the filter-land fixed-mode simplification all stay open.
**Wire expectation**: **PROTOCOL 26 → 27** (machine-forced). **HASH stays 63** — justified §3.4.

---

## 0. Premise verification — what the brief got right, and two things it got wrong

Per `feedback_verify_cr_before_implement` and `feedback_verify_full_chain`, every hop of the
binding spec (`rider-seed-triage-2026-07-19.md` §2.2) was re-walked against source at HEAD.

### 0.1 CONFIRMED hops (line numbers accurate)

| Claim | Verified at | Verdict |
|---|---|---|
| `casting.rs:3990-3991` flattens before payment | `casting.rs:3988-3994` | **TRUE** |
| Phyrexian life deducted `:4015-4021` | `casting.rs:4014-4023` | **TRUE** |
| `abilities.rs:748-758` gates on `mana_value() > 0`, then `can_spend`/`spend` on raw cost | `abilities.rs:748-758` | **TRUE**, exact |
| `can_spend` (`player.rs:148-175`) / `spend` (`:185-206`) never read `cost.hybrid`/`.phyrexian` | `crates/card-types/src/state/player.rs:148-175`, `:185-206` | **TRUE**, exact |
| `mana_value()` counts hybrid + Phyrexian | `game_object.rs:133-154` (`hybrid_mv` `:142-149`, `phyrexian_mv` `:151`) | **TRUE** |
| `Command::ActivateAbility` has no choice fields | `crates/engine/src/rules/command.rs:78-103` | **TRUE** (span is `:78-103`, brief said `:78-102` — off by one, immaterial) |
| `CastSpell` has them | `command.rs:634-643` on `CastSpellData` | **TRUE** |
| 7 filter lands carry a hybrid pip in an ability cost | all 7 read; identical shape | **TRUE** |

The core defect is real and reproduces exactly as described: a pure `{B/R}` cost has
`mana_value() == 1`, passes the `> 0` gate, then `can_spend` evaluates six all-zero color
requirements plus `generic == 0` → `remaining >= 0` → `true`, and `spend` deducts nothing.

### 0.2 CORRECTION 1 (material, expands scope) — **the brief names the wrong handler for the 7 filter lands**

The brief routes the whole PB through `abilities.rs:748` (`handle_activate_ability`). But the
filter lands' `{B/R}, {T}: Add ...` ability is a **mana ability** (CR 605.1a — it produces mana,
doesn't target, isn't a loyalty ability), so it does **not** use the stack (CR 605.3) and is
**not** dispatched by `Command::ActivateAbility`. It is dispatched by `Command::TapForMana` →
`crates/engine/src/rules/mana.rs`.

`mana.rs` has the **identical, independent** defect:

- `mana.rs:213-220` — legality: `if mana_cost.mana_value() > 0 { ... can_spend(mana_cost, None) }`
- `mana.rs:304-313` — payment: `if mana_cost.mana_value() > 0 { ... spend(mana_cost, None) }`

Both take the **raw** `ability.mana_cost`. No flatten. Same free-pip outcome.

**Consequence**: a PB that fixes only `abilities.rs` would leave **all 7 filter lands — the entire
live-wrong roster the seed was filed for — still free.** The seed's own headline
("All 7 filter lands are live '{T}: Add two mana' lands today") would remain true after the fix.
This is the single most important correction in this plan.

**Scope impact**: `Command::TapForMana` needs the choice fields too. Its variant is small
(`command.rs:38-44`, 4 fields) and `chosen_color: Option<ManaColor>` (PB-EF12) is an exact
precedent for adding an optional choice channel to it.

### 0.3 CORRECTION 2 (material, narrows work) — **the "shared flatten helper" already exists and is already `pub`**

`memory/primitive-wip.md` step 2 and AC 5119 say "extract the `casting.rs` flatten logic into a
shared helper — no second open-coded copy." The logic is **already** a standalone free function:

```rust
// crates/engine/src/rules/casting.rs:6500
pub fn flatten_hybrid_phyrexian(
    cost: &ManaCost,
    hybrid_choices: &[HybridManaPayment],
    phyrexian_life_payments: &[bool],
) -> (ManaCost, u32)   // (flattened cost, life to pay)
```

It is pure, borrows nothing from `GameState`, is already `pub`, and `casting.rs:3991` calls it as
a free function (not inline) — **there is no borrow/ownership friction to resolve**, contrary to
the brief's anticipation. AC 5119 is satisfiable by *calling* it, not extracting it. The only open
question is whether to *relocate* it (§4).

### 0.4 CORRECTION 3 (CR-level) — **"the cast path is correct" is not quite true: it violates CR 119.4**

`casting.rs:4014-4023` deducts Phyrexian life with the comment *"CR 119.4: Life payment is a cost.
Life can go below 0 (SBA handles death)."* That reading is wrong. CR 119.4, verbatim:

> If a cost or effect allows a player to pay an amount of life greater than 0, the player may do so
> **only if their life total is greater than or equal to the amount of the payment.**

Phyrexian mana is exactly "a cost [that] allows a player to pay an amount of life" (CR 107.4f).
So a player at 1 life **may not** choose to pay 2 life for a `{G/P}`; today `casting.rs` lets them
and drops them to −1. (At exactly 2 life it *is* legal — 2 ≥ 2 — and lethal via SBA. Legal, and
the bots must not choose it; see §7.)

Note the same file gets this right 6 lines later for Bolas's Citadel (`:4029-4036` checks
`life_total < original_mana_value` and returns `InvalidCommand`), and `mana.rs:225-233` gets it
right for `life_cost`, and `abilities.rs:766-773` gets it **wrong** for `life_cost` (no check —
though SR-36/SG-1 pushed that check into `legal_actions.rs:440`, the engine itself does not
enforce it there).

**Disposition**: the new activate/mana-ability Phyrexian payment paths **must** enforce CR 119.4
from day one. Repairing `casting.rs:4014-4023` is a one-line-plus-guard change in the same
subsystem and should ride along (it is the shared helper's sibling); it is called out as an
explicit deliverable in §5.3 rather than left as a follow-up seed, because shipping a *new* path
that is stricter than the *existing* path would be an incoherent engine.

### 0.5 CORRECTION 4 (record) — **`drivnod_carnage_dominus.rs:43-44` is *technically* accurate, misleading in effect**

Verbatim at `:43-44`: *"The {B/P}{B/P} cost (PB-9) and the AddCounter-on-Source effect are both
already expressible."* The claim is about **DSL expressibility**, and that is true — `ManaCost`
has a `phyrexian` field and PB-9 added it. It is not the false claim the triage says it is.

What is misleading is the implication a reader draws: that authoring `{B/P}{B/P}` would produce
correct game state. It would not — it would produce a **free** ability. **Fix**: reword to
"expressible in the DSL, and (as of PB-RS2) actually charged at activation time." Drivnod does
**not** flip — its two real blockers (`Cost::ExileFromGraveyard`, `CounterType::Indestructible`)
are untouched by this PB. It stays `partial`.

### 0.6 Adjacent site inventory (residue-guard blast radius)

`can_spend`/`spend`/`can_pay_cost` reach the raw-cost pattern at **20 engine sites**. Full list,
because the §6 fail-loud guard fires at every one of them if a card ever carries a hybrid pip there:

| File:line | Path | Hybrid-carrying card today? |
|---|---|---|
| `casting.rs:4004`, `:6995`/`:7003` | spell cast | **flattened** ✅ |
| `abilities.rs:750`, `:753` | activated ability | **NO — fix here** |
| `mana.rs:216`, `:307` | mana ability | **NO — fix here (§0.2)** |
| `casting.rs:3882` | Assist | none |
| `plot.rs:118`, `suspend.rs:142`, `foretell.rs:86` | alt costs | none |
| `engine.rs:642` (echo), `:827`, `:1026` (recover), `:1318` (craft), `:1569`/`:1574`, `:2730` (level) | keyword costs | none |
| `abilities.rs:1398`, `:1638` (forecast), `:1816` (bloodrush), `:2006` (unearth), `:2215` (ninjutsu), `:2392` (embalm), `:2563` (eternalize), `:2741` (encore), `:9082` (scavenge) | keyword costs | none |
| `effects/mod.rs:8401` | affordability predicate | none |

Verified by roster inspection: of the **26** card defs containing `hybrid:` or `phyrexian:`, 19 put
it on the card's own `mana_cost` (cast path — already flattened) or on `AbilityDefinition::MutateCost`
(`brokkos_apex_of_forever:45`, `nethroi_apex_of_death:46` — an alt cast cost, flows through
`casting.rs`; **the runner must confirm the Mutate cost reaches the `casting.rs:3988` flatten and
not a bypass**), and **7** — the filter lands — put it on a mana-ability cost. `birthing_pod:24`
has one on `mana_cost` (already fine) and needs one on an activated cost (§8).

**Net**: the 18 non-fixed sites are provably hybrid-free today, so the guard is safe to add as a
`debug_assert` without a 20-site flatten campaign. It converts every one of them into a
compile-time-silent / test-time-loud landmine for the next author — which is the point.

---

## 1. CR Rule Text (verbatim, MCP-sourced)

**CR 107.4e** — "A hybrid mana symbol is also a colored mana symbol, even if one of its components
is colorless. Each one represents a cost that can be paid in one of two ways, as represented by the
two halves of the symbol. A hybrid symbol such as {W/U} can be paid with either white or blue mana,
and a monocolored hybrid symbol such as {2/B} can be paid with either one black mana or two mana of
any type. A hybrid mana symbol is all of its component colors."

**CR 107.4f** — "Phyrexian mana symbols are colored mana symbols: {W/P} is white, {U/P} is blue,
{B/P} is black, {R/P} is red, and {G/P} is green. A Phyrexian mana symbol represents a cost that
can be paid either with one mana of its color or by paying 2 life. There are also ten hybrid
Phyrexian mana symbols. A hybrid Phyrexian mana symbol represents a cost that can be paid with one
mana of either of its component colors or by paying 2 life. A hybrid Phyrexian mana symbol is both
of its component colors."

**CR 602.2b** — "The remainder of the process for activating an ability is identical to the process
for casting a spell listed in rules 601.2b–i. Those rules apply to activating an ability just as
they apply to casting a spell. **An activated ability's analog to a spell's mana cost (as referenced
in rule 601.2f) is its activation cost.**"

> This is the load-bearing citation for the entire PB: it is why the activation path is *required*
> to do everything `casting.rs` does with hybrid/Phyrexian pips. The engine's asymmetry is not a
> design choice; it is a CR 602.2b violation.

**CR 601.2f** — "The player determines the total cost of the spell. … Once the total cost is
determined, any effects that directly affect the total cost are applied. Then the resulting total
cost becomes 'locked in.' If effects would change the total cost after this time, they have no
effect."

**CR 119.4** — "If a cost or effect allows a player to pay an amount of life greater than 0, the
player may do so only if their life total is greater than or equal to the amount of the payment.
If a player pays life, the payment is subtracted from their life total; in other words, the player
loses that much life."

**CR 119.4b** — "Players can always pay 0 life, no matter what their (or their team's) life total
is, and even if an effect says players can't pay life."

**CR 202.3f / 202.3g** (already implemented at `game_object.rs:141-151`) — hybrid contributes its
largest component to mana value; each Phyrexian symbol contributes 1. **Unchanged by this PB** —
this is the rule that makes the `mana_value() > 0` gate pass for a free pip (§3.3).

---

## 2. Step 0 — the probe that must fail today (write FIRST, before any edit)

**File**: `crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs` (new; register in
`crates/engine/tests/primitives/mod.rs` — SR-9a: **never** create a top-level `tests/*.rs`).

Two probes, one per broken handler. Both must be written and **observed red-after-fix /
green-before-fix** before any engine edit.

### Probe A — activated (non-mana) ability, `abilities.rs` path

```
fn probe_hybrid_pip_is_currently_free_activated_ability()
```
- `GameStateBuilder`: P1 controls a permanent whose ability 0 is
  `AbilityDefinition::Activated { cost: Cost::Sequence([Cost::Mana(ManaCost { hybrid:
  vec![HybridMana::ColorColor(Black, Red)], ..Default::default() }), Cost::Tap]), effect:
  <a non-mana effect, e.g. DealDamage 1 to target player>, .. }`.
  It **must not** be a mana ability, or it routes to `TapForMana` instead (§0.2).
- P1's mana pool is **empty**.
- Issue `Command::ActivateAbility { ability_index: 0, .. }`.
- **Assert (pre-fix)**: `Ok(_)` — the ability activates for free. CR 107.4e violated.
- **Post-fix**: the same assertion is inverted to `Err(GameStateError::InsufficientMana)` and the
  test is renamed `hybrid_pip_in_activated_cost_requires_mana`. Keep permanently.

### Probe B — mana ability, `mana.rs` path (**the one that covers the 7 filter lands**)

```
fn probe_hybrid_pip_is_currently_free_mana_ability()
```
- Put a real `graven_cairns` on the battlefield (`enrich_spec_from_def` — see
  `memory/gotchas-infra.md`; `ObjectSpec::card()` alone produces a naked object with no abilities).
- Empty pool. Issue `Command::TapForMana { ability_index: 1, .. }` (index 1 = the `{B/R},{T}`
  filter ability; index 0 is the `{T}: Add {C}` ability).
- **Assert (pre-fix)**: `Ok(_)`, and P1's pool afterwards holds 1{B}+1{R} produced from nothing —
  a two-mana-from-zero profit. This is the live undercharge, on a shipped card, verbatim.
- **Post-fix**: `Err(InsufficientMana)` with an empty pool; and with `{B}` in pool and
  `hybrid_choices: vec![HybridManaPayment::Color(Black)]`, `Ok` with the `{B}` consumed.

**If Probe B passes (i.e. errors) before the fix, stop and re-scope** — that would mean the mana
path is charging somewhere this plan did not find, and §0.2 is wrong.

---

## 3. Engine Change 1 — Command schema

### 3.1 `Command::ActivateAbility` (`crates/engine/src/rules/command.rs:78-103`)

Add two fields at the end of the variant, mirroring `CastSpellData:634-643` **exactly** in type,
name, and default semantics:

```rust
        /// CR 107.4e (via CR 602.2b): for each hybrid pip in the resolved activation
        /// cost, how it was paid. Length must match the hybrid pip count after cost
        /// calculation. Empty = default to the first color option for each pip.
        #[serde(default)]
        hybrid_choices: Vec<crate::state::game_object::HybridManaPayment>,
        /// CR 107.4f (via CR 602.2b): for each Phyrexian pip, true = pay 2 life,
        /// false = pay mana. Empty = default to paying with mana for each pip.
        #[serde(default)]
        phyrexian_life_payments: Vec<bool>,
```

Types are re-exported from `card-types` (`HybridManaPayment` at `game_object.rs:99-104`) and are
already in the wire closure via `CastSpellData` — **the closure's type count does not grow.**

### 3.2 `Command::TapForMana` (`command.rs:38-44`) — required by §0.2

Same two fields, same `#[serde(default)]`, same doc shape, with the CR 605.1a framing (a mana
ability's activation cost per SR-34). `chosen_color: Option<ManaColor>` at `:43` is the shape
precedent: an optional choice channel with a documented default and an engine-side legality check.

### 3.3 Construction-site migration — **the dominant cost of this PB**

`Command::ActivateAbility` is an **enum struct variant**, so Rust's functional-update syntax
(`..Default::default()`) is **not available**. Every construction site must gain two literal lines.

Measured surface (`rg -c` at HEAD):

| Area | Files | `ActivateAbility` occurrences | `TapForMana` occurrences |
|---|---|---|---|
| Engine src | `rules/{command,abilities,engine,protocol}.rs`, `testing/replay_harness.rs` | 12 | ~10 |
| Simulator | `legal_actions.rs`, `random_bot.rs`, `heuristic_bot.rs`, `mana_solver.rs` | 9 | 6 |
| TUI | `play/input.rs`, `play/panels/action_menu.rs` | 4 | 1 |
| Card-types | `cards/card_definition.rs`, `state/game_object.rs` (doc refs only) | 2 | 2 |
| Engine tests | 45 files | ~236 | ~150 |
| **Total (code)** | **62 / 61 files** | **263** | **207** |

Occurrence counts include matches, patterns, and doc mentions; **actual struct literals needing two
new lines are roughly 200 (`ActivateAbility`) + 150 (`TapForMana`)**. The runner should get the
exact set with:

```
rg -n --multiline 'Command::ActivateAbility \{' crates tools
rg -n --multiline 'Command::TapForMana \{' crates tools
```

**Migration approach — two options; recommend (A):**

- **(A) RECOMMENDED — mechanical literal expansion.** Add `hybrid_choices: vec![],
  phyrexian_life_payments: vec![],` to every literal. Large diff (~700 lines), **zero semantic
  risk**, every site independently verified by the compiler, trivially reviewable (`git diff
  --stat` should show only `+2` per site outside the ~8 sites that intentionally pass non-empty
  vectors). A `sed`/`comby` pass followed by `cargo fmt` + `tools/check-defs-fmt.sh` does most of it.
- **(B) NOT recommended for this PB — box into `ActivateAbilityData` + derive `Default`.** This is
  the SR-10 treatment applied to `CastSpell` (`docs/sr-remediation-plan.md:601`, `:1213`), and it is
  the *right long-term shape* — it would make the *next* field addition free. But it converts a
  mechanical diff into a semantic refactor across 200 sites in the same commit as a correctness
  fix, and SR-10's own note records that boxing **moves the protocol digest independently** (closure
  grew 90 → 91). Two digest-moving changes in one PB makes the fingerprint delta un-attributable.
  **File as a follow-up seed (OOS-RS2-*) instead; do not do it here.**

`Command` is `#[derive(Serialize, Deserialize)]` with `#[serde(default)]` on both new fields, so
existing golden scripts in `test-data/generated-scripts/` that emit `activate_ability` /
`tap_for_mana` actions deserialize unchanged. **Verify** this against SR-9c's strict-script-JSON
rule: if the script action struct carries `#[serde(deny_unknown_fields)]`, *reading* old scripts is
still fine (no new required key); only *writing* a new key needs a schema field (§7.2).

### 3.4 `PROTOCOL_VERSION` 26 → 27; `PROTOCOL_SCHEMA_FINGERPRINT` re-pinned; `HASH_SCHEMA_VERSION` **stays 63**

**PROTOCOL — bump required.** `Command` is one of the three wire frames
(`protocol.rs:27-31`). Two of its variants change declared shape. Exact precedent, same variant,
same mechanism: **`- 12: PB-EF7`** at `protocol.rs:123-129` — *"`Command::ActivateAbility` (a wire
frame) gains `modes_chosen: Vec<usize>` … the closure's type count is unchanged; both `Command` and
`AbilityDefinition`'s declared shapes moved, so the digest moves."*

- Set `PROTOCOL_VERSION = 27` (`protocol.rs:248`).
- Append a `- 27: PB-RS2 (2026-07-20)` history entry above it, naming both variants, citing
  CR 107.4e/107.4f/602.2b, and stating that the closure's type count is **unchanged** (both
  `HybridManaPayment` and `bool` are already reachable via `CastSpellData`).
- Append the `(27, <new fingerprint>)` row to `PROTOCOL_SCHEMA_HISTORY` (the SR-27 append-only
  ledger at `protocol.rs:282+`).
- Re-pin `PROTOCOL_SCHEMA_FINGERPRINT` (`protocol.rs:265-266`, currently
  `315a211a729431c5688f89d1d3517453cb2d5ffd9c3833c68cf8622387a01559`) to the value **printed by the
  failing `crates/engine/tests/core/protocol_schema.rs`**. Derive it from the test output; do not
  compute it by hand. The fingerprint is blake3 over the normalized *declaration text* of the
  closure, so it is not predictable from this plan.
- **Do not re-pin without bumping.** `protocol.rs:287-295` documents exactly that cheat and the
  `protocol_version_sentinel` exists to catch it.

**HASH — must NOT move; stays 63.** Justification, positively asserted rather than assumed:

1. `Command` has **no `HashInto` implementation**. Grepping `state/hash.rs` for `Command` returns 23
   hits, all of them `ZoneId::Command`, `LossReason::CommanderDamage`, `ObjectFilter::Commander`,
   `Condition::YouControlYourCommander`, and prose — **zero** hash arms for the `Command` enum.
   `hash.rs:612` says so directly ("cured it for the `Command`/`GameEvent` protocol only"). Commands
   are wire frames, not hashed state.
2. No `GameEvent` variant is added or reshaped. The two events this PB emits — `ManaCostPaid`
   and `LifeLost` — already exist and keep their current shapes.
3. No `GameState`-reachable type changes. `ManaCost`, `HybridMana`, `PhyrexianMana`,
   `HybridManaPayment`, `AbilityDefinition`, `ActivatedAbility` are all untouched in shape.
4. The `state::diagnostics`-adjacent guard (§6) adds a `debug_assert`, not a field.

**If `crates/engine/tests/core/` reports a HASH drift, that contradicts this plan** — stop and
re-scope rather than re-pinning. The most likely innocent cause would be an accidental field
addition to a hashed struct while threading the choices; the most likely guilty cause would be
having chosen migration option (B).

---

## 4. Engine Change 2 — the shared flatten helper (AC 5119)

**No extraction is needed** (§0.3). `flatten_hybrid_phyrexian` is already a `pub` free function at
`casting.rs:6500-6605` with signature:

```rust
pub fn flatten_hybrid_phyrexian(
    cost: &ManaCost,
    hybrid_choices: &[HybridManaPayment],
    phyrexian_life_payments: &[bool],
) -> (ManaCost, u32)
```

**Relocation decision — recommend a narrow move, not a copy:**

- **Do NOT** leave it in `casting.rs` and call `crate::rules::casting::flatten_hybrid_phyrexian`
  from `mana.rs`. `mana.rs` reaching into `casting.rs` for a pure cost helper is a layering smell,
  and it puts a name with "casting" in its path on the mana-ability hot path.
- **DO** move it to **`crates/card-types/src/state/game_object.rs`**, adjacent to `ManaCost`,
  `HybridMana`, `PhyrexianMana`, `HybridManaPayment`, and `ManaCost::mana_value()` — every type it
  touches already lives there. It references **no** engine type and **no** `GameState`, so this
  respects SR-6 (`card-types` may not reference `GameState`; nothing in `card-defs` gains an engine
  dep — `card-defs` already depends on `card-types`).
  - Preferred form: an inherent method, `impl ManaCost { pub fn flatten_hybrid_phyrexian(&self,
    hybrid_choices: &[HybridManaPayment], phyrexian_life_payments: &[bool]) -> (ManaCost, u32) }`.
  - Keep a `pub use` / thin `#[deprecated]`-free re-export at `casting.rs` so `casting.rs:3991`
    and any external caller keep compiling; the engine re-exports `card-types` wholesale, so
    `crate::state::game_object::…` resolves inside the engine unchanged (SR-6 mechanism).
- **SR-6 cost check**: moving code *into* `card-types` **will** trigger a rebuild of all 1,798 card
  defs on this commit. That is a one-time build cost, not an invariant violation (the invariant is
  the *arrow direction*, not "never touch card-types"). Verify with `cargo check -p mtg-engine -v`
  afterwards that a subsequent *engine-only* edit again leaves `mtg-card-defs` `Fresh`.

**Borrow/ownership friction: none.** `casting.rs:3988-3994` already calls it through a
`&ManaCost` obtained from `if let Some(ref cost) = mana_cost`, returning owned values before
`state.player_mut` is taken at `:4003`. The activate and mana paths can follow the identical
shape: flatten first (pure, no `state` borrow), then take `state.player_mut`.

**One helper defect to fix while relocating** (it becomes reachable for the first time here):
`casting.rs:6537-6538` computes the hybrid color and then writes `let _ = (a, b); // used above via
default` — i.e. **it never validates that a `HybridManaPayment::Color(c)` choice is actually one of
the pip's two halves.** Today a caster can declare `{B/R}` paid with `Green` and the flattener will
happily require green mana. On the cast path this is a latent illegal-payment hole; on the new
activate/mana paths it would be a fresh one. **Add validation**: return
`Err(GameStateError::InvalidCommand)` (or have the helper return `Result`, or validate in the two
new call sites) when the chosen color is not a component of the pip — CR 107.4e, "can be paid in
one of two ways, as represented by the two halves of the symbol." Same for
`PhyrexianMana::Hybrid(a, b)`: `:6589-6600` hard-codes `a` and drops `b` with the comment "A more
precise choice would need a separate field." That limitation is acceptable to carry forward (no
card on the roster has a hybrid-Phyrexian in an *activation* cost — `ajani_sleeper_agent:21` is a
card mana cost), but **document it in the new call sites** so the next author does not read the
`hybrid_choices` field and assume it covers the hybrid-Phyrexian color choice. File as a follow-up
seed if the runner prefers; do not silently widen this PB to add a third field.

---

## 5. Engine Change 3 — thread the choices through both payment paths

### 5.1 `handle_activate_ability` (`crates/engine/src/rules/abilities.rs:748-758`)

Replace the block with the `casting.rs:3988-4023` shape, adapted:

```rust
        // CR 107.4e/107.4f (via CR 602.2b): flatten hybrid/Phyrexian choices before payment.
        // An activated ability's activation cost is its analog to a spell's mana cost.
        let (flat_cost, phyrexian_life) =
            if !resolved_cost.hybrid.is_empty() || !resolved_cost.phyrexian.is_empty() {
                resolved_cost.flatten_hybrid_phyrexian(&hybrid_choices, &phyrexian_life_payments)?
            } else {
                (resolved_cost.clone(), 0)
            };
        if flat_cost.mana_value() > 0 {
            let player_state = state.player_mut(player)?;
            if !player_state.mana_pool.can_spend(&flat_cost, None) {
                return Err(GameStateError::InsufficientMana);
            }
            player_state.mana_pool.spend(&flat_cost, None);
        }
        // CR 107.4f + CR 119.4: pay life for Phyrexian pips paid with life.
        if phyrexian_life > 0 {
            let player_state = state.player_mut(player)?;
            if player_state.life_total < phyrexian_life as i32 {
                return Err(GameStateError::InvalidCommand(
                    "cannot pay Phyrexian life: life total is less than the payment (CR 119.4)".into(),
                ));
            }
            player_state.life_total -= phyrexian_life as i32;
            events.push(GameEvent::LifeLost { player, amount: phyrexian_life });
        }
        if flat_cost.mana_value() > 0 || phyrexian_life > 0 {
            // Emit the ORIGINAL cost (with pip info) for event consumers — mirrors casting.rs:4045.
            events.push(GameEvent::ManaCostPaid { player, cost: resolved_cost });
        }
```

**Placement relative to the `mana_value() > 0` gate — the brief asks whether the gate is now
wrong. It is. Reasoning:**

The gate must be evaluated on the **flattened** cost, not the raw one, and the flatten must come
**before** it. Two concrete failure modes prove it:

1. **Free-pip (today's bug)**: raw `{B/R}` has `mana_value() == 1` (CR 202.3f), so the gate passes,
   and `can_spend` then sees an all-zero cost. Flattening first turns it into `black: 1`, so the
   gate passes *and* the check is meaningful.
2. **The brief's own hypothetical — pure `{B/P}` paid entirely with life**: raw
   `mana_value() == 1` (CR 202.3g), so the gate passes; **flattened** with
   `phyrexian_life_payments == [true]`, the cost is `{0}` and `mana_value() == 0`, so the gate
   correctly *skips* the mana check — and the life deduction, which must sit **outside** the gate,
   still fires. This is precisely why the Phyrexian-life block above is a **sibling** of the
   `if flat_cost.mana_value() > 0` block, not nested inside it. `casting.rs:4014` has the same
   structure and is correct on this point.

The converse (`{2/W}` paid with `Generic`) also only works post-flatten: raw mv is 2 by CR 202.3f
but the *payable* shape is `generic: 2`, which only `can_spend` on the flattened cost expresses.

**Atomicity / rollback — verified, no new mechanism needed.** `abilities.rs:760-765` documents it
explicitly: *"an `Err` anywhere below discards the whole `GameState` regardless, since
`process_command` takes `GameState` by value and only returns it on `Ok`."* This is Architecture
Invariant #2/#3 doing the work — `GameState` is immutable-by-move, so a mid-cost `Err` after a
`state.player_mut` mutation discards the mutated state entirely. Confirmed against
`memory/gotchas-infra.md`'s note that `process_command()` takes ownership. **No manual rollback,
no snapshot, no partial-payment refund is required.** The runner must *not* invent one; doing so
would be dead code. (One caveat to keep: because the whole state is discarded on `Err`, the
CR 119.4 life check may be placed either before or after the mana spend without correctness
consequence. Place it *before* the deduction anyway, for legibility and to match `casting.rs:4031`.)

### 5.2 `handle_tap_for_mana` (`crates/engine/src/rules/mana.rs:213-220` and `:304-313`) — §0.2

Identical treatment, at both the legality site and the payment site, from the **same** flattened
value. Two specifics:

- Flatten **once**, above the `:213` legality block, and reuse the result at `:304`. Recomputing
  risks the two sites drifting — which is structurally how this whole class of bug arose.
- The Phyrexian life deduction must merge with the **existing** `ability.life_cost` handling at
  `:225-233` (legality) and `:314-320` (payment). Note `:225` already implements CR 119.4 correctly
  (`life_total < life_cost` → `InsufficientLife`) — **check the combined total**
  (`ability.life_cost + phyrexian_life`) against life once, not each independently, because CR 119.4
  is a check on "the amount of the payment" for the whole cost, and CR 601.2h/602.2b lets the
  components be paid in any order. A player at 3 life activating a `Pay 2 life` + `{G/P}`-with-life
  ability may not pay 4.
- `mana.rs`'s SR-28 snapshot boundary comment at `:295-303` explains the ordering contract
  ("cost components that cannot move the source, then the one that can"). Mana and life payment
  both sit on the safe side; keep the new code inside that same region.

### 5.3 `casting.rs:4014-4023` — CR 119.4 repair (§0.4)

Add the `life_total < phyrexian_life` guard, returning `InvalidCommand` with a CR 119.4 citation,
and correct the misleading comment at `:4017`. Mirror the Bolas's Citadel guard 12 lines below.
**Expect this to red a test** if any existing test casts a Phyrexian spell at low life; if so, that
test was encoding the bug and must be updated with a CR 119.4 citation (SR-9c spirit).

---

## 6. Engine Change 4 — the fail-loud residue guard (AC 5120)

### 6.1 SR-4 classification, with the doc consulted

`docs/engine-invariants.md:41-48` (SR-4) states the vocabulary: an absence that is an **engine bug**
goes through `state::diagnostics`'s `expect_*` family (`debug_assert!`, `#[track_caller]`); an
absence that is a **rules-correct fizzle** goes through `fizzle_*`/`lki_*` with a CR citation.

**Classification: engine bug.** A non-empty `cost.hybrid` / `cost.phyrexian` arriving at
`can_spend`/`spend` means the **caller failed to flatten**. There is no game state and no CR rule
under which that is correct behavior: CR 107.4e/107.4f say the pip *is* a cost that *must* be paid
one of two ways, so a payment routine that ignores it is not fizzling — it is undercharging. The
brief's framing is confirmed against the doc.

### 6.2 But it cannot literally use `expect_*` — an honest deviation

Two reasons the guard must be a plain `debug_assert!`, not a `state::diagnostics` call:

1. **Scope.** SR-4's swept surface is `effects/mod.rs` and `rules/resolution.rs`;
   `docs/engine-invariants.md:47-48` says "the rest of `rules/` is not yet swept (`scutemob-66`)."
   `abilities.rs` and `mana.rs` are unswept.
2. **Architecture — the binding reason.** `can_spend`/`spend` live in
   `crates/card-types/src/state/player.rs`. The `expect_*` family is a set of **`GameState`
   methods** (`crates/engine/src/state/diagnostics.rs:38`, `:90` — `impl` on `GameState`), and
   SR-6 forbids anything in `card-types` from referencing `GameState`. The diagnostics vocabulary
   is also specifically about *state lookups returning `Option`* (`diagnostics.rs:36-40`), which a
   pure cost-shape precondition is not.

**Therefore**: implement the SR-4 *classification* with the SR-4 *mechanism's* semantics
(`debug_assert!` + `#[track_caller]`), sited in `player.rs`, and document the deviation in the
guard's own doc comment with a pointer to `docs/engine-invariants.md` SR-4 and to
`diagnostics.rs`'s "engine bug" column. Do **not** invent a `card-types` mirror of `diagnostics`.

### 6.3 Shape

```rust
    /// CR 107.4e/107.4f: hybrid and Phyrexian pips are **not** payable here. A caller
    /// must resolve them into standard pips with `ManaCost::flatten_hybrid_phyrexian`
    /// first (CR 601.2f, and CR 602.2b for activation costs).
    ///
    /// Reaching this with a non-empty `hybrid`/`phyrexian` is an **engine bug**, not an
    /// LKI fizzle (SR-4 classification, `docs/engine-invariants.md`): the pip names a
    /// real cost, so ignoring it silently *undercharges* the player. It cannot use the
    /// `state::diagnostics` `expect_*` family — those are `GameState` methods and SR-6
    /// bars `card-types` from referencing `GameState` — so it asserts directly.
    ///
    /// This exact silence made every filter land a free "{T}: Add two mana" for the
    /// life of the project (OOS-RS-2). PB-RS2.
    #[track_caller]
    fn debug_assert_flattened(cost: &ManaCost) {
        debug_assert!(
            cost.hybrid.is_empty() && cost.phyrexian.is_empty(),
            "unflattened mana cost reached the payment path: {} hybrid + {} Phyrexian pip(s) \
             would be paid for free (CR 107.4e/107.4f). Call \
             ManaCost::flatten_hybrid_phyrexian first. cost = {:?}",
            cost.hybrid.len(), cost.phyrexian.len(), cost,
        );
    }
```

Called at the top of both `can_spend` (`:148`) and `spend` (`:185`).

### 6.4 Release vs debug behavior, and how a test observes it

- **Debug (every `cargo test` build)**: `debug_assert!` fires → panic at the **call site**
  (`#[track_caller]`), naming the offending cost. This is the behavior the mandatory test asserts.
- **Release**: `debug_assert!` compiles out entirely. Payment proceeds on the un-flattened cost,
  i.e. today's undercharge. This is the deliberate SR-4 tradeoff
  (`diagnostics.rs:38` — "releases return `None`"; assertions are diagnostic, not load-bearing).
  Correctness in release is guaranteed by the **call sites** flattening, not by the guard.
- **Observing it in a test**: `#[should_panic(expected = "unflattened mana cost reached the payment
  path")]`, calling `ManaPool::can_spend` directly with a hybrid-carrying `ManaCost`. Because the
  guard is in `card-types`, this test belongs in
  `crates/card-types/src/state/player.rs`'s `#[cfg(test)]` module, **not** in the engine's
  integration suite — an engine integration test cannot reach an unflattened cost once §5 lands
  (that is the whole point), so testing it from there would require constructing an artificial
  bypass. Gate the test with `#[cfg(debug_assertions)]` so a `--release` test run does not fail.

---

## 7. Simulator + harness (AC 5122)

### 7.1 `crates/simulator/src/legal_actions.rs`

**`LegalAction::ActivateAbility`** (`:35-38`) and **`LegalAction::TapForMana`** (`:26-34`) both need
the choice fields. `TapForMana::chosen_color` (`:29-33`) is the template, including its doc note:
*"When `Some`, always a concrete legal colour (never `Colorless`) — a bot must never suggest a
colour the engine rejects (SR-38 precedent)."* Carry that standard forward verbatim in spirit.

**Affordability gate must be recomputed on the flattened cost.** `legal_actions.rs:430-435`:

```rust
                if let Some(ref cost) = ability.cost.mana_cost {
                    if !can_afford(state, player, cost) { continue; }
                }
```

Post-fix this is **wrong in the offering direction**: it checks the raw cost, which for a pure
`{B/R}` is all-zero and therefore always "affordable" — the provider would offer an action the
engine now rejects with `InsufficientMana`. That is exactly the SR-38 failure the `chosen_color`
doc warns about. The provider must:

1. Enumerate the candidate payment plans for the ability's pips.
2. For each plan, flatten and test `can_afford` (mana) **and** CR 119.4 (life).
3. Offer the action **only if at least one plan is fully payable**, carrying that plan's
   `hybrid_choices` / `phyrexian_life_payments` on the `LegalAction`.

**Plan-selection policy — keep it deterministic and non-suicidal:**

- **Hybrid `ColorColor(a, b)`**: prefer whichever half the pool can actually cover; tie-break to
  `a` (matching the flattener's documented default at `casting.rs:6526-6528`). Never offer a half
  the pool cannot pay.
- **Hybrid `GenericColor(c)`** (`{2/W}`): prefer `Color(c)` (1 mana) over `Generic` (2 mana) —
  again matching the flattener's default (`:6557-6567`) — falling back to `Generic` if the color is
  unavailable but 2 generic is.
- **Phyrexian**: **prefer mana; use life only when mana is unavailable.** And gate the life option
  on **both** rules:
  - **Legality (CR 119.4)**: `life_total >= 2 * (number of pips paid with life) + ability.life_cost`.
    At 1 life, paying 2 is illegal — never offer it.
  - **Non-suicide (the explicit hazard in the brief)**: at exactly 2 life, paying 2 life is
    **legal** (2 ≥ 2) and drops the bot to 0, which SBA converts into a loss (CR 104.3b). A bot
    that does this has not made an illegal move; it has made a losing one. **The provider must not
    offer a Phyrexian-life plan that would reduce the bot's life total to 0 or below** unless no
    other plan exists — and if none does, do not offer the action at all. Encode this as a named
    predicate (`phyrexian_life_plan_is_survivable`) with the CR 104.3b citation, not as a bare
    inequality, so a reviewer can see the *policy* is deliberate and distinguish it from the
    *legality* check beside it. These are two different rules and must not be collapsed.
  - Precedent for the shape: `legal_actions.rs:1154-1159` and `:1202-1213` already test the
    life-cost boundary cases (`life_cost == life` offered; `life_cost 0` offered at negative life,
    CR 119.4b). Add sibling tests for the Phyrexian boundary at life = 1 / 2 / 3.

### 7.2 `crates/simulator/src/random_bot.rs` (`:168-180`) and `heuristic_bot.rs` (`:47`)

`random_bot` translates `LegalAction` → `Command`; pass the plan through verbatim. It currently
hard-codes `hybrid_choices: vec![], phyrexian_life_payments: vec![]` on `CastSpell` (`:155-156`,
`:285-286`, `:308-309`) — that is a **pre-existing** gap on the cast side and stays out of scope
(§9), but the new activate/tap fields must be threaded, not stubbed, or the bots regress to
never being able to activate a filter land at all. `heuristic_bot` scores `ActivateAbility { .. }`
at a flat 40 and needs no change beyond compiling.

### 7.3 `crates/engine/src/testing/replay_harness.rs` — `translate_player_action`

The `"activate_ability"` arm is at **`:671-701`**; `"tap_for_mana"` is a sibling arm. Both build the
`Command` and both need the new fields. To let scripts *express* a choice (not just default), add
two optional JSON keys to the action struct, mirroring the existing optional keys
(`discard_card_name`, `sacrifice_card_name`, `x_value`, `modes_chosen`):

- `"hybrid_choices": ["black", ...]` → `Vec<HybridManaPayment>` (accept `"generic"` for the
  `{2/C}` half).
- `"phyrexian_life_payments": [true, ...]` → `Vec<bool>`.

Both `#[serde(default)]`. **SR-9c check**: the golden-script corpus is triaged and cannot skip
silently; confirm the action struct's `deny_unknown_fields` posture before adding keys, and confirm
that omitting them keeps all 210 approved scripts byte-identical in outcome (they will — empty
vectors reproduce today's defaults on every non-pip cost, and no approved script currently drives a
filter land's pip ability, since doing so was free and nobody wrote an assertion for it).

Note that `hybrid_choices`/`phyrexian_life_payments` are currently `vec![]` at **all 20**
`CastSpellData` construction sites in this file — i.e. the choice channel has never been exercised
through the harness at all. That is context for §9's scope boundary, not a task.

### 7.4 SR-31 equivalence ratchet

**What SR-31 is**: `docs/sr-remediation-plan.md:78` — "Equivalence coverage ratchet (6 of 60+
command shapes)", DONE 2026-07-16 (merge `64446b1e`). It lives in
`crates/engine/tests/scripts/harness_equivalence.rs`.

**What it currently asserts** (`:1648-1740`):
- `CROSS_VALIDATED_SHAPES` (`:1657-1671`) lists **11** labels: 6 base shapes (`pass_priority`,
  `play_land`, `tap_for_mana`, `cast_spell`, `activate_ability`, `declare_attackers`) plus 5
  alt-cost shapes added by SR-31 (`cast_spell:convoke`, `:delve`, `:kicker`, `cast_spell_escape`,
  `cast_spell_modal`).
- `TRANSLATE_PLAYER_ACTION_ARMS = 79` (`:1684`) is the honest *denominator* — the count of
  `"action" =>` arms in `translate_player_action`, so nobody can redefine the ratchet's baseline as
  "everything."
- `cross_validated_shape_coverage_is_ratcheted` (`:1708`) asserts set **equality** between the
  shapes actually driven by `ALL_VALIDATED_MOVE_SETS` (`:1689-1698`) and `CROSS_VALIDATED_SHAPES`:
  **⊇** so the list can't claim fiction, **⊆** so adding a covering scenario *forces* a list update.

**Extension required by this PB** — add **two** new labels and their scenarios:
- `activate_ability:hybrid` — a `MoveSet` driving an activated ability with a `{B/R}` pip through
  both the JSON-script regime and the direct-`Command` regime, asserting identical per-step
  fingerprints (SR-9b).
- `tap_for_mana:hybrid` — the same for a filter land, i.e. **the exact live-wrong case**. This is
  the highest-value addition in the PB: it is the shape that had no cross-validation and was free.

Optionally add `activate_ability:phyrexian` (birthing_pod) if §8 lands. Each new label requires a
new `const *_MOVES` appended to `ALL_VALIDATED_MOVE_SETS` **and** a new entry in
`CROSS_VALIDATED_SHAPES`, or the ratchet fails — which is the mechanism working.

### 7.5 TUI and replay-viewer

`tools/tui/src/play/input.rs` (3 `ActivateAbility` refs, 1 `TapForMana`) and
`play/panels/action_menu.rs` (1) construct `Command`s and must compile. There is **no** exhaustive
match on `Command` in the replay-viewer's `view_model.rs` (its exhaustive matches are on
`StackObjectKind` and `KeywordAbility`, neither of which changes here), so no display arm is
expected — but per the standing gotcha that runners miss this ~50% of the time, **`cargo build
--workspace` is the gate** and must be run after the implement phase, not just `cargo check -p
mtg-engine`.

---

## 8. Card dispositions (AC 5121)

### 8.1 The 7 filter lands — **all 7 stay `known_wrong`; zero flips**

All seven were read. They are byte-identical in structure:

| Card | Pip site | Filter colors | Current marker |
|---|---|---|---|
| `twilight_mire.rs:31` | ability 1 cost | B/G | `known_wrong` (`:49`) |
| `graven_cairns.rs:31` | ability 1 cost | B/R | `known_wrong` (`:49-52`) |
| `sunken_ruins.rs:31` | ability 1 cost | U/B | `known_wrong` (`:49`) |
| `flooded_grove.rs:31` | ability 1 cost | G/U | `known_wrong` (`:49`) |
| `rugged_prairie.rs:31` | ability 1 cost | R/W | `known_wrong` (`:49`) |
| `fetid_heath.rs:33` | ability 1 cost | W/B | `known_wrong` (`:51`) |
| `cascade_bluffs.rs:31` | ability 1 cost | U/R | `known_wrong` (`:49`) |

**Disposition: the unrelated fixed-mode simplification remains a real blocker for all 7.**
Each card's `Effect::AddManaFilterChoice { color_a, color_b }` hard-codes the *middle* of the three
printed modes ("Add {B}{B}, {B}{R}, **or** {R}{R}" → always 1{B}+1{R}). PB-RS2 charges the input pip
correctly but does not touch the output. Fixing the output needs a player choice at *resolution*
time, which is the same missing M10 interactive-decision channel that keeps `hidden_strings`
dormant (triage §4) — `Effect::Choose` executes `choices.first()` and is machine-barred from
`Complete` by `crates/engine/tests/core/effect_choose_gate.rs:81-93`.

**Do NOT flip these.** Per `feedback_pb_yield_calibration`, an optimistic flip here would be exactly
the overcount pattern. **What to do instead**: append one sentence to each `known_wrong` note
recording that the activation cost is, as of PB-RS2, correctly charged — so the marker text names
the *one* surviving deviation rather than silently covering two. Also update each file's header
comment (line 1-2), which currently describes the ability as `{B/R},{T}: ...` without noting the pip
was free.

**Net effect for these 7: an integrity repair, not a coverage flip.** They stop being free.

### 8.2 `birthing_pod` — **the one real flip**, contingent on a DSL walk

Oracle text (MCP-sourced, authoritative):

> **Birthing Pod** — `{3}{G/P}` Artifact
> ({G/P} can be paid with either {G} or 2 life.)
> `{1}{G/P}`, `{T}`, Sacrifice a creature: Search your library for a creature card with mana value
> equal to 1 plus the sacrificed creature's mana value, put that card onto the battlefield, then
> shuffle. Activate only as a sorcery.

Current state: `completeness: Completeness::inert(...)` (`:39-48`), `abilities: vec![]` with a
`TODO(OOS-OS8-1)` at `:34-37`. Its header comment `:10-15` states the blocker precisely and
correctly. Its card-level `mana_cost` at `:22-26` already carries the `{3}{G/P}` and is fine
(cast path).

**DSL walk — every piece verified present:**

| Requirement | DSL | Verified |
|---|---|---|
| `{1}{G/P}` activation cost | `Cost::Mana(ManaCost { generic: 1, phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)], ..Default::default() })` | `card_definition.rs:1242` (`Cost::Mana`); `game_object.rs:91-96` (`PhyrexianMana::Single`) |
| ...**paid** | **THIS PB** (§5.1) | — |
| `{T}` | `Cost::Tap` | `card_definition.rs:1244` |
| Sacrifice a creature | `Cost::Sacrifice(TargetFilter)` with a creature filter | `card_definition.rs:1257` |
| Combined | `Cost::Sequence(vec![...])` | `card_definition.rs:1268` |
| MV **equal to** 1 + sacrificed MV | `max_cmc_amount == min_cmc_amount == EffectAmount::Sum(Fixed(1), ManaValueOfSacrificedCreature)` | `card_definition.rs:3096` (`min_cmc_amount`), `:2798` (`ManaValueOfSacrificedCreature`); reference impl `eldritch_evolution.rs:44` |
| ...**resolves for an activated (not cast) ability** | `stack_obj.sacrificed_creature_lki = sacrificed_lki` | **`abilities.rs:1302-1305`** — confirmed: the activate path captures cost-sacrifice LKI and threads it to `EffectContext`, read at `effects/mod.rs:7694-7695`. This was the one hop that could have silently killed the flip; it works. |
| Search → battlefield, then shuffle | `Effect::SearchLibrary { destination: Battlefield, .. }` | `card_definition.rs:2039` |
| Activate only as a sorcery | `timing_restriction` / `sorcery_speed` on `AbilityDefinition::Activated` | present (see `graven_cairns.rs:19`) |

**Disposition: author the ability and flip `inert` → `Complete`.** The runner must (a) verify the
`SearchLibrary` shuffle semantics and the "put onto the battlefield" destination against a
reference def, and (b) if any single piece fails on inspection, **stop and record an honest
remaining-blocker note rather than authoring a partial** — W6 policy (no card authored until its
primitives exist; no TODOs, no partials, no wrong game state).

**Note the Phyrexian choice interaction**: with `phyrexian_life_payments: [true]`, the cost becomes
`{1}` + 2 life. The `{1}` still needs paying. This is the multi-component case the CR 119.4
combined-total check in §5.2 exists for, and it is the natural integration test (§9.4).

### 8.3 `drivnod_carnage_dominus` — **no flip**; correct the record narrowly

Per §0.5: the `:43-44` claim is technically accurate about *expressibility* and misleading about
*payability*. Reword the completeness note; leave `partial`. Its two real blockers
(`Cost` has no `ExileFromGraveyard`; `CounterType` has no `Indestructible`) are untouched.

### 8.4 Mandatory roster sweep (SR-36 — enumerate, do not grep)

`docs/engine-invariants.md` SR-36: *"enumerate `all_cards()` for rosters, never grep source."*
Add a scan test (sibling to `crates/engine/tests/core/completeness_deviation_scan.rs`):

```
fn every_hybrid_or_phyrexian_pip_in_an_activation_cost_is_accounted_for()
```

Walk `all_cards()`; for each `CardDefinition`, inspect every `AbilityDefinition::Activated`'s
`cost` (recursing through `Cost::Sequence`) and every mana-ability cost, collecting any
`Cost::Mana(mc)` where `!mc.hybrid.is_empty() || !mc.phyrexian.is_empty()`. Assert the resulting
name set equals a pinned constant. **Expected pinned set after this PB: the 7 filter lands +
`birthing_pod` = 8.** This makes the roster a machine fact and makes the *next* such card fail the
suite until someone confirms its cost is charged.

My grep baseline for cross-checking (26 files contain `hybrid:`/`phyrexian:`, but 19 are card-level
`mana_cost` or `MutateCost` — cast path): `tezzerets_gambit`, `deathrite_shaman`, `blade_historian`,
`vraska_betrayals_sting`, `boggart_ram_gang`, `connive`, `brokkos_apex_of_forever`,
`noxious_revival`, `mental_misstep`, `ajani_sleeper_agent`, `revitalizing_repast`, `dismember`,
`kitchen_finks`, `nethroi_apex_of_death`, `necropanther`, `leyline_of_the_guildpact`,
`gitaxian_probe`, `vexing_shusher`, plus `birthing_pod`'s card cost. **The `all_cards()` sweep is
authoritative; this grep list is a sanity check only.**

---

## 9. Mandatory tests (AC 5123) — every test cites its CR section (Invariant #8)

**Primary file**: `crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs` (new; add the
`mod` line to `crates/engine/tests/primitives/mod.rs` — SR-9a: a dropped `mod` line silently
deletes coverage).

### 9.1 Step-0 probes, kept as permanent regressions (§2)
1. `hybrid_pip_in_activated_cost_requires_mana` — CR 107.4e, 602.2b. Empty pool + `{B/R}` activated
   ability → `Err(InsufficientMana)`. **Was `Ok` before the fix.**
2. `hybrid_pip_in_mana_ability_cost_requires_mana` — CR 107.4e, 605.1a, 602.2b. `graven_cairns`,
   empty pool, `TapForMana` ability 1 → `Err(InsufficientMana)`. **Was `Ok` before the fix, and
   produced 2 mana from nothing.**

### 9.2 Hybrid — both halves payable
3. `hybrid_activated_cost_payable_with_either_half` — CR 107.4e. `{B/R}`: with only `{B}` in pool
   and `hybrid_choices: [Color(Black)]` → `Ok`, pool empty after. With only `{R}` and
   `[Color(Red)]` → `Ok`. With only `{B}` and `[Color(Red)]` → `Err(InsufficientMana)`.
4. `hybrid_choice_must_name_a_component_of_the_pip` — CR 107.4e ("as represented by the two halves
   of the symbol"). `{B/R}` with `[Color(Green)]` → `Err(InvalidCommand)`. **This is the §4
   validation gap; the test will fail until it is added.**
5. `hybrid_empty_choices_defaults_to_first_color` — CR 107.4e + the documented default at
   `casting.rs:6526-6528`. `{B/R}`, `hybrid_choices: vec![]`, pool `{B}` → `Ok`; pool `{R}` → `Err`.
   Pins the backward-compatibility contract that ~200 migrated call sites now rely on.
6. `monocolored_hybrid_payable_as_two_generic` — CR 107.4e ("{2/B} … either one black mana or two
   mana of any type"), CR 202.3f. `{2/B}` with `[Generic]` and 2 colorless in pool → `Ok`.

### 9.3 Phyrexian — mana vs life
7. `phyrexian_activated_cost_payable_with_mana` — CR 107.4f. `{G/P}`, `[false]`, `{G}` in pool →
   `Ok`, life unchanged.
8. `phyrexian_activated_cost_payable_with_two_life` — CR 107.4f, 119.4. `{G/P}`, `[true]`, **empty
   pool**, 20 life → `Ok`, life 18. This is the case the whole seed exists for.
9. `phyrexian_life_payment_requires_sufficient_life` — CR 119.4. `{G/P}`, `[true]`, life **1** →
   `Err(InvalidCommand)` citing CR 119.4, life unchanged. (Life **2** → `Ok`, life 0, SBA loss —
   assert this separately as *legal*; it documents the legal-vs-suicidal boundary §7.1 encodes.)
10. `phyrexian_and_explicit_life_cost_check_combined_total` — CR 119.4, 601.2h/602.2b. An ability
    with `life_cost: 2` **and** a `{G/P}` paid with life, at 3 life → `Err`. At 4 life → `Ok`,
    life 0.
11. `phyrexian_paid_with_life_skips_the_mana_gate` — CR 107.4f, 202.3g. A **pure** `{B/P}` cost
    (raw `mana_value() == 1`) with `[true]` and an **empty pool** → `Ok`. Proves the flatten sits
    *before* the `mana_value() > 0` gate and that the life deduction is a sibling, not a child, of
    that gate (§5.1). **This is the brief's own hypothetical, pinned.**

### 9.4 Filter-land cost regression (the live-wrong roster)
12. `filter_land_charges_its_hybrid_pip` — CR 107.4e, 605.1a. For **each** of the 7 lands (a
    table-driven loop over the card ids, so a new filter land is covered automatically): empty pool
    → `Err`; correct half in pool → `Ok` with the half consumed and the two filtered mana produced;
    wrong half only → `Err`. Assert net mana delta is **+1**, never +2.
13. `birthing_pod_activation_charges_the_phyrexian_pip` — CR 107.4f, 602.2b. `{1}{G/P}`: with
    `{1}{G}` and `[false]` → `Ok`; with `{1}` only and `[true]` → `Ok`, life −2; with `{1}` only
    and `[false]` → `Err`. (Only if §8.2 authors the card.)

### 9.5 Residue guard
14. `unflattened_cost_panics_in_debug` — in `crates/card-types/src/state/player.rs`'s test module,
    `#[cfg(debug_assertions)]` + `#[should_panic(expected = "unflattened mana cost reached the
    payment path")]`. See §6.4 for why it cannot live in the engine suite.

### 9.6 Simulator
15. `provider_never_offers_an_unpayable_pip_ability` — SR-38 precedent, CR 107.4e. Sibling of the
    existing `legal_actions.rs:1216+` `chosen_color` test. A `{B/R}` ability with an empty pool must
    **not** appear in `legal_actions`.
16. `provider_never_offers_a_suicidal_phyrexian_life_plan` — CR 104.3b, 119.4. At 2 life with a
    `{G/P}` ability and no green mana: the mana plan is unpayable and the life plan is legal but
    lethal → the action is **not** offered. At 1 life: not offered (illegal). At 5 life: offered
    with `[true]`.

### 9.7 Cross-regime
17. SR-31 ratchet extension — §7.4. Two new labels + move sets.

### 9.8 Pattern to follow
`crates/engine/tests/primitives/primitive_sr34_composite_mana_costs.rs` (composite mana-ability
costs — 17 `TapForMana` sites; the closest existing analogue) and
`crates/engine/tests/primitives/pb_ef12_any_color_choice.rs` (a `TapForMana` *choice channel* with
a legality check and a matching simulator guard — the closest structural analogue).

---

## 10. Honest yield estimate (`feedback_pb_yield_calibration`)

The triage's §3 R2 row reads: *"Discounted ship: **1** (birthing_pod) + 7 lands correctly costed."*

**That headline is already honest**, and my card-by-card verification confirms it — with one
sharpening the triage did not state explicitly:

- **Coverage flips: 1.** `birthing_pod` `inert` → `Complete`, and only if the §8.2 DSL walk holds
  end-to-end. Call it **1, with real (~20%) risk of 0** if `SearchLibrary`'s
  battlefield-destination + shuffle shape turns out to have its own gap.
- **Filter lands: 0 flips.** All 7 stay `known_wrong`. Anyone reading "7 lands" as coverage is
  misreading it. Their marker text improves; their marker does not.
- **Drivnod: 0 flips.** Two unrelated blockers survive.
- **Integrity repairs: 8** (7 lands stop being free + `casting.rs`'s CR 119.4 hole).
- **Latent-defect closures: 2** (the unvalidated hybrid-color choice §4; the whole 20-site
  free-pip class, now guarded §6).

**Bottom line: 1 flip, ~62.9% coverage moves by roughly +0.05pp.** The value of this PB is not
coverage — it is that a shipped card currently produces two mana from nothing, and that the payment
path stops being able to silently undercharge anyone ever again. Judge it on that.

**Effort calibration**: the ~350 mechanical construction-site edits (§3.3) plus a second handler
(§0.2) make this **materially larger than the triage's "budget accordingly" note implies**. The
brief anticipated one command and one handler; the reality is two commands and two handlers.

---

## 11. Scope boundaries — explicitly NOT in this PB

1. **The filter lands' fixed-mode simplification.** `AddManaFilterChoice` hard-codes the middle of
   three printed modes. Needs the M10 interactive-decision channel. Same root cause as
   `hidden_strings` (triage §4). **The 7 lands stay `known_wrong`.**
2. **Boxing `ActivateAbility` into `ActivateAbilityData`.** The right long-term shape, the wrong
   commit (§3.3 option B). File as a follow-up seed.
3. **A third choice field for hybrid-Phyrexian pips** (`{G/W/P}`'s two-color choice, currently
   hard-coded to the first color at `casting.rs:6589-6600`). No card on the roster has one in an
   *activation* cost. Document at the call sites; do not widen.
4. **Flattening the other 18 `can_spend`/`can_pay_cost` sites** (§0.6). All provably hybrid-free
   today; the §6 guard covers the future.
5. **The cast path's harness/bot choice channel.** `hybrid_choices` is `vec![]` at all 20
   `CastSpellData` sites in `replay_harness.rs` and all 3 in `random_bot.rs` — the *cast* side's
   choice channel has never been driven through the harness either. Real, adjacent, pre-existing.
   **File as a follow-up seed; the activate/tap side is this PB's job.**
6. **`abilities.rs:766-773`'s missing CR 119.4 check on `ability_cost.life_cost`.** Adjacent
   (§0.4), currently masked by `legal_actions.rs:440`. Tempting; out of scope unless the §5.2
   combined-total work makes it a one-liner, in which case take it *and say so in the commit*.
7. **`Cost::RemoveCounter` dynamic-X** (OOS-RS-6), `LibraryPosition` inertness (OOS-RS1-1),
   `AtBeginningOfCombat` sweep (OOS-OS9-1 / R3). Separate queue items.
8. **PB-R6 (CR 611.2c set snapshot, OOS-OS7-2) must NOT be batched with this PB.** Standing
   constraint from triage §3 sequencing note — flagged as a collision risk by independent
   verification. Both force a PROTOCOL bump; ship them as separate bumps (26→27 here, 27→28 there)
   rather than sharing one, so each digest delta is attributable.

---

## 12. Verification checklist

- [ ] Step-0 probes written **first**, observed passing (bug live) before any engine edit (§2)
- [ ] `mana.rs` handler fixed, not just `abilities.rs` (§0.2) — **the correction that makes the PB work**
- [ ] `flatten_hybrid_phyrexian` relocated to `card-types`, re-exported; hybrid-color validation added (§4)
- [ ] SR-6 re-verified: `cargo check -p mtg-engine -v` leaves `mtg-card-defs` `Fresh` after a later engine-only edit
- [ ] Flatten precedes the `mana_value() > 0` gate; Phyrexian life is a **sibling** of it, not nested (§5.1)
- [ ] CR 119.4 enforced on all three Phyrexian-life sites (activate, tap-for-mana, and the `casting.rs:4014` repair)
- [ ] Residue guard added with the SR-4 classification **and** the documented `card-types` deviation (§6)
- [ ] `PROTOCOL_VERSION = 27`; history entry appended; `PROTOCOL_SCHEMA_HISTORY` row appended; fingerprint re-pinned **from test output**
- [ ] `HASH_SCHEMA_VERSION` **still 63** — if it moved, stop and re-scope (§3.4)
- [ ] All ~350 construction sites migrated (option A); `rg 'Command::(ActivateAbility|TapForMana) \{'` shows no site missing the new fields
- [ ] Simulator offers no unpayable **and** no suicidal action (§7.1)
- [ ] SR-31 `CROSS_VALIDATED_SHAPES` extended with `activate_ability:hybrid` + `tap_for_mana:hybrid`
- [ ] `all_cards()` roster sweep test pins the 8-card set (§8.4)
- [ ] 7 filter-land notes updated; **none flipped**
- [ ] `birthing_pod` authored + flipped, **or** an honest remaining-blocker note (§8.2)
- [ ] `drivnod` note reworded; stays `partial`
- [ ] `cargo build --workspace` (TUI + replay-viewer + simulator) — **not just `cargo check -p mtg-engine`**
- [ ] `cargo test --all`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35)
- [ ] `primitive-impl-reviewer` pass; every finding dispositioned

---

## 13. Risks & edge cases

1. **HIGHEST — the `mana.rs` path.** If the runner follows the brief literally and fixes only
   `abilities.rs`, the PB ships with its entire live-wrong roster unrepaired while every test
   passes. Probe B (§2) is the guard against this and **must be written before any edit.**
2. **~350-site migration fatigue.** The likeliest failure is a mechanical pass that misses the TUI
   or `mana_solver.rs`. `cargo build --workspace` is the gate; `cargo check -p mtg-engine` is not.
3. **Fingerprint re-pin without a version bump.** `protocol.rs:287-295` names this exact cheat.
   Take the new value from the failing test's output; append the history row; bump the version.
4. **The residue guard turning red in unrelated suites.** If any existing test constructs a
   hybrid-carrying cost and feeds it to a non-flattening path, the `debug_assert` fires. That is
   the guard working — the fix is to flatten at that site (or prove the test was encoding the bug),
   never to weaken the assert.
5. **Golden-script drift.** Adding `#[serde(default)]` fields should leave all 210 approved scripts
   passing. If any script's outcome changes, it was almost certainly asserting the free-pip
   behavior — repair it with a CR 107.4e/107.4f citation (SR-9c), do not retire it.
6. **`Cost::Sequence` recursion.** Filter lands nest `Cost::Mana` inside `Cost::Sequence`
   (`graven_cairns.rs:29-35`). Both the payment threading and the §8.4 roster sweep must recurse;
   a non-recursive implementation will silently find zero pips and pass every test.
7. **Bot suicide via Phyrexian.** At exactly 2 life, paying 2 is *legal*. Legality and policy are
   two different rules (CR 119.4 vs CR 104.3b) and must be two different named predicates in
   `legal_actions.rs`, or a future reader will "simplify" them into one and reintroduce the bug.
8. **Mutate costs.** `brokkos_apex_of_forever:45` / `nethroi_apex_of_death:46` carry hybrid pips on
   `AbilityDefinition::MutateCost`. Mutate is an alt *cast* cost and should reach the
   `casting.rs:3988` flatten — **but this was not traced to source.** Verify; if it bypasses, that
   is a third defective path and a scope decision (recommend: fix it here, it is the same helper
   call, and the §6 guard will make it panic in tests anyway).
9. **`x_count` interaction.** `flatten_hybrid_phyrexian` copies `x_count` through unchanged
   (`casting.rs:6517`), while `abilities.rs:703-708` folds X into `generic` *before* the flatten
   point. Order is fine as planned (X first, then flatten), but the runner must not reorder them —
   flattening first would leave `x_count` unconsumed.
