# Primitive Batch Review: PB-DX4 — the 97-entry decision `BASELINE`, triaged against oracle text

**Date**: 2026-08-01
**Reviewer**: primitive-impl-reviewer (Opus)
**Seed**: OOS-DP10-8 (+ OOS-M11-6 closed incidentally)
**Task**: `scutemob-168` · branch `feat/pb-dx4-triage-the-97-entry-decision-baseline-against-oracle-`
**CR rules checked**: 603.3d, 601.2c, 608.2b, 613.1e, 514.2, 115.10, 118.12, 704.5b, 108.3/109.4, 903.5b/903.5c, 702.147a, 702.40a
**Engine files reviewed**: `crates/engine/src/effects/mod.rs` (`RevealAndRoute`, `LookAtTopThenPlace` arms — read only), `crates/engine/src/rules/abilities.rs` (CR 603.3d trigger-target skip, `is_nontoken` ETB path), `crates/engine/src/rules/events.rs` (`TriggerTargetChoiceRequired` doc), `crates/engine/src/rules/protocol.rs`, `crates/engine/src/state/hash.rs`
**Test files reviewed**: `crates/engine/tests/primitives/pb_dx4_baseline_triage.rs` (new, 8 tests), `crates/engine/tests/core/decision_gate.rs`, `crates/engine/tests/core/decision_site_walk.rs`, `crates/engine/tests/core/completeness_deviation_scan.rs`, `crates/engine/tests/rules/modal_triggers.rs`, `crates/engine/tests/scripts/run_all_scripts.rs`
**Non-engine files reviewed**: `crates/simulator/src/deck.rs`, `tools/play-server/src/api.rs`, `tools/play-server/src/main.rs`, `test-data/generated-scripts/baseline/112_shambling_ghast_decayed_sacrifice.json`
**Card defs reviewed**: 11 dispositioned defs (all), plus a 12-def sample of the 86 class-B calls: Risen Reef, Coiling Oracle, Sylvan Messenger, Chaos Warp, Goblin Ringleader, Growing Rites of Itlimoc, Hullbreaker Horror, Geier Reach Sanitarium, Yawgmoth, Victimize, Accursed Marauder, Sword of Feast and Famine
**Docs reviewed**: `memory/primitives/pb-dx4-baseline-triage.md`, `pb-dx4-triage-batch1..7.md`, `docs/audits/decision-point-audit.md` §8.1, `docs/authoring-status.md`, `memory/workstream-state.md`, `CLAUDE.md`, `memory/primitive-wip.md`

> **Tooling limitation, stated up front.** This reviewer session had no Bash tool. `cargo
> test --workspace`, `cargo clippy`, `cargo fmt --check`, `tools/check-defs-fmt.sh` and
> `git diff main -- crates/engine/src crates/card-types/src` **could not be executed**. Every
> claim below rests on reading source, and every gate check in §"Gates" says explicitly what
> was verified by reading versus what was taken on the batch's word.

---

## Verdict: needs-fix

The core of the batch is real and well done: all eleven dispositions I re-derived independently
against MCP printed text hold, the five repairs are correct (Metastatic Evangel's four defects,
Radstorm's cost, Shambling Ghast's three, and both put-≤1 arity fixes all verified clause by
clause), the demotion arguments are each backed by a DSL gap I confirmed exists, the
`staff_of_compleation` allowlist is genuinely the same class as the shipped `nether_traitor`
precedent, the golden-script retirement is honest and gated, and every moved pin's arithmetic
reconciles (661+5=666; BASELINE hand-counted at exactly 92 entries against
`MAX_AUTO_CHOSEN_COMPLETE_UNION = 92`; `triggered_targets` and `scry` deltas each attributable
to exactly one demoted def, which I checked by reading the other four defs' `targets` and
effects). The `deck.rs` / play-server excursion was necessary and is correct.

**But the batch's headline deliverable — the classification itself — is not a function of the
card.** Two sub-agents applied contradictory standards to the same pattern and the disposition
document did not notice. `contaminant_grafter` was demoted to `partial` because a dropped free
"you may" "makes the def do something the card cannot make you do" (batch2), while
`risen_reef`'s identical dropped free "you may" was ruled class-B because "the engine resolves
[it] by always taking the battlefield branch — that is the auto-choice the row records"
(batch6). Separately, `shambling_ghast` was demoted for a flat mode-target that CR 603.3d turns
into a dead trigger, while `hullbreaker_horror` — which the batch's own notes and its own new
seed OOS-DX4-2 identify as carrying the same defect, in a strictly worse form — was left
`Complete`. Both survivors are deck-legal today and both do something the printed card cannot
make a player do. Whichever standard is the right one, at least two of these four defs are
mis-dispositioned, and the two left standing are on the unsafe side.

Two HIGH, five MEDIUM, six LOW. Nothing here undoes the batch; the HIGHs are two more members
of classes the batch already opened, plus the doc corrections that follow from them.

---

## Engine Change Findings

None. Consistent with the batch's declared zero-engine gate, no file under `crates/engine/src`
or `crates/card-types/src` carries a PB-DX4 marker, and `PROTOCOL_VERSION = 32` /
`HASH_SCHEMA_VERSION = 69` are both unmoved (read directly at `rules/protocol.rs:335` and
`state/hash.rs:679`). The `crates/simulator` and `tools/play-server` changes are outside that
gate by construction and are assessed under Card/Scope Findings 12 and LOW 6.

---

## Card Definition & Triage Findings

