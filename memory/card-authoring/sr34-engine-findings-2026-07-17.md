# Engine findings from SR-34 (`scutemob-90`) — for the next SR

Filed, **not fixed** here, per the SR protocol and the task's explicit scope boundary
(§8 items 3, 4, 10 of `memory/primitives/pb-plan-sr34.md`). Ordered by severity.

---

## SF-8 — `Cost::Tap` + `AddManaScaled` produces exactly 1 mana, not the scaled amount

**Severity: HIGH. Pre-existing, unchanged by SR-34, but now load-bearing for a design
decision (Finding A) rather than an invisible accident.**

`try_as_tap_mana_ability`'s `AddManaScaled` arm (`replay_harness.rs`) registers
`produces = {colour: 1}` and calls it, in its own comment, *"a marker; actual production
is dynamic."* Nothing makes it dynamic: `handle_tap_for_mana` (`rules/mana.rs`) has no
`AddManaScaled` branch at all — it only ever reads `ability.produces` and
`ability.any_color`. The real per-Swamp/per-creature evaluation lives exclusively in
`effects/mod.rs`, reachable only through stack resolution (`ActivateAbility`).

Consequence: every bare-`Cost::Tap` `AddManaScaled` card — Gaea's Cradle, Elvish
Archdruid, Priest of Titania, Marwyn the Nurturer, Circle of Dreams Druid, Howlsquad
Heavy (not `Complete`; speed-gate omitted), and by the same shape Everflowing Chalice,
Elvish Guidance, Brightstone Ritual, Battle Hymn, Black Market (re-check each against the
registry — not re-verified in this task) — taps for **exactly 1** of its colour
regardless of board state when activated via `Command::TapForMana`. `Cabal Coffers` /
`Cabal Stronghold` / `Crypt of Agadeem` do NOT have this bug today, because their cost is
`Sequence[Mana, Tap]`, not bare `Tap`, so pre-SR-34 they were excluded from the mana-ability
lowering entirely and stayed on the stack, where the real `AddManaScaled` evaluation runs.

**SR-34 made this bug load-bearing, not just present.** The widened lowering gate
(`mana_ability_lowering` in `replay_harness.rs`) had to add an explicit, named exclusion
(Finding A) refusing `AddManaScaled` for any cost shape *other than* bare `Cost::Tap` —
specifically so Cabal Coffers is not captured and demoted from "correct via the stack" to
"exactly one black mana." That exclusion is a documented seam: **fixing SF-8 is exactly
what makes deleting the exclusion correct**, and at that point Cabal Coffers /
Cabal Stronghold / Crypt of Agadeem should also be widened into real mana abilities.

Two tests now know about this precisely and by name:
- `crates/engine/tests/casting/mana_filter.rs::test_add_mana_scaled_registered_as_mana_ability`
  and `::test_add_mana_scaled_orphan_fix_all_cards` — shape-only (never activate, never
  assert the amount), annotated with an explicit doc-comment note per SR-34 §9's "decide
  and write down why" instruction, rather than left as silent SF-5-pattern cover.
- `crates/engine/tests/primitives/primitive_sr34_composite_mana_costs.rs::composite_cost_add_mana_scaled_stays_on_the_stack`
  pins the Finding-A exclusion itself (Cabal Coffers registers 0 mana abilities, 1
  activated ability) and says explicitly to delete it when SF-8 lands.
- `crates/engine/tests/core/effect_choose_gate.rs::printed_tap_mana_colors` now documents
  the blind spot in its own doc comment (it reads colours, never amounts, so it cannot
  see this bug) rather than silently passing by luck, per SR-34 §9's instruction not to
  let the blind spot survive a second time undocumented.

**Fix shape**: `handle_tap_for_mana` needs a resolution context to evaluate an
`EffectAmount` (the same kind of context stack resolution already has) inside the
stackless `TapForMana` path. That is a distinct primitive from SR-34's cost-widening —
filed here, not attempted.

---

## SF-9 — `Cost::PayLife` is silently unpaid for *non-mana* activated abilities

