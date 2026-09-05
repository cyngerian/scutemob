# Landscape Assessment: scutemob against phase.rs, Manabrew, XMage and Forge

<!-- last_updated: 2026-09-05 -->

*Companion to `docs/mtg-engine-landscape.md` (the survey). This document answers the survey's
§6 assessment tasks against scutemob at `7a13c763` (main, 2026-09-05). phase.rs and Manabrew were
cloned at depth 1 into `~/projects/scutemob-landscape/` (phase `e76ca2e5`, "Fix Ultimate
Spider-Man (#8579)"; manabrew `79da22c6`, "chore(release): 14 package(s)"), both 2026-09-05.
Every number below was measured on those checkouts or on this repo; nothing is taken from the
survey's prose without re-checking.*

---

## 0. Corrections to the survey's baseline

The survey is accurate about the two Rust projects in almost every particular. Its description of
scutemob is not, and two of its phase.rs facts have moved. Fix these before reasoning from it.

| Survey claim | Measured | Where |
|---|---|---|
| scutemob "roughly 60K+ lines with 130K+ tests" | engine crate 492,619 lines (93,981 src + 398,395 tests, 468 test files); card-defs 99,198; simulator 48,002; card-types 14,563; view-model 3,843; play-server 22,039. **5,379 `#[test]` functions** plus **271 golden JSON scripts (208 approved)**. | `find`/`grep` over `crates/`, `tools/`, `test-data/` |
| "currently in the card-authoring phase" | Pod-first course correction (P1) approved 2026-09-05; authoring resumes after CC-6. | `docs/course-correction-2026-09.md` |
| "friend group that currently plays on Cockatrice" | Not recorded anywhere in the repo or memory. Treat as the survey author's assumption. | `grep -ri cockatrice` |
| "four Commander decks" (§6 task 8) | Six pod decks (`docs/end-state.md`, CC-5). | |
| phase.rs "`.planning/` per phase" | Directory does not exist at HEAD. | `ls phase/.planning` |
| phase.rs crate list | Also has `manabrew-compat` (adapter to Manabrew's wire protocol 5.2.0), `draft-core`, `seat-reducer`, `lobby-broker`, `engine-inventory-gen`, `probe-pin`. | `ls phase/crates` |
| phase.rs coverage "thousands unimplemented" | Live badges 2026-09-05: card coverage **89%**, cards **31,868 / 35,804**, keywords **190/190**, **Commander 92%**. | `curl data.phase-rs.dev/badges/*.json` |
| phase.rs "weeks old" | The HEAD commit is PR #8579. Whatever the calendar age, the throughput is thousands of merged PRs. | `git log -1` |
| Manabrew 646 tests | 665. | `grep '#\[test\]'` |

One survey fact that matters more than it is given credit for: **phase.rs now speaks Manabrew's
client protocol** (`crates/manabrew-compat`, translating `WaitingFor` ↔ `PromptInput`). The two
Rust engines are converging on one client/prompt wire format. See §10 for what that means for
scutemob's deferred P3.

---

## 1. Card model classification

**scutemob sits between XMage and phase.rs, and is closer to phase.rs than the survey says.**

XMage composes a card from a library of *classes* with behaviour in them. phase.rs produces a
typed *data* AST (`AbilityDefinition` / `Effect` / `TriggerDefinition` / `StaticDefinition` /
`ReplacementDefinition`) from Oracle text. scutemob's card def is exactly that second thing, a
typed data AST (`AbilityDefinition` 68 variants, `Effect` 106, `TriggerCondition` 50, plus
`ContinuousEffectDef` / `ReplacementTrigger` / `LayerModification`), written as a Rust struct
literal by an agent instead of emitted by a grammar. The runtime representation is the same kind
of object; only the *front end* differs (agent-authored vs parsed). That is a much smaller
architectural gap than "XMage's model", and it is what makes option B in §10 a front-end question
rather than a rewrite.

### Three cards side by side

**Lightning Bolt.**

| | Representation |
|---|---|
| scutemob | `AbilityDefinition::Spell { effect: Effect::DealDamage { source: None, target: DeclaredTarget{0}, amount: Fixed(3) }, targets: [TargetAny], modes: None, cant_be_countered: false }` (24-line file, oracle text beside it) |
| phase.rs | `{"kind":"Spell","effect":{"type":"DealDamage","amount":{"type":"Fixed","value":3},"target":{"type":"Any"}}}` (emitted, no file) |
| Forge | `A:SP$ DealDamage \| ValidTgts$ Any \| NumDmg$ 3` |

Isomorphic. Nothing to choose between them.

**Beast Within** ("Destroy target permanent. Its controller creates a 3/3 green Beast").

| | Representation |
|---|---|
| scutemob | `Effect::Sequence([DestroyPermanent{DeclaredTarget 0}, CreateToken{ spec: TokenSpec{ name, 3/3, Green, Creature, Beast, count: Fixed(1), ..Default::default() } }])`, `targets: [TargetPermanent]` |
| phase.rs | `Destroy{target: Typed{Permanent}}` with `sub_ability: CreateToken{ ..., controller: ControllerRef::TargetController }` (target propagates to the sub-ability) |
| Forge | `A:SP$ Destroy \| ValidTgts$ Permanent \| SubAbility$ DBToken` / `SVar:DBToken:DB$ Token \| TokenScript$ g_3_3_beast \| TokenOwner$ TargetedController` |

Here the representations diverge in a way that matters. Forge and phase.rs carry the token
recipient explicitly. scutemob's `TokenSpec.recipient` (added by PB-EF2) defaults to
`PlayerTarget::Controller`, i.e. the caster, and `beast_within.rs` leaves it defaulted. **At a
four-player table Beast Within gives the Beast to whoever cast it, not to the destroyed
permanent's controller.** Verified by running the retired golden script
`tokens/002_beast_within_creates_beast.json` through the harness (`SCRIPT_FILTER=002_beast_within
cargo test -p mtg-engine --test scripts run_all_scripts`): Sol Ring is destroyed, and p2's
battlefield has no Beast. The def is deck-legal because it carries no `completeness:` marker and
`Complete` is the `#[default]`. The primitive that fixes it,
`PlayerTarget::ControllerOf(Box<EffectTarget>)`, already exists. Full list in §2.

**Rest in Peace / Blood Moon** (replacement with an "instead" rider; layer-dependent static).

| | Representation |
|---|---|
| scutemob RiP | `Triggered{WhenEntersBattlefield, ForEach{EachCardInAllGraveyards, ExileObject}}` + `Replacement{ trigger: WouldChangeZone{to: Graveyard, filter: Any}, modification: RedirectToZone(Exile), is_self: false }` |
| scutemob Blood Moon | `Static{ ContinuousEffectDef{ layer: TypeChange, modification: SetLandTypes({Mountain}), filter: AllNonbasicLands, duration: WhileSourceOnBattlefield } }` with the CR 305.7 ability-clearing and the intrinsic `{T}: Add {R}` derived by the engine (`rules::layers::derive_intrinsic_land_mana_abilities`) |
| phase.rs | `replacements[]: { event: Moved, valid_card: Typed{..., InZone Battlefield}, destination_zone: Graveyard, execute: ChangeZone{destination: Exile, target: SelfRef}, mode: Mandatory }`; `statics[]: { mode: Continuous, affected: Typed{Creature, controller You}, modifications: [AddPower 1, AddToughness 1] }` (from the pipeline snapshots; Blood Moon itself would be a `SetSubtypes`-class static) |
| Forge | `R:Event$ Moved \| Destination$ Graveyard \| ValidCard$ Card \| ReplaceWith$ Exile ...` / `S:Mode$ Continuous \| Affected$ Land.nonBasic \| AddType$ Mountain \| RemoveLandTypes$ True \| RemoveSubTypes$ True` |

All three encode the layer explicitly. scutemob's is the most *rules-literal* (the def names the
CR layer; the engine derives the 305.6 mana ability rather than the card asserting it), and the
Blood Moon file's 30-line header explains why the previous three-static shape was a latent CR
305.7 violation. phase.rs and Forge share Forge's trigger/replacement vocabulary (`Mode$`,
`ValidCard$`, `Destination$`), which is why phase.rs can fall back to Forge scripts for gaps.

### Which is fastest to author, and which to audit

- **Author:** phase.rs (zero per-card cost once the grammar covers the class) > Forge script (one
  line, but the author must know the interpreter's parameter names) > scutemob (a 25–60 line
  struct literal; the agent must know 106 Effect variants and their field conventions).
- **Audit one card against its oracle text:** scutemob > Forge > phase.rs. scutemob's def has the
  oracle text and the DSL in one file, the DSL names the CR concept (`EffectLayer::TypeChange`,
  `ReplacementModification::RedirectToZone`), and a reviewer reads clause by clause. phase.rs's
  output is only inspectable by dumping JSON, and the question "did the parser swallow a clause?"
  needed an 11K-line `swallow_check.rs` to answer at scale.
- **Audit the corpus:** phase.rs > scutemob. phase.rs's gaps are machine-derived
  (`Effect::Unimplemented` + swallow check + `cargo coverage` in CI). scutemob's are
  author-asserted (`completeness:` markers) and audited by `tools/authoring-report.py` and review
  agents. §2 shows where that difference bites.

---

## 2. Fail-closed audit

**scutemob fails closed at the card, at deck-build time. phase.rs fails closed at the clause, at
parse time. The unit of failure is the difference, and it is what let Beast Within through.**

scutemob's gates, all machine-enforced:

| Gate | Unit | Mechanism | Measured |
|---|---|---|---|
| SR-2 / Invariant 9 | card | `Completeness::{Complete (default), Inert, Partial, KnownWrong}`; `validate_deck` and `build_initial_state_checked` reject non-Complete | 1,140 Complete (176 explicit, **964 by default**), 415 partial, 147 inert, 101 known_wrong |
| SR-4 | engine site | every silent-failure site in `effects/mod.rs` + `rules/resolution.rs` is `expect_*` (bug) or `lki_*` (fizzle) | 218 `expect_*` calls, 2 `lki_*` |
| SR-5 | keyword | exhaustive `keyword_registry::handling` | 149 Handled / 19 Marker |
| authoring-report | def file | scans `// TODO` / `// ENGINE-BLOCKED` lines | 589 defs carry them; **0** have a TODO with a defaulted marker; 1 (`chord_of_calling.rs`) has a TODO with an explicit `Complete`, and that TODO is prose about a *removed* TODO |
| engine `todo!`/`unimplemented!`/`unreachable!` | | | 7 in 94K src lines |

So the marker discipline is good where a TODO comment exists. The hole is a def whose author
believed it complete and wrote neither a TODO nor a marker. phase.rs cannot have that hole for a
clause its grammar does not recognise (it becomes `Effect::Unimplemented` mechanically), and it
has a second guard for the clause its grammar *mis*-recognises (swallow check). scutemob's only
guard for that second class is review, which is exactly the "legal-but-wrong" risk already named
in memory (`project_legal_but_wrong_gap.md`).

**Concrete silent no-ops found (the survey's "these will lose games at the table" list):**

| Card | Marker | Defect | Fix |
|---|---|---|---|
| `beast_within.rs` | default Complete | token to caster, not target's controller | `recipient: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget{index:0}))` |
| `generous_gift.rs` | default Complete | same | same |
| `stroke_of_midnight.rs`, `emergency_eject.rs`, `saw_in_half.rs` | Partial | same, but correctly gated; the TODO text says "fix when CreateToken gains a player field", which it did (PB-EF2) — **the blocker note is stale** | set `recipient`, drop marker |
| `pongify.rs` | KnownWrong | same, gated | same |
| `tokens/002_beast_within_creates_beast.json` | retired | retirement reason ("unthreaded target, Sol Ring survives") is stale: the destroy now works and the assertion that fails is the token's controller | re-approve after the def fix; it becomes the regression pin |

That is two deck-legal wrong cards found by pulling one thread. It argues for one cheap addition
(§9 item 2): make `completeness:` **required** rather than defaulted, so "I forgot to think about
it" is a compile error instead of `Complete`.

---

## 3. The reducer

| | scutemob | phase.rs |
|---|---|---|
| Entry point | `process_command(state: GameState, command: Command) -> Result<(GameState, Vec<GameEvent>), GameStateError>` (`rules/engine.rs:364`) | `apply(state: &mut GameState, actor: PlayerId, action: GameAction) -> Result<ActionResult, EngineError>` (`game/engine.rs:882`) |
| State model | by-value in, new value out; `imbl` 7 persistent collections; sealed `pub(crate)` (SR-3) | `&mut`; `im` 15 persistent collections; `declare_game_state!` macro with serde-default field evolution |
| Actor | inside each of the 45 `Command` variants as `player: PlayerId` | a separate parameter, documented as "must come from the connection, not the wire frame" |
| Pending decision | split: engine `BlockingDecision` (CleanupDiscard, TriggerTargets, EffectChoice) + simulator `DecisionKind` (8 kinds) derived from state by `LegalActionProvider` | `ActionResult.waiting_for: WaitingFor` (136 variants), returned by the engine with every result |
| Outputs | `Vec<GameEvent>` (133 variants) + the new state | `events` + `waiting_for` + `log_entries` |
| Rejections | `GameStateError`, Debug-formatted into `HaltReason::EngineError(String)` by the play-server | `apply_with_rejection` maps to a viewer-filtered `ActionRejection` |

Three findings.

1. **Actor binding is not yet a problem, and will be at P3.** Today `tools/play-server` has one
   human seat (`session.human`, hard-coded `human_seats: [HUMAN_SEAT]`) and bots act in-process, so
   there is no second trust domain. CC-9 hot-seat adds more human seats in the *same* browser, still
   one trust domain. The first networked seat (P3) is when `command.player()` must be overwritten
   or validated from the connection. Write that rule into `crates/network`'s brief now (it is a
   two-line crate today) so it is not rediscovered.
2. **"What may I do?" lives outside the engine.** phase.rs's engine answers it (`WaitingFor`).
   scutemob's engine answers only the three blocking cases; priority-window legality is enumerated
   by `crates/simulator/legal_actions.rs`. That is the offer layer CC-15 just put a ceiling ratchet
   on, and it is a mild tension with Invariant 1's "everything else is the caller's
   responsibility": every client (TUI, play-server, a future network client) must depend on the
   simulator crate to know its legal moves. Not urgent; note it as the shape P3 will want to
   change (move `LegalActionProvider` into the engine, or make the engine emit a `WaitingFor`).
3. **Rejections are not viewer-scoped.** A `GameStateError::InvalidTarget { object }` rendered to a
   seat that may not identify `object` is a hidden-information path. The play-server's `redact`
   labels unknown ids `(hidden card)` for state views, and `view.rs` documents the halted arm, but
   I did not trace every rejection string end to end; treat as **unverified, P3**. phase.rs solved
   it by routing rejections through the same `visibility.rs` filter as state.

---

## 4. CR annotation census

| | scutemob | phase.rs | Manabrew |
|---|---|---|---|
| `CR \d{3}` occurrences, engine src | **8,594** (93 distinct sections) | 74,828 (all `.rs` incl. tests; src-only not separable) | 79 |
| in tests | 13,833 (437 of 468 test files cite at least one) | (included above) | — |
| in card defs | 2,254 | n/a | n/a (Java-provenance comments instead: 1,663) |
| Density, src | ~1 per 11 lines | ~1 per 22 lines over the whole tree | — |
| Policy | Invariant 8 + `memory/conventions.md` "Comprehensive Rules Citation Format" | mandatory, "a wrong CR number is worse than none", grep-verified, `validate-cr-annotations` skill, `cargo rules-audit` | mirror Java, list CR deviations in `forge-dsl-semantics.md` §11.1 |

scutemob already has the practice at phase.rs density. What it lacks is the **verifier**: nothing
checks that a cited `CR 702.140e` exists in `.scryfall-cache/MagicCompRules.txt`, and there is no
rules-coverage report. Both are one script. Cost: an afternoon; it slots under the existing gate
list as a class-0 change. Recommended (§9).

---

## 5. Layer-system parity

| CR 613 | scutemob (`rules/layers.rs`, 4,013 lines) | phase.rs (`game/layers.rs`, 24,404 lines) | Manabrew / Forge |
|---|---|---|---|
| 1 copy (incl. 729 merge, face-down) | yes; merged permanents integrated at Layer 1 and 6 | `Layer::Copy` | yes |
| 2 control | yes | `Layer::Control` | yes |
| 3 text | yes (`EffectLayer` has it; `LayerModification` text changes) | `Layer::Text` | yes |
| 4 type (+CDAs first, 305.7 land-type clearing, intrinsic 305.6 mana) | yes, with the ordering comments to prove it | `Layer::Type` | yes |
| 5 color | yes | `Layer::Color` | yes |
| 6 abilities (Humility semantics, Yixlid Jailer in graveyards) | yes | `Layer::Ability` | yes |
| 7a CDA P/T | yes | `CharDef` | yes |
| 7b set | yes | `SetPT` | yes |
| 7c modify + counters | yes | `ModifyPT` + `CounterPT` | yes (counters folded in) |
| 7d switch | yes (`EffectLayer::PtSwitch`) | `SwitchPT` | **no** |
| 613.8 dependency | yes: Humility+Opalescence, Blood Moon+Urborg both orders, Opalescence+Parallax Wave (corner-case ledger rows 1–7, all COVERED) | yes | yes, cycle → timestamp fallback |
| Forge's "layer 8" rule-changes | n/a (modelled as restrictions, not a layer) | n/a | yes, a CR deviation |

Parity is complete on the CR axis. scutemob's 4K lines vs phase.rs's 24K is not a coverage gap;
phase.rs carries pruning of every duration kind, casting-permission durations and an incremental
flush ("mark_layers_full / mark_layers_entered") in the same file. The one open row in scutemob's
ledger (row 36, Blood Moon + Urza's Saga) is a *card* gap (`urzas_saga.rs` Partial), not a layer
gap.

---

## 6. Hidden-information audit

| Surface | scutemob | phase.rs |
|---|---|---|
| Where the projection lives | `crates/view-model` (`redact.rs`, `event_view.rs`), reached via `StateViewModel::from_game_state_for(.., Viewer::Seat)` | `crates/engine/src/game/visibility.rs` (8,525 lines); `server-core/filter.rs` is a 1K-line shim over it |
| Engine-side hook | `GameEvent::private_to() -> Option<PlayerId>` covers **2 of 133** event variants (EffectChoiceRequired, CleanupDiscardChoiceRequired); everything else is field-level entitlement in the view-model (`viewer_may_identify`) | all of state, events **and rejections** |
| State | hand → placeholders with count; library never enumerated (own included); face-down battlefield/exile/stack/combat name-redacted; graveyard and command zone public. `redact.rs` carries a site-by-site leak inventory ("the leak follows the rendering site, not the zone") and documents one leak layers cannot close (`is_commander` on a face-down permanent) | same set; `HIDDEN_CARD_NAME` sentinel; look-result provenance redacted from `WaitingFor` payloads |
| Events | `event_view_for(ev, state, viewer)` drops or name-frees lines | `filter_events_for_player` |
| Rejections | not viewer-scoped (§3) | `filter_action_rejection_for_viewer` |
| Actor | in the command (§3) | connection-bound parameter |

scutemob's redaction is careful and well argued, and `event_view.rs`'s own header already says the
architectural thing: the entitlement logic "is the part `private_to()` cannot express, and either
it moves into the engine or..." phase.rs made that move. For hot-seat (one process, one trust
domain) the current placement is fine. For P3 the recommendation is to follow phase.rs: move
`redact` + `event_view` into the engine crate as `visibility`, so local, hot-seat and networked
modes share one implementation, and add rejection filtering there.

---

## 7. What the tests are

5,379 `#[test]` functions + 271 golden scripts, bucketed:

| Bucket | Count | Examples |
|---|---|---|
| Building-block / mechanic tests with CR citations | ~4,100 (`primitives/` 1,393, `mechanics_*` 1,641, `rules/` 600, `casting/` 146, `combat/` 76, plus much of `core/` 831) | `test_evolve_noncreature_does_not_trigger`, `test_107_4e_insufficient_pool_cannot_pay_an_otherwise_payable_hybrid_attack_tax`, `test_restriction_grand_abolisher_blocks_opponent_cast` |
| Regression pins named after batches / seeds | several hundred (`test_dx1_lookback_dies_trigger_not_suppressed`, `t8_magus_of_the_moon_keeps_artifact_land_card_type...`, `f3_modification_blanks_abilities_recognises_both_channels_and_no_others`) | pins **known deviations** too (OOS-DP1-4 mana-ability `players_passed`) |
| Golden JSON replays | 271 files, 208 approved (stack 145, combat 53, baseline 35, etb 16, commander 7, replacement 7, layers 3, tokens 3, forage 2), partition-gated (SR-9c) | |
| Property tests (`proptest`) | 6 files: state hashing, state/turn invariants, harness equivalence | |
| Source-text gates | SR-4/5/8/35/36/37, `no_stray_test_binaries`, `unread_field_allowlist_has_no_dead_entries` | |
| Simulator | 359 (fuzzer, invariants, bots, decision coverage) | |
| Snapshot tests (insta-style) | **none** | |

CR-citation density is high everywhere except `scripts/` (38 cites for 45 tests, but the scripts
themselves carry `cr_sections_tested`). Ratio in the mechanics groups is ~3–4 cites per test.

Distribution vs the peers: scutemob looks like phase.rs's runtime-scenario layer (their `card-test`
recipe) at similar density, minus phase.rs's grammar-level parser tests (which scutemob has no
grammar to test) and minus snapshots. Manabrew has almost no unit tests by design; its oracle is
Forge.

