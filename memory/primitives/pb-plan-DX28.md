# PB-DX28 — implementation plan

**Task**: `scutemob-210`. **Seeds**: `OOS-DX4-6` (untargeted-choice class) + `OOS-DX4-1` (owner axis).
**Queue**: `memory/primitives/seed-rerank-2026-08-02.md` §4 rank 12 (table-only row; this file is the
brief).

**Baseline, measured on this branch BEFORE any edit** — tests **4,605 / 0 / 5**, 46
result-producing targets; **PROTOCOL 36 / HASH 75**; coverage 1,136/1,803 = 63.0%.

---

## 0. Census (AC 6448) — the two classes, by enumeration

Two independent axes were run, neither of them a grep of the registry's member names.

**Axis A (slot arithmetic, `all_cards()` walk).** For every def, sum every declared
`TargetRequirement` slot across `Activated`/`Triggered`/`Spell`/`LoyaltyAbility`/`SagaChapter`/
`ClassLevel` and both extra faces, and compare with the number of `"target"` occurrences in the
combined oracle text. `slots > words` is the candidate signal. 50 rows, 34 of them `Complete`.

**Axis B (inverse, printed text first).** Scan every def's *printed* oracle text for ownership
needles (`you own`, `opponent owns`, `owned by`, `into your graveyard`, `its owner controls`, …)
with no reference to what the def declares. 63 rows.

Both axes are **floors**: axis A cannot see a def whose oracle happens to spend its `"target"`
budget on a different ability, and axis B cannot see an ownership clause phrased without a needle.

### 0.1 Untargeted-choice class — `Complete` members (18; the seed said "≥14")

| # | def | printed clause (no "target" — CR 115.10) | authored as | disposition |
|---|---|---|---|---|
| 1-10 | the ten Karoos (`azorius_chancery`, `boros_garrison`, `dimir_aqueduct`, `golgari_rot_farm`, `gruul_turf`, `izzet_boilerworks`, `orzhov_basilica`, `rakdos_carnarium`, `selesnya_sanctuary`, `simic_growth_chamber`) | "return **a land you control** to its owner's hand" | `TargetPermanentWithFilter(Land + You)` | MIGRATE |
| 11 | `cloud_of_faeries` | "untap **up to two lands**" | `UpToN{2, TargetLand}` | MIGRATE — **NEW, not in the seed** |
| 12 | `frantic_search` | "untap **up to three lands**" | `UpToN{3, TargetLand}` | MIGRATE |
| 13 | `rewind` | "Counter target spell. Untap **up to four lands**." | slot 0 `TargetSpell` (correct) + slot 1 `UpToN{4}` | MIGRATE slot 1 only — **NEW** |
| 14 | `shrieking_drake` | "return **a creature you control** to its owner's hand" | `TargetCreatureWithFilter(You)` | MIGRATE |
| 15 | `whitemane_lion` | same | same | MIGRATE |
| 16 | `sword_of_truth_and_justice` | "put a +1/+1 counter on **a creature you control**" | `TargetCreatureWithFilter(You)` | MIGRATE |
| 17 | `takenuma_abandoned_mire` | "return **a creature or planeswalker card from your graveyard** to your hand" | `TargetCardInYourGraveyard(...)` | MIGRATE — **NEW**, and the only **graveyard**-zone member |
| 18 | `sword_of_war_and_peace` | "deals damage to **that player**" | `TargetRequirement::TargetPlayer` | REPAIR — **NEW**, a *player* clause, so a different repair (see §3) |

**Refuted candidates** (axis A surfaced them; adjudication cleared each):

* every `Equip`-carrying Equipment (`batterskull`, `bone_saw`, `kite_shield`, `paradise_mantle`,
  `swiftfoot_boots`, `helm_of_the_host`, `sword_of_body_and_mind`, `sword_of_feast_and_famine`,
  `sword_of_vengeance`, `umezawas_jitte`, and the equip half of every Sword) — CR 702.6a's granted
  ability *does* say "target creature you control"; the printed line is only the cost. PB-DX26
  authored these deliberately.
* `curtains_call` ("Destroy **two** target creatures"), `huddle_up` ("**Two** target players"),
  `victimize` ("Choose **two** target creature cards") — one `"target"` word, two real slots.
