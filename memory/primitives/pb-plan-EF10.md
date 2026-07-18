# Primitive Batch Plan: PB-EF10 — sacrifice-driven `EffectAmount` / runtime `max_cmc` / "if you do" `Condition`

**Generated**: 2026-07-18
**Primitive**: Three independent sub-gaps from finding **EF-W-MISS-7**, shipped as three
cleanly-separated commits inside one PB:
1. **`EffectAmount::ToughnessOfSacrificedCreature`** — the LKI-toughness twin of the
   existing `PowerOfSacrificedCreature`, reading layer-resolved toughness captured at the
   sacrifice moment (anthem/+X/+X counted; graveyard state NOT re-read; CR 608.2b/608.2i).
2. **Runtime-computed search cap** — a new `TargetFilter.max_cmc_amount: Option<EffectAmount>`
   honored by the `SearchLibrary` executor, so "mana value X or less, where X = N + the
   sacrificed creature's mana value" is expressible (Eldritch Evolution N=2; Birthing
   Ritual N=1). Requires a companion `EffectAmount::ManaValueOfSacrificedCreature`.
3. **`Condition::SacrificeFired`** — reports whether a resolution-time
   `Effect::SacrificePermanents` in this resolution actually moved ≥1 permanent, so a
   following `Conditional { condition: SacrificeFired, .. }` gates the "if you do" clause
   (CR 608.2c/608.2h sequencing; Victimize ruling 2020-11-10).

All three are backed by a **single data-model change**: `EffectContext` /
`StackObject` / `AdditionalCost::Sacrifice` stop carrying `Vec<i32>` powers and instead
carry `Vec<SacrificedCreatureLki { power, toughness, mana_value }>`, plus a new transient
`EffectContext.sacrifice_fired: bool`.

**CR Rules**: 608.2b, 608.2c, 608.2h, 608.2i (LKI + sequential resolution), 701.21a
(Sacrifice keyword action), 202.3 (mana value), 613.1d (layer-resolved characteristics),
400.7 (object identity). *(Brief's "CR 717 interlocked abilities" is a stale reference —
CR 717 is Attraction. The "if you do" pattern is governed by 608.2c/608.2h sequencing, not
by an intervening-if (603.4); verified via MCP.)*

**Cards affected**: 4 candidates, **~3 flip Complete** (discounted per
`feedback_pb_yield_calibration`):
- **Momentous Fall** — new file, Complete (sub-gap 1).
- **Eldritch Evolution** — new file, Complete (sub-gap 2, cost-sac).
- **Victimize** — new file, Complete (sub-gap 3, resolution-sac).
- **Birthing Ritual** — new file, **stays partial** (sub-gaps 2+3 wired, but the
  "look at top seven / put one / rest to bottom in random order" dig is inexpressible →
  named blocker + OOS seed **OOS-EF10-1**).

**Optional bonus (EF-EF1-A)**: the returned-`Vec` design (Step 5) makes it nearly free to
also populate `ctx` from the **optional** (`MayPayThenEffect`) sacrifice path, closing
**EF-EF1-A** and flipping **disciple_of_freyalise** front-face. Marked OPTIONAL — do NOT
let it block or complicate the core review; split it if it adds risk.

**Dependencies**: none. All prerequisites exist — `PowerOfSacrificedCreature` machinery
(effects/mod.rs:7263, abilities.rs:1023, casting.rs:4237, resolution.rs:393-403),
`EffectAmount::Sum` (Box/Box), `Effect::Conditional`, `Effect::MoveZone` with
`controller_override` + `ZoneTarget::Battlefield { tapped }`, `TargetCardInYourGraveyard`,
the `self_exile_on_resolution` card flag.

