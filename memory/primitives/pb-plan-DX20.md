# Primitive Batch Plan: PB-DX20 — the offer layer cannot see a keyword-carried target requirement

**Generated**: 2026-08-04
**Primitive**: a *total*, shared `EnchantTarget -> TargetRequirement` synthesis so the offer layer
and `handle_cast_spell` derive an Aura's CR 303.4a target requirement from **literally the same
function**; plus the CR 702.151a "another target creature you control" requirement on the
Reconfigure attach synth site.
**CR rules**: 303.4 / 303.4a / 303.4d / 303.4g, 702.5 / 702.5a / 702.5c / 702.5d, 702.103a-b,
702.151a, 601.2c, 115.6, 704.5m, 205.4a, 109.1
**Seeds**: `OOS-CARDS2-4` (HIGH) + `OOS-CARDS1-2`; touches `OOS-SIM4-2`, `OOS-SIM5-4`
**Cards affected**: **14 deck-legal `Complete` defs** (13 Auras + `boon_satyr` via Bestow) +
**1 `Complete` Reconfigure** (`lizard_blades`) — **0 card-def edits**
**Dependencies**: none (every type this plan uses already exists)
**Wire**: **none expected** — no new `TargetRequirement` variant, no new `Command`/`GameEvent`/
`Effect`/`Characteristics` field. **Gate-execute PROTOCOL/HASH anyway; do not assume.**
**Deferred items from prior PBs consumed here**: `OOS-CARDS1-2` (filed by CARDS-1 as the
half it deliberately did not fix). `OOS-SIM5-4` (offer suppression) stays deferred — see §8.

---

## 0. Baselines and measurements — every number in this plan names the command that produced it

Re-verify each of these at HEAD **before any edit** and record the output. If a number moved,
the plan's arithmetic is stale and the divergence is a finding, not something to paper over.

| # | claim | command that measured it | value |
|---|---|---|---|
| 0.1 | pre-edit workspace tests | `cargo test --workspace --no-fail-fast` (to a file, never `\| tail`) | **4,373 / 0 / 5** (coordinator-measured; re-measure) |
| 0.2 | defs whose source mentions `"Aura"` | `rg -l '"Aura"' crates/card-defs/src/defs` | **27 files** |
| 0.3 | defs carrying `KeywordAbility::Enchant(` | `rg -l 'KeywordAbility::Enchant\(' crates/card-defs/src/defs` | **23 files** |
| 0.4 | of those 23, explicitly `partial` | `rg -n 'completeness' <the 23>` | **10** (`kayas_ghostform`, `aqueous_form`, `elvish_guidance`, `curiosity`, `smoke_shroud`, `shiny_impetus`, `breath_of_fury`, `ophidian_eye`, `bear_umbra`, `crown_of_skemfar`) |
| 0.5 | ⇒ `Complete` by the `#[default]` derive | 0.3 − 0.4 | **13** |
| 0.6 | Bestow defs | `rg -l 'AbilityDefinition::Bestow' crates/card-defs/src/defs` | **2** — `boon_satyr` (**`Completeness::Complete`, explicit**, line 78) and `springheart_nantuko` (`inert`) |
| 0.7 | Aura defs carrying an `AbilityDefinition::Spell` | `rg -l 'AbilityDefinition::Spell' <the 24 Aura/Bestow defs>` | **0** |
| 0.8 | live `EnchantFilter` field usage | `rg -n 'EnchantFilter' -A6 crates/card-defs/src/defs` | only `has_card_type`, `has_subtype`, `basic`, `controller: You` |

### 0.9 — CORRECTION to the brief: "27 Aura defs / 4 without `Enchant`" is a grep artefact

The brief (and `seed-rerank-2026-08-02.md:854`) names `animate_dead`, `curse_of_opulence`,
`open_the_armory`, `sram_senior_edificer` as "**4 Aura defs** that declare no `Enchant` keyword".
Measured:

* `sram_senior_edificer.rs:17` — `full_types(...)` **Legendary Creature — Dwarf Artificer**. Its
  `"Aura"` is a `has_subtypes` entry inside a spell-cast **trigger filter** (`:34`).
* `open_the_armory.rs:15` — `types(&[CardType::Sorcery])`. Its `"Aura"` is a `SearchLibrary`
  filter entry (`:23`).

Neither is an Aura permanent. **The true set is 2**: `animate_dead` (`inert`) and
`curse_of_opulence` (`inert`). Acceptance criterion 3 says "pin what is measured" — so the probe
pins **2**, and its doc comment records why the brief said 4. The rosters in §6 T4 must be built
by enumerating `mtg_engine::all_cards()` and testing
`def.types.subtypes.contains(&SubType("Aura".into()))` — **never** by grepping source (SR-36).

### 0.10 — MANDATORY pre-existing-TODO sweep (roster-recall gate) — RUN, 5 relevant hits

`rg -n -i 'TODO.*(Aura|Enchant|enchant)' crates/card-defs/src/defs` → 16 hits;
`rg -n 'TODO.*([Rr]econfigure|target_min|offer layer|TargetRequirement|targets: vec!\[\])'
crates/card-defs/src/defs` → 2 hits. Dispositions:

| site | TODO text | disposition |
|---|---|---|
| `curse_of_opulence.rs:20` | *"'Enchant player' not in EnchantTarget enum"* | **FALSE** — `EnchantTarget::Player` exists (`types.rs:377`). Aspirationally-wrong comment (conventions.md §"Aspirationally-wrong code comments"). Its own `completeness` note at `:25` already contradicts it. → **seed `OOS-DX20-4`**, no edit (brief pins `git diff -- crates/card-defs` EMPTY; the def stays `inert` either way because player *attachment* is genuinely unimplemented). |
| `kayas_ghostform.rs:22` | *"DSL gap — 'Enchant creature or planeswalker'"* | **FALSE** — `EnchantTarget::CreatureOrPlaneswalker` exists; the def's own note at `:30` says so and adds that `Enchant(Creature)` "wrongly narrows legal targets and drops 'you control'". PB-DX20 makes this **visible** (the browser will now offer a creature-only picker for a card whose printed line is wider). `partial`, not deck-legal. → **seed `OOS-DX20-5`**, no edit. |
| `animate_dead.rs:9` | *"no EnchantTarget variant for [graveyard card]"* | **TRUE** and genuinely out of scope. `inert`. → pinned by T4. |
| `open_the_vaults.rs:27` | *"TODO(M10+): Add Aura placement choice"* | CR 303.4f (ETB placement), a **different** primitive. Not in scope. |
| `polymorphists_jest.rs:6` | *"no `TargetRequirement::TargetPlayer`"* | **FALSE** — it exists (`card_definition.rs:2948`). Not keyword-carried, so out of this batch. → **seed `OOS-DX20-6`**. |

The remaining 11 hits are unrelated trigger/DSL gaps (`aqueous_form`, `shiny_impetus`,
`breath_of_fury`, `elvish_guidance`, `enduring_*`, `destiny_spinner`, `tear_asunder`).