**Severity: HIGH (upgraded from MEDIUM by `scutemob-90`). Pre-existing, unchanged by SR-34.
NOT hypothetical — live victims proven by activation, named below.**

### Live victims (probed 2026-07-17, criterion 4767)

This finding originally said the defect needed "its own roster" and illustrated it with a
*hypothetical* `"{T}, Pay 2 life: Destroy target creature"`. It has real victims, and one of
them was shipping `Complete`:

- **Staff of Compleation** — `{T}, Pay 3 life: Proliferate.` probed at **life 40 → 40**.
  `{T}, Pay 4 life: Draw a card.` probed at **life 40 → 40**. Both free. A `Complete` card was
  shipping a **free repeatable proliferate and a free repeatable draw**. (Its `{T}, Pay 1 life:
  Destroy target permanent you own` is the same defect; not separately probed because it needs
  a target.) Only its `{T}, Pay 2 life: Add one mana of any color` charges life — because SR-34
  lowered *that* one into a `ManaAbility` with `life_cost: 2` (probed: life 40 → 38). The card
  is now `known_wrong` for this plus its colour bug.
- **Voldaren Estate** — `{T}, Pay 1 life: Add one mana of any color. Spend this mana only to
  cast a Vampire spell.` probed at **life 40 → 40**. This one is a *mana* ability, so SR-34
  should have caught it — it does not, because `Effect::AddManaAnyColorRestricted` has no arm
  in `try_as_tap_mana_ability`, so the ability never reaches the `ManaAbility` path and falls
  back to `handle_activate_ability`, where `flatten_cost_into` drops the life cost. **This is
  the exact interaction the original note said could not happen** ("the two never overlap — an
  ability either lowers into a `ManaAbility` or it goes through `handle_activate_ability`"):
  the paths do not overlap, but a *mana* ability can fall into the non-mana path when the
  effect matcher misses it, and then it silently loses its cost.

The roster this finding said was not built is therefore partially built: any
`AbilityDefinition::Activated` whose cost contains `Cost::PayLife` and whose effect is **not**
matched by `try_as_tap_mana_ability` pays nothing. That set is larger than the two above — a
full corpus scan is still owed.

**Severity rationale for the upgrade**: a free proliferate/draw engine on a `Complete` card is
wrong game state reachable in a legal deck, not a missing clause.

---

### Original text

**Severity: MEDIUM. Pre-existing, unchanged by SR-34.**

`flatten_cost_into` (`replay_harness.rs`, feeds `cost_to_activation_cost`) maps
`Cost::PayLife(_) => {}` with the comment *"no ActivationCost representation yet."*
`ActivationCost` (`card-types/src/state/game_object.rs`) has no life field at all. So any
`AbilityDefinition::Activated` ability that is **not** a mana ability (has a target,
produces no mana, or is a loyalty-style effect) but carries a `Cost::PayLife` component
pays **nothing** when activated through `handle_activate_ability`
(`rules/abilities.rs`) — the life cost is silently dropped.

SR-34 fixes the *mana-ability* half of this exact defect class: `ManaAbility::life_cost`
+ the new step 5b/6b in `handle_tap_for_mana` pay it correctly for horizon lands, Mana
Confluence, Staff of Compleation, etc. — anything that lowers into a `ManaAbility`. This
finding is about the **other** path: a non-mana ability with `Cost::PayLife` (e.g. a
hypothetical "{T}, Pay 2 life: Destroy target creature") still silently charges nothing.

**Not actioned in SR-34**: cleanly separable from the mana-ability path (the two never
overlap — an ability either lowers into a `ManaAbility` or it goes through
`handle_activate_ability`, never both), and needs its own roster: a corpus scan for
`AbilityDefinition::Activated` entries whose `cost` contains `Cost::PayLife` and whose
effect/targets make them NOT eligible for `mana_ability_lowering`. Not attempted here —
no such roster was built. Fix shape: add a `life_cost: u32` (or similar) field to
`ActivationCost`, wire `flatten_cost_into` to populate it, and add a payment step to
`handle_activate_ability` mirroring the new step 5b/6b in `handle_tap_for_mana`.

---

## SF-10 — `ManaAbility` has no `activation_condition`; a conditioned mana ability ignores its own condition

**Severity: MEDIUM. Pre-existing (Tainted Field was already lowered before SR-34, being
bare `Cost::Tap`); SR-34 does not widen its blast radius, but does not fix it either.**

`tainted_field.rs` authors its two coloured arms with
`activation_condition: Some(Condition::ControlLandWithSubtypes([Swamp]))` (CR 605.1:
an activation restriction does not disqualify a mana ability). The lowering loop in
`enrich_spec_from_def` destructures
`AbilityDefinition::Activated { cost, effect, targets, .. }` — the `..` **silently drops
`activation_condition`** — and `handle_tap_for_mana` never checks one. So Tainted Field
taps for `{W}` or `{B}` with **no Swamp controlled**, contrary to its printed text.

**Why this is not "just fix it while you're in the file"**: the tempting fix — refuse to
lower an `Activated` ability that carries a non-`None` `activation_condition`, forcing it
back onto the stack where `handle_activate_ability` *does* check
`activation_condition` — regresses SR-33's `every_complete_land_registers_each_printed_tap_mana_color`
gate (Tainted Field would go back to registering zero mana abilities for its coloured
arms) and reintroduces the exact CR 605.1a/605.3b violation SR-33 fixed. The correct fix
carries the condition through into `ManaAbility` (a new field, mirroring `mana_cost` /
`life_cost`) and checks it in `handle_tap_for_mana`'s step 5b alongside the cost-legality
check — a small, well-scoped follow-up, but out of SR-34's declared surface (the plan's
§3 covers only `mana_cost` / `life_cost`).