* `sword_of_fire_and_ice` / `sword_of_light_and_shadow` / `sword_of_sinew_and_steel` — the trigger
  half genuinely prints "target"/"any target".

### 0.2 Owner class — `Complete` members (2), and four refutations that matter

| def | printed clause | authored as | disposition |
|---|---|---|---|
| `staff_of_compleation` | "Destroy target permanent **you own**" (CR 108.3) | `TargetController::You` (CR 109.4) | REPAIR — owner axis |
| `nether_traitor` | "Whenever another creature is put **into your graveyard** from the battlefield" (CR 404.3 — the graveyard's owner is the card's owner) | `WheneverCreatureDies { controller: Some(You) }` | REPAIR — owner-scoped trigger |

Not `Complete`, listed so the class is counted rather than the cards: `athreos_god_of_passage`
("another creature **you own** dies", `partial`, and its own note already names this gap),
`hellkite_courser` ("a commander **you own**", `partial`), `maskwood_nexus` ("creature cards **you
own**", `partial`), `mishra_claimed_by_gix` ("you both **own and control** them", `partial`),
`leyline_of_the_void` ("a card an opponent **owns**", `known_wrong`).

**REFUTED, and each refutation is load-bearing:**

* **The six mutate defs** (`brokkos_apex_of_forever`, `gemrazer`, `sea_dasher_octopus`,
  `necropanther` — all `Complete` — plus `mindleecher`, `nethroi_apex_of_death`) print "put it over
  or under target non-Human creature **you own**" (CR 702.140a). They are **not** members:
  `casting.rs`'s mutate validation checks `target_obj.owner != player` **open-coded**, not through
  `TargetFilter`. The ownership restriction is honoured today. This is the largest single group the
  census found and it is entirely clean.
* **`fecundity` is not a member, and `nether_traitor`'s allowlist note says it is.** That note names
  "`athreos`, `fecundity`" as further instances of the corpus-standard controller-for-owner
  expression. `athreos` is one. `fecundity` is not: its printed clause is "**that creature's
  controller** may draw a card" and its gap is `PlayerTarget::ControllerOf(TriggeringCreature)` —
  a *controller* gap, not an ownership approximation. Its own `partial` note says exactly that.
  The citation is corrected in place by this batch.
* `hanweir_battlements` ("If you both **own and control** this land and a creature named …") —
  `Effect::Meld` checks `obj.owner == controller && obj.controller == controller`
  (`effects/mod.rs:5067-5068`). Correct today.

---

## 1. The untargeted-choice channel (AC 6449)

### 1.1 DSL — `crates/card-types/src/cards/card_definition.rs`

```rust
/// CR 115.10 / CR 608.2: which zone an untargeted resolution-time choice draws from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceZone {
    /// Permanents on the battlefield (CR 400.1). Phased-out permanents are excluded
    /// (CR 702.26b), matching every other battlefield enumeration in this file.
    Battlefield,
    /// Cards in the CHOOSING player's graveyard (CR 404.1). "your graveyard".
    YourGraveyard,
}
```

and a new `EffectTarget` variant:

```rust
    /// CR 115.10: an object chosen **on resolution, without targeting**.
    ///
    /// A spell or ability targets only where its text says "target". A printed
    /// "return a land you control to its owner's hand" therefore (a) is unaffected
    /// by hexproof / shroud / protection / "can't be the target of", and (b) has no
    /// CR 608.2b fizzle window, because nothing was chosen when the ability went on
    /// the stack.
    ///
    /// Resolved through the CR 608.2d suspend-and-replay channel
    /// (`EffectChoiceQuestion::ChooseObject`) and banked on
    /// `EffectContext.chosen_objects`, keyed **by this variant's own value**.
    ChosenObject {
        zone: ChoiceZone,
        filter: Box<TargetFilter>,
        /// How many objects the chooser picks. `up_to == false` means exactly this
        /// many *if that many exist*, CR 608.2 "as much as possible".
        count: u32,
        up_to: bool,
    },
```

**Keyed by value — this is deliberate and load-bearing.** `azorius_chancery` names the same
`ChosenObject` twice in one effect (`MoveZone.target` and `MoveZone.to.Hand.owner =
PlayerTarget::OwnerOf(ChosenObject)`) and both must denote the *same* chosen land. The cost of
keying by value is that two structurally identical `ChosenObject`s in one resolution that were
meant to be two separate choices would collapse into one; no corpus def does that, R3 pins the
population, and it is filed as a residual seed rather than left unsaid.

### 1.2 State — `crates/card-types/src/state/stubs.rs`

```rust
    /// CR 115.10 / 608.2d: the objects a resolution-time UNTARGETED choice may pick
    /// from, in ascending `ObjectId` order.
    ///
    /// Unlike the four existing variants this names **public** information: every id
    /// is a permanent on the battlefield or a card in a graveyard, both public zones
    /// (CR 400.2). `private_to()` still returns `Some(player)` — the question is
    /// addressed to one seat — but no hidden information rides on it.
    ChooseObject { candidates: Vec<ObjectId>, count: u32, up_to: bool },
```
and `EffectChoiceAnswer::ChooseObject { chosen: Vec<ObjectId> }`.

Answer legality (`effects::handle_answer_effect_choice`, the existing
`(question, answer)` match): `chosen` has no duplicates, every id is drawn from the question's
`candidates`, and `chosen.len() == count` when `!up_to`, `<= count` when `up_to` — with the
CR 608.2 "as much as possible" clamp `min(count, candidates.len())` applied to the `!up_to` case.

`default_effect_choice_answer` / `default_discard_answer`'s siblings gain a `ChooseObject` arm
returning the first `min(count, candidates.len())` candidates — the deterministic recovery of the
pre-batch auto-pick, and the bot/simulator fallback.

### 1.3 `EffectContext` — `crates/engine/src/effects/mod.rs`

```rust
    /// CR 115.10: answers to this resolution's untargeted object choices, keyed by
    /// the `EffectTarget::ChosenObject` value that asked. NOT hashed and NOT on the
    /// wire — `EffectContext` is per-resolution scratch.
    pub chosen_objects: Vec<(EffectTarget, Vec<ObjectId>)>,
```

### 1.4 The ask, and where it happens

At the top of `execute_effect`, before the big `match`:

```rust
if !resolve_pending_object_choices(state, effect, ctx) {
    return Vec::new();   // suspended; the wrapper rolls the whole resolution back
}
```

`resolve_pending_object_choices` matches **only the supported arms** and, for each
`EffectTarget::ChosenObject` it finds there that is not already banked in `ctx.chosen_objects`:

1. derives candidates — `ChoiceZone::Battlefield`: `obj.zone == Battlefield && obj.is_phased_in()`
   and `matches_filter(&expect_characteristics(state, id), filter)` plus the runtime axes
   `matches_filter` cannot see (`controller`, `owner`, `exclude_self`, `is_token`/`is_nontoken`,
   `is_tapped`/`is_untapped`, `is_attacking`/`is_blocking`, `has_counter_type`) — reusing the
   existing helpers rather than re-deriving them; `ChoiceZone::YourGraveyard`:
   `obj.zone == ZoneId::Graveyard(ctx.controller)` + `matches_filter`;
2. short-circuits when the answer set is DETERMINED — `candidates.is_empty()`, or
   (`!up_to && candidates.len() <= count`) — banking that set with no round trip. This is the same
   argument the discard arm makes at `n == 0 || n >= hand.len()`, and it is what keeps a Karoo with
   one land from costing a question;
3. otherwise calls `ask_or_consume_effect_choice(state, ctx, ctx.controller, question)`; `None`
   means suspended → return `false`.

**Supported arms** (the whole corpus population, §0.1): `Effect::MoveZone { target, to }`,
`Effect::AddCounter { target }`, `Effect::UntapPermanent { target }`. `Effect::Sequence` /
`Conditional` / `ForEach` need no arm — they call `execute_effect` per child, so each child gets
its own pre-pass at its own granularity.

**Fail-closed, two ways.** (a) `resolve_effect_target_list_indexed`'s `ChosenObject` arm returns
**empty** when nothing is banked, with a `state::diagnostics` `expect_*` record — an SR-4
engine-bug classification, not an LKI fizzle, because reaching it means the pre-pass did not cover
the position. (b) `pb_dx28_chosen_object_roster.rs` **R3** pins the exact set of corpus defs that
name `ChosenObject`, so a 19th use reddens and the author must confirm the arm is supported.

### 1.5 Card-def migration (17 defs)

| def | from | to |
|---|---|---|
| 10 Karoos | `targets: [TargetPermanentWithFilter(Land+You)]`, `MoveZone{DeclaredTarget{0}}` | `targets: []`, `MoveZone{ ChosenObject{Battlefield, Land+You, 1, false} }`, and the `Hand{owner}` `OwnerOf` argument rewritten to the same `ChosenObject` value |
| `whitemane_lion`, `shrieking_drake` | `TargetCreatureWithFilter(You)` | `ChosenObject{Battlefield, Creature+You, 1, false}` |
| `sword_of_truth_and_justice` | `TargetCreatureWithFilter(You)` | `ChosenObject{Battlefield, Creature+You, 1, false}` on `AddCounter.target` |
| `cloud_of_faeries` | `UpToN{2, TargetLand}` + 2 `UntapPermanent` | `targets: []`, ONE `UntapPermanent{ ChosenObject{Battlefield, Land, 2, true} }` |
| `frantic_search` | `UpToN{3, TargetLand}` + 3 `UntapPermanent` | ONE `UntapPermanent{ ChosenObject{Battlefield, Land, 3, true} }` |
| `rewind` | slot 0 `TargetSpell` **kept**, slot 1 `UpToN{4}` removed | ONE `UntapPermanent{ ChosenObject{Battlefield, Land, 4, true} }`; `CounterSpell` keeps `DeclaredTarget{0}` |
| `takenuma_abandoned_mire` | `TargetCardInYourGraveyard(Creature|Planeswalker)` | `ChosenObject{YourGraveyard, that filter, 1, false}` |

**`rewind` is the index hazard.** Removing slot 1 leaves slot 0 as the only requirement, and
`DeclaredTarget { index: 0 }` still names the countered spell — unchanged. But the def's long
in-source comment about pooled indexing is now describing something that no longer exists and must
be rewritten, not left.

### 1.6 Probes (AC 6449) — `crates/engine/tests/primitives/pb_dx28_untargeted_choice.rs`

Both defect directions, each revert-proven:

* **Direction 1 — eligibility.** A Karoo ETB with two lands you control, one of them given
  hexproof/shroud by a continuous effect: the hexproofed land IS in `candidates` and IS legally
  choosable. Sibling: `sword_of_truth_and_justice` can put its counter on a shrouded creature.
  Both fail on the pre-batch code because target validation refuses.
* **Direction 2 — no fizzle window.** Karoo ETB goes on the stack; in response the opponent
  removes a land; on resolution the choice is made from what is *then* on the battlefield and the
  ability still returns a land. Sibling: `sword_of_truth_and_justice` still **proliferates** when
  the creature it would have chosen is removed in response — the pre-batch CR 608.2b fizzle killed
  the counter *and* the proliferate.
* The trigger goes on the stack with **zero** declared targets (CR 603.3d cannot remove it).
* `frantic_search`/`rewind`/`cloud_of_faeries`: `up_to` lets the chooser take fewer, and the
  determined short-circuit fires when candidates ≤ count.
* `takenuma_abandoned_mire`: a graveyard-zone choice, and its **mill still happens** when the
  card it would have chosen is exiled in response.

---

## 2. The owner axis (AC 6450)

### 2.1 `TargetFilter.owner`

```rust
/// CR 108.3 — whose OWNERSHIP an object must be under. Distinct from
/// [`TargetController`] (CR 109.4): they diverge under any control-change effect.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetOwner { #[default] Any, You, Opponent }
```
field `#[serde(default)] pub owner: TargetOwner` on `TargetFilter`.

`GameObject.owner` is a runtime field, **not** a `Characteristics` property, so — exactly like
`exclude_self` / `is_attacking` — `matches_filter()` cannot see it and it MUST be checked
explicitly at each call site. Sites, mirroring `exclude_self`'s documented list:

* `casting::validate_object_satisfies_requirement` — the four filter-carrying requirements
  (`TargetCreatureWithFilter`, `TargetPermanentWithFilter`, `TargetCardInYourGraveyard`,
  `TargetCardInGraveyard`);
* `rules::abilities`' auto-target picker for triggered abilities;
* `rules::queries::spell_target_requirements`' consumers inherit it for free (PB-DX20 made the
  offer layer and the cast path one arithmetic — verify, do not assume);