| # | Severity | File / subject | Description |
|---|----------|----------------|-------------|
| 1 | **HIGH** | `crates/card-defs/src/defs/risen_reef.rs` | **A free "you may" left mandatory on a `Complete` def, in the same batch that demoted two cards for exactly that.** Oracle: "you **may** put it onto the battlefield tapped." Def: unconditional `RevealAndRoute`. **Fix:** re-author with `Effect::LookAtTopThenPlace { count: 1, filter: Land, destination: Battlefield{tapped:true}, rest_to: Hand{Controller}, optional: true }` (behaviourally identical today, encodes the "may", and matches the printed "look at"), **or** demote to `partial` alongside `contaminant_grafter`. Then reconcile the triage doc's split. |
| 2 | **HIGH** | `crates/card-defs/src/defs/hullbreaker_horror.rs` | **The trigger is dead in ordinary play and the def stays `Complete`.** Both mode targets are flat with `mode_targets: None`; `abilities.rs`'s CR 603.3d path skips a trigger if **any** required slot has no legal candidate, and "target spell you don't control" is unsatisfiable in the ordinary case. **Fix:** demote to `partial` with the same note `shambling_ghast` now carries, citing OOS-DX4-2. |
| 3 | MEDIUM | `crates/engine/tests/core/decision_gate.rs:762` | **`FROZEN_2026_07_27 = 97` was not lowered when five entries left, opening five unjustified-entry slots in a gate whose own comment says it "deliberately does not grow".** **Fix:** lower to `92` with a dated derivation. |
| 4 | MEDIUM | 4 docs + `pb_dx4_baseline_triage.rs:550` | **"All eleven class-D defs never declared a marker" is false — two of them did.** **Fix:** correct to 9 of 11; correct the "975 that would otherwise stand" parenthetical to 973 / "lowered it by three". |
| 5 | MEDIUM | `grisly_salvage.rs`, `satyr_wayfinder.rs` | **`optional` is inert (OOS-DP10-5); the printed "you may" is still unimplemented, with no in-def note and a test message that says otherwise.** **Fix:** add the PB-DX3 `reveal: true` style in-def comment to both defs, and reword T4's third assertion message. |
| 6 | MEDIUM | `sword_of_truth_and_justice.rs` | **The printed clause has no "target"; the repair fixed the controller axis and left the targeting axis unrecorded and unseeded.** **Fix:** add an in-def note; file the untargeted-choice-as-target class as a seed (batch3 found a second member, `frantic_search`). |
| 7 | MEDIUM | `memory/primitives/pb-dx4-baseline-triage.md` | **The ~33-item WATCH band is nowhere in the durable record**, including three defs the sub-agents themselves named as promotion candidates. **Fix:** add a WATCH section to the disposition doc. |
| 8 | LOW | `pb-dx4-baseline-triage.md:161` | 123 vs `deck.rs`'s and audit §8.1's 122 colourless singletons. **Fix:** pick one and say which population it counts. |
| 9 | LOW | `memory/primitive-wip.md` | Still describes PB-DX3b. **Fix:** rewrite for PB-DX4 (or note the batch ran without one). |
| 10 | LOW | `crates/card-defs/src/defs/shambling_ghast.rs:65` | Mode indices are inverted vs the printed bullet order. **Fix:** either reorder, or extend the existing note to say that a client rendering `modes` in order shows them backwards. |
| 11 | LOW | `crates/engine/tests/primitives/pb_dx4_baseline_triage.rs:548` | The `MAX_UNDECLARED` comment's denominator is not the number the test measures. **Fix:** print/record the measured `total`. |
| 12 | LOW | `tools/play-server/src/api.rs:485` | The `#[cfg(test)]` sentinel returns *before* `session::new_game`, so the two atomicity tests prove a slightly weaker statement than their names say. **Fix:** one sentence in the injection comment. |
| 13 | LOW | (process) | No `memory/primitives/pb-plan-DX4.md` exists. **Fix:** note the deviation in the WIP file. |

---

### Finding Details

#### Finding 1 (HIGH): Risen Reef — a free "you may" left mandatory on a `Complete` def

**File**: `crates/card-defs/src/defs/risen_reef.rs:36-47`
**Oracle (MCP, verified this review)**: "Whenever this creature or another Elemental you
control enters, **look at** the top card of your library. If it's a land card, **you may** put
it onto the battlefield tapped. If you don't put the card onto the battlefield, put it into
your hand."

