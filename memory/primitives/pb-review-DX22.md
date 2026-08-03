# Primitive Batch Review: PB-DX22 — Make the Fuzzer a Real Instrument

**Date**: 2026-08-03
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 103.3, 103.5, 400.7, 704.5m, 704.5n, 903.6, 903.8, 903.9a, 903.9b, 903.10a
**Engine files reviewed**: none changed (verified by inspection — see "Method limits")
**Simulator files reviewed**: `crates/simulator/src/fuzz_setup.rs` (new, 213 lines),
`crates/simulator/src/bin/fuzzer.rs`, `crates/simulator/src/lib.rs`,
`crates/simulator/src/{legal_actions,local_game,invariants,setup,deck}.rs` (read for correctness
and for the claimed doc-only corrections), `crates/simulator/tests/local_game.rs`,
`crates/simulator/tests/pb_dx22_fuzz_instrument.rs` (new, 11 probes)
**Docs/memory reviewed**: `memory/primitives/pb-plan-DX22.md`, `memory/primitive-wip.md`,
`docs/audits/decision-point-audit.md` §8.1 rows `OOS-DX22-1..11`,
`docs/mtg-engine-simulator.md`, `memory/workstream-state.md`
**Card defs reviewed**: 0 — the batch changes no card def (`crates/card-defs` untouched)

## Verdict: needs-fix

The two behavioural fixes are **CR-correct and correctly seeded**. `build_fuzz_state` reproduces
`setup::build_initial_state`'s RNG discipline exactly (one `StdRng::seed_from_u64(seed)`, ascending
`PlayerId`, `random_deck` then `shuffle` interleaved per seat), `place_registered_deck` makes the
CR 903.6 place-and-register pair structurally inseparable, and `register_commander_zone_replacements`
is called at the same point `setup.rs:452` calls it. The extraction is genuinely behaviour-neutral,
the probe set is unusually strong, the revert-proof ledger is honest, and the two places where the
**plan's own revert-proofs were refuted by execution** were recorded rather than accommodated —
which is exactly what this batch exists to do.

The findings are not about game state; they are about **evidence**, which for an
EVIDENCE-INTEGRITY batch is the deliverable. Two are HIGH. (1) The instrument that produced the
batch's headline numbers — `CommanderCastFromCommandZone` 0 → 36, 13 CR 903.9a returns, non-empty
`commander_damage_received` in 16/20 games, the 20-seed first-cast band `[3..29]`, first `PlayLand`
1-7 — was a scratch `examples/dx22_p10.rs` that was **deleted**, and **no committed code in the
repo can re-derive any of them** (`grep CommanderCastFromCommandZone crates/simulator/src` returns
three comments and zero code). CR 903.10a in particular has **no probe at all**; its only evidence
is that deleted binary, so acceptance criterion 2's "commander mechanics exercised or explicitly
probed" is satisfied for 903.6/903.8/903.9a/903.9b and **not** for 903.10a. (2) `invariants.rs`
now ships, in bold, "**426 total violations, and not one of them is `stack_consistency`**" — a
universal negative over 426 violations evidenced by **94** of them, because `mtg-fuzzer` prints
per-violation detail for only the first five offending games (`bin/fuzzer.rs:246`). That sentence
is committed into the very block whose SIM-3 caveat this batch was correcting, and its audit row
instructs successors to act on it.

Everything else is MEDIUM/LOW and cheap. Criterion 5 is correctly reported as in progress; §"What
criterion 5 must still contain" below lists what I think it must carry.

---

## Answers to the six questions asked

**1. Is the shuffle CR-correct and correctly seeded? YES, and the determinism claim holds.**
CR 103.3 and CR 903.6 both require the library to be randomised after the commander is set aside;
`fuzz_setup.rs:176` does it, on `deck.main_deck` only, after the commander has been split off into
`DeckConfig::commander`. The draw order matches `setup.rs:350/372/422` byte-for-byte
(`deck₁, shuffle₁, deck₂, shuffle₂, …`). Determinism in `seed` alone is real, and I checked the
three things that could have broken it:
* `all_cards()` is a statically-ordered `Vec` and every `random_deck` filter preserves that order;
* the only iteration inside `register_commander_zone_replacements` is over `state.players`, which
  is `OrdMap<PlayerId, PlayerState>` (`state/mod.rs:125`) — **key-ordered, not hasher-seeded** — so
  the eight replacement IDs are assigned in a seed-independent, process-independent order. Had this
  been an `im::HashMap` with `RandomState`, P3 (two builds in **one** process) could not have caught
  it; it is not, so there is nothing to catch.
