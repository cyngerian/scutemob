# PB-DX45 — `Effect::MayPayThenEffect` is pay-when-able (CR 118.12)

**Task**: `scutemob-217` · v4 queue rank 4 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 4)
**Seeds**: `OOS-DX24-9` ≡ `OOS-DX27-5` — the same defect filed twice, five days apart, neither row
citing the other (v4 memo §1d).

---

## §0 — Predictions, written BEFORE any code changed

> This section is written at the top of the batch and is **never edited afterwards**. Everything
> below §0 may correct it; §0 itself stands as the record of what was predicted.
> Baseline commit: `a9671666`. Pre-edit constants read from source: **PROTOCOL 38**
> (`protocol.rs:412`, fingerprint `50e69006…205a27`), **HASH 77** (`hash.rs:863`).

### §0.1 Wire prediction (AC 7244)

| axis | prediction | reasoning, traced not guessed |
|---|---|---|
| **PROTOCOL** | **38 → 39**, ONE bump for the whole PB | `EffectChoiceQuestion` is reachable from `GameEvent::EffectChoiceRequired` and `EffectChoiceAnswer` from `Command::AnswerEffectChoice`; both are therefore inside the `PROTOCOL_ROOTS` closure (`protocol_schema.rs:74`). Adding a variant to either changes the serialized shape of an in-closure type, so the fingerprint moves. |
| **PROTOCOL closure type count** | **98 → 98 (UNCHANGED)** | The new variants carry only `Cost` (already in-closure via `Effect::MayPayThenEffect { cost, .. }`) and `bool`. No new type enters the closure — unlike PB-DX28, whose `ChoiceZone`/`TargetOwner` were genuinely new members and moved 96 → 98. |
| **HASH** | **77 → 78**, ONE bump for the whole PB | `AnsweredEffectChoice { question, answer }` is reachable from `GameState.effect_choice_answers` and is folded into `public_state_hash`; `hash.rs` has a `HashInto` impl for both enums with one arm per variant. A new arm changes the hash schema. |
| **`hash_schema` / `protocol_schema` gate behaviour** | both go **RED first**, then are re-pinned from **their own output** | Never predicted numerically. The fingerprints below are transcribed from the failing gates, never invented (PB-DX8's "publish the figure, do not transcribe it" rule; PB-DX28's execution notes quoted two fingerprints that had never existed). |
| **`history_is_append_only`** | green after appending ONE row to each of `PROTOCOL_HISTORY` and `HASH_SCHEMA_HISTORY` | rows appended, never edited |
| **`frozen_prefix_is_pinned`** | RED until `FROZEN_HISTORY_PREFIX_DIGEST` is re-pinned in **both** gate files | version 38 / hash 77 join the frozen prefix when 39 / 78 ship |
| **sentinels** | re-pinned **by symbol**, not by hand-copied literal | the SR-27/SR-8 procedure |

**Stop condition, stated in advance**: if either gate moves in a way this table does not explain —
or does **not** move at all — stop and re-read rather than edit a pin (v4 memo's inherited
addendum; the PB-DP2/DP3 precedent where two predicted bumps were falsified).

### §0.2 Coverage / completeness flips predicted (AC 7242)

See §3 for the policy ruling and the named flip list. Written before regeneration.

### §0.3 Population prediction (AC 7243)

The memo's **11 deck-legal `Complete` defs** is treated as a **FLOOR** (dispatch hygiene 6). §2
records the inverse-method census at HEAD.

---

## §1 — The census, measured at HEAD before any engine line changed

Every figure below is **printed** by `crates/engine/tests/core/pb_dx45_may_pay_roster.rs::t_census_report`
and read off its output. None is transcribed from a prior document.

### §1.1 Forward axis — `Effect::MayPayThenEffect`

**14 corpus defs**, of which **10 are deck-legal `Complete`**:

> Crossway Troublemakers · Disciple of Freyalise · Hazoret's Monument · Kalastria Highborn ·
> Leaf-Crowned Visionary · Miara, Thorn of the Glade · Nadir Kraken · Nether Traitor ·
> Springbloom Druid · Tainted Observer

The four non-`Complete` carriers are `Ezuri, Stalker of Spheres`, `Mana Vault`,
`Ruthless Technomancer` and `Vampire Gourmand`.

**The v4 memo says ELEVEN, and it does not reproduce.** §1d of
`seed-rerank-2026-08-14.md` records *"Two independent measurements of this task both returned 11
deck-legal `Complete` defs for the same population, which is what proves they are one thing."*
This batch re-derived it at HEAD by two independent routes and got **10** both times: the
`all_cards()` walk above, and `decision_gate.rs`'s frozen `BASELINE`, which carries exactly ten
`may_pay_then_effect` entries. No member of this class has changed its `completeness` marker since
PB-DX27 (`3390b6a9`, 2026-08-13) — *before* the memo's census closed — so the corpus did not move
underneath it.