**Issue.** The def authors this as `Effect::RevealAndRoute { filter: Land, matched_dest:
Battlefield{tapped:true}, unmatched_dest: Hand }`. `RevealAndRoute` has no optional flag and
moves every match unconditionally (`effects/mod.rs:5745-5757`), so a land is **always** put
onto the battlefield and the printed "put it into your hand" branch is unreachable for a land.
The controller is forced into a land drop they may not want — the choice is real (holding the
land in hand for a later "put a land from your hand" effect, for a discard outlet, for hand
size, or to deny an opponent's landfall watcher) and the printed card gives it to them.

This is not a new class. It is *exactly* the class this batch demoted two cards for, in the
same seven-day sitting:

* `contaminant_grafter.rs` — "then you **may** put a land card from your hand onto the
  battlefield", authored unconditionally → demoted to `partial`. Batch2's own words
  (`pb-dx4-triage-batch2.md:57`): "**the printed optionality on the land-put is dropped**,
  which makes the def do something the card cannot make you do."
* `smugglers_copter.rs` — "you **may** draw a card" → demoted to `known_wrong`.

Batch6's contrary ruling (`pb-dx4-triage-batch6.md:105-107`) is: "The printed 'you may put it
onto the battlefield tapped / if you don't, put it into your hand' is a genuine binary player
choice that the engine resolves by always taking the battlefield branch — that is the
auto-choice the `look_at_top_or_route` row records." That reading is internally coherent, but
it applies verbatim to Contaminant Grafter (both branches legal, engine picks one) and to
Smuggler's Copter (ditto, modulo CR 704.5b). **Two sub-agents answered the same question
differently and the disposition document reports only the sum.** The published "86 B / 11 D"
therefore is not a measurement of the corpus; it is a measurement of which agent read which
card.

Batch6's own WATCH 2 records the consequence plainly — "the auto-choice is hardwired to the
battlefield branch, so 'put it into your hand instead' is unreachable for a land" — and then
classifies B anyway.

**Why this is HIGH and not a bookkeeping MEDIUM.** `risen_reef` is `Complete` by the
`#[default]` derive (no marker declared), therefore deck-legal, therefore playable in a game
whose replay history Architecture Invariant 9 requires to be faithful. It is a widely played
Commander card. The batch was chartered precisely to find deck-legal `Complete` defs that do
something the printed card cannot force, and it found this one, wrote the observation down, and
left it.

**Fix.** Preferred: re-author in place with the primitive this very batch introduced for the
two put-≤1 reveals —

```
Effect::LookAtTopThenPlace {
    player: PlayerTarget::Controller,
    count: EffectAmount::Fixed(1),
    filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() },
    place_cost: None,
    destination: ZoneTarget::Battlefield { tapped: true },
    rest_to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
    optional: true,
}
```

I verified this is behaviourally identical today: the `LookAtTopThenPlace` arm
(`effects/mod.rs:5885-5903`) and the `RevealAndRoute` matched arm (`:5745-5757`) use the same
`resolve_zone_target` + `dest_tapped` + `expect_move_object_to_zone` + `zone_move_event`
sequence, and with `count: 1` the ≤1 cardinality is not a restriction. It additionally matches
the printed "look at" (not "reveal") and encodes the "may" on the wire for the future
interactive path. Note this makes `risen_reef` subject to Finding 5 as well (the flag is inert),
so it must carry the same in-def note.
Alternative, if the runner prefers the strict reading: demote to `partial` with
`contaminant_grafter`'s note. **Either way, the triage doc's split and the `decision_gate.rs`
doc block must be corrected**, and the doc must record which of the two standards is the
batch's, because right now it holds both.

---

#### Finding 2 (HIGH): Hullbreaker Horror — the same flat-mode-target defect Shambling Ghast was demoted for, left `Complete`

**File**: `crates/card-defs/src/defs/hullbreaker_horror.rs:46-57`
**Oracle (MCP, verified)**: "Whenever you cast a spell, choose **up to one** — • Return target
spell you don't control to its owner's hand. • Return target nonland permanent to its owner's
hand."
**CR 603.3d** — a triggered ability whose targets cannot all be chosen legally is removed from
the stack. Confirmed in-engine at `rules/abilities.rs:8294` ("If any required target has no
legal candidate, skip this trigger") and in `rules/events.rs:1450-1453` ("a slot with no legal
candidate removes the trigger instead").

**Issue.** The def declares **both** mode targets flat, with `mode_targets: None`:

* slot 0 — `TargetSpellWithFilter { controller: Opponent }`
* slot 1 — `TargetPermanentWithFilter { non_land: true }`

Both are unconditionally required. So the trigger fires only when *simultaneously* an
opponent-controlled spell is on the stack **and** a nonland permanent exists. The ordinary
case — you cast a spell on your own turn with nothing of an opponent's on the stack — has no
legal candidate for slot 0, so the trigger is skipped entirely. A 7-mana `Complete` creature's
entire textbox is silently inert in the common case, and "choose **up to one**" (`min_modes: 0`,
i.e. choosing zero modes is a legal announcement under CR 601.2c) is unreachable.

This is strictly worse than the `shambling_ghast` defect the batch **did** demote for (there,
one mode's target blocks the other mode; here, one mode's target blocks the trigger's very
existence in the normal case). The batch knew: `pb-dx4-triage-batch4.md:248` lists it as a
WATCH ("flat `targets` with `mode_targets: None` may make mode 1 unreachable"), and the batch's
own new seed **OOS-DX4-2** names it — "`hullbreaker_horror` was independently flagged with the
same shape during the triage (batch 4) and is a second member." Naming a defect in a seed is
not a disposition; the def is still `Complete` and still deck-legal, and the seed is an engine
change nobody has scheduled.

**Fix.** Demote `hullbreaker_horror` to `Completeness::partial(...)` with a note in the shape
of `shambling_ghast.rs:122-130` — flat mode targets, `mode_targets` honoured only on the
casting path, CR 603.3d removal, OOS-DX4-2 owns the closure. Then update the triage doc
(86/11 → 85/12 or whatever the runner's final call makes it), `MAX_AUTO_CHOSEN_COMPLETE_UNION`
(92 → 91, read off `T6`, not computed — the same discipline the batch already used), the
`completeness_deviation_scan` floor (666 → 667), and `docs/authoring-status.md`. Note that
`hullbreaker_horror` sits in the `modal_trigger` BASELINE row, not `triggered_targets`, so the
DP8 roster pin (75) does **not** move — verify by running, don't take this from me.

---

#### Finding 3 (MEDIUM): the freeze constant was not closed, so five unjustified-entry slots opened

**File**: `crates/engine/tests/core/decision_gate.rs:759-772`

The comment immediately above the constant states the contract: *"The freeze is closed at
exactly its measured size, so every LATER entry must carry `Some(reason)`. This deliberately
does not grow: shrinking is fine (a def demoted or fixed just leaves), growing is not."* The
assertion is `unexplained <= FROZEN_2026_07_27` with `FROZEN_2026_07_27 = 97`.

PB-DX4 removed five entries. `BASELINE` now holds 92 (hand-counted, entry by entry, lines
384-483), all with `None`. So `unexplained == 92` and the gate now tolerates **five** new
unreasoned entries before it reddens — precisely the "not silent but unjustified" state the
comment above it says it exists to prevent. The batch lowered `MAX_AUTO_CHOSEN_COMPLETE_UNION`
97 → 92 and the two roster pins, and did not lower this one.

This is the batch's own subject class (a claim wearing a gate's authority) reproduced in the
gate the batch was editing.

**Fix.** `const FROZEN_2026_07_27: usize = 92;` with a dated derivation comment — "97 at the
2026-07-27 freeze; 5 members demoted by PB-DX4's OOS-DP10-8 triage and removed, so 92 of the
freeze survive; this constant tracks *surviving* freeze members, not the historical freeze
size." (If Finding 2 is applied, 91.)

---

#### Finding 4 (MEDIUM): "all eleven class-D defs never declared a marker" is false — two of them did

**Files**: `memory/primitives/pb-dx4-baseline-triage.md:215`;
`crates/engine/tests/primitives/pb_dx4_baseline_triage.rs:511` and `:547-549`;
`memory/workstream-state.md:18`; `docs/audits/decision-point-audit.md` §8.1 row OOS-DP10-8 (via
the workstream-state duplicate).

The claim, in three phrasings: *"**All eleven** of this batch's class-D defs were in that
group"*; *"Every one of this batch's eleven class-D defs was in that group"*; *"(This batch's
own five demotions each ADDED a marker, so the count fell by five from the 975 that would
otherwise stand.)"*

All three are wrong, and the batch's own working notes say so.
`pb-dx4-triage-batch6.md:374-376`: *"The four explicit-marker defs are Raffine's Informant,
Roalesk, **Shambling Ghast, Smuggler's Copter** — note that two of *those* are class D too, so
an explicit marker is no protection."*

I verified this structurally rather than taking batch6's word: `smugglers_copter.rs` (lines
6-106) and `shambling_ghast.rs` (lines 21-131) are both **exhaustive struct literals with no
`..Default::default()`**, so the `completeness` field is syntactically mandatory in both and
was present before this batch. Only three of the five demotions (`contaminant_grafter`,
`grateful_apparition`, `thrasios_triton_hero`, all of which do carry `..Default::default()`)
added a new mention.

