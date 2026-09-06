# LL-1 execution notes — `scutemob-255`

**Batch**: LL-1 — required `completeness:` marker gate (SR-39) + the six "its controller creates"
token defs. Source: `docs/mtg-engine-landscape-assessment.md` §2 / §9.
**Branch**: `feat/ll-1-required-completeness-marker-gate-fix-caster-recipient-`, forked at `168d8569`.
**Change class**: filed as rows 2 ("Card defs only") + 4 ("New source gate added"). **Promoted
mid-batch to rows 1 + 2 + 4** — see §2. Owner approved the promotion in a task comment before any
engine code was written.

---

## 1. Numbering: SR-38 → SR-39

The acceptance criterion says "SR-38". SR-38 was already taken: it is the simulator channel-probe
family (`crates/simulator/tests/cc15_raw_characteristics_ratchet.rs`). The gate therefore ships as
**SR-39**, and the criterion is satisfied by that number. Recorded in the gate's own doc comment
and in `docs/engine-invariants.md`'s new section.

---

## 2. The class promotion: why this stopped being a card-defs-only batch

The brief said: set `TokenSpec.recipient = PlayerTarget::ControllerOf(DeclaredTarget 0)` on six
defs; no engine changes; if you need one, STOP and post a task comment. I did that, and the answer
was that the card-only fix **does not work and makes things worse**.

**Probe, executed before writing anything else.** Patched `beast_within.rs` with the brief's
recipient line, then `SCRIPT_FILTER=002_beast_within cargo test -p mtg-engine --test scripts
run_all_scripts`. Result: `zones.battlefield.p2 actual=[]` — **no Beast token for anyone**. At HEAD
the bug was "the caster gets the 3/3"; with the recipient set and nothing else, nobody does.

Three source facts, read at `168d8569`:

1. `state/mod.rs` `move_object_to_zone` does `self.objects.remove(&object_id)` (CR 400.7). The
   destroyed permanent's `ObjectId` is dead before the second element of the `Effect::Sequence`
   runs.
2. `effects/mod.rs` `resolve_effect_target_list_indexed`'s `DeclaredTarget` arm returns `vec![]`
   for an id in neither `state.objects` nor `state.stack_objects` — CR 608.2b's partial-fizzle
   skip, and correct for every *object*-target consumer.
3. `PlayerTarget::ControllerOf` reads those same two collections. Empty target list → empty
   recipient list → `Effect::CreateToken`'s `for recipient in recipients` loop runs zero times.

`ctx.target_remaps` does not rescue it: it is written only by the Exile/MoveZone paths, never by
`DestroyPermanent`, and a remap would point at the **graveyard** object, whose controller
`move_object_to_zone` has reset to its **owner**.

**Two options were put to the owner, with a recommendation.** Option B (reorder the `Sequence` so
the token is created before the destroy) goes green today with no engine change, and was
recommended AGAINST: it is observably wrong when the destroyed permanent is the recipient's own
token doubler (Beast Within a p2 Doubling Season: real Magic makes one Beast, option B makes two),
i.e. exactly `project_legal_but_wrong_gap`'s class, and it would bury the engine gap under a green
script. The owner **refused B** and approved option A (the engine fix) with five conditions.

### 2a. Condition 1 could not be met as written, and I said so rather than substituting quietly

The approved shape was "the `ControllerOf` / `OwnerOf` arms consult
`GameState::lki_object_snapshot` for an id absent from objects/stack_objects". Implemented exactly
that; the script still failed. Cause: **the LKI store is selectively populated.**
`state/mod.rs` `capture_lki_snapshot` returns early unless the departing permanent carries
Wither / Infect / Deathtouch / Lifelink **or** is `is_source_of_a_pending_ability`. A destroyed Sol
Ring is none of those. That gate is deliberate and measured (SR-24,
`docs/sr-24-lki-capture-cost.md`), and PB-DX39's comment on it says outright that widening the
keyword list is not the way to serve a new non-keyword reader. There is even a chokepoint for this
exact moment — `core::pb_dx39_source_view_gates::r3` fails on an unlisted consumer of
`lki_objects` — whose purpose is to make a new reader check that the capture gate captures what it
intends to read. Here it does not.

