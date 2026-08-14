# PB-DX27 — execution notes (`scutemob-209`, 2026-08-13)

v3 queue rank **11**. Seeds: **OOS-CARDS2-8** + **OOS-CARDS2-11** + **OOS-CARDS2-10** +
**OOS-RR3-2**, rider **OOS-ADJ-7**. All five CLOSED. Nine new seeds filed
(**OOS-DX27-1..9**).

This file holds the *measurements* and the *dispositions*. The narrative lives in
CLAUDE.md and `memory/workstream-state.md`; the registry of record for every seed is
`docs/audits/decision-point-audit.md` §8.1.

---

## 0. Headline

**A blocker note is a claim, and nothing in this project had ever re-checked one.**
`OOS-DX3-1`'s closure called the corpus-wide re-check "a cheap standing sweep" and then
closed without filing it; `OOS-RR3-2` is that missing filing, and this batch is the sweep.

Two things are worth carrying forward more than the card repairs:

1. **The population figure the seed was ranked on does not reproduce, and it moved the
   wrong way from what the brief assumed.** The brief said the memo's 67 machine-checkable
   notes was "a FLOOR and a snapshot". It is a snapshot. It is **not** a floor — every
   reproduction is *smaller*. See §1.
2. **Existence is necessary and never sufficient**, and the batch had to decline two
   repairs its own adjudication had marked REFUTED to honour that. See §3.3.

---

## 1. The census — three derivations, and the figure that would not reproduce

The brief mandated re-deriving the population rather than citing the memo's **67**.

| method | definition | result |
|---|---|---:|
| **A — the memo's own, literal** | a gap phrase (`DSL gap` \| `Blocker` \| `does not exist`) and a bare `Effect::`/`Condition::`/`Cost::`/`TriggerCondition::`/`TargetFilter` token on the SAME LINE | **49** |
| **B — ground-truth-restricted** | as A, but the identifier must resolve against a parse of the DSL's declared enums/structs | **46** |
| **C — inverse** | start from the DSL's declared vocabulary, then find every def comment naming one of those identifiers inside a gap-asserting sentence, regardless of phrase | **109** |

**No variant reproduces 67.** The nearest is 62 (memo phrases + a widened identifier
prefix list, case-insensitive); case-sensitive matching gives 35 or 45 depending on the
prefix list.

Recorded as `OOS-DX27-7`. The generalisable point is the one the re-rank memo already
makes about its own line cites: **a count measured against a dated corpus is a snapshot in
both directions**, and "floor" is a claim about monotonicity that nobody checked. Six
PB-DX batches shipped between the measurement and this one, several of them repairing
defs.

The adjudication ran over **method B's 46**, because those are the rows whose identifier
can be machine-decided.

### 1.1 Why the "refutable" label is a candidate list, not a finding list

Method B's 46 are defs where a gap sentence names an identifier that **exists**. That is
not by itself a defect, and the commonest correct shape in this corpus is a *contrast*:

```
// Cost::TapAnotherCreature (no such Cost variant; only Cost::Tap taps this permanent).
```

`Cost::Tap` exists, the note is right, and four defs share exactly this shape
(`glare_of_subdual`, `opposition`, `springleaf_drum`, `azami_lady_of_scrolls`). No textual
rule separates "names the live thing it is contrasting against" from "asserts the live
thing is missing". That is a judgement, and it is why the shipped gate freezes a **count**
rather than pretending to a 107-row verdict list (§4).

---

## 2. Disposition table — the 46 adjudicated defs

Verdicts: **REFUTED** (primitive exists and expresses the clause) · **REFUTED-PARTIAL**
(exists but does not fully express it) · **CONFIRMED** (claim holds; the still-missing
identifier is named) · **STALE-WORDING** (substance right, identifier or sibling statement
wrong).