Consequences: the correct figure is **9 of 11**, the pre-batch undeclared count was **973** not
975, and the demotions lowered it by **three** not five. `MAX_UNDECLARED = 970` itself is
unaffected (it is a measured value); only the derivations are wrong.

This matters beyond arithmetic. The batch's stated conclusion — the answer to the corpus-wide
`#[default]` question PB-DX3b handed forward — is that an unmarked def is where the defects
live. Batch6's own counter-evidence ("an explicit marker is no protection") points the opposite
way for two of the eleven and was dropped on the way from the working notes to the durable
record.

**Fix.** Correct all four sites to "9 of 11", correct the 975 → 973 / "by five" → "by three"
parenthetical, and add batch6's counter-observation to the triage doc's `#[default]` section:
two of the eleven carried explicit markers and were wrong anyway.

---

#### Finding 5 (MEDIUM): `optional: true` is inert, so the repaired "may" is still not implemented — and the test says it is

**Files**: `crates/card-defs/src/defs/grisly_salvage.rs:39`,
`crates/card-defs/src/defs/satyr_wayfinder.rs:44`;
`crates/engine/tests/primitives/pb_dx4_baseline_triage.rs:367-371`.
**Engine**: `crates/engine/src/effects/mod.rs:5797-5802` — the arm destructures `optional: _`
and its own comment says *"`optional` is not read by this M7 deterministic executor … Currently
inert, not a live gate."* This is filed as **OOS-DP10-5**.

The repair is genuine and I endorse it: `RevealAndRoute` moved *every* creature-or-land card
from the top five into hand, so Grisly Salvage was a two-mana "draw 3-5". `LookAtTopThenPlace`
fixes the cardinality, which is the larger half of the defect.

What it does **not** fix is the "you **may**". The engine still always places the best
candidate. Both defs stay `Complete` with no in-def acknowledgement of that. The disposition
doc is aware — `pb-dx4-baseline-triage.md:199-201` lists *"Grisly Salvage's `may` half"*
alongside Smuggler's Copter and Contaminant Grafter as "exactly that class" — and does not
reconcile why two members of the class were demoted and this one was not.

There **is** a defensible principle available (an encoded-but-inert flag is a strictly better
state than no encoding at all, and `growing_rites_of_itlimoc.rs:78,141` ships exactly this
shape as explicitly `Complete` from PB-OS8), but the batch does not state it, and PB-DX3
established the standard for precisely this situation: when it found `Effect::SearchLibrary`'s
`reveal: true` inert, it added an in-def comment *"because a `Complete` marker should not
silently cover an unimplemented printed clause."*

Worse, the new test asserts the opposite. `pb_dx4_baseline_triage.rs:367-371`:

```
assert!(rendered.contains("optional: true"),
    "{name}: the printed clause is 'you MAY put', so `optional` must be true");
```

and the module doc calls `LookAtTopThenPlace` the primitive that *"carries the `optional` flag
the printed 'you may' needs."* A reader takes that as "the may is handled". It is not; the flag
is discarded one line into the executor.

**Fix.** (a) Add to both defs the PB-DX3-style note: `optional: true` is currently inert
(`effects/mod.rs` destructures `optional: _`, OOS-DP10-5), so the printed "you may" is not yet
a real decline; the marker stays `Complete` on the `growing_rites_of_itlimoc` precedent, and
this comment is why it is not silent. (b) Reword the T4 assertion message to
"…so `optional` must be true — the wire encoding is correct even though the M7 executor
currently ignores it (OOS-DP10-5)". (c) In the triage doc, state the principle explicitly, since
it is what separates these two defs from `contaminant_grafter`.

---

#### Finding 6 (MEDIUM): Sword of Truth and Justice — the printed clause does not target, and the repair left that unrecorded and unseeded

**File**: `crates/card-defs/src/defs/sword_of_truth_and_justice.rs:70-75`
**Oracle (MCP, verified)**: "Whenever equipped creature deals combat damage to a player, put a
+1/+1 counter on **a creature you control**, then proliferate."
**CR 115.10** — an object is a target only if the spell/ability says "target".