* `filter_states_a_quality` (`effects/mod.rs`) gains `owner` to its **exclusion list**: ownership
  is a runtime board property, not a CR 701.23b "stated quality", and a card in a library has an
  owner but the axis narrows nothing there.

Repair: `staff_of_compleation` → `TargetPermanentWithFilter(TargetFilter { owner: TargetOwner::You, ..default })`,
`controller: TargetController::You` **removed** (it was the approximation, keeping it would make
the card strictly narrower than printed).

### 2.2 The owner-scoped trigger (the census demands it — `nether_traitor` is `Complete`)

`TriggerCondition::WheneverCreatureDies` gains `#[serde(default)] owner: Option<TargetOwner>`,
lowered into `DeathTriggerFilter` beside the existing `controller`. CR 603.10a's look-back applies
to ownership the same way it applies to control: the dying creature's **pre-death owner** is used
(ownership does not change on death, but the object does — CR 400.7 — so the LKI read is the
correct one and must be the one used).

Repair: `nether_traitor` → `controller: None, owner: Some(TargetOwner::You)`.

### 2.3 Probes (AC 6450) — `crates/engine/tests/primitives/pb_dx28_owner_axis.rs`

Both control-change directions, on both members, each revert-proven:

* **owned-but-opponent-controlled is LEGAL where printed.** Opponent gains control of a permanent
  you own → `staff_of_compleation` may still destroy it (pre-batch: refused). `nether_traitor`'s
  trigger fires when that creature dies (pre-batch: silent).