**Deferred items from prior PBs**: **EF-W-MISS-7** (this batch's origin). **EF-EF1-A**
(PB-EF1 follow-up) is adjacent and optionally closed here. No other carry-forward.

---

## MANDATORY pre-existing TODO sweep (roster-recall gate)

Runner MUST run, at implement time, and record the result in the commit body:

```
Grep -i "TODO|BLOCKED|ENGINE" + "sacrificed creature's toughness"   crates/card-defs/src/defs/
Grep -i "TODO|BLOCKED" + "ToughnessOfSacrificedCreature"            crates/card-defs/src/defs/
Grep -i "TODO|BLOCKED" + "ManaValueOfSacrificedCreature|max_cmc.*sacrific" crates/card-defs/src/defs/
Grep -i "TODO|BLOCKED" + "if you do.*sacrific|SacrificeFired"        crates/card-defs/src/defs/
Grep -i "runtime max_cmc|sacrificed creature's mana value"          crates/card-defs/src/defs/
```

Planner sweep result (2026-07-18): grep of `crates/card-defs/src/defs/` for the sacrifice
LKI notes surfaced **disciple_of_freyalise.rs** (EF-EF1-A, optional bonus below),
**ziatora_the_incinerator.rs** (L48 — "sacrificed_creature_powers only populated on the
activated-ability-cost path" → this is the SAME EF-EF1-A optional-cost-capture gap; a
**forced-add candidate IF the optional-path capture (Step 5b bonus) is done**), and
**plumb_the_forbidden.rs** (L41 — counts creatures sacrificed to an optional cost; a
*count* of sacrifices, a distinct gap, NOT this PB — leave). The 4 brief candidates are not
themselves TODO-tagged (they are missing files, authored fresh). **TODO sweep positive
assertion**: 0 cards self-identify the three core primitives by name; 2 cards
(disciple_of_freyalise, ziatora_the_incinerator) self-identify the EF-EF1-A optional-path
capture that Step 5b optionally closes.

---

## CR Rule Text (from MCP, authoritative)

**608.2c** — "The controller of the spell or ability follows its instructions in the order
written. However, replacement effects may modify these actions. In some cases, later text
on the card may modify the meaning of earlier text … read the whole text and apply the
rules of English to the text." → "Sacrifice a creature. If you do, …" is two sequential
instructions; the second is gated on the first having happened.

**608.2h** — "If an effect requires information from a specific object, including the source
of the ability itself, the effect uses the current information of that object if it's in the
public zone it was expected to be in; if it's no longer in that zone … the effect uses the
object's last known information." → the sacrificed creature's power/toughness/mana value are
read as **LKI** (it is in the graveyard when the amount is computed).

**608.2i** — the look-back exception: an effect that *reads information about a previous
game state* uses that prior state. Together with 608.2h this is why we **capture** the
layer-resolved power/toughness/mana value BEFORE `move_object_to_zone` (a graveyard object
has lost its battlefield-gated layer effects, per gotcha BASELINE-LKI-01) and never re-read.

**701.21a** — "To sacrifice a permanent, its controller moves it from the battlefield
directly to its owner's graveyard. A player can't sacrifice something … they don't control.
Sacrificing a permanent doesn't destroy it, so regeneration or other effects that replace
destruction can't affect this action."

**Victimize ruling (2020-11-10)** — "As Victimize resolves, you must sacrifice a creature if
able. You can't change your mind. … If both [targets] are illegal, Victimize won't resolve.
You won't sacrifice a creature." → the sacrifice is mandatory-if-able; "if you do" is false
only when the controller controls no creature at resolution.

**Birthing Ritual ruling (2024-06-07)** — "Use the mana value of the sacrificed creature as
it last existed on the battlefield to determine the value of X." + "If … you control no
creatures when it tries to resolve, the ability will do nothing." → LKI mana value; optional
sacrifice.

**Momentous Fall ruling (2010-06-15)** — "The sacrificed creature's last known existence on
the battlefield is checked to determine its power **and its toughness**." → sub-gap 1 reads
LKI toughness exactly as PowerOfSacrificedCreature reads LKI power.

**Eldritch Evolution ruling (2016-07-13)** — the sacrificed creature's mana value is its
last-known value; `{X}` in a library card's cost counts as 0 (already handled by
`ManaCost::mana_value()`).

---

## Architecture Decision (resolve + justify) — the LKI carrier

The single `sacrificed_creature_powers: Vec<i32>` must now also carry toughness (sub-gap 1)
and mana value (sub-gap 2). **Decision: one struct, not parallel vecs.**

```rust
// crates/card-types/src/state/types.rs (new, near AdditionalCost)
/// CR 608.2b/608.2h/608.2i: last-known layer-resolved characteristics of a creature
/// sacrificed as a cost or by a resolution-time effect, captured BEFORE `move_object_to_zone`.
/// Read by EffectAmount::{PowerOf,ToughnessOf,ManaValueOf}SacrificedCreature.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SacrificedCreatureLki {
    pub power: i32,
    pub toughness: i32,
    pub mana_value: u32,
}
```

**Why the struct beats parallel vecs (both correctness AND diff size):**
- **Desync-proof (the decisive argument).** With three parallel vecs, a capture site that
  sets `powers` but forgets `toughnesses` makes `PowerOfSacrificedCreature` read correctly
  while `ToughnessOfSacrificedCreature` silently reads 0 — exactly the legal-but-wrong class
  the project prioritizes eliminating (`project_legal_but_wrong_gap.md`). One struct is
  written atomically per creature: a site either captures the LKI or it doesn't.
- **Smaller literal diff.** Every `AdditionalCost::Sacrifice { ids, lki_powers: vec![] }`
  becomes `{ ids, lki: vec![] }` (rename), and every `sacrificed_creature_powers: vec![]`
  becomes `sacrificed_creature_lki: vec![]`. Parallel vecs would instead add TWO fields to
  each of those ~75 literals — strictly more text and more desync surface.
- **Wire cost already paid.** The bump to PROTOCOL/HASH is forced by any of the three
  sub-gaps regardless, so the "struct changes the wire shape" cost is not additional.

**Rename** (semantics changed — the field no longer holds "powers"):
`AdditionalCost::Sacrifice.lki_powers → .lki`; `StackObject.sacrificed_creature_powers →
.sacrificed_creature_lki`; `EffectContext.sacrificed_creature_powers → .sacrificed_creature_lki`.

**Wire/hash confirmation** (all machine-forced, do NOT hand-guess digests):
- `AdditionalCost` and `StackObject` are in **both** the SR-8 protocol closure
  (`Command`/`GameEvent` → StackObject → additional_costs) **and** the GameState hash
  closure. `EffectContext` is neither (runtime resolution scratch — not serialized, not
  hashed; confirmed hash.rs:41-43).
- The new `EffectAmount` variants, `Condition::SacrificeFired`, and
  `TargetFilter.max_cmc_amount` all reach the SR-8 closure via `Effect`/`Characteristics`.