The batch's repair (bare `TargetRequirement::TargetCreature` →
`TargetCreatureWithFilter{controller: You}`) is correct and necessary: the engine's auto-target
could otherwise put the counter on an opponent's creature. But the printed clause has **no
"target"**, and the def keeps a `TargetRequirement`, so two live deviations remain on a
`Complete` def:

1. A creature you control with hexproof or shroud, or with protection from artifacts, cannot
   receive the counter — the printed card lets you choose it freely.
2. If the chosen creature leaves the battlefield in response, CR 608.2b removes the whole
   ability from the stack, so the **proliferate** does not happen either. The printed card
   proliferates regardless.

I checked whether the DSL offers an untargeted chooser: `EffectTarget`
(`card_definition.rs:2563-2604`) has no "a creature you control, chosen at resolution" variant,
so the targeted encoding is the only expression available — the same situation as
`staff_of_compleation`'s owner-vs-controller. The difference in treatment is the problem: the
owner-vs-controller class got an in-def note, an allowlist entry and a corpus-wide seed
(OOS-DX4-1); this class got a silent partial repair.

The batch found the class independently: `pb-dx4-triage-batch3.md:246-247` flags
*"Frantic Search — untargeted 'up to three lands' authored as real targets, so the spell can
fizzle where the printed card cannot, **a systemic DSL approximation**."* Two known members,
found by accident, no seed.

**Fix.** (a) Add an in-def note to `sword_of_truth_and_justice.rs` recording the residual
untargeted-vs-targeted deviation and its two consequences (and correct the existing "CR 601.2c"
citation on that `targets` block — CR 601.2c is target announcement, and this clause announces
none). (b) File a new seed in `docs/audits/decision-point-audit.md` §8.1 (OOS-DX4-5) for the
class, naming `sword_of_truth_and_justice` and `frantic_search`, in the shape of OOS-DX4-1:
enumerate the `Complete` defs that model a resolution-time choice as a target, then decide the
class at once.

---

#### Finding 7 (MEDIUM): the WATCH band is not in the durable record

**File**: `memory/primitives/pb-dx4-baseline-triage.md`

Acceptance criterion 1 asks that the split be "recorded durably". The disposition doc records
86 B and 11 D and nothing between. The seven working-note files carry roughly 33 WATCH entries,
and that band is where the next batch's work is. Three of them are defs a sub-agent explicitly
nominated for promotion to D:

* `geier_reach_sanitarium` — batch3 calls it *"the strongest promotion candidate in this
  batch"*. I verified: the def (`geier_reach_sanitarium.rs:35-47`) is
  `ForEach(EachPlayer, Sequence[Draw, Discard])`, i.e. interleaved per player, while the
  official ruling (2016-07-13, MCP) is *"first each player draws a card. Then … each … sets it
  aside … Then the cards that were set aside are discarded at once."* All-draw-then-simultaneous-
  discard. A real deviation in trigger ordering and in what each player knows when choosing.
* `hullbreaker_horror` — Finding 2 above.
* `frantic_search` — Finding 6's second member.

Plus `felidar_retreat` (the CR 611.2c affected-set class, which is the next batch, PB-DX5) and a
templating-drift group the batch dispositioned by policy (correctly, in my view — `oracle_text`
using an older but genuine printed wording is not a game-state defect).

**Fix.** Add a "WATCH — read, judged B, but borderline" section to
`pb-dx4-baseline-triage.md` listing each WATCH entry with its def, one-line reason and the
per-batch file it came from. The value of the exercise is mostly in this band; leaving it in
seven scratch files is how it becomes a re-read next year.

---

#### Finding 8 (LOW): 123 vs 122 colourless singletons

`pb-dx4-baseline-triage.md:162` — *"measured 40 colourless nonbasic lands + 83 colourless
nonlands = 123 singletons"*. `crates/simulator/src/deck.rs:123-124` and
`docs/audits/decision-point-audit.md` §8.1 (OOS-M11-6, OOS-DX4-4) both say **82 / 122**. Two of
three durable records agree; the triage doc, under a heading that reads "Numbers, each
re-measured rather than derived", is the outlier. Most likely reconciliation: 83 is the total
colourless `Complete` nonland population and 82 is that population minus the commander, which
`random_deck`'s `eligible` closure excludes (`deck.rs:61-63`) — in which case 122 is the number
that matters and 123 is not "available". Either way the margin over 99 is comfortable and no
behaviour depends on it. **Fix:** make the three records agree and say which population is
counted.

#### Finding 9 (LOW): `memory/primitive-wip.md` still describes PB-DX3b

The pipeline state file was never taken by this batch. `/implement-primitive`'s reviewer step —
and this review's own bootstrap — reads it to learn which batch is under review, and it names
PB-DX3b with a completed step checklist. **Fix:** rewrite for PB-DX4 (or, if the batch
deliberately ran without the WIP file, say so there).

#### Finding 10 (LOW): Shambling Ghast's mode indices are inverted vs the printed bullets

`shambling_ghast.rs:65-86` — index 0 is the Treasure token, index 1 is the -1/-1; the printed
card lists them the other way. The in-def defence (lines 16-17) — *"a mode's identity in Magic
is its text, not its index, so index order is an engine artifact"* — is right about the rules
and incomplete about this engine: `modes_chosen` is a wire-level `Vec<usize>` and any client
that renders `ModeSelection.modes` in order shows the card's bullets backwards. Harmless today
(the engine auto-picks mode 0 for triggered abilities, which is why Treasure was put first).
**Fix:** either reorder and re-point the bot-fallback comment, or extend the existing note to
cover the client-rendering consequence.

#### Finding 11 (LOW): the `MAX_UNDECLARED` comment's denominator is not what the test measures