* **controlled-but-not-owned is ILLEGAL where printed.** You gain control of an opponent's
  permanent → `staff_of_compleation` may **not** destroy it (pre-batch: allowed).
  `nether_traitor` does **not** fire when it dies (pre-batch: fired).
* `TargetOwner::Opponent` and the default `Any` each get a probe so the enum is not half-dead.

---

## 3. `sword_of_war_and_peace` (AC 6448's census find)

The printed clause is "deals damage to **that player**" — determined, not targeted — and the def's
own comment says "DamagedPlayer resolves from ctx.damaged_player at resolution" while the code
declares `TargetRequirement::TargetPlayer` and reads `DeclaredTarget { index: 0 }`. **The comment
asserts a mechanism the code does not use** (the PB-DX27 stale-note class, live). The auto-target
picker chooses *a* player, so in a 4-player game the Sword damages the wrong seat.

`PlayerTarget::DamagedPlayer` already exists but `Effect::DealDamage.target` is an `EffectTarget`,
which has no damaged-player variant. Add `EffectTarget::DamagedPlayer`, resolving from
`ctx.damaged_player` exactly as `PlayerTarget::DamagedPlayer` does, and rewrite the def to
`targets: vec![]`. One probe: a 4-player board where the damaged player is NOT the first opponent
in seat order, proving the damage lands on the damaged seat.