**Roster**: Tainted Field is the only known live case (its two coloured arms). Not
re-scanned for other `Cost::Tap` + `activation_condition` mana abilities in this task —
a corpus grep for `AbilityDefinition::Activated` entries with both a non-`None`
`activation_condition` and a mana-producing effect would find the full set.

---

## Roster items not reconciled (§3 step 8, explicitly out of this agent's scope)

Per the coordinating instructions, this task's scope was §3 steps 1–7, §6 tests, and
§9's card-def work (horizon lands + `mana_filter.rs` / `effect_choose_gate.rs`). The
broader roster reconciliation in `memory/primitives/sr34-affected-defs.md` (39 Table-1
rows, 11 Table-2 rows) was **not** worked item-by-item beyond what the engine change
naturally resolved. For the record, post-SR-34 status of the roster's remaining classes:

- **7 filter lands** (Cascade Bluffs, Fetid Heath, Flooded Grove, Graven Cairns, Rugged
  Prairie, Sunken Ruins, Twilight Mire) — now register as real mana abilities (the
  engine widening applies uniformly), but remain `known_wrong` in their markers (their
  hybrid `{W/B}` cost is unenforced — `ManaPool::can_spend` ignores `hybrid`; see §8
  item 6 of the plan and the rewritten `mana_filter.rs` module doc). Marker status
  untouched by this task — reconciling their `Completeness` is the roster-reconciliation
  work, not done here.
