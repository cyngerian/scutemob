# PB-DX4 — the 97-entry decision `BASELINE`, triaged against oracle text

**Seed:** OOS-DP10-8 · **Task:** `scutemob-168` · **Date:** 2026-08-01
**Subject:** `crates/engine/tests/core/decision_gate.rs::BASELINE`

This is the durable record the acceptance criterion calls for. Per-def working notes with
verbatim MCP quotes live in `pb-dx4-triage-batch1.md` … `batch7.md` alongside this file; this
document is the disposition.

---

## What was triaged, and how the roster was derived

PB-DP10 froze 97 `Completeness::Complete` defs that still carry an engine-made choice into a
name-keyed `BASELINE`. An entry asserts **only** that the def hits those `AutoChosen` rows —
nothing about whether the def is otherwise oracle-correct. The table was populated
mechanically and the plan §5.3 class-B / class-D triage was never performed. A closing-review
spot-check of 5 found 2 class-D.

**The roster was derived from `BASELINE` itself, not from prose.** The brief was explicit that
this suite has published a plausible roster and been wrong three times (PB-DP6's 3-vs-14,
PB-DP8's 84-vs-77, PB-DP9's 74/16/8-vs-69/16/7). The const array was parsed directly: **97
entries, 97 distinct names, each mapping to exactly one file** under
`crates/card-defs/src/defs/` — no duplicates, no unmatched names, no name resolving to two
files. All 97 were then read against MCP printed text.

**Definitions used** (plan §5.3):

* **class B** — the def faithfully encodes the printed card; the only deviation is that the
  engine auto-picks among legal options at runtime. This is what a `BASELINE` entry is *for*.
  The def stays `Complete`.
* **class D** — the def is simply wrong against oracle text, independently of the auto-choice.

---

## Result: 84 class-B / 13 class-D — **and this number is not a clean measurement**

The implement phase reported 86 / 11. The closing review found two more class-D defs the
triage had passed (`risen_reef`, `hullbreaker_horror`), so the final split is **84 / 13**.

**Why the number is soft, stated plainly because it is the most useful thing here.** The 97
defs were read by seven sub-agents working in parallel from an identical brief, and they did
not apply an identical standard. `contaminant_grafter`'s costless "you may" was ruled class-D
by batch 2; `risen_reef`'s costless "you may" — the same clause shape, the same effect
primitive, the same forced outcome — was ruled class-B by batch 6, on the reasoning that "the
engine resolves it by always taking the battlefield branch, and that is the auto-choice the row
records". Both readings are defensible in isolation. Summing them is not a corpus measurement,
and the review was right to say so. Treat 84/13 as *at least* 13, established by non-uniform
readers.

**What survives that caveat**: PB-DP10's 2-of-5 spot-check still overstated the D rate
substantially (13/97 ≈ 13% against 40%), and its own "2-of-5 is a very noisy sample" caution
was right. A spot-check of five is not an estimator.

| batch | defs | B | D |
|---|---:|---:|---:|
| 1 (Accursed Marauder … Cankerbloom) | 14 | 14 | 0 |
| 2 (Chaos Warp … Dreadhorde Invasion) | 14 | 13 | 1 |
| 3 (Dromoka … Goblin Ringleader) | 14 | 14 | 0 |
| 4 (Grateful Apparition … Leaf-Crowned Visionary) | 14 | 12 | 2 |
| 5 (Make Disappear … Pull from Tomorrow) | 14 | 13 | 1 |
| 6 (Radstorm … Stubborn Denial) | 14 | 9 | 5 |
| 7 (Sword of Feast and Famine … Yawgmoth) | 13 | 11 | 2 |
| *(as reported by the sub-agents)* | *97* | *86* | *11* |
| **+ found by the closing review** (`risen_reef` b6, `hullbreaker_horror` b4) | | **−2** | **+2** |
| **total** | **97** | **84** | **13** |

The 86 class-B defs are left `Complete` and their `BASELINE` entries stand unchanged. That is
the recorded disposition for them: read on 2026-08-01, faithful to the printed card, engine
auto-choice only.

---

## The eleven class-D defs

Every disposition below was re-verified by the runner against MCP printed text before being
acted on, not taken on the reviewing agent's word.

### Repaired in place — still `Complete` (5)