The memo's **conclusion** stands: `OOS-DX24-9` and `OOS-DX27-5` do name one defect, because they
name the same `Effect` variant and the same handler. What does not stand is the **evidence** it
offered, which was two agreeing wrong numbers. Six batches of this queue have learned that a
published member list is a FLOOR; this is the first time one has been an **over**-count, and it is
recorded rather than quietly corrected for that reason.

### §1.2 The site list was short by one, and the second site is live

`effects/mod.rs` has **two** callers of `try_pay_optional_cost`, not one:

| # | site | population | status before PB-DX45 |
|---|---|---|---|
| 1 | `Effect::MayPayThenEffect` (`:4692`) | 14 defs / 10 deck-legal `Complete` | unconditional |
| 2 | `Effect::LookAtTopThenPlace { place_cost: Some(..) }` (`:6365`) | **1 def, deck-legal `Complete`** (`birthing_ritual`) | unconditional |

Site 2 is the identical CR 118.12 decision one function over, on a `Complete` def, and no document
in the chain names it — the seed rows, the v4 memo row and the task brief all say
`MayPayThenEffect`. **It is taken, not deferred**: closing "CR 118.12's player decision is
engine-made" while a known deck-legal `Complete` member of the same helper keeps the old shape
would close it on a false premise (the PB-DX28 / `Connive // Concoct` precedent). Pinned by
`r4_second_pay_site_population_is_pinned`.

`LookAtTopThenPlace`'s own `optional` field — the *costless* "you **may** put …" half — is
**not** taken. It is inert today (its own in-source note says so) and belongs to the
`OOS-DP10-9` / DP-12 costless-"may" class. Filed as `OOS-DX45-4`.

### §1.3 Inverse axis — printed priced optional costs with no `MayPayThenEffect`

**27 `Complete` defs** whose printed oracle text (walked over **every** `oracle_text` in the
serialized def, not `def.oracle_text` alone — PB-DX8's blindness to transformed faces and
Adventure halves) carries `you may pay` / `you may sacrifice` / `you may discard` / `may pay `
while carrying no `MayPayThenEffect`. Sub-classified:

| bucket | n | disposition |
|---|---|---|
| cast-time optional costs — Squad 2, Buyback 2, Casualty 1, Kicker 1, Foretell 1, Plot 1, Assist 1, Pitch 1 | **10** | **out of class.** A cast-time optional cost is announced at CR 601.2b, and PB-DX29/PB-DX44 already built its pickers. |
| as-enters `ReplacementModification::EntersTappedUnlessPayLife` (CR 614.1c) — the ten shocklands + `sea_gate_restoration` | **11** | **FILED, not taken** (`OOS-DX45-5`). Same rule, different channel: a replacement effect, not a resolution, so it has no `resolve_top_of_stack` wrapper to roll back to. |
| keyword-carried optional costs — Extort ×2, Devour ×1, Recover ×1 | **4** | **FILED, not taken** (`OOS-DX45-6`). |
| `birthing_ritual` | **1** | covered by site 2 above. |
| **`teneb_the_harvester`** | **1** | **TAKEN.** See §1.4. |

### §1.4 `teneb_the_harvester` — a `Complete` def whose own comment says it is wrong

Found only by the inverse axis. Printed: *"Whenever Teneb deals combat damage to a player, **you
may pay {2}{B}**. If you do, put target creature card from a graveyard onto the battlefield under
your control."* The def is `Complete` (by the `#[default]` derive — it declares no marker at all)
and its trigger carries a bare `Effect::MoveZone`: **the {2}{B} is never charged and the
reanimation is unconditional.**

Its own in-source comment says so, and then explains the gap with a claim that is **false at
HEAD**:

> *"DSL GAP (PB-10 Finding 5): 'you may pay {2}{B}. If you do, …' requires an optional mana payment
> on triggered abilities (Cost on triggers or `Effect::PayManaOrElse`), which does not exist in the
> DSL yet."*

`Effect::MayPayThenEffect` is exactly that primitive and has four corpus users on a triggered
ability, `nether_traitor` — this batch's own headline card — among them. This is PB-DX27's
"a blocker note is a claim" a second time, on a def nobody re-checked. Repaired here, with the
primitive this batch is fixing. **No completeness flip**: the def was already `Complete` and stays
`Complete`; what changes is that it stops giving away a free reanimation every combat.

### §1.5 Completeness flips — PREDICTED AND NAMED BEFORE REGENERATION (AC 7242)

**Exactly one flip, UP: `Vampire Gourmand` `partial` → `Complete`.** Predicted coverage
**1,136 / 1,803 = 63.0% → 1,137 / 1,803 = 63.1%**.

The three defs `OOS-DX27-5` names are re-adjudicated under the post-fix rule in §3, and the row's
own framing is corrected there: it says PB-DX27 *"left `ruthless_technomancer` and
`vampire_gourmand` at `partial` on the same shape"*, and only one of those two markers actually
cites this deviation.

**One marker flip re-deals every seeded fixture** (`OOS-CARDS2-3`): `CORPUS_COMPLETE` moves, so
`UI3_SPLIT_COMBAT_SEED` and every seeded pin must be re-observed. PB-DX27 needed **two**
reconciliation passes for a single flip; two are budgeted.