---

## 4. Allowlist retirement (AC 6451)

Remove the `sword_of_truth_and_justice` and `staff_of_compleation` entries from
`completeness_deviation_scan.rs`'s `ALLOWLIST`, and rewrite `nether_traitor`'s (its whole reason
text is the now-false "the DSL has no owner-scoped death trigger", and its `fecundity` citation is
wrong — see §0.2). Then **prove the scan still reddens** by planting one instance of each class in
a scratch def and executing the scan: argued removals are how an allowlist rots.

## 5. Wire (AC 6452)

Predicted: `EffectTarget` gains two variants and `ChoiceZone`/`TargetOwner` are new types inside
the `Effect` closure; `EffectChoiceQuestion`/`EffectChoiceAnswer` each gain a variant;
`TargetFilter` gains a field; `TriggerCondition::WheneverCreatureDies` gains a field. That is
**one** PROTOCOL bump and **one** HASH bump. **Take both numbers from the failing gates' own
output — never predict them** (PB-DX27's brief predicted "NONE" and the gate refuted it). Sentinel
re-pins by SYMBOL. Both histories append-only.

## 6. Coverage

0 flips expected: every migrated def is already `Complete` and stays `Complete`. Regenerate
`tools/authoring-report.py` and name any flip that happens anyway.