**No forced adds.** All five relevant hits are card-def-side and the brief pins zero card-def
edits; each is routed to a seed instead. This is the positive assertion the gate asks for.

---

## 1. CR rule text (verbatim, MCP-retrieved 2026-08-04)

**303.4a** — "An Aura spell requires a target, which is defined by its enchant ability."

**303.4d** — "An Aura can't enchant itself. If this occurs somehow, the Aura is put into its
owner's graveyard. An Aura that's also a creature can't enchant anything… (These are state-based
actions.)"

**303.4g** — "If an Aura is entering the battlefield and there is no legal object or player for
it to enchant, the Aura remains in its current zone, unless that zone is the stack. In that case,
the Aura is put into its owner's graveyard instead of entering the battlefield."

**702.5a** — "Enchant is a static ability, written 'Enchant [object or player].' The enchant
ability restricts what an Aura spell can target and what an Aura can enchant."

**702.5c** — "**If an Aura has multiple instances of enchant, all of them apply.** The Aura's
target must follow the restrictions from all the instances of enchant."

**702.5d** — "Auras that can enchant a player can target and be attached to players. Such Auras
can't target permanents and can't be attached to permanents."

**702.103b** — "As a spell cast bestowed is put onto the stack, it becomes an Aura enchantment
and gains enchant creature… **Because the spell is an Aura spell, its controller must choose a
legal target for that spell as defined by its enchant creature ability and rule 601.2c.**"

**702.151a** — "Reconfigure represents two activated abilities. Reconfigure [cost] means
'[Cost]: Attach this permanent to **another target creature you control**. Activate only as a
sorcery' and '[Cost]: Unattach this permanent. Activate only if this permanent is attached to a
creature and only as a sorcery.'"

**601.2c** — "The player announces their choice of an appropriate object or player for each
target the spell requires… If the spell has a variable number of targets, the player announces
how many targets they will choose before they announce those targets."

**115.6** — "A spell or ability that requires targets may allow zero targets to be chosen…"
(This is *not* the hexproof rule; the coordinator's brief cites 115.6 for hexproof/shroud/
protection. The governing rules there are **702.11b** (hexproof), **702.18a** (shroud) and
**702.16b** (protection) — see §4.3, where the question is settled by reading the code anyway.)

---

## 2. The seam, verified against source

### 2.1 The cast path (authoritative today)

`crates/engine/src/rules/casting.rs`:

* `:3615-3616` — `let (requirements, cant_be_countered) = card_def_target_requirements(state,
  card_id.as_ref(), casting_with_aftermath);` reads **`AbilityDefinition::Spell.targets` only**
  (`:5373-5411`). An Aura has no `AbilityDefinition::Spell` (§0.7), so this is `vec![]`.
* `:3619-3629` — overload override.
* `:3718` — `validate_targets_with_source(state, &targets, &requirements, …)`. With
  `requirements` empty, `validate_targets_inner` (`:5937`) **skips the count check entirely**
  and `req_for_target` is all-`None` (`:5960-5962`).
* `:3723-3774` — the CR 303.4a Aura gate. Derives the requirement from
  `super::sba::get_enchant_target(&chars.keywords)` (`sba.rs:999`), rejects `spell_targets
  .is_empty()`, then for each `Target::Object` checks battlefield + `sba::matches_enchant_target`
  (`sba.rs:1014-1036`).
* `:980-988` — **bestow already mutates `chars` in place**: removes `Creature`, inserts
  `Enchantment`, inserts `SubType("Aura")`, inserts `KeywordAbility::Enchant(EnchantTarget::
  Creature)` (CR 702.103b). This happens at Step 1b, ~2,600 lines *before* `:3615`, so the
  bestow case is already carried by `chars` at the synthesis point.

### 2.2 The offer path (blind today)

`crates/engine/src/rules/queries.rs:61-110` `spell_target_requirements` → the same
`casting::card_def_target_requirements` → `vec![]`. Every downstream consumer therefore sees
`(min, max) = (0, 0)`:

| consumer | line | what it does with the empty list |
|---|---|---|
| `tools/play-server/src/view.rs::action_target_requirements` | `:1457-1461` | feeds `action_option_view` |
| `tools/play-server/src/view.rs::action_option_view` | `:2362-2394` | `target_count_range` → `(0,0)`; `slots()` returns `Vec::new()` on `reqs.is_empty()` — **no picker rendered** |
| `crates/simulator/src/targeting.rs::plan_targets` | `:148` | `TargetPlan::NotTargeted` → bot announces nothing |
| `crates/simulator/tests/local_game_playthrough.rs` | `:240-243` | `min == 0` ⇒ the policy *selects* the Aura, then eats the refusal |
| `tools/play-server/src/main.rs` (test module) | `:1743-1748` | `KNOWN_FALSE_OFFERS` excuses the 422 |

`crates/simulator/src/legal_actions.rs` contains **zero** occurrences of `Enchant(`,
`get_enchant_target` or `target_min` — confirmed by `rg -c` — so the provider is not the place
to fix this. **`tools/play-server` needs zero production-source changes**: it already reads
everything through `spell_target_requirements`.

### 2.3 Decision — where the single insertion point goes, and why NOT `card_def_target_requirements`

`card_def_target_requirements(state, card_id, casting_with_aftermath)` takes a `CardId` and reads
the **registry**. The Enchant keyword must come from **layer-resolved characteristics**: bestow
grants it at cast time from caster intent (`casting.rs:987`), and a Layer-6 `AddKeyword` grant or
a Humility-class removal must be honoured (CR 613.1f). A registry read cannot see either.

