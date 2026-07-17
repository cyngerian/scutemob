# SR-34 roster reconciliation — criterion 4767

**Date**: 2026-07-17 · **Task**: `scutemob-90` · **Method**: empirical probe, deleted before commit.

Every one of the 27 affected `Complete` defs in `memory/primitives/sr34-affected-defs.md` was
put on a battlefield and **activated** — every mana ability via `Command::TapForMana`, every
activated ability via `Command::ActivateAbility` with the stack then resolved to completion —
and the resulting mana pool, life total, stack depth and cost payment recorded. Oracle text is
MCP-authoritative (Scryfall), not the def's own `oracle_text` field, which is itself wrong on
one card (see Magnifying Glass).

**A shape assertion is not evidence.** Nothing below rests on `!mana_abilities.is_empty()`.
Every verdict cites a pool delta.

---

## 0. The prediction this replaced, and what the probe falsified

A prior trace-only review (no Bash) predicted **10 work / 17 need markers**. The probe agrees on
the headline split by luck, not by reasoning: **7 of 27 rows were wrong**, in both directions.

| # | Card(s) | Predicted | Probe found | Verdict moved? |
|---|---|---|---|---|
| 1 | **Temple of the Dragon Queen** | `KnownWrong` — `ChooseColor(Color::White)` is hardcoded → always White | **FALSE.** `ChooseColor` is a *fallback* default behind a real board-scanning heuristic (`replacement.rs`, CR 613.1e/614.12a): white board → White, **black board → Black**. It produces a **legal** colour. Only defect is CR 605.3b (uses the stack). | **YES: KnownWrong → Partial** |
| 2 | **Three Tree City / Secluded Courtyard / Unclaimed Territory** | also hardcode `ChooseCreatureType("Human")` | **FALSE.** Same heuristic — probed choice was **"Soldier"**, not the declared "Human" default. The `KnownWrong` verdict survives but on a *different* clause (colorless mana). | verdict same, **note was wrong** |
| 3 | **Magnifying Glass** | WORK → `{C}` | **The def contradicts its own card.** Real oracle is a bare `{T}: Add {C}` and `{4},{T}: Investigate`; the def modelled `{1},{T}: Add {C}` and `{3},{T}`. It was shipping a **net-zero mana rock**. Fixed here. | Complete only **after a fix** |
| 4 | **Cabal Stronghold / Crypt of Agadeem / Three Tree City / Voldaren Estate / Maelstrom / Secluded Courtyard / Unclaimed Territory / Phyrexian Tower** | treated as single-ability cards | Each has a **correct `{T}: Add {C}`/`{B}` mana ability** *plus* the problem ability. The roster's Table 1 listed only the problem ability. | verdict same, notes rewritten |
| 5 | **Voldaren Estate** | KnownWrong (colour only) | **Also an SF-9 victim** the claim did not name: its `Pay 1 life` is never charged (life 40 → 40). | note expanded |
| 6 | **Crypt of Agadeem** | — | *Probe self-correction*: first run showed +0 black and looked like a bug. It counts **black** creature cards; the probe had seeded green ones. With 3 black cards → **+3**. The engine was right and the probe was wrong. | no change |
| 7 | **Darkwater Catacombs / Viridescent Bog** | WORK | Confirmed — **but** only after checking the oracle: these are Odyssey filter lands whose *printed* text was a three-way `{U}{U}, {U}{B}, or {B}{B}` choice. Current Scryfall oracle is a plain `{1},{T}: Add {U}{B}`. The defs match the errata. **Do not "fix" them back into a choice.** | no change |

The prior agent's **central claim is CONFIRMED**: SR-34 did **not** cause the `any_color`
colorless bug. `handle_tap_for_mana` step 8 and `Effect::AddManaAnyColor` add the *same* one
`ManaColor::Colorless`, so these cards produced `{C}` before SR-34 too — SR-34 strictly improved
the mechanism (stack → no stack, cost now charged) and left the colour bug untouched. Escaping
into a real `ManaAbility` does not help. **No regression; a pre-existing lie surfaced.**

---

## 1. The Partial-vs-KnownWrong line

