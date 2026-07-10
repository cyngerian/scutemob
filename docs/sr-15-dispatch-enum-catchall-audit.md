# SR-15 — Catch-All Audit for the Other Dispatch Enums

<!-- last_updated: 2026-07-10 -->

> Task: `scutemob-67`. Discovered during SR-5 (`scutemob-57`). Companion to
> `docs/sr-5-keyword-catchall-audit.md`, which built the registry pattern this task
> transfers. Read that first — it defines the method and the four-stage trial-variant
> demonstration reused here.

## What SR-5 handed off

SR-5 was sent to find "~117 `_ => {}` catch-alls on `KeywordAbility`." The count was
real; the enum was wrong. Grouped by the enum their arms actually name, the catch-alls
sat on **other** enums:

| Enum matched | Catch-all arms (SR-5's count) |
|---|---:|
| `AbilityDefinition` | 20 |
| `ZoneId` | 19 |
| `ZoneChangeAction` | 17 |
| `TargetRequirement` | 6 |
| `AdditionalCost` | 6 |
| `KeywordAbility` | **2** |

SR-5 filed the two it was named after; this task audits the top of that list. The
acceptance criteria name **`AbilityDefinition`** and **`ZoneChangeAction`**.

The headline finding mirrors SR-5's: **the catch-alls are not the hazard, and one of
the two named enums is a genuine dispatch table while the other is not.**

## `AbilityDefinition` — a genuine dispatch table, and the hazard has no arm to grep

`AbilityDefinition` is the card DSL's ability enum: 68 variants (activated / triggered /
static / spell / replacement, plus one per alt-cost and keyword-action mechanic). Unlike
`KeywordAbility`, it is never read through `keywords.contains(..)`. It is **dispatched
on** — but not through a single exhaustive `match`. Its behavior is spread across:

1. **`enrich_spec_from_def`** (`crates/engine/src/testing/replay_harness.rs`) — the
   lowering pass that turns a card def's `AbilityDefinition`s into runtime structures
   (`ActivatedAbility`, mana abilities, continuous effects, …). It is a *sequence of
   `if let AbilityDefinition::X { .. }` loops*, not a `match`. A variant it doesn't
   mention is simply never lowered.
2. **~26 projection sites** of the form
   `def.abilities.iter().filter_map(|a| match a { AbilityDefinition::X { .. } => Some(..), _ => None })`
   — "does this card have an X ability?" — scattered across `rules/{abilities, casting,
   resolution, replacement, engine, sba, mana, turn_actions, plot, miracle, suspend}.rs`
   and `effects/mod.rs`.

The **catch-all arms are all projections** (AC 4500): each `_ => None` is the correct
answer to "is this ability an X?" for every other variant, exactly like the 13
`KeywordAbility` projection catch-alls SR-5 left unchanged. Converting them to
exhaustive 68-arm listings would add thousands of lines and catch nothing.

The real hazard is the one SR-5 named for keywords, one enum over: **a newly added
`AbilityDefinition` variant compiles everywhere, is lowered by nothing, read by
nothing, and does nothing — with no `_ => {}` arm to find.** There is no exhaustive
`match` on `AbilityDefinition` anywhere except `state/hash.rs` (a discriminant table
that assigns a hash byte — not behavior). So `AbilityDefinition` *is* a dispatch table
with a silent hole, and it gets a registry.

### The registry

`crates/engine/src/state/ability_definition_registry.rs`, a direct sibling of
`keyword_registry.rs`:

```rust
pub enum AbilityHandling {
    Handled { sites: &'static [&'static str] },   // engine code reads it, at these files
    Marker  { carrier: &'static str, cr: &'static str },  // inert; a KeywordAbility twin carries it
}
pub fn handling(ability: &AbilityDefinition) -> AbilityHandling { /* 68 arms */ }
```

