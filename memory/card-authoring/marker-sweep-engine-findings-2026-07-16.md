# Engine findings from the marker sweep (scutemob-88) — for SR-33+

Filed, **not fixed** here, per the task brief ("New engine findings get filed as SR-33+
tasks, not fixed inline"). Each was surfaced while auditing completeness markers and
verified against source by the auditing agent; the two marked **[coordinator-verified]**
were additionally re-verified by hand, one of them empirically.

Ordered by severity.

---

## EF-1 — 88 dual/tri lands are `Complete` but produce only their first colour **[coordinator-verified, empirically proven]**

**Severity: HIGH.** These are deck-legal today and silently wrong.

The affected defs model `{T}: Add {G} or {U}` as:

```rust
abilities: vec![AbilityDefinition::Activated {
    cost: Cost::Tap,
    effect: Effect::Choose { choices: vec![AddMana(G), AddMana(U)] },
}]
```

Two independent defects stack:

1. **`Effect::Choose` is a stub.** `crates/engine/src/effects/mod.rs:3190` unconditionally
   executes `choices.first()` ("M9+: interactive modal choice. For M7, execute the first
   option"). There is no `Command` that supplies a choice — the `Command` enum has
   `ChooseDredge` / `ChooseMiracle` / `ChooseDungeonRoom` and no general `MakeChoice`.
   So Tropical Island can only ever produce `{G}`.
2. **They are not mana abilities at all.** `try_as_tap_mana_ability`
   (`crates/engine/src/testing/replay_harness.rs:3630`) recognises `AddMana`,
   `AddManaAnyColor`, `AddManaFilterChoice`, `AddManaScaled`, and the pain-land
   `Sequence([AddMana, DealDamage])` — but **not** `Effect::Choose`. So no `ManaAbility`
   is registered. `handle_tap_for_mana` (`rules/mana.rs:132`) indexes
   `chars.mana_abilities`, so `Command::TapForMana` cannot find them; they resolve as
   stack-using activated abilities. **CR 605.1a violation** — opponents can respond, and
   they cannot be activated while casting.

**Empirical proof** (temporary probe via `enrich_spec_from_def`, since removed):

```
PROBE Tropical Island    mana_abilities=0 activated_abilities=1
PROBE Underground Sea    mana_abilities=0 activated_abilities=1
PROBE Watery Grave       mana_abilities=0 activated_abilities=1
PROBE Breeding Pool      mana_abilities=0 activated_abilities=1
PROBE Forest             mana_abilities=1 produces={Green: 1}
PROBE Island             mana_abilities=1 produces={Blue: 1}
```

**Blast radius: 88 defs, all `Complete`** — the full original dual cycle (Tropical Island,
Underground Sea, Tundra, Badlands, Bayou, Plateau, Savannah, Scrubland, Taiga, Volcanic
Island), every shockland, and the check / fast / slow / temple / guildgate / triome cycles.
Full list reproducible with: inert-scan for `Effect::Choose` whose `choices` are `AddMana`.

**The correct pattern already exists in-repo**: `tainted_field.rs` models the "or" as **two
separate activated abilities, one per colour**, and the deviation-scan allowlist calls this
"faithful decomposition of a hybrid mana ability, fully implemented". `try_as_tap_mana_ability`
converts each into a real `ManaAbility`, and `ability_index` selects the colour. The fix is
likely mechanical (rewrite the 88 defs to the two-ability shape), not an engine change.

**Not actioned here.** Marking 88 staple lands non-`Complete` would invalidate most decks
and many tests; rewriting them is authoring work this task explicitly excludes. Needs a
coordinator decision.

---

## EF-2 — `completeness_deviation_scan`'s allowlist entry for `path_to_exile` rests on a false premise **[coordinator-verified]**

**Severity: HIGH (hole in a checker).**

`crates/engine/tests/core/completeness_deviation_scan.rs` allowlists `path_to_exile` with:

> "'May search' — modelled as MayPayOrElse with zero cost" — **a faithful encoding of the
> optional search, not a simplification of it.**

That justification is false. `Effect::MayPayOrElse` (`effects/mod.rs:3196`) destructures away
`cost` and `payer` and **unconditionally executes `or_else`**. The search therefore *always*
fires; it is not optional at all. The SR-12 reviewer reasoned from the *intent* of the DSL
shape without tracing into the effect's implementation — the exact failure mode
`feedback_verify_full_chain` warns about, inside the gate that exists to catch it.

`path_to_exile` should be `known_wrong`, and its allowlist entry removed. Note the gate is
well-built in one respect: it asserts an allowlist entry is only valid while the file is
still `Complete`, so correcting the marker forces the allowlist edit.

Related: **`rhystic_study`** is `Complete` and its own header comment admits *"the draw
always fires (payment never collected)"*. Its comment contains no deviation needle
("simplif"/"modeled as"/"approximat"/"deviation"), so the scan never looked at it — a
**needle-coverage gap**, not just a bad entry.

---

## EF-3 — `Effect::Choose` / `MayPayOrElse`: "you may" has no correct expression

**Severity: HIGH (systemic authoring hazard).** 95 defs reference `Effect::Choose`.

- `Effect::Choose` always takes `choices.first()` → "you may X" modelled as
  `Choose{[X, Nothing]}` **silently always does X**.
- `Effect::MayPayOrElse` always declines → always `or_else`. Its `payer` field is silently
  unread, while `MayPayThenEffect` two arms below *does* honour its payer (and pays when
  able, so it cannot express "may" either — it over-pays instead).
- There is no `optional` / `may` field on `AbilityDefinition::Triggered`.

Any def whose oracle says "you may" is therefore **not** `Complete` regardless of its marker.
At minimum these variants need doc comments recording that their choice fields are inert;
properly, they need the M9+ interactive-choice work.

---

## EF-3b — `PlayerTarget::ControllerOf` / `OwnerOf` resolve to `ctx.controller` **inside Manifest and Cloak only** **[coordinator-verified — scope corrected]**

**Severity: LOW-MEDIUM.** Recorded here with its **corrected scope**, because the auditing
agent reported it as engine-wide and that is wrong. The correction is the point.

```rust
// crates/engine/src/effects/mod.rs:3609  (inside Effect::Manifest)
// crates/engine/src/effects/mod.rs:3661  (inside Effect::Cloak)
PlayerTarget::ControllerOf(_) | PlayerTarget::OwnerOf(_) => ctx.controller,
```

The target is discarded (`_`). But these are **inline arms local to `Effect::Manifest` and
`Effect::Cloak`**. The canonical resolver `resolve_player_target_list`
(`effects/mod.rs:6287`, `ControllerOf` arm at `:6333`) resolves the object and reads its real
controller, with a stack-object fallback.

Verified that the effects named in the original report use the **canonical** resolver and are
therefore **not** affected:
- `Effect::GainLife` → `resolve_player_target_list` (`effects/mod.rs:502`) — **Swords to
  Plowshares is fine**.
- `Effect::SearchLibrary` → `resolve_player_target_list` (`effects/mod.rs:2787`) — **Path to
  Exile routes the search to the creature's controller correctly** (its defect is optionality,
  EF-2/EF-3, not routing).

45 defs use `ControllerOf`/`OwnerOf`; 31 are `Complete`. They are fine unless they route
through Manifest/Cloak. The two inline arms should delegate to the canonical resolver.

**Method note.** This is the second time in this sweep that a confident, precisely-cited
finding was wrong about *scope* — a real code fragment generalised past its match arm. It is
the same error class the sweep exists to catch, one level up: cite the line, then check what
encloses it.

## EF-4 — `Cost::Sacrifice(TargetFilter)` silently drops most of the filter

**Severity: MEDIUM.** Lowering to the runtime `SacrificeFilter`
(`testing/replay_harness.rs:3743-3767`) reads only `has_chosen_subtype`, `has_subtype`,
`has_card_type`. **`exclude_self` is dropped**, and `SacrificeFilter`'s six variants have no
self-exclusion. A def authoring "Sacrifice another creature" as
`Cost::Sacrifice(TargetFilter { exclude_self: true, .. })` compiles and silently permits
sacrificing the source itself.

## EF-5 — `TargetFilter.exclude_self` ignored by `Condition::YouControlNOrMoreWithFilter`

**Severity: MEDIUM.** `check_static_condition` (`effects/mod.rs:8508-8536`) calls only
`matches_filter` + `check_has_counter_type` and never reads `exclude_self`; it is honoured
only at `effects/mod.rs:7032` / `:7066`. Every "you control another X" card authored against
it gets the inclusive reading with no error.

## EF-6 — `TargetFilter.is_tapped` silently ignored by `matches_filter`

**Severity: MEDIUM.** `is_tapped` is a `GameObject` field enforced only at
`casting.rs:6221/6265/6308/6339` and the `abilities.rs` auto-target picker. `matches_filter`
ignores it, so `Condition::YouControlNOrMoreWithFilter { is_tapped: true }` fires on untapped
creatures. Same shape as EF-5: the field exists, the dispatch cannot see it.

## EF-7 — `CounterType::Stun` is a silent no-op

**Severity: MEDIUM.** `CounterType::Stun` exists and hashes (`hash.rs:670`), but
`grep -r Stun crates/engine/src/rules` returns **zero matches** — nothing implements CR
701.59's untap replacement. `kaito_bane_of_nightmares.rs:82` emits
`Effect::AddCounter { counter: CounterType::Stun }` that no engine code reads.
(`crimestopper_sprite.rs` correctly declines to place one.)

## EF-8 — pain-land damage bypasses the CR 615 replacement pipeline

**Severity: MEDIUM.** `rules/mana.rs:303` applies pain-land damage as a raw
`player_state.life_total -= n` and emits `GameEvent::DamageDealt` **without routing through
damage replacement/prevention**. Prevention shields and damage replacement never see it.
Affects every pain land (`caves_of_koilos`, Ancient Tomb, …).

## EF-9 — `EnchantTarget::Player` exists but is inert

**Severity: LOW-MEDIUM.** `sba.rs:995` returns `false` for it. Any triage that "closes" an
Enchant-player blocker on variant existence alone is wrong (e.g. `curse_of_opulence`).

## EF-10 — `ModifyBothDynamic` family resolves amounts with the wrong controller

**Severity: LOW-MEDIUM.** `layers.rs:1270-1275` resolves the amount using the controller of
the **object being modified**, not the effect's source. For "for each card in *your* hand" on
an equipped creature this reads the creature's controller's hand — identical in normal play,
wrong under gain-control effects. Must be settled before `empyrial_plate` is flipped to
`Complete`.

## EF-11 — `TargetFilter.is_token` doc comment understates its coverage

**Severity: LOW (doc rot, but it misleads authors).** The comment
(`card_definition.rs:2862-2865`) claims the field is checked only in the
`combat_damage_filter` path. It is now also checked on the attack (`abilities.rs:6092/6108`),
death (`:4416`), and ETB (`:6260`) paths. The comment could talk an author out of a valid
filter — it nearly did during this sweep.

## EF-12 — `try_as_tap_mana_ability`'s doc comment is stale (understates coverage)

**Severity: LOW (doc rot, but it caused a near-miss in this sweep).**

`crates/engine/src/testing/replay_harness.rs:3629-3634` says:

> "If `effect` is `AddMana` with exactly one non-zero single-color entry, return a
> corresponding `ManaAbility::tap_for`. … This covers all 5 basic land colors (produces
> exactly 1 mana of one color). **Sol Ring ({T}: Add {CC}) produces 2 colorless — handled
> via ActivateAbility in scripts instead of TapForMana.**"

Both claims are false. `mana_pool_to_ability` (`:3698`) accepts **any** non-zero amounts across
**any** number of colors — there is no "exactly one" or "exactly 1 mana" restriction. Verified
empirically:

```
PROBE Ancient Tomb     mana_abilities=1  produces={Colorless: 2} dmg_to_controller=2  activated=0
PROBE Sol Ring         mana_abilities=1  produces={Colorless: 2} dmg_to_controller=0  activated=0
PROBE Caves of Koilos  mana_abilities=3  {Colorless:1}/{White:1 dmg1}/{Black:1 dmg1}
```

Sol Ring **is** registered as a mana ability. This comment nearly talked this sweep out of a
correct upgrade (Ancient Tomb's `{C}{C}` looked excluded by it). Fix the comment to describe
what `mana_pool_to_ability` actually does.

---

## EF-13 — 105 defs are marked `partial` but register no behaviour at all (they are `inert`) **[coordinator-verified against the compiled registry]**

**Severity: MEDIUM (taxonomy/bookkeeping, not safety).** Raised by `/review` on scutemob-88;
**not fixed here** — see "why deferred" below.

`Completeness`'s taxonomy (`card_definition.rs:186-193`) is explicit:

- `Inert` — "registers with no abilities at all … a blank permanent that happens to have the
  right name, types, and mana cost."
- `Partial` — "Some clauses are implemented and at least one is not."
- `KnownWrong` — "**Every** clause is implemented, but at least one deliberately deviates."

**105 defs marked `partial` implement *nothing*** — `registers_no_behavior` is true for them.
By the taxonomy they are `Inert`. Examples: Academy Manufactor, Birthing Pod, Azami, Black
Market Connections, Brokers Ascendancy, Agadeem's Awakening, Al Bhed Salvagers, Alandra.

**Not a safety issue**: `Partial` and `Inert` are both non-`Complete`, so `validate_deck`
rejects them identically (invariant #9 holds). It is a *bookkeeping* issue — it misreports the
campaign's `todo` vs `empty` buckets — and a *trust* issue, since the taxonomy is the thing this
sweep exists to make reliable.

### The count is 105, not 99 — and the difference is itself the finding

`/review` reported 99 from a source scan. The authoritative number, computed from the
**compiled registry** (`all_cards()` + the `registers_no_behavior` predicate), is **105**.

A text scan undercounts because the regex `abilities:\s*vec!\[\s*\]` also matches the substring
inside **`mana_abilities: vec![]`**, so defs that really do have abilities get miscounted and
defs that don't get missed. This is **the same bug CLAUDE.md already records** against the
authoring report ("The prior 56.2% was an undercount: the authoring report's `abilities: vec![]`
regex also matched nested `mana_abilities: vec![]`"). It was reproduced twice more during this
task — once by the reviewer, once by the worker's own verification script, which is exactly why
`pongify` and `martial_coup` were briefly and wrongly suspected of registering no behaviour.
**Count this class from `all_cards()`, never from source text.**

### Why deferred rather than fixed

- It is 105 defs and would move the campaign's headline buckets (`todo` 667 → ~562,
  `empty` 62 → ~167). That is a reporting change the campaign owner should make deliberately.
- It is **inherited drift**, not introduced by scutemob-88: these markers' *kinds* were already
  wrong; this task corrected their *notes* and their stale *blockers*.
- It is not "a stale marker" in this task's sense (the stated blocker is often perfectly valid).

### Recommended fix (one ticket)

Reclassify the 105, then **machine-enforce the kind↔shape rule** so it cannot recur — the gate
already has the predicate:

```rust
// a def that registers no behaviour is Inert by definition; Partial/KnownWrong both
// claim something IS implemented.
assert!(!(registers_no_behavior(d) && matches!(d.completeness, Partial(_) | KnownWrong(_))));
```

Pair it with the two defs already corrected here (`seedborn_muse`, `scavenging_ooze`), which are
the same error and were fixed because `/review` named them individually.