| def | printed clause (MCP) | what was authored | fix |
|---|---|---|---|
| **Metastatic Evangel** | `{1}{W}`, `Creature — Phyrexian Human Cleric`, `3/1`, "another **nontoken** creature" | `{2}{W}`, `Phyrexian Cleric`, `1/3` (transposed), no token axis | all four corrected; `is_nontoken: true` |
| **Grisly Salvage** | "You may put **a** creature or land card from among them into your hand" | `RevealAndRoute` — routes **every** match, mandatorily | → `LookAtTopThenPlace { optional: true }` |
| **Satyr Wayfinder** | "You may put **a** land card from among them into your hand" | same | same |
| **Sword of Truth and Justice** | "put a +1/+1 counter on **a creature you control**" | bare `TargetRequirement::TargetCreature` | → `TargetCreatureWithFilter { controller: You }` |
| **Radstorm** | `{3}{U}` | `{2}{U}` | corrected |
| **Risen Reef** *(found by the closing review, not the triage)* | "you **may** put it onto the battlefield tapped" | unconditional `RevealAndRoute` — the "may" dropped entirely | → `LookAtTopThenPlace { optional: true }` (also the primitive matching the printed "look at"); the flag is inert, so this records the may rather than implementing it — OOS-DX4-5 |

Metastatic Evangel's fourth defect is the notable one. The def carried a note saying
`is_token` "is only checked in combat_damage_filter paths; for ETB trigger matching it is
silently ignored". **That note was stale.** PB-AC0 forwards the whole `TargetFilter` as
`triggering_creature_filter` through the creature-ETB lowering, and `rules/abilities.rs`
honours `is_nontoken` explicitly on that path. This is the PB-DX3 / PB-DX3b stale-blocker-note
class recurring — reached here by a completely different route (a §5.3 oracle triage rather
than a note sweep), which is evidence the class is bigger than the note-shaped searches that
have found it so far.

Radstorm's was a plain data error, but not a harmless one: a Storm card castable a mana cheap
is castable a turn earlier, and Storm turns that into extra copies (CR 702.40a).

### Demoted, each with an oracle citation (5)

| def | → | why it is not authorable today |
|---|---|---|
| **Smuggler's Copter** | `known_wrong` | "you **may** draw a card. If you do, discard a card" authored as an unconditional `Sequence(DrawCards, DiscardCards)` on both the attack and block triggers — forced loot every attack and block, and a deck-out on an empty library (CR 704.5b). The 20th instance of audit §5's DP-12 class; the other 19 are already `known_wrong`, so the **marker**, not the encoding, was the defect. |
| **Contaminant Grafter** | `partial` | "then you **may** put a land card from your hand onto the battlefield" authored unconditionally. Same costless-"you may" class. |
| **Grateful Apparition** | `partial` | "deals combat damage to a player **or planeswalker**"; `WhenDealsCombatDamageToPlayer`'s only dispatch site gates on `CombatDamageTarget::Player(_)`. `TriggerCondition` has a self player-only variant and an **equipped-creature** any-recipient variant, but no self any-recipient variant. |
| **Thrasios, Triton Hero** | `partial` | "Otherwise, **draw a card**" authored as `RevealAndRoute`'s `unmatched_dest: Hand` — a zone move, not a draw, so no draw event fires and draw triggers, draw replacements (Notion Thief / Leovold / Hullbreacher), PB-DP5's `WouldDraw`/dredge channel and "can't draw" restrictions are all bypassed. No `Effect` branches a reveal between a zone destination and a real draw. |
| **Shambling Ghast** | `partial` | See below — its three named defects were **fixed**; the marker is for a fourth the fix surfaced. |
| **Hullbreaker Horror** *(found by the closing review, not the triage)* | `partial` | Printed "choose **up to one**", both mode targets declared flat with `mode_targets: None`. `rules/abilities.rs`'s CR 603.3d path skips a trigger when **any** required slot has no legal candidate, and "target spell you don't control" is unsatisfiable unless an opponent has a spell on the stack — so the trigger is dropped outright and the second mode (an unconditional bounce, the card's main use) is unreachable. The **identical** defect Shambling Ghast was demoted for in the same batch; the original pass classified it class-B and recorded the shape only in seed OOS-DX4-2, which is a description, not a disposition. |

The costless-"you may" gap is the same one PB-DX3b hit with `emeria_the_sky_ruin`, and the
search was re-run rather than copied forward: `MayPayThenEffect` requires a `Cost` and a free
one always trivially pays; `MayPayOrElse` and `Effect::Choose` are both barred from `Complete`
by `effect_choose_gate.rs`; PB-DP9's `pending_effect_choice` channel serves search/scry/surveil
only.

### Shambling Ghast in detail — three fixed, one seeded

The brief alleged three deviations. All three held, and all three are **fixed**:

1. **Phantom `KeywordAbility::Decayed`** — MCP keywords are `["Treasure"]` only. CR 702.147a
   made every Shambling Ghast unable to block and self-sacrificing after any attack.
2. **Permanent `MinusOneMinusOne` counter** for a printed "-1/-1 **until end of turn**". Fixed
   with `ApplyContinuousEffect` + `EffectFilter::DeclaredTarget` + `UntilEndOfTurn`, the
   `drown_in_ichor.rs` idiom (a sibling in this same `BASELINE`). A counter is a different
   object: it survives cleanup, is proliferate-able, and annihilates against +1/+1 counters
   under CR 122.3.
3. **Stored `oracle_text`** asserting the Decayed reminder and "When Shambling Ghast
   **enters**" against the def's own `WhenDies` trigger.

The demotion is for a **fourth**, which fixing the others surfaced: the mode-1 target is
declared **flat** on the trigger, so it is required whichever mode is chosen — with no
opponent creature on the battlefield the trigger is removed from the stack (CR 603.3d) and the
controller gets nothing, where the printed card lets them simply take the Treasure.
`ModeSelection.mode_targets` is the CR 601.2c-correct scoping, but **every consumer of it is
on the casting path** (`rules/casting.rs`, plus read-only `rules/queries.rs`) and nothing on
the triggered-ability path reads it — so moving the target there would *drop* the requirement
rather than scope it. Seeded as **OOS-DX4-2**.

### Left `Complete` deliberately (1)

**Staff of Compleation** — printed "Destroy target permanent **you own**" (ownership, CR
108.3) authored as `TargetController::You` (control, CR 109.4). Real and reachable: any
control-change effect breaks it in both directions. `TargetFilter` has **no owner axis at
all**, so controller is the only expression the DSL offers.

It is **allowlisted** in `completeness_deviation_scan.rs` rather than demoted, matching the
shipped `nether_traitor` entry — which is the identical deviation, on an explicitly-`Complete`
def, reviewed and allowlisted back in `scutemob-95`, and whose own note names `athreos` and
`fecundity` as further instances. Demoting the two members that happen to sit inside PB-DP10's
97 would have reported a corpus-wide class as a pair of cards. The class question is
**OOS-DX4-1**.

---

## The WATCH band — ~33 defs read as class-B with a recorded doubt

The sub-agents were told to classify B when unsure and attach a WATCH note rather than inflate
the D count. About 33 did. **Those notes existed only in the seven scratch files until the
closing review pointed out they were absent from this document** (Finding 7), which would have
left the batch's own uncertainty undiscoverable by anyone reading the disposition. The band
matters precisely because the B/D line turned out to be reader-dependent: these are the defs
where the readers said so out loud.

The ones worth a later look, in descending order:

* **`geier_reach_sanitarium`** — batch 3 called this "the strongest promotion candidate in this
  batch" and the review independently confirmed it. Printed: "Each player draws a card, then
  discards a card", with ruling 2016-07-13 spelling out that *all* draws happen, then all
  discards happen. The def is `ForEach { over: EachPlayer, effect: Sequence([DrawCards,
  DiscardCards]) }`, which interleaves per player — so a player's discard choice is made before
  a later player has drawn. Real, and a promotion candidate for the next pass.
* **`birthing_ritual`** — printed "Then you **may** sacrifice a creature", authored as
  `place_cost: Some(Cost::Sacrifice(Creature))`, which the def's own comment says "fires
  whenever a creature is available, even into a whiff". Same free-"may" class as OOS-DX4-5.
* **`frantic_search`** — printed "Untap **up to three** lands" with no "target"; authored with
  real targets, so CR 608.2b can fizzle the whole spell. Second known member of OOS-DX4-6.
* **`deflecting_swat`** — printed "target spell **or ability**"; the def declares
  `TargetRequirement::TargetSpell`. Correct today only because `casting.rs` validates
  `TargetSpell` by zone and never checks `StackObjectKind` — i.e. right by accident.
* **`felidar_retreat`** — "those creatures gain vigilance" as a live-evaluated
  `EffectFilter::CreaturesYouControl`; already the CR 611.2c engine class (OOS-OS7-2, PB-DX5).
* **`nether_traitor`** — the owner-vs-controller approximation; allowlisted since `scutemob-95`
  and now the anchor precedent for OOS-DX4-1.
