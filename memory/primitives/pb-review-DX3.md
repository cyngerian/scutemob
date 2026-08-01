# PB-DX3 Review — `garruks_uprising` + `inventors_fair` (OOS-DP6-3)

<!-- last_updated: 2026-08-01 -->

- **Task**: `scutemob-164` · **Branch**: `feat/pb-dx3-two-stale-blocker-notes-garruksuprising-inventorsfair`
- **Reviewed commit**: `f3e92ecc`
- **Plan**: `memory/primitives/pb-plan-DX3.md`
- **Reviewer**: Opus, read-only, via `/review`. Independently re-ran every gate and re-derived
  every §1 premise cite rather than trusting the plan's transcription; independently looked both
  cards up through MCP `lookup_card` and found the plan's oracle-text and ruling transcriptions
  **byte-identical** — no drift.
- **Verdict**: **1 MEDIUM / 5 LOW, 0 HIGH.** Both `partial → Complete` flips are **justified**,
  clause-by-clause, against oracle text and all eight rulings. All four hard gates green.

> A first review attempt (`card-batch-reviewer`) terminated without writing its findings file.
> The review recorded here is the `/review` Opus pass, which covers the same checklist plus the
> acceptance criteria. Noted because "the review ran" is a claim like any other.

---

## §1 Criteria assessment

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | Premise re-verified; both cards MCP-verified before flip | **PASS** | `Condition::YouControlNOrMoreWithFilter` at `card_definition.rs:3834`; queue-time-evaluable arm at `effects/mod.rs:10151`; evaluator at `:10201` (scopes to `obj.controller == controller`, `is_phased_in()`, layer-resolved chars via `expect_characteristics`, honours `exclude_self`). PB-DP6 queue sites live: ETB `replacement.rs:2131`, upkeep `turn_actions.rs:310`. Resolution re-check `resolution.rs:2337-2352`. `activation_condition` enforced `abilities.rs:257-292`. |
| 2 | `garruks_uprising` intervening-if + fail-before probe + flip | **PASS** | Def `garruks_uprising.rs:35-42`; flip `:78`. T1 genuinely fail-before on its **stack** assertion; T2 is the exact-structure positive control. |
| 3 | `inventors_fair` both corrections + probes + flip | **PASS** | Upkeep trigger `inventors_fair.rs:24-41`; `activation_condition` `:89-95`; flip `:101`. T5/T6/T7 (both CR 603.4 ends), T8 (message-asserted rejection), T9 (end-to-end `EffectChoiceRequired`), T10 (no resolution-time re-check). |
| 4 | Zero engine lines; wire unmoved; gates green | **PASS** | `git diff main --stat -- crates/engine/src crates/card-types/src` **empty**. PROTOCOL 32 (`protocol.rs:335`) / HASH 69 (`hash.rs:679`). `cargo test --all` **3,998 / 0** across 31 binaries. clippy/fmt/`check-defs-fmt.sh` clean. **PB-DP10's `decision_gate` suite green** — the two new `Complete` defs add no unrecorded engine-made choice. |
| 5 | Close-out bookkeeping | **PENDING at review time** | `docs/authoring-status.md` regenerated (1,140 → **1,142**, 63.2% → 63.3%; TODO lines 947 → 942). The rest is the close-out commit. |

---

## §2 Flip verdicts

### Garruk's Uprising — `Complete` **JUSTIFIED**