**Totals: 10 REFUTED · 3 REFUTED-PARTIAL · 30 CONFIRMED · 9 STALE-WORDING · 3 clean.**
(Rows exceed 46 because several defs carry more than one gap claim.)

### 2.1 REFUTED and repaired — 12 rows, 10 of them `REPAIRED_BY_PB_DX27`

The heading of this table's first draft said "10 defs" while listing 12 rows — corrected by
the `/review` (its Issue 11). The 10 is right for `REPAIRED_BY_PB_DX27` in the gate; the two
extras (`the_world_tree`, `encroaching_dragonstorm`) are the `REVIEWED_CONTRAST_MENTIONS`,
which were repaired *and* still legitimately name a live type while asserting a missing
field on it.

| def | the false claim | what shipped | marker |
|---|---|---|---|
| `chord_of_calling` | "`max_cmc` should be XValue" (as a gap) | `TargetFilter.max_cmc_amount = EffectAmount::XValue`, **plus an explicit `Effect::Shuffle`** (fix cycle) | `partial` → **`Complete`** |
| `green_suns_zenith` | identical claim, + a phantom oracle clause | same, + phantom clause removed, + explicit `Effect::Shuffle` (fix cycle) | `partial` → **`partial`** (promoted by the implement phase, **demoted back by the review** — §9) |
| `reconnaissance` | "`Effect::RemoveFromCombat` does not exist" | `Sequence[RemoveFromCombat, UntapPermanent]` | `inert` → **`Complete`** |
| `wight_of_the_reliquary` | "`Cost::Sacrifice` has no another/exclude-self" | `Cost::Sequence[Tap, Sacrifice{exclude_self}]` | `partial` → **`Complete`** |
| `chandra_flamecaller` | "`EffectAmount::HandSize` not in DSL" | `Effect::WheelHand{Discard, ThatMany}` + `DrawCards(1)` | `partial` → **`Complete`** |
| `the_world_tree` | inline TODO: "count_threshold + grant-ability gap" | `AddManaAbility` + `LandsYouControl` + `YouControlNOrMoreWithFilter{6}` | `partial` (God tutor still blocked) |
| `marisi_breaker_of_the_coil` | TODO denied `TargetController::DamagedPlayer` | the goad clause | `inert` → `partial` |
| `ruthless_technomancer` | "`Cost::Sacrifice` threads only a PlayerId" | `MayPayThenEffect` + `PowerOfSacrificedCreature` | `inert` → `partial` |
| `vampire_gourmand` | same false mechanism claim | attack trigger + `CantBeBlocked` grant | `inert` → `partial` |
| `kaito_shizuki` | "unblockable is not a keyword" | −2 Ninja token with `CantBeBlocked` | `partial` |
| `encroaching_dragonstorm` | "`Effect::ReturnToHand` does not exist" | `Effect::MoveZone` (the right primitive) | `partial` |
| `blackblade_reforged` | "Equip legendary creature {3} has no representation" | second Activated ability, `TargetFilter.legendary` | `partial` — see §3.3 |

**`vampire_gourmand` and `wight_of_the_reliquary` cited each other in a loop**: each
deferred to the other as precedent for a blocker that PB-EF1 had closed on 2026-07-18.
Neither note had been re-read since.

### 2.2 The dominant shape, as the CARDS-2 sweep predicted

Nine of the adjudicated defs carry **an inline `// TODO` and a `Completeness` note that
disagree with each other, with the note correct**: `marisi_breaker_of_the_coil`,
`wake_the_dead`, `polymorphists_jest`, `crucible_of_the_spirit_dragon`, `thassas_oracle`,
`scryb_ranger`, `springheart_nantuko`, `megrim`, `the_world_tree`.

`marisi_breaker_of_the_coil` is the clearest instance: its note **literally contains the
word STALE**, while its inline TODO two lines away denies a variant that **six** corpus
defs already use — and the def itself appears in a grep for that variant *only through the
TODO text denying it exists*.

