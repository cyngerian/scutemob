# PB-DX3b — OOS-DX3-1: the stale-blocker bucket, remainder

**Task**: `scutemob-166` · **Branch**: `feat/pb-dx3b-the-oos-dx3-1-insert-jadar-live-wrong-complete-ophio`
**Seed**: `OOS-DX3-1`, filed by PB-DX3 (`scutemob-164`) in `docs/audits/decision-point-audit.md` §8.1.
**Class**: correctness (live-wrong `Complete` defs) + card yield. **CARD-DEF ONLY** — zero engine lines.

---

## 0. The premise, re-verified on this tree (not taken from the seed)

Every claim the seed makes about the DSL was re-read in source before this plan was written.
All hold, and one of them is **stronger** than the seed says.

| claim | verified where | verdict |
|---|---|---|
| `Condition::Not(Box<Condition>)` exists | `card-types/src/cards/card_definition.rs:3761` | ✅ |
| `Condition::Not` is queue-time evaluable (delegates to inner) | `effects/mod.rs:10109` — `Condition::Not(a) => condition_is_queue_time_evaluable(a)` | ✅ |
| `Condition::YouControlNOrMoreWithFilter { count, filter }` exists | `card_definition.rs:3834` | ✅ |
| …and is queue-time evaluable | `effects/mod.rs:10151` (inside the `true` arm) | ✅ |
| …and honours `TargetFilter.exclude_self` | `effects/mod.rs:10228` — `&& (!filter.exclude_self \|\| obj.id != source)`, added by **PB-EF1** (marker EF-5) | ✅ **the seed is right and `dwynen_s_elite`'s own note is stale** |
| `matches_filter` checks `has_card_type` / `has_keywords` / `has_subtype` | `effects/mod.rs:9507` / `:9512` / `:9548` | ✅ |
| `KeywordAbility::Decayed` exists | `card-types/src/state/types.rs:910` | ✅ |
| `zombie_decayed_token_spec` puts `Decayed` in `keywords` | `card_definition.rs:4263-4281` | ✅ |
| PB-DP6 gates the card-def `intervening_if` at **upkeep** queue time | `rules/turn_actions.rs:310` (covers `AtBeginningOfYourUpkeep` **and** `AtBeginningOfEachUpkeep`) | ✅ |
| …at **end-step** queue time | `rules/turn_actions.rs:781` — its own comment names *"Jadar's zombie token creation"* | ✅ |
| …at **self-ETB** queue time | `rules/replacement.rs:2131` and `:2166` | ✅ |
| resolution-time re-check is retained | `rules/resolution.rs` (PB-DP6 hard constraint 2; PB-DX1 added the `InterveningIf::CardDef` counterpart) | ✅ |

**Consequence**: both halves of CR 603.4 are available for all four defs below, at every
trigger moment they use, with **no engine change**.

### The `#[default]` trap, and why it produced a second live-wrong card

`Completeness` derives `Default` with `#[default] Complete` (`card_definition.rs:196-200`).
A def that ends `..Default::default()` **without** an explicit `completeness:` field is therefore
**`Complete` and deck-legal**. `aurelia_the_warleader` was live-wrong this way (PB-DX1);
`emeria_the_sky_ruin` is live-wrong this way **right now** and the seed did not notice —
it dispositioned Emeria into the "genuinely does not exist yet" pile.

---

## 1. Disposition of all seven bucket defs (AC 5939)

The bucket is the `pb-plan-DP6.md:395` "KEEP — different class" row. It lists nine defs;
`garruks_uprising` and `inventors_fair` were closed by PB-DX3, leaving **seven**. (The seed says
"six more" because it counts `guardian_project` as a half-entry. Enumerated here as seven so
nothing is dropped.)