`pb_dx4_baseline_triage.rs:547-549` says "970 of 1,804 def files". The test counts `.rs` files
in `crates/card-defs/src/defs` excluding `mod.rs`; 1,804 is `all_cards()`'s **definition**
count. I tried to reconstruct 970 independently: 171 files contain
`completeness: Completeness::Complete` (grep), and 666 defs are non-Complete (the
`completeness_deviation_scan` floor this batch just re-measured), each of which must declare the
field — 837 declaring files, giving 967, three short of 834/970. The likeliest explanation is
files declaring more than one card, which would double-count in my reconstruction; the pin is a
`<=` ceiling so nothing breaks either way. **Fix:** have the test print `total` in the failure
message (it already computes it) and record the measured `total` in the comment rather than
"1,804", so the next reader can reproduce the subtraction.

#### Finding 12 (LOW): the play-server sentinel proves a slightly weaker statement than its tests' names

`tools/play-server/src/api.rs:485-492`. The injection `return Err(...)` sits **above**
`let mut play = session::new_game(cfg, seq_base)?;`, so
`test_poison_recovery_is_atomic_when_the_rebuild_fails` and
`test_a_failed_rebuild_leaves_a_running_game_untouched` now demonstrate "an error return taken
after the poison recovery and before `*guard = Some(play)` leaves the session untouched" rather
than "a `session::new_game` failure does". For the property under test — that the `?` does not
leave a half-recovered session — the two are equivalent, and the injection's own comment is
otherwise unusually good (it records the `AtomicBool` race that was tried and rejected, which is
exactly the right thing to write down). **Fix:** one sentence saying which of the two statements
the tests now prove.

#### Finding 13 (LOW): no plan file

There is no `memory/primitives/pb-plan-DX4.md`; the batch worked from the queue row and
PB-DP10's plan §5.3 definitions. Given the batch's shape (read-and-classify, not design) that is
defensible, but the pipeline's review step expects one and its absence is not recorded anywhere.
**Fix:** note the deviation in the WIP file.

---

## Dispositions I re-derived and found correct

Every one of these was checked against MCP printed text in this session, not taken from the
batch's notes.

| def | disposition | verified |
|---|---|---|
| Metastatic Evangel | repaired, `Complete` | MCP `{1}{W}` / `Creature — Phyrexian Human Cleric` / 3/1 / "another **nontoken** creature you control enters" — all four now match. The `is_nontoken` claim is real, not aspirational: `rules/abilities.rs:6957-6966` checks `creature_filter.is_nontoken` against the entering object on the `triggering_creature_filter` path. |
| Radstorm | repaired, `Complete` | MCP `{3}{U}`. Storm + Proliferate both present. |
| Grisly Salvage | repaired, `Complete` | Arity fix correct (`RevealAndRoute` moves all matches, `effects/mod.rs:5747`). See Finding 5 for the residual. |
| Satyr Wayfinder | repaired, `Complete` | As above. |
| Sword of Truth and Justice | repaired, `Complete` | Controller axis now right. See Finding 6 for the residual. |
| Shambling Ghast | 3 fixed, → `partial` | MCP keywords `["Treasure"]` only — the `Decayed` grant was a phantom. "-1/-1 **until end of turn**" now `ApplyContinuousEffect` + `UntilEndOfTurn` (the `drown_in_ichor` idiom). `oracle_text` corrected. The fourth defect is real: CR 603.3d removal confirmed at `abilities.rs:8294`. |
| Smuggler's Copter | → `known_wrong` | MCP "you **may** draw a card. If you do, discard a card"; def is an unconditional `Sequence` on both triggers. DP-12 class, 19 prior members already `known_wrong`. |
| Contaminant Grafter | → `partial` | MCP "then you **may** put a land card from your hand onto the battlefield". Trigger, `Toxic(1)`, Trample and the CR 603.4 `OpponentHasPoisonCounters(3)` intervening-if all correct; only optionality lost. |
| Grateful Apparition | → `partial` | MCP "deals combat damage to a player **or planeswalker**". I confirmed `TriggerCondition` has `WhenDealsCombatDamageToPlayer` and an *equipped-creature* any-recipient variant (`card_definition.rs:3502`) but no self any-recipient variant. |
| Thrasios, Triton Hero | → `partial` | MCP "Otherwise, **draw a card**"; def routes to `ZoneTarget::Hand`. Confirmed no `Condition` inspects a revealed card's type (`TopCardIsCreatureOfChosenType` / `TopCardIsInstantOrSorcery` are the only two, neither is "is a land"). Good cross-check: `coiling_oracle` has the identical structure but printed "put that card into your **hand**", and is correctly class-B. |
| Staff of Compleation | `Complete`, allowlisted | MCP "Destroy target permanent **you own**". `TargetFilter` has no owner axis. `nether_traitor`'s shipped allowlist entry (`completeness_deviation_scan.rs:132-138`) is the same approximation on an explicitly-`Complete` def, reviewed in `scutemob-95`, and names `athreos` / `fecundity` as further members. **The differential treatment is defensible**: the batch followed the *shipped precedent for each class* — DP-12's 19 `known_wrong` members for the free-"may" class, `nether_traitor`'s allowlist for the ownership class. That is a coherent rule, and it is the rule Findings 1 and 5 ask the batch to apply consistently rather than abandon. |

**Class-B sample (12 defs), all confirmed correct**: Coiling Oracle, Sylvan Messenger, Goblin
Ringleader, Chaos Warp (`Battle` missing from its permanent-card type list is the deferred
Battle subsystem, not this batch's), Growing Rites of Itlimoc, Yawgmoth (incl. the `UpToN`
"up to one target" and the `exclude_self` sacrifice cost), Victimize (incl. the
`Condition::SacrificeFired` gate matching the 2020-11-10 ruling), Accursed Marauder, Sword of
Feast and Famine, Felidar Retreat / Cached Defenses (oracle only), and — **not** confirmed —
Risen Reef (Finding 1), Hullbreaker Horror (Finding 2), Geier Reach Sanitarium (Finding 7).