`handling` is exhaustive, so **a new variant is a compile error until it is
classified**. `crates/engine/tests/core/ability_definition_registry.rs` then checks the
classification against the source tree in both directions (see Part "what the gate
catches" below).

### Results

| Class | Variants |
|---|---:|
| `Handled` | 64 |
| `Marker` | 4 |
| **Total** | **68** |

### The 4 marker variants

These four variants have **zero** engine dispatch sites. Each is an inert **data
duplicate**: it pairs with a `KeywordAbility` twin of the same name that carries the
identical payload, and the engine reads the count/cost from the **keyword**, never from
the `AbilityDefinition` variant. (Cards carry both by the DSL's presence-marker
convention; `enrich_spec_from_def` copies the keyword into `characteristics.keywords`,
and `rules/{lands,turn_actions,resolution}.rs` read it from there.)

| Variant | Twin the engine actually reads | CR |
|---|---|---|
| `Vanishing { count }` | `KeywordAbility::Vanishing(n)` | 702.63a |
| `Fading { count }` | `KeywordAbility::Fading(n)` | 702.32a |
| `Echo { cost }` | `KeywordAbility::Echo(cost)` | 702.30a |
| `CumulativeUpkeep { cost }` | `KeywordAbility::CumulativeUpkeep(cost)` | 702.24a |

Verified directly: `grep 'AbilityDefinition::Vanishing'` (comment/string-stripped) over
`crates/engine` and `crates/card-types` returns nothing outside the declaration, while
`lands.rs:138` reads `if let KeywordAbility::Vanishing(n) = kw`. Same shape for the other
three. This is the same class SR-5's 18 marker keywords are in — "the rules text is
implemented by another first-class construct" — just pointing the other way (an
`AbilityDefinition` deferring to a `KeywordAbility`, rather than the reverse).
`marker_abilities_are_the_reviewed_set` pins these four so one cannot drift in unnoticed.

The other 64 variants are `Handled`, each with the exact set of files whose code names
`AbilityDefinition::<Variant>` outside a comment or string (computed by the same
comment-stripping scan `keyword_registry.rs` uses).

## `ZoneChangeAction` — not a dispatch table; already compile-gated

`ZoneChangeAction` (`crates/engine/src/rules/replacement.rs`) is **not** a card-DSL
dispatch enum. It is a 3-variant **result type** returned by
`check_zone_change_replacement`:

```rust
pub enum ZoneChangeAction { Proceed, Redirect { .. }, ChoiceRequired { .. } }
```

Every one of its ~10 consumers (`sba.rs`, `engine.rs`, `turn_actions.rs`,
`effects/mod.rs`, `replacement.rs`) matches **all three variants explicitly** —
`Proceed`, `Redirect`, and `ChoiceRequired`, with no `_` wildcard. Adding a variant to
`ZoneChangeAction` is therefore **already a compile error** at every site. It needs no
registry: the exhaustive-match compile gate SR-15 builds for `AbilityDefinition` already
exists for `ZoneChangeAction`, by construction, in every consumer.

SR-5's "17 catch-alls on `ZoneChangeAction`" was an over-count of its rough heuristic
(the same premise error SR-5 itself warns about). The only two `_ =>` arms anywhere near
a `ZoneChangeAction` are:

- `replacement.rs:809` `_ => { … ZoneChangeAction::Proceed }` — the scrutinee is
  `ReplacementModification` (the comment reads "Non-redirect modifications (EntersTapped,
  etc.) don't change the zone"); the arm *constructs* a `ZoneChangeAction`, it does not
  match one.
- `replacement.rs:928` `_ => dest` — the scrutinee is the redirect destination, not a
  `ZoneChangeAction`.

Neither is a catch-all *on* `ZoneChangeAction`. Classified: both **projections on other
enums**. No gate is warranted or added.

## `ZoneId` — out of scope, and projections anyway

`ZoneId`'s 19 catch-alls (named by SR-5) are not in this task's acceptance criteria.
Spot-checked, they are all projections — `match zone { ZoneId::Command | ZoneId::Exile =>
…, _ => … }` answering "is this a public zone / the battlefield / a hidden zone?" — where
the wildcard is the correct answer for the remaining zones. `ZoneId` is a small, stable
enum (7 zones, unchanged since M0); a new zone is a once-a-format-generation event, not
the churning dispatch surface `AbilityDefinition` is. Left for a future task if ever
warranted; noted here so the SR-5 hand-off list is fully accounted for.

## What the gate catches

`handling()`'s exhaustiveness is the compile half. Three test-failure halves close what a
compile error cannot (a lazy author can always satisfy a compile error with a wrong arm),
each with a non-vacuity guard:

- **`all_ability_definitions_covers_every_variant`** — `all_ability_definitions()` is
  hand-written (Rust cannot enumerate variants), so it re-derives the truth by parsing
  the `AbilityDefinition` declaration out of `cards/card_definition.rs` (embedded with
  `include_str!`, keeping the library IO-free per invariant #1) and set-compares. Guarded
  by `declared_variants_parser_is_not_vacuous` (>60 variants, plus prefix-pair anchors
  `Static`/`StaticRestriction` and `Morph`/`Megamorph`).
- **`registry_sites_match_the_source_tree`** — for each `Handled` variant the declared
  `sites` must be non-empty and **exactly equal** the comment-stripped source scan; for
  each `Marker` that scan must be empty. Exact equality fires in four directions (lose a
  site, gain an unlisted site, start branching on a marker, let a `Handled` entry go
  inert). Guarded by `site_scan_is_not_vacuous` (>40 files per root, `card-defs/` never
  scanned, `Spell` really dispatches in `casting.rs`).
- **`marker_abilities_are_the_reviewed_set`** — pins the four marker names.

The comment/string stripper is the same proven code as `keyword_registry.rs`
(`comment_stripper_blanks_prose_and_strings` asserts a doc comment, a string literal, and
a block comment naming a variant are all blanked — without this the anti-rot direction is
vacuous, and `AbilityDefinition::X` appears in many `panic!("… AbilityDefinition::X …")`
strings).

## The trial variant (acceptance criterion 4501)

`AbilityDefinition::TrialVariantDoNotShip` was added and driven through four escalating
attempts to sneak it in. Each was caught. It was then reverted; it is not in the tree.

**Stage 1 — add the variant, nothing else.** `cargo build -p mtg-engine`:

```
error[E0004]: non-exhaustive patterns: `&AbilityDefinition::TrialVariantDoNotShip` not covered
    --> crates/engine/src/state/ability_definition_registry.rs:80:11
error[E0004]: non-exhaustive patterns: `&AbilityDefinition::TrialVariantDoNotShip` not covered
    --> crates/engine/src/state/hash.rs:6271:15
```

**Stage 2 — satisfy both compile errors; classify it as a `Marker`; forget
`all_ability_definitions()`.** Builds clean. `cargo test --test core
ability_definition_registry::`:

```
test all_ability_definitions_covers_every_variant ... FAILED
  AbilityDefinition variants declared in cards/card_definition.rs but absent from
  ability_definition_registry::all_ability_definitions(): ["TrialVariantDoNotShip"].
  Add them, and classify them in handling().
```

**Stage 3 — add it to `all_ability_definitions()`, still an unreviewed `Marker`.**

```
test marker_abilities_are_the_reviewed_set ... FAILED
  the set of marker-only AbilityDefinition variants changed. …
  left:  {"CumulativeUpkeep", "Echo", "Fading", "TrialVariantDoNotShip", "Vanishing"}
  right: {"CumulativeUpkeep", "Echo", "Fading", "Vanishing"}
```

**Stage 4 — reclassify as `Handled { sites: ["crates/engine/src/rules/combat.rs"] }`,
then actually branch on it in `rules/lands.rs`.**

```
test registry_sites_match_the_source_tree ... FAILED
  TrialVariantDoNotShip: declared Handled at {"crates/engine/src/rules/combat.rs"}
  but the source tree says {"crates/engine/src/rules/lands.rs"}
```

There is no ordering of these edits that produces a green build and a green suite for a
variant nothing reads.

## Deliberately out of scope

- **The ~26 `AbilityDefinition` projection catch-alls are unchanged.** They are correct;
  rewriting them as exhaustive listings would trade thousands of lines for zero coverage.
  If a future variant ever needs to participate in one of those projections, the
  registry's `Handled` site list for that variant is what makes the omission visible.
- **`ZoneChangeAction` gets no registry** — it is already compile-gated by exhaustive
  consumption. Adding one would be ceremony over a guarantee rustc already provides.
- **`ZoneId`, `TargetRequirement`, `AdditionalCost`** and the rest of SR-5's list are not
  in this task's acceptance criteria and are projection-dominated; left for a future task
  if ever warranted.
- **`state/hash.rs` is excluded from the site scan.** It is exhaustive, so a second
  compile gate in its own right, but a variant named only there is inert by this audit's
  definition — which is exactly the point.

## Method (reusable, extends SR-5's)

1. **Check the premise.** SR-5 handed off three counts; two of the three named enums did
   not warrant the fix they implied (`ZoneChangeAction` is already gated; `ZoneId` is
   projections). Only `AbilityDefinition` did.
2. **Find the dispatch surface, not the syntax.** `AbilityDefinition`'s hazard is in
   `enrich_spec_from_def`'s scattered `if let`s, which have no `_ => {}` to grep for — the
   same "never mentioned" failure SR-5 found for keywords.
3. **Distinguish a result enum from a dispatch table.** A 3-variant enum consumed by
   exhaustive matches (`ZoneChangeAction`) is already gated by the compiler; a
   68-variant enum lowered by an open-ended sequence of `if let`s
   (`AbilityDefinition`) is not.
4. **Every derived set needs a non-vacuity guard**, and **demonstrate the gate
   adversarially** — both carried over verbatim from SR-5.