| # | def | marker today | blocker note says | truth on this tree | **disposition** |
|---|---|---|---|---|---|
| 1 | `jadar_ghoulcaller_of_nephalia` | **`Complete`** (explicit) | "`Condition::NoTokensNamedX` does not exist" | **stale AND chasing the wrong card** — printed text is *"if you control no creatures with decayed"* (MCP), never a token-name filter | **FIX** — gate + correct the stored `oracle_text`; stays `Complete` |
| 2 | `ophiomancer` | `partial` | its own note already says *"Blocker stale"* | stale | **FIX** — gate; `partial` → `Complete` |
| 3 | `dwynen_s_elite` | `inert` | "`exclude_self` is silently ignored by `YouControlNOrMoreWithFilter`'s evaluator" | **stale** — PB-EF1 wired it at `effects/mod.rs:10228` | **FIX** — author the ability + gate; `inert` → `Complete` |
| 4 | `emeria_the_sky_ruin` | **`Complete` by `#[default]`** (no explicit marker) | "needs `Condition::YouControlNOrMorePermanentsWithSubtype`, which does not exist" | **stale** — `YouControlNOrMoreWithFilter { count: 7, filter: has_subtype Plains }` says exactly this today | **FIX the live-wrong half + demote honestly** (see §5) |
| 5 | `vampire_socialite` | `partial` | "`Condition::OpponentLostLifeThisTurn` does not exist" | **true** — not in the enum (`ControllerGainedLifeThisTurn` is the nearest, and it is the wrong side) | **DEFER** — re-affirmed against the current enum; second blocker (conditional ETB counter replacement) also real |
| 6 | `thousand_faced_shadow` | `partial` | "'enters from your hand, if it's attacking' not expressible" | **true** — no zone-of-origin `Condition`; `TargetFilter.is_attacking` is a runtime `GameObject` field `matches_filter` cannot see (`card_definition.rs:1816`, `:3153`) | **DEFER** — re-affirmed |
| 7 | `guardian_project` | `known_wrong` | "name-uniqueness `Condition` does not exist; `TargetFilter.is_nontoken` ignored by `matches_filter`" | **true** on both counts | **DEFER** — re-affirmed; marker is already honest |

Rows 5–7 are **re-affirmed, not copied forward**: each was checked against the current
`Condition` variant list and the current `matches_filter` body this batch, and each note is
updated in-def with a dated re-verification line so the next reader knows *when* the claim was
last true.

---

## 2. Fixes — exact authoring

### 2.1 `jadar_ghoulcaller_of_nephalia` (AC 5940)

Oracle (MCP, verified this batch): *"At the beginning of your end step, if you control no
creatures with decayed, create a 2/2 black Zombie creature token with decayed."*

Three edits, all in `crates/card-defs/src/defs/jadar_ghoulcaller_of_nephalia.rs`:

1. **`oracle_text` field** — replace *"no tokens named Shambling Ghast"* with *"no creatures with
   decayed"*. The stored text was wrong, which is why the blocker note chased a filter the card
   never had. Fix the leading file comment identically.
2. **`intervening_if`** —
   ```rust
   intervening_if: Some(Condition::Not(Box::new(Condition::YouControlNOrMoreWithFilter {
       count: 1,
       filter: TargetFilter {
           has_card_type: Some(CardType::Creature),
           has_keywords: [KeywordAbility::Decayed].into_iter().collect(),
           ..Default::default()
       },
   }))),
   ```
   Leave `TargetFilter.controller` at its default: the `YouControlNOrMoreWithFilter` arm does the
   controller check itself (`obj.controller == controller`, `effects/mod.rs:10208`) and
   `matches_filter` — which takes only `&Characteristics` — **cannot see** a `controller` field.
   Setting it would imply a restriction the predicate does not enforce. Add a one-line comment
   saying so.
3. **Replace the stale TODO block** with a dated note recording what was actually wrong
   (the stored oracle text, not the DSL).