---

## Test Review

All eight probes in `pb_dx4_baseline_triage.rs` are non-vacuous. Verified by reading, and by
naming the mutation that would redden each:

| test | reddens if | non-vacuous? |
|---|---|---|
| `shambling_ghast_minus_one_minus_one_wears_off_at_end_of_turn` | mode 1 reverts to `AddCounter{MinusOneMinusOne}` — the second assertion (post-cleanup P/T) fails, the first does not | **Yes, and this is the strongest test in the file.** It takes the effect from the shipped def (`run_ghast_mode_1` reads `modes.modes[1]` out of `card_def("Shambling Ghast")`), so a regression is *executed*, not asserted against a copy. It also guards its own fixture (`base_t > 1`, `turn_number == 2`). |
| `shambling_ghast_does_not_have_the_decayed_keyword` | the `KeywordAbility::Decayed` arm returns, or `oracle_text` regains "decayed" / "enters" | Yes |
| `metastatic_evangel_matches_its_printed_card` | any of the four defects returns; the `is_nontoken` half asserts on the `Debug` rendering of `trigger_condition` | Yes (the `Debug`-string assertion is brittle but does fire) |
| `put_at_most_one_reveals_use_the_put_one_primitive` | either def reverts to `RevealAndRoute` | Yes — and its own doc argues correctly why it asserts the primitive rather than driving the reveal (a one-matching-card fixture passes under both). See Finding 5 for the third assertion's message. |
| `sword_of_truth_and_justice_targets_only_your_creature` | reverts to bare `TargetCreature` → falls to the `other => panic!` arm | Yes |
| `radstorm_costs_three_generic_and_one_blue` | cost reverts | Yes |
| `class_d_defs_without_a_dsl_expression_are_demoted` | any of the 5 markers reverts, **or** any of the 5 repaired defs is demoted | Yes — and the second loop is a genuinely good addition: without it, demoting all eleven would pass and look like success. |
| `defs_that_never_declare_a_completeness_marker_are_ratcheted` | the unmarked population grows past 970 | Yes, though see Finding 11 on the denominator, and note it is a ceiling, so it does not redden if a batch adds markers. The `total >= 1_700` denominator guard is the right instinct. |

**On the module doc's honesty claim** (lines 20-48), which the brief asked me to judge
specifically: it is honest, and unusually so. It names exactly one test as having an executed
pre-fix observation, describes the mutation (`AddCounter{MinusOneMinusOne, 1}`), reports both
numbers in both directions, and then *declines* to claim pre-fix drives for the other seven —
including a paragraph explaining why manufacturing one for
`put_at_most_one_reveals_use_the_put_one_primitive` would have meant choosing a fixture to make
the claim true. That is exactly the standard PB-DX3's MEDIUM established, applied without being
asked twice. The executed claim is also *plausible on the mechanics*: a permanent -1/-1 counter
survives cleanup, so a 3/3 reads 2/2 on both sides of the turn boundary, and the restored
`UntilEndOfTurn` version reads 2/2 then 3/3 (CR 514.2). I could not re-execute it, but nothing
about it is off.

**Pre-existing test reconciliation**: `crates/engine/tests/rules/modal_triggers.rs` was touched
doc-only (the Shambling Ghast structural test still asserts 2 modes / `min_modes` 1 /
`max_modes` 1 and does not depend on which mode is which) — no assertion was weakened to
accommodate the fix, which I specifically looked for.

**Golden script**: `baseline/112` is retired, not deleted, with `review_status: "retired"`, a
non-empty `retirement_reason`, and an appended dispute carrying MCP-verified printed text. It
therefore satisfies `retired_scripts_carry_a_reason` and `the_corpus_is_fully_accounted_for`
(`run_all_scripts.rs:197-272`), neither of which pins a count. The retirement reasoning is the
best writing in the batch: it checks that CR 702.147a coverage survives (12 unit tests in
`mechanics_a_d/decayed.rs`), says what is genuinely lost (golden-level Decayed coverage), files
it as OOS-DX4-3 rather than leaving it silent, and names the provenance failure — the script
cited the *card definition* as its authority for the printed text, so a phantom keyword in a def
became evidence for itself.

---

## Gates

| gate | status | how I checked |
|---|---|---|
| `git diff main -- crates/engine/src crates/card-types/src` empty | **not executed** (no Bash) — no contrary evidence | A repo-wide grep for `PB-DX4` returns no file under `crates/engine/src` or `crates/card-types/src`; all engine-side hits are under `crates/engine/tests/`. Weak but consistent. |
| `PROTOCOL_VERSION = 32` | **verified** | `crates/engine/src/rules/protocol.rs:335` |
| `HASH_SCHEMA_VERSION = 69` | **verified** | `crates/engine/src/state/hash.rs:679` |
| `MAX_AUTO_CHOSEN_COMPLETE_UNION = 92` matches `BASELINE.len()` | **verified by hand count** | 92 entries, lines 384-483 |
| deviation floor 661 → 666 | **arithmetic verified** | 5 demotions, 0 promotions; 1804 − 1138 = 666; `docs/authoring-status.md:28` independently reports 1,138 / 63.1% / Δ −5 |
| DP8 roster 76 → 75 | **derivation verified** | Of the five demoted defs, only `shambling_ghast` carries a non-empty `targets` on an `AbilityDefinition::Triggered`; I read all four others (`smugglers_copter` `targets: vec![]` ×2, `contaminant_grafter` `vec![]` ×2, `grateful_apparition` `vec![]`, `thrasios` is `Activated`). |
| `scry` 16 → 15 | **derivation verified** | Only `thrasios_triton_hero` among the five carries `Effect::Scry`. |
| `cargo test --workspace` / clippy / `fmt` / `check-defs-fmt.sh` | **not executed** (no Bash) | Taken on the batch's word. The runner should re-run before merge; Findings 2 and 3 will move test counts and pins if applied. |
| SR-9a `mod` registration | **verified** | `crates/engine/tests/primitives/main.rs:34` |
| SR-9c script triage | **verified** | retired with reason + dispute; gates read at `run_all_scripts.rs:197-272` |
| close-out bookkeeping | **partially verified** | `CLAUDE.md` (3 hits), `memory/workstream-state.md`, `memory/primitives/seed-rerank-2026-07-27.md`, `docs/authoring-status.md` and `docs/audits/decision-point-audit.md` §8.1 (OOS-DX4-1..4 filed, OOS-DP10-8 + OOS-M11-6 marked closed) all updated. **`memory/primitive-wip.md` was not** (Finding 9). |