⇒ **The insertion point is a new pair of `pub(crate)` helpers in `casting.rs` that take
`&Characteristics`**, called by `handle_cast_spell` and by `queries::spell_target_requirements`.
Both callers already compute `calculate_characteristics` for their own reasons
(`casting.rs`'s `chars`, `queries.rs:70`), so no new layer walk is introduced.

⇒ **`handle_cast_spell` consumes the synthesized list**, so `target_count_range` /
`validate_targets_inner` operate on the *same* `Vec<TargetRequirement>` the browser was shown.
That is the SR-38 / SIM-1 lesson taken literally (`effective_cast_cost` consuming
`apply_commander_tax` rather than re-deriving it), not "the two agree today".

---

## 3. The primitive: a TOTAL `EnchantTarget -> TargetRequirement` mapping

### 3.1 New code (all in `crates/engine/src/rules/casting.rs`, beside `card_def_target_requirements`)

```rust
/// CR 702.5a / 303.4a — the `TargetRequirement` an Enchant restriction is equivalent to.
/// Exhaustive on purpose: a new `EnchantTarget` variant must be a compile error here.
pub(crate) fn enchant_target_to_requirement(et: &EnchantTarget) -> TargetRequirement { … }

/// CR 303.4a — `base`, plus the Aura's keyword-carried requirement when it has one.
/// Returns `base` unchanged for every non-Aura spell.
pub(crate) fn aura_spell_target_requirements(
    chars: &Characteristics,
    base: Vec<TargetRequirement>,
) -> Vec<TargetRequirement> { … }
```

`aura_spell_target_requirements` synthesizes only when **all four** hold — the same conjunction
`casting.rs:3723-3726` already uses, plus an emptiness guard:

1. `chars.subtypes.contains(&SubType("Aura".to_string()))`
2. `chars.card_types.contains(&CardType::Enchantment)`
3. `base.is_empty()` — an Aura that also declared `AbilityDefinition::Spell.targets` keeps its
   own list (measured: **0 such defs**, §0.7; the guard exists so a future one is not silently
   given two requirements, and so the modal-spell guard at `casting.rs:3696` cannot newly fire)
4. `super::sba::get_enchant_target(&chars.keywords)` is `Some`

**CR 702.5c is NOT implemented, deliberately and identically on both sides.**
`get_enchant_target` returns the **first** `Enchant` keyword via `find_map` (`sba.rs:1000-1006`);
the synthesis inherits exactly that. This is behaviour-preserving — the cast path and the SBA
already share this deviation — and widening it would be a scope change and a behaviour change.
File as **`OOS-DX20-1`**, and prove it currently costs nothing with the T4 roster gate
(no def carries two `Enchant` keywords).

### 3.2 The mapping table — all 9 variants, with the equivalence argument per row

Read across: `sba::matches_enchant_target` (`sba.rs:1014-1036`) is the incumbent predicate;
`validate_object_satisfies_requirement` (`casting.rs:6418-6524`) + `effects::matches_filter`
(`effects/mod.rs:9721-9848`) is the synthesized one. Both run against
`expect_characteristics(state, id)` — layer-resolved on both sides.

| `EnchantTarget` | synthesized `TargetRequirement` | incumbent predicate | synthesized predicate | equivalent? |
|---|---|---|---|---|
| `Creature` | `TargetCreature` | `card_types.contains(Creature)` (`sba.rs:1021`) | `on_battlefield && is_creature` (`:6419`) | **YES** — the `on_battlefield` conjunct is what the gate checks separately at `casting.rs:3738-3747` |
| `Permanent` | `TargetPermanent` | `true` (`sba.rs:1026`) | `on_battlefield` (`:6420`) | **YES**, same reason |
| `Artifact` | `TargetArtifact` | `contains(Artifact)` | `on_battlefield && is_artifact` (`:6424`) | **YES** |
| `Enchantment` | `TargetEnchantment` | `contains(Enchantment)` | `on_battlefield && is_enchantment` (`:6425`) | **YES** |
| `Land` | `TargetLand` | `contains(Land)` | `on_battlefield && is_land` (`:6426`) | **YES** |
| `Planeswalker` | `TargetPlaneswalker` | `contains(Planeswalker)` | `on_battlefield && is_planeswalker` (`:6427`) | **YES** |
| `Player` | `TargetPlayer` | `false` for **objects** (`sba.rs:1027`) | object side: `validate_object_satisfies_requirement` has no `TargetPlayer` arm ⇒ falls to the catch-all ⇒ reject; player side: `validate_player_satisfies_requirement` accepts (`:6271`) | **YES on objects** (CR 702.5d "can't target permanents"). **See §4.4 — the player side is a real hazard.** |
| `CreatureOrPlaneswalker` | `TargetPermanentWithFilter(TargetFilter { has_card_types: vec![Creature, Planeswalker], ..Default::default() })` | `contains(Creature) \|\| contains(Planeswalker)` (`sba.rs:1028-1031`) | `on_battlefield` + `matches_filter` `has_card_types` **OR** (`effects/mod.rs:9814-9822` — verified: `.any(\|ct\| chars.card_types.contains(ct))`) + `controller: Any` (`:6486-6489` ⇒ `true`) + `exclude_self:false` + `is_attacking/is_blocking/is_tapped/is_untapped` all `false` ⇒ all no-ops (`:6497-6516`) | **YES** |
| `Filtered(f)` | `TargetPermanentWithFilter(TargetFilter { has_card_type: f.has_card_type, has_subtype: f.has_subtype, has_subtypes: f.has_subtypes, basic: f.basic, nonbasic: f.nonbasic, controller: map(f.controller), ..Default::default() })` | `sba::enchant_filter_matches` (`sba.rs:1047-1092`) | `matches_filter` + `passes_controller` | **YES, field by field** — see §3.3 |

**Do NOT use `TargetAny` for `CreatureOrPlaneswalker`**: `TargetAny` also accepts **players**
(`:6273`), which CR 702.5d forbids for a non-player Aura.

**No `TargetRequirement` variant is added.** (Adding one would move `HASH_SCHEMA_VERSION` —
`hash.rs` hashes `TargetRequirement` discriminants — and the brief forbids it.)

### 3.3 `EnchantFilter -> TargetFilter`, field by field

| `EnchantFilter` field | incumbent (`sba.rs`) | `TargetFilter` field | synthesized (`effects/mod.rs`) | equivalent? |
|---|---|---|---|---|
| `has_card_type: Option<CardType>` | `:1054-1058` `chars.card_types.contains` | `has_card_type` | `:9732-9736` same | **YES** |
| `has_subtype: Option<SubType>` | `:1060-1064` `chars.subtypes.contains` | `has_subtype` | `:9773-9777` same | **YES** |
| `has_subtypes: Vec<SubType>` (OR) | `:1066-1068` `!empty && !any(contains)` ⇒ reject | `has_subtypes` | `:9779-9786` identical expression | **YES** |
| `basic: bool` | `:1070-1072` `chars.supertypes.contains(Basic)` | `basic` | `:9758-9764` same (**verified `matches_filter` DOES check supertypes**) | **YES** |
| `nonbasic: bool` | `:1074-1076` `!contains(Basic)` | `nonbasic` | `:9766-9772` same, CR 205.4a | **YES** |
| `controller: EnchantControllerConstraint` | `:1078-1090`, `target_controller` vs `aura_controller` | `controller: TargetController` | `:6486-6493`, `obj.controller` vs `caster` | **YES** — `casting.rs:3755-3764` passes `player` (the caster) as `aura_controller`, and `caster` in `validate_object_satisfies_requirement` is the same `player`. Map `Any→Any`, `You→You`, `Opponent→Opponent`. **Never** map anything to `TargetController::DamagedPlayer` — its arm is a hard `false` for spells (`:6493`). |

`EnchantFilter` has exactly these six fields (`crates/card-types/src/state/types.rs:336-357`) —
confirmed by reading the struct. Every one maps 1:1. **The mapping is EXACT; there is no residue
to file for `Filtered`.**

### 3.4 Fields that must be left at `Default::default()` and why

`TargetFilter` has 30 fields (`card_definition.rs:3036-3239`). Everything not in §3.3 must stay
default, and each default is a proven no-op in the synthesized predicate:
`max_power`/`min_power`/`max_toughness`/`max_cmc`/`min_cmc`/`has_name`/`colors`/
`exclude_colors`/`has_counter_type` are `None` (early-`if let` skipped);
`has_keywords`/`has_card_types`(for non-CoP)/`has_subtypes`(for non-`Filtered`)/`exclude_subtypes`
are empty (loops/`!is_empty()` guards skipped); `non_creature`/`non_land`/`legendary`/`is_token`/
`is_nontoken`/`is_attacking`/`is_blocking`/`is_tapped`/`is_untapped`/`has_chosen_subtype`/
`exclude_chosen_subtype`/`exclude_self` are `false`. Use `..Default::default()` so a *new*
`TargetFilter` field cannot silently acquire a non-default meaning here.

---

## 4. The four hazards the brief asks to be adjudicated explicitly

### 4.1 Direction of divergence — stricter vs looser

Per §3.2/§3.3 the mapping is **exact** for all 9 variants at cast time, so neither direction
occurs *for the enchant restriction itself*. Three **structural** differences remain, and each is
argued and probed rather than assumed:

* **The count check becomes live.** With `requirements` non-empty, `validate_targets_inner:5937`
  now runs `target_count_range` ⇒ `(1,1)`. Today an Aura cast with **two** object targets passes
  `validate_targets_inner` (count skipped) and the gate loop simply checks both. Post-fix it is
  rejected. **This is a tightening in the CR-correct direction** (CR 303.4a: "*a* target").
* **The slot-assignment pass becomes live.** A declared target that satisfies nothing now fails
  at `:6021-6027` with `InvalidTarget("declared 1 target(s) but 1 could not be matched…")`
  instead of at the gate with `InvalidTarget("target does not match Enchant restriction …")`.
  Same variant, different message. **Consequence: the play-server allowlist string dies with
  it — see §5 step 7.**
* **One error VARIANT changes.** A zero-target Aura cast today returns
  `InvalidCommand("Aura spells require exactly one target (CR 303.4a)")` (`casting.rs:3729-3731`);
  post-fix the count check fires first and returns
  `InvalidTarget("expected 1..=1 target(s) but got 0")` (CR 601.2c). This reddens exactly one
  existing test — `crates/engine/tests/mechanics_e_l/enchant.rs:500`
  `test_702_5_enchant_casting_rejected_without_target`, which asserts
  `GameStateError::InvalidCommand(_)` at `:542-545`. **Update it to `InvalidTarget` and cite
  CR 601.2c in the doc comment.** Its reddening is *positive evidence* the synthesis took effect;
  say so in the test's comment. `GameStateError` is not in the `PROTOCOL_SCHEMA_FINGERPRINT`
  closure (`Command`/`GameEvent`/`Effect`/`Characteristics`) and no variant is added, so this
  moves no wire number.

### 4.2 The CR 303.4a gate at `casting.rs:3723-3774` — KEEP it

Do **not** delete the gate. Two of its three checks stay load-bearing:

* the battlefield check (`:3738-3747`) is redundant with the requirement (every mapped variant
  carries `on_battlefield`) but produces the specific CR 303.4a message;
* the `matches_enchant_target` check (`:3760-3769`) is the **SBA's own predicate** (CR 704.5m,
  `sba.rs:1093+`). Keeping it at cast time is what guarantees cast-time and SBA-time agree —
  which is a *different* property from "the offer and the cast agree", and this batch must not
  trade one for the other.