Marker stays `Complete`. Note `Decayed` is checked against **layer-resolved** characteristics —
`expect_characteristics` at `effects/mod.rs:10222` — so a Humility-style effect removing Decayed
correctly re-enables the trigger.

### 2.2 `ophiomancer` (AC 5941)

Oracle: *"At the beginning of each upkeep, if you control no Snakes, create a 1/1 black Snake
creature token with deathtouch."* Rulings (2013-10-17) pin **both** halves of CR 603.4 explicitly:
"if you do, the ability won't trigger" (queue time) and "if you control a Snake when it tries to
resolve, the ability will do nothing" (resolution).

```rust
intervening_if: Some(Condition::Not(Box::new(Condition::YouControlNOrMoreWithFilter {
    count: 1,
    filter: TargetFilter { has_subtype: Some(SubType("Snake".to_string())), ..Default::default() },
}))),
```

**Deliberate deviation from the def's own suggested fix.** The stale note proposes
`Not(ControlCreatureWithSubtype(Snake))`. That variant hard-requires `CardType::Creature`
(`effects/mod.rs:9873`). CR reads "you control no **Snakes**" — permanents with the Snake
subtype, not necessarily creatures. `has_subtype` alone is the exact translation and is a superset
of the ruling's "any creature you control with the creature type Snake" (2013; only creatures
could be Snakes then). Record this choice in-def.

`AtBeginningOfEachUpkeep` triggers on every player's upkeep, but the gate evaluates against
Ophiomancer's **controller** — which is what "if **you** control no Snakes" means. Pin with a probe.

Marker `partial` → **`Complete`**.

### 2.3 `dwynen_s_elite` (AC 5941)

`abilities` is **empty** — the ability must be *authored*, not merely gated. (Same shape as
`inventors_fair` in PB-DX3; expect this, do not be surprised by it.)

Oracle: *"When this creature enters, if you control another Elf, create a 1/1 green Elf Warrior
creature token."* Ruling 2024-11-08 names both CR 603.4 halves explicitly.

```rust
AbilityDefinition::Triggered {
    once_per_turn: false,
    trigger_condition: TriggerCondition::WhenEntersBattlefield,
    effect: Effect::CreateToken { spec: /* 1/1 green Elf Warrior */ },
    intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
        count: 1,
        filter: TargetFilter {
            has_subtype: Some(SubType("Elf".to_string())),
            exclude_self: true,
            ..Default::default()
        },
    }),
    targets: vec![],
    modes: None,
    trigger_zone: None,
}
```

Token: name `"Elf Warrior"`, `card_types: [Creature]`, `subtypes: [Elf, Warrior]`,
`colors: [Green]`, 1/1, no keywords. Follow an existing green-token def for the exact `TokenSpec`
literal shape rather than hand-rolling it.

`exclude_self: true` is the whole point — CR 109.1 "another". The `inert` note claims it is
ignored; **it is not** (PB-EF1, `effects/mod.rs:10228`). A probe must prove the exclusion, i.e.
Dwynen's Elite alone creates **no** token.

Marker `inert` → **`Complete`**.

### 2.4 `emeria_the_sky_ruin` — the second live-wrong `Complete` (§5 for the marker)

Oracle: *"This land enters tapped. / At the beginning of your upkeep, if you control seven or more
Plains, you may return target creature card from your graveyard to the battlefield. / {T}: Add {W}."*

```rust
intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
    count: 7,
    filter: TargetFilter { has_subtype: Some(SubType("Plains".to_string())), ..Default::default() },
}),
```

Emeria is a Land with no Plains subtype, so it does not count itself and `exclude_self` is
unnecessary — say so in the comment rather than setting a field that does nothing.

---

## 3. Probes — fail-before, and **observed, not narrated**

**Standing rule from PB-DX3's single MEDIUM**: every "pre-fix, X happened" sentence in the test
module must be *read off an actual run against the reverted def*, on a fixture where the number
is meaningful. A claim reasoned-to is indistinguishable in a document from a claim observed, and
only the second survives. Revert → run → read → restore, per claim.