**External oracle: none.** The fuzzer and the property tests are self-consistency oracles; the
golden corpus is agreement with the script author. The Beast Within finding is what "we agreed
with ourselves" looks like: the script that disagreed was retired with a stale reason, and the def
went Complete by default. Options, cheapest first:

1. **phase.rs as a parse-level oracle for pod cards** (MIT, Rust). For each pod card, diff the
   *clause inventory* of scutemob's def (abilities / triggers / statics / replacements, with
   recipients and targets) against phase.rs's parsed JSON for the same oracle text. This catches
   missing and mis-attributed clauses (the Beast Within class) without running a game. Cost: build
   phase.rs's `oracle-gen` once (1.35M src lines; feasible on this machine, budget 15–30 min the
   first time), export the pod cards, write a ~200-line comparer. Fits beside CC-6 / CC-13.
2. **phase.rs as a runtime oracle**: seeded deterministic games driven through both engines. The
   action vocabularies differ (45 `Command` variants vs `GameAction` + 136 `WaitingFor`), so a
   shared driver is real work. Defer until 1 has paid off.
3. **Forge via Manabrew's `forge-harness`**: JVM + Maven + Forge submodule, and Forge's documented
   CR deviations (5-layer replacement model, no 7d, fizzle check ignores source LKI) would show up
   as "divergences" that are scutemob being *right*. Differential testing against GPL code does not
   contaminate scutemob, but the noise argues against it. Not recommended.