---

## Out-of-scope changes: `crates/simulator/src/deck.rs` and `tools/play-server`

The brief asked me to judge necessity and correctness. Both hold.

**Necessary.** Demoting `thrasios_triton_hero` — a Legendary Creature, hence a member of
`random_deck`'s commander pool — shifts every `rng.random_range(0..commanders.len())` draw
downstream of it, so every seeded deck in the workspace re-deals. That is not avoidable by
scoping; any completeness flip on a legendary creature does it. The seeds then landed on a
colourless commander and hit the pre-existing OOS-M11-6 bug, which `validate_deck` refuses. The
alternatives were to re-pin the play-server seed (leaving a live CR 903.5c bug in the fuzzer,
which was *playing* illegal decks, not refusing them) or to fix the bug. Fixing it was right.
It also stays outside the batch's declared zero-engine gate, which is scoped to
`crates/engine/src` + `crates/card-types/src`.

**Correct.** I read the new padding arm (`deck.rs:128-142`). It pads from `eligible` — already
colour-identity-filtered and `Complete`-filtered, already excluding the commander — skipping
cards already in `main_deck`, so CR 903.5b singleton is respected; it returns `None` rather
than an illegal deck if the pool cannot reach 99; and both Forest fallbacks are gone, with
`basics_for_colors` now returning an empty vec that *means* something. Headroom: 65 cards from
the nonland/nonbasic-land passes plus ~57 filler ≥ 99. The only defect I found is the 122/123
inconsistency (Finding 8).

The `#[cfg(test)]` sentinel in a production handler is a smell, but the injection comment
argues its way to the right design (per-request rather than process-global, after the
`AtomicBool` version raced under `--workspace` — twice), and the maintenance note in the tests
it serves had *predicted this exact need* before the bug was closed. Finding 12 is the only
adjustment I'd make.

---

## CR Coverage Check

| CR rule | Implemented / respected? | Tested? | Notes |
|---|---|---|---|
| 613.1e / 514.2 (until end of turn) | Yes — Shambling Ghast mode 1 | Yes | `shambling_ghast_minus_one_minus_one_wears_off_at_end_of_turn`, drives past a real turn boundary |
| 702.147a (Decayed) | Phantom removed | Yes | `shambling_ghast_does_not_have_the_decayed_keyword`; 12 unit tests survive in `mechanics_a_d/decayed.rs`; golden coverage gone → OOS-DX4-3 |
| 603.3d (trigger targets) | Correctly diagnosed | Indirectly | Shambling Ghast demoted for it; **Hullbreaker Horror not** — Finding 2 |
| 601.2c ("up to one" = zero is legal) | Not honoured for `hullbreaker_horror` | No | Finding 2 |
| 608.2b (all targets illegal → no resolution) | Residual on `sword_of_truth_and_justice` | No | Finding 6 |
| 115.10 (target vs choose) | Not addressed | No | Finding 6 |
| 118.12 (costless "you may") | Diagnosed for 3 of 4 members | Partly | Findings 1 and 5 |
| 108.3 / 109.4 (own vs control) | Deliberate approximation, allowlisted + seeded | Yes (allowlist liveness test) | OOS-DX4-1 |
| 702.40a (Storm) | Radstorm cost corrected | Yes | `radstorm_costs_three_generic_and_one_blue` |
| 903.5b / 903.5c (colour identity, singleton) | Yes — `deck.rs` padding arm | Existing simulator fixtures | OOS-M11-6 closed |
| 704.5b (deck-out) | Cited in Smuggler's Copter demotion | n/a | correct |

---

## Card Def Summary

| Card | Oracle match after batch | Markers correct | Game state correct | Notes |
|---|---|---|---|---|
| Metastatic Evangel | Yes | Yes (`Complete`) | Yes | 4 defects fixed; `is_nontoken` genuinely honoured |
| Radstorm | Yes | Yes | Yes | — |
| Grisly Salvage | Arity yes, "may" no | Defensible | Mostly | Finding 5 |
| Satyr Wayfinder | Arity yes, "may" no | Defensible | Mostly | Finding 5 |
| Sword of Truth and Justice | Controller yes, targeting no | Defensible | Mostly | Finding 6 |
| Shambling Ghast | Yes (3 fixed) | Yes (`partial`) | 4th defect marked | mode order LOW 10 |
| Smuggler's Copter | n/a | Yes (`known_wrong`) | No, and marked | — |
| Contaminant Grafter | n/a | Yes (`partial`) | No, and marked | — |
| Grateful Apparition | n/a | Yes (`partial`) | No, and marked | — |
| Thrasios, Triton Hero | n/a | Yes (`partial`) | No, and marked | — |
| Staff of Compleation | Approximation | Yes (allowlisted) | Latent, control-change only | precedent-consistent |
| **Risen Reef** | **No** | **No — `Complete`** | **No** | **Finding 1** |
| **Hullbreaker Horror** | **No** | **No — `Complete`** | **No** | **Finding 2** |
| Geier Reach Sanitarium | No (ruling 2016-07-13) | `Complete` | Ordering deviation | Finding 7 |

---

## Previous Findings

First review of this batch. No prior findings table.