New file: `crates/engine/tests/primitives/pb_dx3b_stale_blocker_bucket.rs`
(register the `mod` line — **SR-9a**: a dropped `mod` silently deletes coverage).
Load every def from the real corpus via `all_cards()`; never re-declare a copy.

| # | def | scenario | pre-fix (to be observed) | post-fix assertion | cite |
|---|---|---|---|---|---|
| T1 | jadar | end step, controls a creature **with** decayed | trigger queued + token created anyway | trigger does **not** queue | CR 603.4 |
| T2 | jadar | end step, no decayed creature | token created | token created (regression guard) | CR 603.1 |
| T3 | jadar | decayed creature present at queue time, gone by resolution | — | resolution re-check also declines | CR 603.4 |
| T4 | jadar | `oracle_text` field matches MCP printed text | field said "Shambling Ghast" | field says "creatures with decayed" | — |
| T5 | ophiomancer | own upkeep, controls a Snake | Snake token made anyway | no trigger | CR 603.4 |
| T6 | ophiomancer | own upkeep, no Snake | token made | token made | CR 603.1 |
| T7 | ophiomancer | **opponent's** upkeep, controller has no Snake | — | trigger fires (`AtBeginningOfEachUpkeep`), gate reads *controller's* board | CR 603.4 |
| T8 | dwynen | ETB alone (no other Elf) | vacuous — ability absent pre-fix | **no** token — proves `exclude_self` | CR 109.1 |
| T9 | dwynen | ETB with another Elf on board | vacuous — ability absent pre-fix | token created | CR 603.4 |
| T10 | dwynen | another Elf at queue time, removed before resolution | vacuous | resolution re-check declines | CR 603.4 |
| T11 | emeria | upkeep, 6 Plains | **reanimation happened anyway** (the live bug) | no trigger | CR 603.4 |
| T12 | emeria | upkeep, 7 Plains | reanimation | reanimation | CR 603.1 |

T8/T9/T10's "vacuous" pre-fix state is **honest and must be labelled as such** — the ability did
not exist, exactly as with `inventors_fair`'s T5 in PB-DX3. Do not manufacture a pre-fix number.

---

## 4. Golden script `combat/191` (AC 5940)

`test-data/generated-scripts/combat/191_decayed_jadar_zombie_token_eoc_sacrifice.json`.