4. **XMage `Mage.Tests`** as the survey suggests: MIT and scriptable, but Java and 15 years of
   test-framework idiom. Not recommended over 1.

---

## 8. Coverage report

This is CC-5 + CC-6 (`scutemob-241`, `scutemob-242`), already filed and sequenced; nothing to add
except two calibrations from the peers:

- phase.rs's coverage report documents a **noise floor** (export nondeterministic on ~20 faces).
  scutemob's `authoring-report.py` is a deterministic filesystem scan, so the pod-coverage number
  will have none; the Δ column in CC-6's headline can be trusted at face value.
- Once the six decklists exist, running them through phase.rs's `cargo coverage` gives a
  **calibration number**: what fraction of the pod's cards a 92%-Commander-coverage engine already
  parses. If that number is far above scutemob's pod coverage after CC-6, it is the strongest
  single input to the §10 decision.

---

## 9. Process comparison

What `phase/CLAUDE.md` and `add-engine-effect/SKILL.md` have that scutemob's ESM + skills lack,
in the survey's priority order, with a recommendation each:

| # | phase.rs practice | scutemob today | Recommend |
|---|---|---|---|
| a | **Lockstep registration checklist** for a new effect: types → `effect_variant_name` → `EffectKind` → resolver → `effects/mod.rs` arm → trigger target extraction → targeting → parser → `WaitingFor` + `apply` arm → visibility filter → frontend → AI → tests. "Missing any step causes silent failures." | The steps exist but are scattered: SR-8 wire bump (`engine-invariants.md`), the two exhaustive matches in `view-model/lib.rs` (`gotchas-infra.md`, "runners miss the keyword one ~50%"), TUI `stack_view.rs`, SR-4 classification, SR-5 registry, `helpers.rs` prelude, play-server DTOs, golden script | **Yes, high.** One file, `memory/checklists/new-effect-variant.md` (or a section in `conventions.md`), listing every registration point with its file path, and have `/implement-primitive`'s runner brief cite it. CC-12 (split `execute_effect_inner`) is the natural moment, since it moves the resolver arms. |
| b | **Parameterize, don't proliferate** + sibling-cluster smell + categorical boundary rule | Type Consolidation (2026-03-09) *was* this refactor; `Effect` is 106 variants vs phase.rs's 233, healthy. No standing rule prevents re-growth | **Yes, cheap.** Two paragraphs in `conventions.md`; the "three variants sharing a name root" test is greppable and could be a source-text gate, but pair-or-demote (CC-17) says a gate must ship with a probe, so keep it a review checklist item |
| c | **Multi-agent file-collision rules** (never revert, surgical edits, never stash, wait 10 min) | Not needed: scutemob isolates workers in worktrees + ESM; the analogous hard-won rules are dispatch hygiene 9/10 (shared stash stack, never `git add -A`) | No change |
| d | **`engine-inventory.json`** as the canonical engine-surface index, regenerated by a binary, grepped before proposing a variant | SR-36 (`all_cards()` for rosters, never grep), rust-analyzer MCP for symbols | Low. rust-analyzer covers discoverability; a generated inventory would mostly duplicate `card-types`' enum docs |
| e | **Measurement noise floor** documented before claiming coverage wins | N/A (deterministic) | No change; §8 |
| — | **`Completeness` required, not defaulted** (scutemob-specific, from §2) | `#[default] Complete`; 964 defs rely on it | **Yes, high.** Remove the `Default` derive path for `completeness` (make the field required, or add a source gate that every def names it). Then fix `beast_within.rs`, `generous_gift.rs`, un-stale the three Partial notes, re-approve `tokens/002`. Class 1 change |
| — | **CR-cite verifier + rules-coverage report** (§4) | none | **Yes, cheap.** Script over `MagicCompRules.txt`; run under the gate list |
| — | **Fluent scenario driver** (`GameScenario` → `GameRunner.cast(..).modes().x().target_player().resolve()` → `CastOutcome.assert_*`) that makes six recurring test foot-guns "structurally impossible" | test helpers per group + JSON scripts; foot-guns are documented in `gotchas-infra.md` rather than designed out | Medium. Worth reading `phase/.claude/skills/card-test/SKILL.md` before the next test-helper refactor |
| — | **Verify the card, not just the rule** / **Read the card before the code** | already: MCP authoritative (`feedback_oracle`), SR-37 printed-field fidelity gate | No change |
| — | **AI-CONTRIBUTOR.md**: model-tier gate, `Model:` line in PR body, pre-PR combinator-purity script | solo project | No change; the *pre-PR gate script* idea is what `tools/check-defs-fmt.sh` already is |
| — | Tilt instead of cargo (build-lock contention between concurrent agents) | worker isolation makes it moot; dispatch hygiene 11 (`/tmp` quota) is the local analogue | No change |

