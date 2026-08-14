# PB-DX43 — CR 305.6/305.7 intrinsic mana abilities from basic land subtypes

**Task**: `scutemob-213` · **v4 queue rank 1** · closes `OOS-DX27-1` + `OOS-DX27-10`
**Branch**: `feat/pb-dx43-cr-30563057-intrinsic-mana-abilities-from-basic-land`
**Pre-edit baseline** (measured on THIS branch before any edit):
**4,721 passing / 0 failing / 5 ignored**, 49 result-producing targets, residual list empty.
Keyed test-name set saved for the close set-diff.

---

## 1. The rule, verbatim

**CR 305.6** (MCP `get_rule`): *"The basic land types are Plains, Island, Swamp, Mountain, and
Forest. ... An object with the land card type and a basic land type has the **intrinsic ability**
'{T}: Add [mana symbol],' even if the text box doesn't actually contain that text or the object has
no text box. For Plains, [mana symbol] is {W}; for Islands, {U}; for Swamps, {B}; for Mountains,
{R}; and for Forests, {G}."*

**CR 305.7**: *"If an effect sets a land's subtype to one or more of the basic land types, the land
no longer has its old land type. **It loses all abilities generated from its rules text, its old
land types, and any copiable effects affecting that land, and it gains the appropriate mana ability
for each new basic land type. Note that this doesn't remove any abilities that were granted to the
land by other effects.** ... If a land gains one or more land types in addition to its own, it keeps
its land types and rules text, and it gains the new land types and mana abilities."*

**CR 613.1d**: layer 4 is type-changing. CR 305.6's intrinsic ability and CR 305.7's
loss-and-gain are consequences *of the type change*, so they belong in **layer 4**, not layer 6.
This is the single most load-bearing reading in this batch and every design decision below follows
from it.

---

## 2. What the engine does today (re-located at HEAD, not trusted from the brief)

`Characteristics.mana_abilities` is written from exactly these sites, and **none reads
`chars.subtypes`**:

| file:line | path |
|---|---|
| `rules/layers.rs:342` | CR 708.2a face-down blank — clears |
| `rules/layers.rs:1624` | `LayerModification::CopyOf` (layer 1) — wholesale replace |
| `rules/layers.rs:1767` | `LayerModification::RemoveAllAbilities` (layer 6) — clears |
| `rules/layers.rs:1783` | `LayerModification::AddManaAbility` (layer 6) — **append-only** |
| `rules/copy.rs:86` | `get_copiable_values` |
| `rules/face.rs:115`, `rules/resolution.rs:891` | face change — rebuild base from the def |
| `state/builder.rs:1014`, `effects/mod.rs:9243-9259` | object / token construction from the spec |

`LayerModification::SetLandTypes` (`layers.rs:1729-1740`) touches **only `chars.subtypes`**. It does
not remove abilities. Blood Moon and Magus each carry **three** statics to compensate: layer-4
`SetLandTypes`, layer-6 `RemoveAllAbilities`, layer-6 `AddManaAbility({T}: Add {R})` — the last
listed after the second so its timestamp is later and it survives.

**The layer loop is genuinely layer-partitioned** (`layers.rs:159-170` ordered array,
`:359` outer loop, `:449-461` per-layer filter+sort, `:469-477` apply). "Run after layer 4 resolves"
is directly expressible, and the file already precedents per-layer inline blocks guarded by
`if layer == EffectLayer::TypeChange` (`:365`, `:388`, `:412`, `:439`).

---

## 3. Census at HEAD — the memo's 5 is a floor, and it is short by three