- ⇒ **PROTOCOL_VERSION 14 → 15** (protocol.rs:144) and **HASH_SCHEMA_VERSION 52 → 53**
  (hash.rs:464), both re-pinned from the failing gate output.

**`EffectContext.sacrifice_fired: bool`** — new, transient (not hashed, not on the wire),
default `false`. Set `true` by the `Effect::SacrificePermanents` executor when ≥1 permanent
was sacrificed this resolution. Read by `Condition::SacrificeFired`.

---

## Engine Changes

### COMMIT 1 — data-model migration + sub-gap 1 (`ToughnessOfSacrificedCreature`)

**Step 1.1 — new carrier type.**
`crates/card-types/src/state/types.rs`: add `SacrificedCreatureLki` (above). Change
`AdditionalCost::Sacrifice { ids: Vec<ObjectId>, lki_powers: Vec<i32> }` (L223-226) →
`{ ids: Vec<ObjectId>, lki: Vec<SacrificedCreatureLki> }`; rewrite the doc block
(L214-222) to describe the three-field LKI and keep the "parallel to `ids`, LKI consumers
must verify `lki.len() == ids.len()` or fall back to 0" contract.

**Step 1.2 — rename the two persistent fields.**
- `crates/card-types/src/state/stack.rs:464`: `sacrificed_creature_powers: Vec<i32>` →
  `sacrificed_creature_lki: Vec<SacrificedCreatureLki>`; update the doc (L457-463) and the
  `blank`/default init at stack.rs:562.
- `crates/engine/src/effects/mod.rs:134`: same rename on `EffectContext`; update doc
  (L130-134). Add `pub sacrifice_fired: bool` immediately after, with a doc citing CR
  608.2c/608.2h and "transient — not hashed, not serialized." Init both in
  `EffectContext::new` (L199 area) and the `Default`/other constructor (L236 area):
  `sacrificed_creature_lki: vec![], sacrifice_fired: false`.

**Step 1.3 — mechanical literal migration** (compiler-driven; `cargo build --workspace`
enumerates them). Grep inventory (planner-measured):
| Pattern | Sites | Action |
|---|---|---|
| `lki_powers: vec![]` in `AdditionalCost::Sacrifice { .. }` literals | ~40 (replay_harness.rs ×5; tests: bargain, emerge ×10, casualty ×10, devour ×4, cost_primitives ×2, pbp ×2, pb_ac8 ×1) | → `lki: vec![]` |
| `AdditionalCost::Sacrifice { ids, lki_powers }` **patterns** (casting.rs:186 uses `{ ids, .. }` — unaffected) | casting.rs:4240, resolution.rs:397, hash.rs:3844 | → bind `lki` |
| `sacrificed_creature_powers: vec![]` ctx/stack inits | ~35 (copy.rs ×3, engine.rs ×4, casting.rs 4580/7929, abilities.rs 279, + ~25 test files) | → `sacrificed_creature_lki: vec![]` |
These are pure renames of empty-`vec![]` sites; no logic. Do NOT add `sacrifice_fired` to
struct-init sites that use `EffectContext::new(..)`/`..Default::default()` — only the two
constructors initialize it.

**Step 1.4 — capture full LKI at the three capture sites** (this is where the toughness +
mana value actually get recorded):
- `crates/engine/src/rules/abilities.rs:1019-1023` (activated-ability sac cost): `resolved`
  chars are in scope. Replace `sacrificed_lki_powers.push(lki_power.unwrap_or(0))` with a
  push of `SacrificedCreatureLki { power: resolved.power.unwrap_or(0), toughness:
  resolved.toughness.unwrap_or(0), mana_value: resolved.mana_cost.as_ref().map(|c|
  c.mana_value()).unwrap_or(0) }`. Rename the local `sacrificed_lki_powers` (L916, L1023,
  L1277) → `sacrificed_lki`. Update the assignment `stack_obj.sacrificed_creature_lki =
  sacrificed_lki` (was L1277).
- `crates/engine/src/rules/casting.rs:4231-4250` (spell additional-cost sac — Momentous
  Fall & Eldritch Evolution path): `chars` (from `expect_characteristics(state, sac_id)`)
  is in scope. Build the struct: `power = chars.power.unwrap_or(0)`, `toughness =
  chars.toughness.unwrap_or(0)`, `mana_value = chars.mana_cost.as_ref().map(|c|
  c.mana_value()).unwrap_or(0)`. Replace the `lki_powers.resize/[pos]=` patch (L4240-4248)
  with the equivalent over `lki: Vec<SacrificedCreatureLki>` (resize with
  `SacrificedCreatureLki { power:0, toughness:0, mana_value:0 }`, then set `lki[pos] = ..`).
- `crates/engine/src/rules/resolution.rs:393-403` (copy AdditionalCost→ctx for spells):
  change the `find_map` arm `AdditionalCost::Sacrifice { lki_powers, .. } =>
  Some(lki_powers.clone())` → `{ lki, .. } => Some(lki.clone())`; assign
  `ctx.sacrificed_creature_lki = lki_from_ac`.
- `crates/engine/src/rules/resolution.rs:1868` (copy StackObject→ctx for activated): →
  `ctx.sacrificed_creature_lki = stack_obj.sacrificed_creature_lki.clone();`
