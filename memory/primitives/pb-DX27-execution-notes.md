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

### 2.1 REFUTED and repaired — 10 defs

| def | the false claim | what shipped | marker |
|---|---|---|---|
| `chord_of_calling` | "`max_cmc` should be XValue" (as a gap) | `TargetFilter.max_cmc_amount = EffectAmount::XValue` | `partial` → **`Complete`** |
| `green_suns_zenith` | identical claim, + a phantom oracle clause | same, + phantom clause removed | `partial` → **`Complete`** |
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
2. **The ruling's third sentence was implemented nowhere.** `ALL_LAND_TYPES` having **zero
   users outside its own declaration** is the proof that no CR 305.6 intrinsic-mana
   derivation exists, so a Blood-Mooned land previously lost every ability and gained
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

**Coverage 1,133/1,803 (62.8%) → 1,137/1,803 (63.1%)**, regenerated with
`tools/authoring-report.py`, not derived. Net **+4**: five promotions, one honest demotion.
Inside the brief's "~4-8 flips" estimate — worth recording only because PB-DX26's estimate
was wrong in both directions and the lesson there was that *a card-yield estimate counting
defs to REPAIR measures the wrong thing*. Here it happened to land, and the reason is that
this batch's repairs were mostly on **non-`Complete`** defs, which is the case where a
repair really does flip.

| gate | moved | why |
|---|---|---|
| `completeness_deviation_scan` marker floor | 670 → **666** | a LOWERING — every prior entry in that comment block avoided having to make one |
| `completeness_deviation_scan` ALLOWLIST | +5 entries | the promoted defs carry **historical** prose about the refuted claim; the `hazorets_monument`/`reforge_the_soul` shape one step further |
| `pb_dx42a_continuous_condition_roster` t5 | 1 → **2** | exit (b) — see §6.1 |
| `cards1_equip_target_repair` t6 | 38 → **39** | see §6.2 |
| `pb_dx32_fuzz_output` `CORPUS_COMPLETE` | 1133 → **1137** | `COMMANDER_POOL` measured unchanged at 90 by executing the gate, not reasoned |

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