**Rewrite the gate's comment** (`:3720-3722`) to say it is now a deliberately redundant second
check whose purpose is SBA parity, and that the *announceable* requirement is synthesized
upstream. Leaving the old "it is derived from the card's keywords rather than from an explicit
TargetRequirement" wording standing would be exactly the aspirationally-wrong-comment class
(conventions.md). The `spell_targets.is_empty()` arm at `:3728-3732` becomes **unreachable** for
any Aura whose synthesis fired; keep it (it still covers an Aura whose `base` was non-empty and
whose declared targets were all players) and say so in the comment.

### 4.3 Do hexproof / shroud / protection newly apply? — **NO. Settled by reading the code.**

`validate_mapped_targets` (`casting.rs:6111-6234`) applies the protection/hexproof/shroud checks
**unconditionally** for any `Target::Object` in `Battlefield | Stack`
(`:6207-6219`, `super::validate_target_protection`) and the player-protection/player-hexproof
checks for any `Target::Player` (`:6134-6178`). Neither is gated on `req`; `req` is consulted
only afterwards at `:6223-6225`. Since today's empty-requirements path still calls
`validate_mapped_targets` with `req_for_target` all-`None` (`:5960-5962, :6048`), **CR 702.11b /
702.18a / 702.16b already apply to Aura casts at HEAD.** The synthesis changes nothing here, and
there is no behaviour change to make deliberate.

**Write a probe anyway** (T2.4): a hexproof creature is rejected as an Aura target both before
and after. It is cheap, it pins a property this plan asserts, and if it were ever to go red the
assertion above was wrong.

### 4.4 `EnchantTarget::Player` — the one place where opening the offer could create new wrong state

CR 702.5d says such an Aura *can* target a player. Mapping `Player → TargetPlayer` is:

* **engine-behaviour-preserving on the object side**: today `matches_enchant_target(Player, …)`
  is a hard `false` (`sba.rs:1027`), post-fix `validate_object_satisfies_requirement`'s catch-all
  rejects. Both reject.
* **engine-behaviour-preserving on the player side too**: today `requirements` is empty so a
  hand-built `CastSpell` naming `Target::Player(p)` already passes `validate_targets_inner`, and
  the gate loop at `:3733` iterates `Target::Object` only, so it is skipped. Post-fix
  `validate_player_satisfies_requirement` accepts `TargetPlayer` (`:6271`). Both accept.
* **but it OPENS THE OFFER**: post-fix `legal_targets_per_slot` enumerates players (`:190-196`)
  and the browser renders a player picker. **Aura-to-player attachment is not implemented** —
  `GameObject.attached_to` is `Option<ObjectId>` and `curse_of_opulence.rs:25-27` records that
  "sba.rs:995 rejects it (Auras cannot attach to players yet)".

**Decision**: map `Player → TargetPlayer` (total, CR 702.5d-correct, engine-behaviour-preserving)
**and gate the reachability**. T4 asserts by `all_cards()` enumeration that the set of defs
carrying `KeywordAbility::Enchant(EnchantTarget::Player)` is **EMPTY**, with a failure message
instructing the future author to implement player attachment *first*. File **`OOS-DX20-2`**:
"player-enchanting Auras have a target requirement and no attachment path; the offer is only
unreachable because the corpus is empty."

The alternative — synthesize nothing for `Player` — was considered and **rejected**: it would
make the mapping non-total in a way that is invisible at the type level, and it would leave the
offer at `(0,0)` for a card whose cast the engine *accepts*, which is the mirror image of the
defect this batch closes.

### 4.5 Bestow (`boon_satyr`, `Complete`) — include it, with the reason