- `crates/engine/src/effects/mod.rs:3233, 3273` (nested/ForEach ctx clones): rename field.

**Step 1.5 — the new EffectAmount variant + resolver.**
- `crates/card-types/src/cards/card_definition.rs`: in `EffectAmount` (after
  `PowerOfSacrificedCreature`, ~L2578 region — keep near the other sacrifice reads), add
  ```rust
  /// CR 608.2b/608.2h: LKI toughness of the first creature sacrificed as a cost/effect.
  /// Layer-resolved (anthem counted), captured before move_object_to_zone. 0 if none.
  ToughnessOfSacrificedCreature,
  ```
- `crates/engine/src/effects/mod.rs:7263` (`resolve_amount`, next to the
  `PowerOfSacrificedCreature` arm): add
  ```rust
  EffectAmount::ToughnessOfSacrificedCreature =>
      ctx.sacrificed_creature_lki.first().map(|l| l.toughness).unwrap_or(0),
  ```
  and change the existing `PowerOfSacrificedCreature` arm (L7264) to
  `.first().map(|l| l.power).unwrap_or(0)`.
- `crates/engine/src/state/hash.rs`: EffectAmount HashInto — add
  `EffectAmount::ToughnessOfSacrificedCreature => 22u8.hash_into(hasher)` (next after
  HandSize=21, before the closing brace at L5362).

**Step 1.6 — hash arms for the reshaped persistent fields.**
- `crates/engine/src/state/hash.rs`: add `impl HashInto for SacrificedCreatureLki`
  (power/toughness/mana_value each hashed). At L3755-3756 (StackObject field) hash
  `self.sacrificed_creature_lki` (len + each element). At L3844-3854
  (`AdditionalCost::Sacrifice { ids, lki }`) hash `lki` (len + each element). Update the
  comment block at hash.rs:41-43.

### COMMIT 2 — sub-gap 2 (runtime search cap)

**Step 2.1 — new EffectAmount variant + resolver** (mirror Step 1.5): add
`ManaValueOfSacrificedCreature` to the enum; resolver arm
`ctx.sacrificed_creature_lki.first().map(|l| l.mana_value as i32).unwrap_or(0)`; hash
discriminant **23**.

**Step 2.2 — `TargetFilter.max_cmc_amount`.**
`crates/card-types/src/cards/card_definition.rs`, `TargetFilter` (next to `max_cmc` at
L2942): add
```rust
/// Runtime-computed max mana value cap (inclusive). None = no runtime cap.
/// CR 202.3 / 608.2h. Resolved from EffectContext at execution — therefore ONLY honored by
/// the `Effect::SearchLibrary` executor (which has `ctx`), NOT by `matches_filter`
/// (which sees only `Characteristics`). Same "field the predicate can't see" contract as
/// `exclude_self` / `is_attacking`. Used for "mana value X or less, where X = N + the
/// sacrificed creature's mana value" (Eldritch Evolution, Birthing Ritual).
#[serde(default)]
pub max_cmc_amount: Option<EffectAmount>,
```
This is the cheap path: `TargetFilter` derives `Default` and defs build it with
`..Default::default()`, so **none of the 99 `SearchLibrary` def files change** (contrast: a
field on `Effect::SearchLibrary` would force all 99). Verified: TargetFilter derive line is
`#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]` (L2900).

**Step 2.3 — honor it in the SearchLibrary executor.**
`crates/engine/src/effects/mod.rs:2858-2891`: before the candidate loop, resolve the cap
once: `let runtime_cap: Option<i32> = filter.max_cmc_amount.as_ref().map(|a|
resolve_amount(state, a, ctx));`. In the candidate predicate (after
`matches_filter(&obj.characteristics, filter)` at L2887), add:
```rust
&& runtime_cap.is_none_or(|cap| {
       obj.characteristics.mana_cost.as_ref().map(|c| c.mana_value() as i32).unwrap_or(0) <= cap
   })
```
(Library cards have no layers; `obj.characteristics` is correct. `is_none_or` is stable in
1.95 — else `map_or(true, ..)`.) The static `filter.max_cmc` path (matches_filter) still
applies independently, so a card carrying both caps is ANDed correctly.

**Step 2.4 — hash arm.**
`crates/engine/src/state/hash.rs:5059` (`impl HashInto for TargetFilter`): add a
`self.max_cmc_amount.hash_into(hasher)` line (Option<EffectAmount> — EffectAmount already
has HashInto). No change to the `Effect::SearchLibrary` hash arm (hash.rs:6045) — the field
lives on `filter`, already hashed there.

**Note (pre-existing, do NOT fix here):** the M7 `SearchLibrary` executor does not shuffle
the library *after* placing (only the `shuffle_before_placing` path shuffles). Eldritch
Evolution says "then shuffle." The deterministic-M7 fallback (library order by ObjectId) is
unchanged and acceptable; this is a pre-existing behavior shared by every search-to-battlefield
card, not introduced by this PB. Record in the Eldritch def note; do not gate Completeness on it.

### COMMIT 3 — sub-gap 3 (`Condition::SacrificeFired` + resolution-sac capture)