* **A long cosmetic tail** — stored `oracle_text` fields using pre-2024 templating ("Shambling
  Ghast" for "this creature", dropped "of their choice", omitted reminder text). Not scored as
  class-D anywhere, because no behavioural clause is implicated. Worth one mechanical sweep
  someday, not a card-by-card pass.

Per-def WATCH text is in `pb-dx4-triage-batch1.md` … `batch7.md`.

---

## New seeds

| id | finding |
|---|---|
| **OOS-DX4-1** | **Owner-vs-controller is a corpus class, not two cards.** How many `Complete` defs approximate a printed ownership clause ("you own", "your graveyard", "an opponent owns") with `TargetController`? Four are known — `staff_of_compleation`, `nether_traitor`, and the `athreos` / `fecundity` pair the latter's note names — and all were found by accident. `TargetFilter` has no owner axis; adding one is the fix, and the class needs counting before it is sized. |
| **OOS-DX4-2** | **`ModeSelection.mode_targets` is honoured only on the casting path.** Every consumer is in `rules/casting.rs` (+ read-only `rules/queries.rs`); nothing on the triggered-ability path reads it. So a modal *triggered* ability must declare its targets flat, and they are then required for **every** mode — `shambling_ghast` (demoted here) and `hullbreaker_horror` (flagged in batch 4) both have this shape. Engine change, wire impact unmeasured. |
| **OOS-DX4-3** | **Decayed has no golden-script coverage.** `baseline/112` was the corpus's only Decayed script and tested it on a card that does not have the keyword. CR 702.147a keeps 12 unit tests in `mechanics_a_d/decayed.rs`, so engine coverage is intact, but no golden script exercises it. The only remaining in-corpus source is the token `jadar_ghoulcaller_of_nephalia` creates, which a script's `initial_state` cannot place. |
| **OOS-DX4-5** | **`LookAtTopThenPlace`'s `optional` flag is inert, so five `Complete` defs record a printed "you may" they do not implement.** `effects/mod.rs`'s arm destructures `optional: _` (pre-existing OOS-DP10-5). Members: `birthing_ritual`, `growing_rites_of_itlimoc`, `grisly_salvage`, `satyr_wayfinder`, `risen_reef` — the last three touched by PB-DX4, the first two shipped `Complete` long before it. Deliberately NOT demoted piecemeal, on the OOS-DX4-1 rule: settle a shipped class at once rather than demote whichever members a batch happened to read. Closing it needs `optional` honoured — which, being a free choice with no cost, has no channel today (`MayPayThenEffect` needs a `Cost`; `MayPayOrElse` and `Effect::Choose` are barred from `Complete`), so it is the same underlying gap as DP-12 and probably closes with it. | correctness (agency), 5 `Complete` defs; engine change | filed by PB-DX4 fix cycle (`scutemob-168`, review Finding 5) |
| **OOS-DX4-6** | **An untargeted printed choice authored as a real target changes what can be chosen and adds a fizzle mode.** CR 115.10: an effect targets only if it says "target". `sword_of_truth_and_justice` ("put a +1/+1 counter on **a creature you control**") and `frantic_search` ("untap **up to three** lands") both say no such thing and both declare real `TargetRequirement`s, so hexproof / shroud / protection / "can't be the target of" wrongly bite, and CR 608.2b fizzles the whole effect if the chosen permanent leaves — for the Sword that costs the proliferate as well as the counter. The DSL has no choose-on-resolution-without-targeting channel, and `EffectTarget::AllPermanentsMatching` is the wrong shape (it takes every match, not one). Both members ship `Complete` and are allowlisted in `completeness_deviation_scan.rs`. Enumerate the class before deciding it; the two known members were both found by accident. | correctness (latent: needs hexproof/shroud/protection or a leaves-the-battlefield response); engine change | filed by PB-DX4 fix cycle (`scutemob-168`, review Finding 6) |
| **OOS-DX4-4** | **There is no `Wastes` def.** `basics_for_colors` has no colourless basic to return, which is what made the CR 903.5c Forest padding (OOS-M11-6) reachable. Worked around by padding from identity-legal colourless cards; authoring Wastes would let a colourless commander's deck use basics like any other. |

## Seeds closed

* **OOS-DP10-8** — this document is its closure.
* **OOS-M11-6** — the CR 903.5c colourless-commander Forest padding, closed in
  `crates/simulator/src/deck.rs`. Found independently here: PB-DX4 demoted
  `thrasios_triton_hero`, a Legendary Creature and therefore a member of `random_deck`'s
  commander pool, which shifted every subsequent `rng.random_range(0..commanders.len())` and
  landed seed 9001's seat 2 on Rograkh, Son of Rohgahh — the corpus's **only** colourless
  `Complete` legendary creature, so ~1% of draws hit it and no pinned seed ever had. Fixed the
  way the seed itself preferred (pad from the identity-legal colourless pool: measured 40
  colourless nonbasic lands + 82 colourless nonlands = 122 singletons against the 99 a deck
  needs) rather than by excluding colourless commanders, so they stay playable. **The 122 was itself corrected before this document was final**: the first probe reported 123 because it counted Rograkh himself, and `random_deck`'s `eligible` closure excludes the commander. Off by one, harmless to the conclusion, and recorded because the failure mode — measuring a set slightly different from the one the code uses — is exactly the class this batch spent its day finding in other people's documents. **Both** Forest
  fallbacks were removed — the audit was right that there were two, and that each named Wastes
  in a comment and pushed `forest` anyway.

---

## Numbers, each re-measured rather than derived

| pin | before | after | why |
|---|---:|---:|---|
| coverage (`Complete` defs) | 1,143 | **1,137** | 6 demotions, 0 promotions |
| coverage % | 63.4% | **63.0%** | of 1,804 |
| `MAX_AUTO_CHOSEN_COMPLETE_UNION` | 97 | **91** | the 6 demoted defs leave `BASELINE` |
| `completeness_deviation_scan` floor | 661 | **667** | non-`Complete` count +6 |
| `canonical_walk_reproduces_pb_dp8_roster` | 76 | **74** | `shambling_ghast` and `hullbreaker_horror` both carried `targets` |
| `canonical_walk_reproduces_pb_dp9_rosters` (`scry`) | 16 | **15** | `thrasios` carried `Effect::Scry` |
| workspace tests | 4,040 | **4,048** | 8 new probes |

Two more rows the review added: `FROZEN_2026_07_27` 97 → **91** (it had been left at 97 when
entries departed, which would have let five unjustified `None` entries in through a gate whose
own comment says it must not grow — Finding 3), and `MAX_UNDECLARED` **966**, re-derived after
Finding 4 caught the first version pinning the *pre-batch* 970 as though it were the
post-batch figure.

The union pin moved **three times inside this batch** — 93, then 92, then 91. First because
Shambling Ghast's fourth defect only became visible after its first three were fixed; then
because the closing review found `hullbreaker_horror` carrying that same defect and still
`Complete`. Recorded because it is the argument for reading the value off `T6` rather than
computing it: the arithmetic was right at each moment it was done and wrong by the next.

**Wire and engine gates:** `git diff` over `crates/engine/src` **and** `crates/card-types/src`
is empty; PROTOCOL 32 / HASH 69 unmoved; clippy, `cargo fmt --check` and
`tools/check-defs-fmt.sh` (1,804 defs) clean.

---

## What this triage does *not* establish

Worth stating plainly, because the value of the exercise is easy to overstate:

1. **It is a dated claim.** A def read as class-B on 2026-08-01 can drift, and nothing
   re-reads it. `BASELINE` is not now a "reviewed" table in perpetuity — it is a table that
   was reviewed once, on a date this document records.
2. **It only sees what the DSL encoded.** OOS-DP10-9's blind spot is untouched: a "you may"
   dropped at authoring time leaves no variant to walk. Three of this batch's eleven
   (Smuggler's Copter, Contaminant Grafter, and Grisly Salvage's `may` half) are exactly that
   class, and they were caught by reading oracle text — which is precisely the instrument
   OOS-DP10-9 says is needed and which no gate performs.
3. **It covers 97 of 1,143 `Complete` defs.** These 97 were selected for carrying an
   engine-made choice, not for being error-prone. An 11% class-D rate in this slice is not
   evidence for or against the rate in the other 1,046.

## The corpus-wide `#[default]` question, answered

PB-DX3b closed by naming "which defs never declare a marker at all?" a cheap corpus-wide
question nobody had asked, after `#[default] Completeness::Complete` produced two live-wrong
deck-legal defs (`aurelia_the_warleader`, `emeria_the_sky_ruin`) by different routes.

**Answer: 970 of 1,804 def files never mention `completeness` at all** — a clear majority of
the `Complete` population is `Complete` because nobody wrote the field, not because anyone
decided it. **All eleven** of this batch's class-D defs were in that group, and every batch
agent independently reported the pattern in its own slice before being asked about it.

That does not make the default wrong — requiring an explicit marker on 1,804 files is its own
cost — but it does mean the two anecdotes were not anecdotes. Now ratcheted in the direction
that matters (`pb_dx4_baseline_triage.rs::defs_that_never_declare_a_completeness_marker_are_ratcheted`):
the count may fall, and fails if it grows.