`handle_cast_spell` gets bestow for free (`chars` is already transformed at `:980-988` long
before the synthesis point). `queries::spell_target_requirements` does **not**: `view.rs:1460`,
`main.rs:1871` and `targeting.rs:148` all pass `alt_cost: None`, and even with
`Some(AltCostKind::Bestow)` the function reads untransformed `chars`. That is a real drift
between the "same function" the batch promises.

**Decision**: mirror the existing Overload (`queries.rs:82-86`) and Aftermath (`:89-91`)
precedent exactly — in `spell_target_requirements`, when
`alt_cost == Some(AltCostKind::Bestow) && casting::get_bestow_cost(&obj.card_id,
&state.card_registry).is_some()`, apply the CR 702.103b keyword transform to a **local clone** of
`chars` before calling `aura_spell_target_requirements`. Requires widening
`casting::get_bestow_cost` (`:5148`) from `fn` to `pub(crate) fn` — a visibility change only,
no new public surface, no wire impact.

**State plainly what this does and does not deliver**: bestow is *not* reachable from the browser
today, because `StubProvider` enumerates no alt-cost casts (CLAUDE.md M11-local R4) and every
`spell_target_requirements` caller passes `alt_cost: None`. The value is that the two derivations
cannot drift the day that changes, and it is directly probeable
(`spell_target_requirements(&state, id, &[], Some(AltCostKind::Bestow))` → 1 requirement). If the
runner judges this out of scope, it must be **deferred with a filed seed**, not dropped silently.

---

## 5. Engine changes — numbered, independently checkable steps

Every step names its file, its CR citation, and its own gate. Run
`cargo check --workspace --all-targets` after each.

### Step 1 — the mapping helper
**File**: `crates/engine/src/rules/casting.rs`, immediately after `card_def_target_requirements`
(`:5411`).
**Action**: add `enchant_target_to_requirement` and `aura_spell_target_requirements` per §3.1/§3.2.
**CR**: 303.4a, 702.5a, 702.5d, 205.4a, 601.2c.
**Rules**: exhaustive `match` on `EnchantTarget`, **no wildcard arm** (a future variant must be a
compile error). Doc-comment every row of the §3.2 table at the function, including the CR 702.5c
first-instance-only deviation and the `Player` hazard, so the argument lives next to the code.
**Imports**: `EnchantTarget` is already in scope (`casting.rs:987`); `TargetController` is
(`:6439`); add `EnchantControllerConstraint`, `EnchantFilter`, `TargetFilter` if absent.
**Gate**: `cargo check -p mtg-engine`.

### Step 2 — the cast path consumes it
**File**: `crates/engine/src/rules/casting.rs:3619-3629`.
**Action**: fold the synthesis into the **else** branch of the overload override, so the shape is
`let requirements = if casting_with_overload { …; vec![] } else {
aura_spell_target_requirements(&chars, requirements) };`.
**CR**: 702.96b (overload has no targets) + 303.4a.
**Why the else branch specifically**: `queries::spell_target_requirements` returns `vec![]` for
overload at `:84-86` *before* any synthesis. Putting the synthesis after the override in
`handle_cast_spell` would give an overloaded Aura a requirement the query does not report —
re-creating the exact drift this batch closes, in a case no card reaches. Mirror the query.
**Gate**: `cargo test -p mtg-engine --test mechanics_e_l enchant` (expect the one variant change
of §4.1 — fix it in step 8, not here).

### Step 3 — the gate's comment
**File**: `crates/engine/src/rules/casting.rs:3720-3722` (and the `spell_targets.is_empty()` arm
`:3727-3732`).
**Action**: rewrite per §4.2. No behaviour change.

### Step 4 — the query path
**File**: `crates/engine/src/rules/queries.rs:61-110`.
**Action**: (a) widen `casting::get_bestow_cost` to `pub(crate)`; (b) in
`spell_target_requirements`, after the overload early-return, build the effective `chars` (clone +
CR 702.103b transform when bestowed, per §4.5); (c) wrap **both** remaining return points — the
aftermath early-return at `:99` and the final `match` at `:104-109` — in
`casting::aura_spell_target_requirements(&eff_chars, …)`.
**CR**: 303.4a, 702.103b, 702.127a, 700.2c.
**Action (doc)**: extend the function's "two deliberate divergences" doc block to a third entry
describing the bestow transform and *why* it is a query-side re-derivation of caster intent
(the same reason divergence 1 and 2 exist).
**Gate**: `cargo test -p mtg-engine --test rules queries`.

### Step 5 — the Reconfigure synth site (`OOS-CARDS1-2`)
**File**: `crates/engine/src/testing/replay_harness.rs:4000-4015` (the
`AbilityDefinition::Reconfigure { cost }` **attach** arm).
**Action**: replace `targets: vec![]` (`:4001`) with

```rust
targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
    controller: TargetController::You,
    exclude_self: true,
    ..Default::default()
})],
```

**CR**: 702.151a — "Attach this permanent to **another target creature you control**".
**Do NOT copy CARDS-1's equip repair** (`skullclamp.rs:77-80`): CR 702.6a is "target creature you
control" with **no** "another", so that repair correctly omits `exclude_self`. Reconfigure's does
not. This is the single most likely place for the runner to be wrong.
**Leave the detach ability (`:4018-4032`) at `targets: vec![]`** — `Effect::DetachEquipment` has
no `DeclaredTarget` and CR 702.151a's second ability takes no target. The golden script
`test-data/generated-scripts/baseline/script_189_reconfigure.json:218-219` activates it with
`"targets": []` and must stay green.
**Enforcement is already single-source**: `handle_activate_ability` reads
`ab.targets` from `expect_characteristics(state, source).activated_abilities[ability_index]`
(`abilities.rs:315-334`) — the same list `queries::ability_target_requirements` returns
(`queries.rs:135-139`). One edit fixes offer **and** validation.
**Note the legacy guard, do not delete it**: `abilities.rs:539-582` special-cases
`Effect::AttachEquipment` and checks battlefield + controller via `targets.first()`. It is
`if let Some(...)`, so with `targets` **empty** it silently passes and the effect then fizzles
with the cost paid — that is the live defect. Post-fix `validate_targets_with_source`
(`abilities.rs:495-508`) runs first and rejects. The guard becomes redundant but still covers
card-def-authored equip abilities; **add a comment saying so** rather than removing it (removing
it is a separate decision with a separate blast radius).
**Gate**: `cargo test -p mtg-engine --test mechanics_m_z reconfigure` and
`SCRIPT_FILTER=189 cargo test -p mtg-engine --test scripts -- --nocapture`.

### Step 6 — the Reconfigure hand-built fixture stops pinning the defect
**File**: `crates/engine/tests/mechanics_m_z/reconfigure.rs:54-90`
(`reconfigure_attach_ability`).
**Action**: give it the same `targets` as step 5, so the fixture mirrors the production synth.
**Verified safe**: `test_reconfigure_cant_attach_to_self` (`:434`, `:486-499`) and
`test_reconfigure_cant_attach_to_opponents_creature` (`:667`, `:719-732`) both accept
`Err(_) | Ok(no attachment)`, so they stay green under either outcome — **which also means
neither is discriminating**, and that is why T5 in §6 exists.
Leave `reconfigure_detach_ability` (`:93-128`) alone.