The mechanism is visible and was named by the CARDS-2 sweep: **when a repair pass rewrites
the `Completeness` note it does not revisit the inline TODOs, and the inline TODO is what
the next author reads first.**

### 2.3 CONFIRMED — the notes that were right

30 rows. Each now names an identifier a grep can decide. A selection where the *named*
identifier was wrong even though the substance was right:

- `kogla_the_titan_ape` — "no defender tracking" is false (`PlayerTarget::DefendingPlayer`
  and `EffectFilter::CreaturesControlledByDefendingPlayer` both exist and are consumed);
  the single absent thing is `TargetController::DefendingPlayer`.
- `wayward_swordtooth` — the carrier, the condition and the *block* half all exist; the
  only absent identifier is `KeywordAbility::CantAttack`.
- `moraug_fury_of_akoum` — the note is wrong in **both** directions: it says
  `Effect::AdditionalCombatPhase` does not exist (it does) and that `Condition::MainPhase`
  does (it does not).
- `wake_the_dead` — "delayed sacrifice exists only on `TokenSpec`" is half wrong: the
  runtime machinery (`GameObject.sacrifice_at_end_step`) is generic; only the *writer* is
  token-only.
- The four `Cost::TapAnother*` defs name **four different nonexistent identifiers for one
  gap**. Worth normalising so a future sweep costs one grep.

---

## 3. The wrong-oracle register (OOS-CARDS2-10)

All six defect entries repaired against MCP-verified printed text and **deleted** from
`KNOWN_DIVERGENT_ORACLE_TEXT`. `cards2_printed_field_fidelity` R1–R8 pass **8/8** with them
gone — and it is the register's **own staleness assertion** ("a def that gets repaired and
stays on the list turns the register into a permanent exemption") that forced the deletion.
The gate certified the repair; the batch did not assert it.

| def | the divergence | disposition |
|---|---|---|
| `voldaren_epicure` | **`Complete`, deck-legal**, silently dropped "it deals 1 damage to each opponent" | clause authored (`EffectTarget::EachOpponent`); **stays `Complete`** |
| `qarsi_sadist` | **`Complete`, deck-legal**, dropped its whole second printed clause | **DEMOTED to `partial`** — see below |
| `blasphemous_edict` | wrong in two clauses (cost reduction vs alternative cost; "a creature" vs "thirteen") | text repaired |
| `scheming_symmetry` | different players, and untargeted | text repaired |
| `delighted_halfling` | invented "{T}: Add {G}" | text repaired |
| `flare_of_malice` | a different card's text — **in the header comment too** | both repaired |

**`qarsi_sadist` is demoted, not authored, and that is the honest outcome.** Its second
clause needs a trigger for CR 702.110b, and `grep Exploit crates/card-types/src/cards/card_definition.rs`
returns **zero**. Its two sibling Exploit defs (`fell_stinger`, `sidisi_undead_vizier`) were
already `partial` naming exactly that — **this def was the outlier nobody had ruled on**,
which is `OOS-RR3-1`'s second finding (defs that declare `Complete` explicitly and were
simply never checked) rather than the `#[default]` derive. Filed as `OOS-DX27-2`, which
also records that Exploit's own ETB trigger unconditionally declines the sacrifice
(`resolution.rs:4095-4104`), so nothing is ever exploited and the trigger could not fire
even if it existed.

Correction to the seed: it says "**four** further entries are fixture artefacts". There are
**three** (`OOS-DX27-8`). They are untouched, because a def repair cannot clear a
fixture-shape entry.

---

## 3.3 The two repairs that were REFUTED and still declined

Both are the batch's own subject matter turned on itself: **a primitive existing is not the
same as a primitive working**, and a verdict of REFUTED on the *identifier* is not a verdict
of "safe to author".

