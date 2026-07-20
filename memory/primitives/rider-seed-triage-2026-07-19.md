# Rider-Seed Mini-Triage — 2026-07-19 (task `scutemob-142`)

<!-- last_updated: 2026-07-19 -->

**Scope**: the rider seeds filed by the PB-OS4..OS11 wave (`scutemob-115`..`141`), chain-verified
against the current engine at `PROTOCOL_VERSION = 26` (`crates/engine/src/rules/protocol.rs:248`) /
`HASH_SCHEMA_VERSION = 63` (`crates/engine/src/state/hash.rs:578`).

**Precedent**: `memory/primitives/oos-retriage-plan-2026-07-18.md` (`scutemob-115`). This doc is its
successor for the OS-wave riders and is the canonical inventory + queue for them. Method: five
parallel `primitive-impl-planner` chain-verifications, each required to walk the full
filter→effect→layer→cost dispatch chain and to re-check every seed premise against oracle text /
CR rather than trusting the filing doc.

**Zero engine/card-def code changed by this task.** Doc-only.

---

## 0. Headline

The brief named **8 rider seeds**. Triage found **11 OS-series IDs** in the docs, of which **2 were
never actually filed**, and — more importantly — the chain walks surfaced **four untracked
correctness defects that outrank every filed seed**, two of them live on cards currently marked
`Complete` (Architecture Invariant #9).

**The headline is not the seed queue. It is this:**

1. **Library top/bottom is inverted between the draw path and the reveal/scry family.** Scry,
   `RevealAndRoute`, and `LookAtTopThenPlace` read the *opposite end of the library* from
   `draw_card`. Confirmed at source (§2.1). ~52 card files affected, several `Complete`.
2. **Every hybrid and Phyrexian pip in an activated cost is currently free.** `can_spend`/`spend`
   never read `cost.hybrid` or `cost.phyrexian`. All 7 filter lands are live "{T}: Add two mana"
   lands today (§2.2).
3. **`helm_of_the_host` is `Complete`, passes `validate_deck`, and its only real ability silently
   never fires** — no card-def `AtBeginningOfCombat` sweep exists (§2.3).
4. **OOS-OS4-2 is documented as CLOSED but has three surviving CR 712.8d/e deviations** (§2.4).
   `CLAUDE.md` and `oos-retriage-plan-2026-07-18.md:579` both state it flatly closed.

Of the 8 briefed seeds: **6 are active-PB-candidates**, **1 is card-gated** (authorable today, no
engine change), **1 is dormant** (blocked on an M10 interactive-decision channel). None were
resolved-stale — unlike the `scutemob-115` retriage, this wave's riders were filed recently enough
that no later wave silently closed them.

**Honest discounted yield across the whole queue: ~11-13 clean flips**, plus repairs to 10+
already-`Complete` cards that are legal-but-wrong today. As always the correctness items are worth
more than the flip count suggests — they buy integrity, not coverage.

---

## 1. Seed inventory

Legend: **CANDIDATE** = ranked into §3. **CARD-GATED** = no engine work needed. **DORMANT** = real
but 0 reachable yield. **NEVER-FILED** = the ID appears only in a conditional or a proposal.

### 1a. Briefed rider seeds — 8

| Seed | Source doc | Verdict | Class | Honest yield |
| --- | --- | --- | --- | --- |
| **OOS-OS9-1** | `oos-retriage-plan:516` | **CANDIDATE** | **correctness** | 2 flips + 1 `Complete`-but-inert repair |
| **OOS-OS8-1** | `pb-review-OS8:115` | **CANDIDATE** | **correctness** (premise wrong — worse than filed) | 1 flip + 7 live-wrong lands |
| **OOS-OS7-2** | `pb-plan-OS7:110` | **CANDIDATE** | **correctness** | 0 flips (repairs `Complete` cards) |
| **OOS-OS7-1** | `pb-plan-OS7:295` | **CANDIDATE** (split R1 / R2+R3) | correctness (R1) + capability | 1 flip on R1; 2 for the full build |
| **OOS-OS6-1** | `pb-plan-OS6:72` | **CANDIDATE** | capability | 4 flips (seed said 2) |
| **OOS-OS4-3** | `ef-batch-plan:1163` | **CANDIDATE** | capability | 1 flip, oracle-gated |
| **OOS-OS8-2** | `oos-retriage-plan:479` | **CARD-GATED** | capability | 1 flip, no engine change |
| hidden_strings optionality | `pb-review-OS10:77` | **DORMANT** | capability | 0 flips (needs M10) |

### 1b. Strays found by grep — not in the brief

| Item | Status | Disposition |
| --- | --- | --- |
| **OOS-OS10-1** | **NEVER-FILED (phantom)** | Filed only conditionally at `pb-plan-OS10:335` ("if Jitte fails execution-verification"). `umezawas_jitte.rs:109` is `Complete` — the condition never triggered. Appears nowhere else. **Strike from all carry-forward lists.** |
| **OOS-OS7-3** | **NEVER-FILED, ID contested** | Double-proposed: `pb-plan-OS7:321` renumbered the `CreaturesControlledByTargetPlayer` note into it, while `pb-review-OS7:118` simultaneously proposed it for the 611.2c content. The 611.2c content won and shipped as OOS-OS7-2; the filter note was orphaned. Underlying gap is real (2 cards). **Refiled below as OOS-RS-5 under a fresh, uncontested ID.** |
| **OOS-OS4-1** | **OPEN, fell off the list** | `CardFace` has no `starting_loyalty` (only `CardDefinition:102` does); keeps `nicol_bolas_the_ravager` + `grist_voracious_larva` unauthored. Dropped from `primitive-wip.md:11-16`. **Re-added to the queue (§3, R10).** |
| **crucible dynamic-X** | **DORMANT, now filed** | `Cost::RemoveCounter { count: u32 }` (`card_definition.rs:1271`) can't express "Remove X". Corpus sweep: crucible is the **only** such card; needs 3 coupled gaps. **Filed as OOS-RS-6, not queued.** |

### 1c. New seeds filed by this triage

| New ID | What | Class | Rank |
| --- | --- | --- | --- |
| **OOS-RS-1** | Library top/bottom inversion, draw path vs reveal/scry family | **correctness, wide** | **R1** |
| **OOS-RS-2** | Hybrid/Phyrexian pips never paid in activated costs | **correctness, live** | **R2** |
| **OOS-RS-3** | OOS-OS4-2 residuals (3 surviving CR 712.8d/e deviations) | correctness, latent | R4 |
| **OOS-RS-4** | Anim Pakal live-counter-vs-LKI deviation on a `Complete` card | correctness | R5 |
| **OOS-RS-5** | `EffectFilter::CreaturesControlledByTargetPlayer` (replaces contested OOS-OS7-3) | capability | folded into R7 |
| **OOS-RS-6** | Dynamic-X counter-removal cost (crucible) | capability | dormant |

---

## 2. Chain-verification notes — the four new correctness findings

### 2.1 OOS-RS-1 — library top/bottom inversion **(highest blast radius)**

Two self-consistent but **mutually opposed** conventions coexist:

| Camp | Sites | "Top" is |
| --- | --- | --- |
| **A** | `Zone::top()` = `v.last()` (`card-types/src/state/zone.rs:159-164`); `draw_card` uses `top()` (`rules/turn_actions.rs:1193-1195`); `move_object_to_bottom_of_zone` = `push_front`, doc'd "front (= bottom)" (`engine/src/state/mod.rs:1787-1792`); cascade/hideaway bottom-moves (`rules/resolution.rs:5935`, `:6014`; `rules/copy.rs:484-495`) | **last** element |
| **B** | `RevealAndRoute` (`effects/mod.rs:4985-4993`), `Scry` (`:3082-3090`), `LookAtTopThenPlace` (`:5065-5073`) — all `object_ids().take(n)`, all commented "the top N cards… ordered from top"; harness declares library "top-to-bottom" then `zone.insert` = `push_back` (`testing/replay_harness.rs:207-212` → `state/mod.rs:994`) | **index 0** |

`Zone::object_ids()` returns ordered zones front-to-back (`zone.rs:128-133`), so camp B's `.take(n)`
reads indices `0..n` — the exact end camp A calls the **bottom**.

**Consequence, in any game state regardless of setup**: a Scry looks at the opposite end of the
library from the next draw. A cascade that puts a card "on the bottom" places it precisely where the
next Scry reads first. **The inconsistency is certain; only which camp is authoritative is open.**

Tests do not discriminate — `crates/engine/tests/mechanics_m_z/reveal_and_route.rs` uses 2-card
libraries with `count: 4` and asserts membership/counts only, never position (`:83-104`, `:150-156`,
`:223-237`, `:270`).

Affected: ~52 files using `RevealAndRoute`/`Scry`/`LookAtTopThenPlace` — `goblin_ringleader`,
`coiling_oracle`, `sylvan_messenger`, `risen_reef`, `chaos_warp`, `satyr_wayfinder`,
`birthing_ritual`, `growing_rites_of_itlimoc`, `yuriko_the_tigers_shadow`, `six`, … several
`Complete`.

**Secondary (same area)**: `resolve_zone_target` **discards** `ZoneTarget::Library { position }`
(`effects/mod.rs:7826-7834`); `LibraryPosition` has **zero** engine read sites (only re-exports at
`engine/src/cards/mod.rs:28`, `engine/src/lib.rs:12`, plus `HashInto` at `state/hash.rs:5474-5479`).
Every `position: LibraryPosition::Bottom` in every card def is inert decoration.

Not a wire change — no enum shape moves. Scenario hashes will move.

### 2.2 OOS-RS-2 — hybrid/Phyrexian pips are free in activated costs

OOS-OS8-1's filed premise ("`abilities.rs` has 0 Phyrexian; birthing_pod can't be expressed") is
**true but is the wrong diagnosis**. The real defect is a silent undercharge:

1. `casting.rs:3990-3991` flattens hybrid/phyrexian **before** payment; life deducted `:4015-4021`.
   **Cast path is correct.**
2. `abilities.rs:748-758` gates on `resolved_cost.mana_value() > 0`, then calls `can_spend`/`spend`
   on the **raw** cost. **No flatten.**
3. `player.rs:148-175`, `:185-206` — `can_spend`/`spend` read only the six colors + generic.
   **`cost.hybrid` and `cost.phyrexian` are never read.**
4. `game_object.rs:133-153` — `mana_value()` *does* count hybrid/phyrexian. So a pure `{B/R}` cost
   has mv=1, passes the `> 0` gate, then `can_spend` sees an all-zero cost → `remaining >= 0` →
   **always true**; `spend` deducts nothing.

**Live today in 7 filter lands** — `twilight_mire.rs:30-31`, `graven_cairns.rs:30-31`,
`sunken_ruins.rs:30-31`, `flooded_grove.rs:30-31`, `rugged_prairie.rs:30-31`, `fetid_heath.rs:32-33`,
`cascade_bluffs.rs:30-31`. Every one is currently a "{T}: Add two mana" land. They are `known_wrong`
for an *unrelated* reason (fixed-mode simplification, `graven_cairns.rs:49-52`); the free-pip defect
is undocumented.

**Record correction**: `drivnod_carnage_dominus.rs:43-44` claims its `{B/P}{B/P}` cost is "already
expressible (PB-9)" — **false** per the chain above.

### 2.3 OOS-OS9-1 — no card-def `AtBeginningOfCombat` sweep

The only engine-side occurrences are two `HashInto` arms (`state/hash.rs:3175`, `:5726`) and the
emblem call at `rules/turn_actions.rs:1689-1698`. `begin_combat` (`:1684-1703`) builds `CombatState`,
collects **emblem** triggers only (CR 114.4), and returns `Vec::new()`. There is **no card-def scan**.
Exactly one broken hop — hops 6-8 (queue→stack `abilities.rs:8251-8258`; resolution +
intervening-if `resolution.rs:2018-2048`, `:2185-2219`; `Condition::YouControlYourCommander`) all
work and are exercised by three sibling sweeps.

**`helm_of_the_host.rs` has no `completeness` field ⇒ `Complete` by `#[default]`
(`card_definition.rs:199-200`), and its only non-Equip ability is the `AtBeginningOfCombat`
`CreateTokenCopy` at `:27-42`.** It enters real games and silently does nothing — a corrupted replay
history per Invariant #9.

### 2.4 OOS-RS-3 — OOS-OS4-2's closure is partial, and the docs say it is closed

The gathering paths really were fixed (`rules/face.rs:1` + readers across
turn_actions/resolution/mana/sba/engine). **Three CR 712.8d/e deviations survive in-source:**

1. `rules/replacement.rs:1180-1191` — `apply_self_etb_from_definition` reads FRONT `def.abilities`;
   the comment self-labels it "PB-OS4b limitation (OOS-OS4-2)".
2. `rules/replacement.rs:1907-1913` — same for non-self permanent replacements.
3. `rules/face.rs:118-148` — `deregister_face_statics` handles only `Static`; **nine** other
   registered families are never deregistered on transform. **This one is a deliberate, well-argued
   deferral, not an oversight**: the doc comment enumerates all nine, explains that PB-OS4b review E2
   had named only four, and reasons that a precise structural remove for all nine is materially
   larger and riskier than the `Static` case (heterogeneous collection shapes, 1-or-2 entries per
   ability, no shared tuple to compare outside `state.continuous_effects`). It asks a future
   implementer to extend symmetrically when a DFC back face first declares one of the nine.

All three are latent — unreachable on today's roster, guaranteed to bite the first DFC with a
back-face ETB replacement. **The in-source comments are honest; the summary docs are not.**
`CLAUDE.md` (Current State, PB-OS4b line) and `oos-retriage-plan-2026-07-18.md:579` both record
OOS-OS4-2 as flatly closed, with no mention of surviving deviations. **The doc claim is the part
worth correcting first** (§6) — the code deferrals are defensible; a reader trusting "closed" is
the actual hazard.

---

## 3. Ranked mini-queue — correctness-first

Ordering rule (inherited from `oos-retriage-plan` §3): (1) live-wrong `Complete` defs first
(integrity, Invariant #9); (2) other correctness bugs; (3) capability by discounted yield.
Discounted ship = expected clean-`Complete` after the PB, at the historical 2-3× overcount discount.

| Rank | PB | Seed(s) | Class | Discounted ship | Wire bump |
| --- | --- | --- | --- | --- | --- |
| **R1** | library ordering reconciliation | **OOS-RS-1** | **correctness, wide** | 0 flips; repairs ~52 files' behavior incl. several `Complete` | **none** (scenario hashes move) |
| **R2** | activated-cost pip payment | **OOS-RS-2** + OOS-OS8-1 | **correctness, live** | **1** (birthing_pod) + 7 lands correctly costed | PROTOCOL (new `ActivateAbility` fields); HASH likely neutral |
| **R3** | `AtBeginningOfCombat` sweep | **OOS-OS9-1** | **correctness** | **2** (loyal_apprentice, siege_gang_lieutenant) + helm_of_the_host repaired | **none** |
| **R4** | face-aware residuals | **OOS-RS-3** | correctness, latent | 0 flips; closes 3 real deviations + a false doc claim | none expected |
| **R5** | Anim Pakal LKI counters | **OOS-RS-4** | correctness | 0 flips; repairs 1 `Complete` card | none expected |
| **R6** | CR 611.2c set snapshot | **OOS-OS7-2** | correctness | 0 flips; repairs `golgari_charm` + siblings | PROTOCOL + HASH |
| **R7** | target-scoped filters (folded) | **OOS-OS7-1 R1** + **OOS-RS-5** | correctness + capability | **3** (kogla, polymorphists_jest, great_oak_guardian) | PROTOCOL + HASH (one shared bump) |
| **R8** | multi-count sacrifice cost | **OOS-OS6-1** | capability | **4** (teysa, bolass_citadel, kellogg, westvale) | PROTOCOL + HASH |
| **R9** | edgar return-transformed | **OOS-OS4-3** | capability | **1**, oracle-gated | PROTOCOL 26→27 / HASH 63→64 |
| **R10** | back-face starting loyalty | **OOS-OS4-1** | capability | **2** (nicol_bolas_the_ravager, grist) | PROTOCOL + HASH |
| **R11** | attacked-player trigger family | **OOS-OS7-1 R2+R3** | capability | **1** (karazikar) | none (TriggerCondition is outside the closure) |

**Not queued**: **OOS-OS8-2** (muxus) — card-authoring follow-up, no engine change, but **gate it
behind R1** or it ships legal-but-wrong (would reveal the wrong six cards). **hidden_strings** and
**OOS-RS-6** (crucible) — dormant, see §4.

**Total discounted ship: ~11-13 flips**, plus integrity repairs on 10+ already-`Complete` cards.

**Sequencing note**: R2, R6, R7, R8, R9, R10 each force a PROTOCOL bump. Batch adjacent capability
PBs where possible to minimize churn (`oos-retriage-plan` §5), but **do not batch R2 with R6** — they
were flagged as a collision risk by independent verification.

### Notable per-seed corrections to the filed record

- **OOS-OS7-1 under-scopes itself.** It names 2 sub-primitives (R1 defending-player target filter,
  R2 opponent-attack trigger); the chain shows **3** are needed. Karazikar's "Whenever you attack a
  player" needs a **per-attacked-player fan-out** (R3) — `WheneverYouAttack` dispatch at
  `abilities.rs:4147-4170` fires **once per combat** and sets no `defending_player_id`. Without R3,
  R1 has nothing to read for Karazikar even though it fully unblocks Kogla.
- **OOS-OS6-1's sweep missed cards.** It keyed on "Sacrifice five" and missed "three"/"ten" —
  `teysa_orzhov_scion` and `bolass_citadel` carry the identical blocker. Yield is 4, not 2.
- **OOS-OS9-1 over-counted.** Filed as "~4-6 flips"; honest count is **2** —
  `legion_warboss`, `goblin_rabblemaster`, `mirage_phalanx` each carry surviving independent
  blockers (Mentor, forced-attack, Soulbond).
- **OOS-OS4-3's wire numbers are stale.** The seed says "PROTOCOL 19→20 / HASH 56→57" (branch-local
  values). Live is 26/63; the real bump is **26→27 / 63→64**.
- **OOS-OS4-3's shape was overruled.** `pb-plan-OS4:281` described a *delayed* "next end step"
  return; the OS4 review overruled it (`pb-review-OS4.md:37`, "Edgar returns immediately") and
  deleted the speculative `...NextEndStep` variant. Implement the **immediate** shape.

### Oracle-sourcing hazard (blocks R8 and R9)

`mcp__mtg-rules__lookup_card` **does not flatten `card_faces`**. It returned the DFC header only —
type line, color identity, keywords — with an **empty oracle text field** for *Edgar, Charmed Groom*,
*Edgar Markov's Coffin*, *Westvale Abbey*, and *Ormendahl, Profane Prince*. The oracle text these two
seeds rely on is quoted **second-hand** from prior plan docs. **R8 and R9 must re-source both faces
from `cards.sqlite` / `.scryfall-cache` before authoring.** Flagged, not guessed — per the PB-EF5 and
PB-OS11 precedent where filed premises did not match the printed card.

---

## 4. Dormant — real, but do not queue

| Item | Why dormant |
| --- | --- |
| **hidden_strings** tap-or-untap + "you may" | Both claimed gaps verified TRUE at the *executor*, not just the type level: `Effect::Choose` executes `choices.first()` and discards `prompt` (`effects/mod.rs:3422-3427`), machine-barred from `Complete` by `crates/engine/tests/core/effect_choose_gate.rs:81-93`; no `TapOrUntap` variant; `MayPayOrElse` always declines (`:3428-3431`); the lone `optional: bool` (`card_definition.rs:2058`) is **explicitly inert** — destructured as `optional: _` at `effects/mod.rs:5054-5059`. **`ModeSelection` does NOT resolve it and would be CR-wrong**: CR 700.2 requires a bulleted list (Hidden Strings has none) and CR 700.2a puts mode choice at *cast* time while this card chooses on *resolution* (CR 608.2). The real blocker is the missing M10+ interactive-decision channel. **0 flips. Keep `known_wrong`.** |
| **OOS-RS-6** crucible dynamic-X | `crucible_of_the_spirit_dragon` is the **only** card with a literal "Remove X" cost (all near-misses — khalni_heart, tekuthal, spawning_pit, ominous_seas, ramos — are fixed-count). Needs 3 coupled gaps (X-count cost + variable any-color mana + Dragon `ManaRestriction`), and widening the count type touches the PB-OS11 mana-ability lowering path. 1-card payoff does not justify a wire change. |

---

## 5. First dispatchable PB — full spec

### PB-RS1 — library ordering reconciliation (OOS-RS-1) · **CORRECTNESS** · wide

**Title**: `PB-RS1: reconcile library top/bottom — reveal/scry family reads the opposite end from draw (OOS-RS-1)`
**Class**: CORRECTNESS (Invariant #9 — live-wrong on shipped `Complete` defs)
**Pipeline**: `/implement-primitive` (plan → implement → review → fix). Agents:
`primitive-impl-planner` → `primitive-impl-runner` → `primitive-impl-reviewer`.

#### Step 0 — the probe that decides the fix direction (do this FIRST)

The inconsistency is **confirmed** (§2.1); which camp is authoritative is **not**. Before any edit,
write an executing test that builds a ≥3-card library via `GameStateBuilder` and compares:

- which card `draw_card` yields, versus
- which card `Effect::Scry { count: 1 }` / `RevealAndRoute { count: 1 }` sees.

They will disagree. **Then decide by CR, not by which is easier to change**: CR 103.2 (decks are
libraries in random order), CR 121.1 ("top" is a defined position), CR 701.19 (Scry), CR 702.85a
(cascade → bottom). Camp A (`top()` = last) is load-bearing for draw, cascade, hideaway, and copy;
camp B is confined to three effect arms plus a harness comment. **Expect the fix to be on camp B's
side**, but let the probe and CR say so — do not assume.

#### Scope (what to change)

1. **`crates/engine/src/effects/mod.rs`** — the three top-N reads: `RevealAndRoute` (`:4985-4993`),
   `Scry` (`:3082-3090`), `LookAtTopThenPlace` (`:5065-5073`). Replace `object_ids().take(n)` with a
   top-N helper that agrees with `Zone::top()`. Add **one** shared helper
   (`Zone::top_n(n) -> Vec<ObjectId>`) in `card-types/src/state/zone.rs` next to `top()` rather than
   three ad-hoc `.rev()` calls — the whole defect is that this logic was open-coded three times.
2. **The matching "to the bottom" writes** in those same three arms currently use plain
   `expect_move_object_to_zone` (= `push_back` = the top end). Route them through
   `move_object_to_bottom_of_zone` (`state/mod.rs:1787`), which is already correct.
3. **`testing/replay_harness.rs:207-212`** — the "top-to-bottom order" comment and the insert loop
   must be reconciled with the chosen convention. If camp A wins, the harness must insert library
   cards in reverse so that the *first-declared* card is the one `draw_card` yields.

#### Explicitly NOT in scope

- **`ZoneTarget::Library { position }` being discarded** (`effects/mod.rs:7826-7834`) and
  `LibraryPosition` having zero read sites. Real, adjacent, and it means Muxus's "rest on the bottom
  in a random order" is currently inexpressible — but it is a *separate* capability gap. **File as a
  follow-up seed; do not widen this PB.** Fixing the top-N reads without it still leaves R1 a strict
  improvement.
- Any card-def marker flips. This PB repairs behavior; the flips land in the follow-up authoring.

#### Wire-change expectation

- **None.** No DSL/`Effect`/`Command`/`GameEvent` type is added or reshaped — this changes the order
  in which existing code reads an existing zone. **No `PROTOCOL_VERSION` / `HASH_SCHEMA_VERSION`
  bump.** Scenario/state hash *values* will move (the pinned artifacts are schema fingerprints, not
  state digests). *If the planner finds a schema fingerprint must be re-pinned, that contradicts this
  expectation and is a signal to stop and re-scope.*

#### Cards (currently legal-but-wrong)

~52 files use the three affected effects. **Mandatory roster sweep from `all_cards()`, not grep**
(SR-34/36): every def containing `Effect::Scry`, `Effect::RevealAndRoute`, or
`Effect::LookAtTopThenPlace`. Known members include `goblin_ringleader`, `coiling_oracle`,
`sylvan_messenger`, `risen_reef`, `chaos_warp`, `satyr_wayfinder`, `birthing_ritual`,
`growing_rites_of_itlimoc`, `yuriko_the_tigers_shadow`, `six`. **Report the full affected set in the
close-out — the count is the deliverable**, these are integrity repairs and not coverage flips.

#### Mandatory tests

1. **The discriminating probe, kept as a permanent regression test**: ≥3-card library; assert
   `Scry`/`RevealAndRoute` see the *same* card `draw_card` would yield. This test must **fail before
   the fix and pass after** — the existing `reveal_and_route.rs` suite passes today while the bug is
   live, which is exactly why it did not catch it.
2. **De-vacuous `crates/engine/tests/mechanics_m_z/reveal_and_route.rs`**: `:83-104`, `:150-156`,
   `:223-237`, `:270` use 2-card libraries with `count: 4` and assert membership only. Give them
   libraries longer than `count` and assert **position**, citing CR 121.1.
3. **Round-trip against cascade**: cascade a card to the bottom (CR 702.85a), then Scry 1 → assert
   the Scry does **not** see the just-bottomed card. This is the interaction that proves the two
   camps are reconciled.
4. **Scry-to-bottom ordering**: Scry 2, send both to the bottom → assert they land below every
   pre-existing library card and that `draw_card` still yields the original top.
5. **Golden-script reconciliation**: any `test-data/generated-scripts/` script asserting library
   order must be updated to the correct behavior with a CR 121.1 / 701.19 citation. The reviewer must
   confirm none silently encode the old inversion.

#### Why this is the right first dispatch

Highest correctness leverage in the queue: it is live-wrong on already-`Complete` cards, it is the
**only** item whose defect silently corrupts *other* subsystems' results (every scry/reveal card
reads the wrong end, and cascade actively feeds the wrong end), the fix is engine-internal with **no
wire bump**, and it de-vacuouses a test suite that currently passes while the bug is live. It is also
a hard **gate on OOS-OS8-2**: authoring muxus before this lands would ship a card that reveals the
wrong six cards while marked `Complete`. Direct analogue of PB-OS1's "de-vacuous the canary that
lies" opening.

---

## 6. Source-doc updates applied by this task

**Zero engine/card-def code changed.** Doc-only edits:

1. `memory/primitive-wip.md` — banner repointed from the "8 rider seeds" list to this doc; OOS-OS4-1
   restored to the list; OOS-OS10-1 struck as a phantom.
2. `memory/primitives/pb-plan-OS10.md` — OOS-OS10-1 marked **NEVER-FILED** (condition never
   triggered; Jitte flipped `Complete`).
3. `memory/primitives/pb-plan-OS7.md` + `pb-review-OS7.md` — OOS-OS7-3 marked **NEVER-FILED, ID
   contested**; pointer to OOS-RS-5.
4. `memory/primitives/oos-retriage-plan-2026-07-18.md` — the flat "OOS-OS4-2 closed" claim at `:579`
   corrected to **partially closed**, pointing at OOS-RS-3.
5. `memory/workstream-state.md` — rider-seed line repointed here; seed count corrected 8 → 11+4.

Nothing else needed a per-doc status change: no rider seed was found resolved-stale.