### Step 7 — delete the excusal mechanism (acceptance criterion 4)
**File**: `tools/play-server/src/main.rs` (test module).
**Actions**:
1. Delete `const KNOWN_FALSE_OFFERS` (`:1743-1748`) — its register is now empty.
2. Replace the `assert!(KNOWN_FALSE_OFFERS.iter().any(…), …)` at `:1750-1757` with an
   **unconditional** failure naming the refused label and reason. *This is the staleness
   assertion the list never had* (`:1736-1737` says so in as many words), and it is what proves
   the deletion: if any provider/engine disagreement remains, this driver goes red.
3. Rewrite the comment block at `:1666-1683` — it documents `OOS-CARDS2-4` as **live**. Record
   instead that the Aura case is closed by PB-DX20, that the fallthrough loop survives as a
   *policy-order* device (PlayLand → develop-castable → PassPriority) rather than as an excusal,
   and that the develop policy's `target_min == 0` filter (`:1691`) now *correctly* excludes
   Auras at the source.
**Precedent to follow**: `crates/simulator/tests/local_game_playthrough.rs:495-506` — "an excusal
list is a debt register with a maturity date… the whole excusal mechanism is deleted along with
it". Keep the loop, delete the excuse.
**Gate**: `cargo test -p play-server` (expect **78 / 0** unmoved unless a seeded driver diverges
— see §7 R3).

### Step 8 — the one error-variant test
**File**: `crates/engine/tests/mechanics_e_l/enchant.rs:500-546`.
**Action**: `test_702_5_enchant_casting_rejected_without_target` — change the expected variant
from `InvalidCommand` to `InvalidTarget`, cite **CR 601.2c**, and record in the doc comment that
the change is the count check at `casting.rs:5937-5946` becoming live, i.e. the batch's own
evidence. Verified by reading: the other seven error-asserting tests in this file
(`:115`, `:553`, `:833`, `:945`, `:1059`, `:1380`, `:1440`) all expect `InvalidTarget` and all
still get `InvalidTarget` (from slot assignment instead of from the gate) — **do not touch them**.

### Step 9 — every prose site that documents the defect as live
Aspirationally-wrong comments are correctness hazards (conventions.md). All five must land in
the same commit as the fix:

| file | line | what it claims | action |
|---|---|---|---|
| `crates/simulator/src/targeting.rs` | `:37-43` | "**Auras are still unannounceable** (`OOS-CARDS2-4`)… `spell_target_requirements` returns an empty list for one" | rewrite: closed by PB-DX20; the predicate is now reached *through* `queries.rs`, still with zero re-derivation here |
| `crates/simulator/src/targeting.rs` | `:81` | "It does not cover `OOS-CARDS2-4`" (on `TargetPlan::Unsatisfiable`) | rewrite; note that `Unsatisfiable` is now **reachable for Auras** (an Aura with no legal target on the board) — a new, correct behaviour |
| `crates/simulator/src/setup.rs` | `:234-235` | seed 1 "landed on a deck that exposes a pre-existing engine defect ('Aura spells require exactly one target')" | rewrite: the defect is closed; the historical reason the two-pass `resolve_decks` was reverted is unchanged and must be preserved |
| `crates/simulator/src/report.rs` | `:88-89` | "`OOS-CARDS2-4` (Aura offers refused by CR 303.4a) … open. Ratchet DOWNWARD as each closes" | strike `OOS-CARDS2-4` from the open list; **do not** change the constant without §7 R2's measurement |
| `crates/engine/tests/primitives/cards1_equip_target_repair.rs` | `:638-648` | "Darksteel Garrison (Fortify) and **Lizard Blades (Reconfigure)** BOTH carry the exact same `targets: vec![]` defect shape… this batch does NOT fix either" | rewrite the Reconfigure half only. **The Fortify half stays unfixed — do not widen.** Say which batch closed the Reconfigure half and that Fortify remains open |

---

## 6. Probes — file placement, CR citation, and the revert that must be watched fail

**SR-9a**: integration tests are nine grouped targets. **Never** add a top-level
`crates/engine/tests/*.rs`.

**New file**: `crates/engine/tests/primitives/pb_dx20_keyword_carried_target_requirements.rs`.
**Required `mod` line**: `crates/engine/tests/primitives/main.rs`, inserted **between**
`mod pb_dx1_lowered_intervening_if;` (`:33`) and `mod pb_dx2_command_gates;` (`:34`) — that is
the file's byte-order position (`pb_dx19` < `pb_dx1_` < `pb_dx20` < `pb_dx2_`). A dropped `mod`
line silently deletes coverage and the SR-9a gate catches it; add it in the same edit.

### T1 — **the differential-equivalence probe** (acceptance criterion 1, headline)
CR 303.4a / 702.5a / 702.5d. For **each of the 9 `EnchantTarget` variants** (built synthetically
with `ObjectSpec` + `with_types([Enchantment])` + `with_subtypes(["Aura"])` +
`with_keyword(KeywordAbility::Enchant(v))`, the `aura_in_hand` idiom from
`mechanics_e_l/enchant.rs:557`), against a fixed candidate board (own creature, opponent creature,
own basic Mountain, own nonbasic land, opponent Mountain, artifact, enchantment, planeswalker,
a creature in a graveyard, both players):

> `legal_targets_per_slot(&state, caster, aura_id, &spell_target_requirements(…)).contains(cand)`
> **⟺** `process_command(CastSpell{ targets: vec![cand] }).is_ok()`

for every (variant × candidate). This decides §3.2's equivalence claim **by execution, not by
argument**, and it is the only assertion that can catch a stricter-or-looser mapping in either
direction. Assert non-vacuity: at least one accepted and one rejected candidate per variant.
**Revert to watch fail**: change the `Filtered` arm's `controller` mapping to
`TargetController::Any`. Expect the `Enchant Mountain you control` row to start accepting the
opponent's Mountain in `legal_targets_per_slot` while `process_command` still rejects it — the
two sides diverging is exactly what T1 exists to see.

### T2 — the cast path and the offer path are the same function
CR 303.4a / 601.2c. Four sub-cases:
* **T2.1** For an `Enchant creature` Aura, `spell_target_requirements` returns exactly one
  requirement and `target_count_range` is `(1, 1)` — the value `view.rs:2363` reads.
* **T2.2** A zero-target cast is rejected with `InvalidTarget` (CR 601.2c), and a two-target cast
  is rejected (CR 303.4a "a target") — both newly, both stated in §4.1.
* **T2.3** A legal single-target cast still succeeds end-to-end and the Aura attaches on
  resolution (regression floor; mirrors `enchant.rs:223`).
* **T2.4** §4.3's assertion: a **hexproof** opponent creature is refused as an Aura target
  (CR 702.11b) — proving the generic per-target checks applied before this batch and still do.