---

## 10. Strategic recommendation

**(A) Keep hand-authoring cards against scutemob's primitives, with two borrowings from phase.rs
and one explicit switch condition.**

Why A over B (Oracle grammar front end): the target is finite and demand-driven. Six decks are at
most ~600 distinct cards, and scutemob has authored 1,803 defs already. phase.rs's grammar is
1.35M source lines and thousands of PRs to reach 89%; its long tail (`imperative.rs` 24K,
`oracle_replacement.rs` 27K, an 11K swallow guard) is the cost of covering *all* 35K cards. For
~600 named cards, a linear per-card cost is cheaper than a sublinear grammar whose fixed cost
exceeds the whole job. B only wins if the pod changes decks faster than the agent authors, which
CC-6's Δ column will show.

Why A over C (Forge scripts): GPL is the smaller problem; the larger one is that Forge's runtime
semantics deviate from the CR in ways scutemob has invested heavily in getting right (5-category
replacement model vs CR 616.1; no 7d; fizzle check ignoring CR 608.2b source LKI). A Forge front
end would import those deviations or require a second interpreter that corrects them.

Why A over D (contribute to phase.rs instead): honest answer first. **If the only goal were "the
pod plays rules-enforced Commander online as soon as possible", D is the shortest path today.**
phase.rs has 92% Commander parse coverage, a 4-player server with hidden information, Tauri and
browser clients, a Resolve-All consent protocol for tables, and a contributor pipeline built for
exactly the agent workflow scutemob uses. scutemob's owner ruling (`end-state.md`) names playable
pod matches as the end state, but the motivation on record is completeness-driven and the engine
is the point (`user_project_motivation.md`). A is consistent with that; D is not. The decision is
the owner's, and it should be made on a number, not a feeling, which is the switch condition
below.