Its `generation_notes` and its one open `dispute` both describe a **long-closed** engine gap
("resolution reads `characteristics.triggered_abilities`, which is never populated from
`AbilityDefinition::Triggered`, so the token is never created"). As a consequence the script's
final assertion checks only *"Jadar on battlefield, stack empty, life totals unchanged"* and
**never asserts the token at all** — it passes whether or not the token exists. That is the same
vacuous-assertion failure mode PB-DX3's MEDIUM was about, sitting in the corpus.

Reconcile by **strengthening, not weakening**:

1. Assert the Zombie token **is** on P1's battlefield after the trigger resolves (the gap is closed).
2. Rewrite `generation_notes` to describe current behaviour; keep the CR citations.
3. **Append** a dated dispute `resolution` recording the closure and naming the batch that closed
   it — append-only, do not delete or edit the existing dispute record (PB-DX2 precedent).
4. Verify the script still exercises what it claims: at P1's end step Jadar controls no decayed
   creature, so the newly-gated trigger **still fires**. The script is a positive-path witness;
   the negative path is T1's job, not the script's.

Validate with `SCRIPT_FILTER=191 cargo test --test run_all_scripts -- --nocapture`.
**Do not start the replay-viewer HTTP server** (agents get SIGKILL 137 — `gotchas-infra.md`).

---

## 5. The one genuine judgement call: Emeria's marker

Fixing the intervening-if leaves **one printed clause still unimplemented**: *"you **may** return"*.
There is no free-optional effect in the DSL — `Effect::MayPayThenEffect` requires a `Cost`
(`card_definition.rs:1792`), and PB-DP9's `pending_effect_choice` channel serves search/scry/surveil
only. So the engine takes the reanimation unconditionally.

`Completeness`'s own contract (`card_definition.rs:204-206`): *"Some clauses are implemented and at
least one is not … Deck-build error."* That is `Partial`.

**Decision: set `completeness: Completeness::partial(...)` explicitly**, with a note naming the
missing "may" and cross-referencing the OOS-DP10-8 class (Smuggler's Copter's "you may draw"
authored as an unconditional `Sequence` is the same shape).

Reasoning, stated so the reviewer can attack it:

- Emeria is `Complete` **only by `#[default]`** — nobody ever asserted it was complete. Making the
  marker explicit is correct regardless of which value it takes.
- The alternative (fix the gate, leave it silently `Complete`) reproduces this batch's own subject:
  a marker asserting something no one checked.
- PB-DP10's precedent — *seed, do not demote* — was scoped to a **test-only** batch that was
  forbidden from editing defs. This batch is editing the def, so the precedent does not transfer.
- Net coverage is **+1**, not +3: `ophiomancer` and `dwynen_s_elite` flip up (+2), `emeria` flips
  down (−1) from a value it never earned. Report it that way; do not quietly bank +3.
  *(Corrected during close-out — this line originally read "+2", which was this plan's own
  arithmetic slip. Measured with `tools/authoring-report.py`: **1,142 → 1,143**.)*

**Falsifier**: if the reviewer finds a free-optional mechanism this plan missed, Emeria is
authorable in full and stays `Complete` at +3. Look before concluding.

---

## 6. Gates (AC 5942)

- `git diff --stat main -- crates/engine/src crates/card-types/src` → **empty**. Not just
  `protocol.rs`/`hash.rs` — the whole of both trees (PB-DX3 standard).
- `PROTOCOL_VERSION` **32** / `HASH_SCHEMA_VERSION` **69** unmoved. A card-def edit cannot move
  either; if a gate fires, the design drifted — **stop and report, never hand-bump**.
- `cargo build --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` ·
  `cargo fmt --check` · `tools/check-defs-fmt.sh` (**SR-35** — the only thing that checks the
  1,804 defs; `cargo fmt` checks none of them and still exits 0) · `cargo test --all`.
- **PB-DP10's `decision_gate` / `decision_site_walk` suites must stay green.** Three defs become
  `Complete`, which changes the gate's denominator; none of the four appears in `BASELINE` today
  (checked). If a new `BASELINE` entry is genuinely required, that is a finding to record, not a
  number to paste in.
- Baseline on this branch: **3,998 / 0** (main pin at the `scutemob-164` merge).

## 7. Close-out (AC 5943)

Review phase (`primitive-impl-reviewer`) is **mandatory** — four completeness flips, and
legal-but-wrong is the project's stated top pre-alpha risk. Then: regenerate
`tools/authoring-report.py`; update the CLAUDE.md snapshot with the **honest** coverage delta;
`memory/workstream-state.md` handoff; `seed-rerank-2026-07-27.md` §4 banner (DX3b shipped,
OOS-DX3-1 dispositioned, next = PB-DX4); close OOS-DX3-1 in audit §8.1 with the Emeria finding
recorded; file any new seeds.

**The generalisation to carry** (already in the seed, reinforced by Emeria): a blocker note records
what the DSL could express *on the day it was written*, and nothing re-reads it when a later batch
adds the variant. "Blocked on a DSL gap" is a **dated** claim. Emeria shows the corollary — a def
can be silently `Complete` *and* carry a stale blocker note at the same time, so a corpus-wide sweep
of every `TODO: … DSL gap` / `Blocker stale` note against the current `Condition` enum should also
check whether the def ever declared a marker at all.