**Step 3.1 — `sacrifice_permanents_for_player` returns what it sacrificed.**
`crates/engine/src/effects/mod.rs:7702`: change the return type from `()` to
`Vec<SacrificedCreatureLki>`. At the pre-zone-move capture block (L7718-7744) the full
layer-resolved `chars` are already computed (`sac_perm_pre_chars` / `pre_death_power`);
build a `SacrificedCreatureLki { power: chars.power.unwrap_or(0), toughness:
chars.toughness.unwrap_or(0), mana_value: chars.mana_cost.as_ref().map(|c|
c.mana_value()).unwrap_or(0) }` for each permanent that is *actually moved* (push only in
the `Redirect`/`Proceed` success arms where the zone move succeeded — NOT in
`ChoiceRequired`, which defers). Return the accumulated vec.

**Step 3.2 — the mandatory executor records into ctx.**
`crates/engine/src/effects/mod.rs:3359-3371` (`Effect::SacrificePermanents`): collect the
returns across `player_ids`; then set
```rust
if !sacrificed.is_empty() { ctx.sacrifice_fired = true; }
ctx.sacrificed_creature_lki = sacrificed; // most-recent sacrifice wins; overwrite, not append
```
(Overwrite is correct: "if you do" refers to *this* sacrifice instruction. Document that
`sacrifice_fired` / `sacrificed_creature_lki` reflect the most-recent `SacrificePermanents`
in the resolution — sufficient for Victimize/Birthing Ritual, single-sacrifice cards.)

**Step 3.3 — the new Condition variant.**
- `crates/card-types/src/cards/card_definition.rs`, `Condition` enum (after `Always`,
  ~L3531): add
  ```rust
  /// CR 608.2c/608.2h: "if you do" — true iff an Effect::SacrificePermanents earlier in
  /// THIS resolution actually sacrificed ≥1 permanent (ctx.sacrifice_fired). Pair it with
  /// Effect::Conditional after Effect::SacrificePermanents (Victimize, Birthing Ritual).
  /// NOTE: for an OPTIONAL "you may sacrifice; if you do, X" use
  /// Effect::MayPayThenEffect{ cost: Cost::Sacrifice(..), then: X } instead — the `then`
  /// arm is the implicit "if you do" and needs no Condition.
  SacrificeFired,
  ```
- `crates/engine/src/effects/mod.rs:8378` (`check_condition`): add
  `Condition::SacrificeFired => ctx.sacrifice_fired,`.
- `crates/engine/src/state/hash.rs:5778` (Condition HashInto, after
  `OpponentControlsMoreLandsThanYou => 47`): add
  `Condition::SacrificeFired => 48u8.hash_into(hasher),`.

**Step 3.4 (OPTIONAL BONUS — EF-EF1-A) — the optional path records into ctx too.**
`crates/engine/src/effects/mod.rs:3307-3328` (`Effect::MayPayThenEffect`): `try_pay_optional_cost`
→ `pay_optional_cost` → (for `Cost::Sacrifice`) `sacrifice_permanents_for_player` now returns
the LKI vec. Thread that return up (change `pay_optional_cost`/`try_pay_optional_cost` to
return `Vec<SacrificedCreatureLki>` for the sacrifice branch, `vec![]` otherwise) and, in the
MayPayThenEffect executor after a successful pay, set `ctx.sacrificed_creature_lki = returned;
if !returned.is_empty() { ctx.sacrifice_fired = true; }` **before** `execute_effect_inner(then)`.
This closes **EF-EF1-A** (PowerOfSacrificedCreature in the optional path) and lets
**disciple_of_freyalise** front-face + **ziatora_the_incinerator** work. **If this threads
cleanly, flip disciple_of_freyalise to Complete and add a decoy test; otherwise leave it and
keep EF-EF1-A filed — do not risk the core.** This is the *only* place the "do NOT regress
the MayPayThenEffect optional-cost path" caution bites: keep the existing `exclude_self`
source threading and the controller-rebind logic intact.

### Exhaustive match sites (compile-forcing — the #1 error source)

| File | Match | Action |
|---|---|---|
| `crates/engine/src/state/hash.rs` (EffectAmount HashInto ~L5326) | exhaustive | arms 22 (Toughness), 23 (ManaValue) |
| `crates/engine/src/state/hash.rs` (Condition HashInto ~L5611) | exhaustive | arm 48 (SacrificeFired) |
| `crates/engine/src/state/hash.rs` (TargetFilter/StackObject/AdditionalCost HashInto) | field-based | new `lki`/`max_cmc_amount` lines; `impl HashInto for SacrificedCreatureLki` |
| `crates/engine/src/effects/mod.rs` `resolve_amount` (~L7194) | exhaustive on EffectAmount | 2 new arms |
| `crates/engine/src/effects/mod.rs` `check_condition` (L8378) | exhaustive on Condition | 1 new arm |
| `crates/engine/src/rules/layers.rs:1955` `resolve_amount` twin? | verify | if a second exhaustive EffectAmount match exists, add 2 arms |
| `crates/engine/src/rules/casting.rs:186`, `resolution.rs:1185` `AdditionalCost::Sacrifice { ids, .. }` | uses `..` | no change (rest pattern) |
| replay-viewer / TUI | grep `EffectAmount`/`Condition`/`AdditionalCost` | RUN `cargo build --workspace`; add display arms if any exhaustive match flags (per gotchas-infra: view_model.rs / stack_view.rs are the usual offenders) |