**Scope bound (SR-34 review Finding 5, added by `scutemob-90` fix phase)**: the taxonomy
below was applied to this task's 27-def roster only. The same `any_color` → `{C}` defect
demonstrably persists in `Complete` defs *outside* the roster — calibration cases:
`crates/card-defs/src/defs/birds_of_paradise.rs:38` (`{T}: Add one mana of any color`,
`Completeness::Complete`) and `crates/card-defs/src/defs/command_tower.rs:21` (`Complete`
by default, no marker), both `Effect::AddManaAnyColor`, both producing `{C}` by the exact
mechanism (`handle_tap_for_mana` step 8 / `Effect::AddManaAnyColor`'s stack arm) that got
Mana Confluence demoted here. They were **not** demoted by this task — extending the
demotion corpus-wide needs its own roster and moves headline coverage, the same reason
EF-13 was deferred — and are tracked instead as live victims in SF-11
(`memory/card-authoring/sr34-engine-findings-2026-07-17.md`). Read "stated once, applied
uniformly" below as bounded to the 27-def roster, not as a corpus-wide claim.

Stated once, applied uniformly, because the prior agent flagged it as a genuine judgment call:

> **Does the def write wrong game state, or does it write only correct state while omitting a clause?**

- **`Partial`** — every observable state the card produces is *correct*; what is missing is CR
  605.3b timing (it resolves through the stack, so it cannot fund a cast in the same priority
  window and grants opponents a window it should not). Nothing false enters the history.
- **`KnownWrong`** — it writes state that is *wrong*. "Add one mana of any color" producing `{C}`
  is not a lesser version of the right answer: **CR 106.1a/106.1b make colorless a mana _type_,
  not a colour**, so `{C}` is outside the legal option set entirely. This matches SR-33's
  precedent (a land producing an unprinted colour is `KnownWrong`).

This agrees with the prior agent's recommendation for the "right mana, wrong mechanism" group.

---

## 2. Verdict table (27 defs: 10 Complete · 6 Partial · 11 KnownWrong)

### Certified `Complete` — 10

Pinned by `sr34_certified_defs_produce_exactly_their_printed_mana`
(`tests/primitives/primitive_sr34_composite_mana_costs.rs`), which asserts the **declared**
generic cost and the **exact** pool for all six colours. Verified non-vacuous against two
attacks (wrong cost; swapped colours) — both caught.

| Card | Oracle (MCP) | Probe evidence | CR |
|---|---|---|---|
| Boros Signet | `{1},{T}: Add {R}{W}` | pool `C5→C4, W5→W6, R5→R6`, stack empty, `ManaCostPaid{generic:1}` | 605.1a, 605.3b, 118.3a |
| Dimir Signet | `{1},{T}: Add {U}{B}` | `C5→C4, U5→U6, B5→B6`, stack empty | 605.1a, 605.3b |
| Golgari Signet | `{1},{T}: Add {B}{G}` | `C5→C4, B5→B6, G5→G6`, stack empty | 605.1a, 605.3b |
| Izzet Signet | `{1},{T}: Add {U}{R}` | `C5→C4, U5→U6, R5→R6`, stack empty | 605.1a, 605.3b |
| Orzhov Signet | `{1},{T}: Add {W}{B}` | `C5→C4, W5→W6, B5→B6`, stack empty | 605.1a, 605.3b |
| Rakdos Signet | `{1},{T}: Add {B}{R}` | `C5→C4, B5→B6, R5→R6`, stack empty | 605.1a, 605.3b |
| Simic Signet | `{1},{T}: Add {U}{G}` | `C5→C4, U5→U6, G5→G6`, stack empty | 605.1a, 605.3b |
| Darkwater Catacombs | `{1},{T}: Add {U}{B}` (errata'd) | `C5→C4, U5→U6, B5→B6`, stack empty | 605.1a, 605.3b |
| Viridescent Bog | `{1},{T}: Add {B}{G}` (errata'd) | `C5→C4, B5→B6, G5→G6`, stack empty | 605.1a, 605.3b |
| **Magnifying Glass** | `{T}: Add {C}` / `{4},{T}: Investigate` | **def was wrong — fixed here.** Now `C5→C6` free, stack empty; Investigate charges `{4}` | 605.1a, 605.3b |

### `Partial` — right mana, wrong mechanism (CR 605.3b) — 6

| Card | Probe evidence | Real blocker |
|---|---|---|
| Cabal Coffers | 4 Swamps → **`B5→B9`** (+4, correct), paid `{2}`, via stack | `Effect::AddManaScaled` deliberately excluded from the widened gate (Finding A) — `handle_tap_for_mana` has no `AddManaScaled` branch and would read `produces:{B:1}` literally. **SF-8.** |
| Cabal Stronghold | 4 basic Swamps → **`B5→B9`** (+4), paid `{3}`, via stack. `{T}: Add {C}` mana ability is correct (`C5→C6`) | same — SF-8 |
| Crypt of Agadeem | 3 **black** creature cards in GY → **`B5→B8`** (+3), paid `{2}`, via stack. `{T}: Add {B}` mana ability correct (`B5→B6`) | same — SF-8 |
| Ashnod's Altar | **`C5→C7`** (+2, correct), creature sacrificed, via stack | `Cost::Sacrifice(filter)` needs a caller-supplied `ObjectId`; `Command::TapForMana{player,source,ability_index}` has no payload for it (Krark-Clan Ironworks class) |
| Phyrexian Tower | **`B5→B7`** (+2, correct), creature sacrificed, via stack. `{T}: Add {C}` correct (`C5→C6`) | same — no ObjectId channel |
| **Temple of the Dragon Queen** | ETB set `chosen_color=Some(White)` on a white board and **`Some(Black)` on a black board**; `{T}` then added **`W5→W6`** — the *chosen* colour, via stack | `Effect::AddManaOfChosenColor` has no arm in `try_as_tap_mana_ability` → 0 mana abilities. Choice is deterministic, not player-made (M10), but always legal. |

### `KnownWrong` — produces mana outside the legal option set (CR 106.1b) — 11

| Card | Probe evidence | Note |
|---|---|---|
| Mana Confluence | `C5→C6` (colorless), **life 40→39** | cost now correct; colour is not. SR-34 lowered it properly. |
| Staff of Compleation | MA: `C5→C6`, **life 40→38** (correct). **`{T},Pay 3: Proliferate` → life 40→40. `{T},Pay 4: Draw` → life 40→40.** | **SF-9 live victim ×2** — free proliferate + free draw, shipping `Complete` |
| Voldaren Estate | act[0]: `restricted=[Colorless×1(SubtypeOnly(Vampire))]`, **life 40→40** | restriction honoured; colour wrong; **SF-9 victim** (life uncharged). `{T}: Add {C}` correct. |
| Phyrexian Altar | `C5→C6` (colorless) via stack | prints "any color" |
| Goldhound | `C5→C6` (colorless), sac'd self, **stack empty** | SR-34 lowered it (`sacrifice_self` already existed); mechanism now right, colour wrong |
| Druids' Repository | 5 charge counters → `C5→C6` (colorless) via stack | `Cost::RemoveCounter` unlowerable; **no tap at all** — and every `try_as_tap_mana_ability` return site hardcodes `requires_tap: true`, so the `false` path is **unexercised corpus-wide** |
| Gemstone Array | 5 charge counters → `C5→C6` (colorless) via stack | same; `{2}: Put a charge counter` is correct |
| Three Tree City | ETB chose **"Soldier"**; `{2},{T}` → **2 mana for 2 creatures (amount correct)** but **colorless**, not a chosen colour | `AddManaOfAnyColorAmount` ignores colour choice outright. `{T}: Add {C}` correct. |
| Maelstrom of the Spirit Dragon | `restricted=[Colorless×1(SubtypeOrSubtype(Dragon,Omen))]` | restriction honoured, colour wrong. `{T}: Add {C}` correct. |
| Secluded Courtyard | `restricted=[Colorless×1(CreatureSpellsOnly)]` | ditto; "or activate an ability of…" also unenforced |
| Unclaimed Territory | `restricted=[Colorless×1(CreatureSpellsOnly)]` | ditto |

---

## 3. Consequences for the campaign

### Coverage delta (measured from the **compiled registry**, both sides)

| | Complete | Partial | KnownWrong | Inert |
|---|---:|---:|---:|---:|
| before (`git stash` of this task's def edits) | **1015** (58.1%) | 570 | 101 | 62 |
| after | **998** (57.1%) | 576 | 112 | 62 |
| delta | **−17** (−1.0pp) | +6 | +11 | 0 |

The arithmetic closes exactly: 1015 − 17 = 998, and +6 Partial +11 KnownWrong = the 17
demotions. Magnifying Glass is **not** among them — its def bug was fixed, so it stays
`Complete` on merit.

**Note for CLAUDE.md**: the recorded baseline of "57.9%" is the `tools/authoring-report.py`
text heuristic, not the registry. The registry baseline is **58.1% (1015/1748)**; the report
now reads 57.1% / 998 clean, which happens to agree post-change but is measuring a different
thing (a TODO scan, not `completeness`). Prefer the registry when the number is load-bearing —
this is the same class of discrepancy CLAUDE.md already records against that report's
`abilities: vec![]` regex.

- 10 stay `Complete` (1 of them only because a def bug was fixed); **17 demoted**.
- The `any_color → colorless` class is the single biggest blocker here: **8 of 11** `KnownWrong`
  verdicts are that one bug. One primitive (a colour channel for `any_color` mana, deterministic
  or via `TapForMana{ability_index}` per `memory/decisions.md`) un-blocks all eight.
- `Effect::AddManaAnyColorRestricted` having no `try_as_tap_mana_ability` arm costs 4 more cards
  a CR 605.3b violation *on top of* the colour bug (Voldaren Estate, Maelstrom, Secluded
  Courtyard, Unclaimed Territory).
- **SF-9 is not hypothetical** — see the findings doc.