- **`kaito_shizuki` −7** (`OOS-DX27-3`). `Effect::CreateEmblem` exists;
  `TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer` exists. But
  `collect_emblem_triggers_for_event` is called from exactly **six** sites
  (`turn_actions.rs:356/362/821/1981`, `abilities.rs:3754/3760`) and **none is a
  combat-damage site**. Authoring it ships a 7-loyalty ability that silently does nothing —
  strictly worse than the honest omission. The adjudicator's REFUTED-**PARTIAL** verdict was
  load-bearing, and collapsing it to REFUTED would have shipped the bug.
- **`blackblade_reforged`'s land-count static** (`OOS-DX27-4`). The DSL shape exists, but
  `resolve_cda_amount` resolves the controller via the **modified object** (the equipped
  creature) rather than the Equipment's controller, which is CR 108.5/611.2c-wrong whenever
  the two diverge. Two sibling defs hit the identical question and also declined
  (`crown_of_skemfar`, `empyrial_plate`). The def therefore stays `partial` — **the applier
  disagreed with its brief and did not promote**, which is the correct call and is recorded
  as an honest non-promotion rather than a silent one.

---

## 4. The rider — OOS-ADJ-7, and the wire prediction that was wrong

`blood_moon` and `magus_of_the_moon` (both `Complete`) registered
`SetTypeLine { card_types: [Land], subtypes: [Mountain] }` over `AllNonbasicLands`, which
strips the **Artifact** card type. The 2020-08-07 ruling, verbatim: *"This effect doesn't
affect names or supertypes"* and *"Nonbasic lands will lose any other **land types** and
abilities they had. They will gain the land type Mountain and gain the ability
'{T}: Add {R}.'"*

Shipped: `LayerModification::SetLandTypes(OrdSet<SubType>)`, the exact analogue of the
already-shipped `SetCreatureTypes`, consuming `ALL_LAND_TYPES` (`types.rs:1890`).

**Three corrections to the filing, all found by measuring rather than reading:**

1. **The population is 3 `Complete` defs, not 2.** `ancient_den` and `treasure_vault`
   (Artifact Land) plus **`dryad_arbor`** (Land Creature), which the filing missed. The fix
   keeps its `Creature` card type *and* its `Dryad` creature subtype while `Forest` becomes
   `Mountain`. The filing's own §7 admitted "no systematic pass was made over the corpus's
   other `SetTypeLine` uses" — the inverse census cost one grep and found the third member.
2. **The ruling's third sentence was implemented nowhere.** No CR 305.6 intrinsic-mana
   derivation exists anywhere in the engine — verified by direct search over
   `crates/engine/src` and `crates/card-types/src`, and corroborated by the CARDS-2 audit
   from the other direction (`lonely_sandbar` and `windbrisk_heights` author explicit
   `{T}: Add` lines *because* nothing derives them). **Corrected by this batch's own
   `/review`**: the first draft argued from `ALL_LAND_TYPES` having "zero users outside its
   own declaration", which is false — `correlated_card_types()` (`types.rs:1962`) reads it.
   Right conclusion, wrong proof, in a batch whose thesis is that a note is a claim. So a Blood-Mooned land previously lost every ability and gained
   nothing — it could not tap for red at all. Now authored as a third Layer-6 static ordered
   after `RemoveAllAbilities`. The general class is `OOS-DX27-1`.
3. **The brief predicted "expected wire impact NONE" and the gate refuted it.**
   `ContinuousEffectDef.modification` is a sibling of `filter`/`duration`, both already in
   the `Command`/`GameEvent` closure, so `LayerModification` is on the wire.
   **PROTOCOL 35 → 36** and **HASH 74 → 75**, both taken from the gates' own output.

The CR 613.8 `(SetLandTypes, AddSubtypes)` dependency arm was re-derived so the Blood Moon +
Urborg interaction survives — a change that would otherwise have broken silently.

---

## 5. The standing gate, and why it is a count

`crates/engine/tests/core/pb_dx27_stale_blocker_notes.rs`.