**The two borrowings** (both MIT-compatible, both small):

1. phase.rs as a **parse-level oracle** for the pod cards (§7 option 1). It attacks the one class of
   defect scutemob's gates cannot see, and it is the cheapest external oracle available.
2. **Required `completeness` marker + CR-cite verifier + one lockstep checklist** (§9). These close
   the gaps this assessment actually found.

**What would change the recommendation to D.** After CC-6 lands, compute two numbers for the six
decks: scutemob pod coverage (CC-6 headline) and phase.rs parse coverage for the same card list
(§8). If scutemob is below ~50% *and* the missing-card list is dominated by engine gaps rather
than un-authored cards, or if two more handoffs pass with an empty operator-delta line after CC-9
ships, D becomes the right call and this document should be reopened. If scutemob is above ~70%
with an authoring-dominated remainder, A is confirmed and B/C/D are closed.

**One P3 note, low regret.** phase.rs now implements Manabrew's client protocol
(`crates/manabrew-compat`). When P3 networking is un-deferred, target that protocol rather than
inventing a wire format: it is the emerging shared client contract in the Rust MTG space, its
prompt shapes (`PromptInput` / `PromptOutput`) map onto `WaitingFor`-style decision points, and
adopting it would let scutemob use either project's clients as a benchmark or a fallback.

---

## Appendix: where to look

- Clones: `~/projects/scutemob-landscape/{phase,manabrew}` (depth 1, 2026-09-05; `manabrew/forge`
  submodule not initialised).
- phase.rs reading order that paid off: `CLAUDE.md` → `.claude/skills/add-engine-effect/SKILL.md`
  → `crates/engine/src/game/engine.rs:882` → `types/layers.rs` → `game/visibility.rs` →
  `crates/manabrew-compat/CLAUDE.md`.
- Manabrew: `docs/PARITY_AND_IR.md`, `docs/agents/PARITY_PHILOSOPHY.md`,
  `docs/forge-dsl-semantics.md` §6 and §11.1.
- scutemob evidence: `crates/card-types/src/cards/card_definition.rs:197` (`Completeness`),
  `:4531` (`TokenSpec.recipient`), `crates/engine/src/rules/engine.rs:364` (`process_command`),
  `crates/simulator/src/local_game.rs:119` (`DecisionKind`), `crates/view-model/src/redact.rs`,
  `test-data/generated-scripts/tokens/002_beast_within_creates_beast.json`.