* `card_defs: HashMap<String, _>` is only ever `.get()`, never iterated.
The `None`-fallback branch does **not** desynchronise anything, but the doc's stated reason is
imprecise — see LOW 5. `FuzzGameSetup::decks` returning the pre-shuffle list is doc'd in bold and
is self-defending (P2 fails loudly if it ever holds the dealt order), so it is a LOW, not a trap —
see LOW 6.

**2. Is the commander registration complete? YES — I found nothing missing beyond the two filed
items, and I looked line by line against `setup.rs:341-452`.** The full divergence list is:
no `validate_deck` (`OOS-DX22-5`), no CR 103.5 opening hand (`OOS-DX22-1`), silent
`MissingCardDefinition` skip (`OOS-DX22-4`), no `DeckSource` (recorded in `setup.rs`'s module doc),
no `names` map (not a rules property, the fuzzer builds bot names itself at `fuzzer.rs:325`), and
**no `debug_assert_eq!(deck.main_deck.len(), 99)`** — the one item on that list that is not filed
anywhere (LOW 4). Everything CR 903 needs is present: `player_commander` populates
`commander_ids`, which is what `legal_actions.rs:757` (CR 903.8 offer), `effective_cast_cost`
(CR 903.8 tax), `rules/commander.rs` (CR 903.9a SBA), `rules/combat.rs` (CR 903.10a damage) and
`register_commander_zone_replacements` (CR 903.9b) all key off. The `commander_tax` map defaults
empty and is read, not required to be seeded. CR 903.3 partner is unrepresentable in `DeckConfig`
and correctly asserted as exactly-one rather than at-least-one (`OOS-SIM4-3`, cited at P5).

**3. Do the ten probes gate the real code, or a reconstruction? Nine gate the real code; P5's
half (b) gates a reconstruction — and that is acceptable, but its doc oversells it.**
`run_single_game` (`fuzzer.rs:302`) does nothing to the state except call `build_fuzz_state`, so
P1-P9 are genuinely probes on the binary; §B3's constraint is met for the fuzzer. P5(b) cannot call
`tests/local_game.rs::build_state` (private to a different test crate) so it rebuilds the builder
scaffolding and duplicates `fixed_deck`. It therefore adds **no discrimination over P5(a)** — the
same revert (`delete builder.player_commander`) reddens both, because both call the same
`place_registered_deck`. That is acceptable **only because the hole it appears to cover is actually
covered elsewhere**: if someone re-inlined the placement into `build_state`, **P11 would redden**
(the string `in_zone(ZoneId::Command(` would reappear in a file with no `player_commander`), and
`test_dx22_cr_903_9b_replacements_exist_in_the_fixed_deck_build` would redden too (the eight
redirects derive from `commander_ids`). So coverage is fine; the claim at
`pb_dx22_fuzz_instrument.rs:333-334` that half (b) is "that function's live path" invites a reader
to believe P5 gates `build_state`, which it does not. LOW 7.

**4. Does P9 still prove what its name claims? NO — and this is a real hole, correctly diagnosed
by the batch but not repaired.** `LegalAction::CastSpell { from_zone: command_zone }`
(`legal_actions.rs:779-783`) becomes an ordinary `Command::CastSpell(CastSpellData)`, and
`CastSpellData` carries only `card: ObjectId` — no zone. P9 matches
`matches!(rec.command, Command::CastSpell(_))`, so **a commander cast alone satisfies it**. The
batch measured exactly that (shuffle reverted ⇒ 3 of 4 seeds still green at turns 26/25/25) and
recorded it honestly. What was *not* measured is the shipped direction: nobody checked whether
seeds 1-4's observed first casts (17/9/25/23) are library casts or command-zone casts. Since P10
puts the typical first commander cast at game turn 38-107 they are *probably* library casts, but
"probably" is the word this batch exists to delete. This is one line to fix — see MEDIUM 2.
Mitigating: an un-shuffle regression would still be caught by P2, so the hole is in P9's *claim*
and in `OOS-UI2-1`'s closure evidence, not in the batch's aggregate coverage.

**5. Is the `attachment_validity` "not caused by this batch" argument sound? Yes for causation of
the engine defect, no as a complete account — and the row mis-cites the CR.** 0 engine lines is
decisive that the batch did not *write* the bug. But the row (and the wip) cite **CR 704.5n** for
"an Aura attached to an illegal or absent object is put into its owner's graveyard" — 704.5n is the
**Equipment/Fortification** rule, whose disposition is *unattach and remain on the battlefield*;
the Aura rule is **704.5m** (to owner's graveyard). Those are two different fixes, and the row
sends the successor at the wrong one. Separately, the row does not classify the violation as
*transient* vs *persistent*: `check_attachment_validity` (`invariants.rs:399`) runs on a sampled
state, and CR 704.3 SBAs in this engine are checked on step entry and at resolution, not on every
priority grant (`OOS-M11-7`) — so a three-sample violation at one turn is exactly the shape of the
known self-healing SBA lag, and could be a **false positive of the same family SIM-3 found in
`stack_consistency`**. There *is* also a simulator-side path worth naming that the row misses: the
batch made commanders change zones for the first time in fuzz games (13 CR 903.9a returns), and a
commander leaving the battlefield for the command zone is precisely the CR 400.7 event that orphans
an attachment — so the likely mechanism is **commander-specific**, i.e. created by *this batch's*
registration rather than generic. That is a cheap, valuable sharpening. MEDIUM 4.

**6. Vacuity and non-vacuity floors: all present except P11's, which counts itself.** P2/P3/P4
floor both sides at 99; P6 asserts 8 exactly *and* one-hit-per-(seat, zone) naming the seat's own
commander; P7 asserts the pending vector is empty *before* the SBA runs; P8 asserts
`taxed != printed`; P9 asserts `observed.len() == 4`; the 4b probe has a naming floor. **P11's
`matched.len() >= 4` is inflated by one**: the gate file itself contains
`const PLACEMENT: &str = "in_zone(ZoneId::Command(";` (`:270`) and the word `player_commander`
(`:271`, `:319`), so it matches its own needle and satisfies its own rule. Genuine matches post-fix
are `src/setup.rs`, `src/fuzz_setup.rs`, `src/legal_actions.rs`, `tests/commander_cast.rs` = 4; the
gate reports 5. Effective floor is 3, not 4. MEDIUM 3.

---

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `memory/primitive-wip.md:366-380` (instrument deleted) | **The batch's headline evidence is unreproducible, and CR 903.10a has no probe at all.** Six published numbers come from a deleted scratch `examples/dx22_p10.rs`; no committed code counts them. **Fix:** commit the instrument (or an `#[ignore]`d probe) and add a CR 903.10a assertion. |
| 2 | **HIGH** | `crates/simulator/src/invariants.rs:225-227` (+ audit row `OOS-DX22-3`) | **A universal negative over 426 violations is evidenced by 94 of them.** The binary prints per-violation detail for only the first five offending games. **Fix:** re-word to the measured scope, or obtain the full by-check tally. |
| 3 | MEDIUM | `crates/simulator/src/bin/fuzzer.rs:44-48`; `docs/mtg-engine-simulator.md:401-404` | **Numbers attributed to an instrument that cannot produce them.** `mtg-fuzzer --games 20 …` prints neither a first-cast turn nor a `CommanderCastFromCommandZone` count. **Fix:** attribute to the real instrument and state both denominators, as `legal_actions.rs:729` already does. |
| 4 | MEDIUM | `crates/simulator/tests/pb_dx22_fuzz_instrument.rs:582-643` | **P9 does not prove what its name claims** — a command-zone commander cast satisfies it. **Fix:** exclude the four pre-recorded command-zone `ObjectId`s from the journal match. |
| 5 | MEDIUM | `crates/simulator/tests/pb_dx22_fuzz_instrument.rs:269-315` | **P11's non-vacuity floor counts the gate file itself.** Effective floor 3, not 4. **Fix:** skip the gate's own path (or split the needle with `concat!`) and re-derive the floor. |
| 6 | MEDIUM | `docs/audits/decision-point-audit.md` row `OOS-DX22-8` | **Wrong CR subrule cited, and the violation is unclassified.** 704.5n is Equipment; the Aura rule is 704.5m. **Fix:** correct the cite, add the transient-vs-persistent question and the commander-zone-change hypothesis. |
| 7 | LOW | `crates/simulator/tests/pb_dx22_fuzz_instrument.rs:330-338` | **P5(b) is a reconstruction with no discrimination over P5(a)**, and its doc implies it gates `build_state`. **Fix:** name P11 + the 4b probe as `build_state`'s real gates. |
| 8 | LOW | `crates/simulator/tests/pb_dx22_fuzz_instrument.rs` (all probes) | **Every structural probe builds seed 1 only**; the CR 903.5c colourless-commander padding arm is never exercised. **Fix:** loop P2 over a seed known to draw a colourless commander. |
| 9 | LOW | `crates/simulator/src/fuzz_setup.rs:94,105` | **The O(defs × deck) linear scan `setup.rs::find_def` exists to avoid** was carried into the lift and is now paid by nine probes too. **Fix:** file, or adopt the `by_card_id` index in a successor. |
| 10 | LOW | `crates/simulator/src/fuzz_setup.rs:160-178` | **No `debug_assert_eq!(main_deck.len(), 99)`** where `setup.rs:375-379` has one; combined with `OOS-DX22-4` a short library is unobservable outside seed 1. **Fix:** add the assert or file it with `OOS-DX22-4`. |
| 11 | LOW | `crates/simulator/src/fuzz_setup.rs:134-137` | **"Both branches advance the stream identically" is imprecise** (`SliceRandom::shuffle` consumption is data-dependent under rejection sampling), and the fallback deck would redden P2. Harmless; unreachable. **Fix:** re-word to "nothing depends on cross-branch alignment". |
| 12 | LOW | `crates/simulator/src/fuzz_setup.rs:42-45` | **`FuzzGameSetup::decks` is the pre-shuffle list under an unqualified name.** **Fix:** rename to `decklists`, or expose `dealt` alongside. |
| 13 | LOW | `memory/primitive-wip.md:465-472` | **`tools/authoring-report.py` regeneration required by plan §7 was not executed**; coverage was asserted by construction. Sound here, but the plan mandated both. **Fix:** run it at collect, or record the by-construction argument as the deliberate substitution. |

---

### Finding Details

#### Finding 1: The headline evidence is unreproducible, and CR 903.10a has no probe

**Severity**: HIGH
**File**: instrument deleted; consumers at `crates/simulator/src/bin/fuzzer.rs:44-48`,
`crates/simulator/src/legal_actions.rs:728-732`, `docs/mtg-engine-simulator.md:398-407`,
`docs/audits/decision-point-audit.md` rows `OOS-DX22-1/-3/-7/-9`,
`crates/simulator/tests/pb_dx22_fuzz_instrument.rs:548-556`
**CR Rule**: 903.8, 903.9a, **903.10a** — "A player who's been dealt 21 or more combat damage by
the same commander over the course of the game loses the game."
**Issue**: Six numbers are published across five committed artefacts:
`CommanderCastFromCommandZone` 0 → **36** in 16/20 games; **13** CR 903.9a returns; non-empty
`commander_damage_received` in **16 of 20** games; **670** `SpellCast`; the 20-seed first-cast band
`[3,5,5,6,8,9,9,10,10,11,12,17,17,18,18,18,23,25,26,29]`; first `PlayLand` on turn **1-7** for all
20 seeds. All six were produced by a scratch `crates/simulator/examples/dx22_p10.rs` which the wip
records as **deleted** (`primitive-wip.md:366-369`), and the run's raw output was never committed.
`grep -n "CommanderCastFromCommandZone\|SpellCast" crates/simulator/src` returns **three comment
lines and zero code** — nothing in the shipped tree can re-derive any of them.

The asymmetry is the sharp part: the batch **committed its "before"**
(`memory/primitives/pb-dx22-measurement-head.txt`, present in the tree) and **discarded its
"after"**. The plan's §6 objection to P10 was specifically to a *non-ignored, statistical assertion
in the suite* — a committed-but-`#[ignore]`d instrument is not that, and would have cost nothing.

The consequence for acceptance criterion 2 is concrete. Criterion 2 requires commander mechanics
"exercised **or explicitly probed**". CR 903.6 is probed (P5), CR 903.8 is probed (P8), CR 903.9a is
probed (P7), CR 903.9b is probed (P6 + the 4b probe). **CR 903.10a is neither probed nor
reproducibly measured**: `grep -c "commander_damage" crates/simulator/tests/pb_dx22_fuzz_instrument.rs`
is 0 (the single `903.10` hit is P11's prose), and its only evidence is the deleted binary.

**Fix**: (a) commit the P10 instrument — either `crates/simulator/examples/dx22_p10.rs` or, better,
`#[ignore]`d probe `test_dx22_p10_commander_mechanics_are_exercised` in
`pb_dx22_fuzz_instrument.rs`, so `cargo test -- --ignored` re-derives all six numbers; and (b) add
a non-ignored CR 903.10a probe on the fuzz-built state — build → `start_game` → set up a registered
commander dealing combat damage (or, minimally, assert `commander_damage_received` is reachable and
keyed on `commander_ids`), proven red by deleting `player_commander`. **Settling check**: run
`grep -rn "CommanderCastFromCommandZone" crates/` on the shipped tree; every hit is a comment.

#### Finding 2: A universal negative over 426 violations, evidenced by 94

**Severity**: HIGH
**File**: `crates/simulator/src/invariants.rs:220-230` (shipped source doc), mirrored in
`docs/audits/decision-point-audit.md` row `OOS-DX22-3`
**Issue**: The shipped text reads, in bold: "**426 total violations, and not one of them is
`stack_consistency`.**" followed by the parenthetical "(The binary prints only the first five
offending games; those 94 printed lines are 90 `no_orphaned_tokens` + 3 `attachment_validity` + 1
`player_consistency`.)" I verified the sampling mechanism in the binary: with `--verbose`,
`print_game_result(result, false)` (`bin/fuzzer.rs:237`) prints the summary line only, and the
per-violation loop at `:243-253` is capped at `violation_seeds.len() <= 5`. So **332 of the 426
violations were never printed and never inspected**, and the bolded sentence asserts a property of
all 426 from a 22% sample. The parenthetical discloses the mechanism but the bold states the
conclusion, and the row's operative instruction — "Read a `stack_consistency` violation as a real
finding… The clean side is now evidence about games containing real spells" — is what a successor
will act on.

The same mechanism weakens two more sentences the batch shipped, and they should be corrected in
the same pass: "no new check class" at Stage 2 (`primitive-wip.md:230`) and "**Zero** occurrences of
this check in the stage-0 and stage-2 runs" in `OOS-DX22-8` — both are universal negatives over
sampled output.

For a batch whose thesis is that a measurement's *scope* must be stated, publishing an unsupported
universal into the very file whose SIM-3 caveat it was correcting is the defect class it exists to
remove. It costs zero game-state correctness, which is why it is not blocking; it is HIGH because
it will be cited as measured fact.

**Fix**: either (a) re-word all three sentences to their measured scope — "of the 94 violations the
binary printed (5 of the offending games), none is `stack_consistency`" — **or** (b) obtain the real
tally, which is cheap: run `--games 20 --seed 1 --max-turns 200 --threads 1 --verbose --profile
fuzz`, read the per-game violation counts, and `--replay <seed>` each offending seed (the replay
path calls `print_game_result(&result, true)` at `bin/fuzzer.rs:151`, which prints **all** of that
game's violations). Option (b) also settles Finding 6's transient/persistent question for free.
**Settling check**: `bin/fuzzer.rs:246` — `if violation_seeds.len() <= 5`.

#### Finding 3: Numbers attributed to an instrument that cannot produce them

**Severity**: MEDIUM
**File**: `crates/simulator/src/bin/fuzzer.rs:44-48`; `docs/mtg-engine-simulator.md:401-404`
**Issue**: `fuzzer.rs` says "Measured on `--games 20 --seed 1 --max-turns 200 --threads 1 --profile
fuzz`: avg turns 191.7 → 103.4, 9 wins / 11 `MaxTurnsReached` → 20 wins / 0 errors, first
`SpellCast` game turn 143-154 → a 3-29 band, `CommanderCastFromCommandZone` 0 → 36." The first two
pairs are genuinely from that command. The second two are not, and **cannot be**: the binary prints
no first-cast turn and counts no commander casts (verified: no such symbol exists in
`crates/simulator/src` outside comments). The "143-154" is the 5-seed pre-plan measurement
(`primitive-wip.md:33-37`); the "0" is from the same 5-game run (~56,800 commands); the "3-29" and
"36" are 20-game numbers from the deleted P10 instrument. Three different denominators are
presented as one A/B under one command name. `docs/mtg-engine-simulator.md:401-404` repeats it.
`legal_actions.rs:729-730` gets it **right** ("0 in ~56,800 commands over 5 games" → "36 casts
across 16 of 20 games"), which shows the batch knew the distinction and lost it at two of three
sites.
**Fix**: at both sites, name the real instrument (the P10 measurement, per Finding 1) and carry the
denominators the way `legal_actions.rs:729` does. **Settling check**: run the named command and
observe that neither number appears in its output.

#### Finding 4: P9 does not prove what its name claims

**Severity**: MEDIUM
**File**: `crates/simulator/tests/pb_dx22_fuzz_instrument.rs:620-632`
**CR Rule**: 903.8 — a commander is cast **from the command zone**, which is not the library.
**Issue**: The probe is named `test_dx22_a_spell_is_cast_at_an_ordinary_depth` and carries the
closure evidence for `OOS-UI2-1` and `OOS-SIM3-1`, both of which are library-order defects. It
matches `Command::CastSpell(_)` with no zone discrimination, and `CastSpellData`
(`crates/engine/src/rules/command.rs:775-777`) carries only `card: ObjectId` — so a commander cast
from `ZoneId::Command` satisfies it identically. The batch measured this from one side (shuffle
reverted ⇒ 3 of 4 seeds still green) and documented it fully at `:566-575`, which is good practice
and is why this is MEDIUM rather than HIGH. What is **not** measured is the shipped side: whether
seeds 1-4's observed first casts at turns 17/9/25/23 are library casts or commander casts. P10's
"first commander cast typically game turn 38-107" makes library casts likely, but the batch's own
rule is that likely is not measured.
**Fix**: capture the command-zone `ObjectId`s before `LocalGame::start`
(`setup.state.objects_in_zone(&ZoneId::Command(pid))[0].id` per seat — ids are stable across
`start_game`) and require the matched `CastSpellData.card` to be **outside** that set; keep the
turn-30 gate. This restores a single-variable probe for the shuffle. **Settling check**: with the
strengthened predicate, re-run the shuffle-only revert — it must now redden all four seeds, which
is what the plan predicted and the current probe cannot deliver.

#### Finding 5: P11's non-vacuity floor counts the gate file itself

**Severity**: MEDIUM
**File**: `crates/simulator/tests/pb_dx22_fuzz_instrument.rs:269-315` (needles at `:270-271`,
floor at `:310-315`)
**Issue**: The walk reads every `.rs` under `crates/simulator/{src,tests}` and matches on the
literal `in_zone(ZoneId::Command(`. The gate file **contains that literal** as its own `PLACEMENT`
const, and contains `player_commander` as its `REGISTRATION` const and in its panic message — so it
enters `matched` and passes the offender test **because of its own needle declarations**. Genuine
placing files post-fix are 4 (`src/setup.rs`, `src/fuzz_setup.rs`, `src/legal_actions.rs`,
`tests/commander_cast.rs`); the gate reports 5. The floor was set to ≥4 from a 5-file pre-fix census
that included `tests/local_game.rs` (which no longer places, correctly). Net: the floor is one
weaker than it reads — two genuine files could stop placing and the gate would still pass at
3 genuine + 1 self.
**Fix**: skip the gate's own file (`path.file_name() != Some("pb_dx22_fuzz_instrument.rs")`) or
break the self-match with `concat!("in_zone(ZoneId::", "Command(")`, then re-derive the floor from
the genuine census and state it (4). **Settling check**: with the exclusion in place the gate must
report exactly 4 matched files.

#### Finding 6: `OOS-DX22-8` cites the wrong CR subrule and does not classify the violation

**Severity**: MEDIUM
**File**: `docs/audits/decision-point-audit.md` row `OOS-DX22-8`; same text at
`memory/primitive-wip.md:331`
**CR Rule**: **704.5m** — "If an Aura is attached to an illegal object or player, or is not attached
to an object or player, that Aura is put into its owner's graveyard."
**704.5n** — "If an Equipment or Fortification is attached to an illegal permanent or to a player,
it becomes unattached from that permanent or player. **It remains on the battlefield.**"
**Issue**: The row cites 704.5n for "an Aura attached to an illegal or absent object is put into its
owner's graveyard as an SBA" — that is 704.5m's text under 704.5n's number. The two prescribe
**different dispositions**, so a successor told to look at 704.5n will look for a graveyard move
that 704.5n never performs, on an object type it may not be. Two further gaps in the row:
1. It does not ask whether the violation is **transient**. `check_attachment_validity`
   (`invariants.rs:399-414`) samples a state; CR 704.3 SBAs in this engine are checked on step entry
   and at resolution, not on every priority grant (`OOS-M11-7`), so a dangling `attached_to` between
   those points is *expected and self-healing*. Three samples at one turn is exactly that shape. It
   could therefore be a **false positive of the family SIM-3 found in `stack_consistency`** — the
   most valuable question to ask about a check firing for the first time, and the one this batch's
   own history should have prompted.
2. It misses a **commander-specific mechanism**. The batch's registration is what made commanders
   change zones in fuzz games at all (13 CR 903.9a returns in 20 games). A commander leaving the
   battlefield for the command zone is precisely the CR 400.7 event that orphans an attachment, so
   the hypothesis "ObjectId(677) was a commander that returned to the command zone" is both the
   cheapest to test and the one this batch uniquely enabled. The "0 engine lines" argument remains
   sound for *causation of the engine bug* and I am not disputing it.
**Fix**: correct the cite to 704.5m (keeping 704.5n if 532 may be Equipment, with both dispositions
named), and add the two questions above as the successor's first two checks. **Settling check**:
`cargo run --profile fuzz --bin mtg-fuzzer -- --replay 5 --players 4 --max-turns 200` prints all of
that game's violations (`bin/fuzzer.rs:151`); if the violation appears at exactly one turn number
and the game continues cleanly, it is the transient class.

#### Findings 7-13

Summarised in the table above; each is a one-line edit or a filing. The two worth a sentence more:

* **Finding 8** — plan §8 risk 8 explicitly designed P2 to be structure-independent *because* a
  colourless commander's deck is padded with colourless nonlands rather than basics
  (`deck.rs:128-142`). That design decision is untested: no probe ever builds such a seat. One extra
  seed in P2's loop closes it and would also exercise the only branch of `random_deck` that can
  return `None` after consuming RNG.
* **Finding 11** — the fallback branch (`fuzz_setup.rs:163-166`) is unreachable with `all_cards()`,
  but if it ever fired it would produce 99 identical `plains`, and P2's `assert_ne!` would fail as a
  false positive; it would also place **no commander at all** if `teysa-karlov` is absent from the
  pool, since `place_registered_deck`'s `if let Some(def)` guards both the object and the
  registration. Worth one sentence at the branch.

---

## CR Coverage Check

| CR Rule | Implemented? | Probed? | Measured? | Notes |
|---------|-------------|---------|-----------|-------|
| 103.3 shuffle | Yes (`fuzz_setup.rs:176`) | Yes | Yes | P2 (permutation), P3 (seed-determinism), P4 (seed-sensitivity) |
| 103.5 opening hand | **Deliberately not** | Yes — pinned absent | n/a | P1 asserts empty hand; `OOS-DX22-1`. Correct per §B2 |
| 903.6 commander placed + registered | Yes (`fuzz_setup.rs:94-101`) | Yes | Yes | P1, P5, P11; P11 was red on the pre-fix tree |
| 903.8 tax | Yes (via `commander_ids`) | Yes | Yes | P8, incl. an explicit `taxed != printed` floor |
| 903.9a zone-return SBA | Yes | Yes | Yes | P7; **proven independent of 903.9b** by revert B |
| 903.9b redirects | Yes (`fuzz_setup.rs:209`) | Yes | **0 occurrences** | P6 + the 4b probe; `OOS-DX22-9` records reachable-but-unreached |
| **903.10a commander damage** | Yes (via `commander_ids`) | **NO** | only by a deleted instrument | **Finding 1** |
| 903.5a 100-card / 903.4 identity | **No** — no `validate_deck` | n/a | n/a | `OOS-DX22-5`, correctly filed |
| 400.7 / 704.5m / 704.5n | engine, out of scope | n/a | 3 violations, seed 5 | `OOS-DX22-8`; **Finding 6** |

## Acceptance Criteria Check

| # | Criterion | Verdict |
|---|-----------|---------|
| 1 | Pre-plan measurement ran at HEAD, answer recorded **before** acceptance evidence | **MET, and exemplary.** Raw output committed at `memory/primitives/pb-dx22-measurement-head.txt`; the answer (offer *suppressed*, not late) resolved the brief's disjunction and reduced the batch's scope. |
| 2 | Shuffle from the game's own seeded RNG; `player_commander` in **both** sites; post-fix run demonstrating ordinary-depth casts and commander mechanics | **PARTIAL.** Shuffle and both registration sites: met, verified line by line against `setup.rs`. Ordinary-depth casts: met (P9, with the caveat of Finding 4). Commander mechanics: 903.6/903.8/903.9a/903.9b probed; **903.10a neither probed nor reproducibly measured** — Finding 1. |
| 3 | Every seeded pin **re-derived**, not adjusted; full suite `--workspace --no-fail-fast` to a file, residual empty | **MET.** No pin moved at all: `play-server` 78/0 (executed, plan §D's chain verified rather than assumed), `tests/local_game.rs` 23/0 unchanged. 4,356 / 0 / 5 over 42 targets, residual empty. Nothing was numerically adjusted anywhere. |
| 4 | PROTOCOL 35 / HASH 72 gate-executed and unmoved; 0 wire changes; coverage unmoved | **MET** (coverage by construction rather than by regeneration — Finding 13). Gates executed at Stage 0 **and** Stage 4/5, not predicted. |
| 5 | Close-out bookkeeping | **IN PROGRESS**, as reported. See below. |

## What criterion 5 must still contain

Reported as in progress and not failed here. From the plan and from what I found, it must carry:

1. **`memory/workstream-state.md` handoff** — absent today (the file's only PB-DX22 mentions are the
   pre-existing sequencing notes at `:2467`, `:2553-2555` and the dead-repro annotation at `:2904`).
   It must include the P10 numbers *with their instrument named as deleted* (Finding 1) so the
   successor is not misled into thinking `mtg-fuzzer` produces them.
2. **The plan's 10th correction site**, deliberately deferred: `memory/primitives/seed-rerank-2026-08-02.md`
   §2.4's "one open measurement this task could not settle" and the §4 PB-DX22 row. Plan §B4 required
   mirroring one line each; the wip records this as the coordinator's. It is the only unexecuted item
   from the §5 correction table.
3. **CLAUDE.md**: a NEW short bullet (never grow a line, per the 2026-08-02 formatting rule). It
   should say PROTOCOL **35** / HASH **72** unmoved, tests **4,356 / 0 / 5**, coverage unmoved at
   **1,133/1,803 = 62.8%**, 0 engine lines — and, in one clause, that **every fuzz seed recorded
   before this merge is dead** (`OOS-DX22-7`), because that is the fact most likely to be tripped
   over by the next reader.
4. **The three seed closures** (`OOS-SIM1-4`, `OOS-UI2-1`, `OOS-SIM3-1`) are already appended in-row
   in `docs/audits/decision-point-audit.md` §8.1 with a banner — I verified all three plus rows
   `OOS-DX22-1..11`. Nothing further needed there except Finding 2's and Finding 6's corrections.

## Method limits (stated plainly)

I had no shell in this session, so **every `git diff` claim in the wip is unverified by me** —
including "0 engine lines", the Stage-1 byte-identical fuzz-output diff, and "`setup.rs` doc lines
only". What I *can* say from reading: `crates/engine/src/state/builder.rs`,
`crates/engine/src/rules/command.rs`, `crates/simulator/src/invariants.rs`'s check bodies and
`crates/simulator/src/legal_actions.rs`'s command-zone loop all read as unmodified logic with
PB-DX22 content confined to comments, which is consistent with the claim. Coverage and the test
totals are likewise taken from the wip. A collector with a shell should spot-check
`git diff main..HEAD --numstat -- crates/engine/ crates/card-defs/` and
`git diff main..HEAD -- crates/simulator/src/{invariants,legal_actions,local_game,setup}.rs | grep -c '^[+-][^+-/ ]'`
(expect 0).

## Previous Findings

First review of this batch. No previous findings table.