| Printed clause | Def | Verdict |
|---|---|---|
| `{2}{G}` Enchantment | `:12-17` | ✓ |
| "When this enchantment enters, **if you control a creature with power 4 or greater**, draw a card." | `:23-47` | ✓ `matches_filter` (`effects/mod.rs:9496`) checks `min_power` against `chars.power` and treats `None` as **failing** (`:9502-9506`), so a `*/*` CDA creature correctly does not count until layers give it power. The `YouControlNOrMoreWithFilter` arm supplies controller-scoping itself, so the filter's unused `controller` default is inert; it reads `expect_characteristics`, so anthems and pumps count. **The enchantment cannot wrongly satisfy its own filter** (not a Creature, and `power: None` fails `min_power` — two independent reasons), and cannot wrongly fail it, so omitting `exclude_self` is right. Both CR 603.4 ends fire. |
| Ruling 2024-11-08 #2 ("just one card") | `EffectAmount::Fixed(1)` `:28` | ✓ |
| "Creatures you control have trample." | `:48-56` | ✓ |
| "Whenever a creature you control with power 4 or greater enters, draw a card." | `:57-76` | ✓ Verified end to end: `build_face_ability_vectors` (`replay_harness.rs:2891-2952`) forwards the full `TargetFilter` as `triggering_creature_filter`, and `abilities.rs:6957-6971` applies `matches_filter` against **layer-resolved** entering-creature chars, so `min_power` and `controller: You` are genuinely enforced (ruling #3). `intervening_if: None` here is **correct** per ruling #4 — once triggered, lowering the power does not stop the draw. |

### Inventors' Fair — `Complete` **JUSTIFIED** (one pre-existing engine gap noted, LOW-4)

| Printed clause | Def | Verdict |
|---|---|---|
| Legendary Land | `:12` | ✓ |
| "At the beginning of your upkeep, **if you control three or more artifacts**, you gain 1 life." | `:24-41` | ✓ Gated at queue time (`turn_actions.rs:310`, controller == active) and re-checked at resolution (`resolution.rs:2337-2352`) — rulings #1 **and** #2 both satisfied. The land is not an artifact, so it never self-counts, matching the printed card. Layer-resolved, so animated artifacts count. |
| "{T}: Add {C}." | `:43-55` | ✓ `helpers.rs:77` signature is `(w,u,b,r,g,colorless)` → 1 colorless. Lowered into `mana_abilities` and **excluded** from `activated_abilities` (SR-34/SF-6, `replay_harness.rs:2440-2478`). |
| "{4}, {T}, Sacrifice Inventors' Fair: …" | `:58-65` | ✓ T9 pins that the sacrifice is paid at **activation** (CR 602.2c), not at resolution. |
| "Search …, reveal it, put it into your hand, **then shuffle**." | `:66-83` | ✓ on ordering — `shuffle_before_placing: false` + a trailing `Effect::Shuffle` is correct; `shuffle_before_placing: true` is the *opposite* pattern (Vampiric Tutor, `effects/mod.rs:3637-3644`). The trailing unconditional `Effect::Shuffle` also correctly shuffles on a fail-to-find. ✗ on `reveal` — LOW-4. |
| "Activate only if you control three or more artifacts." | `:89-95` | ✓ **Correct CR placement.** `abilities.rs:257-292` evaluates `activation_condition` inside `activate_ability`'s pre-payment block only; no second read exists on any resolution path. Ruling 2016-09-20 #3 satisfied, and T10 pins it. An `Effect::Conditional` wrapper would have been **wrong**. |

---

## §3 Probe vacuity

- **T9 `candidates[1]` vs `candidates[0]` — not a gap.** Candidates are provably in **ascending
  `ObjectId` order**: collected by iterating `state.objects` (an `imbl::OrdMap`), and
  `effects/mod.rs:3584-3590` both documents it ("`candidates[0]` IS the old auto-pick") and
  enforces it with a `debug_assert!` over `windows(2)`. With `candidates.len() == 2` asserted,
  `candidates[1]` is strictly greater and therefore provably **not** the pre-PB-DP9
  lowest-`ObjectId` auto-pick. Non-vacuous as written; would read better if it said so.
- **T9/T10 `ability_index: 0` — right by construction, and separately pinned observably.**
  `build_face_ability_vectors` never puts an `AbilityDefinition::Triggered` into
  `activated_abilities`, and skips any ability `mana_ability_lowering` claimed
  (`replay_harness.rs:2455-2461`, explicitly to avoid index shift — SF-6). Inventors' Fair's
  `activated_abilities` therefore holds exactly one entry, the search ability, both before and
  after the upkeep trigger was inserted at def-index 0. Independently, T8 asserts an error string
  only `activation_condition` produces and T9 asserts the land reaches the graveyard — neither is
  reachable via the mana ability. **Not accidental.**
- **T5 vacuity / is T6 sufficient? — yes.** T5 and T6 use the same `inventors_fair_fixture`
  differing only in `num_artifacts` (2 vs 3), so together they are a controlled comparison. T6 is
  itself genuinely fail-before (pre-fix: no such ability at all, 0 life gained).
- **T4** — non-vacuous on identity (`WhenEntersBattlefield` is never lowered into runtime
  `triggered_abilities`, so the only trigger carrying `source == Garruk's Uprising` on a foreign
  ETB event is the third ability) but weak on content. See LOW-2.
- **T1** — the stack assertion is load-bearing and genuinely fail-before; the **hand-count**
  assertion is vacuous. See MEDIUM-1.
- **T2, T3, T6, T7, T8, T10** — all non-vacuous; each has a positive/negative counterpart or an
  assertion on a message or state reachable only through the intended mechanism.
- **Ability-index churn** — confirmed safe. A repo-wide search for `inventors_fair` /
  "Inventors' Fair" outside `target/` and `.git/` hits only the def, the new test, two
  `test-data/test-cards/*.json` **name lists** (no ability indices), and prose. No golden script,
  fixture or engine site references the card.

---

## §4 Findings

### MEDIUM-1 — a false "pre-fix observation" for T1, and a vacuous companion assertion

`pb_dx3_stale_blocker_notes.rs:45-50` records that pre-fix "the post-resolution hand count was
**1**, not 0". T1's fixture (`:274-284`) builds **no library objects**, and `GameStateBuilder`
inserts an empty `Zone::new_ordered()` per library (`state/builder.rs:286`) — T2 and T3 add an
explicit `"Library Filler"` precisely because of this. A pre-fix wrongful draw would therefore
have hit an empty library, which is **not** a no-op: `replacement.rs:1035-1049` sets
`has_lost = true` and emits `PlayerLost`, leaving the hand at 0. The recorded observation is
not reproducible, and the live assertion at `:318-322` (`hand_count == hand_before`) passes
whether or not the trigger fires.

No correctness impact — the stack assertion at `:311-316` carries T1 — but this is exactly the
doc-vs-code honesty class PB-DX2's re-review was burned by, in a batch whose own subject is
stale notes. **Fix**: give T1 a library card so the hand assertion is real, and re-derive the
pre-fix note empirically rather than by inference.

### LOW-2 — T4 is a weak regression guard for a now-`Complete` clause

`:451-477` proves only that *some* trigger sourced from Garruk's Uprising fires when a 4/4
enters. No negative case (a power-3 creature must **not** trigger it), no assertion that the
effect is a draw, no resolution. The third clause's `min_power: Some(4)` and `controller: You`
are unpinned by this batch even though the flip to `Complete` now asserts them.

### LOW-3 — T9 leaves two halves of the search clause unpinned

`:717-727` asserts only that the announced card reached hand and the stack emptied. It does not
assert the un-chosen candidate is still in the library, and **nothing anywhere asserts the
trailing `Effect::Shuffle` ran** — the printed "then shuffle" is untested on a card this batch
marks `Complete`.

### LOW-4 — `reveal: true` is silently a no-op, unmentioned in the def

`effects/mod.rs:3479` destructures `reveal: _`, and `:3501-3503` records this as pre-existing
seed **OOS-DP9-9**. Inventors' Fair's printed "reveal it" is therefore not implemented. Hidden-
information-only, no game-state consequence, and there is precedent (`thaumatic_compass.rs` is
`Complete` with the same flag), so it does not block the flip — but a `Complete` marker on a
card with an unimplemented printed clause should carry an explicit in-def note pointing at
OOS-DP9-9 rather than being silent.

### LOW-5 — stale header comment ordering

`inventors_fair.rs:1-5` still lists `{T}: Add {C}` before the upkeep trigger, the pre-batch
`abilities` order; the vec now correctly follows oracle order (upkeep, mana, search).

### LOW-6 — line-cite drift inside the plan's own risk note

`pb-plan-DX3.md:207` cites `matches_filter` at `effects/mod.rs:9502`; it is at `:9496` (`:9502`
is the first line of the `min_power` block). Cosmetic, but it is the one cite in the plan that
ignores the plan's own §1 instruction — and PB-DX2's lesson — to cite by **symbol**.

---

## §5 Summary

Both flips are correct and defensible clause-by-clause against MCP oracle text and all eight
rulings; all four hard gates are green. Fix MEDIUM-1 (and ideally LOW-2..6, all small) and the
batch is ready to merge.
