# SR-5 — KeywordAbility Catch-All Audit

<!-- last_updated: 2026-07-10 -->

> Task: `scutemob-57`. Companion to `docs/sr-remediation-plan.md` and to
> `docs/sr-4-silent-failure-audit.md`, which swept the same files for a different
> failure mode. Line numbers refer to the pre-change tree at `12e8a44e`.

## The task's premise was wrong, and the real hazard is worse

The ESM description reads:

> ~117 `_ => {}` catch-all arms on `KeywordAbility` across the six big rules files
> (abilities.rs: 54, resolution.rs: 24, effects/mod.rs: 22, replacement.rs: 17, plus
> casting.rs/layers.rs) mean a newly added keyword variant compiles everywhere and
> silently does nothing.

The counts are real; the attribution is not. Those ~117 wildcard arms exist, but
they belong to matches on **other enums**. Grouping each catch-all-bearing `match`
in those six files by the enum its arms name:

| Enum matched | Catch-all arms |
|---|---:|
| (no path pattern — bool, tuple, `Option`, integer, …) | 38 |
| `ZoneId` | 19 |
| `ZoneChangeAction` | 17 |
| `AbilityDefinition` | 20 |
| `TargetRequirement` | 6 |
| `AdditionalCost` | 6 |
| `ResolvedTarget` | 5 |
| **`KeywordAbility`** | **2** |
| everything else (`GameEvent`, `PlayerTarget`, `CounterType`, …) | ~24 |

Across the **entire engine crate** there are only 15 `match` expressions whose arms
name `KeywordAbility` at all. So "117 catch-alls swallow new keywords" describes a
hazard that does not exist in that shape.

What does exist is larger and has no `match` to audit. A keyword is read the way a
keyword is *asked about*:

```rust
chars.keywords.contains(&KeywordAbility::Flying)
```

There are ~350 such reads across 23 files. A newly added variant is not swallowed by
a wildcard arm — **it is never mentioned by anything**, which is a strictly quieter
failure. No arm to find, no `_ => {}` to grep for. The only thing that names every
variant is `state/hash.rs`, and naming a keyword there assigns it a byte for state
hashing; it does not wire up behavior.

So SR-5's deliverable is not "replace catch-alls with explicit listings" — there is
almost nothing to replace. It is the second half of the task's own text: **a harness
that fails when a new `KeywordAbility` variant lands unhandled.**

## Part 1 — the 15 `KeywordAbility` match sites, classified

| Site | Arms | Variants named | Catch-all | Verdict |
|---|---:|---|---|---|
| `src/state/hash.rs:641` | 166 | all 166 | no | **EXHAUSTIVE** — discriminant table for state hashing |
| `src/rules/resolution.rs:43` | 71 | 41 | no | **NOT A KEYWORD MATCH** — scrutinee is `StackObjectKind`; keywords appear only in nested payload patterns |
| `src/rules/abilities.rs:2857` | 2 | `Backup` | yes | **PROJECTION** |
| `src/rules/abilities.rs:4931` | 2 | `Renown` | yes | **PROJECTION** |
| `src/rules/abilities.rs:4944` | 2 | `Renown` | yes | **PROJECTION** |
| `src/rules/abilities.rs:4988` | 2 | `Poisonous` | yes | **PROJECTION** |
| `src/rules/abilities.rs:5001` | 2 | `Poisonous` | yes | **PROJECTION** |
| `src/rules/combat.rs:1659` | 2 | `Toxic` | yes | **PROJECTION** |
| `src/rules/replacement.rs:1480` | 2 | `Fabricate` | yes | **PROJECTION** |
| `src/rules/resolution.rs:936` | 2 | `Modular` | yes | **PROJECTION** |
| `src/rules/resolution.rs:973` | 2 | `Graft` | yes | **PROJECTION** |
| `src/rules/resolution.rs:1018` | 2 | `Amplify` | yes | **PROJECTION** |
| `src/rules/resolution.rs:1105` | 2 | `Bloodthirst` | yes | **PROJECTION** |
| `src/rules/resolution.rs:1171` | 2 | `Devour` | yes | **PROJECTION** |
| `src/rules/resolution.rs:1411` | 2 | `Tribute` | yes | **PROJECTION** |