**Revert to watch fail (T2.1/T2.2)**: make `aura_spell_target_requirements` return `base`
unconditionally. Both must redden, and T2.1's message must print the observed `(0,0)`.

### T3 — Bestow (§4.5)
CR 702.103b. `spell_target_requirements(&state, boon_satyr_obj, &[], Some(AltCostKind::Bestow))`
returns one `TargetCreature`, while the same call with `None` returns `vec![]` (a bestow card is
a creature spell until the caster says otherwise). Skip only if step 4(b) is deferred — and if
deferred, the seed must be filed.
**Revert**: drop the bestow branch; the `Some(Bestow)` half returns `vec![]`.

### T4 — the second failure mode, and the roster gates (acceptance criterion 3)
CR 303.4a / 702.5c / 702.5d. **Enumerate `mtg_engine::all_cards()`** (SR-36 — never grep source)
and pin four **exact** sets, each with a failure message that tells the future reader what to do:

| assertion | measured value | why it is the shape that rots silently |
|---|---|---|
| defs with the `"Aura"` subtype **and no** `KeywordAbility::Enchant` | **exactly** `{"Animate Dead", "Curse of Opulence"}`, **both `inert`** — assert the completeness too | `casting.rs`'s whole Aura gate is skipped for these; nothing is live today and nothing says so |
| defs with `Enchant(EnchantTarget::Player)` | **EMPTY** | §4.4 — the offer would open onto an unimplemented attachment path |
| defs carrying **two or more** `Enchant` keywords | **EMPTY** | §3.1 — `get_enchant_target`'s `find_map` silently drops the rest, contra CR 702.5c |
| Aura defs also carrying an `AbilityDefinition::Spell` | **EMPTY** | §3.1 guard 3 — such a def would keep its own list and get no Enchant requirement |
Plus a non-vacuity floor: the `Enchant`-carrying roster is **23** and its `Complete` subset is
**13** (§0.3/§0.5), each named in the assertion message, with the instruction that a move here
means the batch's yield claim changed and must be restated rather than re-tuned.
**Revert**: comment out one roster row / plant an `Enchant(Player)` on a test-local def and watch
each assertion name the exact offender.

### T5 — Reconfigure, through the REAL synth path (acceptance criterion 2)
CR 702.151a. Build `lizard_blades` from the corpus via `enrich_spec_from_def` (**not** the
hand-built helper — that is the whole point), then assert, strictly:
* **T5.1** `ability_target_requirements(&state, blades, 0)` == the exact `TargetCreatureWithFilter
  { controller: You, exclude_self: true }` requirement, and `(min,max) == (1,1)`;
* **T5.2** attach targeting **itself** ⇒ `Err(InvalidTarget)` — **assert `Err`, not
  `Err | Ok(no attachment)`**; the two existing tests are tolerant by construction (§ step 6)
  and therefore cannot see this;
* **T5.3** attach targeting an **opponent's** creature ⇒ `Err(InvalidTarget)`;
* **T5.4** attach targeting another creature you control ⇒ `Ok`, and after resolution
  `attached_to == Some(target)` and `is_reconfigured` is set (CR 702.151b);
* **T5.5** attach with **zero** targets ⇒ `Err` — the live defect: today it is `Ok` with the cost
  paid and a silent fizzle. Assert the mana was **not** spent (CR 602.2c: an illegal activation
  rewinds). This is the discriminating one.
* **T5.6** ability index 1 (unattach) still has `targets == vec![]` and still activates with none.
**Revert to watch fail**: set `exclude_self: false` at `replay_harness.rs`. T5.2 must redden and
T5.3/T5.4/T5.5 must stay green — that is what proves the exclusion specifically, and not merely
"some requirement is present".

### T6 — the browser path, end to end (acceptance criterion 1, second half)
**File**: `tools/play-server/src/main.rs` test module. Follow the `DeckSource::Fixed` fixture
precedent (`:3565`, `:4009`, `:9524`) and the SIM-6 single-purpose-fixture practice: a two-player
game whose human seat holds **Rancor** with one of its own creatures on the battlefield. Assert on
the real payload:
* the `CastSpell` action for Rancor carries `target_min == 1` (today: `0`) and a
  `target_slots[0].candidates` list containing that creature;
* `POST /api/game/action` with that target returns **200**, and the follow-up view shows the
  command count advanced (today: **422**).
If a full HTTP round trip proves disproportionate, the fallback is a `view::decision_view`
assertion on the same fixture — **but say which was done**; do not claim the browser path was
exercised if only the view layer was.
**Revert**: the same one as T2, watched through the HTTP status code.

### T7 — no other test regressed
Not a new test: `cargo test --workspace --no-fail-fast` to a **file** (never `| tail` — a tail
pipe once hid a compile failure and faked a green run), residual list empty.

---

## 7. What could go wrong — the tests most likely to go red, and why

**R1 (near-certain, planned).** `mechanics_e_l/enchant.rs:500`
`test_702_5_enchant_casting_rejected_without_target` — error variant change (§4.1, step 8). If it
does **not** redden, step 2 did not take effect; treat a green here as a failure signal.

**R2 (likely, and the biggest one). Bot behaviour changes, so every seeded simulator fixture can
diverge.** `targeting.rs::plan_targets` (`:148`) now returns `Announce(vec![target])` for Auras
instead of `NotTargeted`, so bots **cast Auras for the first time**. The choice is deterministic
(first legal candidate, no RNG), so no *deck* changes — but the **command stream** does, and every
downstream count is a function of it. Specifically at risk:
* `crates/simulator/tests/pb_dx32_fuzz_output.rs` — **T4.1** (pins seed 2 / 25 turns producing
  exactly 4 raw `no_orphaned_tokens` reports), **T4.3**, **T6.3** (asserts an **exact** partition
  of reached decision rows: `triggered_targets, search_library, scry, discard_cards`), **T2.2**
  (`MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG = 40` against a measured 31.081‰), **T3.1**
  (`MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG = 95` and the `total_taps >= 77` floor).
* `crates/simulator/tests/sim5_bot_cast_discipline.rs` — **T3.3**
  (`MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED = 1`).
* `crates/simulator/tests/local_game_playthrough.rs` — its cheapest-cast policy (`:240-243`)
  filters on `min > 0`, so Auras are now **excluded** from selection where they were previously
  selected-then-refused. Seed 1 is documented (`setup.rs:234`) to land on exactly such a deck.
**Protocol**: measure each of these at HEAD *before* editing and again after; if one moved,
**re-pin it with the measured value and the reason in the constant's own doc**, and report the
move as a finding. Do not re-tune a threshold to make a test pass without stating what moved and
why. Exposure estimate: 13 of 1,133 `Complete` defs (~1.1%) are Auras, ~1.15 per 100-card deck —
divergence is likely over 20 games × 200 turns, not certain over 3 seeds × 25 turns.