**Method 1 (the memo's stated rule, reproduced literally)**: scan `crates/card-defs/src/defs/*.rs`
comment-stripped for a land-type-conferring `LayerModification` whose payload names a basic land
subtype → **6 hits, minus `awaken_the_ancient`** (Mountain appears only in its `EnchantFilter`) =
**5**. This reproduces the memo exactly.

**Method 2 (inverse — start from the printed card, not from the layer modification)**: scan oracle
text for "basic land type" / "is a <basic> in addition to". This finds **three members the payload
method structurally cannot see**, because they do not confer a subtype through a
`LayerModification` at all:

| def | shape | `Complete`? | status after this batch |
|---|---|---|---|
| `awaken_the_woods` | creates a **token** that is a `Forest` land with `mana_abilities: vec![]` | **yes** (`#[default]` derive) | **4th live-wrong def — fixed for free** |
| `overlord_of_the_hauntwoods` | its `Everywhere` **token** hand-authors all five basic subtypes **and** all five mana abilities | **yes** (explicit) | **3rd double-grant risk — must not double** |
| `leyline_of_the_guildpact` | prints the Dryad clause, authors nothing | `Inert` | out of scope; its own note already says the clause is expressible |

**So the class is 8 defs, not 5**, and the derivation rule the memo published measures
*layer-modification payloads*, not *printed cards*. `awaken_the_woods` is the batch's own instance
of PB-DX26's durable lesson: **a roster derived from one declaration construct measures that
construct.** Both new members are recorded as roster rows so the class cannot silently regrow.

**Live-wrong today, all deck-legal `Complete`**: `urborg_tomb_of_yawgmoth`,
`yavimaya_cradle_of_growth`, `dryad_of_the_ilysian_grove` (every land they empower produces
nothing) and `awaken_the_woods` (its Forest token produces nothing).

---

## 4. Reachability — measured, not assumed

Every production consumer of `mana_abilities` already reads **layer-resolved** characteristics.
This is the whole reason the derivation belongs in `calculate_characteristics`:

| consumer | file:line | verdict |
|---|---|---|
| `Command::TapForMana` ability fetch | `rules/mana.rs:152-160` (`expect_characteristics`) | RESOLVED |
| offer layer `TapForMana` loop | `simulator/legal_actions.rs:1092-1094` | RESOLVED |
| mana solver `gather_sources` | `simulator/mana_solver.rs:395-397` | RESOLVED |
| `can_afford` / auto-tap | `legal_actions.rs:2218`, `local_game.rs:1111-1149` | RESOLVED (delegate) |
| `LegalAction`→`Command` any_color re-read | `simulator/params.rs:365-373` | RESOLVED |
| play-server / browser | `tools/play-server/src/view.rs:1352/1385/1422` | RESOLVED (transitive) |

**Therefore: 0 production lines outside the engine.** The offer layer, the mana solver and the human
channel see the derived ability the moment `calculate_characteristics` returns it. This is proven by
probe, not by this table (criterion 6506 demands the real activation path).

**The two sites that would have made it invisible, and why we avoid them**: `rules/face.rs:115` and
`rules/resolution.rs:891` **overwrite base** `characteristics.mana_abilities` wholesale from the
def on every face change. A derivation written into *base* characteristics (e.g. in
`enrich_spec_from_def`) would be silently erased there. It goes in the layer walk instead.

---

## 5. Design decisions, with reasons

### D1 — the derivation runs at the END of the layer-4 iteration, not after the whole walk

Insertion point: `layers.rs`, immediately after the `for effect in ordered { apply_layer_modification(..) }`
loop body closes (`:477`), guarded `if layer == EffectLayer::TypeChange`.

**Why not after the whole walk (post-`:521`)**: that would make the derived ability immune to layer-6
ability removal, which is CR-wrong. A Humility-style "lands lose all abilities" must remove the
intrinsic mana ability too, because CR 613.1f layer 6 applies after CR 305.6's layer-4 grant.
Placing it at end-of-layer-4 keeps the correct subordination and needs no dedup against Blood Moon
(see D2).

**Why end-of-layer-4 and not inside the `SetLandTypes`/`AddSubtypes` arms**: the subtype set must be
**fully resolved** first. Urborg's `AddSubtypes(Swamp)` and Blood Moon's `SetLandTypes(Mountain)` are
ordered by the CR 613.8 dependency arm (`layers.rs:2101-2103`); deriving inside an arm would derive
from an intermediate subtype set and re-open that dependency, which the brief's constraint (i)
names explicitly.

### D2 — `SetLandTypes` performs the CR 305.7 ability removal; both moons delete TWO statics

CR 305.7's ability loss is part of the type-setting effect, so it belongs in layer 4. Today it is
modelled as a *separate layer-6* `RemoveAllAbilities` on each moon def. Keeping that while deleting
only the mana grant would have the moons' own layer-6 removal **wipe the layer-4 derived ability** —
Blood Moon would stop working entirely. So the removal moves into the primitive:

`SetLandTypes(new_types)` additionally clears the five ability fields **iff the payload contains at
least one basic land type** (CR 305.7's own precondition: *"sets a land's subtype to one or more of
the **basic** land types"* — a hypothetical "becomes a Gate" sets a land type without triggering the
loss).

`blood_moon.rs` and `magus_of_the_moon.rs` then each drop **both** the layer-6 `RemoveAllAbilities`
**and** the layer-6 `AddManaAbility`, keeping one static apiece.

**This also closes a latent CR 305.7 violation.** CR 305.7's last sentence — *"Note that this
doesn't remove any abilities that were granted to the land by other effects"* — is violated today: a
blanket **layer-6** `RemoveAllAbilities` strips grants made by earlier-timestamped layer-6 effects
(Cryptolith Rite, Chromatic Lantern, Wrenn and Realmbreaker, The World Tree, Bootleggers' Stash all
grant into `LandsYouControl`/`AllLands`). Moving the removal to layer 4 makes every layer-6 grant
survive, which is what the rule says. Probed explicitly.

**Alternative considered and rejected**: leave the primitive alone and just re-label each moon's
`RemoveAllAbilities` to `EffectLayer::TypeChange`. Behaviourally equivalent for these two cards, and
rejected because it leaves CR 305.7 un-encoded — the next author to reach for `SetLandTypes` gets no
removal and no error. Encoding the rule once, in the primitive, is the same structural choice
PB-DX25 (`stack_registry`) and PB-DX20 (`enchant_target_to_requirement`) made.

### D3 — basic lands KEEP their hand-authored `{T}: Add`, and the derivation is idempotent

**Decision: do not touch `swamp.rs` / `forest.rs` / `island.rs` / `mountain.rs` / `plains.rs` or any
other def's printed mana ability.** Three reasons, in order of force:

1. **A registry-def gate would go red, correctly.**
   `crates/engine/tests/core/effect_choose_gate.rs:747`
   `every_complete_land_registers_each_printed_tap_mana_color` compares a def's **oracle text**
   against its **`enrich_spec_from_def` lowering** (`:657-665`) — a pure registry path with no
   `GameState` and no layers. Deleting the printed ability makes every basic land report
   `missing {B}` etc. The gate is right: a def that *prints* "{T}: Add {B}" and lowers nothing is
   a def whose spec lies about the card. CR 305.6 says a Swamp does not *need* the printed text; it
   does not say a Swamp that *has* the printed text should stop declaring it.
2. **`Command::TapForMana.ability_index` is a dense index into `mana_abilities`**
   (`rules/command.rs:25-29`, consumed `mana.rs:152-160`). A basic Swamp's printed ability is
   index 0 today. Idempotent derivation keeps it at index 0; deletion would move every basic land's
   ability from base into a derived append, changing the replay-log index space for the single most
   common object in the game (**OOS-DX26-3** hazard, discharged by test).
3. **`face.rs:115` / `resolution.rs:891` rebuild base `mana_abilities` from the def.** A basic land
   whose def declared nothing would have an empty base vector at every face change.

**Idempotence is therefore load-bearing, not a nicety.** The derivation appends
`{T}: Add [symbol]` for a basic subtype **only if no equivalent unconditional tap-for-one ability is
already present**. "Equivalent" is defined structurally (D4).

This is also exactly what closes `OOS-DX27-10` **without a `push_back` dedup guard**: two moons both
set `{Mountain}`, the set-valued subtype is idempotent, and one derivation pass appends one `{R}`.

### D4 — what counts as "already present"

An existing `ManaAbility` discharges the intrinsic for colour `c` iff **all** of:
`produces == {c: 1}`, `requires_tap`, `!sacrifice_self`, `!any_color`, `damage_to_controller == 0`,
`mana_cost.is_none()`, `life_cost == 0`, `scaled_amount.is_none()`,
`activation_condition.is_none()`, `!exile_self_from_hand`, `remove_counter.is_none()`.

Written as an **exhaustive struct destructure with no `..` rest pattern**, so a future field added to
`ManaAbility` is a compile error until classified — the SR-5 idiom. A conditioned or costed
ability (SR-37's `activation_condition`, a Phyrexian-style `life_cost`) does **not** discharge the
intrinsic, because CR 305.6's ability is unconditional: a land with a restricted `{T}: Add {B}` that
becomes a Swamp genuinely gains a second, unrestricted one.

### D5 — derivation order is CR 305.6's own listed order

Plains {W}, Island {U}, Swamp {B}, Mountain {R}, Forest {G} — a fixed array, not `OrdSet` iteration
order, so the appended order is documented and stable rather than incidentally alphabetical.

### D6 — scope: every zone, gated on the Land card type

CR 305.6 says "an object with the land card type and a basic land type", with no zone restriction.
The derivation is applied uniformly wherever `calculate_characteristics` runs, gated on
`chars.card_types.contains(&CardType::Land)`.

**CR 708.2a falls out for free**: the face-down blank (`layers.rs:329-342`) runs *before* the layer
loop and sets `card_types = {Creature}` and `subtypes = {}`. A face-down land is neither a Land nor
of any basic type, so the derivation yields nothing. Asserted, not assumed.

### D7 — wire prediction

**PROTOCOL unmoved / HASH unmoved.** No new type, no new variant, no new field: the derivation is a
*computation* over existing `Characteristics` fields, and `hash.rs` hashes **base**
`obj.characteristics`, not the resolved value (`hash.rs:2286`, `:6137`) — the same reason
`AddManaAbility` grants have never moved a state hash. The brief predicted "HASH LOW / PROTOCOL
none"; this plan sharpens that to *both unmoved*, and **both are taken from the gates' own output,
not from this paragraph.**

---

## 6. Implementation steps

### S1 — `crates/card-types/src/state/types.rs`
Add beside `ALL_LAND_TYPES`:
- `BASIC_LAND_TYPES: [(&str, ManaColor); 5]`-shaped source of truth in CR 305.6 order, and
- `pub fn basic_land_type_mana_color(st: &SubType) -> Option<ManaColor>` returning the CR 305.6
  symbol for a basic land subtype, `None` otherwise.
Cite CR 305.6 in the doc comment. Keep `ALL_LAND_TYPES` as the superset it is, and add a
`debug_assert`-style unit test that every basic type is a member of `ALL_LAND_TYPES`.

### S2 — `crates/engine/src/rules/layers.rs`, the derivation
New `fn derive_intrinsic_land_mana_abilities(chars: &mut Characteristics)`:
- return immediately unless `chars.card_types.contains(&CardType::Land)`;
- for each `(subtype, color)` in CR 305.6 order, if `chars.subtypes.contains(subtype)` and no
  existing ability satisfies D4's predicate, `push_back(ManaAbility::tap_for(color))`.
Called from the layer loop under `if layer == EffectLayer::TypeChange` immediately after the
`for effect in ordered` loop, with the CR 305.6 / 613.1d / 613.1f layer-order argument written in
source at the call site.

Helper `fn discharges_intrinsic_mana_ability(ma: &ManaAbility, color: ManaColor) -> bool`
implementing D4 as an exhaustive destructure.

### S3 — `crates/engine/src/rules/layers.rs`, the `SetLandTypes` arm
Extend `:1729-1740` to also clear `keywords` / `mana_abilities` / `activated_abilities` /
`triggered_abilities` / `abilities` **iff** the payload intersects the basic land types. Cite
CR 305.7 sentence 2 and its "doesn't remove abilities granted by other effects" clause, and state
why layer 4 (not 6) is the correct home. Update the arm's existing doc comment, which currently
says it leaves everything but `subtypes` untouched.

### S4 — card defs
- `blood_moon.rs`: delete the layer-6 `RemoveAllAbilities` static and the layer-6 `AddManaAbility`
  static; rewrite the header comment to cite CR 305.6/305.7 and the new primitive behaviour.
- `magus_of_the_moon.rs`: same.
- No other def is edited. `overlord_of_the_hauntwoods`'s hand-authored token abilities are
  **deliberately kept** — they are the token's *base* characteristics (what `face.rs`-style rebuilds
  and every spec reader see), and D3/D4 make the derivation skip them. Proven by probe, not asserted.

### S5 — tests (all revert-proven red)
New `crates/engine/tests/rules/pb_dx43_intrinsic_land_mana.rs`:
- **P1-P3** the three staples through `calculate_characteristics`: a Plains under Urborg has {B};
  under Yavimaya {G}; under the Dryad all five.
- **P4** two-moon fixture → **exactly one** `{T}: Add {R}` (`OOS-DX27-10`, the evidence the memo says
  is the minimum: PB-DX27's `t6` builds a one-moon board).
- **P5** idempotence: Urborg + a basic Swamp → still exactly one `{B}`, at **index 0**.
- **P6** CR 305.7 removal: Ancient Den under Blood Moon loses its printed `{W}` and has exactly `{R}`.
- **P7** CR 305.7 last sentence: a layer-6 grant with an **earlier** timestamp than the moon survives.
- **P8** CR 708.2a: a face-down Swamp derives nothing.
- **P9** `RemoveAllAbilities` (layer 6) still removes the derived ability — the subordination in D1.
- **P10** multi-basic and snow-covered cases.
- **P11** a non-land object with a basic-land subtype derives nothing.
- **P12** a conditioned/costed existing ability does **not** discharge the intrinsic (D4).

New `crates/simulator/tests/pb_dx43_intrinsic_mana_channel.rs` — **criterion 6506's real-path
evidence**, not characteristics inspection:
- **C1** the offer layer emits a `TapForMana` for a Plains under Urborg, and driving that
  `LegalAction` through `params.rs` → `process_command` actually puts `{B}` in the pool.
- **C2** the mana solver funds a `{B}` cost off a Plains under Urborg (auto-tap path).
- **C3** the same for Yavimaya {G} and the Dryad's any-colour case.
- **C4** `awaken_the_woods`' Forest token taps for `{G}` through the real path.
- **C5** `overlord_of_the_hauntwoods`' Everywhere token offers exactly five, not ten.

New `crates/engine/tests/core/pb_dx43_land_type_roster.rs` — SR-36 roster gate walking
`all_cards()` (never grepping source):
- **R1** the conferring population by the memo's payload rule, pinned by name.
- **R2** the **inverse** population by printed text, pinned by name — the axis that found
  `awaken_the_woods` and `overlord_of_the_hauntwoods`; non-vacuity floor asserted.
- **R3** every `Complete` def whose token spec grants a basic land subtype either authors the
  matching mana ability or is discharged by the derivation — fails when a new one is authored.
- **R4** ability-index neutrality: for every `Complete` land, the resolved `mana_abilities` prefix
  equals the base spec's, so no existing `TapForMana` index moved (**OOS-DX26-3**).

### S6 — gates
`hash_schema` + `protocol_schema` executed and the numbers **read off**; full-workspace re-run and
set-diff by test NAME; `tools/authoring-report.py` regenerated (0 flips expected — every def touched
is already `Complete` and no completeness marker changes); `clippy --workspace --all-targets -D
warnings`; `cargo fmt --check`; `tools/check-defs-fmt.sh`.

---

## 7. Known hazards for the implementer

1. **`no_dependency_cycle_is_constructible_from_current_relation`** (`layers.rs:1998-2020`, SR-30)
   guards `depends_on`. This batch adds **no** new `depends_on` arm; if you find yourself adding
   one, stop and re-read D1.
2. **`t5`/`t6`/`t7`/`t8` in `pb_dx27_blood_moon_type_scope.rs` must stay green** and must not be
   weakened. `t6`'s doc comment predicts this batch by name; update it to record the closure rather
   than deleting it.
3. **`every_complete_land_registers_each_printed_tap_mana_color`** must stay green — it is the gate
   D3 is decided by. If it goes red, the basics decision was violated somewhere.
4. **SR-9a**: tests go in the existing group directories (`tests/rules/`, `tests/core/`) with a
   `mod` line added; never a new top-level `tests/*.rs`.
5. **SR-35**: `cargo fmt` checks none of the 1,803 card defs — run `tools/check-defs-fmt.sh`.
6. The derived ability makes lands offer a `TapForMana` where they offered none. Fuzz/seeded
   fixtures that depended on a land producing nothing may move; re-observe any seeded constant
   rather than editing it to taste.