**Existence oracle**: `blob.contains("Type::Member")` over non-comment DSL source — a
*usage* test, not a declaration parse, because a declaration parser fails **open** (stop
matching and everything reads as absent, i.e. every note correct, i.e. green). The oracle
is itself pinned against **15 hand-adjudicated identifiers** (8 present, 7 absent) and
agreed **15/15**. *A checker whose reference set is derived from the thing it checks can
never disagree with it* — `OOS-DX7` and `OOS-DX8` both, so the check that decides every
other row is checked.

**Needles were calibrated, not chosen**: three candidate sets scored over the corpus (A =
memo's three → 10 defs; B = assertive negation → 18; C = B + `lacks`/`has no` → 24). C
ships; the rejections are recorded with reasons so the next editor does not re-add them.

**R1 is a COUNT ratchet at 107, not a 107-row verdict list.** A 107-row hand-verdict list
is not a review — it is a rubber stamp with 107 signatures, and the next author would append
to it exactly as thoughtlessly as the stale notes it exists to catch. The count forbids
growth and certifies nothing about any individual def, stated up front rather than
discovered later.

**R2 is the closure proof**: the 10 repaired defs must stay OUT of the live-naming set.
Measured: 10 of 12 candidates left the set; the 2 that remain (`the_world_tree`,
`encroaching_dragonstorm`) both name a live *type* while asserting a missing *field* on it,
which is the precision limit that made a count the right instrument.

**R3 gives the blind spot a number**: **357** defs assert a gap while naming no identifier
at all, ratcheted downward-only (`OOS-DX27-6`). That is exactly the population
`OOS-CARDS2-8`'s "make the notes name their primitive" recommendation aims at.

### Revert matrix — the gate

| row | reverted | result |
|---|---|---|
| V-G1 | planted a stale note on a clean def (`lightning_bolt`) | **RED** (ratchet) → restored GREEN |
| V-G2 | reintroduced a stale note on a REPAIRED def (`reconnaissance`) | **RED** (ratchet + closure proof) → restored GREEN |
| V-G3 | made an "absent" identifier appear in source (`Effect::LookAtTopN`) | **RED** (oracle) → restored GREEN |
| V-G4 | repaired a REVIEWED_CONTRAST def's note | **RED** (staleness) → restored GREEN |

All four discriminate; none UNDISCRIMINATED.

The rider's own matrix is 6 rows (V1–V6), all executed red then restored, none
undiscriminated. The two applier batches contributed 5 + 4 positive-direction rows; their
paired *negative*-direction probes stay green under a single-ability revert and are
**disclosed as vacuous in that direction rather than counted as discriminating**.

---

## 6. Corpus movement, and the four gates that announced it

**Coverage 1,133/1,803 (62.8%) → 1,136/1,803 (63.0%)**, regenerated with
`tools/authoring-report.py`, not derived. Net **+3**: four promotions, two honest
demotions. (The implement phase measured +4 / 1,137; the `/review` demoted
`green_suns_zenith` back — §9 — and the corpus was reconciled a second time.)
Inside the brief's "~4-8 flips" estimate — worth recording only because PB-DX26's estimate
was wrong in both directions and the lesson there was that *a card-yield estimate counting
defs to REPAIR measures the wrong thing*. Here it happened to land, and the reason is that
this batch's repairs were mostly on **non-`Complete`** defs, which is the case where a
repair really does flip.

| gate | moved | why |
|---|---|---|
| `completeness_deviation_scan` marker floor | 670 → **666**, then **667** after the demotion | a LOWERING — every prior entry in that comment block avoided having to make one |
| `completeness_deviation_scan` ALLOWLIST | +5 entries | the promoted defs carry **historical** prose about the refuted claim; the `hazorets_monument`/`reforge_the_soul` shape one step further |
| `pb_dx42a_continuous_condition_roster` t5 | 1 → **2** | exit (b) — see §6.1 |
| `cards1_equip_target_repair` t6 | 38 → **39** | see §6.2 |
| `pb_dx32_fuzz_output` `CORPUS_COMPLETE` | 1133 → **1136** | `COMMANDER_POOL` measured unchanged at 90 by executing the gate, not reasoned |

### 6.1 `OOS-ADJ-2` came true on its own gate's first real event

The PB-DX42a roster gate shipped as PB-DX8's rider on the strength of `OOS-ADJ-2`:
*"Nothing gates the size of the corpus population carrying a layer-querying
`ContinuousEffectDef.condition`. It is 1 today… a new conditional static passes
`no_condition_evaluator_resolves_characteristics_directly` and silently joins the
deviation."*

That is precisely what this batch did, by authoring The World Tree's six-lands static. **It
was not silent.** The gate fired, its message named both legal exits, and exit (b) was
taken: the population is now **2**, and the consequences are named rather than absorbed —
`docs/audits/mtg-characteristics-recursion-adjudication.md` §5.2 ranks **PB-DX42b at 13 on
a measured population of exactly 1**, and that premise is now false (`OOS-DX27-9`). Its
§2.3 supply census ("7 deck-legal `Complete` pairs") was computed for the Archangel's
**Artifact** filter and does not carry over to The World Tree's **Land** filter.

Note the pleasing interaction: this batch's *own* Blood Moon fix makes the commonest case
**more** correct, since a Blood-Mooned nonbasic land now keeps its card types and is still
counted as a Land.

### 6.2 Two gates whose "38" meant different things

`cards1_equip_target_repair::t6` pushes `def.name` once per **matching ability**;
`core::cards1_equip_target_roster` R1 builds a **set of names**. They agreed at 38 only
because every equip def happened to carry exactly one equip ability. `blackblade_reforged`
now carries two (CR 702.6c makes "Equip legendary creature {3}" a *separate* ability, not a
second cost), so the ability count moves to 39 while the def count correctly stays 38.
**A coincidence between two numbers is not an invariant.**

---

## 7. Corrections carried back into the sources

| source | what it said | what is true |
|---|---|---|
| seed-rerank memo §1f | 67 machine-checkable blocker notes; brief called it "a FLOOR" | **49** by its literal method at HEAD; a snapshot, and not a floor |
| `OOS-CARDS2-10` filing | "four further entries are fixture artefacts" | **three** at HEAD |
| `OOS-ADJ-7` filing | 2 `Complete` defs affected | **3** — `dryad_arbor` missed |
| `OOS-ADJ-7` filing | scoped to the card-type strip | the ruling's `{T}: Add {R}` clause was also unimplemented |
| dispatch brief | "expected wire impact NONE" | **PROTOCOL 35→36, HASH 74→75**, gate-computed |
| dispatch brief | 4 seeds unrowed in the audit | **5** — the rider `OOS-ADJ-7` was unrowed too |
| `fell_stinger` note | Exploit decline at `resolution.rs:3794` | `:4095-4104` |
| `crucible_of_the_spirit_dragon` note | `Cost::RemoveCounter` at `card_definition.rs:1240` | `:1271` (1240 is the enum header) |
| `the_world_tree` note | `Effect::SearchLibrary` at `card_definition.rs:1648` | `:1701-1719` |

---

## 8. Open after this batch

- **`OOS-DX27-9`** — PB-DX42b's rank premise is invalid; recompute against a population
  of 2 and re-measure the supply side for a `Land`-reading filter.
- **`OOS-DX27-5`** — `MayPayThenEffect` is pay-when-able, and the corpus is inconsistent
  about it: `disciple_of_freyalise` ships `Complete` on the identical shape that
  `ruthless_technomancer` and `vampire_gourmand` now carry at `partial`. One of the two
  readings is wrong and nothing decides which. **Policy call, then a sweep.**
- **`OOS-DX27-2`** — the Exploit trigger condition *and* the interactive sacrifice choice
  (the latter is a `Command` ⇒ PROTOCOL bump).
- **`OOS-DX27-1`, `-3`, `-4`, `-6`, `-7`, `-8`** — as filed.
- The **357** opaque gap notes are the cheapest standing rider available to any future
  card-def batch: every note rewritten to name its primitive moves one def out of the blind
  spot and into the machine-checkable population.

---

## 9. The `/review` fix cycle — 1 HIGH / 5 MEDIUM / 6 LOW, all 12 taken

The reviewer had a shell and used it: it rebuilt the census derivation independently in
Python (reproducing **107** and **357** exactly), re-executed both fingerprint gates, and
planted **11 stale-note shapes** into a clean def to try to defeat the new gate.

**The HIGH is this batch committing its own subject matter, and it is the sharpest finding
of the cycle.** `chord_of_calling` and `green_suns_zenith` were promoted to deck-legal
`Complete` with their printed **"then shuffle"** clause unauthored. `Effect::SearchLibrary`
has no post-search shuffle — its only shuffle is the `shuffle_before_placing` branch
(`effects/mod.rs:3839-3844`), which is the Vampiric-Tutor *shuffle-then-put-on-top* pattern.
And `eldritch_evolution.rs:12-14`, **the very precedent both defs cite**, says so in-source:

> "then shuffle" is modeled explicitly with `Effect::Shuffle` … rather than relying on the
> `SearchLibrary` executor's `shuffle_before_placing` flag, which only shuffles BEFORE
> placing.

So the batch cited a file as precedent and omitted the one thing that file's comment exists
to warn about — §3.3's own lesson ("a primitive existing is not the same as a primitive
working") reproduced inside the batch that wrote it. Both defs now carry an explicit
`Effect::Shuffle`.

**And the fix uncovered a second, worse one.** Checking whether `green_suns_zenith`'s
*other* clause held revealed that `self_shuffle_on_resolution` does not shuffle at all: it
picks `ZoneId::Library(owner)` and plain-moves the card there, with
`resolution.rs:2023-2025` stating the deviation in its own comment ("deterministic library
placement (top of library)"). `nexus_of_fate` is the flag's only other user and is
**`partial`** for exactly that reason. `green_suns_zenith` claiming `Complete` on the
identical mechanism is **the same outlier shape this batch demoted `qarsi_sadist` for** —
so it is **demoted back to `partial`**, and the coverage delta drops from +4 to **+3**.

Two failures of the same kind in one batch, on the two defs it promoted, is the honest
headline: **a promotion to `Complete` is a claim that every printed clause is authored, and
this batch verified that claim by reading rather than by executing.** Which is precisely
why the reviewer's Issue 2 matters.

| # | sev | finding | disposition |
|---|---|---|---|
| 1 | HIGH | "then shuffle" unauthored on two defs promoted to `Complete` | **TAKEN** — explicit `Effect::Shuffle` on both; `green_suns_zenith` additionally **demoted to `partial`** for its second clause |
| 2 | MED | the three criterion-3 headline defs had **zero** behavioural coverage — which is why #1 shipped | **TAKEN** — new `primitives/pb_dx27_headline_defs.rs`, including a `LibraryShuffled` probe that would have caught #1 |
| 3 | MED | the gate's calibration table publishes 24/403 while its own constants are 107/357 | **TAKEN** — table re-measured against the shipped code; the reporter now prints all three rows so it cannot rot again |
| 4 | MED | **74 defs** carry gap prose naming a live identifier reachable only by an out-of-set phrase (`blocked on` 53, `blocker` 23, `unimplemented` 9); invisible to BOTH ratchets and unstated | **TAKEN** — stated as a second recall bound with the measured figure, and given its own downward-only ratchet |
| 5 | LOW | ratchets are per-def, so a def already in the 107 or the 357 is a free-write zone (464 defs, 25.7%) | **TAKEN** — quantified in the module doc |
| 6 | MED | no `/review` artifact; CLAUDE.md and `workstream-state.md` untouched | **TAKEN** — this section, plus both coordination files |
| 7 | MED | merge hazard: `main` advanced by `afd4a72f` (Blood Moon + Urza's Saga flag); the habitual "take the worker's richer `workstream-state.md`" would DELETE it | **TAKEN** — called out in the handoff as a collect-time instruction |
| 8 | LOW | "`ALL_LAND_TYPES` had zero users" is **false** (`correlated_card_types()` reads it) and was asserted as *the proof* in three places | **TAKEN** — corrected in all three; the conclusion was re-verified by direct search and stands. Right conclusion, wrong proof, in a batch whose thesis is that a note is a claim |
| 9 | LOW | `hash.rs` v75 row cites `t_set_land_types_is_hashed`; the test is `t9_...` | **TAKEN** |
| 10 | LOW | the rider test's module doc asserts "PROTOCOL_VERSION unmoved" — the batch's own headline correction | **TAKEN** |
| 11 | LOW | §2.1 heading says 10 defs, table has 12 rows | **TAKEN** |
| 12 | LOW | two moons on the battlefield stack two `{T}: Add {R}` grants (`AddManaAbility` is `push_back`); CR 305.7 gives one intrinsic ability regardless | **TAKEN** — filed as `OOS-DX27-10`, and t6 is noted as structurally unable to see it |

**What the reviewer could NOT defeat, which is worth recording as much as what it could**:
`/* */` block comments are caught (PB-DX8's defeated gate does not recur here); and every
identifier-shape variation it tried — an unlisted type prefix, a bare identifier with no
`::`, a spaced `Effect :: X`, a lowercase member, a needle and identifier on different
lines — falls into R3's opaque count and reddens. The one real escape is phrasing, which is
finding 4 and is now bounded and ratcheted.

### 9.1 The corpus was reconciled TWICE, and that is the batch's cheapest durable lesson

The implement phase re-observed **nine** seeded fixtures after its +4 completeness move
(4 simulator, 5 play-server), each by an executed sweep. The `/review` then demoted
`green_suns_zenith`, the count went **1,137 → 1,136**, and **every seeded fixture in the
workspace re-dealt again** — the `pb_dx32` orphaned-token pair needed a fresh 0..=399
sweep (118 → **18**) and the play-server opening hand had to be read off a new run.

`random_deck` draws its commander from the `Complete` pool and fills by colour identity,
so **one marker flip anywhere in 1,803 defs invalidates every seeded pin in the
repository.** The operational consequence, now written into the fixtures themselves: a
marker-flipping batch must expect to re-observe its seeded pins **after its review**, not
only after its implement phase. Budget for two passes, not one.

### 9.2 Final numbers

| | baseline | final |
|---|---:|---:|
| tests | 4,561 / 0 / 5 | **4,605 / 0 / 5** (46 targets, +44 by name, **0 removals**) |
| coverage | 1,133/1,803 = 62.8% | **1,136/1,803 = 63.0%** |
| PROTOCOL | 35 | **36** (gate-computed) |
| HASH | 74 | **75** (gate-computed) |

New tests by file: `pb_dx27_blood_moon_type_scope` 9, `pb_dx27_headline_defs` 9,
`pb_dx27_stale_blocker_notes` 7, `pb_dx27_stale_blocker_repairs` 9,
`pb_dx27_sweep_repairs_b` 10.

Coverage flips, named: **UP** `chord_of_calling`, `reconnaissance`,
`wight_of_the_reliquary`, `chandra_flamecaller`. **DOWN** `qarsi_sadist` (no
`TriggerCondition::WhenThisExploitsACreature`), and `green_suns_zenith` — which the
implement phase had promoted and the review demoted, the batch's own outlier-`Complete`
pattern recurring inside it.