**Runner MUST `cargo build --workspace`** after each commit — it is the seal gate (invariant
#3) and the only thing that proves every exhaustive match got its arm.

### Version-bump checklist (machine-forced — copy digests from the failing gates)

- `crates/engine/src/rules/protocol.rs:144` — `PROTOCOL_VERSION` **14 → 15**; add `- 15:`
  History line; set `PROTOCOL_SCHEMA_FINGERPRINT` from the failure; **append** a
  `ProtocolEpoch { version: 15, fingerprint: <same> }` row to `PROTOCOL_HISTORY` (never edit
  existing rows).
- `crates/engine/tests/core/protocol_schema.rs` — bump the version sentinel + FROZEN digest.
- `crates/engine/src/state/hash.rs:464` — `HASH_SCHEMA_VERSION` **52 → 53**; append a
  `HASH_SCHEMA_HISTORY` row with recomputed `decl_fingerprint` + `stream_fingerprint` (both
  move: decl for the reshaped enums/structs, stream for the changed HashInto arms).
- `crates/engine/tests/core/hash_schema.rs` — bump sentinel + fingerprints.
- Grep the whole test tree for scattered `assert_eq!(PROTOCOL_VERSION, 14)` /
  `assert_eq!(HASH_SCHEMA_VERSION, 52)` and the `pbn_subtype_filtered_triggers.rs:554`-style
  sentinel comments; bump each (SR-17/27 note ~29 such sentinels historically).

---

## Card Definition Fixes / New Cards

All four candidates are **missing files** (authored fresh). Chain-verified against MCP oracle
text per `feedback_verify_full_chain` (filter → amount → effect → cost):

### momentous_fall.rs — NEW → Complete (sub-gap 1)
**Oracle**: "As an additional cost to cast this spell, sacrifice a creature. You draw cards
equal to the sacrificed creature's power, then you gain life equal to its toughness."
**Chain**: cost = `spell_additional_costs: [SpellAdditionalCost::SacrificeCreature]` (exists,
captures LKI via casting.rs) → effect = `Sequence[ DrawCards { player: Controller, count:
PowerOfSacrificedCreature }, GainLife { player: Controller, amount:
ToughnessOfSacrificedCreature } ]`. Power may be ≤0 → draw 0 (resolve_amount clamp). **Fully
Complete after this PB.**

### eldritch_evolution.rs — NEW → Complete (sub-gap 2)
**Oracle**: "As an additional cost to cast this spell, sacrifice a creature. Search your
library for a creature card with mana value X or less, where X is 2 plus the sacrificed
creature's mana value. Put that card onto the battlefield, then shuffle. Exile Eldritch
Evolution."
**Chain**: cost = `SpellAdditionalCost::SacrificeCreature` → effect = `SearchLibrary {
player: Controller, filter: TargetFilter { has_card_types: [Creature], max_cmc_amount:
Some(Sum(Box(Fixed(2)), Box(ManaValueOfSacrificedCreature))), ..Default::default() },
destination: ZoneTarget::Battlefield { tapped: false }, reveal: true, .. }`; self-exile via
card flag `self_exile_on_resolution: true` (exists; replaces the 608.2n graveyard move).
**Fully Complete after this PB** (modulo the pre-existing post-search shuffle note above,
which does not affect correctness of the deterministic build).

### victimize.rs — NEW → Complete (sub-gap 3)
**Oracle**: "Choose two target creature cards in your graveyard. Sacrifice a creature. If you
do, return the chosen cards to the battlefield tapped."
**Chain**: targets = `[TargetCardInYourGraveyard(creature), TargetCardInYourGraveyard(creature)]`
(exists, L2854) → effect = `Sequence[ SacrificePermanents { player: Controller, count:
Fixed(1), filter: Some(creature) }, Conditional { condition: SacrificeFired, if_true:
Sequence[ MoveZone { target: DeclaredTarget{0}, to: Battlefield{tapped:true},
controller_override: Some(Controller) }, MoveZone { target: DeclaredTarget{1}, .. } ],
if_false: Nothing } ]`. Per the ruling: one illegal target still sacs + returns the other
(each MoveZone independently no-ops on an already-moved target); both illegal → spell doesn't
resolve → no sac (CR 608.2b target check, existing behavior). **Fully Complete after this PB.**

### birthing_ritual.rs — NEW → **stays partial** (sub-gaps 2+3 wired; dig blocked)
**Oracle**: "At the beginning of your end step, if you control a creature, look at the top
seven cards of your library. Then you may sacrifice a creature. If you do, you may put a
creature card with mana value X or less from among those cards onto the battlefield, where X
is 1 plus the sacrificed creature's mana value. Put the rest on the bottom of your library in
a random order."
**Chain-verify**: trigger `AtBeginningOfYourEndStep` + intervening-if `YouControlPermanent(creature)`
(exist) → *optional sacrifice* is expressible via `MayPayThenEffect { cost: Cost::Sacrifice(creature),
then: <dig> }` → sac MV via `ManaValueOfSacrificedCreature` (this PB) → cap `Sum(Fixed(1),
ManaValueOfSacrificedCreature)` (this PB). **The blocker is the DIG**: "look at top seven, you
may put ONE creature with MV ≤ X **from among those seven** onto the battlefield, put the rest
on the bottom in a **random order**." No existing `Effect` expresses "reveal/look at top N →
optionally place one matching a *runtime* MV cap onto the battlefield → rest to bottom
randomized." `Scry`/`Surveil` don't place on the battlefield; `SearchLibrary` searches the
*whole* library, not a looked-at top-7 subset, and can't leave the remainder bottom-randomized.
**Author partial** with `Completeness::partial("...")` naming the exact blocker; **file
OOS-EF10-1** (see below). Do NOT ship a `SearchLibrary`-of-the-whole-library approximation —
that would be legal-but-wrong (ignores the top-7 restriction and the bottom-random remainder).

### disciple_of_freyalise.rs — OPTIONAL flip → Complete (only if Step 3.4 done)
Front-face ETB "you may sacrifice another creature; if you do, gain X life and draw X cards,
X = its power" becomes expressible once the optional path populates
`ctx.sacrificed_creature_lki`. If Step 3.4 lands, wire the front face
(`MayPayThenEffect { cost: Cost::Sacrifice(TargetFilter { exclude_self:true, creature }), then:
Sequence[ GainLife{ PowerOfSacrificedCreature }, DrawCards{ PowerOfSacrificedCreature } ] }`)
and flip to Complete. Otherwise leave partial with the EF-EF1-A note intact.

---

## Unit Tests

Test files live in `crates/engine/tests/primitives/` as **modules** (SR-9a). Add
`mod pb_ef10_sacrifice_driven_amounts;` to `crates/engine/tests/primitives/main.rs` (after
`mod pb_ef9_while_you_control_source;` L30). Run: `cargo test --test primitives pb_ef10`.
**Pattern**: follow `primitives/pbp_power_of_sacrificed_creature.rs` (the direct twin — LKI
capture, AdditionalCost::Sacrifice construction, resolve_amount assertions) and
`primitives/pbn_subtype_filtered_triggers.rs:554` for the version sentinel.

Every test cites its CR (invariant #8). **Each decoy must fail on exactly the field under
test** — verify by temporary revert-and-rerun.

**Sub-gap 1 (`ToughnessOfSacrificedCreature`):**
- `test_toughness_of_sacrificed_creature_basic` (CR 608.2b) — sac a vanilla 2/5 as a cost;
  assert `GainLife` = 5.
- `test_toughness_of_sacrificed_creature_reads_layer_resolved` (CR 613.1d/608.2h) — sac a
  creature under a +0/+2 anthem; assert toughness read = printed + 2 (anthem counted at the
  sacrifice moment; graveyard state NOT re-read).
- **DECOY** `test_toughness_amount_reads_toughness_not_power` — sac a **1/3** creature (power
  ≠ toughness); assert `ToughnessOfSacrificedCreature` = 3. Fails if the arm was copy-pasted
  to read `.power` (the copy-from-PowerOfSacrificedCreature hazard).
- `test_momentous_fall_draws_power_gains_toughness` (integration) — cast Momentous Fall
  sacrificing a 3/4; assert draw 3 AND gain 4 (both LKI reads live in one card).

**Sub-gap 2 (runtime search cap):**
- `test_search_max_cmc_amount_caps_by_runtime_value` (CR 202.3/608.2h) — direct: ctx with a
  sac LKI mana_value = 3; `SearchLibrary` filter `max_cmc_amount = Sum(Fixed(2), ManaValueOfSacrificedCreature)`
  (cap 5). Library has a MV-5 and a MV-6 creature. Assert MV-5 found, MV-6 not.
- **DECOY** `test_search_cap_uses_both_terms` — same, but assert the found card is exactly
  MV-5 (= 2 + 3): fails if the `+2` is dropped (cap 3, MV-5 rejected) OR if the sac MV is
  dropped (cap 2, MV-5 rejected). Pins both summands.
- `test_eldritch_evolution_finds_up_to_two_plus_sac_mv` (integration) — cast Eldritch
  Evolution sacrificing an MV-2 creature (cap 4); assert a MV-4 creature enters the
  battlefield and a MV-5 does not; assert Eldritch Evolution is exiled (not in graveyard).

**Sub-gap 3 (`SacrificeFired`):**
- `test_sacrifice_fired_true_when_sacrificed` (CR 608.2c) — resolve `Sequence[
  SacrificePermanents{1, creature}, Conditional{ SacrificeFired, <marker effect> } ]` with a
  creature present; assert the conditional branch ran.
- `test_sacrifice_fired_false_when_none_available` (CR 608.2c, Victimize ruling) — same, but
  controller controls no creature; assert `sacrifice_fired == false` and the conditional
  branch did NOT run.
- **DECOY** `test_sacrifice_fired_not_hardcoded_true` — the false case above IS the decoy:
  fails if the executor sets `sacrifice_fired = true` unconditionally (ignoring whether
  anything moved).
- `test_victimize_returns_both_when_creature_sacrificed` (integration) — two creature cards
  in graveyard, one creature on battlefield; cast Victimize; assert both cards return
  tapped under controller and the on-board creature is sacrificed.
- `test_victimize_no_return_when_no_creature_to_sacrifice` (integration, Victimize ruling) —
  two graveyard targets but NO creature to sac; assert neither card returns.
- `test_victimize_one_illegal_target_still_sacs_and_returns_other` (CR 608.2b) — one target
  leaves the graveyard before resolution; assert sac still happens and the legal card returns.

**Optional (only if Step 3.4 done):**
- `test_disciple_of_freyalise_optional_sac_draws_power` — front-face ETB; sac a 4/4; assert
  gain 4 + draw 4 (EF-EF1-A closed).

**Hash / version parity:**
- `test_hash_new_effect_amount_variants_distinct` — two ctx with distinct
  `sacrificed_creature_lki` (differing toughness, differing mana_value) hash differently;
  equal ones hash equal. Assert `ToughnessOfSacrificedCreature` and
  `ManaValueOfSacrificedCreature` produce distinct `resolve_amount`/hash behavior.
- `test_sacrificed_creature_lki_struct_hash` — `AdditionalCost::Sacrifice` with LKI structs
  differing in one field hash differently (proves all three fields feed the stream).
- `test_pb_ef10_version_sentinels` — assert `PROTOCOL_VERSION == 15` and
  `HASH_SCHEMA_VERSION == 53` (the machine-forced values).

---

## New OOS Seed

**OOS-EF10-1 — top-N look / conditional-battlefield-place / bottom-random dig.**
Blocks **birthing_ritual.rs** (and any impulse-style "look at the top N, you may put one
matching [filter, possibly a runtime MV cap] onto the battlefield/into hand, put the rest on
the bottom in a random order"). Needs a new `Effect` (working shape:
`LookAtTopThenPlace { count: EffectAmount, filter: TargetFilter, destination, rest_to:
BottomRandom | Graveyard, optional: bool }`) that (a) scopes candidates to the looked-at top
N (not the whole library, unlike `SearchLibrary`), (b) honors a runtime `max_cmc_amount`, (c)
places at most one, (d) sends the remainder to the bottom in a randomized (non-deterministic
in M10+, deterministic-by-ObjectId in M7) order. File in
`memory/card-authoring/w-miss-engine-findings-2026-07-17.md` (EF-W-MISS-7 section) and the
active queue's OOS list.

---

## Verification Checklist

- [ ] `cargo check -p mtg-engine` clean after each commit
- [ ] `cargo build --workspace` clean (seal gate — proves every exhaustive match arm)
- [ ] TODO sweep run + result recorded in commit body (positive assertion)
- [ ] momentous_fall, eldritch_evolution, victimize authored `Complete`; birthing_ritual
      authored `partial` with the OOS-EF10-1 blocker named; disciple flipped ONLY if Step 3.4 done
- [ ] All ~75 `lki_powers`/`sacrificed_creature_powers` literal sites migrated (rename only)
- [ ] New unit tests pass incl. all three decoys, each proven non-vacuous by revert-and-rerun
- [ ] PROTOCOL 14→15 + HASH 52→53 + all sentinels bumped; fingerprints copied from failures
- [ ] `cargo test --all` green (incl. `core card_defs_fmt` + `tools/check-defs-fmt.sh`)
- [ ] `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` + defs-fmt clean
- [ ] `python3 tools/authoring-report.py` regenerated; coverage delta = the flips (+3, or +4 with disciple)
- [ ] OOS-EF10-1 filed

---

## Risks & Edge Cases

- **Copy-paste hazard (sub-gap 1).** `ToughnessOfSacrificedCreature`'s resolver is one field
  away from `PowerOfSacrificedCreature`; the 1/3 decoy exists precisely to catch a `.power`
  copy-paste. Same hazard for `ManaValueOfSacrificedCreature`.
- **Desync (the whole reason for the struct).** If any of the three capture sites (abilities
  1023, casting 4240, sacrifice_permanents 7727) forgets a field, that fact silently reads 0.
  The struct makes each capture atomic; the hash-parity test proves all three fields feed the
  stream. Do NOT reintroduce parallel vecs "for a smaller diff."
- **`sacrifice_fired` lifetime.** It is per-resolution and reflects the *most recent*
  `SacrificePermanents`. Correct for all four candidates (single sacrifice). A future card
  with two sacrifices + two "if you do" clauses would need per-clause tracking — document the
  limitation, don't over-engineer.
- **Optional vs mandatory "if you do".** Victimize (mandatory sac) needs
  `Condition::SacrificeFired`; Birthing Ritual/disciple (optional "you may sacrifice") use
  `MayPayThenEffect`'s implicit gating and do NOT need the Condition. Using the Condition on
  an optional path, or MayPayThenEffect on a mandatory path, both mis-model the card.
- **EF-EF1-A regression surface (Step 3.4).** Threading a return through
  `pay_optional_cost`/`try_pay_optional_cost` must preserve the existing `exclude_self` source
  threading and the payer/controller rebind. If it can't be done cleanly, SKIP it — the core
  three sub-gaps stand alone.
- **99-def trap avoided.** The runtime cap lives on `TargetFilter` (Default + serde-default),
  NOT on `Effect::SearchLibrary` — putting it on the Effect would have forced editing 99 card
  defs. Do not "simplify" it onto the Effect.
- **Object identity (CR 400.7).** Every LKI value is captured BEFORE `move_object_to_zone`;
  after the move the old ObjectId is dead and a graveyard `calculate_characteristics` has lost
  battlefield-gated layers (BASELINE-LKI-01). Never re-read post-move.
- **Birthing Ritual honesty.** It is the calibration case: three of its four mechanics are
  now expressible, but shipping it Complete via a whole-library `SearchLibrary` would be
  legal-but-wrong. Partial + OOS seed is the correct outcome (mirrors the W-MISS discipline).