**Substitute, posted as a task comment before the code and approved by continuing:**
`Effect::DestroyPermanent` records the departed permanent's `(controller, owner)` into
`EffectContext::departed_permanent_players`; `ControllerOf` / `OwnerOf` fall back to it when the
live path returns nothing. Both values were **already computed at that site** for the death events
(`pre_death_controller`, carrying the comment "CR 603.3a: capture controller before
move_object_to_zone resets it", and `owner` beside it) — the change records what the code already
held. In-repo precedent for the identical card text: Swan Song's "Counter target spell. **Its
controller** creates a 1/1 Bird" is served by `ctx.countered_spell_controller` +
`PlayerTarget::ControllerOfCounteredSpell`. This is that pattern for the destroy shape **without**
a new `PlayerTarget` sibling — the general form gains a fallback, which is the
"parameterize, don't proliferate" answer (`memory/conventions.md` → "## Landscape rules").

Scope discipline held: `resolve_effect_target_list_indexed` is untouched, so CR 608.2b's
partial-fizzle skip is unchanged for every object-target consumer. The fallback fires only for the
`EffectTarget::DeclaredTarget` shape and only after the live path returned nothing.

### 2b. Row 1's ritual, executed

**Wire prediction, stated in a task comment BEFORE the code**: no PROTOCOL bump, no HASH bump — no
variant added to `Effect` / `PlayerTarget` / `EffectTarget` / `ResolvedTarget`, no field added to
any hashed `GameState` type; `TokenSpec.recipient` and `PlayerTarget::ControllerOf` both already
existed (PB-EF2). **Confirmed after**: `core::protocol_schema` 17/17 and `core::hash_schema` 36/36
green with no constant touched.

**Revert-proven probe**, `primitives::ll1_token_recipient_controller_of`, executed three ways:

| revert applied to `effects/mod.rs` | t1 | t2 |
|---|---|---|
| none (shipped) | ok | ok |
| the CR 608.2h fallback returns `Vec::new()` | **FAILED** — "NO Beast token was created for anyone" | **FAILED** |
| the fallback returns the recorded `owner` instead of the `controller` (what reading the graveyard object gives) | ok | **FAILED** — got p4, expected p1 |

The third row is why `t2` exists as its own test: the cheap implementation passes `t1` and is
wrong, and only a fixture where owner and controller are different seats separates them.

**No hot-path file** (`layers.rs` / `sba.rs` / `priority.rs` / `combat.rs`) touched, so no bench.

**One container choice worth recording.** `departed_permanent_players` is a `BTreeMap`, not a
`HashMap`. It is `get`-only today so a `HashMap` would be sound, and `core::unordered_iteration_ratchet`
offered exactly that escape (raise the file's ceiling and say why). Ordered by construction costs
nothing at this size and removes the hazard the ratchet exists for — PB-DP9 re-executes a whole
resolution after a suspended choice and `RandomState` re-keys per map — from any future edit that
starts iterating it. Better than raising a ceiling and hoping.

---

## 3. SR-39: the gate

`crates/engine/tests/core/card_defs_completeness_marker.rs`, registered in `core/main.rs`
(SR-9a: under `core/`, never a top-level `tests/*.rs`).

**What it checks.** Every `crates/card-defs/src/defs/*.rs` except `mod.rs` must contain a
`completeness:` **field assignment** followed by `Completeness::<Variant>`, and that variant must
be one of the four `tools/authoring-report.py` recognizes (`Complete`, `inert`, `partial`,
`known_wrong`). Whitespace-blind between the colon and the path — byte-for-byte the report's
`MARKER_RE`, so the gate, the report and the coverage headline cannot disagree about what "has a
marker" means. A cross-check test compares the two recognized sets **in both directions**, so the
two copies cannot drift.

**The substring trap, found before it bit.** `misdirection.rs:36` reads "*Neither ever affected
this def's completeness: no card was …*" — prose, in a `//` comment, containing the exact
substring. A gate written as `text.contains("completeness:")` passes that file while it is
unmarked. That is one of the four canaries.

**Probes (change-class row 4).** Four throwaway-corpus canaries running the SHIPPED scanner —
no marker (RED), marker in prose only (RED), marker named (GREEN), capitalized `Completeness::Partial`
(RED as *unrecognized*) — plus **one executed defeat of the LIVE gate**, because a canary proves
the scanner works and only that proves the gate is wired to the corpus people edit:

```text
$ (deleted line 27 of crates/card-defs/src/defs/sol_ring.rs)
$ cargo test -p mtg-engine --test core card_defs_completeness_marker::every_card_def
test ...every_card_def_names_its_completeness_marker ... FAILED
SR-39: 1 card def(s) do not name `completeness:`. …
Unmarked:
  sol_ring.rs
test result: FAILED. 0 passed; 1 failed; 0 ignored; 838 filtered out
```

Line restored, gate green. Transcribed in the test's own doc comment, which is where row 4 says to
record it.

**Pair-or-demote.** SR-39's subject is a property of the source itself, like SR-5 and SR-36, so it
is in the exempt class: there is no behaviour a probe could observe, because an unmarked def and an
explicitly-`Complete` one compile to the identical `CardDefinition`. That is not asserted in prose —
`deliberateness_is_invisible_at_runtime` PINS it (identical serialized values, identical
`validate_deck` verdict) and carries the control showing a `partial` marker *does* change the
verdict (Architecture Invariant 9).

---

## 4. The sweep

964 defs had no marker and were `Complete` by `#[derive(Default)]`. All 964 now say so. The sweep
is behaviour-preserving **by construction**: it writes down the value the compiler was already
choosing. No marker outside the six defs in §5 was re-judged.

The insertion point is identified **structurally**, not by "the last match": `completeness` is
`CardDefinition`'s last field, so the marker goes immediately before the outer literal's
8-space-indented `..Default::default()`, which must be the third-from-last line of the file,
followed by `    }` and `}`. Five defs open with a `let` binding whose own `..Default::default()`
also sits at 8 spaces (`chained_to_the_rocks.rs` and four siblings); the tail check separates them.
Any file not matching the shape is **REFUSED, not guessed at**. Result: **964 swept, 839 already
explicit, 0 refused**, one line added per file and nothing else (`964 files changed, 964
insertions(+)`).

Two facts the brief asked for:

- The "extra" file in `ls defs/*.rs` (1,804 vs 1,803) is **`mod.rs`**.
- `misdirection.rs` contains the string `completeness:` in prose while carrying no marker — which
  is why every consumer here keys on the field assignment.

The script, verbatim:

```python
#!/usr/bin/env python3
"""LL-1 (scutemob-255) one-shot sweep: name `completeness:` explicitly on every def."""
import re, sys, pathlib

DEFS = pathlib.Path("crates/card-defs/src/defs")
MARKER_RE = re.compile(r"\bcompleteness:\s*Completeness::(\w+)")
TAIL = ["        ..Default::default()", "    }", "}"]
INSERT = "        completeness: Completeness::Complete,"

def main() -> int:
    swept, already, refused = 0, 0, []
    for path in sorted(DEFS.glob("*.rs")):
        if path.name == "mod.rs":
            continue
        text = path.read_text(encoding="utf-8")
        if MARKER_RE.search(text):
            already += 1
            continue
        lines = text.split("\n")
        while lines and lines[-1] == "":
            lines.pop()
        if lines[-3:] != TAIL:
            refused.append((path.name, lines[-3:]))
            continue
        lines.insert(len(lines) - 3, INSERT)
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        swept += 1
    print(f"swept {swept}, already explicit {already}, refused {len(refused)}")
    for name, tail in refused:
        print(f"  REFUSED {name}: tail={tail}")
    return 1 if refused else 0

if __name__ == "__main__":
    sys.exit(main())
```

---

## 5. The six defs

Oracle text verified for all six through the `mtg-rules` MCP `lookup_card`, not from memory.
`recipient: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))` — each
def's `targets:` vec has exactly one entry, so index 0 is unambiguously the destroyed permanent.

| def | before | after | why |
|---|---|---|---|
| `beast_within.rs` | `Complete` (by default) | `Complete` | the live bug; recipient set. Both clauses modelled. |
| `generous_gift.rs` | `Complete` (by default) | `Complete` | same. |
| `stroke_of_midnight.rs` | `partial` | **`Complete`** | its blocker note ("CreateToken has no player field") was STALE — `TokenSpec.recipient` has existed since PB-EF2. Note + TODO block deleted. |
| `emergency_eject.rs` | `partial` | **`Complete`** | see below. |
| `pongify.rs` | `known_wrong` | **`Complete`** | same stale blocker, stated as `known_wrong`. Both the KI-8 note and the marker deleted. |
| `saw_in_half.rs` | `partial` | `partial` | STAYS. Its blocker is real and unrelated: "creates two tokens that are copies of that creature, except their power/toughness are each half. Round up each time" has no modelled counterpart. |

**`emergency_eject.rs` is the one that needed proving, not asserting.** Its own `partial` note said
the ONLY blocker was the token recipient and that a Lander "IS expressible today". Taking a note at
its word and flipping a marker to `Complete` makes the card **deck-legal**, and CLAUDE.md calls a
guessed `completeness:` the single most prohibited pattern in this repo. So the Lander was authored
(artifact token, `activated_abilities` carrying `{2}, {T}, Sacrifice this token: Search your library
for a basic land card, put it onto the battlefield tapped, then shuffle`) **and executed end to
end** by `ll1_token_recipient_controller_of::t3`: at a four-player table the Lander reaches the
destroyed permanent's controller, that player activates it, the cost is paid (the token leaves the
battlefield on activation, CR 601.2h), the search is answered, and a basic land arrives **tapped**
under their control. A token ability authored into a `Complete` def and never run is SR-36's hazard
with a deck-legality stamp on it.

**`saw_in_half.rs`'s note is a bounded claim** per `memory/conventions.md` → "Never write
'unsupported' without naming the population you searched": `Effect` at `17fc2834` (106 variants),
the three copy-token primitives named with their fields, and `EffectAmount` at the same sha (26
variants) having `PowerOf`/`ToughnessOf` but no halving arithmetic to feed one. The same gap used
to be stated three times in that file (header comment, in-struct comment, marker); only the marker
is machine-read, so the other two were removed rather than maintained.

### Marker tally

| | before | after |
|---|---|---|
| `Complete` | 1,140 (176 explicit + **964 defaulted**) | **1,143 (all explicit)** |
| `partial` | 415 | 413 |
| `inert` | 147 | 147 |
| `known_wrong` | 101 | 100 |
| **defaulted** | **964** | **0** |
| total | 1,803 | 1,803 |

---

## 6. Pins

- `test-data/generated-scripts/tokens/002_beast_within_creates_beast.json` — `retired` →
  **`approved`**, `retirement_reason` deleted. That reason was stale twice over: it claimed the
  destroy did not resolve and Sol Ring survived. The destroy worked; what failed was the token's
  recipient, which is exactly what this script's `zones.battlefield.p2` assertion pins. Approved
  script count 208 → **209**.
- `stack/047_beast_within_destroys_land.json` and `stack/048_generous_gift_destroys_artifact.json`
  — **blessed-bug assertions, corrected.** Both scripts' own titles, descriptions and generation
  notes always said the token goes to p2; their ASSERTIONS read `zones.battlefield.p1` and their
  step notes said "p1 (caster) gets the 3/3". The assertion had been bent to the engine's defect
  rather than the printed card. Restored to p2.
- `primitives::ll1_token_recipient_controller_of` — t1 (four players, caster ≠ target's controller),
  t2 (controller ≠ owner ≠ caster, three distinct seats), t3 (Emergency Eject + the Lander,
  executed).
- **CR citations corrected while re-approving.** Verified through the `mtg-rules` MCP: CR 701.6a is
  **Counter**, CR 701.7a is **Create**, CR 701.8a is **Destroy**. All three scripts had them
  transposed (`701.6a` for creating a token, `701.7` for destroying a permanent) and the batch's own
  first draft copied the error forward. Fixed in the three scripts, `beast_within.rs` and the new
  test. `608.2h` added to each script's `cr_sections_tested`.

---

## 7. The corpus re-deal (`OOS-CARDS2-3`), and its ablation

Three markers flipped to `Complete`, so `CORPUS_COMPLETE` moved **1,140 → 1,143**,
`deck.rs::random_deck` draws its commander and colour-identity pool from `all_cards()`, and every
seeded fixture in the workspace deals a different game. Eight tests reddened in six files.

**Attributed by an EXECUTED ablation, not argued.** With the whole engine change in the tree and
ONLY the three promoted markers forced back to non-`Complete` (pool back to 1,140), every one of
them is green:

| file | under ablation |
|---|---|
| `simulator/tests/pb_dx22_fuzz_instrument.rs` | 12/12 |
| `simulator/tests/pb_dx32_fuzz_output.rs` T6.3 | ok |
| `simulator/tests/sim5_bot_cast_discipline.rs` | 6/6 |
| `tools/play-server` | 130/130 |

So **the engine half of LL-1 is fuzz-neutral by measurement**; all of the movement is the re-deal.
Every pin below was re-observed by running, never hand-edited to match a diff.

| pin | old | new | how |
|---|---|---|---|
| `pb_dx32` `CORPUS_COMPLETE` | 1140 | **1143** | direct count; `COMMANDER_POOL` re-measured by EXECUTING the gate and UNCHANGED at 90 (all three promotions are Instants — measured, not reasoned, per PB-DX26's lesson) |
| `pb_dx32` T4.1/T4.3 orphan-token seed | 162 | **18** | re-swept 0..=399 at the unchanged 4-player/25-turn config. Hits: 18 (raw 4 / distinct 1), 64 (4/1), 163 (5/1), 164 (4/1), 186 (4/1), 335 (4/1) — all all-orphan, 0 hard violations, 0 leaks. 18 is the smallest and reproduces the raw-4/distinct-1 shape exactly (one orphan at turn 22, four priority checkpoints). |
| `pb_dx22` P2 colourless-commander seed | 50 | **40** | re-swept 1..=200. Hits with exactly one colourless seat: 40, 73, 119, 128, 132, 163, 175, 182, 200 — all `rograkh-son-of-rohgahh`, still the only `Complete` colourless legendary creature. 40 is the smallest; its seat 3 is the colourless one, so the non-vacuity assertion stays an EQUALITY. |
| `pb_dx22` P13 census seed | 6 | **0** | re-swept 0..=40 against every assertion the test makes. 0 is the first and smallest. Strictly richer than the seed-6 tally it replaces (2 command-zone casts vs 1, 3 seats dealt commander damage vs 1, plus a commander return seed 6 never produced). |
| `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` | 1 | **2** | re-measured on all three A/B seeds: 2 / 0 / 0. The wasted-tap dimension did NOT move (`wasted_taps: 0` on all three, as before); what changed is which spells seed 0's bots get to cast. Raised rather than re-tuning the seed set, which is the move the gate's siblings tell you not to make. |
| play-server exact-hand pin | Sol Ring / Simic Initiate | **Slither Blade / Signal Pest** | read off a real run |
| play-server `secrets.len()` | 20 | **16** | read off a real run |
| `completeness_deviation_scan` marker floor | 663 | **660** | re-measured directly: 1,143 Complete / 660 non-Complete of 1,803 |

**T6.3's decision-coverage partition moved and is REPORTED, not re-tuned** (the gate's own
instruction): `discard_cards` LEAVES the reached set, 6 of 7 → 5 of 7
(`{look_at_top_then_place_optional, scry, search_library, surveil, triggered_targets}` reached;
`{discard_cards, may_pay_then_effect}` never reached). Attributed by the ablation above — nothing
about the engine's willingness to SERVE the row changed, `decision_site_walk`'s partition is
untouched, and `discard_cards`'s reachable sources simply are not drawn at this budget any more.

---

## 8. Review findings

Batch review of the six defs: `memory/primitives/ll-1-card-review.md` (0 HIGH, 1 MEDIUM, 4 LOW).

- **MEDIUM F1 — fixed in-cycle.** `beast_within.rs` cited `CR 701.6a` for creating a token. 701.6a
  is **Counter**. Grep-verified against the CR through the `mtg-rules` MCP, exactly as
  `memory/conventions.md` requires for 701.x/702.x numbers. Corrected to 701.7a (Create) / 701.8a
  (Destroy) — and the same sweep caught the pre-existing transposition in all three golden scripts.
- **LOW F2 — fixed** (the same miscitation propagated into the new test file).
- **LOW F3 — fixed** (`saw_in_half.rs`'s bounded claim now carries the `at 17fc2834` anchor the
  conventions template asks for, so the variant count cannot silently rot).
- **LOW F4 — fixed** (the same gap stated three times in `saw_in_half.rs`; only the marker is
  machine-read, so the other two copies were removed).
- **LOW F5 — LOGGED, not fixed.** The Lander is a CR 111.10u predefined token authored inline
  rather than as a `lander_token_spec()` helper beside `treasure_token_spec` / `food_token_spec` /
  `clue_token_spec` in `card_definition.rs`. Structural only — every characteristic is correct and
  executed. A new predefined-token helper is a `card-types` change with corpus-wide reach and no
  second user today; it belongs to whichever batch authors the second Lander card.

---

## 9. What this batch did NOT fix (a real, adjacent gap), now with its population named

`PlayerTarget::ControllerOf` on a **remapped** declared target still answers the OWNER, not the
controller. When an Exile/MoveZone effect tracks its target into `ctx.target_remaps`, the live path
resolves the NEW object, and `move_object_to_zone` reset that object's controller to its owner
(`state/mod.rs`). Same CR 608.2h family as the bug this batch closed, different trigger, and fixing
it changes existing card behaviour well outside this task. **Deliberately out of scope; recorded
here so the next reader finds it.** It was named in a task comment before the code, not discovered
afterwards.

The census in §11 sizes it: **three defs** pair `ControllerOf` with an exile rather than a destroy —
`swords_to_plowshares.rs` (**`Complete`**, "its controller gains life equal to its power"),
`reality_shift.rs` (**`Complete`**) and `path_to_exile.rs` (`known_wrong`, for an unrelated
reason). The two `Complete` ones are wrong only when the exiled permanent's controller differs from
its owner — you exile a creature you had gained control of, and the life or the land goes to the
player who owns the card. Narrower than the destroy bug (which was wrong at every multiplayer
table) but the same shape, and it is a seed a future batch should file rather than rediscover.


---

## 10. Suite bookkeeping (change-class row 2)

`cargo test --workspace --no-fail-fast` to a file, twice: once mid-batch (which is how the eight
re-deal failures in §7 were found) and once at the end.

**Baseline at `168d8569`: 5,333 passing / 0 failing / 6 ignored, 73 targets.**
**Final: 5,346 passing / 0 failing / 6 ignored, 73 targets.** Delta **+13, zero removed**,
itemised by test NAME (the last two were added by the `/review` fix cycle, §11):

| test | file |
|---|---|
| `card_defs_completeness_marker::every_card_def_names_its_completeness_marker` | the SR-39 gate |
| `card_defs_completeness_marker::the_completeness_gate_is_not_vacuous` | its denominator |
| `card_defs_completeness_marker::the_recognized_set_matches_the_authoring_report` | anti-drift, both directions |
| `card_defs_completeness_marker::gate_catches_a_def_with_no_completeness_marker` | canary (RED) |
| `card_defs_completeness_marker::gate_catches_a_def_whose_only_marker_is_prose_in_a_comment` | canary (RED) — the `misdirection.rs` trap |
| `card_defs_completeness_marker::gate_passes_a_def_that_names_its_marker` | canary (GREEN) |
| `card_defs_completeness_marker::gate_catches_an_unrecognized_marker_spelling` | canary (RED as unrecognized) |
| `card_defs_completeness_marker::deliberateness_is_invisible_at_runtime` | why SR-39 must be a source scan |
| `ll1_token_recipient_controller_of::t1_beast_lands_with_the_destroyed_permanents_controller_at_four_players` | CR 701.7a at N=4 |
| `ll1_token_recipient_controller_of::t2_controller_not_owner_when_the_permanent_was_under_someone_elses_control` | CR 608.2h vs CR 400.7 |
| `ll1_token_recipient_controller_of::t3_emergency_eject_lander_reaches_the_targets_controller_and_actually_works` | the Lander, executed |
| `card_defs_completeness_marker::gate_catches_a_commented_out_marker` | canary (RED) — `/review` LOW 1 |
| `ll1_token_recipient_controller_of::t4_natures_claim_gains_life_for_the_destroyed_permanents_controller` | `/review` MEDIUM — the `Complete` def with a dead clause |

Golden scripts: 208 approved → **209** (`tokens/002` re-approved). The partition gate
(`scripts::run_all_scripts`) is green: 45/45.

Other gates: `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --check`
clean; `tools/check-defs-fmt.sh` reports 1803 defs checked, clean; `core::protocol_schema` 17/17
and `core::hash_schema` 36/36 green with no constant touched (the wire prediction, met).
`docs/authoring-status.md` regenerated **once** (the script overwrites its Δ snapshot every run, so
a second run would erase the delta): clean 1,140 → **1,143 (63.4%)**, TODO 516 → 513, empty 147
unchanged — moved only by §5's three promotions, exactly as criterion 2 requires.


---

## 11. `/review` findings (Opus reviewer, after the four implementation commits)

All five criteria PASS; every change-class obligation PASS. The reviewer re-executed both classes
of defeat itself — the SR-39 live-corpus defeat and both CR 608.2h reverts — and reproduced the
transcripts verbatim, restoring the tree byte-exact. Its own gate run matched the pre-fix tree exactly: 5,344 / 0 / 6 over
73 targets (5,346 after the two tests its own findings added), clippy and fmt clean, and an independent marker count of
`{Complete: 1143, partial: 413, inert: 147, known_wrong: 100}`, 0 unmarked.

**1 MEDIUM + 4 LOW. Four fixed in-cycle, one recorded.**

### MEDIUM — the fix repairs ELEVEN defs, not six, and two of the extras were `Complete`

The reviewer found that the engine half silently repairs five defs the brief never named. **Census
re-run to confirm it, and to name the population rather than take the number on trust**: of 1,803
defs, **21** mention `PlayerTarget::ControllerOf`. Eleven pair it with `Effect::DestroyPermanent`
over `DeclaredTarget { index: 0 }` in one `Effect::Sequence` — the shape whose second clause
resolved to an empty player list:

| def | marker | dead clause |
|---|---|---|
| the six in §5 | — | "its controller creates <token>" |
| `natures_claim.rs` | **`Complete`** | "Its controller gains 4 life" |
| `boseiju_who_endures.rs` | **`Complete`** | "its controller may search their library for a basic land" |
| `assassins_trophy.rs` | `known_wrong` (unrelated reason) | the same search clause |
| `ghost_quarter.rs` | `known_wrong` (unrelated) | same |
| `sundering_eruption.rs` | `partial` (unrelated) | same |

The remaining ten `ControllerOf` users are unaffected: six resolve it against a LIVE object
(`fecundity`, `mesmeric_orb`, `massacre_wurm`, `magmatic_hellkite`, `edric_spymaster_of_trest`,
`demolition_field`), two use `ControllerOfCounteredSpell` (`swan_song`,
`an_offer_you_cant_refuse`), and three pair it with an EXILE — the separate, still-open defect
in §9.

Fixed in-cycle: `t4_natures_claim_gains_life_for_the_destroyed_permanents_controller` pins it (p3
gains the 4, p1 gains nothing), and it is revert-proven alongside the other three — the full
defeat matrix in the test's header now covers t1–t4. `natures_claim` earns the pin because it is
the worst of the five: `Complete`, deck-legal, no TODO, and its only non-destroy clause was dead.
The assessment §2 framing ("that is two deck-legal wrong cards") is corrected to four, with the
observation that **`natures_claim`'s marker was explicit and wrong** — SR-39 closes the "nobody
decided" class, and this is the "someone decided, incorrectly" class, which no source gate can
reach. The aspirational comment at `natures_claim.rs:17-18` ("Using ControllerOf(DeclaredTarget{0})
to **correctly** give life to the destroyed permanent's controller") became true only at
`17fc2834`; it now says so.

### LOW 1 — a commented-out marker defeated SR-39. FIXED.

`// completeness: Completeness::Complete,` contains both halves the scanner looked for, so the
prose canary could not catch it: a def carrying only that line was unmarked and looked marked.
`OOS-DX32-6`'s shape one gate over. `markers_in` now requires the match to be **line-leading**,
with a fifth canary (`gate_catches_a_commented_out_marker`) pinning it. Line-leading rather than
stripping `//` comments first — which is what `bare_lookup_ratchet.rs` does — because this
corpus's string literals contain `//` for real (every split card's name and `oracle_text`:
`"Cut // Ribbons"`), so a comment stripper would need to understand string literals to be safe
and a position test does not. rustfmt plus SR-35 make it free: all 1,803 markers are line-leading
today.

### LOW 2 — SR-38 had no entry in `docs/engine-invariants.md`. FIXED.

The doc jumped SR-37 → SR-39, so the numbering-correction paragraph was the only mention and a
reader following it found nothing. It now says what SR-38 is (the play-server channel-probe
family) and why it does not live in that file.

### LOW 3 — the insert could in principle record `(caster, caster)`. FIXED.

`pre_death_controller` falls back to `ctx.controller` when the object lookup misses — right for
the death events, exactly wrong here, because it would name the CASTER as the departed permanent's
controller and hand the token straight back to the player this batch took it away from.
Unreachable today (the indestructible read above already proved the object exists), which is
precisely why it is worth closing structurally rather than trusting that ordering to survive the
next edit. The insert is now guarded on `state.fizzle_object(id).is_some()` — through the
diagnostics vocabulary rather than a bare `state.objects.get`, because SR-4 requires new code in
this file to say which kind of absence it tolerates, and an absent id here is a rules-correct
nothing-to-record.

### LOW 4 — `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED` 1 → 2 is the one relaxation. RECORDED.

Not reverted: it is ablation-attributed, re-measured on all three A/B seeds, and the wasted-tap
dimension is separately pinned unmoved. But the constant now carries a written stop: another raise
is the wrong move, the right one is closing `OOS-SIM2-1` (which drops it to 0 and retires the
constant), and a batch that wants 3 must file that seed or say in writing why the slack widened.

### Recorded, not fixed (the reviewer's own "non-issue worth knowing")

`Effect::DestroyAll` shares the destroy pipeline but records no departures. Correct today — no def
pairs `DestroyAll` with `ControllerOf`, and the §11 census is what says so rather than an
assumption — but it is an asymmetry the first "destroy all X; their controllers …" card will trip
over. Left alone deliberately: writing a per-object map from a mass-destroy for zero current
readers is speculative machinery, and the census above is the cheap way to notice when it stops
being speculative.