**PROJECTION** means the match is the body of a `filter_map` that pulls a payload out
of one variant:

```rust
let fabricate_instances: Vec<u32> = def.abilities.iter()
    .filter_map(|a| match a {
        AbilityDefinition::Keyword(KeywordAbility::Fabricate(n)) => Some(*n),
        _ => None,
    })
    .collect();
```

The `_ => None` is not a dispatch table with a hole in it — it is the answer to "is
this ability a Fabricate?", and `None` is correct for every other keyword that exists
or ever will. A new variant reaching this arm behaves correctly. **Zero of the 13
catch-alls are hazardous, and none were changed.** Converting them to explicit
166-arm listings would add 2,000 lines and catch nothing.

The two exhaustive matches (`hash.rs`, and `view_model.rs` in the replay viewer) mean
a new variant already fails to compile today. That is a real gate, but a weak one: it
forces you to give the keyword a hash byte and a display string, neither of which is
behavior. Nothing forced you to decide whether the keyword *does* anything.

## Part 2 — the registry

`crates/engine/src/state/keyword_registry.rs` makes that decision mandatory and
machine-checked.

```rust
pub enum KeywordHandling {
    /// Engine code branches on this exact variant, at these files.
    Handled { sites: &'static [&'static str] },
    /// Presence marker only. The rules text is implemented by `carrier`;
    /// `cr` cites the rule that licenses the substitution.
    Marker { carrier: &'static str, cr: &'static str },
}

pub fn handling(keyword: &KeywordAbility) -> KeywordHandling { /* 166 arms */ }
```

`handling` is exhaustive, so **a new variant is a compile error until it is
classified**. The classification is then held honest by `crates/engine/tests/
keyword_registry.rs`, which checks it against the source tree in both directions.

### Results

| Class | Variants |
|---|---:|
| `Handled` | 148 |
| `Marker` | 18 |
| **Total** | **166** |

### The 18 marker-only keywords

These have **zero** reads anywhere in the engine. Each is a legitimate presence
marker: the Comprehensive Rules define the keyword as shorthand for an ability the
engine already models as a first-class construct, so `keywords.contains(..)` is the
only thing that ever needs to see the variant. Every CR number below was read from
the `mtg-rules` MCP server against the rule text, not from card rulings, and every
one already matched the citation in the variant's doc comment in `state/types.rs`.

| Keyword | CR | What the rule says it means | Carrier in the engine |
|---|---|---|---|
| `Equip` | 702.6a | "[Cost]: Attach this permanent to target creature you control. Activate only as a sorcery." | `Effect::AttachEquipment` / `Effect::DetachEquipment` |
| `Fortify` | 702.67a | "[Cost]: Attach this Fortification to target land you control." | `Effect::AttachFortification` |
| `Transmute` | 702.53a | "[Cost], Discard this card: Search your library for a card with the same mana value…" | `AbilityDefinition::Activated` (`Cost::DiscardSelf` + `Effect::SearchLibrary`) |
| `Adapt` | 701.46a | "If this permanent has no +1/+1 counters on it, put N +1/+1 counters on it." | `Effect::Conditional(SourceHasNoCountersOfType)` → `Effect::AddCounter` |
| `Outlast` | 702.107a | "[Cost], {T}: Put a +1/+1 counter on this creature." | `AbilityDefinition::Outlast { cost }` |
| `Craft` | 702.167a | "[Cost], Exile this permanent, Exile [materials]: Return this card transformed…" | `AbilityDefinition::Craft` + `Command::ActivateCraft` |
| `Kicker` | 702.33a | "You may pay an additional [cost] as you cast this spell." | `AbilityDefinition::Kicker { cost, is_multikicker }` |
| `Buyback` | 702.27a | additional cost; spell returns to hand instead of graveyard | `AbilityDefinition::Buyback` + `AltCostKind::Buyback` |
| `Bestow` | 702.103a | alternative cost; spell becomes an Aura | `AbilityDefinition::Bestow` + `AltCostKind::Bestow` |
| `Overload` | 702.96a | alternative cost; "target" → "each" | `AbilityDefinition::Overload` + `AltCostKind::Overload` |
| `Emerge` | 702.119a | alternative cost; sacrifice a creature, reduce generic | `AbilityDefinition::Emerge` + `AltCostKind::Emerge` |
| `Cleave` | 702.148a | alternative cost; remove bracketed text | `AbilityDefinition::Cleave` + `AltCostKind::Cleave` |
| `Disturb` | 702.146a | alternative cost; cast transformed from graveyard | `AbilityDefinition::Disturb` + `AltCostKind::Disturb` |
| `Prototype` | 702.160a | static; alternative P/T and mana cost | `AbilityDefinition::Prototype` + `AltCostKind::Prototype` |
| `Transform` | 701.27a | keyword *action*: turn the permanent over | `Command::Transform` + `Effect::TransformPermanent` |
| `Manifest` | 701.40 | keyword action | `Effect::Manifest { player }` |
| `Cloak` | 701.58 | keyword action | `Effect::Cloak { player }` |
| `Discover` | 701.57a | keyword action | `Effect::Discover { player, n }` |