- **RemoveCounter group** (Druids' Repository, Gemstone Array, Ramos) — untouched;
  `mana_ability_cost_components` correctly excludes `Cost::RemoveCounter` (needs a
  `counter_cost` field + payment path, per the plan's roster analysis).
- **Sacrifice-another group** (Ashnod's Altar, Phyrexian Altar, Phyrexian Tower) —
  untouched; correctly excluded (needs a caller-supplied `ObjectId`, which
  `Command::TapForMana` has no payload for — the Krark-Clan Ironworks class, plan §2).
- **`Cost::Mana(∅)` mismodelling** (Elvish Spirit Guide, Simian Spirit Guide) — these
  DID start registering as (free, repeatable, stackless) mana abilities post-widening,
  since `Cost::Mana(ManaCost::default())` trivially satisfies `mana_ability_cost_components`
  and its `mana_value() == 0` short-circuits the cost-legality check. This is the SAME
  pre-existing bug the defs' own `known_wrong`/`partial` notes already document ("ships a
  FREE, repeatable ... Add {G}" / "{R}") — SR-34 does not make it worse in kind (it was
  already a free mana source pre-SR-34, just reached via the stack instead of
  `TapForMana`), but it is now reachable with no priority window at all. Verified via
  the mechanical before/after probe (§3 step 6); no test references either card. Not
  reconciled — out of scope per the plan's §8 item 5 framing (the real fix is
  `ActivationZone::Hand` + an exile-self-from-hand cost, unrelated to SR-34).
- **`Cost::SacrificeSelf` mismodelling** (Food Chain) — same story: now registers as a
  mana ability (bare `SacrificeSelf`, no tap, matches the Goldhound/Treasure shape) but
  the def is already a known-wrong placeholder (sacrifices Food Chain itself instead of
  exiling a creature). Unreferenced by any test. Not reconciled.
- **Staff of Compleation** — a genuine SF-6 index shift (its `{T}, Pay 2 life: Add one
  mana of any color` ability moved out of `activated_abilities`, shifting the three
  abilities after it down by one index each). Verified via the mechanical probe;
  confirmed via grep that no test or script references Staff of Compleation by name at
  all, so nothing regresses. Left as-is; noted here so it is not rediscovered as a
  mystery (mirrors SF-6's own original note about Creeping Tar Pit from SR-33).

---

## SF-11 — `effect_choose_gate.rs` blocks `AddManaChoice` from `Complete` on a **false** justification, and lets the identically-broken `AddManaAnyColor` through

**Severity: MEDIUM. Filed by `scutemob-90` (criterion 4767). Verified by probe.**

`no_complete_def_uses_the_add_mana_choice_stub`'s doc comment
(`crates/engine/tests/core/effect_choose_gate.rs`) says:

> Unlike `AddManaAnyColor` — which escapes into a real `ManaAbility` with `any_color: true`
> via `try_as_tap_mana_ability` and so never reaches that arm — `AddManaChoice` is not
> recognised there, so its users always route through the stack and into the colorless arm.
> That asymmetry is why sharing the match arm is not the harmless simplification it looks like.

**The asymmetry does not exist.** Escaping into a `ManaAbility` does not avoid the defect:

- `handle_tap_for_mana` step 8 (`rules/mana.rs`): `if ability.any_color { ... mana_pool.add(ManaColor::Colorless, amount) }`
- `Effect::AddManaAnyColor` (`effects/mod.rs`): `ps.mana_pool.add(ManaColor::Colorless, 1)`

**The same one colorless mana on both paths.** Probed: Mana Confluence (`any_color: true`, a
real `ManaAbility` post-SR-34, stack empty) → pool `C5→C6`. Goldhound → `C5→C6`. Phyrexian
Altar (stack path) → `C5→C6`. Identical.

So the gate refuses `AddManaChoice` in `Complete` defs while admitting `AddManaAnyColor`, which
does exactly the same wrong thing, justified by a claim that is false. Per **CR 106.1a/106.1b**
colorless is a mana *type*, not a colour — so "Add one mana of any color" producing `{C}` is not
a degraded choice, it is outside the legal option set. `scutemob-90` demoted the 8 affected
`Complete` defs by hand (see `memory/primitives/sr34-roster-reconciliation.md`); the gate should
enforce it.

**Fix shape**: extend the gate to `AddManaAnyColor` / `AddManaAnyColorRestricted` /
`AddManaOfAnyColorAmount` (all three add colorless), and correct the doc comment. Not done here
because it demotes defs beyond this task's roster and moves headline coverage — the same reason
EF-13 was deferred. Delete all of it when a colour channel for `any_color` lands.

**Known live `Complete` victims outside the SR-34 roster (added by the SR-34 review fix
phase, `scutemob-90`, per review Finding 5)**: `birds_of_paradise.rs:38` and
`command_tower.rs:21` both ship `Effect::AddManaAnyColor` on a `Complete` def (Birds of
Paradise marked explicitly; Command Tower `Complete` by default, no marker) and both
produce `{C}` by the identical mechanism demonstrated above (probed: `handle_tap_for_mana`
step 8 / `Effect::AddManaAnyColor`'s stack arm both add one `ManaColor::Colorless`). Two
of the most-played cards in the format, deck-legal today, carrying the same defect Mana
Confluence was demoted for. Not demoted by `scutemob-90` — see the scope-bound note at
`memory/primitives/sr34-roster-reconciliation.md` §1. This roster is a starting point, not
exhaustive; the corpus-wide sweep that would complete it is the SF-11 fix itself.

---

## SF-12 — `every_complete_land_registers_each_printed_tap_mana_color` is structurally blind to every "any color" land

**Severity: MEDIUM. Filed by `scutemob-90`. Verified by reading both halves of the chain.**

SR-33's broad colour gate cannot see the SF-11 defect class, for **two independent reasons**,
either of which alone is sufficient:

1. **The parser never produces a colour.** `printed_tap_mana_colors` walks the clause after
   `": Add "` and requires `strip_prefix('{')`. `"Add one mana of any color."` has no brace, so
   the walk breaks immediately and `clause_colors` is empty. The gate then hits
   `if printed.is_empty() { continue; }` — **the card is skipped entirely**.
2. **The registered side is empty too.** `registered_colors` reads `ma.produces.keys()`, and an
   `any_color: true` `ManaAbility` leaves `produces` **empty** (probed: Mana Confluence
   `MA[0] produces={} any_color=true`). So even if `printed` were populated, `registered` would
   report `{}`.

Mana Confluence therefore passes this gate **vacuously** while producing a colour it does not
print. This is the same hole in a *checker* that SR-33's own root cause was, and the tenth-plus
consecutive SR finding of that shape.

**Fix shape**: teach the parser the `"one mana of any color"` / `"an amount of mana of that
color"` phrasings (→ "all five colours"), and teach `registered_colors` that `any_color: true`
means "claims all five" — at which point Mana Confluence fails with
`missing {W,U,B,R,G}, invented {}`, correctly. Pairs naturally with SF-11. **Whichever lands
first must not land alone** — fixing the parser without fixing `registered_colors` produces a
`missing`-everything failure whose message blames the wrong side.

---

## NOT A FINDING — `ChooseColor` / `ChooseCreatureType` are **not** ungated stubs (claim falsified)

Recorded so it is not re-filed. `scutemob-90` was asked to check whether
`ReplacementModification::ChooseColor(Color::White)` and `ChooseCreatureType(SubType("Human"))`
are hardcoded defaults slipping past SR-33's `def_uses(d, "Choose")` exact-key match — a fourth
stub family in the `Effect::Choose` class.

**Falsified by probe.** They are *fallback* defaults behind a real deterministic heuristic
(`rules/replacement.rs`, CR 613.1d/e, CR 614.12a): the engine scans the controller's battlefield,
counts layer-resolved colours / creature subtypes, and picks the most common, falling back to the
declared default only when the board is empty (with a documented tie-break preferring the
default).

- Temple of the Dragon Queen on a **white** board → `chosen_color=Some(White)`.
- Temple of the Dragon Queen on a **black** board → `chosen_color=Some(Black)`. The declared
  default is White, so the heuristic demonstrably overrides it.
- Three Tree City with two Human **Soldiers** → `chosen_creature_type=Some("Soldier")`, **not**
  the declared `"Human"`.

This is categorically unlike `Effect::Choose`, which literally executes `choices.first()` and has
no board-sensitivity at all. The choice is deterministic rather than player-made (an M10
simplification), but it always yields a **legal** option — so it does not belong in the stub
family and `def_uses(d, "Choose")` is not under-matching here.

Temple of the Dragon Queen is `Partial` (CR 605.3b — `AddManaOfChosenColor` has no
`try_as_tap_mana_ability` arm, so it uses the stack), **not** `KnownWrong`. Reading the
`Color::White` literal and concluding "always White" — without running it — would have shipped a
false marker on a card whose colour logic works. That is the `megrim.rs` calibration error from
SR-33's marker sweep, in the opposite direction.