**R3 (possible).** `tools/play-server` — **78 / 0** today. The 6 seeded name-pinning fixtures read
the *deal*, which this batch does not touch, so those are safe. The `S7`-loop drivers
(`main.rs:1655-1765`) advance bot seats and can diverge for R2's reason; and step 7 makes any
residual refusal fatal. A red here is a **finding**, not something to re-allowlist.

**R4.** The golden script corpus — `stack/062_rancor_aura_attach_and_return.json` (casts Rancor
*with* a target) and `baseline/script_189_reconfigure.json` (targets Monastery Swiftspear at
index 0, empty targets at index 1) were both read and should stay green. If either reddens, the
mapping is wrong, not the script. **Do not start the replay-viewer HTTP server to validate** —
use `SCRIPT_FILTER=<name> cargo test --test scripts -- --nocapture` (agents get SIGKILL 137 on
the HTTP binary).

**R5.** `clippy -D warnings` — `..Default::default()` on a struct literal that already sets every
field trips `clippy::needless_update` (PB-DX32 hit this exact class). Not expected here
(`TargetFilter` has 30 fields and we set ≤6), but if a revert experiment ever makes the update
vacuous, the revert will fail to **compile** rather than fail the assertion — and a
non-rebuilding revert is a false green. **Confirm `Compiling mtg-engine` appears in the output of
every revert experiment before trusting its result.**

**R6.** `handle_cast_spell`'s modal guard at `:3696` (`"modal spell has both Spell.targets and
ModeSelection.mode_targets"`) now has a new way to fire — a modal Aura. Measured: 0 such defs
(§0.7). T4 gates it.

**R7.** `mechanics_m_z/reconfigure.rs` tests 4 and 7 are tolerant (`Err(_) | Ok(no attachment)`),
so step 6 cannot redden them — **and that is precisely why they are not acceptable evidence**.
T5.2/T5.3 must be the strict versions.

**R8.** `crates/engine/tests/rules/queries.rs` — `test_spell_target_requirements_missing_object_
is_empty` (`:221`) and the Overload probe (`:205-212`). Both should stay green: the missing-object
early-return (`queries.rs:67-69`) precedes the synthesis, and overload returns before it.

---

## 8. Seed dispositions and new seeds

**Closed by this batch**
* **`OOS-CARDS2-4`** (HIGH) — CLOSED. Evidence: T1, T2, T6 + the deletion of
  `KNOWN_FALSE_OFFERS` with the register's emptiness proven by an unconditional assertion.
* **`OOS-CARDS1-2`** — CLOSED. Evidence: T5, + the t7b prose update.

**Narrowed, not closed**
* **`OOS-SIM4-2`** — PARTIAL. The Aura-specific 422 is gone; the general class ("the provider
  offers a targeted `CastSpell` with no legal target on the board") is untouched and is
  `OOS-SIM5-4`'s subject. Restate the row with the Aura clause struck and the general clause kept.
* **`OOS-SIM5-4`** (offer suppression) — **stays deferred**, and PB-DX20 *increases* its value:
  `TargetPlan::Unsatisfiable` is now reachable for Auras, and `get_enchant_target` is no longer
  needed outside the engine to decide it (`spell_target_requirements` answers it). Record that
  the "needs an engine query; `get_enchant_target` is `pub(crate)`" blocker in its row is now
  **stale**.

**New seeds to file**
* **`OOS-DX20-1`** — CR 702.5c: multiple `Enchant` instances; `get_enchant_target`'s `find_map`
  keeps the first only, on both the cast path and the synthesis. Corpus exposure 0 (T4 gates it).
* **`OOS-DX20-2`** — player-enchanting Auras: a requirement now exists, an attachment path does
  not (`GameObject.attached_to: Option<ObjectId>`; `sba.rs` rejects). Unreachable only because the
  corpus is empty (T4 gates it). Blocks `curse_of_opulence`.
* **`OOS-DX20-3`** — CR 303.4g: an Aura with **no** legal target on the board is still *offered*
  by `StubProvider` and rejected at cast. Sub-case of `OOS-SIM5-4`; noted because PB-DX20 makes it
  the *only* remaining Aura refusal shape.
* **`OOS-DX20-4`** — `curse_of_opulence.rs:20` TODO is false (§0.10).
* **`OOS-DX20-5`** — `kayas_ghostform` declares `Enchant(Creature)` for a printed
  "Enchant creature or planeswalker **you control**": narrower in type, wider in controller. Now
  *visible* in the browser picker. `partial`, so not deck-legal.
* **`OOS-DX20-6`** — `polymorphists_jest.rs:6` TODO is false (`TargetRequirement::TargetPlayer`
  exists).
* **`OOS-DX20-7`** — `abilities.rs:539-582`'s legacy `AttachEquipment` guard is now redundant with
  declarative validation for every def that carries a requirement, but silently permissive for any
  that does not (`if let Some(...)` on an empty vec). A roster gate on
  "Activated + `AttachEquipment` ⇒ non-empty `targets`" would close it; out of scope here.
* **`OOS-DX20-8`** (only if step 4(b) is deferred) — bestow's query-side transform.

---

## 9. Verification checklist

- [ ] §0 baselines re-measured at HEAD **before any edit**, output captured to files
- [ ] §0.10 TODO sweep re-run and its 5 dispositions confirmed (positive assertion, 0 forced adds)
- [ ] `cargo check --workspace --all-targets` clean after every step
- [ ] `cargo build --workspace` clean
- [ ] `cargo test -p mtg-engine --test primitives pb_dx20` — all of T1–T5 green
- [ ] Every revert named in §6 **executed**, `Compiling` observed in its output, failure message
      recorded verbatim, `git diff` confirmed clean after each restore
- [ ] `cargo test -p mtg-engine --test mechanics_e_l enchant` — step 8's variant change applied,
      the other 7 error tests untouched and green
- [ ] `cargo test -p mtg-engine --test mechanics_m_z reconfigure` green
- [ ] `SCRIPT_FILTER=062 …` and `SCRIPT_FILTER=189 …` green (no HTTP server)
- [ ] `cargo test -p play-server` — report the count; 78/0 expected, any move investigated
- [ ] `cargo test -p mtg-simulator` — report the count; §7 R2 measured before/after
- [ ] `cargo test --workspace --no-fail-fast` **to a file**, 0 failures, residual list empty
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — the script is the only thing
      that checks the 1,803 defs)
- [ ] `cargo test -p mtg-engine --test core hash_schema` / `--test core protocol_schema`
      **executed**, not predicted — expect PROTOCOL **35** / HASH **72** unmoved
- [ ] `git diff main..HEAD --numstat -- crates/card-defs/` **EMPTY** (the coverage evidence)
- [ ] `tools/authoring-report.py` regenerated; body byte-identical apart from the sha/date stamp;
      coverage unmoved at **1,133 / 1,803 = 62.8%**; regeneration churn reverted before commit
- [ ] Every §5 step 9 prose site updated **in the same commit** as the code
- [ ] Seeds §8 filed; CLAUDE.md **new short delta line** (never grow an existing line — git merges
      at line granularity); `memory/workstream-state.md` handoff appended