Note the shape: **every marker is either a keyword *action* (CR 701, not an ability at
all) or a keyword ability that CR defines as literally equal to an activated /
alternative / additional cost.** That is the whole justification for the class, and it
is why the class should stay small. `marker_keywords_are_the_reviewed_set` pins the
list so a keyword cannot drift into it unnoticed.

## Part 3 — what the gate actually catches

`handling()`'s exhaustiveness is the compile half. Three test-failure halves close
what a compile error cannot reach, because a lazy author can always satisfy a compile
error with a wrong arm.

- **`all_keywords_covers_every_variant`.** Rust cannot enumerate an enum's variants
  without a derive macro, so `all_keywords()` is hand-written and could silently drop
  one — which would make every other test in the file skip it. The test re-derives
  the truth by parsing the `KeywordAbility` declaration out of `state/types.rs`,
  embedded at compile time with `include_str!`, and set-compares. (`include_str!`
  keeps this inside invariant #1: the *library* still does no IO.)

- **`registry_sites_match_the_source_tree`.** For each `Handled` variant, the declared
  `sites` must be non-empty and must **exactly equal** the set of engine files whose
  code names it; for each `Marker`, that set must be empty. Exact equality, not
  containment, so this fires in four directions: a keyword losing its last dispatch
  site, a keyword gaining a site in an unlisted file, a `Marker` that someone starts
  branching on, and a `Handled` entry that has quietly become inert. The non-empty
  check closes the one remaining escape — `Handled { sites: &[] }` on a keyword nothing
  reads would otherwise compare `{} == {}` and pass. (Found by `/review`, not by me.)

- **`marker_keywords_are_the_reviewed_set`.** Pins the 18 names above.

Each has a non-vacuity guard, and they earned their keep immediately: on the first
run, `declared_variants()` had a broken state machine and returned **zero** variants,
while `registry_sites_match_the_source_tree` reported **pass** — because it was
comparing an empty declared set against an empty found set, for every keyword. Only
`declared_variants_parser_is_not_vacuous` and `site_scan_is_not_vacuous` failed. A
green suite proved nothing until the guards were there.

The site scan strips comments, string literals, and char literals before searching,
blanking them in place rather than deleting them. Without that, a doc comment reading
``/// see `KeywordAbility::Flying` `` would register as a dispatch site, and the
anti-rot direction — "this keyword no longer has any code that reads it" — would be
unfalsifiable. `comment_stripper_blanks_prose_and_strings` asserts this directly. The
stripper handles nested block comments, raw and byte strings, and distinguishes a
lifetime (`'static`) from a char literal (`'x'`, `'\''`).

## Part 4 — the trial variant (acceptance criterion 4445)

`KeywordAbility::TrialVariantDoNotShip` was added and driven through four escalating
attempts to sneak it in. Each was caught. It was then reverted; it is not in the tree.

**Stage 1 — add the variant, nothing else.** `cargo build --workspace`:

```
error[E0004]: non-exhaustive patterns: `&KeywordAbility::TrialVariantDoNotShip` not covered
    --> crates/engine/src/state/hash.rs:641:15
error[E0004]: non-exhaustive patterns: `&KeywordAbility::TrialVariantDoNotShip` not covered
    --> crates/engine/src/state/keyword_registry.rs:53:11
```

(and `tools/replay-viewer/src/view_model.rs` once the engine compiles.)

**Stage 2 — satisfy every compile error; classify it as a `Marker`; forget
`all_keywords()`.** Builds clean. `cargo test --test keyword_registry`:

```
test all_keywords_covers_every_variant ... FAILED
  KeywordAbility variants declared in state/types.rs but absent from
  keyword_registry::all_keywords(): ["TrialVariantDoNotShip"].
  Add them, and classify them in handling().
```

**Stage 3 — add it to `all_keywords()`, still an unreviewed `Marker`.**

```
test marker_keywords_are_the_reviewed_set ... FAILED
  the set of marker-only keywords changed. Each entry means "this keyword needs no
  engine dispatch" — justify the change in docs/sr-5-keyword-catchall-audit.md
  before editing this list.
  left:  {..., "TrialVariantDoNotShip"}
  right: {...}
```

**Stage 4 — reclassify as `Handled { sites: ["src/rules/combat.rs"] }`, then actually
branch on it in `rules/lands.rs`.**

```
test registry_sites_match_the_source_tree ... FAILED
  TrialVariantDoNotShip: declared Handled at {"src/rules/combat.rs"}
  but the source tree says {"src/rules/lands.rs"}
```

There is no ordering of these edits that produces a green build and a green suite for
a keyword nothing reads.

## Deliberately out of scope

- **The 13 projection catch-alls are unchanged.** They are correct, and rewriting them
  as exhaustive listings would trade 2,000 lines for zero coverage. If a future
  keyword ever needs to participate in one of those projections, the registry's
  `Handled` site list for that keyword is what makes the omission visible — not the
  `_ => None`.

- **Catch-alls on the other enums.** `ZoneChangeAction` (17), `AbilityDefinition`
  (20), `ZoneId` (19), `TargetRequirement` (6), `AdditionalCost` (6) and the rest
  carry the same class of hazard, and `AbilityDefinition` in particular is a genuine
  dispatch table where a new variant *would* be silently inert. That is a different
  audit over different enums; the registry pattern here transfers directly. Filed as
  `scutemob-67` (SR-15).

- **`state/hash.rs` is excluded from the site scan** and so is not a "dispatch site"
  for any keyword. It is exhaustive and therefore a compile gate in its own right,
  but a keyword named only there is inert by this audit's definition — which is
  exactly the point.

## Method (reusable)

1. **Check the premise before implementing it.** The task named a count and a
   location. The count was right and the location was wrong, and the fix implied by
   the wrong location (expand catch-alls into explicit listings) would have produced
   a large, useless diff. Group the catch-alls by the enum they match on first.
2. **Find the dispatch surface, not the syntax.** For `KeywordAbility` the surface is
   `keywords.contains(..)`, which has no arms and no wildcard. A hazard that has no
   syntactic marker cannot be found by grepping for a syntactic marker.
3. **Make the classification a value in the code**, the way SR-4 made LKI-vs-bug a
   function call. `handling()` is data the compiler checks for totality and the test
   suite checks for truth.
4. **Every derived set needs a non-vacuity guard.** A test comparing two sets that are
   both empty is green and worthless. Assert the denominators: the parser found >100
   variants, the scan walked >40 files, `Flying` really does appear in `combat.rs`.
5. **Demonstrate the gate adversarially.** Not "a new variant fails to compile" but
   "here are the four cheapest ways to make it compile, and here is the test that
   catches each one."
