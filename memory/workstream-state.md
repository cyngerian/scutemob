# Workstream State

> Coordination file for parallel sessions. Read by `/start`, claimed by
> `/start-work`, released by `/eot`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | B16 complete (Dungeon + Ring); all abilities done |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | LOW Sweep campaign COMPLETE 2026-05-16 (`scutemob-31..38`): 36 LOWs closed, LOW-OPEN 45→6. 6 remain (honestly deferred). Plan: `memory/archive/2026-07/low-sweep-plan.md` (archived 2026-07-18). |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available (**PB-DX39 `scutemob-230` SHIPPED 2026-09-05 — task 2 of 5 of the approved chain; next is PB-DX53, rank 16**) (**PB-DX52 `scutemob-229` SHIPPED 2026-09-04 — task 1 of 5**) (**four-task dispatch chain COMPLETE 2026-09-04**: PB-DX18 `scutemob-225` `61f9d5e1`, PB-DX51 `scutemob-226` `275b00af`, PB-DX35 `scutemob-227` `e8c212e7`, PB-DX36 `scutemob-228` `d15692f7`; v4 ranks 1-13 all shipped; **FIVE-task chain APPROVED by user 2026-09-04 (exactly five, sequential, collect-before-next): PB-DX52 (rank 14) → PB-DX39 (15) → PB-DX53 (16) → PB-DX54 (17) → PB-DX42b (18); PB-DX52 dispatching**)

## Last Handoff (worker, 2026-09-05) — PB-DX53 / `scutemob-231`

**v4 rank 16. `OOS-DX21-1` CLOSED.** Next dispatch: **PB-DX54** (rank 17, `OOS-DX25c-6`).
Full record: `memory/primitives/pb-DX53-execution-notes.md`. Plan and the pre-committed
predictions: `memory/primitives/pb-DX53-plan.md` (`a37f8239`, before any production line).

### The one thing to read first

**One DSL identifier carried two CR concepts, and the seed's own prescribed fix would have broken
a card to fix a card.** `Condition::YouAttackedWithNOrMore` was read by `windbrisk_heights`
(ruling 2007-10-01 — per TURN, deduplicated, CR 508.4 entrants excluded) and by `legions_landing`
(CR 508.3d — per DECLARATION). The row says to make the field *"a per-turn accumulation with
per-creature dedup … and the migration must leave Legion's Landing reading the per-declaration
count"* — right about the two REQUIREMENTS, wrong about their being one field. **The DSL split**:
two `PlayerState` fields, two `Condition` variants, both old names renamed because both were lies.

### What the next batch should carry forward

1. **A wire cell that names the field you expect to add says nothing about the IDENTIFIER you may
   have to split.** The AC predicted PROTOCOL unmoved on a ground that is TRUE (`PlayerState` is in
   `CLOSURE_MUST_NOT_CONTAIN`) and insufficient. `Condition` is on the wire via `Effect::Conditional`
   — verified by execution at stage 0, and already written down in `rules/protocol.rs`'s v21 row by
   the batch that created this very field.
2. **A "N deck-legal `Complete`" yield cell is a floor whenever the class is defined by a PRINTED
   phrase rather than a declared variant.** `minas_tirith` was invisible to the declared axis
   because its ability was UNAUTHORED. Only the inverse ORACLE axis finds a card whose defect is
   that it is missing — and its blocker note demanded an identifier that had shipped six weeks
   earlier.
3. **A census walk over a def has TWO axes** — how exhaustively it reaches, and whether what it
   reaches is CODE or PROSE. This batch's own roster gate defended the first in its module doc and
   was blind to the second: a `Debug` render counted a `Completeness` note as a declaration.
   **Use `decision_site_walk::def_contains_variant`**; its `PROSE_FIELDS` already carries the
   `Completeness` variant keys. (`OOS-DX53-2`.)
4. **Assert a revert PATCH APPLIED before reading any verdict**, and key the build-failure detector
   on `^error\[E` / `could not compile` — not `^error:`, which cargo also prints for ordinary test
   failures. This matrix got both wrong once each and caught both before publishing.
5. **`GameState`'s size, not `PlayerState`'s, looks like PB-DX18's real bench driver.** This batch
   grew `PlayerState` MORE (376 → 400, +6.4% vs PB-DX18's +4.4%) and left `GameState` unmoved at
   3536 — and measured **no regression**. Offered as an inference with its evidence, not a finding.

### State

Tests **5,209 / 0 / 5**, 67 targets (+13, 0 leavers). **HASH 85 / PROTOCOL 44**, closure counts
**98 / 132** unmoved. Coverage **1,140/1,803 = 63.2%**, one flip. All gates clean against the FINAL
tree; `npm run build` N/A (`tools/` diff empty). Filed **OOS-DX53-1..3**.

**Open and deliberately not taken**: `OOS-DX53-1` (Melee counts CR 508.4 entrants — 2 deck-legal
`Complete`), `OOS-DX53-3` (`moraug_fury_of_akoum` needs a per-creature attack TALLY, the opposite
structure from this batch's dedup'd set — filed so nobody mistakes one for the other).

---

## Last Handoff (worker, 2026-09-05) — PB-DX39 / `scutemob-230`

**Task**: `scutemob-230` — PB-DX39, v4 queue rank 15 (standing v3 rank 33). Branch
`feat/pb-dx39-source-relative-filters-through-lki-a-continuous-eff`, merge base `604b7242`.
**Seeds**: `OOS-DX5-3` (headline) and `OOS-DX5-7`'s named residual **both CLOSED**, each row
corrected against three and four of its own claims. Filed `OOS-DX39-1..10`.

**Shipped**: `rules::layers::SourceView<'a>` — ONE borrowed view of everything an `EffectFilter`
arm needs to know about a continuous effect's source (`controller`, `attached_to`,
`chosen_creature_type`, `chosen_color`) — with TWO constructors carrying two different CR
justifications: `source_view_live` (CR 611.3a, no fallback) and `source_view_at_resolution`
(CR 608.2h / CR 113.7a, live-then-LKI, exactly one caller). All **20** source-relative arms consume
one `Option<&SourceView>` parameter; `snapshot_affected_set` resolves the view ONCE outside its
candidate loop. Plus two LKI-CAPTURE clauses sharing one store function, because the store was
empty for both subjects.

**READ THESE FIVE THINGS BEFORE PB-DX53.**

1. **The seed's preferred fix does not work as written, and the reason is SR-24.** The brief
   prefers *"resolve against the LKI snapshot the engine already keeps … so nothing new is stored"*.
   `capture_lki_snapshot` stores a snapshot only when the departing permanent carries one of
   `[Wither, Infect, Deathtouch, Lifelink]`, and neither subject does — `lki_objects` was **EMPTY**
   for both. Option (a) is *read the LKI **AND** make it carry the source*.
2. **The two subjects leave at DIFFERENT moments, so one gate closes half the batch.** The Jitte is
   destroyed **in response**, with its ability already on the stack. Mardu pays `Cost::SacrificeSelf`,
   and `abilities.rs` moves the source to the graveyard *"before pushing to stack"* — so a
   *"is this object the source of a stack object"* test **cannot see it**. Both clauses are needed
   and the coordinator-run revert matrix proves it: R2 and R3 are **precise complements**.
3. **The site list was a floor and the three missing sites were the headline seed's own arm.**
   The brief said 17 `state.objects.get(&source_id)` reads; it is **20 arms / 20 reads**, because
   `AttachedCreature`, `AttachedLand` and `AttachedPermanent` spell the read across a line break
   (`OOS-DX50`'s multi-line lesson, recurring inside the census of a batch about source reads).
   Sweeping "the 17" would have missed `OOS-DX5-3` while reporting a complete sweep.
4. **A CLASS FIX REPAIRS THE ARITHMETIC, NOT EVERY CALLER'S ROUTE TO IT — and this batch published
   the overclaim before withdrawing it.** The census finds a fourth axis no document names: 28
   occurrences across 20 defs, **16 deck-legal `Complete`**, where the source can simply be killed
   in response (Craterhoof Behemoth, Mirror Entity, Purphoros, Massacre Wurm, …). The coordinator
   read PB-DX49's own test-file note about `binding_the_old_gods` and published *"the deck-legal
   live-wrong count is at least TWO"* **before executing it**. Execution refuted the cause: the
   CR 608.2h condition IS reached and the LKI IS captured, and `state.continuous_effects()` comes
   back **EMPTY** — `resolution.rs`'s registry fallback opens with `fizzle_object`, a documented
   live-only lookup, so the whole ability is a no-op and the filter is never consulted
   (`OOS-DX39-3`, live on a deck-legal `Complete` def; PB-DX49's note corrected in place). The
   other 15 axis-(iv) members are **unmeasured individually** and that is stated rather than
   rounded up (`OOS-DX39-5`).
5. **Two hazards of hand-run revert matrices in a multi-agent worktree, both of which produced a
   wrong GREEN before being caught** (`OOS-DX39-7`): `cp -p` on restore preserves the **backup's**
   mtime, so cargo does not rebuild and the next run reports the reverted binary against restored
   source; and a file backup taken at 20:02 was **stale by 20:09** because a sibling agent wrote
   the file. Both detected by md5. And (`OOS-DX39-8`) a revert that removes a function's only caller
   **does not compile** under `-D warnings`, so it yields no verdict at all — a matrix that cannot
   tell *"the gate stayed silent"* from *"the crate did not build"* fails in the safe-looking
   direction.

**Wire**: **HASH 84 / PROTOCOL 43 both gate-executed and UNMOVED — zero bumps**, predicted PER
OPTION in writing before any production line (`60975661`). Option (b), a `StackObject` source
snapshot, was costed at HASH +1 and rejected **on CR grounds rather than cost**: a snapshot taken
at activation answers *"the creature equipped when the ability was ACTIVATED"*, and the Jitte's
2005-02-01 ruling says **"most recently equipped"**.

**The design was SCOPED to the locked path, and that scoping was a CR decision measured rather than
argued.** Enumerating all 36 `ContinuousEffect` construction sites found three that forward an
arbitrary card-def filter with a duration that need not be `WhileSourceOnBattlefield`; an
unconditional fallback would have made a departed source's STATIC ability start applying —
CR 611.3a-wrong, this batch creating a defect while closing one. The coordinator predicted the
exposed population at ZERO and **`r7` refuted it**: 4 emblem registrations across 3 cards, 0
statics, harmless only because nothing in the engine moves an emblem out of the command zone, so CR 400.7
never retires its `ObjectId` -- a MEASUREMENT, not a rule (`OOS-DX39-2`; the `CR 114.1` this line
cited until the `/review` says only what an emblem is).

**Also corrected**: the coordinator's own `t6` specification was **CR-wrong** — it said a creature
entering between activation and resolution is not a member, when CR 611.2c determines the set at
RESOLUTION, so it is. The probe author wrote it to the CR and said so.

**Also unreachable and stated rather than substituted**: AC 7359 asks for both subjects on a real
`LocalGame` drive. `mardu_ascendancy` is `partial`, so Architecture Invariant 9 refuses the game
(`IncompleteCardsInGame`) and **no validated game can contain the card** — which is the same fact
the census states from the other side, that `OOS-DX5-7`'s deck-legal blast radius is ZERO. Its
channel probes drive the same production mapping `LocalGame::submit` calls, and the file says what
that omits.

### `/review` FIX CYCLE — 7 findings, ALL TAKEN; FIVE of the seven gates had been defeated

Full record with every plant, its verbatim failure output and the byte-exact restores:
`memory/primitives/pb-DX39-execution-notes.md` §7.

**The durable half is that five of seven source gates were green against planted regressions, and
four of the five failures are shapes this queue had already recorded.** `r1`'s "two axes" were one
idea measured twice (a map-iteration read and a helper-fn read both walked past it, with the whole
suite green); `r3` and `r4` fell together to `GameState::lki_objects(state)`, because `r3`'s needle
had a leading dot and `r4` was an ABSENCE list — **and an absence list cannot be completed, because
the attacker picks the name**, so it is now a whitelist of the 47 identifiers `is_effect_active` may
contain; `r3` was additionally blind to a read added INSIDE an allowlisted function, which is
PB-DX49's own finding on its `r7` inherited without its fix, so the roster now stores COUNTS; `r6b`
fell to `let src_id = source;`, PB-DX51's `r1` failure verbatim. `r5` required a trailing paren and
missed a function POINTER — mitigated, and the half that held is worth naming: the behavioural
in-source probe DID redden, so only the gate was blind.

**Two new rows, and both were proven necessary by executing a defeat that the other one misses.**
`r1c` pins the per-function occurrence count of `effect.source` in `layers.rs` (a source read
hoisted above the arms under an unbanned name reddens it while `r1` axis 3 stays green); `r2c` pins
the new laziness classifier's `true` set against the arms that consume the view (mis-classifying one
filter silently hands it `None` — the pre-batch defect restored for one variant — and reddens
nothing else).

**Six CR citations were wrong and all six reproduce.** `CR 602.2c` **does not exist**; `CR 702.34`
is Flashback and Channel is an ability word (CR 207.2c) with no rule entry, so the cite is DELETED
rather than renumbered; `CR 114.1` was quoted for text it does not contain (*"neither a card nor a
permanent"* is CR 114.5) and for a claim **no rule makes at all** (an emblem never leaving the
command zone) — which is load-bearing, because `r7`'s emblem ratchet rested on it and now rests on
the engine measurement instead; `CR 118.12` is a RESOLUTION cost and was cited for activation costs;
`CR 611.2b` is the resolution-generated *"for as long as"* duration and was cited for a static
population (CR 611.3b); `CR 508.1m` was cited for the word *"nontoken"*, which is card text. **The
MCP rules server was not reachable from the fix session and that is stated rather than worked
around**: every rule was verified in `.scryfall-cache/MagicCompRules.txt`, the file the MCP server
indexes. **Jitte ruling #5 is now quoted in full wherever #3/#4 is** — #3 and #4 alone read as a
refutation of the LKI design, and #5 is what settles it.

**The eager source resolution is now lazy, and the benches are reported as measuring NOTHING here.**
Six runs, same-code band taken first (`sba_check` moves 5.3-7.5% on identical code). The verdict is
not "no regression" but "structurally unmeasurable": a `panic!` at the top of `effect_applies_to`,
env-gated, **did not fire on any of the six benches** and **did fire** on the in-source layer-walk
tests, so `effect_applies_to` is called zero times by this bench suite. Non-vacuity proven both ways
rather than assumed.

**Figures RE-TAKEN after the cycle, not inherited**: tests **5,196 / 0 / 5**, 66 targets (5,196 + the
two new rows). HASH **84** / PROTOCOL **43** gate-executed and UNMOVED. `clippy --workspace
--all-targets -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1,803 defs) and
`cargo build --workspace` all clean against the FINAL tree. No `Completeness` marker moved.

## Last Handoff (worker, 2026-09-04) — PB-DX52 / `scutemob-229`

**Task**: `scutemob-229` — PB-DX52, v4 queue rank 14. Branch
`feat/pb-dx52-bolt-bends-printed-or-ability-half-is-unreachable-an`, merge base `cecf0ba0`.
**Seeds**: `OOS-DX25b-1` (headline) and `OOS-DX25b-5` (rider) **CLOSED**, plus **`OOS-DX25c-3`
CLOSED** as a third — see below, it is the one that matters. Filed `OOS-DX52-1..9`.

**Shipped**: `Target::StackObject(ObjectId)` naming an ability's stack ENTRY by its own
`StackObject::id`, plus `TargetRequirement::TargetSpellOrAbility` (CR 115.1a/115.7d, ANY target
count — Deflecting Swat prints no *"with a single target"* clause and had no expressible form).
New `casting::validate_stack_object_satisfies_requirement`; `queries::legal_targets_per_slot` and
`retarget::retarget_candidates` both enumerate ability entries; `resolution::is_target_legal`
gains an existence-based CR 608.2b arm. PROTOCOL **42 → 43**, HASH **83 → 84**, one bump each.
Tests **5,117 → 5,156 / 0 / 5** on 65 targets. Coverage **UNMOVED at 63.2%**, 0 flips.

**The five things worth carrying into the next batch**

1. **A seed's "unreachable today" is a claim with an expiry date, and the batch that closes its
   blocker is the batch that has to honour it.** `OOS-DX25c-3` was filed as *"doubly unreachable,
   blocked behind `OOS-DX25b-1`"*. Closing `OOS-DX25b-1` makes an ability a reachable
   `Effect::ChangeTargets` victim, and `plan_target_change` derived CR 702.16b protection
   characteristics from `card_in_stack_zone`, which is `None` for every ability — so shipping the
   id space alone would have let Bolt Bend redirect a red ability onto a creature with protection
   from red. **Before closing any seed, grep the registry for rows whose stated blocker IS that
   seed.** Closed with `stack_registry::source_of` (CR 113.7, exhaustive, no wildcard).
2. **A revert row that reddens only a SOURCE gate is a coverage measurement, not a pass.** Row R6
   (undo the protection fix) reddened exactly one thing — `r7b`, a text-comparison gate. No
   behavioural probe moved. The fix this batch describes as its own near-miss was standing on a
   string comparison a later "simplify the helper, keep the name" edit would satisfy. Closed by
   `t10`; filed as `OOS-DX52-2` because the matrix convention currently scores such a row green.
3. **The CR can argue for the wrong design, and only a measured blast radius settles it.**
   CR 113.1c says an ability on the stack **is an object**, which argues for registering entries
   in `state.objects`. Measured across 241 walk sites and rejected: an entry there must claim a
   `ZoneId`, and `casting.rs`'s `TargetSpell` arm decides *"is this a spell"* by zone ALONE — so a
   registered ability becomes a legal *"counter target spell"* target. `state.objects` is the
   CARD-object map; CR 113 abilities live in `state.stack_objects`.
4. **Check your own CR cites against the rules server.** Four families of cite in this batch's
   first draft were wrong: CR 113.3 (cited 4× for a claim NO rule makes), CR 113.7a (cited 6× for
   CR 113.7's sentence), a bare CR 113.1 where CR 113.1c/110.1/102.1 was meant, and *"ceases to
   exist"* which is **CR 608.2n**. The pass happened because a delegated agent reported it had no
   MCP rules tools and flagged that rather than proceeding as if it had.
5. **A gate's own justification can rot, and the gate cannot see it.** `deflecting_swat`'s
   `RECORDED_BASELINE` entry quotes a sentence this batch DELETED, and kept passing because the
   def still matched the same needles. Nothing checks that an entry's quoted fragment still occurs
   in the def it names (`OOS-DX52-1`). Found by reading why the gate had NOT fired.

**Refuted premises of the dispatch brief / acceptance criteria, reported not skipped**
* AC 7352 predicts `tools/play-server/frontend` *"will"* move. It does **not** — `TargetPicker`
  echoes `.value` verbatim and never reads `.kind`. Zero frontend production lines; `npm run
  build` N/A.
* The brief's `mod.rs` is `main.rs` (SR-9a's entry point), twice.
* `retarget.rs`'s existing R6 parity gate cannot exercise the new stack tail — `GameStateBuilder`
  cannot populate `state.stack_objects`, so its fixture keeps that vector empty (`OOS-DX52-5`).
* `resolution::is_target_legal` is private and unreachable from `tests/`, so its CR 608.2b arm is
  kept in step with `check_condition`'s twin by TEXT, not execution (`OOS-DX52-4`).
* The v4 memo's *"1 deck-legal `Complete`"* is a FLOOR: four defs declare a stack-object
  requirement, and closing the seed is what makes Misdirection's spell-only restriction
  enforceable for the first time.

**Riders NOT taken, with reasons posted**: `OOS-DX25-4` (its "natural fit" premise is false —
this batch changed zero lines in `stack_registry`'s existing functions; its PREREQUISITE
`source_of` is now built) and `OOS-DX25b-4` (CR 115.7d is a player decision needing an
`EffectChoiceQuestion` variant — a second wire half). Both left for **PB-DX54**.

**Process failure of my own, disclosed**: I used `git add -A` twice while three delegated agents
were writing files concurrently, and commit `eb56ebd3` — message *"v4 memo row 14 struck"* —
silently swept up 1,209 lines of an agent's probe file. The tree is correct; the commit message
is not. `git add -A` is unsafe whenever anything else can write to the tree, which is exactly
the condition a delegating batch creates.

**Next**: **PB-DX39** (v4 rank 15 — source-relative filters through LKI, `OOS-DX5-3` +
`OOS-DX5-7`'s residual), task 2 of the user-approved five-task chain. Full record:
`memory/primitives/pb-DX52-execution-notes.md`.

## Last Handoff (oversight session, 2026-09-04)

**Date**: 2026-09-04 (coordinator/oversight session; the user-approved FOUR-task autonomous chain)
**Workstream**: W6 dispatch loop
**Tasks**: `scutemob-225` (PB-DX18) → `226` (PB-DX51) → `227` (PB-DX35) → `228` (PB-DX36),
dispatched sequentially, each collected before the next launched; merges `61f9d5e1`,
`275b00af`, `e8c212e7`, `d15692f7`.

**Completed**: v4 ranks 10-13 shipped (details in the four worker handoffs below — this entry
does not duplicate them). Tests 5,015 → **5,117 / 0 / 5**; PROTOCOL 41 → **42**, HASH 80 → **83**
(every bump predicted in writing before code); coverage 63.1% → **63.2%** (1,139/1,803) on two
named flips. 15 seeds CLOSED (incl. `OOS-CARDS2-6` filed-and-closed), 31 filed
(`OOS-DX18-1..6`, `OOS-DX51-1..7`, `OOS-DX35-1..9`, `OOS-DX36-1..9`). All four `/review` cycles
had every finding taken; PB-DX36's found two HIGHs by execution (per-assignment vs per-event
trigger firing, CR 603.2c; a class gate bypassable via `use` alias) — both fixed before merge.

**Not done / deferred**: PB-DX52 (v4 rank 14) NOT dispatched — the approved chain was exactly
four; next dispatch awaits user go. **↻ 2026-09-04 (evening session): user approved a FIVE-task
chain, ranks 14-18, "make sure only 5" — PB-DX52 → PB-DX39 → PB-DX53 → PB-DX54 → PB-DX42b.
PB-DX42b's dispatch-time precondition (re-word `OOS-DX19-2` per `OOS-ADJ-3`) is the coordinator's,
to be done before launch 5; PB-DX9 has NOT shipped, so its supply-census re-measure is not owed.** `urzas_saga` authoring (`OOS-RR4-2`) still open. The
pod-decks check-in (`docs/end-state.md`) still not started; user reaffirmed 2026-09-04 to
"keep going on this tack" (the correctness queue) for now.

**Next session candidates**: PB-DX52 (rank 14); the remaining correctness tier is ranks 14-21
(eight batches, ~two sessions at today's pace), after which the v4 memo's own instruction is a
v5 census/re-rank — ranks 22-41 are yield/hygiene with ~15 flips total, so treat rank 21 as the
natural pivot point to the pod-decks check-in + the stale-TODO triage (565 unclassified TODO
lines in `docs/authoring-status.md`).

**Hazards** (carrying forward):
- Coordinator chore commits on `workstream-state.md` made AFTER a worktree is created conflict
  at collect (happened on `225`). Commit the chore FIRST, then `esm worktree create`; or
  `git -C .worktrees/<id> merge --ff-only main` before launch (done for 227/228, no conflict).
- A Monitor "STALL? 30 min" line is NOT a stall while a delegated implementation agent runs —
  check `git log main..HEAD` + dirty mtimes in the worktree before reacting (three false alarms).
- A subagent inside a worker ran a shared-stash git command that wiped the worker's in-progress
  edit (PB-DX18; recovered from reflog). Subagent briefs in this repo must forbid git OUTRIGHT.
- Brief cites drift within a chain: four PB-DX36 cites moved after the three earlier merges.
  Re-derive cites at HEAD and post them as a task comment right before each launch.
- This file is ~7.6k lines with ~24 stacked worker handoffs — an archive pass is overdue.

**Commit prefix used**: `merge:` (worker collects), `chore:` (chain bookkeeping + this close)

---

## Worker Handoff (PB-DX36, `scutemob-228`, 2026-09-04)

**Shipped**: v4 rank 13. **`OOS-CARDS2-6` FILED — it had no registry row at all — and CLOSED, both
halves.** Filed `OOS-DX36-1..9`. Next dispatch is **PB-DX52** (v4 rank 14); ranks 1-13 all shipped.
**This was the LAST of the four-batch chain the user approved on 2026-09-04. STOP here — no further
dispatch without an explicit user go.**

### What was wrong

`TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { combat_only }` was dispatched inside
the `GameEvent::CombatDamageDealt` arm only, under a `TODO(PB-37)`, and the lowering destructured
`combat_only` away with `{ .. }` — the runtime `TriggeredAbilityDef` had no home for it, so the flag
was read in **exactly one place in the workspace: `state/hash.rs:6848`**. `true` and `false` were
behaviourally identical. `sigil_of_sleep` (`Complete` by derive, deck-legal) silently dropped the
noncombat half of its printed trigger. Separately there was no general *"whenever this permanent
deals damage"* condition and no damage-dealt `EffectAmount`, so `exalted_angel` was unauthored.

### What shipped

One shared `rules::abilities::queue_damage_source_triggers`, called from BOTH damage arms with
`is_combat` a property of the **EVENT**. Seven new `TriggerEvent` unit variants (one retired), the
new `DamageRecipient { Any, Player, Opponent }` axis on both conditions, new
`TriggerCondition::WhenDealsDamage`, new `EffectAmount::DamageDealt`, and `damage_dealt_amount`
carried on `PendingTrigger` / `StackObject` / `EffectContext`. `TODO(PB-37)` and its echoes deleted.

### Five things worth carrying into the next batch

1. **Do not delete a flag only the hasher reads until you have run the INVERSE census.** 0 defs
   *declare* `combat_only: true`; **1 def prints it** (`breath_of_fury`). The declared axis and the
   printed axis do not nest — third batch in this queue saved by running both.
2. **A brief is a claim, including its CR cites.** The task description and AC 7333 both cite
   **CR 603.10a** for *"that much"*. CR 603.10a is look-back-in-time **zone-change** triggers.
   Shipped against CR 603.2c + CR 608.2h/113.7a instead; recorded, not obeyed.
3. **`TriggeredAbilityDef` still costs 190 exhaustive literals across 44 files** (`OOS-DX35-1`'s
   figure, reproduced at HEAD). Any axis that could live on `TriggerEvent` should.
4. **A survivor scan has TWO axes.** PB-DX50's rule ("not the same regex as the re-pin") was obeyed
   — different SHAPE, line-window vs symbol-adjacent — and the scan still reported 0 while a
   `41u32` sentinel stood, because the **value** pattern `\b41\b` was unchanged. Prefer an
   absence-based check: find the symbol, assert the adjacent numeral is the NEW value.
   (`OOS-DX36-8`.)
5. **A COUNT assertion proves exactly-once only on the FIXTURE SHAPE it drives.** The `/review`'s
   HIGH 1: this batch dispatched the self family once per damage ASSIGNMENT, so a multi-blocked or
   trampling creature triggered twice against CR 603.2c — while the doc comment asserted
   exactly-once and every count probe drove a single-assignment fixture, including the one whose
   docstring boasts about not being a `>= 1` check. The emit-site census behind the claim was
   *correct*; it bounded the ARMS and said nothing about the LOOP INSIDE one arm.
6. **The re-deal budget is real and is attributable.** One marker flip reddened five seeded
   fixtures. An ablation in an isolated worktree (engine change in, marker reverted) turned all five
   green, which is what licences the phrase "fuzz-neutral by measurement". Two of the five needed a
   fresh **executed sweep**, not a bump: `pb_dx22`'s census seed 0 → 1 and
   `UI3_SPLIT_COMBAT_SEED` 13 → 26.

### Numbers

Tests **5,117 / 0 / 5** (+20, **64** targets, byte-exact NAME delta: 20 / 0 / 0 / 0, re-taken
AFTER the `/review` fix cycle rather than before it).
**PROTOCOL 41 → 42 / HASH 82 → 83**, one bump each, predicted per half before any code, closure
type counts confirmed UNCHANGED at **98 / 132**. Coverage **1,138 → 1,139 / 1,803 = 63.2%**, ONE
flip (`exalted_angel`) named before regeneration. `clippy -D warnings`, `cargo fmt --check` and
`tools/check-defs-fmt.sh` (1,803 defs) all clean against the FINAL tree; `npm run build` N/A.
Benches: six runs, **no regression**, same-code band measured first and wider than every difference.
Full record: `memory/primitives/pb-DX36-execution-notes.md`.

## Worker Handoff (PB-DX35, `scutemob-227`, 2026-09-04)

**Task**: `scutemob-227`, v4 queue rank 12. **Both seeds CLOSED** (`OOS-DX4-2`, `OOS-DX4-5`),
plus **`OOS-DP10-5`** CLOSED and **`OOS-DX8-3`** updated.
**Full record**: `memory/primitives/pb-DX35-execution-notes.md` (§0 is the pre-code prediction and
is immutable; §B is Half B's post-code record).

**Shipped, Half A** — one shared `rules::abilities::trigger_modal_plan` implementing CR 700.2b
(*"If one of the modes would be illegal … that mode can't be chosen"*) and slicing
`ModeSelection.mode_targets` through `casting::per_mode_target_requirements`, the SAME helper
`handle_cast_spell` and `queries::spell_target_requirements` call. All four trigger-path consumers
delegate to it. `shambling_ghast` `partial` → `Complete`; `retreat_to_kazandu` (the live-wrong
deck-legal member neither the seed nor the memo names) and `retreat_to_coralhelm` repaired in
place.

**Shipped, Half B** — `Effect::LookAtTopThenPlace.optional` stops being inert: `false` or an empty
candidate set keeps the deterministic take-when-able winner byte-for-byte, `true` with candidates
asks `EffectChoiceQuestion::ChooseObject { count: 1, up_to: true }` on the channel `place_cost`
already uses. **No new question variant**, so **zero fingerprint bumps for the whole PB** —
HASH **82**, PROTOCOL **41**, both gate-executed and both predicted in writing at `c6646052`
before any production line changed.

**Read these seven before the next batch touches this surface:**

1. **"Scope the targets to mode 0" — the option the brief offers — is CR-WRONG, and CR 700.2b
   says so in one sentence.** The engine may not choose a mode it cannot choose legal targets for.
   Taking that option would have left `retreat_to_kazandu` unable to gain 2 life on an empty
   board — which is the defect — and made the predicted flip a false claim. Legality-aware
   auto-choice is not scope creep here; it is the minimum correct behaviour, and it is what makes
   the criterion's own headline probe pass.

2. **The `ModeSelection` lookup and the target-requirement lookup read DIFFERENT INDEX SPACES, and
   three of the seven corpus modal triggered abilities fall in the gap.** A `Normal`-kind trigger's
   `ability_index` indexes the runtime `Characteristics::triggered_abilities`; both `ModeSelection`
   read sites index the registry `CardDefinition::abilities`. They agree only when no non-`Triggered`
   ability precedes the modal one. `hullbreaker_horror` (registry 1, behind `Keyword(Flash)`),
   `glissa_sunslayer` (2) and `junji_the_midnight_sky` (2) are misaligned. **Two symptoms, not one**:
   the first two resolve `Effect::Nothing` (the whole modal ability is a no-op), while junji's
   `WhenDies` is one of the three lowering arms that pre-resolve `modes.first()` into `effect`, so
   it executes **mode 0 forever** and the mode choice is a fiction. Zero deck-legal blast radius —
   all three are non-`Complete` — which is measured, and is why it is `OOS-DX35-1` rather than a
   fix. The fix is lowering `modes` into `TriggeredAbilityDef`: **190 exhaustive struct literals
   across 44 files plus both bumps**, because that struct has no `Default` derive and is reachable
   from `Characteristics`.

3. **That is also why the memo's second predicted flip did not happen, and the seed row predicted
   its own failure.** `OOS-DX4-2` warns that *"moving the targets into `mode_targets` looks like the
   CR 601.2c-correct repair and would silently DROP the requirement"*. For `hullbreaker_horror` that
   trap is still armed — not because the trigger path ignores `mode_targets` (it no longer does) but
   because the modes lookup cannot find its `ModeSelection`, so the slice falls back to the flat
   list the repair would have emptied. It is **re-adjudicated, not re-shaped**: `partial` kept,
   marker rewritten to name the surviving blocker. **A yield cell that names members is a FLOOR on
   the census and a CEILING on the flips** — `OOS-DX4-2`'s member list was short by more than double
   (5 of 7) while its two named flips delivered one.

4. **`OOS-DP10-5`'s standing instruction — *"Sweep for others not yet found"* — had been inherited
   unrun by nine batches. It was executed, and it found a live defect on seven deck-legal cards.**
   `Effect::CounterUnlessPays` destructures `cost: _` and delegates to `Effect::CounterSpell`, so
   CR 118.12a's *"unless its controller pays"* is never offered: `make_disappear`, `spell_pierce`,
   `stubborn_denial`, `mana_leak`, `izzet_charm`, `mana_tithe`, `flusterstorm`, **all `Complete` by
   derive**. Its in-source justification (*"the payer never has an incentive to voluntarily tax
   themselves"*) is **false on its face** — the payer is the OPPONENT whose spell is being
   countered. Filed as `OOS-DX35-3`; the fix needs no new question variant, only PB-DX45's shape
   addressed to a different player. The sweep also produced one **checked-and-CLEAN** result
   (`Replacement.unless_condition` IS consumed, at `replacement.rs:2044`), recorded because it is
   what proves the sweep read each discard rather than counting them.

5. **A test-name collision across two `tests/` binaries makes a byte-exact NAME delta undercount,
   and the byte-exact method cannot see it.** Half B named its bot-path probe
   `c3_the_bot_path_is_offered_and_answers_the_same_action`, which PB-DX45's channel file already
   used. One binary per file, so both compile and both run — and the close-out delta, a set
   difference over NAMES (the method `OOS-DX20b-5` mandates), collapses the pair: **32 additions
   against a count delta of 33**. The check costs one line and is the durable half: compare the
   name-set delta against the `passed + ignored` delta. `OOS-DX35-8`. The `cN_`/`tN_`/`rN_`
   convention makes this MORE likely, not less, since every batch starts its channel file at `c1`.

6. **Two seed IDs collided because the batch ran its halves as two delegated implementations and
   both claimed `OOS-DX35-1`.** Found at close-out by grepping the ID rather than trusting either
   report. The index-space defect keeps the number on **12** in-source cites against **1**; the
   `RevealAndRoute` residual is `-2` and its single cite was repointed in the same commit
   (`OOS-M11-10`'s renumbering orphaned 30 cites under a note asserting it had not). **A seed ID is
   allocated against the registry, and two workers on one task cannot both read a registry neither
   has written to yet.**

7. **Two of the coordinator's own published figures were wrong and the PRINTING TEST is what caught
   them.** The corpus-wide "you may" population was written into two registry rows as 213 / 90 from
   a throwaway script whose `oracle_text` extractor did not join Rust's `\`-newline continuations
   and silently truncated every multi-line string; the true figures are **365 / 165**, read off
   `core::pb_dx35_optional_placement_roster::t_census_report`. The pinned MDFC is also named
   `Turntimber Symbiosis // Turntimber, Serpentine Wood`, not its file stem. **PB-DX8's rule —
   publish the figure, do not transcribe it — caught its own author.**

**Numbers** (every one re-run by the coordinator, not accepted from the implementers): tests
**5,058 → 5,097 / 0 / 5**, **63** result-producing targets (61 → 63: two new simulator/engine test
binaries), byte-exact set difference **39 additions / 0 leavers / 0 removals / 0 renames** after
the rename in item 5, and the count delta (39) EQUALS the name-set delta with an empty
duplicate-name scan. **This cell has now published 5,091 and 5,096 before settling at 5,097, and
that is disclosed rather than quietly overwritten**: 5,091 was taken before the close-out added the
Half B census roster and 5,096 before the `/review` fix cycle added `r8`. That is PB-DX28's
"re-take the measured table" MEDIUM, twice, on a cell whose own preface claims every number was
re-run — the first catch was this batch's `/review`, the second was dispatch hygiene 8 applied to
itself. **A re-verification claim attached to a figure a later commit supersedes is worse than no
claim**, and the only reliable discharge is to re-take the number AFTER the fix cycle rather than
before it. Coverage **1,137 → 1,138 / 1,803 = 63.1%**, ONE flip, named before any code.
HASH **82** / PROTOCOL **41** unmoved. `clippy --workspace --all-targets -D warnings`,
`cargo fmt --check`, `tools/check-defs-fmt.sh` (1,803 defs) and `cargo build --workspace` all clean
against the FINAL tree. `npm run build` **N/A and said so**: `git diff main..HEAD --numstat --
tools/play-server/frontend` is EMPTY and `node_modules` is absent from this worktree.

## Worker Handoff (PB-DX51, `scutemob-226`, 2026-09-04)

**Task**: `scutemob-226`, v4 queue rank 11. **All three seeds CLOSED**: `OOS-DX21-4`,
`OOS-DX21-2`, and rider `OOS-DX21-5`.
**Full record**: `memory/primitives/pb-DX51-execution-notes.md`.

**Shipped**: `CombatState.had_attackers` — CR 508.8's own predicate as a monotone marker — set by
ONE new mutator `CombatState::add_attacker`, which is now the only production path into
`combat.attackers` and therefore serves both the CR 508.1 declaration loop and all four CR 508.4
entrant sites; `advance_step`'s skip reads `!had_attackers && attackers.is_empty()` instead of a
step-end `attackers.is_empty()`; one conjunct on `legal_actions.rs`'s `DeclareBlockers` offer
(CR 509.1a, SR-38); and the `CombatState::new` init moved below every `return Err` in
`handle_declare_attackers`. HASH **81 → 82**, PROTOCOL **41 unmoved**, both predicted in writing
at `06ba6760` before any production line changed.

**Read these six before the next batch touches this surface:**

1. **`OOS-DX21-4`'s reproduction recipe is wrong in all three of its named routes, and that made
   the defect BIGGER than filed.** *"Kill it / phase it out / stop it being a creature"* removes
   nothing — the engine implements **2 of CR 506.4's 6** removal causes (`OOS-DX51-2`). The route
   that reproduces is `reconnaissance` (`Complete`, deck-legal, `{0}`, instant-speed, repeatable),
   so this was **live on 2 deck-legal `Complete` defs**. *A row can be right about a defect and
   wrong about every way of reaching it — re-derive the reachability, not just the site list.*
2. **One field, not the "third piece of state" the row asks for.** CR 508.8 ORs its two facts in
   one sentence, so the predicate is a single existential. Two fields would have been two things
   to drift, and no CR rule separates them for this purpose. The empty-declaration case needs
   **no special case at all**, which is what makes one mutator serve both CR rules.
3. **This batch's own `r1` gate was defeated SIX TIMES across three drafts, and twice BOTH halves
   were blind at once.** A sixth site written `let map = &mut combat.attackers; map.insert(..)`
   left `r1` green while `r1b`'s exact-5 count stayed green too (it ADDS a site rather than
   replacing one). After the re-key, the `/review` defeated it again with a wholesale
   `*combat = CombatState { attackers, ..combat.clone() }` (the field appears with no leading dot)
   and with a second `&mut self` mutator on `CombatState` itself (`r1` exempts that file; `r1b`
   counts one literal name) — **both halves blind for the second time**. And the widening written
   for a third finding **did not fix it**, because `r1d` skipped allowlisted files WHOLESALE. **If
   you add a roster gate, plant a bypass in a spelling you did NOT think of first — and then have
   someone else plant one.** `OOS-DX51-6`; the residual is `OOS-DX51-7`.
4. **A second SR-38 hole sits on the same `if` statement** — the `DeclareBlockers` offer is made
   to the attacking player, whom the engine refuses. `is_active` is computed three lines away and
   used only by the attacker offer. Filed, not fixed (`OOS-DX51-3`).
5. **The fuzz movement was ATTRIBUTED, not excused.** A third run carrying the full engine change
   with ONLY the offer conjunct ablated reproduces the merge base byte-identically, so the engine
   half is fuzz-neutral **by measurement** and every bit of the HEAD-vs-base delta — including
   HARD 90 → 198 — is `OOS-DX21-6` reindexing. **Do this ablation; it is cheap and it is the
   difference between a measurement and an excuse.**
6. **PB-DX18's published close pin of 5,041 does not reproduce** (5,044 at a byte-identical `.rs`
   tree). Take your own baseline on your own branch before any edit — do not inherit the previous
   batch's published number (`OOS-DX51-5`).

**Standing gates added**: `core::pb_dx51_attacker_entry_roster` `r1`/`r1b`/`r1c`/`r1d`. `r1` is
the one to know — it polices every mutable path to `combat.attackers` (mutating method, `&mut`
borrow, whole-map assignment, `mem::replace`/`swap`/`take`) across the whole workspace's
production source, over-collecting on ALL receivers deliberately, with `r1c` re-checking each
allowlist entry's stated reason in source.

**Known open, filed**: `OOS-DX51-1` (the new field's `#[serde(default)]` is lossy in the
skip-happy direction — same class as `OOS-DX21-3`); `OOS-DX51-2` (CR 506.4, 4 of 6 causes
unimplemented — the biggest of the six and a real correctness seed);
`OOS-DX51-3` (the attacking-player blocker offer); `OOS-DX51-4` (`canonical_fixture()` never
populates `combat`, so the STREAM digest is blind to every `CombatState` field);
`OOS-DX51-5` (the non-reproducing test pin); `OOS-DX51-6` (**closed in the batch that filed it**);
`OOS-DX51-7` (**filed by the `/review` fix cycle** — `CombatState::attackers` is `pub`, so no
textual gate over it is closable; the compile-enforced version is ~160 sites and was not taken).

**Next dispatch**: **PB-DX35**, v4 rank 12 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row
12 — modal trigger targets + the inert `optional`, `OOS-DX4-2` + `OOS-DX4-5`). Ranks 1-11 shipped.

---

## Worker Handoff (PB-DX18, `scutemob-225`, 2026-09-04)

**Task**: `scutemob-225`, v4 queue rank 10. **All six seeds CLOSED**: `OOS-DP2-7`,
`OOS-DP2-4`, `OOS-DP2-8`, `OOS-DX2-4`, `OOS-DX2-1`, `OOS-M11-5`.
**Full record**: `memory/primitives/pb-DX18-execution-notes.md`.

**Shipped**: two new stored fields designed together for ONE HASH bump
(`GameState.pregame: PregamePhase`, which carries BOTH of CR 103.5's restrictions, and
`PlayerState.miracle_pending`); CR 103.5's mulligan cap derived from the same constant the
draw loop counts to; CR 601.2c rejection of targets on a targetless spell, in ONE place;
CR 702.47a splice targets (a new `AbilityDefinition::Splice.targets` field); a real seeded
shuffle at both `ShuffleIntoOwnerLibrary` sites, discharged AFTER the move; and one
`GameState::shuffle_library_seeded` with the PRNG pinned in-tree.

**Read these five before the next batch touches this surface:**

1. **The census was short by a whole mechanism and it is SPLICE.** No document in the chain
   names it. `AbilityDefinition::Splice` had no `targets` field at all, so closing
   `OOS-M11-5` REQUIRED shipping one — a batch that only added the rejection breaks the
   corpus's one splice card.
2. **44 of 46 CR 601.2c rejections are naked `ObjectSpec::card()` fixtures.** Measured by
   instrumenting the rejection before repairing anything. Architecture Invariant 9 makes
   that shape unreachable in a real game, so 42 green tests were resting on it — the
   `OOS-DX47-4` class at scale.
3. **`OOS-DP2-4`'s addendum names ONE re-permutation channel and there are TWO.**
   `Rng::random_range`'s sampling is as unpinned as `StdRng`. `rand` is now dropped from
   both crates, so the pin is structural rather than a convention.
4. **The `*_SEED` axis is the wrong axis for the PRNG re-deal.** The memo budgets "18+
   fixtures"; exactly ONE pin moved, because the simulator's opening deal is not one of the
   four sites. Attributed by an executed A/B.
5. **A re-pin regex can be too WIDE, and a survivor scan cannot see that.** New failure
   mode, opposite of the two this queue has recorded. `OOS-DX18-3`.

**Standing gates added**: `core::pb_dx18_trust_boundary_roster` r1/r1b/r2/r3/r4. `r1` is the
one to know — it enumerates every `ZoneChangeAction::Redirect` arm that MOVES the object and
requires it to discharge the CR 701.24 obligation with its own bound field, because the
consumers all destructure with `..` and **the compiler cannot enforce this**.

**Known open, filed**: `OOS-DX18-1` (splice offer/cast disagreement — SR-38, deliberately not
gated, pinned wrong-way-round, needs per-card target slots on `SpliceCostView` plus a
frontend change that could not be verified here: `node_modules` is absent);
`OOS-DX18-2` (`crates/engine/tests/rules/effects.rs` is still a one-byte `mod`'d module);
`OOS-DX18-3`; `OOS-DX18-4` (the activated/loyalty analogue of `OOS-M11-5`, population
UNMEASURED); `OOS-DX18-5` (the mulligan draw loop still short-draws in silence);
`OOS-DX18-6` (**filed by the `/review` fix cycle**: `CR 701.20` is used to mean *shuffle*
across the tree and it is *Reveal* — 23 occurrences at HEAD, and this registry had already
corrected the same number once at `:143` for a different wrong usage, which is how PB-DX18
inherited it and propagated it to 41 new lines before its own review caught it. Rides
PB-DX38).

**Benches**: a REAL uniform ~2.5-4.5% regression, four runs with the same-code band measured
first. `size_of::<PlayerState>()` 360 → 376. Stated rather than mitigated; both fields are
load-bearing state.

**A process incident worth recording**: the delegated fixture-repair subagent ran `git stash`
in this worktree. The stash stack is shared with the main checkout and every other worktree
(CLAUDE.md forbids bare `git stash` for exactly this reason), and it wiped both its own nine
files of work and an in-progress engine edit. Recovered with `git stash apply <sha>` + drop,
verified against the reflog, nothing lost. **A subagent brief for this repo should forbid git
outright**, not merely say "do not commit".

---

## Previous Handoff (oversight session, 2026-09-03 — preserved for chain context)

**Date**: 2026-09-03 (coordinator/oversight session; second of the two three-PB runs)
**Workstream**: W6 dispatch loop + user-directed docs side task
**Tasks**: `scutemob-220` (PB-DX49), `scutemob-221` (PB-DX50), `scutemob-222` (PB-DX20b) —
dispatched, monitored, collected, merged (`e135c78e`, `e457f931`, `2c7844f9`); plus
`scutemob-223`/`scutemob-224` (self-assigned docs: `docs/interactions/blood-moon-urzas-saga.html`,
merged `51ed0774` + `0ad90adf`).

**Completed**: v4 ranks 7-9 shipped (details in the three worker handoffs below — this entry
does not duplicate them). Docs side task: a shareable two-layer HTML deconstruction of corner
case #36 (table layer + engine-room layer, 5 inline SVG diagrams, dual-theme, CSS-only tab
toggle after `scutemob-224` fixed the iOS QuickLook no-JS case). CLAUDE.md secondary-docs
table gained a `docs/interactions/` row.

**Not done / deferred**: PB-DX18 (v4 rank 10) NOT dispatched — awaiting user go per the
2026-08-01 chaining retraction. `urzas_saga` authoring (`OOS-RR4-2`, the card half of #36)
still open.

**Next session candidates**: dispatch PB-DX18 (v4 rank 10); or the pod-decks check-in from
`docs/end-state.md` (decklists still not in repo, pod-coverage metric still uncomputable).

**Hazards carried forward**: this file is ~7.3k lines with ~20 stacked worker handoffs —
well past the 5-entry rotation window; a /cleanup-style archive pass to
`memory/archive/` is a candidate, not taken unilaterally here. `decision-point-audit.md`'s
`last_updated` stamp lags worker registry edits (bumped this close).

**Commit prefix used**: `merge:` (worker collects), `scutemob-223:`/`scutemob-224:` (docs), `chore:` (this close)

---

## Worker Handoff (PB-DX20b, `scutemob-222`) — a printed restriction the DSL could not say, and the one gate the compiler cannot replace

**Shipped**: v4 queue **rank 9**. **`OOS-DX20-10` ≡ `OOS-DX20-5` CLOSED as ONE defect**,
cross-cited — both rows named the same expressiveness gap, and the memo's §1 pairing was right.
Filed **`OOS-DX20b-1..7`** (`-6`/`-7` by the `/review` fix cycle). **Next dispatch: PB-DX18** (v4 rank 10).

**The defect.** CR 702.5a: *"Enchant is a static ability … The enchant ability restricts what an
Aura spell can target and what an Aura can enchant."* `imprisoned_in_the_moon` (`Complete`,
deck-legal) prints *"Enchant creature, land, or planeswalker"* and declared
`EnchantTarget::Permanent` — which also admits artifacts, enchantments and battles — because
`EnchantFilter` had `has_card_type` (ONE type) and `has_subtypes` (an OR over **sub**types) and
no OR over card **types**. `sba::matches_enchant_target`'s `Permanent` arm is a bare `true`, so
CR 704.5m would not clean up an illegal attachment either. PB-DX20 made the widened offer
human-reachable in the browser. Fix: `EnchantFilter::has_card_types: Vec<CardType>`, lowered onto
the **already existing** `TargetFilter.has_card_types` — no parallel OR mechanism was built, which
is what the memo's *"cheaper than the row implies"* cell predicted and it held exactly.

### Five things worth carrying forward

**1. "Three sites" was the right NUMBER and the wrong SHAPE, and only re-deriving it caught that.**
The v4 row's site cell says three. Re-derived at stage 0 before any code: **two ARITHMETICS and
three CONSUMERS**. `casting::enchant_target_to_requirement` and `sba::enchant_filter_matches` were
independent hand-written copies of one six-field predicate; the CR 303.4a gate, the CR 704.5m SBA
and `queries::spell_target_requirements` each already consumed one of them. A batch that patched
"three sites" one at a time would have carried the new field in **two copies**. Shipped as ONE
arithmetic: `casting::enchant_filter_to_target_filter` is the single lowering, `sba.rs`'s
predicate is deleted in favour of calling it and handing off to `effects::matches_filter` — the
same predicate `validate_object_satisfies_requirement` already runs on the cast path. **The
CR 303.4a gate's call is KEPT deliberately** (PB-DX20 put it there so cast-time and SBA-time
agree; that property now holds by construction), and R3+R10 measured it as **one-directional** —
it adds nothing in the accepting direction and is decisive in the refusing one, so a later batch
deleting it as "covered upstream" would be half right and half wrong.

**2. The compiler will not catch an eighth `EnchantFilter` field, and this was proven, not
assumed.** Adding a field produces **ZERO** compile errors workspace-wide: every construction site
— engine, tests, all 1,803 card defs — uses `..Default::default()`, and `#[serde(default)]` covers
deserialization. Executed twice (implement stage, then independently by the coordinator in an
isolated worktree): `cargo build --workspace` printed `Finished` and all ten behavioural probes
stayed green. `r5_every_enchant_filter_field_is_lowered` is the only thing that reddens, and
**R5b proves its second half separately** — planting the field *and* updating the pin while
leaving the lowering alone still reddens, on the *unlowered* assertion, because a field-list pin
alone is satisfied by the edit that hides the bug. Filed `OOS-DX20b-2`; **the class is not
`EnchantFilter`-specific** — `TargetFilter`, `TokenSpec` and every `Default`-constructed config
struct has it, and `OOS-DX28-1` and PB-DX43's `TOKEN_SPEC_FIELDS` are the same finding from two
other directions.

**3. The population was short by one, and the axis that found it is not the seeds' axis.**
`breath_of_fury` prints *"Enchant creature you control"* and declared `EnchantTarget::Creature`,
silently dropping the controller clause — named by neither seed row nor the memo cell, and needing
**no new expressiveness at all**. Repaired here. And the roster's own execution corrected the
batch a second time: the population needing a `Filtered` filter is **SEVEN, not the six** an
OR-or-controller substring axis finds, because `awaken_the_ancient` prints *"Enchant Mountain"* —
no OR, no comma, no controller clause — and still cannot be any bare variant. *A substring axis
would have pinned six and called it measured.* Both populations now pinned separately.

**4. Closing a seed can kill a NEIGHBOURING batch's row, and PB-DX49 had designed for exactly
this.** Pair A (`imprisoned_in_the_moon` × `binding_the_old_gods`) was reachable **only** because
of the over-wide `Permanent`. `r4a_pair_a_depends_on_oos_dx20_10` went red on schedule and was
**re-adjudicated, not deleted**: the death is COMPUTED from the intersection of the two card-type
sets, so a widening resurrects the pair loudly. Verified it vacates no behavioural coverage.
`ReachRow.enchant` also had to change type — `EnchantTarget::Filtered` carries `Vec`s and
`REACH_ROWS` is a `const` — and now pins a const-expressible card-type slice, which is the part
that decides reach, rather than degrading to a `{:?}` compare. **Take the lesson, not just the
outcome: a wrong-way-round pin is worth writing precisely because the batch that closes the seed
is not the batch that wrote it.**

**5. Two close-out methodology hazards, both of which produce a plausible wrong answer.**
(a) **PB-DX50's sentinel lesson recurred inside the batch that had it available.** The census
(47 HASH + 13 PROTOCOL, multi-line-aware) reproduced PB-DX50's corrected figures exactly — and
then the first re-pin regex replaced **2 of 47**, because the tree spells the sentinel `79u8` and
`\b` between `9` and `u` is not a boundary. Caught by an independent survivor scan with a
differently-shaped regex. *A re-pin is only as wide as the spelling its regex matched, and
"spelling" includes the literal's type suffix.* (b) **A NAME delta taken with `sort` + `comm`
under a UTF-8 locale is not a delta** — it reported 24 additions / **2** leavers, the extras being
two untouched tests present once with `... ok` in BOTH logs, because `sort` collates by locale and
`comm` compares byte-wise. **It fabricates a REMOVAL**, which is the single thing the criterion
exists to detect. Filed `OOS-DX20b-5`; use a byte-exact set difference.

### Standing hazards this batch reproduced or confirmed

- **`OOS-DX20b-1` (new, pre-existing, LIVE)**: `legal_actions.rs:1276` builds the
  `DeclareAttackers` eligible list from **raw printed** `obj.characteristics.card_types`, never
  `calculate_characteristics`. Once Imprisoned resolves, its Layer-4 `SetTypeLine(Land)` makes the
  enchanted permanent a Land and the offer layer keeps offering it as an attacker; the engine
  refuses. `status.tapped`, `Defender` and `Haste` are read from the same raw struct three lines
  away, so a **granted** Defender is equally invisible — the class is wider than the instance.
  Byte-identical under revert, so not this batch's. Pinned by CLASS and COUNT in `c4`.
- **Two revert axes are not enough when both are over-wide.** R-A and R-B could not redden the
  "no printed-legal target refused" half; without a third, UNDER-wide revert (`Creature`, the
  `OOS-DX20-5` shape) two of five channel rows would have been honestly UNDISCRIMINATED.
- **Two agents cannot run revert matrices on one card-def corpus.** Both delegated matrices were
  in flight simultaneously and `imprisoned_in_the_moon.rs` was observed flipping
  `Filtered` → `Permanent` → `Creature` across three consecutive `git status` calls. One agent
  detected it and moved its whole matrix into an isolated `git worktree` with its own
  `CARGO_TARGET_DIR`; that is the pattern to mandate up front next time, not to discover.
- **Bench claims still need the same-code control.** Base → HEAD alone read *"`sba_check` +1.2%,
  everything else 2-4% faster"* — and the second half is not something the change can cause.
  Benching the **same code twice** moved three benches by 1.2-1.5%, and a second merge-base run
  put `sba_check` **slower than either HEAD run** (4.1% spread on identical code). Report *"the
  same-code repeatability band is wider than the effect"*, never *"within the historical band"*.

**Full record**: `memory/primitives/pb-DX20b-execution-notes.md` (§0 stage-0 predictions committed
before any code; §6 the four bench runs; §7 the 16-row revert matrix). Plan:
`memory/primitives/pb-plan-DX20b.md`.

## Worker Handoff (PB-DX50, `scutemob-221`) — a target the engine never modelled, and a rule that is an exception

**Shipped**: v4 queue **rank 8**. **`OOS-DX25-1` CLOSED** and **`OOS-DX29-2` CLOSED** (its two
surviving halves — timing and the copy path; its `on_top`-channel half had already been closed by
PB-DX29 itself and the row said so). Filed **`OOS-DX50-1..11`**, of which **-1** and **-2** are
also closed here.

**The defect.** CR 702.140a: a spell cast for its mutate cost *"becomes a mutating creature spell
and **targets** a non-Human creature with the same owner as this spell."* The engine carried that
choice in `AdditionalCost::Mutate` and **never put it into `spell_targets`** — the only list
`GameEvent::PermanentTargeted` is derived from. So **Ward never fired on a mutate cast**, no
becomes-the-target trigger ever saw one, and the mutate validator checked zone, creature-ness,
non-Human and owner **and nothing else**: no hexproof, no shroud, no protection. Live on **6
deck-legal `Complete` defs** from the moment PB-DX29 made mutate human-reachable from the browser.
CR 702.140c compounds it: the over/under choice belongs at **resolution**, and the engine took it at
announcement, so an opponent learned it before responding.

---

### The four things worth carrying forward

**1. THE SEED'S OWN PRESCRIPTION WOULD HAVE MADE THE FIX CR-WRONG, AND ONLY READING THE RULE CAUGHT
IT.** `OOS-DX25-1` says to route the host into `spell_targets` so that *"CR 608.2b re-validation"*
sees it. **CR 702.140b is an explicit EXCEPTION to CR 608.2b**: *"As a mutating creature spell begins
resolving, if its target is illegal, it ceases to be a mutating creature spell and continues
resolving as a creature spell."* It does **not** fizzle. A batch that obeyed the seed would have
handed the host to the generic fizzle gate and regressed a behaviour the engine already got right.
It does not regress as shipped for a **structural** reason rather than a checked one — the fizzle
gate lives inside the `StackObjectKind::Spell` arm and `MutatingCreatureSpell` is a **disjoint** arm
with no gate of its own — which is exactly the kind of load-bearing accident a later batch deletes by
"unifying the two arms". Pinned by `t7`/`t7b`/`t7c`, each asserting the fallback fires **and** that
no `SpellFizzled` is emitted. *A prescription in a seed is a claim like any other; read the rule.*

**2. "ONE ARITHMETIC" IS AN IMPROVEMENT ONLY WHEN THE SURVIVING ARITHMETIC IS THE RIGHT ONE.** The
plan told the implementer to replace the CR 702.140b re-check's four hand-rolled conjuncts with the
shared `is_target_legal`. That function checks, for an object target, **only** that it is still in
its cast-time zone — so the shared thing was **weaker than the duplicated thing**, and delegating to
it would have *deleted* three checks in the name of removing duplication. Corrected before shipping;
site 2 is now `is_target_legal` **AND** `validate_targets_inner` re-applied to the recorded
requirement, which is strictly more than HEAD ever checked. The implementer then improved on the
coordinator's own follow-up by choosing `validate_targets_inner` — **literally the function the cast
path runs** — over the narrower per-object predicate, so cast-time and resolution-time legality are
the *same call*, not two predicates that agree today. `is_target_legal`'s zone-only reading is an
engine-wide CR 608.2b under-check, filed as `OOS-DX50-5`.

**3. A SITE LIST IS A FLOOR — AND THE MISSING SITE WAS THE ONE THAT WOULD HAVE BROKEN.** Both seeds
and the v4 row name two enforcement sites; there are **three**. The third is
`crates/simulator/src/legal_actions.rs`'s `non_human_own` offer enumeration — a fourth hand-rolled
copy of the predicate, reading **raw** `o.characteristics` rather than layer-resolved ones.
Tightening cast-time legality while it kept a looser predicate is *a clean offer followed by a
guaranteed refusal*: the SR-38 shape PB-DX29 gated Fuse to avoid, PB-DX44 re-created while fixing it
and PB-DX45 shipped — **this batch would have been the fourth in a row.** Fixed by routing it through
`queries::legal_mutate_hosts`, so the offer layer reads layer-resolved characteristics for the first
time. Its host set also had to become **per-CARD**, which no document anticipated: protection
(CR 702.16b) is a property of the *(source, target)* pair, so two mutate cards in hand can have
different legal host sets.

**4. A RULE WRITTEN DOWN IS NOT A RULE APPLIED — THREE INSTANCES, FROM THREE DIFFERENT BATCHES.**
(i) `mana.rs:878` said the CR 605.4a gate covers *"four asking effects"* while it checked **seven** —
the same sentence PB-DX45's `/review` caught one short in `effects/mod.rs`, whose fix corrected the
copy it noticed and never asked whether the sentence lived elsewhere. Fixed by **deleting** the
restated list so both copies point at the gate's own `NEEDLES`. (ii) `rules/engine.rs`'s
**obligation (8)**, added by PB-DX45, states the compile-forcing rule precisely and names exactly one
site; `effects::handle_answer_effect_choice` had **two** non-compile-forced traps ~30 lines apart, in
the file PB-DX45 was editing. (iii) `abilities.rs`'s `collect_permanent_becomes_target_triggers` said
the mutate target *"is never entered into `spell_targets`… this fix only takes effect once that gap
closes"* — **and PB-DX50 half 1 IS that gap closing**, so the comment outlived the commit that
falsified it, missed by both the batch and the review. *A claim corrected where it was noticed
rather than where it lives does not generalise, and neither does a rule.*

---

### What the `/review` found, and why the HIGH is the most useful thing in this batch

**1 HIGH / 1 MEDIUM / 2 LOW-MEDIUM / 3 LOW / 1 NIT — all eight taken, none declined.**

**The HIGH was caused by the coordinator's own instruction.** The `is_copy` guard added to the mutate
resolution arm — which the coordinator ordered, explicitly **overruling** the copy audit's advice to
defer it — shipped as an early `return Ok(events);`. The instruction was *"make it agree with
`resolution.rs:819`"*, and the implementation copied `:819`'s **condition** while dropping its
**control flow**: `:819` is an `if / else if` chain that FALLS THROUGH to the shared resolution tail.
A `return` there skips `check_triggers_with_timing`, `check_and_apply_sbas`, `flush_pending_triggers`
and `grant_priority_to_active_player`, leaving `priority_holder: None` with both players passed and
the spell stranded — **an unrecoverable game**, proven by execution. That is **PB-DP8's own recorded
lesson** — *a guard that returns early inherits the obligation of the statements it skipped* —
committed inside a batch that had the sentence available to it. **The batch's own `r4` gate stayed
GREEN throughout**, because it asserted only that the arm's body contains `stack_obj.is_copy`.

**Two of this batch's own gates were defeated by execution.** `r3` ("exactly one mutate
target-legality predicate in the workspace") was defeated **twice** — it polices the requirement's
**definition** across three named files and is blind to its **consumer**, and the consumer is where
all four historical hand-rolled copies lived. The CR 605.4a site census was defeated **two ways at
once**: it read three files while `abilities.rs` is also an `EffectContext` construction site, and
its needle was the assignment form while five sites use the struct-literal initialiser. Both re-keyed
and both defeats re-run RED.

**And the coordinator's registry edit destroyed a word.** The `OOS-DX29-2` closure split that row by
column, but it has carried **six cells in a four-column table since it was filed** — its own
`` `Entwine | Fuse | EscalateModes` `` uses unescaped pipes — so the edit appended to a fragment
ending `` (`Entwine `` and **overwrote the cell holding `Fuse`**. Repaired, pipes escaped, and the
incident recorded in the row itself. A sweep found **five** such rows; the other four are
deliberately **not** repaired (not this batch's rows, and a confident mis-repair is worse than a row
known to be malformed) and are filed as `OOS-DX50-11` with the gate that would have caught all five
and refused the bad edit. **This matters because the registry is machine-read** — PB-DX49 closed
`OOS-RR4-3` on exactly the finding that *the table a tool reads is not the prose a human reads*.

---

### Numbers (all re-verified by the coordinator, not accepted from the implementers)

* Tests **4,991 / 0 / 5**, **59** targets. Baseline **4,941 / 58**, measured on this branch before
  any edit and reproducing PB-DX49's close pin exactly. Delta by NAME: **53 additions, 3 leavers, 0
  removals, 0 renames**; the three leavers are PB-DX29's mutate trio (2 inversions + 1 re-home),
  each with a named `test_dx50_*` successor.
* **PROTOCOL 39 → 40 / HASH 78 → 79**, ONE bump each, **predicted per half before any code**
  (`595e4e28`) and gate-computed after. Type counts predicted and confirmed unchanged at **98 / 131**.
* **47 HASH + 13 PROTOCOL sentinels** re-pinned, **0 stale survivors** — the plan published 45 + 11
  from a same-line regex *while citing PB-DX45's lesson that a re-pin is only as wide as its regex*,
  and its first survivor check used the same regex. **A survivor check written with the same regex as
  the re-pin is not a check.**
* Coverage **1,137/1,803 = 63.1%**, **0 flips**, **0 card-def edits of any kind**.
* clippy / `fmt --check` / `check-defs-fmt.sh` (1,803) / `npm run build` all clean against the FINAL
  tree. **Benches not measured, so nothing claimed** — the two changes that could move anything both
  *remove* work, which is a reason to expect no regression, not a measurement.

### For the next batch

* **Next dispatch is `PB-DX20b`** (v4 rank 9), not `PB-DX51`.
* `OOS-DX50-3` (CR 707.10f / 608.3f unimplemented engine-wide) is the **bound four other filed rows
  rest on** — closing it makes `OOS-DX50-2`, `-8`, `-9` and half of `-7` live at once.
* `OOS-DX50-11` is gate-shaped and cheap: re-split every registry row and assert 4 cells.
* `Effect::CopySpellOnStack` has **zero** genuine declarations — both grep hits are comments. SR-36's
  failure mode for the **fifth consecutive batch** in this queue. Walk `all_cards()`, never grep.

## Worker Handoff (PB-DX49, `scutemob-220`) — the rule the seed quoted is not the rule that applies

**Shipped**: v4 queue **rank 7**. **`OOS-RR4-1` CLOSED** and rider **`OOS-RR4-3` CLOSED**. Corner
case **#36 GAP → PARTIAL** — the engine half of the corner-case audit's last open GAP closes, and
#36 is deliberately **not** marked COVERED, because the card half (`urzas_saga` authoring,
`OOS-RR4-2`) is genuinely still open and this batch did not take it.

**The defect.** Every CR 714 decision — *is this a Saga*, *what is its final chapter number*,
*which chapter abilities does it still have* — was taken independently at five sites, each by
reading the **printed** card definition and none by consulting the layer axis. A permanent whose
abilities were blanked kept accruing lore counters (CR 714.3b), kept firing chapter triggers
(CR 714.2b) and was sacrificed anyway (CR 714.4).

**Shipped shape.** `layers::abilities_are_blanked` is now **the** ability-blanking predicate — CR
708.2a face-down plus the continuous-effect scan, with classification delegated to PB-DX43's
exhaustive no-wildcard `modification_blanks_abilities` so a fourth channel is a compile error. IG-1
in `queue_carddef_etb_triggers` was refactored to consume it, so **exactly one such predicate exists
in the tree** (verified by enumeration across engine, simulator, view-model and `tools/`).
`rules::saga::saga_view` answers every CR 714 question once, for all five sites.
`resolution.rs`'s two chapter-effect lookups are deliberately **not** consumers (CR 113.7a) and say
so at each site, so a successor cannot "finish the job".

### Read this before taking anything adjacent

**1. The seed's prescription would have made the fix CR-wrong, and only reading the rule caught it.**
`OOS-RR4-1` treats a blanked Saga's surviving ETB lore counter as part of the defect. **CR 714.3a
has no *"with one or more chapter abilities"* clause** — 714.3b and 714.4 both do. CR 613.1f removes
abilities, **not subtypes**, so a Layer-6-blanked permanent is still a Saga and still takes its
counter; only CR 708.2a (*"no text, no name, **no subtypes**"*) makes a face-down permanent not a
Saga. Suppressing it produces a **second** wrong outcome: an un-blanking would fire chapter I
instead of resuming at chapter II. **Site 4 asks two questions, and the query answers them from two
fields.** If you touch CR 714, read the rule, not the seed.

**2. A grep for a variant name is not a census of a behaviour.** Every blanker figure in this
seed's chain (13, its own corrected 9, and this batch's own orientation 6) grepped the string
`RemoveAllAbilities`. PB-DX43 moved CR 305.7's ability loss into `SetLandTypes`, so **both moons are
blankers again through a variant no such grep can see**. The measured population is **11 / 8** and
comes from *calling* `modification_blanks_abilities`. **The deck-legal 8 agrees with the row by
coincidence of totals, not of membership** — a batch checking only the total would have recorded the
row as confirmed. Saga side is likewise **3**, not 4 (`song_of_freyalise` names `SagaChapter` only in
`// TODO`s and its `inert` note): **SR-36's failure mode for the fourth consecutive batch in this
queue.**

**3. `OOS-DX49-1` is LIVE on a deck-legal `Complete` def and is deliberately UNPINNED.**
`binding_the_old_gods`' chapter I destroys nothing: `SagaChapter` is never lowered into
`chars.triggered_abilities` (`grep -c SagaChapter crates/engine/src/rules/abilities.rs` → **0**), so
`flush_sorted`'s requirement lookup returns empty, no CR 603.3d announcement happens, and
`DeclaredTarget { index: 0 }` resolves at nothing. Found **by execution while looking for something
else**. No probe was written, on purpose: a probe asserting today's behaviour would have to be
inverted by whoever fixes it. **PB-DX49 decides which chapters exist; this is what a chapter that
exists can target.** They are different subsystems and nothing here touches that path.

**4. `GameStateBuilder::build()` defaults to `Step::PreCombatMain`** — the exact state a
settle-detecting drive hunts for. This batch's bot-path probe satisfied *"settled at turn-1
precombat main"* **before issuing a single command**. PB-DX48's shape through a different door: the
vacuity came from the fixture's default, not the drive's endpoint. Closed here three ways; filed as
a class in `OOS-DX49-8`, because every `GameStateBuilder` fixture inherits the same default. **If
you write a drive-until-settled probe, assert the drive has not already arrived before the loop.**

**5. Two standing gates fired on this batch's own work and both were right.** The ability-definition
registry's `SagaChapter` site roster (four files → `saga.rs` + `resolution.rs` — the refactor's own
success signal), and SR-25's `bare_lookup_ratchet` on `sba.rs`, whose ceiling was **lowered** 7 → 6
rather than left stale-high. *A stale-high ceiling is slack a regression hides in.*

**6. One claim in CLAUDE.md's own PB-DX48 narrative was refuted and corrected in place.**
`KeywordAbility::Cloak` **does** exist (`card-types/src/state/types.rs:1696`, discriminant 157).
PB-DX48's conclusion and its measurement both survive — zero corpus defs declare it — but the stated
*reason* was wrong, and a reason is the half the next batch reuses.

**Numbers.** Tests **4,941 / 0 / 5** (+41 over the 4,900 pre-edit baseline, **58** targets; 41
additions / 0 removals / 0 leavers / 0 renames by NAME). **PROTOCOL 39 / HASH 78 both gate-executed
and UNMOVED**, predicted in writing before any code. Coverage unmoved **63.1%**, 0 flips, **0
card-def edits**. clippy / fmt / check-defs-fmt clean against the FINAL tree. **Benches: a REAL ~1.7% regression on `sba_check` and both `priority_cycle` benches**, published as
one rather than as "in band" — the `/review` refuted this batch's first, branch-only claim by running
the merge-base A/B it had not. `full_turn_4p`/`full_turn_6p` are noise and `board_wipe_4p` is 5%
FASTER. See the execution notes §6.2 for the matched-set table and why the residual is inherent to
the mandated design.
Filed **OOS-DX49-1..9**. Full record: `memory/primitives/pb-DX49-execution-notes.md`.

---

## Worker Handoff (PB-DX48, `scutemob-219`) — emitting the event is not dispatching it

**Shipped**: v4 queue **rank 6**. **`OOS-ENG2-1` ≡ `OOS-ENG2-2` FILED and CLOSED** (cross-cited —
`-2` is `-1`'s site census, not a second finding), **`OOS-ENG2-3` FILED and NARROWED**. None of the
three had a registry row: they were filed into ENG-2's handoff prose, which is the 61-of-208 blind
spot the v4 re-rank measured. Dispatch hygiene 5 held — grep first, then file, then close.

**The census is EXACT, and that is the rare part.** Re-verified at HEAD by the inverse method (all
`push_target_announcement` sites minus the `PermanentTargeted` emitters, never by trusting either
list): **12 = 3 emitters + 5 missing + 4 structurally target-free**. After three consecutive
batches in which the filed site list was a floor, this one reproduces without correction.

**The two things neither seed says, and they are the whole batch.**

1. **Emitting the event is necessary and NOT sufficient.** `check_and_flush_triggers` scanned a
   command's events and only THEN called `flush_pending_triggers`, so the events a flush itself
   produced were fed back to nothing. A batch that took the rows at their word ships a diff that
   looks like a fix and has a behavioural delta of **zero** at the headline site.
2. **The design was wrong TWICE, and neither correction came from argument.** A hook inside
   `flush_sorted` fired Ward **twice** (observed: two `AbilityTriggered`, two ward stack objects) —
   `Command::ChooseTriggerTargets` re-scans the very events it dispatched. Moving the fixpoint into
   `check_and_flush_triggers` was green on the full suite AND on an end-to-end probe and was still
   short: **`Command::PassPriority` never calls it**, so a targeted ETB trigger placed during a
   spell's *resolution* — the ordinary case — still dispatched nothing. Measured both ways:
   emission **1**, ward on stack **0**. A third path (`handle_concede` → `drop_departed_trigger_flush`,
   CR 800.4d) was found by enumerating callers a third time.

**Shipped shape**: `rules::events::permanent_targeted_events` derives the CR 702.21a payload ONCE
and `push_target_announcement` emits both halves, so all 12 sites dispatch and the three hand-rolled
loops are gone; `abilities::dispatch_becomes_target_waves` is a bounded fixpoint with an
**exactly-once scan cursor**, called from `flush_pending_triggers` and from `handle_concede`, and
deliberately NOT from `resume_trigger_flush` (whose events are already swept — calling it there is
wrong design 1 again). The asymmetry that leaves is `OOS-DX48-3`, and R-B **demonstrates** it:
`c3` is the only channel probe that stays green under a single-wave revert, because its trigger
suspends and is placed by `resume_trigger_flush`.

**Five corrections to this batch's OWN census, all from walking `all_cards()` instead of a grep** —
SR-36's rule, broken by my own brief, one batch after PB-DX47 filed `OOS-DX47-2` for the identical
thing. Ward-declaring population is **4**, not 5 (`vein_ripper` names the variant only in a
`// TODO`). `WhenBecomesTarget` has **1** structural declaration, not 6 (five are comment mentions).
And two LIVE finds where the brief said latent: **`KeywordAbility::Cloak` does not exist** (Cloak is
`Effect::Cloak`), so `cryptic_coat` — `Complete`, deck-legal — puts a face-down permanent on the
battlefield with ward {2} and no Ward trigger (**`OOS-DX48-4`, LIVE**); and an INVERSE oracle axis
found **`brutal_cathar`**, `Complete` and deck-legal, whose back face prints *"Ward—Pay 3 life"*
with no Ward mechanism (**`OOS-DX48-7`**). The three deck-legal `Complete` Ward defs the rank rested
on reproduce exactly.

**Both delegated revert matrices were RE-EXECUTED rather than accepted, and one was wrong.** The
channel suite reported "3/3 RED" truthfully while every probe panicked on the **journal**
assertion its own comment calls *"corroboration, not the verdict"* — `damage_marked == 0` stayed
TRUE under the revert, because the drive ran past CR 514.2's Cleanup, which erases the evidence
either way. Repaired to stop the instant the trigger chain settles; re-executed, all three now fail
on the damage assertion and `c2` reports `left: 1, right: 0`. **"All rows RED" is a true sentence
the wrong assertion can produce; the check that costs one command is reading the PANIC LINE.**

**AC 7252's "ward cost paid" branch is UNREACHABLE at HEAD and is reported, not narrowed.**
`Effect::MayPayOrElse` discards its `cost` and `payer` and always applies `or_else`, so Ward can
only ever counter. Blocker read off the source: `EffectChoiceQuestion::PayOptionalCost`'s payload
cannot distinguish a `MayPayOrElse` ask from a `MayPayThenEffect` one and its default is a hard
`pay: true` under a comment already calling the alternative "a different batch". **Zero deck-legal
`Complete` card defs use the variant** — Ward is its only live consumer — so the fix is bounded but
needs a wire bump. Filed `OOS-DX48-2`.

Tests **4,900 / 0 / 5** (+27 over the 4,873 pre-edit baseline, **57** targets), delta by NAME:
**27 additions, 0 removals, 0 leavers, 0 renames** — with the disclosure that the ENG-2 deviation
pin was inverted **IN PLACE**, so "0 leavers" must not be read as "nothing was touched".
**PROTOCOL 39 / HASH 78 both gate-executed and UNMOVED**, predicted in writing before any code.
Coverage **1,137/1,803 = 63.1%**, **0 flips**, churn reverted, **0 card-def edits**;
`crates/card-defs`, `crates/card-types`, `crates/view-model`, `crates/simulator/src` and `tools/`
are all **zero**, so `npm run build` is N/A. **The fuzz half of the movement budget DID come due**:
HARD **185** unmoved with both sub-checks and both game lists identical, but TRANSIENT
`no_orphaned_tokens` **273 → 275** and **+20 rejections, all twenty inside one game of twenty**.
One divergence; everything else downstream of it.

**The `/review` found 9, all 9 taken (8 fixed, 1 declined with its reason), and THREE of this
batch's own gates fell to mutations it ran.** One MEDIUM is a defect in the shipped engine:
`dispatch_becomes_target_waves` tested suspension at the TOP of its loop, so a batch's **prefix**
lost Ward entirely — and the loop's own comment asserted the resumed call covered it, which was
false in both halves. Fixed by the queue-then-stop ORDER; `t9` pins it, RED under the first draft
with the emission assertion staying green. `r2` fell to **field order** (Rust does not constrain it;
the docstring named only the residual it had thought of and called it "measured"); `r1` fell to a
**duplicated call** collapsing in a set — which IS the Ward-fires-twice defect — and to a hardcoded
six-file list while `push_target_announcement` is `pub(crate)`, mattering concretely because
`OOS-DX48-6` names `effects/mod.rs` as the next dispatch site. All re-keyed on the mechanism,
`SITE_SRCS` deleted, every defeat re-run RED. Two doc MEDIUMs: the v4 memo's row-6 strike still said
the budget "did NOT come due" (written before the fuzz A/B, never re-taken — PB-DX45's own MEDIUM),
and the published engine line count `+235/−61` did not reproduce **twice** (it is **+267/−61**).

**Durable lesson for the next batch that delegates.** Both delegated revert matrices reported "all
rows RED" and one was wrong about which assertion made them red — the channel probes' verdict was
vacuous because the drive ran past CR 514.2's Cleanup. **A matrix reports a row's colour, not which
line coloured it**; re-running the revert and reading the PANIC LINE costs one command and is the
only thing that separates the two.

**Next dispatch: PB-DX49** (v4 rank 7 — every Saga site reads the printed def, `OOS-RR4-1` +
`OOS-RR4-3`). Ranks 1-6 are all shipped.

Full record: `memory/primitives/pb-DX48-execution-notes.md`.

---

## Worker Handoff (PB-DX47, `scutemob-218`) — one dispatcher per trigger

**Shipped**: v4 queue **rank 5**. **`OOS-DX24-4` CLOSED**, with four corrections recorded in its
own registry row. Filed **`OOS-DX47-1..7`**.

### The probe came back with the LARGE answer, and it ran first

This was a probe-first batch: the seed was MEDIUM confidence and the memo blessed the small
outcome. The measurement was committed at `bb5a2f8e`, **before a line of engine source changed**.
`crates/simulator/tests/pb_dx47_double_push_probe.rs` builds through `setup::build_initial_state`
— the **production** pregame path, deliberately not `GameStateBuilder`, because the false comment
under test claims the hand-built path is the special one, so a hand-built fixture would have proven
nothing. Both seats human (no bot RNG). Subject `drana_liberator_of_malakir`: `Complete`,
deck-legal, and **legendary**, so CR 903.6 puts it in the command zone by construction instead of
leaving the probe to a shuffle, and its trigger puts a `+1/+1` counter on each attacking creature
so a double dispatch is visible on the BOARD, not just on the stack.

Measured: `check_triggers` pushed **`{CardDefETB: 1, Normal: 1}`** for one `CombatDamageDealt`, and
a card printing **ONE** counter put **TWO** on its lone attacker.

### Three things worth carrying forward

1. **A boundary census of `state.pending_triggers()` measures ZERO and reads exactly like
   "nothing happened."** The flush runs inside the same `process_command`, so the field is never
   non-empty where a test can look. What caught it was an end-to-end assertion running *beside* the
   census — the census said nothing while the board said twice. Any future batch reasoning about
   trigger counts from outside the engine will reach for that field first; call `check_triggers`
   instead (`OOS-DX47-1`).
2. **A roster typed from `grep -l` over `crates/card-defs/src/defs/` counts `// TODO` comments.**
   This batch's first-draft member list was **30 files**; the `all_cards()` walk is **26 defs**.
   That is SR-36's own rule broken inside the batch whose subject is a false comment. It was caught
   only because the gate re-derives from the compiled corpus rather than trusting the constant
   beside it — **a pinned roster and its derivation must not share a source** (`OOS-DX47-2`).
3. **A fixture that builds a NAKED object** (`ObjectSpec::creature(..).with_card_id(..)`, never
   through `enrich_spec_from_def`) **tests a shape production cannot produce.**
   `pbd_damaged_player_filter`'s Throat Slitter probe was one, and was the only reason
   EF-W-MISS-10's justification for the deleted scan looked live. The population of other such
   fixtures is **UNMEASURED**; the cheap census is a walk of `ObjectSpec::` constructors in
   `crates/engine/tests/` paired with `.with_card_id(` and not `enrich_spec_from_def`
   (`OOS-DX47-4`).

### The `/review` defeated the class gate, and that is the most useful thing in the batch

1 MEDIUM / 1 LOW-MEDIUM / 4 LOW / 1 NIT, **all seven taken, none declined**. The reviewer had a
shell, independently reproduced every published figure, and then **re-created `OOS-DX24-4`
verbatim with all nine roster gates GREEN** — by writing the second dispatcher in the BINDING form
(`let AbilityDefinition::Triggered { trigger_condition, .. } = ability` then `matches!`) instead of
the struct-pattern form `r3`'s parser matched. Only the behavioural probe caught it.

That form is used in this tree already (`collect_graveyard_carddef_triggers`), on two conditions
that ARE in the lowered set — so the gate's header claim was false about the exact family it could
not see. **A gate written for one variant measures that variant**, for the fourth time in this
queue.

`r3` now keys on the MECHANISM (every `TriggerCondition::X` within 3,000 bytes of an ability-list
walk, across five `rules/` files, 6 → 17 conditions), deliberately over-collecting, with each of
the three false positives carrying a named mechanism **and a companion assertion that the mechanism
still exists in source**. **Carry this forward: an allowlist entry whose reason is not machine-
checked is a comment, and this batch exists because of a comment.**

### Numbers

Tests **4,873 / 0 / 5** (+12 over the **4,861** pre-edit baseline measured on this branch before
any edit, reproducing PB-DX45's close pin exactly), `--workspace --no-fail-fast` to a file, **56**
result-producing targets (55 → 56: one new simulator test binary), residual list empty. **Delta
itemised by test NAME: 13 additions, 1 leaver, 0 removals.** The leaver is **disclosed rather than
netted out and is not a removal** — PB-DX24's Q4 probe was INVERTED, because what it pinned is what
this batch deleted (`OOS-DX47-5`).

**PROTOCOL 39 / HASH 78 both gate-executed and UNMOVED**, predicted in writing before any code with
the reason stated (a suppression adds no type, variant or field to the wire closure). Coverage
unmoved **1,137/1,803 = 63.1%**, **0 flips**, **0 card-def edits of any kind**.
`clippy --workspace --all-targets -D warnings`, `cargo fmt --check` and `tools/check-defs-fmt.sh`
(1,803 defs) all clean **against the final tree**. **10 revert rows, 10 RED, 0 UNDISCRIMINATED**,
with three green-under-revert rows disclosed as such.

Census, printed by `core::pb_dx47_dispatch_path_roster::t_census_report`: **26** defs declare the
trigger, **18** deck-legal `Complete` (the v4 memo's conditional figure reproduces exactly); **20**
`Complete` defs print it without declaring it (inverse axis, ratcheted); the class sweep intersects
**34** lowered conditions with **6** registry-scanned ones and the intersection is empty but for
one allowlisted post-filter.

**Next dispatch: PB-DX48** (v4 rank 6 — Ward never fires on a triggered ability, `OOS-ENG2-1` ≡
`OOS-ENG2-2` + `OOS-ENG2-3`). Ranks 1-5 are all shipped.

Full record: `memory/primitives/pb-DX47-execution-notes.md`.

---

## Worker Handoff (PB-DX45, `scutemob-217`) — CR 118.12's optional cost is the player's

**Shipped**: v4 queue **rank 4**. **`OOS-DX24-9` and `OOS-DX27-5` CLOSED as ONE defect**, each
registry row cross-citing the other and each carrying corrections to its own claims. Filed
**`OOS-DX45-1..8`**.

Tests **4,861 / 0 / 5** (+26 over the **4,835** pre-edit baseline measured on this branch before
any edit, which reproduced PB-DX15a's close pin exactly), `--workspace --no-fail-fast` to a file,
**55** result-producing targets (54 → 55: one new simulator test binary). **Delta itemised by test
NAME by set-diffing the two run logs: 26 additions, 0 renames, 0 leavers, 0 removals** — 12 in the
new `crates/engine/tests/primitives/pb_dx45_optional_cost.rs`, 6 in the new
`crates/engine/tests/core/pb_dx45_may_pay_roster.rs`, 3 in the new
`crates/simulator/tests/pb_dx45_optional_cost_channel.rs`, and 5 in
`tools/play-server/src/main.rs`'s `#[cfg(test)]` module. **PROTOCOL 38 → 39 / HASH 77 → 78**, one
bump each, both predicted in writing before any code and both taken from the failing gates' own
output. Coverage **1,136 → 1,137 / 1,803 = 63.0% → 63.1%**, the single flip predicted and NAMED
before regeneration. `clippy --workspace --all-targets -D warnings`, `cargo fmt --check` and
`tools/check-defs-fmt.sh` (1,803 defs) all clean **against the final tree**.

**Next dispatch (as of PB-DX45): PB-DX47** (v4 rank 5). *(↻ SHIPPED `scutemob-218` 2026-09-02;
the live next dispatch is PB-DX48 — see the handoff above.)*

### The headline: the site list was short by one, and the compiler could not say so

`effects/mod.rs` has **two** callers of `try_pay_optional_cost`, and every document in this
batch's chain — both registry rows, the v4 memo's rank-4 row, the dispatch brief — names only
`Effect::MayPayThenEffect`. The other is `Effect::LookAtTopThenPlace`'s `place_cost`
(`effects/mod.rs:6365`): the identical CR 118.12 decision one function over, live on a deck-legal
`Complete` def (`birthing_ritual`), whose auto-paid sacrifice also parameterises the mana-value cap
on what it may then cheat onto the battlefield. Both now ask. The scope line is stated so a reader
can check it rather than infer it: **PB-DX45 repairs every caller of `try_pay_optional_cost`, not
every printed "you may pay"** — which is what puts the second site IN and three never-charged
`Complete` defs (`OOS-DX45-3`) OUT.

### The three figures that did not reproduce

1. **The v4 memo's 11 deck-legal `Complete` defs is 10** (`OOS-DX45-2`). Re-derived at HEAD by two
   independent routes; no member's marker had moved since **before** the memo's census closed. The
   memo offered "two independent measurements both returned 11" as the PROOF that the two rows are
   one defect. They are one defect; the evidence was two agreeing wrong numbers. Six batches have
   taught this queue that a member list is a FLOOR — **this is the first recorded OVER-count**, and
   the durable correction is that a census figure is an estimate in both directions.
2. **`OOS-DX27-5` says two defs were left `partial` "on the same shape". Only one was.**
   `ruthless_technomancer`'s marker names its **activated** ability's missing variable-X sacrifice
   cost, a different and still-live gap. So the policy re-adjudication is ONE flip, not two — and a
   batch that took the row at its word would have promoted a def whose real blocker is live.
3. **`pb_dx32_fuzz_output.rs`'s `MOVED_MSG` predicts five named sibling gates "will redden
   alongside this one" on a `CORPUS_COMPLETE` move. None did.** Exactly one seeded pin in the whole
   workspace moved (`UI3_SPLIT_COMBAT_SEED`, 32 → 13, re-observed by an executed sweep). PB-DX26's
   lesson runs both ways: an unstable count is not necessarily an unstable deal.

### The defect this batch shipped, and the obligation it added

`play-server`'s `api::validate_decision_params` matched `(question, answer)` with a trailing
`_ => Err("… the answer given is a different kind")`. That wildcard was written to mean *wrong
question* and silently also served as the fallback for *unknown question* — so **every legal
`PayOptionalCost` answer 400'd and the browser was offered a `Confirm` picker whose Confirm AND
Decline buttons both failed.** A clean offer followed by a guaranteed refusal is the SR-38 shape
PB-DX29 gated Fuse to avoid and PB-DX44 recreated while fixing it; **this is the third instance**.

Eight consumers had to learn the new variant. **Seven were compile errors. The eighth was the one
that broke.** Fixed structurally — the match now dispatches on `question` alone, exhaustive with no
wildcard — and `rules/engine.rs`'s `BlockingDecision` obligation list gains **obligation (8)**:
*a wildcard arm that encodes a JUDGEMENT cannot also be the fallback for the UNKNOWN, and seven
compile-forced sites are not evidence that the eighth is safe — they are the reason nobody looks
for it.*

### Also worth knowing before the next batch

* **The decline is asserted by RESOLUTION EFFECT everywhere**, never by the offer — the Traitor's
  zone and the floating mana — because before this batch declining was not a reachable state, so an
  offer-shaped assertion would pass on an engine that asked and threw the answer away.
* **`default_effect_choice_answer` returns `pay: true` deliberately.** It is the exact recovery of
  the pre-batch auto-pay, which is what keeps every bot game, the fuzzer and every pre-existing
  golden script behaviourally identical while only the command trace grows. "Decline by default"
  would look safer and would be a behavioural change to every bot game in the tree.
* **`clippy::large_enum_variant` fired on this batch's own enum** (`PayOptionalCost { cost: Cost }`;
  `Cost::Sacrifice(TargetFilter)` makes `Cost` ~296 bytes). `Cost` is `Box`ed — `Box<T>` serializes
  and hashes transparently as `T`, so the WIRE shape is unchanged, but the **declaration text is
  not**, and both fingerprints had to be re-taken. The version numbers never moved twice.
* **A "re-pin by symbol" is only as wide as the spelling the regex matched.** The first pass caught
  44 files and left two behind that spell the assertion across a line break.
* **`OOS-DX45-8`**: the corpus re-deal exposed an SR-38 provider/engine disagreement at fuzz seed
  46 that a previous 0..80 sweep drove clean. Not diagnosed; the sweep was bounded below it and the
  chosen seed (13) does not depend on it.

### ↻ The `/review` cycle (7 findings, all seven taken) — and it found a SECOND silent gate

Figures unchanged by the fix cycle: **4,861 / 0 / 5**, 55 targets, the same 26-name delta, and
zero tests added or removed by the cycle itself.

**The finding that would have cost a future batch.** `test_dp9_mana_ability_gate` asserts that no
`Complete` def puts an asking channel inside a mana ability — CR 605.4a leaves no room to announce
there, so `ask_or_consume_effect_choice`'s `effect_choice_gate_closed` branch silently applies the
default. Its needle list was never taught the sixth channel, and the comment describing it still
said FIVE — **the same sentence PB-DX28's own `/review` caught one variant short and filed as
`OOS-DX28-6`, one channel short again.** The reviewer proved it by planting a `MayPayThenEffect`
inside a `WhenTappedForMana` trigger and watching the gate stay GREEN, then swapping it for an
`Effect::Scry` and watching it go RED. Two needles added (`MayPayThenEffect`; and
`LookAtTopThenPlace` **over-wide on purpose**, because the second site is a FIELD and no variant
name distinguishes a def that sets `place_cost` from one that leaves it `None` — stated rather
than accepted silently), both revert-proven RED as V15/V16.

**Three MEDIUMs, all in the batch's record rather than its code, and all instructive:**
1. **The execution notes' "measured" table published two fingerprints that exist nowhere at HEAD.**
   They moved a SECOND time when `Cost` was `Box`ed for `clippy::large_enum_variant` and the table
   was never re-taken. This is PB-DX28's MEDIUM verbatim, inside a batch whose headline is *three
   published figures that did not reproduce*. Durable form: **a transcribed figure needs a re-take
   every time its source is recomputed; "I took it from the gate" says WHEN, not whether.** The
   version numbers never moved twice, which is what AC 7244 actually claims.
2. **R2's failure message inverted its own consequence.** `can_pay_optional_cost`'s tail returns
   **`false`**, not `true`, so an undecidable cost is never asked about and the whole `then` arm
   **silently never runs** — a defect, not the harmless over-ask the message described. Proved by
   executing `MayPayThenEffect { cost: Cost::Tap }`. The same fact makes `format_optional_cost`'s
   residual arm provably dead, a stronger bound than the corpus gate it originally cited.
3. **Six "pay when able" claims left standing in production source** — including
   `try_pay_optional_cost`'s OWN doc, the `MayPayThenEffect` DSL variant doc a card author reads,
   `birthing_ritual`, `effect_choose_gate`, and PB-DX24's deviation pin. PB-DX27's *a blocker note
   is a claim* left un-applied to this batch's own subject matter, in a batch that invokes that
   lesson to repair `teneb_the_harvester`'s false comment.

**Three LOWs**: the policy ruling's first draft published a rule and then created three violations
of it (narrowed to what it actually adjudicates — it discovers those three markers as wrong, it
does not create them, and `OOS-DX45-3` now says marker and card must be fixed in ONE commit so the
re-deal is paid once); the HTTP pair drives `birthing_ritual`'s `Cost::Sacrifice` at the SECOND
site rather than `nether_traitor`'s `{B}` and the substitution was undisclosed (now disclosed at
the test, in the notes and in CLAUDE.md, with the untested combination bounded to
site 1 × HTTP transport); and a `cargo fmt`-mangled `debug_assert!` format string.

Full record, including the census, the wire table, the seven-plus-one obligations, a 16-row
revert matrix (**16 RED, 0 UNDISCRIMINATED**) and the two `/review` disclosures:
`memory/primitives/pb-DX45-execution-notes.md`. Policy ruling: `memory/decisions.md`.

## Worker Handoff (PB-DX15a, `scutemob-216`) — the two live CR sweeps

**Shipped**: v4 queue **rank 3**. **`OOS-DP9-8` and `OOS-DP9-11` both CLOSED**, each registry row
carrying corrections to its own claims. Riders, FINAL after the `/review` fix cycle (which
inverted both first-draft verdicts — see "The `/review` cycle" below): **`OOS-DX24-1` CLOSED**,
**`OOS-DX24-7` RE-OPENED** — and **both riders' prescribed fixes were wrong as written**, each
refuted by executing it rather than by argument. `OOS-DP9-16` **NOT taken**, parked as the brief directs.
Filed **`OOS-DX15a-1..7`**.

Tests **4,835 / 0 / 5** (+38 over the **4,797** pre-edit baseline, **54** targets; delta itemised
by NAME as **42 additions / 4 leaving / 0 removals**, the four disclosed individually — 2 are the
inversions and 2 are **doctests whose name IS their line number**, shifted +10 by the `ZoneEnd`
declaration). `clippy --workspace --all-targets -D warnings`, `cargo fmt --check` and
`tools/check-defs-fmt.sh` (1,803 defs) all clean.

**Next dispatch: PB-DX45** (v4 rank 4). Ranks 1-3 are all shipped.

### The headline: a pin that pinned nothing, and the reason both seeds survived

`OOS-DP9-8`'s row said its deviation was "pinned as the engine's actual behaviour by
`test_dp9_choice_inside_for_each_each_player`". **It was not, and could not be.** That test ran
on a two-seat fixture with `.active_player(p(1))`, and APNAP (CR 101.4: active player, then the
rest in turn order) starting from the **lowest** `PlayerId` over an ascending `turn_order` **is**
ascending `PlayerId` — rotating a list to start at its first element is the identity. The
assertion `vec![p(1), p(2)]` would have stayed green under either rule.

**That one fact explains three separate things**, which is why it is the headline rather than a
footnote:

1. why `OOS-DP9-8` survived five months and eleven batches behind a test claiming to hold it;
2. why the v4 memo's wire cell ("golden scripts and SR-9b per-step fingerprints move; budget the
   re-pin") was **wrong** — nothing moved, because *every* fixture in the tree makes the same
   choice, so the reorder is invisible to all of them;
3. why the batch has **no inherited red-before evidence anywhere** (full workspace with both
   engine halves in: **4,797 / 0 / 5**, zero failures) and every probe had to earn its own revert.

It is now stated **structurally**, not left as a discovered fact:
`test_dx15a_active_lowest_id_makes_apnap_and_ascending_indistinguishable` asserts the coincidence
over 2..=6 seats plus the contrasting non-vacuous case. Two known non-discriminating survivors are
named in `OOS-DX15a-3`: `fixture_3p` in the same file, and `pb_eng1_effect_discard_choice.rs`'s
3-player discard-order test, whose prose still says the engine "iterates players ascending".

### Both filed populations were floors, and both were mis-framed

**`OOS-DP9-11`: 5 → 17 deck-legal `Complete`.** The five named defs all reproduce, and they are
**one of four mechanisms**: family A (`RevealAndRoute`/`LookAtTopThenPlace` routing to a library,
the named 5), family B (**every `SearchLibrary`-to-library tutor** — 8), family C (Hideaway — 1),
family D (PartnerWith — 3). The two census axes **do not nest**: an oracle-text axis sees only
family A (a tutor's printed text never says "put the rest on the bottom") and a structural
`Effect`-payload axis cannot see C/D at all (a keyword confers through `AbilityDefinition::Keyword`).
PB-DX26's and PB-DX43's lesson a third time. Two more corrections: **`chaos_warp`, one of the row's
own five, reaches the `Library{Top}` branch**, not the bottom helper the row is filed against; and
**family D's blast radius is the whole library** — the PartnerWith arm moved every id to the bottom
in turn, so a 99-card library minted 99 ids and burned 99 `timestamp_counter` values per ETB,
unconditionally.

**`OOS-DP9-8`: the "Fleshbag / Grave Pact family (10 defs)" makes no per-player choice at all.**
`sacrifice_permanents_for_player` sorts the eligible set and takes the first `n`. What this batch
repairs there is the **order the sacrifices happen**, not agency. Only **2** deck-legal `Complete`
defs (`burglar_rat`, `geier_reach_sanitarium`) exercise the row's literal question-order claim. The
agency gap is filed as `OOS-DX15a-2` so the family is not treated as closed, and the probe covering
it carries `assert_ne!(decision.kind, DecisionKind::EffectChoice)` so its doc cannot go stale.

### The same-zone fix is not the sweep the seed asks for, deliberately

`Effect::MoveZone` and `Effect::PutOnLibrary` resolve their destination from a `ZoneTarget` **at
runtime**, so "is this call same-zone" is not a property of the call site at all — a per-caller
sweep closes today's members and cannot close the class. The guard lives inside both `GameState`
move helpers (`from == to` → `reposition_within_own_zone`), which makes a renumbering same-zone
move **unrepresentable**.

**One existing test was a pin ON the defect**: `test_400_7_same_zone_move_produces_new_id` asserted
`assert_ne!(old_id, new_id)` because "the zone-change event creates a new object regardless of the
source and destination zones being the same" — which **inverts CR 400.7**, whose antecedent is
"moves from one zone to another". That test is *why* the seed stayed open: a helper-level fix
reddened it, so every earlier reader concluded the helper was right. Inverted, and it now also
asserts the `timestamp_counter` half.

### Both riders' prescriptions were wrong, in different ways

- **`OOS-DX24-1` — DEFERRED, prescription refuted.** Its "one source-zone conjunct before the
  match" would break Teysa Karlov's doubling of a look-back dies trigger, because such a trigger is
  built as `PendingTrigger::blank(*new_grave_id, ..)` — **its source is a graveyard object too**.
  Zone alone cannot separate the legitimate case from the defect. Proven: `trigger_doubling.rs` had
  **nine** tests and **none** touched the `CreatureDeath` arm, so the missing probe was written
  first, confirmed green, and then the conjunct applied verbatim → `left: 1, right: 2` **with all
  nine still green**. A correct fix needs a set not in scope at the doubling call site, or a marker
  on a hashed serialized type (a second wire bump). Zero deck-legal pairings, so deferral costs
  nothing live; the probe ships as permanent wrong-way-round coverage.
- **`OOS-DX24-7` — TAKEN, prescription inverted.** "Rebuild the set per event prefix" (a) makes
  `sba.rs` wrong — in one CR 704.3 fixpoint pass the deaths *are* simultaneous, which is the
  Gatherer ruling the function already quotes — and (b) has the direction backwards: the set is a
  **suppression** set, so the prefix is what to **subtract**, not what to pass. Passing it
  reproduces the very defect the row describes. Shipped as `EventBatchTiming` + the complement.

### Standing facts for the next batch

- **PROTOCOL 38 / HASH 77 both gate-executed and UNMOVED**, exactly as predicted in writing before
  any code changed; the stop-condition never fired and no pin was edited.
- Coverage unmoved **1,136/1,803 = 63.0%**, **0 flips**, proven by regeneration with the
  self-dating churn reverted. **0 card-def edits.**
- **Moved-pin list by name: EMPTY** — reported as a paid-and-unclaimed budget, with the measured
  reason, rather than dropped.
- **Four gates fired on this batch's own work and all four were right**: SR-25's
  `bare_lookup_ratchet`, PB-DX7's `unordered_iteration_ratchet` (ceiling **lowered** 11 → 6 by
  converting to `BTreeSet` rather than raised), the batch's own scry non-vacuity floor, and its own
  `r5` roster row (which found that `move_object_to_zone` mints **three** ids, not one).
- **One probe passed vacuously before it passed honestly**: a bare `execute_effect` on
  `Effect::SearchLibrary` measures nothing, because PB-DP9 rolls the whole resolution back until
  the choice is answered. Worth carrying to anyone writing an `OOS-DP9-*` probe.
- **The play server structurally exposes ONE seat's questions** (`seat_view` filters on the human
  seat and `post_action` refuses a foreign seat), so the *sequence of asked seats* is not
  observable over HTTP. The HTTP probe asserts what is: which seat the server had already asked,
  and the resolution's event order straight off the wire payload.

### The `/review` cycle: 1 HIGH / 4 MEDIUM / 5 LOW, all ten taken

**The HIGH was a regression this batch introduced, and the argument against it is the batch's own
argument applied to a case it missed.** `§4.2` explains at length why a prefix set makes
*simultaneous* SBA deaths wrong — and the same batch then declared a **wrath's** deaths sequential.
`Effect::DestroyAll` snapshots the battlefield and destroys it in one loop (**21** corpus board
wipes), so a resolution's event slice is not uniformly sequential, and `nether_traitor` (`Complete`,
deck-legal) fired its `trigger_zone: Graveyard` ability off a creature that died at the same instant.
`resolution.rs` is reverted to `Simultaneous` and **`OOS-DX24-7` is re-opened**: its premise survives
(the caller really is coarse) but **`EventBatchTiming` is the wrong granularity** — the correct unit
is a simultaneous GROUP, and one resolution holds both kinds, so closing it needs the event stream to
carry group boundaries. `t5` pins the wrath case wrong-way-round. `Sequential` ships with **no
production caller**, stated in-source rather than left as an unexplained dead variant.

**The second finding is that a deferral reason did not survive checking.** `§4.4`'s first draft said
`OOS-DX24-1`'s discriminator "exists in exactly two places, and neither is available to this batch".
**A third was in data already passed to the function** — the triggering EVENT. The four values
reaching the doubler's `match` with a non-battlefield source split exactly two ways, and the split is
**total** because the battlefield-sourced `AnyCreatureDies` collector filters on `obj.zone ==
ZoneId::Battlefield`. The rider is **CLOSED**, with a **pair** of probes: both sources sit in a
graveyard, so either alone is satisfiable by a wrong implementation and together they are not.
**A deferral is a claim like any other**, and this one reached the registry before it was checked.

**A process failure worth carrying**: the fix cycle introduced **five** further failures (a
`bare_lookup_ratchet` breach, two `completeness_deviation_scan` breaches, a `fmt` diff and a clippy
lint) and **none was caught by the batch**, because after the fix cycle it ran the *targeted* tests
and not the full suite. **The gates must be run against the final tree; a fix cycle is a change like
any other.** Three of those five were gates catching the batch a second time and all three were
right — and the deviation-scan answer was **not** an allowlist entry: the offending paragraph did not
belong in a card def at all.

Full record: `memory/primitives/pb-DX15a-execution-notes.md`.

---

## Worker Handoff (PB-DX44, `scutemob-215`) — the casts you cannot make

**Shipped**: v4 queue **rank 2**. **`OOS-DX29-9`, `OOS-DX29-12`, `OOS-DX29-14` CLOSED**;
**`OOS-DX29-3` NARROWED** (pitch half closed, graveyard half deferred and measured). Filed
**`OOS-DX44-1..5`**. Tests **4,797 / 0 / 5** (+44 over the 4,753 pre-edit baseline, itemised by
NAME: **45 additions, 1 rename, 0 removals**, 53 targets); coverage unmoved
**1,136/1,803 = 63.0%**, **0 flips**; **PROTOCOL 37 → 38 / HASH 76 → 77**, both gate-computed and
both predicted in writing before any code changed.

**Next dispatch: PB-DX15a** (v4 rank 3 — the two live CR sweeps, `OOS-DP9-8` CR 608.2e APNAP +
`OOS-DP9-11` CR 400.7 same-zone renumber). Ranks 1-2 are both shipped.

### What shipped, in one line each

* **Spree** (`OOS-DX29-14`): `effective_cast_cost_with_additional` gains `modes_chosen` and charges
  `ModeSelection.mode_costs`; `insatiable_avarice` — the only deck-legal `Complete` Spree def and
  previously uncastable from *every* channel — casts.
* **Fuse targets** (`OOS-DX29-12`): CR 702.102d's both-halves requirement list, derived once in
  `card_def_target_requirements` and consumed by both `handle_cast_spell` and
  `queries::spell_target_requirements`; the offer suppression deleted with its mechanism.
* **Half selector** (`OOS-DX29-9`): `AltCostKind::SplitRightHalf` + `StackObject.cast_right_half`;
  the right half of all **three** DSL split defs is castable.
* **Pitch** (`OOS-DX29-3`): `params.rs` forwards `alt_cost`; `AdditionalCostPlan.pitch` carries the
  exile candidates; all four pitch defs cast at their printed cost with a non-default card.

### The five things worth carrying forward

**1. A brief that names the arithmetic has named half the fix.** The Spree row's fix shape —
"fold `mode_costs` into `effective_cast_cost_with_additional`" — is necessary and not sufficient.
`auto_tap_commands_for` must pass `&cast.modes_chosen` **verbatim off the `Command` it is about to
apply**, because that is the same value for the human path (`submit`) and the bot path (`advance`,
where `params.rs` has already substituted `spell_default_modes`). `&[]` there is the obvious first
draft and leaves the defect alive on both. Proven by revert: that one substitution reddens three
end-to-end probes with `InsufficientMana`. **This is PB-DX29's own lesson arriving one function
over**, and it will arrive again — the function auto-tap asks is where cost defects live.

**2. Deleting a suppression is not the same as making the offer honest.** `OOS-DX29-12`'s fix shape
says "concatenate the fuse targets, then delete the predicate", and the batch's first draft did
exactly that and **shipped the SR-38 defect the batch exists to delete**: `view.rs` passed
`fuse: false`, `ActionBar`'s stage order is `ValuePrompt → CostPicker → TargetPicker`, so a human
ticked Fuse and was then asked for one target against an engine demanding two. PB-DX29 gated the
offer *precisely* to avoid this and the batch fixing it recreated it. **The probe that missed it
compared `spell_target_requirements(.., true)` against `(.., false)` — both assertions true,
neither about the channel.** *A differential between two arguments of one function proves the
function, not the caller.* Write channel differentials over the DTO the client receives.

**3. Where a defect is noticed is not where it lives.** `OOS-DX44-4`'s first draft said "a **fused**
spell's target indices shift under CR 608.2b", because that is where it surfaced — while designing
the right-half index padding. The ordinary cast path has the identical
`filter(is_target_legal)`-then-positional-`ctx.targets.get(idx)` pattern
(`resolution.rs:452-457`, `effects/mod.rs:7937`). Measured candidate population: **7** deck-legal
`Complete` defs declaring ≥2 flat spell targets, plus the 2 fusable defs — printed by
`pb_dx44_uncastable_roster::r9`. **Check whether the shape you found is general before you scope
the row.**

**4. A source grep counts the token; `all_cards()` counts the declaration.** The census asserted
pitch = 5 because `grep -l "AltCostKind::Pitch"` returns five files. `force_of_despair` mentions it
in a **comment**. SR-36's exact content, committed inside the census written to obey SR-36, and
caught only because the figure was an executed assertion rather than prose. **Two further
self-corrections landed the same way**: `OOS-DX29-13`'s own prescribed fix (assert
`card_name_to_id(name) == card_id`) fails on **50** defs in four classes, so it ships as a pinned
floor and the row's prescription is corrected; and a probe doc claiming Misdirection was "the only
pitch member with no life component" was refuted by a revert that reddened **four** tests —
`force_of_will` is the only member that *pays* life.

**5. Latent vs unreachable is a different number, and scope decisions rest on it.** The graveyard
cast loop is deferred because `casting.rs:283` auto-detects escape from the zone alone, so a loop
shipped without the `EscapeExile` channel converts "never offered" into a **hard refusal**. The
seed's row argues that coupling and omits the population: **zero** deck-legal `Complete` Escape
defs exist. Both halves are now pinned wrong-way-round (`r8` and `t7`). This is the figure PB-DX29
learned to publish when its own "13 of 15 kinds invisible" proved materially misleading.

### Hazards for the next batch in this area

* `card_def_target_requirements` takes **three booleans**, only four of whose eight combinations are
  legal, and it does not validate the exclusion (`OOS-DX44-3`). Recorded as a decision, not an
  oversight — an enum is the right shape when someone next touches it.
* `pb_dx44_uncastable_roster::r6` pins **50** `card_id`/name mismatches as a floor. A new def with a
  mismatched id reddens it; that is deliberate. Two are genuine typos (`OOS-DX44-2`).
* CR 709.4's timing half is unbuilt by decision (`OOS-DX44-5`): no corpus split def has halves of
  differing instant-ness, pinned by `r7`. The **fused** path already widens instant-speed
  (`casting.rs:937-942`), so the two paths diverge the day a member differs.

---

## Worker Handoff (PB-DX43, `scutemob-213`) — a rule the engine had never derived, on cards that print no text for it

**Shipped**: v4 queue **rank 1**. **`OOS-DX27-1` and `OOS-DX27-10` both CLOSED**; filed
**`OOS-DX43-1..7`**. Tests **4,749 / 0 / 5** (+28 over the 4,721 pre-edit baseline, itemised by
NAME, **0 removals**, 50 targets); coverage unmoved **1,136/1,803 = 63.0%**, **0 flips**, proven by
regeneration; **PROTOCOL 37 / HASH 76 both gate-executed and UNMOVED**.

**Next dispatch: PB-DX44** (v4 rank 2 — the casts you cannot make: pitch, split-card halves, fuse
targets, Spree mode costs; `OOS-DX29-3` + `OOS-DX29-14` + `OOS-DX29-9` ≡ `OOS-DX29-12`).

### The defect

CR 305.6: *"An object with the land card type and a basic land type has the intrinsic ability
'{T}: Add [mana symbol],' even if the text box doesn't actually contain that text or the object has
no text box."* `Characteristics.mana_abilities` was written from four kinds of site — def abilities,
copy, wipe, explicit grant — and **none read `chars.subtypes`**. So three deck-legal `Complete`
format staples silently under-delivered their whole printed text in the shipped browser game.

### The layer placement is the batch, and it forced a fix the brief did not scope

CR 305.6's intrinsic is a **consequence of the type change** (CR 613.1d), so a layer-6
ability-removal must still be able to strip it (CR 613.1f) — that is `p9`, and it is what makes the
placement a decision rather than a preference. Post-walk placement would make the ability immune to
Humility, which is CR-wrong; placement inside an `apply_layer_modification` arm would read an
**intermediate** subtype set and re-open the CR 613.8 Blood Moon × Urborg dependency the arm at
`layers.rs`'s `depends_on` exists to settle.

**That reading makes the criterion's own instruction insufficient.** Criterion 6507 says delete the
moons' hand-authored mana grants. Doing only that leaves each moon's **layer-6
`RemoveAllAbilities`** wiping the **layer-4** derived ability — **Blood Moon stops working
entirely**, and `pb_dx27_blood_moon_type_scope::t6` catches it. So CR 305.7's ability-LOSS half
moved into the `SetLandTypes` primitive, conditioned on the payload containing a basic land type
(CR 305.7's own precondition — `p13` proves a `Gate` payload triggers neither the clearing nor any
derivation), and **each moon dropped two statics, not one**, leaving a single layer-4 static apiece.

**The relocation closes a second CR violation nobody had filed.** CR 305.7's last sentence: *"this
doesn't remove any abilities that were granted to the land by other effects."* A blanket **layer-6**
removal is timestamp-ordered against every other layer-6 effect, so it could strip an
earlier-timestamped grant — Cryptolith Rite, Chromatic Lantern, The World Tree, Bootleggers' Stash
and Wrenn and Realmbreaker all grant into `LandsYouControl`/`AllLands`. Moving it to layer 4 makes
every layer-6 grant survive regardless of timestamp (`p7`, red on the pre-PB-DX43 shape).

### The census reproduced exactly and was still a floor short by three

The v4 memo publishes its rule: scan card-def sources for a land-type-conferring
`LayerModification` whose payload names a basic land subtype. **Re-run at HEAD it reproduces its 5
exactly.** It also structurally cannot see a def that confers types through a `TokenSpec`. An
inverse axis over printed text found:

- **`awaken_the_woods`** — `Complete` by derive, its "Forest Dryad land" token declares
  `mana_abilities: vec![]`. A **fourth live-wrong def**, producing nothing, that nobody counted.
  Fixed for free.
- **`overlord_of_the_hauntwoods`** — `Complete`, its Everywhere token hand-authors all five basic
  subtypes **and** all five mana abilities: `OOS-DX27-10`'s double-grant shape on a **third** def.
  Proven to resolve to 5 and not 10.
- **`leyline_of_the_guildpact`** — `Inert`; filed `OOS-DX43-1`, since its land clause is now
  correct-for-free and only the unrelated all-colours clause still blocks `Complete`.

The class is **8 defs, not 5**. Both axes are now standing SR-36 roster rows (R1 payload, R2
inverse), so neither half can silently regrow. PB-DX26's lesson arriving again: **a roster derived
from one declaration construct measures that construct.**

### The basics decision was made by a gate, not by taste

Basics **keep** their printed `{T}: Add`; the derivation is idempotent instead
(`discharges_intrinsic_mana_ability`, an exhaustive `ManaAbility` destructure with no `..`, SR-5).
Three reasons in order of force: (1)
`effect_choose_gate::every_complete_land_registers_each_printed_tap_mana_color` compares oracle text
against the **`enrich_spec_from_def` registry lowering** — no `GameState`, no layers — so deleting
`swamp.rs`'s printed ability reddens it **correctly**; (2) `Command::TapForMana.ability_index` is a
**dense index** into `mana_abilities`, so deletion moves every basic land's ability out of index 0
(`OOS-DX26-3`) on the commonest object in the game; (3) `face.rs` and `resolution.rs` rebuild
**base** `mana_abilities` from the def at every face change. Index neutrality proven by roster row
R3 across all **46** `Complete` defs printing a basic land subtype: resolved equals base exactly,
same length, same order.

### Two fixture defects the batch found in itself — both filed, both worth reading

1. **`GameStateBuilder::build()` registers no static continuous effects** (`OOS-DX43-6`). Nothing
   does until a permanent enters through `Command::PlayLand` or spell resolution. A conferring
   permanent placed straight on the battlefield by the builder confers **nothing** — and the first
   draft of the channel probes therefore failed on all three staples **for a reason they did not
   describe**. The mirror image of PB-DX25b's fixture, which made a probe *pass* by removing the
   only condition under which the code was wrong. Fixed by entering the cards through the real
   command path, which is strictly stronger: the probes now prove *play Urborg, THEN tap your
   Plains for `{B}`*, end to end.
2. **An offer-layer assertion about a non-priority player is structurally vacuous** (`OOS-DX43-7`).
   `StubProvider::legal_actions` returns an empty list for such a player, so the assertion reads 0
   whatever the engine believes. It failed loudly here, which was luck: written as a `== 0`
   expectation it would have **passed vacuously forever**. Re-pointed at the mana solver, which
   filters on `obj.controller` rather than on priority.

### Reachability was measured before the design was chosen

Every production consumer of `mana_abilities` already reads layer-resolved characteristics —
`mana.rs`'s `TapForMana` fetch, `legal_actions.rs`'s offer loop, `mana_solver::gather_sources`,
`can_afford`/auto-tap, `params.rs`'s any-colour re-read, and the play-server transitively. That is
why `crates/view-model` and `crates/simulator/src` are **0 lines**. It is also why the derivation
had to live in `calculate_characteristics` and not in base characteristics: `face.rs:115` and
`resolution.rs:891` **overwrite base `mana_abilities` wholesale** from the def at every face change
and would have silently erased it.

### Hazards for the next batch

- **A seeded constant moved and the plan predicted it**: `UI3_SPLIT_COMBAT_SEED` 28 → 32,
  re-observed by an executed sweep over seeds 0..80 (hits: 32, 47, 48, 79). Any batch that changes
  what lands offer will move it again. Re-observe; do not edit to taste.
- **`OOS-DX43-4`** blocks `OOS-DX43-3`: there is no query anywhere for "what does ability N of this
  object produce", so the browser renders two identical `Tap Plains for mana` rows and a player
  choosing between them is choosing blind. `queries.rs` has **no mana consumer at all** and
  `crates/view-model` surfaces **no abilities of any kind**.
- **`OOS-DX43-5`**: the merge (CR 702.140e) layer-6 block does not propagate `mana_abilities` from
  non-topmost components, under a comment claiming there is no such field. There is. Latency
  deliberately **unmeasured** — do not read that row as claiming zero members.

Full record: `memory/primitives/pb-DX43-execution-notes.md` (both revert matrices, the census
method, the population figures printed by the tests themselves). Plan: `memory/primitives/pb-plan-DX43.md`.

## Worker Handoff (SEED RE-RANK v4, `scutemob-212`) — a census cutoff is a date on a document, and work does not respect it

**Shipped**: doc-only triage. `memory/primitives/seed-rerank-2026-08-14.md` is the authoritative
queue; v3's §4 is banner'd SUPERSEDED with §1-§3 left canonical. **Zero code**: `git diff --numstat`
over `crates/` and `tools/` is **empty**, executed not asserted. Tests **4,721 / 0 / 5**, coverage
**1,136/1,803 = 63.0%**, PROTOCOL **37** / HASH **76** — all untouched **by construction**, which is
a stronger claim than "the gates were re-run green" and the honest one for a doc-only task.

**Next dispatch: PB-DX43.**

**The census, and why the brief's estimate was 6× short.** The brief said "~35+ seeds". The
population is **208**, derived rather than counted: `S = ALL(488) − V3(79) − LEGACY(196) = 213`,
minus five IDs that match the regex but are not seeds (a plan-only closed-on-arrival, a conditional
whose trigger never fired, an explicitly rejected number, a deliberately skipped number, and a
renumbering of a v3 seed). The command is published in the memo's §6 so the next reader re-derives
instead of trusting. **The cause is structural and it has now happened twice**: v3 recorded the
failure about v2 ("v2's census closed 2026-07-31; every PB-DX batch shipped 2026-08-01") and then
reproduced it — v3's census closed **2026-08-02**, the same day the recursion adjudication and the
whole triage-2 successor run shipped. **The fix is not a better cutoff. It is to define the
population as a set difference against the previous census's own table**, which is what makes this
number reproducible.

**61 of 208 seeds — 29% — have no registry row**, and `dispatch hygiene 5` names the registry as
ground truth. Add the 7 standing-row seeds §3.2 found (`OOS-CARDS2-6`, `OOS-OS6-1`, `OOS-OS7-1`,
`OOS-RS-5`, `OOS-OS4-1`, `OOS-RS4-3`, `OOS-OS4-3`) and the blind spot is **68**. The unrowed set is
one era of work — `scutemob-186..194`, SIM-4/5/6, ENG-1/2, UI-4/5/6 and the adjudication — filed
into `memory/workstream-state.md` handoff prose. **The cause is a convention nobody wrote down as a
rule**: `OOS-G1-1`'s own note says a seed closed in its own batch gets no row, *"the gate is the
durable artefact"*. That is defensible for the nine such seeds and does not cover the ~50 that are
**OPEN**. Two seeds (`OOS-CARDS2-3`, `OOS-CARDS2-4`) are recorded CLOSED in CLAUDE.md and appear in
the registry **neither open nor closed** — and PB-DX32's own `/review` caught exactly that
(`pb-review-DX32.md:336`) and it was never carried through.

**Rank 1 is a seed filed "latent" that is live-wrong on three deck-legal `Complete` format
staples.** CR 305.6 (verbatim, MCP): *"An object with the land card type and a basic land type has
the **intrinsic ability** '{T}: Add [mana symbol]', even if the text box doesn't actually contain
that text."* The engine has no such derivation — `Characteristics.mana_abilities` is written from
the def's own abilities, wholesale copy, wipe and explicit grant, and **none reads
`chars.subtypes`**; the `AddSubtypes` arm (`layers.rs:1661-1665`) is three lines.
`urborg_tomb_of_yawgmoth`, `yavimaya_cradle_of_growth` and `dryad_of_the_ilysian_grove` are all
`Complete` by derive and all confer a basic land type with no mana grant. `swamp.rs:11-27`
hand-authors `{T}: Add {B}`, which CR 305.6 says it should not need to. **`OOS-DX27-10` is a strict
sub-case that closes for free** — the double `{T}: Add {R}` under two Moons exists only because
Blood Moon and Magus hand-author the grant no derivation supplies, and `AddManaAbility` is
append-only.

**PB-DX42b was re-decided, not carried, and the falsification does not survive.** `OOS-DX27-9` says
the rank premise is false because the layer-querying population went 1 → 2. Re-measured by
executing the shipped gate's own whole-corpus serde walk: the **total** population is indeed 2, and
`the_world_tree.rs:73` is **`Completeness::partial`**, while `build_roster` walks `all_cards()` with
**no completeness filter**. The adjudication ranked PB-DX42b on **7 deck-legal `Complete` pairs**
under a convention it states two lines above its own table. **The deck-legal population moved
1 → 1.** So the rank stands; the seed's durable half — the supply census was measured for an
**Artifact** filter and does not carry to The World Tree's **Land** filter — lands only when PB-DX9
promotes that def, and that coupling is now a written sequencing constraint.

**Two independent verification passes reached opposite verdicts on the same gate, and reconciling
them is worth more than either.** Asked whether `OOS-ADJ-2` is discharged by the shipped PB-DX42a
rider, one said **yes** (it pins the population by name, states both legal exits in its failure
text, and **fired on its first real event**), the other said **no** (blind to seven of eleven
layer-querying `Condition` variants). Both are right about what they measured: axis 1 filters on
the literal string `"YouControlNOrMoreWithFilter"`, axis 2 needs a `TargetFilter` that eight of the
eleven do not carry, and `t7` pins exactly **one** of the eight absent. **The gate covers the
population as it exists and is blind to 7 of the 11 ways it can grow** — *a gate written for one
variant measures that variant*, arriving at the gate written to close the seed that predicted it.
Recorded **partially discharged** in the adjudication's own §6, with an ~8-line widening carried as
a rider at v4 rank 21 rather than as a reason to move PB-DX42b.

**`OOS-DX24-9` ≡ `OOS-DX27-5`** — the same `Effect::MayPayThenEffect` defect filed twice by two
batches five days apart, neither row citing the other, and **two independent passes of this task
both re-measured it at 11 deck-legal `Complete` defs**. That agreement is what proves they are one
thing, and it is v4 rank 4.

**Four silently-closed seeds, none recorded anywhere**: `OOS-SIM4-2` (CLOSED by PB-DX20, not
"narrowed" — and closed for *both* clients), `OOS-DX20-7` (the roster gate it asks for shipped
inside PB-DX26), `OOS-DX26-7`'s class half (closed by PB-DX8, which that row itself predicted would
close it), and `OOS-DX7-3` (closed in effect; the exclusion list it complains is missing exists and
has held green across three HASH-moving batches).

**Five of twenty-one standing wire cells were wrong** — rank 28 predicts PROTOCOL+HASH and measures
**none** (`CardFace` is off-wire and unhashed); rank 35 predicts PROTOCOL and measures **none**,
voiding its stated dependency on PB-DX34's bump; rank 32's "none" is unsafe (the reachable type is
the runtime `TriggerEvent`, not the DSL `TriggerCondition`); ranks 17 and 21 omit HASH bumps. Every
v4 wire cell now carries a **confidence**, and the memo says what HIGH/MEDIUM/LOW mean.

**Two registry rows are WRONG rather than stale, and one would make a batch ship the defect it
describes.** `OOS-UI2-5` says the TUI routes casts through `params.rs` and gets a silent
`eligible[0]` default. It has never routed a cast — a TUI human gets an outright refusal
(`casting.rs:3315`). **v3 recorded this correction in its own §1c and the registry was never
updated**, so **routing the `CastSpell` site is what would CREATE the silent-default defect**, on
13 deck-legal `Complete` defs. `OOS-DX23-3`'s "the TUI never routes through `params.rs`" is false
since SIM-6. Both corrected in the rows themselves, along with **14 rotted line cites** — every
premise survived, only the addresses rotted, which is `OOS-DX2-2`'s cite-by-symbol discipline
earning its keep for the third triage running.

**The user-directed Blood Moon / Urza's Saga flag is DISCHARGED** as `OOS-RR4-1` (engine),
`OOS-RR4-2` (card) and `OOS-RR4-3` (doc rot), all grep-confirmed absent from the registry first.
**The flag was right that it is a live latent engine defect and wrong in four particulars**, each
corrected in the rows rather than carried: corner case #36 is the audit's **only** remaining GAP
(**35 COVERED / 1 GAP**, measured — CLAUDE.md's "4 GAP" was stale by three closures); the "missing"
gains-an-ability primitive **exists with four corpus users** and the def's own compiled note says so
while its header TODO denies it; the Saga site list is **five** behavioural sites, not two (a fix to
the named two ships a Saga that still takes its ETB lore counter and still fires chapter I); and
"route through layer-resolved abilities" **cannot be written as stated**, because
`AbilityDefinition::SagaChapter` is never lowered into `Characteristics` — the shipped precedent is
IG-1's continuous-effect scan at `replacement.rs:2131-2147`, which hits this exact wall and says so
in-source. Two things the flag did not have: a **second blanking channel** (CR 708.2a face-down,
unchecked by the Saga sites though `queue_carddef_etb_triggers` checks it), and the measurement that
the **famous pair is not deck-legal** while two others are — so the engine half is live today
without the card half, and the card half is what makes the famous case testable.

**Durable lesson for the next re-rank, and it is about where seeds live rather than about any seed**:
**a registry is only ground truth for the kind of work that files into it.** Three passes are
mandatory — the registry, the handoff prose, and the per-batch execution notes — and every range
must be expanded against **its own filing document**, because with 29% of the population unrowed the
registry has no authority to arbitrate a range. That is how `OOS-SIM6-6` was found outside
CLAUDE.md's "`OOS-SIM6-1..5`", and how the W6 row's "`OOS-DX29-1..14`" was found to be 17.

**`/review`: 0 HIGH / 3 MEDIUM / 5 LOW, all 8 taken.** The reviewer had a shell and used it — it
re-ran the census command and reproduced **488 / 79 / 196 / 213 / 208** exactly, checked the §1b
ledger in **both** dimensions, byte-diffed the ordering rule against v2 and v3 and confirmed it
**verbatim**, executed `pb_dx42a_continuous_condition_roster` (10/10, printing every §2.2 figure),
and confirmed the doc-only diff empty. All six criteria PASS. **Every published aggregate
reproduced and four item-level traces were short**, which is the durable half: §6 promised a
registry correction to `OOS-SIM6-3`, **which has no registry row** — this task's own 29%-unrowed
finding biting inside its own fix list; §1b claimed all 45 queue candidates appear in §4 and three
did not; §5 said 63 parked seeds and enumerated 47; and §6 omitted an edit the branch had made. All
fixed, plus a self-found one the reviewer did not raise — §0's "~45 repairs" had no derivation, and
it is **53**, a sum of band-1 row populations with overlaps not deduplicated. Two unescaped-pipe
registry rows were escaped and **ten further pre-existing ones named by line rather than swept into
a re-rank's diff**. *An arithmetic that checks out is not an enumeration that checks out, and only
the enumeration survives being handed to a dispatcher.*

**Full record**: `memory/primitives/seed-rerank-2026-08-14.md` (§1 census, §2 chain-verification,
§3 standing rows + the PB-DX42b re-decision, §4 the queue, §5 parked, §6 source-doc edits + the
fix-cycle table).

---

## Worker Handoff (PB-DX29, `scutemob-211`) — a choice you cannot express is a choice you do not have

> **Full record: `memory/primitives/pb-DX29-execution-notes.md`**, with the refusal-channel A/B
> raw output beside it (`pb-dx29-refusal-before.txt` / `-after.txt`).

**Shipped**: v3 queue **rank 13**. `OOS-M11-10(loyalty)` + `OOS-UI2-4` **both CLOSED**; the
**OOS-M11-10 ID collision RESOLVED** (the closed equip seed renumbered `OOS-M11-10E`), which that
note itself had deferred to "whichever task next touches `params.rs`". Filed **OOS-DX29-1..17**.

**Read this first, because it is the thing the brief could not have told you.** Both halves of
this batch were framed as "the `Command` fields already exist, so this is a routing change". Both
framings were short:

* **The loyalty half needed two engine functions.** The seed says CR 602.2b targets are "already
  reachable through `queries.rs::ability_target_requirements`' sibling path". That is true of the
  *machinery* and **false of the index space**: `ability_target_requirements` indexes
  `Characteristics::activated_abilities`, while a loyalty `ability_index` is minted against the
  **registry** def's filtered `AbilityDefinition::LoyaltyAbility` list and consumed the same way
  by `handle_activate_loyalty_ability`. Index 0 means different abilities to the two.
* **The cost half needed `effective_cast_cost_with_additional` extended**, and no document named
  it. That function read **Squad and nothing else**, and it is what `LocalGame::auto_tap_commands_for`
  asks how much mana to tap. Shipping the seven pickers without it would have tapped the base
  cost, accepted the human's announcement, and let the engine refuse the cast with
  `InsufficientMana` — **the batch would have created the exact SR-38 defect it was dispatched to
  remove.**

**Enforcement-site lists were short in both halves.** Loyalty: five sites, not two — the params
arm, `view.rs::action_target_requirements`, `view.rs::target_query_source` (which renders a picker
with **zero candidates** on its own), `view.rs::action_needs_x` (for `LoyaltyCost::MinusX`), and
`targeting.rs`, the **bot** path, outside `tools/play-server` entirely. Omitting the last would
have re-created SIM-5's zero-target-cast defect on a new action one batch later.

**Populations, re-derived** (SR-36, from `all_cards()`):

| claim | as filed | measured |
|---|---|---|
| `Complete` planeswalkers with a targeted loyalty ability | "4 of 6" | **4 of SEVEN** |
| `AdditionalCost` variants | "sixteen", incl. Kicker | **15**, and **Kicker is not one** — UI-2's README and `api.rs` doc both listed a variant that does not exist |
| cost kinds a human loses | "13 of 15" | right, and **materially misleading**: 4 have **no deck-legal member at all**, 3 more are unreachable by construction |

**A fifth `Complete` planeswalker was live-wrong and no cite had ever named it.**
`chandra_flamecaller` declares `Completeness::Complete` explicitly and carries `LoyaltyCost::MinusX`
with `EffectAmount::XValue`; `params.rs` hard-coded `x_value: None` and the engine reads
`unwrap_or(0)`, so its printed "−X: deals X damage to each creature" was **−0 for 0 damage** in
every client in the tree.

**A new live defect, and it is `r3b`'s Squad shape inverted.** `nocturnal_hunger` is `Complete`
and deck-legal, carries `AbilityDefinition::Gift { Food }` and **no `KeywordAbility::Gift`**, and
`casting.rs` gates on the marker before it looks the cost up — printed gift, unpayable, nothing
red. UI-2 wrote `r3b` *because the corpus had failed it on Squad*, and the same defect had already
recurred one enum variant over. **A gate written for one variant measures that variant** — the
batch's thesis, and it then arrived **three more times inside the batch's own work**:

1. UI-2's R4 promised to "fail loudly the day" a hybrid Squad cost was authored. PB-DX29's R3 is
   that assertion widened past Squad and it went red on its **first run**, on
   `brokkos_apex_of_forever`'s `{2}{G}{G}{U/B}` mutate cost — a counter-example the corpus had
   carried the whole time. Fixed the formatter (CR 107.4e/107.4f/107.3), not the gate.
2. R5 justifies `ActionBar`'s stage order with "no def declares an additional cost together with
   `{X}` or modes" while walking Sacrifice and Squad only — and Escalate and Entwine are
   additional costs on modal spells **by definition**. Live on **five** defs. New R6 prints them
   and asserts the half that matters (**0** defs pair a cost with an `{X}`).
3. The batch's own Fuse cost arm called the seven-component helper under a comment saying
   `casting.rs` mirrors all three extra fields "for Fuse … mirrored deliberately". It did not.
   Proven by execution: predicted mana value 3, engine charged 4, cast refused. **One pip from
   being the clean-offer-then-server-rejection defect the batch exists to delete, inside the
   function added to prevent it.**

**A picker was GATED rather than shipped.** `casting.rs` never concatenates the fuse right half's
targets, so a fused cast of either deck-legal fuse def is a guaranteed `InvalidTarget` — pre-existing,
and unreachable until this batch's picker made it reachable. The offer is suppressed
(`OOS-DX29-12`); the chain is built and proven and turns on the day the engine learns CR 702.102d.

**Mutate is the mandatory-kind proof, and its shape is a scope precedent.** Measured, the mandatory
kinds are Sacrifice (already UI-2's), four that are unreachable by construction, and Mutate. So
Mutate was the only one both reachable and buildable. `LegalAction::CastWithMutate` gains `on_top`
and the provider emits one action per `(target, on_top)` pair — the PayEcho/ChooseDredge idiom, no
new params field and no wire change. `params.rs` hard-coded `true`, so **no client could ever
mutate under**, and CR 702.140e makes the topmost component supply the merged permanent's name,
cost, colours, types and P/T. `on_top: true` is emitted first so an index-choosing bot reproduces
its pre-batch command; no seeded fixture moved. The CR 702.140c **timing** (the choice belongs at
resolution) is filed as `OOS-DX29-2` and deliberately not moved.

**Three machine gates caught this batch's own work and every one was right**: SR-5's keyword
registry (7 keywords), its ability-definition sibling (7 variants + `A::LoyaltyAbility` gaining
`rules/queries.rs`), and `pb_dx27_stale_blocker_notes` — which fired on the batch's own
`dawns_truce` note, because rewording it moved the phrasing from OUTSIDE the gap-needle vocabulary
to inside it.

**Two defects in part A were found by the batch's own test author and reported rather than worked
around**: the new queries used `expect_object` (the impossible-absence lookup, a `debug_assert!`)
while their rustdoc promised "never panics" — *what is impossible for an engine-internal caller is
ordinary input for a UI one*; and joining the params allowlist widened the declared silent-ignore
residual from nine arms to ten while that doc still said nine.

**Numbers (at close, after the fix cycle)**: tests **4,721 / 0 / 5**, +87 over the pre-edit
baseline itemised by name with **0 removals**. Coverage **1,136/1,803 = 63.0%**, **0 flips as
predicted**, proven by regeneration (three card defs were edited, so the empty-diff shortcut was
unavailable). PROTOCOL **37** / HASH **76**, both gate-executed and **unmoved**. Engine lines are
**NOT zero** and the brief predicted zero — **+218/−12**, of which 138 are the new read-only query
surface, 76 are registry *declarations* two machine gates refused to let the batch omit, and 4 are
one comment; zero behaviour-changing engine lines anywhere. `crates/view-model`: **0**. The
prediction went outward twice — zero, then 177, then 218 — and the record says so rather than
quietly settling on the last number. Refusal-channel A/B **105 → 105 with an
empty diff**, reported as proof of bot-path neutrality (no recorded seed moved) rather than as
proof of nothing happening — bots structurally cannot produce an additional-cost refusal.

**The `/review` found 2 HIGH / 6 MEDIUM / 11 LOW and all 19 were taken.** The reviewer had a
shell, reproduced every acceptance figure independently (its own test-NAME set came back
byte-identical) and verified the two mirrors this batch flagged hardest — the `casting.rs` cost
arithmetic arm by arm, and the loyalty query against the handler — **finding no second
divergence**. What it did find were three more instances of the batch's own class, in places the
batch had not looked:

* **H1** — **Splice** was offered with no affordability bound and 422'd after a clean offer, on
  two deck-legal `Complete` cards. `SpliceCostOption`'s doc gives a real reason not to publish a
  bound in the OFFER (a subset-sum); it is not a reason to skip the check at the BOUNDARY, where
  the chosen list is known. Fixed by checking the **whole announced vector**, which also closes
  the joint-rider gap the reviewer only suspected.
* **H2** — the `OOS-M11-10` renumbering **orphaned 30 in-source cites** (17 of them card-def
  comments CARDS-1 authored, four of them live test-failure strings) while the resolution note
  asserted no cite needed rewriting. All 30 rewritten; and the premise the renumbering was chosen
  on — "equip has the fewer external cites" — is **inverted** at HEAD, so it was the *more*
  expensive direction. Kept, and the note now says so.
* **M1** — the validator's catch-all was a **default-ALLOW**. The reviewer POSTed an `Assist` and
  the engine accepted it, draining another seat's pool 5 → 3 without that seat being asked. The
  batch's own doc had argued not-surfacing was the mitigation; it closes the picker, not the wire.
* **M2** — the loyalty `{X}` channel **this batch opened** was unbounded above the engine (X = 9
  on a 4-loyalty `chandra_flamecaller` reached the engine and came back 422) while the batch was
  building `max_count` bounds for counts and `affordable` bounds for markers.
* **M6** — the execution notes' per-area line table **did not reproduce**: measured once and
  republished after a 1,452-line commit. PB-DX8's "publish the figure, do not transcribe it", in
  the file recording it.

**Two of the fixes were themselves caught by gates before shipping**, which is the durable half:
M1's first draft made every legal marker answer a 400 (the marker arms are *guards*, so the accept
case fell through the new catch-all), and L9's first draft opened a new raw `GameState` read in
`view.rs` that the Invariant-7 pin caught on the spot.

**For the next dispatch**: `OOS-DX29-1` (Assist spends an opponent's mana without asking) and
`OOS-DX29-2` (Mutate's `on_top` asked at the wrong time, and `copy.rs` answers it the other way)
are both live on deck-legal `Complete` cards. `OOS-DX29-3` is four `Complete` cards unplayable at
their printed pitch cost, and the graveyard cast loop it names unlocks three cost kinds at once.
`OOS-DX29-4` and `-10` are a **matched pair** — teaching `casting.rs` to charge hybrid rider pips
makes the other live in the same commit.

---

## Worker Handoff (PB-DX28, `scutemob-210`) — coordinator-written pointer at collect

> The worker's close-out landed CLAUDE.md's snapshot delta but no section here; this pointer
> was written by the coordinator at collect (2026-08-14) so the handoff chain stays unbroken.
> **The full record is `memory/primitives/pb-DX28-execution-notes.md`** (both parts), with
> `pb-plan-DX28.md` / `pb-plan-DX28-part2.md` and `pb-DX28-RESUME.md` beside it.

**Shipped**: v3 queue **rank 12**, merge `2bdc3533`. `OOS-DX4-6` + `OOS-DX4-1` **both CLOSED**;
filed `OOS-DX28-1..8`. Part 1: `TargetFilter` owner axis (CR 108.3 vs 109.4) +
`EffectTarget::DamagedPlayer`. Part 2: the choose-on-resolution-without-targeting channel
(CR 115.10) — `ChoiceZone`, `EffectTarget::ChosenObject`,
`EffectChoiceQuestion/Answer::ChooseObject`, legality by the dedicated
`filter_matches_object_untargeted`, deliberately NOT `casting::validate_targets_inner`.
Tests 4,605 → **4,634** (+29 by name, 0 removals); coverage **63.0%**, 0 flips as estimated;
PROTOCOL **36 → 37** / HASH **75 → 76**, gate-computed, one bump each. The batch ran in two
parts around a system reboot with a disclosed non-compiling WIP checkpoint (`92ea0ec2`),
resumed via `pb-DX28-RESUME.md` with no number re-guessed. /review 3 MEDIUM / 4 LOW, all 7
taken — the reviewer defeated two of the batch's own gates by execution and caught two
fingerprints cited in the evidence record that never existed in the repository. Seed-worthy
finds past the plan: the auto-target picker is TWO functions, not one, and `pb_dx42a`'s
`TARGET_FILTER_FIELDS` fingerprint went blind corpus-wide on a routine field addition.
**Next dispatch is not mechanical** — see the W6 row: rank 13 is contested between the memo
table (PB-DX29) and the adjudication's un-rowed PB-DX42b, whose rank premise `OOS-DX27-9`
already recorded as false.

## Worker Handoff (PB-DX27, `scutemob-209`) — a blocker note is a claim, and nothing re-checked one

**Shipped**: v3 queue **rank 11**. `OOS-CARDS2-8`, `OOS-CARDS2-10`, `OOS-CARDS2-11`,
`OOS-RR3-2` and the rider `OOS-ADJ-7` **all FILED and CLOSED** — **none of the five had a
registry row** before this batch wrote one (grep-confirmed absent first; the third batch
running to find its own seeds unrowed). Filed `OOS-DX27-1..10`. Tests **4,561 → 4,605
(+44)**, itemised by test NAME with **zero removals**. Coverage **1,133 → 1,136 / 1,803
(62.8% → 63.0%)**. **PROTOCOL 35 → 36 / HASH 74 → 75**, both gate-computed. Full
measurements, the disposition table and every revert matrix:
`memory/primitives/pb-DX27-execution-notes.md`.

### ⚠️ COLLECT-TIME INSTRUCTION — do NOT take this branch's `workstream-state.md` wholesale

`main` advanced past the merge base by **`afd4a72f`** ("flag Blood Moon + Urza's Saga
(corner case #36) for the next re-rank", user-directed 2026-08-13), which **added** a
25-line coordinator section this branch does not have. This branch **never touched the
file** — `git diff <merge-base> HEAD -- memory/workstream-state.md` is empty — so the
"25 deletions" a `main..HEAD` diff shows are `main`'s addition, not a removal here.

The project's habitual collect rule is *"take the worker's richer version"* of this file.
**Applying that here would delete the flag**, and that flag is the only record of its
finding: it says so itself ("No OOS seed filed — this flag is the record"). Keep `main`'s
section and merge this handoff above it. Flagged by this batch's `/review`.

For the record, the two do not conflict in substance: that flag concerns both Saga engine
sites reading `def.effective_abilities` rather than layer-resolved abilities, and it states
explicitly that PB-DX27's `OOS-ADJ-7` rider "is adjacent but does NOT touch this". Checked
and true — the rider changes which *modification* Blood Moon registers and leaves
`RemoveAllAbilities` exactly as it was.

### What the batch was, and the three things worth carrying forward

**1. The population the seed was ranked on does not reproduce.** The memo's **67**
machine-checkable blocker notes yields **49** by its own literal method at HEAD, 46
ground-truth-restricted, 109 by an inverse method; no needle-set variant reaches 67. The
dispatch brief called it "a FLOOR and a snapshot" — it is a snapshot and **not** a floor,
because every reproduction is *smaller*. A count measured against a dated corpus is a
snapshot in both directions, and "floor" is a monotonicity claim nobody checked.

**2. Existence is necessary and never sufficient.** Two adjudicated-REFUTED repairs were
**declined**: `kaito_shizuki`'s −7 (`Effect::CreateEmblem` exists, but
`collect_emblem_triggers_for_event` has six call sites and none is a combat-damage site, so
authoring it ships a 7-loyalty ability that does nothing — `OOS-DX27-3`) and
`blackblade_reforged`'s land-count static (`resolve_cda_amount` reads the controller off the
**equipped creature**, CR 108.5/611.2c-wrong; two sibling defs declined the same question —
`OOS-DX27-4`). The REFUTED-**PARTIAL** verdict was load-bearing; collapsing it to REFUTED
would have shipped the bug.

**3. One completeness flip re-deals every seeded fixture in the repository — twice, here.**
The implement phase re-observed **nine** seeded pins after its +4 move (4 simulator, 5
play-server), each by executed sweep. The `/review` then demoted `green_suns_zenith`, the
count moved again, and all of them re-dealt a second time. **Budget for two reconciliation
passes, not one**, on any marker-flipping batch.

### The `/review`: 1 HIGH / 5 MEDIUM / 6 LOW, all 12 taken

**The HIGH is the batch committing its own subject matter, and it happened twice on the
same two defs.** `chord_of_calling` and `green_suns_zenith` were promoted to deck-legal
`Complete` with their printed **"then shuffle" unauthored** — `Effect::SearchLibrary` has
no post-search shuffle, only a `shuffle_before_placing` branch, and
`eldritch_evolution.rs:12-14`, **the very file both defs cite as precedent**, states that
in-source. Then checking the *other* clause found the second instance:
`self_shuffle_on_resolution` places deterministically on top of the library rather than
shuffling (`resolution.rs:2023-2025` says so), `nexus_of_fate` is `partial` for exactly
that, and `green_suns_zenith` claiming `Complete` was **the same outlier shape this batch
had just demoted `qarsi_sadist` for**. Demoted back.

The reviewer's diagnosis of *why* it shipped is the durable half: **the three headline defs
had zero behavioural coverage** — they appeared only in source-scanning gates. Closed by 9
probes; revert row **R2 reproduces the exact HIGH** (remove `Effect::Shuffle`) and reddens
**only** the `LibraryShuffled` probe, which is what proves the new coverage isolates it.

Also taken, each worth a line: a **second recall bound the gate never stated** — **74** defs
name a live identifier inside a gap assertion phrased outside the needle set, invisible to
both ratchets, two of which literally record their own note as stale (now ratcheted,
revert-proven 74 → 75 RED); a **calibration table publishing figures that did not reproduce
against the shipped code**, deleted rather than re-transcribed, with every population now
PRINTED by `t_derivation_report` — the same correction PB-DX8 made and this very file's doc
claimed to have learned; and **"`ALL_LAND_TYPES` had zero users" asserted as *the proof* in
three places when `correlated_card_types()` reads it** — right conclusion, wrong proof, in a
batch whose thesis is that a note is a claim.

### The standing gate

`crates/engine/tests/core/pb_dx27_stale_blocker_notes.rs`. Existence is decided by a
**usage** test (does `Type::Member` appear in non-comment DSL source), not a declaration
parse — a parser fails *open*, reporting everything absent, i.e. every note correct, i.e.
green. The oracle is itself pinned against **15 hand-adjudicated identifiers** and agreed
15/15. R1 is a **count** ratchet at 107, deliberately not a 107-row verdict list (that is a
rubber stamp with 107 signatures). R2 is the closure proof: the 10 repaired defs must stay
OUT of the live-naming set. R3 gives the blind spot a number — **357** defs assert a gap
naming no identifier at all. R4 is the review's second bound at **74**.

### Open for the next batch

- **`OOS-DX27-9`** — PB-DX42b's rank rests on a population of exactly 1 and it is now 2;
  its §2.3 supply census was computed for an **Artifact**-reading filter and does not carry
  over to The World Tree's **Land**-reading one.
- **`OOS-DX27-5`** — `MayPayThenEffect` is pay-when-able and the corpus is inconsistent:
  `disciple_of_freyalise` is `Complete` on the identical shape that `ruthless_technomancer`
  and `vampire_gourmand` now carry at `partial`. **Policy call, then a sweep.**
- **`OOS-DX27-2`** — the Exploit trigger condition *and* the interactive sacrifice choice
  (the latter is a `Command` ⇒ PROTOCOL bump); blocks all three corpus Exploit defs.
- **`OOS-DX27-6`** — the **357** opaque gap notes are the cheapest standing rider available
  to any card-def batch: each note rewritten to name its primitive moves one def out of the
  blind spot and into the machine-checkable population.
- `OOS-DX27-1`, `-3`, `-4`, `-7`, `-8`, `-10` as filed.

---

## Coordinator flags for the next re-rank

- **2026-08-13 (user-directed): Blood Moon + Urza's Saga — corner case #36 — rank at the next
  re-rank.** The famous interaction is untested and is the corner-case audit's row 36 (**GAP**,
  one of 4 remaining). Coordinator-verified at HEAD, both halves read rather than trusted:
  (a) **`urzas_saga.rs` is `partial`** — chapters I/II are placeholder `GainLife(0)` behind a
  TODO naming a missing "this Saga gains an activated ability" continuous-effect primitive;
  nothing in any test tree references the def. (b) **Both Saga engine sites read the printed
  def, never layer-calculated abilities**: `check_saga_sbas`
  (`crates/engine/src/rules/sba.rs:827`) computes the final chapter from
  `def.effective_abilities(..)`, and the lore-counter/chapter-trigger path
  (`crates/engine/src/rules/turn_actions.rs:399-402`) gates on the same call. `blood_moon.rs`
  DOES carry `RemoveAllAbilities` at Layer 6, so the layer system blanks the Saga — but neither
  Saga site can see that. CR 714.4's "a Saga permanent **with one or more chapter abilities**"
  exempts a blanked Saga (verified against the CR via MCP): correct behaviour is that it
  survives as a blank Mountain; the engine today would still count chapters, still fire chapter
  triggers, and sacrifice it at 3 lore counters. **The (b) half is a live latent engine defect
  independent of Urza's Saga** — any `RemoveAllAbilities` over a Saga hits it — and is the
  PB-DX19/DX24 "a guard reads the wrong subject" family. Two work pieces for the ranker:
  the gains-activated-ability primitive (unblocks the def) and routing both Saga sites through
  layer-resolved abilities (same neighbourhood as PB-DX42b's layer-bounded queries; weigh
  ordering against it). PB-DX27's OOS-ADJ-7 rider (Blood Moon vs artifact lands) is adjacent
  but does NOT touch this. No OOS seed filed — this flag is the record until the re-rank rows
  it (registry grep per dispatch hygiene 5 at that point).
  - **✅ DISCHARGED 2026-08-14 by the seed re-rank v4 (`scutemob-212`).** Rowed as
    **`OOS-RR4-1`** (the engine half), **`OOS-RR4-2`** (the card half) and **`OOS-RR4-3`**
    (the doc-rot cluster) in `docs/audits/decision-point-audit.md` §8.1 — grep-confirmed
    absent first, per dispatch hygiene 5 — and both work pieces are ranked in
    `memory/primitives/seed-rerank-2026-08-14.md` §4. **The flag was right that this is a
    live latent engine defect and wrong in four particulars**, each corrected in the rows
    rather than carried: it is the audit's **only** remaining GAP, not one of four (35
    COVERED / 1 GAP, measured); the "missing" gains-an-ability primitive **exists and has
    four corpus users** (the def's own compiled note says so while its header TODO denies
    it — PB-DX27's shape exactly), and the two real blockers are a `TokenSpec` CDA and a
    printed-mana-cost predicate, neither of them Saga work; the Saga site list is **five
    behavioural sites, not two** (the ETB lore counter at `replacement.rs:2012-2015` and
    the chapter-trigger enumeration at `:2078` are both missing from the flag, so a fix to
    the named two ships a Saga that still takes its ETB counter and still fires chapter I);
    and "route through layer-resolved abilities" **cannot be written as stated**, because
    `AbilityDefinition::SagaChapter` is never lowered into `Characteristics` — the shipped
    precedent is IG-1's continuous-effect scan at `replacement.rs:2131-2147`, which hits
    this exact wall and says so in-source. Two additions the flag did not have: a **second
    blanking channel** (CR 708.2a face-down, which the Saga sites do not check though
    `queue_carddef_etb_triggers` in the same subsystem does), and the measurement that the
    **famous pair is not deck-legal** (`urzas_saga` is `partial`) while **two other pairs
    are** — so the engine half is live today without the card half, and the card half is
    what makes the famous case testable. See v4 §1g.

## Worker Handoff (PB-DX8, `scutemob-208`) — a gate can only see the vocabulary it was given

**Shipped**: v3 queue **rank 10**. `OOS-DP10-9` **RECORDED (not closed)**, `OOS-CARDS2-7` **FILED
and CLOSED**, rider **PB-DX42a SHIPPED** per adjudication §5.1. **Test-only**: 0 lines across
`crates/engine/src`, `crates/card-types/src`, `crates/card-defs/src`, `crates/view-model/src`,
`crates/simulator/src` and `tools/`, and **0 card-def edits of any kind**. Tests **4,527 →
4,561 (+34)**, delta itemised by test NAME with zero removals. Coverage unmoved at
**1,133/1,803 = 62.8%**, proven by regeneration with the self-dating churn reverted.
PROTOCOL **35** / HASH **74** gate-EXECUTED (`hash_schema` 36/36, `protocol_schema` 17/17) and
unmoved. `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
`tools/check-defs-fmt.sh` clean (1,803 defs).

**Three files**: new `crates/engine/tests/core/pb_dx8_oracle_decision_cross_check.rs` (17 tests),
new `crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs` (10 tests), and
`crates/engine/tests/core/completeness_deviation_scan.rs` rewritten (5 → 12 tests).
Measurements and the full 30-row revert matrix: `memory/primitives/pb-DX8-execution-notes.md`.

### The durable lesson, and it cost three failed derivations to find

**No statistical derivation over this corpus recovers a decision vocabulary**, and the three
failures fail differently enough that all three are worth carrying forward (`OOS-DX8-5`):

1. **Iterated bootstrapping drifts.** Seeded with `{may, choose, up to}` and grown by sentence-level
   lift, V had absorbed `battlefield`, `library`, `graveyard`, `search`, `shuffle` by iteration 3 —
   the vocabulary of the *effects that surround* "choose", not of choice. 3 → 57 in four passes.
2. **Single-pass lift returns object nouns** (`land card`, `basic land`, `mana cost`).
3. **A vocabulary learned from the DSL's own ground truth is self-blinding on the target.** Learn
   from the oracle text of defs that DO carry a decision-bearing construct and you get
   `{choose, chosen, one, up, of the}` — **`may` does not appear at all**, because "you may" is
   precisely the class the DSL cannot encode.

(3) is the one to remember. §2.6's *"derive the category from the thing being checked, not from the
checker"* needs a companion clause: **derive it from the thing being checked, not from the thing
that already handles it correctly** — the second is structurally blind to exactly the population
you are hunting. What shipped is a **morphological closure** (3-character stem of each marker over
the corpus's own oracle text), which cannot drift because it does not iterate, plus one reasoned
lexical exclusion (`mayhem`, 1 occurrence).

### What the OOS-DP10-9 gate actually does

Two axes per **channel**, and the channel split is the load-bearing part. Oracle axis: the
morphological closure, computed at run time from `all_cards()` and pinned. DSL axis: identifier
stemming over the serialized corpus surface (**617 object keys + 711 bare variant strings**;
`may` → 2 elements, `cho` → 20, `up to` → 1). **A `choose`-shaped construct does not discharge a
printed "you may"** — collapsing the channels is what let Smuggler's Copter pass `decision_gate.rs`,
which saw it only through the incidental `Effect::DiscardCards` inside the same unconditional
`Sequence`. `t_smugglers_copter_is_in_the_measured_population` and
`t_channels_are_not_interchangeable` pin both halves.

Measured: `may` **287** oracle-positive / **72** effectively-`Complete` with no may-shaped
construct; `choose` 116 / 2; `up_to` 70 / 10. Union **80**, frozen in a `BASELINE` documented as
mechanical-not-adjudicated (the PB-DP10 correction, applied at write time) and ratcheted exactly.
Six `RECORDED_STRUCTURAL_EVIDENCE` rows, each with a CR cite and a written reason, suppress **18**
defs whose optionality is encoded structurally (`EntersTappedUnlessPayLife` and the rest of the CR
118.12 unless-family, `optional: true`, non-null `modes`) — measured **96 → 85 → 72**, i.e. 24
defs, and **printed by `t_reconciliation_report`** rather than transcribed (the implement phase's
`90 → 80 → 72 / 18` was the pre-fix front-face-only measurement, corrected by the `/review`
cycle).

**Fail-closed proven END-TO-END on a real def**: `lightning_bolt.rs`'s `oracle_text` was given
`"You may draw a card."`; the offender gate named the card, channel and CR, and the union ratchet
went 80 → 81. Restored, both green.

### The batch committed its own subject matter three times, and execution caught every one

1. **The oracle axis read `def.oracle_text` alone.** `CardFace` carries its **own** `oracle_text`
   and a `CardDefinition` holds two (`back_face`, `adventure_face`), so the first draft was blind
   to every transformed face and every Adventure half. Found by the **inverse-method census**
   (dispatch hygiene 6), not by any test. Fixed structurally — harvest every `oracle_text` **key**
   at arbitrary depth. 19 defs expose more than one. **The widening added ZERO offenders**, which
   is a fact about today's corpus and not about the walk (`OOS-DX8-4`).
2. **The deviation scan matched the whole SOURCE file while its needles were derived from PROSE.**
   `drawcards` is also `Effect::DrawCards`: **203 files / 127 unmarked / 37% precision** against the
   derivation's own **20 / 1 / 95%**. It could never have survived the 95% floor under a fair
   measurement, and shipping it would have blown the 45-def freeze past 150 silently. **A needle set
   and the surface it is matched against are two halves of one instrument** (`OOS-DX8-6`).
3. **A ratchet that could never redden.** The first-draft population ratchet filtered its own
   denominator down to the roster it was checking. Caught by revert row V4: the real gate reddened
   with 106+ offenders while the ratchet stayed GREEN (`OOS-DX8-7`). **This is the third instance in
   three batches** — PB-DX7's V14 found a false negative inside its own dead-entry checker, and this
   batch's `t_optional_false_is_not_evidence` re-implemented the predicate it claimed to guard so
   its revert row passed green (PB-DP10 review finding #3, verbatim). **Common cause: a checker
   whose reference set is derived from the thing it checks can never disagree with it.**

### Corrections carried back into the rows themselves

* The brief's dedupe note was imprecise: `cannot be expressed` is **not** a Tier A member (it is
  redundant only because its one unmarked hit is also caught by `dsl gap`), while `dsl gap` itself
  **is** a Tier A duplicate the brief did not mention. The module doc states the true split.
* The adjudication's structural axis for "layer-querying" is **not** a general proxy:
  `Condition::ControlLandWithSubtypes` reaches `characteristics_for_condition` without carrying a
  `TargetFilter`. Absent from today's conditioned population, so `t7` pins the *coincidence* rather
  than assuming it — and **PB-DX42b's rank argument inherits the caveat**.
* PB-DX42a got a **third** non-vacuity floor the adjudication did not ask for: `>= 176` nodes with
  no `Static` ancestor. A `Static`-only walk clears both stated floors while missing the entire
  nesting class the structural walk exists for.

### `/review` fix cycle — 10 findings, all taken, two gates DEFEATED by the reviewer

The reviewer had a shell and used it: it reproduced every published corpus count independently,
re-ran the end-to-end fail-closed proof, and **broke two of the three gates by execution**. Both
defeats are closed and re-executed; the full table is in the execution notes §6.

* **The deviation scan's narrowing to `//` comments silently dropped `/* */` blocks.** The
  byte-identical sentence reddened as a line comment and left every test green as a block comment
  — `OOS-DX32-6`'s class, latent only because the corpus happens to carry zero deviation-language
  block comments today. Fixed (`block_comment_bodies`), pinned, and the reviewer's exact defeat
  re-executed: gate **and** ratchet now RED.
* **Evidence is scoped to the DEF, not the CLAUSE.** A printed "may" appended to a `Complete` def
  whose `optional: true` belongs to an unrelated clause is invisible. **24** `Complete` defs are
  exempted on the `may` channel by a single piece of evidence. Stated as a second recall bound
  rather than left to be discovered; not fixed, because scoping evidence to the ability subtree
  needs a clause-to-ability alignment this gate does not have.
* **A doc comment cited a test that did not exist** — the precise failure the same file cites
  `decision_gate.rs`'s precedent for. Written.
* **A test was a compile-time tautology**: it compared two `const` array lengths and then asserted
  `A || true`. Rewritten to parse the struct declaration and compare field names; a one-field
  desync now reddens **5** tests where it reddened none.
* **Two published numbers were the pre-fix figures**, republished in the same documents that
  celebrate the fix that invalidated them (`may` 285 → **287**, suppression `18` → **24**). Both
  corrected — and a `t_reconciliation_report` added so the code prints them and the next reader
  re-derives rather than trusts. That is the durable half of this finding.
* **A count ceiling's comment claimed a per-entry promise it cannot keep** (a one-for-one swap is
  invisible to it). Corrected to what it enforces, with the residual stated — **and
  `decision_gate.rs`'s identical `FROZEN_2026_07_27` carries the same overclaim**, recorded here
  rather than fixed in a file this batch does not own.
* Six Tier-A needles are ordinary English clearing the concentration floor on base rate (marked
  defs carry ~2.5× the prose of unmarked ones). **Kept and stated as a measured precision bound**,
  not tuned away: dropping a needle for being inconvenient is the defect this batch removes.

Post-cycle: tests **4,561** (+34, zero removed), clippy/fmt/defs-fmt clean, PROTOCOL 35 / HASH 74
re-executed and unmoved, coverage unmoved, test-only scope re-verified.

### For the next batch

* **`OOS-DX8-1`** is the highest-value follow-up: the 80 `BASELINE` entries hold **two** shapes
  needing opposite work — *choice dropped* (`Aura Shards`, `Eternal Witness`: marker or engine PB)
  and *choice elsewhere* (`Force of Will`: an alternative cost, wanting a
  `RECORDED_STRUCTURAL_EVIDENCE` row, which would remove it from the baseline and lower the
  ratchet). Doing the second unmeasured would be needle tuning. `OOS-DX8-8` is the same shape on
  the deviation scan's 45, with six named candidates.
* **`OOS-DX8-2`** states the recall bound in numbers: `unless` (54), `any number of` (24),
  `rather than` (17), `instead of` (5) are attested optionality idioms **outside** the closure.
  Widening is not free — each needs its own DSL-side evidence set.
* **`OOS-DX8-3`**: the DSL has an explicit `optional` flag and **5 defs in 1,803** use it, against
  72 `Complete` defs printing a "may" nothing expresses. That ratio is audit §5's DP-12 class
  measured rather than described.
* **`OOS-DX26-7`'s class is now partly measured**: it asked for a `TargetRequirement`-vs-oracle-text
  scanner for the "up to one target" vs "target" mismatch and called the population unmeasured. The
  `up_to` channel is that scanner's first cut — **10** effectively-`Complete` defs print "up to"
  with no `UpToN` anywhere. Not the same question (this measures absence of the construct, not
  mismatch of optionality), but it is a floor where there was none.

---

## Worker Handoff (PB-DX7, `scutemob-207`) — a gate that reported success while checking nothing

> v3 queue rank 9. **`OOS-DP7-11` + `OOS-DP9-13` CLOSED**; riders **`OOS-DP10-1` CLOSED** and
> **`OOS-DP9-10`'s residual CLOSED** (gated, not deferred). Tests **4,508 → 4,527** (+19),
> 46 targets, residual empty; delta itemised by test NAME, not arithmetic. Coverage **unmoved at
> 1,133/1,803 = 62.8%**, proven by *regeneration* with the self-dating churn reverted.
> **PROTOCOL 35 / HASH 74 both gate-EXECUTED and unmoved.** Test-only held exactly: **0
> non-comment lines** in `crates/engine/src/state/hash.rs`, **0 card-def edits**, 3 files touched.
>
> **Both holes were reproduced at HEAD before anything changed**, which is the closure standard and
> also the only reason the second finding below was believed. Deleting a live field from the
> path-qualified `MergedComponent` impl left **all 21 gates green — including
> `stream_fingerprint_is_pinned`**, which no seed row claims: the canonical fixture carries no
> merged component, so the stream digest did not cover that struct either. The enum demo was green
> *and* `clippy -D warnings` clean, because `..` silences `unused_variables`.
>
> **The durable lesson, and it is about briefs rather than code: a scope that is true about a
> subset reads exactly like a scope that is complete.** The brief scoped the enum half to "the 10
> path-qualified enums". That sentence is true — those 10 are outside the gate. It is also
> irrelevant: path qualification has nothing to do with the enum half, and **all 79** hashed enums
> were outside the struct gate. Obeying the brief would have closed `OOS-DP9-13` on paper with 69
> enums still uncovered — the seed marked closed, the hole still open. Final scope: 79 enums,
> **1,252 variants, 1,097 variant fields**. This is the third consecutive DX-family batch whose
> filed scope was short; the standing instruction to treat a site list as a FLOOR is what caught it.
>
> **Three further holes of the same family surfaced only by widening, and each was found by
> refusing a plausible "this is fine":**
> - **`OOS-DX7-2`** — the coverage predicate cannot tell `self.f.hash_into()` from
>   `self.f.is_some().hash_into()`, so four sites passed as fully covered while their *values* never
>   reached the hasher. The implement phase classified this "confirmed not a violation" and was
>   right about the mechanism and wrong about the disposition: passing on a matcher technicality is
>   this gate's own failure mode, one level down. New `PARTIALLY_HASHED` /
>   `PARTIALLY_HASHED_VARIANT_FIELDS`. The enum half was nearly omitted as a scope call — shipping
>   without it would have left two halves of one file disagreeing about coverage on the same field
>   name.
> - **`OOS-DX7-1`** — `Effect` reuses **9 discriminants across 18 variants** while its comments
>   called them unique. The first disposition was "subsequent field bytes differ", which is an
>   assertion of the same shape that let `OOS-SIM2-6` survive 4.5 months. Settled by **experiment**:
>   all 18 constructed, hashed, digests pairwise distinct — shipped as a test that states its own
>   evidence-not-proof limit — plus a ratchet pinning the 9 known-bad pairs so a 10th cannot appear.
>   Renumbering would move the byte stream, so the debt is filed, not fixed.
> - **`OOS-DX7-3`** — **`GameState` was carved out of the field gate entirely**, and 3 of its 45
>   fields reached neither `public_state_hash` nor any stated exclusion list. Now gated. Note
>   `private_state_hash` is deliberately NOT gated the same way (player-scoped by design); the
>   coordinator's first reading of it was wrong and the correction is recorded rather than quietly
>   dropped.
>
> **34 revert rows (24 numbered + 10 in the fix cycle), all executed red then restored, none
> UNDISCRIMINATED — and three found real bugs
> before shipping.** V14 exposed a **false negative in the new dead-entry checker itself**: it
> searched for the literal tuple-index string `"0"` instead of re-deriving the actual pattern
> binding, so the guard passed GREEN where it should have failed RED. That is this batch's exact
> subject matter recurring inside its own implementation, and it was caught only because every row
> had to *demonstrate* red rather than be argued. V18 forced an artificial digest collision rather
> than assuming the collision detector fired.
>
> **The `/review` cycle found 2 HIGH / 5 MEDIUM / 9 LOW, all 16 taken, and both HIGHs were this
> batch committing its own subject matter.** The reviewer had no shell, so nothing in its report
> was executed; the coordinator executed both before any fix, and both were real.
> - **H1 — the gate was proven with the single input it handled.** The unordered-container ratchet
>   counted the literal `HashSet<` spelling: the type-ANNOTATION form, and the *minority* idiom
>   here. `casting.rs` has **0** annotations, **9** constructions, ceiling **0**. The exact
>   `OOS-DP9-10` defect, written with `HashMap::new()` instead of an annotation, left all three
>   tests green — in `layers.rs`, the same file V5 used. **V5 had reddened only because I wrote its
>   probe with an explicit type annotation.** Widened to whole-token: **27/6 files → 85/9**, all 85
>   traced and classified individually.
> - **H2 — the closure did not hold when first claimed.** `FieldCoverage::Full` meant "the token
>   appears", not "the value is hashed", so `let _ = may_fail_to_find;` on the seed's own card was
>   33/33 green and clippy-clean with the field gone from the stream. That is verbatim
>   `OOS-DP9-13`'s sentence. Fail-open `else` → fail-closed `Unverified`.
>
> **The M5 fix then repeated H2 inside the fix cycle** — its first draft used bare presence and its
> revert proof PASSED when it should have failed. Third instance in one batch, caught only because
> every row must *demonstrate* red. **M7 is the one worth carrying**: both
> `PARTIALLY_HASHED_VARIANT_FIELDS` reasons cited `hash.rs:4105-4111`, the impl header and the
> `Spell` arm, and the `ActivatedAbility` arm had **no comment at all** — a reason asserting
> in-source documentation that did not exist, which the coordinator approved without opening the
> lines after telling the implementer that a false reason is worse than no allowlist.
>
> Two reviewer recommendations **declined with reasons, recorded not buried**: the `OOS-DP9-10`
> rider stays (the reviewer argued it should have been deferred, and H1 gave that teeth — but
> deferring leaves a registry row carrying a wrong count and a gate that green-lights the defect it
> names), and the 18-sample digest experiment stays (it is the only executed evidence behind
> `OOS-DX7-1`; the alternative is the assertion that was rejected).
>
> **Corrections carried back into the rows themselves**, since each was a claim someone trusted:
> the seed's cite `hash_schema.rs:1540-1541` names `COVERAGE_MUST_INCLUDE`, not the skip (which is
> the `else { continue }` in `every_hashed_struct_field_is_hashed_or_allowlisted`); the implement
> phase's "26 revert rows" is **24**; `OOS-DP10-1`'s "cross-checked BY VALUE" was a **floor** check
> with one floor *below* the live count, so a one-def divergence passed in silence; and a premise
> the **coordinator** asserted — that `HASH_SCHEMA_HISTORY` rows inherited the discriminant error —
> was checked and found **false** (zero matches), so a clean-result note was recorded instead of a
> correction being invented to fit the instruction. A brief is a claim like any other.
>
> Measurements, census and the full revert matrix: `memory/primitives/pb-DX7-execution-notes.md`.
> Spec the implement phase worked from: `memory/primitives/pb-DX7-gate-spec.md`.
> **Next dispatch: PB-DX8** (v3 rank 10, oracle-text-vs-DSL cross-check).

## Worker Handoff (PB-DX26, `scutemob-206`) — a printed ability that did not exist

> v3 queue rank 8. **`OOS-CARDS1-3` + `OOS-CARDS1-1` + `OOS-DX3b-1` all CLOSED.**
> Tests **4,491 → 4,508** (+17), 46 targets, residual empty. Coverage **net unmoved at
> 1,133/1,803 = 62.8%** — one flip up (`sword_of_body_and_mind`) and one honest flip
> down (`the_reaver_cleaver`, from the `/review`). **PROTOCOL 35 / HASH 74
> gate-executed and unmoved.** **0 engine-source lines** —
> `git diff main..HEAD --numstat -- crates/engine/src crates/card-types/src
> crates/view-model/src crates/simulator/src` is empty. **`tools/` is NOT zero**
> (review Finding L11 — the first draft of this line implied it was):
> `tools/play-server/src/main.rs` moves ~+50 -24, all inside its `#[cfg(test)]`
> module (the `UI3_SPLIT_COMBAT_SEED` constant and its doc). Measurements and the
> executed revert matrix: `memory/primitives/pb-DX26-fail-before-2026-08-11.md`;
> per-def plan: `memory/primitives/pb-DX26-equip-spec.md`.

### What was wrong

`state/keyword_registry.rs:98` classifies `K::Equip` as a
`KeywordHandling::Marker` whose carrier is "`Effect::AttachEquipment` … activated
through `AbilityDefinition::Activated`". **A marker synthesises nothing.** So 21
defs carrying only `AbilityDefinition::Keyword(KeywordAbility::Equip)` had no
`ActivatedAbility` in their layer-resolved characteristics at all: nothing for the
provider to offer, no ability index for a client to name, no
`Command::ActivateAbility` that could reach one. Where `OOS-M11-10(equip)` was
*"the picker never asks for a target"*, this is ***"there is no action to pick"*** —
the same playtest symptom, one link sooner, on a strictly larger population.
**10 of the 21 were deck-legal `Complete`**, nine of them by the `#[default]`
derive: a human could put Umezawa's Jitte or Sword of Feast and Famine in a real
deck today and simply never be offered an equip.

The two riders are the same defect at one link later and in a different filter
field respectively: `darksteel_garrison`'s Fortify ability existed and declared
`targets: vec![]` (cost paid, silent fizzle — CR 702.67a), and
`guardian_project`'s ETB trigger was missing `is_nontoken: true`.

### What shipped

All 21 defs gain an `AbilityDefinition::Activated { cost, effect:
Effect::AttachEquipment, timing_restriction: SorcerySpeed, targets:
[TargetCreatureWithFilter { controller: You }] }` — CR 702.6a/702.6b/702.6d — with
the printed cost MCP-verified per def and **the `Keyword(Equip)` marker retained
beside it** (the card really does have the keyword; `state/hash.rs:1561` hashes it
and `view-model::format_keyword` renders it, so dropping it would change what the
card *is* in order to fix what it *does*). `darksteel_garrison` gets
`TargetPermanentWithFilter(has_card_type Land + controller You)`.
`guardian_project` gets `is_nontoken: true` and stays `known_wrong`.

### Five things the brief and the seed rows did not say

1. **The "~4-6 flips" estimate was wrong in an instructive direction.** The ten
   deck-legal defs were *already* `Complete`, so repairing them flips nothing —
   the whole yield is **one** flip, and it came from the other half of the roster:
   `sword_of_body_and_mind`, whose `partial` note named the missing Equip {2} as
   its **only** remaining blocker. A batch aimed at ten `Complete` defs produced
   its coverage movement from an eleventh def nobody was counting.

2. **A card-def-only batch is not automatically an index-neutral batch.**
   `Command::ActivateAbility { ability_index }` indexes activated abilities in
   declaration order. `umezawas_jitte` is the only member of the 21 with a
   pre-existing `Activated` ability (the PB-EF7 modal counter-removal), and the
   first pass inserted equip beside the keyword marker at the head of the vec,
   silently renumbering the modal ability **0 → 1** — the index
   `pb_os10_singleton_cleanup.rs` and any golden script already name. Caught only
   by this batch's own `t3`, which now pins the order and says why. Filed as
   **`OOS-DX26-3`**: nothing gates the class corpus-wide.

3. **The inverse census found two defs no keyword-derived roster can see.**
   R4/R5 start from the printed **type line** rather than the marker, and turned up
   `quietus_spike` and `sting_the_glinting_dagger` — Equipment that print
   "Equip {N}" and carry **neither** the marker **nor** the ability, so the seed's
   grep (21 files containing `KeywordAbility::Equip`) and R1's `all_cards()` walk
   are equally blind to them. Both are `Inert`, so the deck-legal blast radius is
   **0**, and both are whole cards deliberately withheld under W5/W6 on one
   genuinely-absent DSL variant each — seeded as **`OOS-DX26-1`** and pinned as
   R4's residual **with the excusal itself asserted**, rather than repaired.
   Dispatch hygiene 6 held: the brief's site list was a floor.

4. **`Lizard Blades` was a false positive of the census, and fixing that is a
   structural point, not a suppression.** Reconfigure is the one attach keyword
   `keyword_registry.rs` classifies as `Handled`: `replay_harness.rs:4049-4085`
   expands `AbilityDefinition::Reconfigure` into a real `ActivatedAbility` carrying
   `Effect::AttachEquipment`. So its attach IS reachable while its own effect tree
   contains no `AttachEquipment` node. The census counts the variant rather than
   listing the card as an excused residual — for that keyword the answer lives at
   the synth site, not in the def.

5. **The one completeness flip re-dealt three seeded fixtures**, exactly as
   `pb_dx32_fuzz_output`'s own `MOVED_MSG` warns. `completeness_deviation_scan`'s
   floor 670 → 669, `CORPUS_COMPLETE` 1133 → 1134, and `UI3_SPLIT_COMBAT_SEED`
   21 → 28. The UI3 seed was **re-observed**, not guessed: a throwaway sweep over
   0..40 (then deleted) found 9/26/28/29/30 split the attack, and running the test
   against each showed only **28 and 29** also reach a declared blocker — a second
   filter the constant's own doc never mentioned. Seed 9 (the lowest split) was
   tried first and failed on the CR 509.1a half alone.

### Gate integrity

`cards1_equip_target_roster` R1 re-pinned **17 → 38** in two labelled groups, and
its `Effect::AttachEquipment` match — plus `cards1_equip_target_repair:541`'s and
its `find_attach_equipment_target` — made **recursive** over all ten
`Box<Effect>`/`Vec<Effect>` nesting sites. That was the hazard
`seed-rerank-2026-08-02.md` §2.7 names by line number, and it is **proven live**:
revert row V6b nests Bone Saw's attach in an `Effect::Sequence` and watches the old
flat `matches!` drop it out of the exact pin silently, while V6a shows the
recursion keeping it. `pb_dx26_attach_keyword_roster::r6` pins the nesting-site
count (**8 Box + 2 Vec** — the hand-written first draft said 7/3 and the gate
caught its author) so the walk cannot go shallow unnoticed; its residual (a source
count cannot see `Option<Box<Effect>>` or a newtype, neither of which exists today)
is stated in the gate and filed as **`OOS-DX26-5`** rather than left implied.

`t7b` was **strengthened rather than merely updated**: a name-set pin
(`{"Darksteel Garrison"}`) would have stayed green through the fix *and* through a
regression of it alike, so it now asserts the requirement **shape** of every
`Activated` + `AttachFortification` member with a non-vacuity count.

### The revert matrix, and the row that does not discriminate

15 rows executed (`pb-DX26-fail-before-2026-08-11.md` §2). **13 red as required**;
V6a is deliberately green (that is the recursion working). **V4b is honestly
UNDISCRIMINATED and is labelled so in the test's own doc comment**: weakening
`bone_saw`'s requirement to a bare `TargetCreature` leaves `t4` green, because
`OOS-DX20-7`'s legacy `Effect::AttachEquipment` guard in `rules/abilities.rs`
separately validates a volunteered target's controller — the rejection has two
providers and `t4` cannot tell which answered. The "you control" clause is proven
instead by **T1** (offer side, row V4c) and by `cards1_equip_target_roster::r2`
(shape side, row V4), both red under the same reversion. Same shape as PB-DX25c's
V3/V7 rows: an assertion shadowed by a redundant downstream check is worth keeping
and not worth overclaiming.

### Blocker notes were re-verified, not copied forward

Every remaining `partial` blocker was re-checked against the **current** enums
(`OOS-DX3-1`'s standing sweep — a blocker note is a dated claim). Confirmed still
absent: `TriggerCondition::WhenEquippedCreatureAttacks`,
`ActivationCost::requires_untap_self`, `Condition::EquippedCreatureIsTapped` /
`EffectFilter::TappedCreaturesYouControl` (the last two appear **only** inside
`sword_of_the_paruns`'s own TODO comment). Confirmed **stale and corrected in
place**: `sword_of_body_and_mind`'s header TODO claiming multi-colour protection was
unexpressible (the def already carried two `AddKeyword(ProtectionFrom)` statics),
`glimmer_lens`'s "Equip {1}{W} cost is also not modeled" and `guardian_project`'s
"`TargetFilter` lacks non_token field" — which was never true. **Correction from the
`/review` (Finding 4): this list originally also claimed `sting`'s header had been
corrected, and it had not been.** The discipline was applied to the `completeness`
field and not to the `//` comments beside it, on six defs; the fix cycle rewrote all
six, each dated. A closure claim is a dated claim too. **`OOS-DX26-6`** records the ten still-`partial` equip defs
as a measured worklist, three of which their own notes say are authorable today.

### Seeds

**CLOSED**: `OOS-CARDS1-3`, `OOS-CARDS1-1`, `OOS-DX3b-1` (each row also carries
corrections to its own original claims). **FILED**: `OOS-DX26-1` (the inverse-census
find), `-2` (CR 702.6c variant equip costs have no DSL representation; two defs
print one), `-3` (the `ability_index` renumbering hazard), `-4` (nothing gates
"has the ability but dropped the marker" — 21 of 38 carry both today), `-5` (R6's
stated residual), `-6` (the ten-def worklist). The registry was grepped for
`OOS-DX26` **before** filing (dispatch hygiene 5): none existed.

### The `/review` cycle: 1 HIGH / 6 MEDIUM / 11 LOW + a CR gap, all 18 taken

Findings in `memory/primitives/pb-review-DX26.md`. The reviewer re-derived the census
on a **fourth axis** the batch had not used — the printed oracle text stored in the
defs — and it reconciled exactly (42 Equipment defs; minus Fortify and Reconfigure = 40
printing an Equip line; minus the two `Inert` ones = **38**, R1's re-pin). *There is no
third def.* Four findings are worth carrying past this batch:

1. **The HIGH was a live defect on a deck-legal `Complete` def the batch had touched.**
   `sword_of_light_and_shadow` declared a MANDATORY `TargetCardInYourGraveyard` for a
   printed *"you may return **up to one target** creature card"*. With an empty
   graveyard the trigger has no legal target and is removed under **CR 603.3d** —
   taking **"you gain 3 life"** with it, since both ride one `Effect::Sequence`.
   `UpToN` exists and its own roster-mate `sword_of_sinew_and_steel` uses it twice for
   exactly this shape, so it was never a DSL gap. Filed as `OOS-DX26-7`: the instance is
   closed, but **nothing enumerates the corpus for "up to" vs. mandatory**, the way
   SR-37's R7 now does for costs.

2. **The gate that claimed exhaustiveness was already wrong when it shipped.**
   `Effect::RollDice { results: Vec<(u32, u32, Effect)> }` is an **eleventh** nesting
   site, invisible to a `Box<Effect>`/`Vec<Effect>` substring count — and the residual
   R6 *did* state (`Option<Box<Effect>>`) was **backwards**: that spelling contains the
   substring and would have fired the gate. So the gate documented a hole that does not
   exist while missing the one that does. All three walks now carry a `RollDice` arm and
   R6 counts a third form, pinned `(8, 2, 1)`.

3. **`the_reaver_cleaver` was `Complete` by the `#[default]` derive with nobody having
   ever ruled on it**, while the trigger it grants under-fires (printed "…to a player
   **or planeswalker**"; neither `TriggerCondition` variant is exact). Demoted. The
   generalisable half is `OOS-DX26-8`: **965 of 1,803 defs declare no marker at all**,
   so the corpus's default answer to "has a human ruled on this card?" is *yes* when the
   truth is *nobody looked*. That is a `card-types` change, not another review pass.

4. **The batch asserted a correction it had not made.** `OOS-DX26-1` and this handoff
   both said Sting's stale header "was corrected in place"; the reviewer found the text
   verbatim. The `OOS-DX3-1` discipline had been applied to the `completeness` field and
   not to the `//` TODOs beside it, on **six** defs — and then the closure prose claimed
   otherwise. All six rewritten and dated; both false claims corrected. **A closure claim
   is a dated claim too.**

Two process notes, recorded rather than tidied away. **A stable `CORPUS_COMPLETE` is not
a stable deal**: the two completeness moves cancelled in the COUNT and not in the SET, so
the fuzz pool holds a different card, `UI3_SPLIT_COMBAT_SEED` needed re-observing a
*second* time (21 → 28 → 26), and the constant that normally shouts about a pool change
stayed green through it. And **the revert harness restores with `git checkout --`, which
reverts to HEAD**: two files still had uncommitted work when their rows ran, one row
consequently measured a file with the assertion missing and came back green, and the
whole thing had to be re-applied and re-run. Commit before running a revert matrix, and
treat a green revert row with the same suspicion as a green test.

### Durable lesson

**A roster derived from a keyword marker measures the marker, not the printed
card.** `OOS-CARDS1-3`'s 21 was right about the population it enumerated and
structurally unable to see two more defs printing the same line — and the fix for
that is not a better grep but a *second axis*: the type line. It is the third
census in this table to be short for a reason its own method could not detect,
and the first to be caught by a gate written to disagree with itself.

## Worker Handoff (PB-DX25c, `scutemob-205`)

**Stage 2 SHIPPED.** Closes `OOS-DX25b-3` (CR 115.7a's "another LEGAL target" at
redirect time). Stage 1 (production code, `cf89a213`) added `StackObject.
target_requirements` (hashed) + `rules::retarget::plan_target_change`, delegating
the whole redirect decision to `casting::validate_targets_inner`. Stage 2: fixed
the 6 fixtures stage 1 left red (real `TargetRequirement`s now recorded), inverted
`t9` + added `t9b`, wrote 9 new probes (`pb_dx25c_retarget_legality.rs`) + 1 bot-path
probe (`pb_dx25c_bot_retarget_is_legal.rs`, S1 only — S2 measured 0/30 fuzz-shaped
games reaching `Effect::ChangeTargets` at 80 turns, so it is NOT shipped, per the
plan's own instruction) + 5 roster/gate tests (`pb_dx25c_retarget_roster.rs`, R1-R5)
+ 1 in-source R6 test in `retarget.rs`, HASH 73 → 74 (gate-computed), `bare_lookup_
ratchet` ceiling 110 → 108, both card-def pointer comments updated (comment-only).
**Tests 4,469 → 4,491 (+22)**; PROTOCOL 35 unmoved; coverage unmoved 1,133/1,803 =
62.8% (proven by regeneration, reverted before commit). Full revert matrix (19
rows, all executed): 15 discriminate (12 exactly as predicted, 3 with a corrected
discriminator — V6/V8/V19); **4 honestly undiscriminated by the full workspace
suite** — V3 and V13 (both predicted-possible by the plan), V7 and V9 (NOT
predicted — `retarget_candidates`'s own `has_conceded` filter is shadowed by
`validate_mapped_targets`'s independent downstream check; the chooser-first
preference is shadowed by a coincidental fixture where the chooser is also first
in seat order). Two structural findings surfaced only by executing tests:
`TargetSpellWithSingleTarget`/`TargetSpellOrAbilityWithSingleTarget` cannot
observe the ACTIVELY-RESOLVING spell as a candidate (its own `StackObject` entry
is popped before its effect runs); `StubProvider`'s offer layer reads `obj.
characteristics.mana_cost` directly, a third instance of the "`ObjectSpec::card()`
is naked" gotcha, in a place `gotchas-infra.md` doesn't mention yet. Filed
**OOS-DX25c-1..4**; closed **OOS-DX25b-3** (4 corrections to its own claims — see
the audit doc). Full measurements, revert matrix results and the R4 non-vacuity-
floor anomaly diagnosis: `memory/primitives/pb-DX25c-execution-notes.md`.

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## M11-local Track (parallel to W6 — `crates/simulator`, `tools/`, no engine surface)

> **✅ MILESTONE COMPLETE — all 8 sessions shipped, closed by `scutemob-173` on
> 2026-08-01.** This section is now a record, not a queue.
>
> Deliberately its own section, not a W-row: M11-local ran concurrently with the W6
> primitive queue and touched a disjoint set of crates. Plan: `memory/m11-session-plan.md`
> (8 sessions, authoritative, now marked COMPLETE). **No new `Command`/`GameEvent` variant
> anywhere in the milestone** — the wire-neutrality claim held end to end; the pins at
> close are PROTOCOL **32** / HASH **70**, both moved by the W6 track (PB-DX1, PB-DX5) and
> never by M11-local, confirmed by an empty `git diff` over `crates/engine/src` +
> `crates/card-types/src` + `crates/card-defs/src` across the whole S8 branch.

| Session | Task | Status | Notes |
|---------|------|--------|-------|
| S1 steppable local-game core | `scutemob-147` | **SHIPPED** | `LocalGame` in `crates/simulator/src/local_game.rs`; `GameDriver::run_game` re-expressed on top of it |
| S2 deterministic pregame setup + mulligans | `scutemob-161` | **SHIPPED** | `setup.rs`: `build_initial_state` / `redeal` — see handoff below |
| S3 action parameterization + engine target queries | `scutemob-163` | **SHIPPED** | the crux (plan §8 R1) is closed: a human can cast a targeted spell. See handoff below |
| S4 view-model crate extraction + seat redaction | `scutemob-165` | **SHIPPED** | this session — `crates/view-model` (`mtg-view-model`); a seat view provably cannot leak another hand or any library order. See handoff below |
| S5 play-server crate skeleton + REST API | `scutemob-167` | **SHIPPED** (+ 2 review cycles) | this session — `tools/play-server` (axum, port 3040), the only crate in this milestone with async or IO. 5 routes + `ServeDir`, **16 tests** (15 `oneshot` HTTP + the source gate, which is a plain `#[test]` and constructs no router), **no port ever bound and now machine-gated crate-wide**. See handoff below |
| S6 play frontend — render and basic input | `scutemob-169` | **SHIPPED** | this session — `tools/play-server/frontend` (Svelte 5 + Vite 7), dev proxy to `127.0.0.1:3040`, `$viewer` alias importing the replay-viewer components **in place**. **Zero Rust**: `git diff main` over `crates/` + `tools/play-server/src` + `tools/play-server/Cargo.toml` is empty — **zero Rust anywhere**; the only change outside `tools/play-server` is one Svelte component, `tools/replay-viewer/frontend/src/lib/ZoneHand.svelte` (the review HIGH below). PROTOCOL 32 / HASH 69 unmoved, tests **4,040 / 0**. See handoff below |
| S7 targeting, combat and choice UIs | `scutemob-171` | **SHIPPED** | this session — `tools/play-server/src/{view.rs,api.rs}` populate `target_slots` / `target_min`/`max` / `modes` (with per-mode slots and ranges) / `attack` / `block` from `mtg_engine::{spell_target_requirements, ability_target_requirements, legal_targets_per_slot, target_count_range}` and the provider's own `DeclareAttackers`/`DeclareBlockers` payloads; `validate_combat_params` refuses an unoffered pair with a 400; `needs_x` now answers `ActivateAbility` (README Limitation 5 CLOSED). Four picker components + an `ActionBar` chain in CR 601.2b → 601.2c → 508.1 → 509.1 order. **One additive change outside `tools/play-server`**: `StackItemView::source_object_id` in `crates/view-model` — see handoff. PROTOCOL 32 / HASH 69 unmoved; play-server tests 18 → **24**. See handoff below |
| S8 playthrough hardening, docs, acceptance | `scutemob-173` | **SHIPPED — CLOSES THE MILESTONE** | this session — scripted playthrough on 5 seeds, human-only `Concede` + `OrderBlockers`, error audit, `GET /api/game/report`, docs, decisions.md, 8 gates. See the handoff below |

**S8 handoff — MILESTONE CLOSE (2026-08-01, `scutemob-173`)**

- **A scripted human plays five full games with nothing swept under the rug.**
  `crates/simulator/tests/local_game_playthrough.rs` drives seat 1 through four-player
  games on seeds 1/7/42/1234/9001 with a deterministic policy (land → cheapest castable
  → attack → pass), through `LocalGame` alone. All five reach the turn cap with **0
  engine rejections and 0 invariant violations**. The five games run over the **real**
  1,804-def pool through `setup::build_initial_state`, not the 99-Plains fixture the rest
  of `crates/simulator/tests` uses, on a hand-built 64 MiB thread (deep resolution
  exhausts the 2 MiB test stack — pre-existing, OOS-DP3-9).

- **Running it found four defects in one afternoon, which is the argument for the test.**
  None was in the plan; each was fixed at its own layer.
  1. **`invariants::check_stack_consistency` compared two different id spaces.** A cast
     spell's card gets `ObjectId` *n* in the Stack zone and its `StackObject` gets *n+1*
     (`casting.rs` mints them consecutively), so the check fired **twice per spell** and
     once per ability, always, in games with no defect. Measured: **501 spurious
     violations across 500 fuzz games** at the merge base, **0** after. Rewritten against
     `StackObjectKind::Spell { source_object }`, the id the two sides actually share.
     This is what `OOS-DP3-9`'s "long games trip `stack_consistency`" always was.
  2. **`mana_solver` tapped one permanent twice.** It held one entry per (permanent ×
     mana ability) and marked only the chosen entry spent, so a permanent with two mana
     abilities was planned into two `TapForMana`s; the second is refused ("already
     tapped"). Fixed with `spend()`, which marks every entry for the permanent.
  3. **`HeuristicBot` froze the table, twice over.** It scores every real play above
     `PassPriority`, so a *free repeatable* action loops forever: `lightning_greaves`'
     Equip `{0}` (which resolves as a **no-op** — its `ActivatedAbility` declares
     `targets: []` while its effect names `DeclaredTarget { index: 0 }`), and
     re-declaring the same combat. A per-turn preference cap (`RepeatKey`) fixes both,
     **in the bot rather than the provider**, so the fuzzer's `RandomBot` draw sequence
     is untouched.
  4. **The playthrough's own `max_commands` was too tight.** `GameDriver`'s
     `max_turns * 200` is the fuzzer's ratio and the fuzzer's games start with empty
     hands; a real four-player table runs ~260 commands/turn, so the *command* valve
     fired before the turn cap and the plan's terminal state was unreachable.

- **Two of those bottomed out in the engine and were filed, not fixed** (M11-local makes
  no engine change — an empty `git diff` over `crates/engine/src` proves it):
  - **`OOS-M11-7`** — CR 704.3 says SBAs are checked whenever a player *would receive*
    priority. This engine checks them on **step entry** and at **resolution**, not on
    each pass within a step, so a Treasure sacrificed to pay a mana cost sits legally in
    the graveyard for several priority passes. Self-healing, never wrong at rest —
    the playthrough asserts the strictly stronger property that **no token is outside the
    battlefield in the final state**, and reports the transient class separately.
  - **`OOS-M11-9`** — neither `StubProvider` nor `combat.rs::handle_declare_attackers`
    gates "attackers have already been declared this combat". CR 508.1 makes it a
    turn-based action performed **once**; the engine accepts a second, a third, and so on.
    With a vigilant attacker (still untapped, so still `eligible`) this is unbounded.
    **CLOSED 2026-08-04 by PB-DX21** (`scutemob-200`) — both halves: the engine guard
    (`GameStateError::AlreadyDeclaredAttackers`) and the offer (`legal_actions.rs:878`).

- **Item 2's premise was stale and that is the reusable part.** The plan (2026-07-26)
  lists Echo / Cumulative Upkeep / Recover as needing new `LegalAction` variants.
  **PB-DP4 (`scutemob-152`) had already shipped all three**, with SR-38 affordability
  gating, later the same day the plan was written. Only `OrderBlockers` (CR 509.2) was
  genuinely unsurfaced. *A plan item that names missing work is a dated claim; check the
  code before building it.* Three tests now verify the existing three reach a human seat
  through `LocalGame`, which is the half `legal_actions.rs`'s own tests do not cover.

- **`Concede` and `OrderBlockers` are offered to human seats ONLY**, appended by
  `local_game::human_only_actions` rather than by `StubProvider`. Two independent reasons,
  both load-bearing: a bot must never auto-concede, and *appending to the provider's list
  re-rolls every `RandomBot` draw downstream of it*, which would change what every
  recorded fuzz seed reproduces. That constraint is what let the R11 gate be **measured**
  rather than argued.

- **The R11 fuzz gate, measured** (`memory/m11/s8-fuzz-parity.md`): 500 games, same seed,
  merge-base worktree vs branch. **0 games differ in turns, commands or outcome.**
  Violations 501 → 0, all of them finding 1's false positives. The gate **cannot** be run
  at the plan's default `--max-turns 200` — `mtg-fuzzer` stack-overflows at the *merge
  base* (OOS-DP3-9), reproduced single- and multi-threaded and with a 128 MiB
  `RUST_MIN_STACK` — so it is 500 games of up to **40** turns, and the record says so.

- **`GET /api/game/report`** ships the repro artefact: `{seed, config, PROTOCOL +
  fingerprint, HASH, final `public_state_hash`, journal}` plus an "Export report" button.
  It is a **pure read** — it uses `journal()` and not `take_new_records()`, so an export
  cannot swallow event lines the live feed has not shipped (tested). It is also the **one
  payload in `play-server` that is not seat-redacted**, deliberately: a redacted repro is
  not a repro. Safe only because M11-local is one human, three bots, one process, no
  networking. **Re-scope it at M10a.**

- **`OOS-M11-8` CLOSED** (the S7 handoff routed it here): `auto_tap_commands_for` now adds
  `x_value × mana_cost.x_count` generic before planning, so a human can cast an `{X}`
  spell. Verified to discriminate by disabling the fix — *and the first attempt at that
  check was invalid*: clippy `-D warnings` failed the disabled build, cargo reused the
  stale test binary, and the test "passed". **A revert-and-rerun proves nothing unless
  the rebuild succeeded.**

- **Gates at close**: tests **4,097 / 0** (merge base measured at **4,072**, so **+25** —
  2 playthrough, 15 human-action, 8 play-server; the implement phase pinned **4,092/+20**
  and the close-out fix cycle added the other 5, so both figures are real and this is the
  final one — measured by running the suite, and the per-file split re-derived against it
  rather than carried), clippy `-D warnings` clean, `cargo fmt
  --check` clean, `tools/check-defs-fmt.sh` 1,804 defs clean, `cargo build --workspace`
  clean, **PROTOCOL 32 / HASH 70 unmoved** (empty diff over `protocol.rs` / `hash.rs` and
  gate-computed by running the `core` suites), fuzz parity as above.

- **CLOSE-OUT ADDENDUM (2026-08-02, same task, after a kitty crash mid-fix-cycle).** The
  `milestone-reviewer` pass filed **MR-M11-01..21** into
  `docs/mtg-engine-milestone-reviews.md`; the fix cycle it opened was interrupted, and the
  resume finished it. **All 10 HIGH/MEDIUM are now closed; of 8 LOW, 1 closed and 7
  open**, each of the seven re-verified as genuinely unchanged rather than assumed. The
  blanket "LOW needs no fix phase" was only half the account and is worth correcting here:
  the reviewer's `memory/m11-fix-session-plan.md` had scoped **four** LOWs into its two
  sessions. **MR-M11-12** was taken (a doc cite pointing at a sentence that does not exist
  — the lying-cite class, doc-only, and the fix documents *both* halves of `OOS-M11-2`,
  the second verified at the read site rather than copied from CLAUDE.md);
  **MR-M11-13/14/17** were deferred with the reason recorded at each item, MR-M11-14 on
  the plan's own advice, since its Session 2 gate names that item as one of the two that
  can perturb the 500-game fuzz parity — and that parity run is the branch's evidence for
  acceptance criterion 5977. The plan's checkboxes are now accurate rather than untouched.
  Five things worth carrying:

  - **The HIGH is the one nobody's gate could see, and it is the reusable shape.**
    `GameSummary.seed` shipped on **every** seat payload for three sessions. Since
    `setup::build_initial_state` is deterministic in its config alone and
    `session::config_for` fixes every other input, `(seed, players, mulligan_count)`
    *rebuilds* every bot's opening hand and library order — the exact pair Architecture
    Invariant 7 names, and the exact words of the milestone's own acceptance criterion.
    Both Invariant-7 gates stayed green the whole time, because one searches the body for
    card **names** and the other scans source for omniscient **view-model entry points**,
    and a seed is neither. **A redaction gate checks the channel it was written for; a new
    channel is invisible to it.** There are now three gates for three channels — names,
    reconstruction keys, free-form engine strings — and the table is in the play-server
    README so the next surface starts from three rather than rediscovering two.
  - **A status word is not a disposition.** Every one of the 18 findings still read `OPEN`
    while eight had shipped, and three of the eight had landed their *code* fix without
    the *test* the finding asked for — so three behaviour changes were held by prose.
    Those three tests now exist and were proven to discriminate by execution: with each
    fix reverted its test fails and the other 29 stay green. **The first revert did not
    compile** (two helpers went dead under `-D warnings`), which is the S8 `{X}` lesson
    recurring within the same task: *a revert-and-rerun proves nothing unless the rebuild
    succeeded.*
  - **Two findings are closed on part of what they asked for, and the part left is named
    in the reviews doc rather than implied.** MR-M11-04's companion handler-set gate is an
    M10a item (the narrowing, which makes the *existing* claim true, was taken);
    MR-M11-06's code half is a capability addition, not a defect repair, and its seed
    **`OOS-M11-10`** is filed — which was the half the finding actually flagged, since the
    in-source comment had promised "to be filed for S6/S7" through three sessions that all
    shipped without filing it. *A comment asserting a seed exists is not a seed.*
  - **`HASH 69` in the reviews doc was stale in four places.** The claim ("unmoved by
    M11-local") was true; the number was not — HASH moved 69 → **70** in PB-DX5 on the
    parallel W6 track before this branch forked. Found by reading
    `crates/engine/src/state/hash.rs` rather than carrying the figure forward, which is
    the same move that caught three arithmetic slips inside PB-DX5 itself. This file
    already had it right.
  - **A fix plan nobody ticks reads as a fix plan nobody ran.** `m11-fix-session-plan.md`
    still had all fourteen boxes unchecked while eleven of its items had shipped — which
    is the same failure as the reviews doc's eighteen `OPEN` rows, in a second file, and
    it is what made the close-out's first account of the LOWs wrong (it said all eight
    were untouched-by-design; four had actually been *scoped into sessions*, so three of
    them needed a stated deferral rather than a blanket rule). Both files are now
    accurate. **The generalisable bit: the artefact a reviewer produces is a second place
    the work has to be recorded, and finishing the work does not update it.**

- **What M11-local did NOT deliver, stated plainly**: card images come from Scryfall over
  the network rather than a cache (M14); the bug-report artefact has no free-text
  description field; no automated test spans browser + game, because there is no frontend
  test harness (plan §8 R7 — revisit at M13); `StubProvider` still enumerates no
  Adventure, alt-cost, or Convoke/Improvise/Delve casts (R4); `OOS-M11-2`'s
  layer-resolution half is open.

**S7 handoff (2026-08-01, `scutemob-171`)**

- **A human can now attack, block, and cast a targeted / X / modal spell.** Server side,
  `ActionOptionView` gained `target_slots` (populated from
  `mtg_engine::spell_target_requirements` / `ability_target_requirements` +
  `legal_targets_per_slot`), `target_min`/`target_max` from `target_count_range`, `modes`
  with **per-mode** `target_slots` + ranges, `mode_min`/`mode_max`, and `attack` / `block`
  payloads rendered straight out of the provider's own
  `LegalAction::DeclareAttackers { eligible, targets }` /
  `DeclareBlockers { eligible, attackers }`. Frontend side, four pickers chained by
  `ActionBar` in CR order — `ValuePrompt` (601.2b) → `TargetPicker` (601.2c) →
  `AttackerPicker` (508.1) → `BlockerPicker` (509.1) — accumulating one `params` object and
  submitting once. Click-through goes through the same entry point, so a targeted spell
  cannot be cast targetless from either path. PROTOCOL 32 / HASH 69 unmoved; play-server
  tests 18 → **24**; `npm run build` clean at 143 modules, 0 warnings.

- **The S6 review's three MEDIUMs are all closed, and the asymmetry between them is the
  durable part.** The targeted-spell gap announced itself with a 422 every single time. The
  other two — `DeclareAttackers`/`DeclareBlockers` silently submitting an **empty set**, and
  an activated ability's `{X}` silently announced as **0** — were indistinguishable from a
  normal click. The `declares none` and `X = 0` badges in `ActionBar.svelte` are gone,
  because they were warnings about an absence that is now filled.

- **`needs_x` for activated abilities: the S6 note was true and looked in the wrong place.**
  It said `LegalAction::ActivateAbility` does not carry the ability's `ActivationCost`,
  which is correct — but the action carries `source` and `ability_index`, and those reach
  the **layer-resolved** `Characteristics::activated_abilities` entry, whose
  `cost.mana_cost.x_count` is the answer. `mirror_entity` (deck-legal, `x_count: 1`, one
  click makes every creature 0/0) now gets a real prompt. **Generalisable: "the action does
  not carry X" is not the same claim as "X is unreachable from the action".**

- **A real defect surfaced by populating the field, not by reasoning about it:
  `StackItemView::id` is a `StackObject` id and `Target::Object` names a `GameObject`.**
  Nothing bridged the two, so every target that is a spell on the stack — i.e. every
  counterspell's target — rendered as `(unknown card)`. Observed on a real payload before
  the fix (seed 2 offers `Cast Dispel`; its one candidate came back `"(unknown card)"` while
  the stack held `Dark Ritual`). Fixed by adding **`StackItemView::source_object_id`** to
  `crates/view-model` — the id was already being computed in `build_zones_view` for
  `source_name` and thrown away. **This is a deliberate scope deviation** (plan §4 S7 says
  `tools/play-server` and its frontend); it is additive, exposes strictly less than the
  `source_name` already shipped beside it, and the alternative was leaving counterspell
  targets unlabelled. Exposing the bare id leaks nothing: CR 405.1 makes the stack public,
  `redact_stack` blanks a face-down source's *name*, and a face-down **permanent** already
  keeps its real `object_id` for the same reason.

  The same fix removed a latent hazard on the play-server side: `NameIndex` had been writing
  `item.id` — a `StackObject` id — into a map keyed by `ObjectId`. Nothing looked it up, and
  the stack is inserted last, so a numerically-colliding id could have overwritten a real
  permanent's name. **Two id spaces that both count from small integers, in one map.**

- **New seed `OOS-M11-8`: a non-zero `{X}` cannot be paid for through this API.**
  `LocalGame::auto_tap_commands_for` reads the spell's **printed** `mana_cost` and knows
  nothing about `cast.x_value`, so it taps for the base cost and the engine then refuses the
  cast — observed as `422 "player does not have enough mana to pay the cost"`, not inferred.
  The human's workaround exists and works (tap sources manually first; S3 made auto-tap
  conditional on the pool, so a covered base cost leaves the surplus for X) and is the path
  `test_x_value_is_forwarded_to_cast_spell_data` drives. The fix belongs in
  `crates/simulator`, out of S7's scope. **S8 item 2's "surface the invisible optional
  decisions" audit should pick this up** — it is the same family as `OOS-M11-2`.

- **Fixtures were observed, not chosen.** A temporary `#[ignore]`d probe swept
  `players` ∈ {2, 4} × `seed` ∈ 0..12 through `oneshot` (**no port bound**) and reported per
  game whether the human is ever offered a `DeclareAttackers`, a `DeclareBlockers`, or a
  `CastSpell` with a non-empty candidate list. Exactly **one** swept pair reaches both halves
  of combat: `players: 4, seed: 6` (attackers turn 5, blockers turn 6). `seed: 9` reaches a
  targeted removal spell (Doom Blade) with real creature candidates. Both are pinned as
  `COMBAT_SEED` / `TARGET_SEED` with a note to **re-observe rather than guess** when a
  card-def completeness flip re-deals the decks — which PB-DX4 has already demonstrated it
  will. The probe was deleted; `git diff` over `tools/play-server/src` shows no probe.

  The sweep also established two absences worth recording rather than discovering later:
  **no seeded game in the sweep dealt the human a modal spell or an `{X}` spell**, so the
  `modes` path and the `needs_x`-on-`CastSpell` path are right by construction and
  **unexercised by any test**. Said plainly in the README rather than left implied.

- **A non-vacuity check that failed, and the fix.** The first version of
  `test_action_option_target_slots_match_engine_query` stopped on the first action with any
  non-empty slot — which at `seed: 9` is a slot with **one** candidate. Reversing the
  candidate order inside `action_option_view` left the test **green**, because reversing a
  one-element list changes nothing: the per-slot *order* assertion, the whole point of that
  test, was never being exercised. The fixture now demands a slot with **at least two**
  candidates, and the same perturbation turns it red. `validate_combat_params` was checked
  the same way (neuter it → `test_declare_blockers_rejects_ineligible_blocker` goes red).
  **Carry: "the assertion is present" and "the assertion can fail" are different facts, and
  only the second is worth anything.**

- **The `ModeSelection` lookup is the one engine rule this crate restates, and it is
  recorded as such.** `rules::casting::spell_mode_selection` is `pub(crate)`, so
  `view::action_modes` re-derives it through the public `GameState::card_registry` for the
  spell case (the *ability* case reads the layer-resolved `ActivatedAbility::modes` and
  cannot drift). It is confined to which modes to *offer*; the engine re-validates
  `modes_chosen` on the cast path regardless (CR 601.2b, PB-DP3), so a drift is a wrong
  picker, never a wrong game state. Everything else — target requirements, target legality,
  combat eligibility — is delegated, per plan §1 fact 4.

- **`ModeOptionView.label` is a truncated `Debug` of the mode's `Effect`.** There is no
  per-mode oracle text anywhere in the DSL (`ModeSelection.modes` is a bare `Vec<Effect>`),
  so the label is visibly machine-shaped rather than pretending to be printed text.

- **Review cycle: 0 HIGH / 3 MEDIUM / 4 LOW, all 7 applied — and the sharpest one is a
  correctness bug the tests could not have caught.** `TargetRequirement::UpToN { count }`
  is a **single** requirement worth up to `count` targets (`target_count_range` adds
  `count` to the maximum for it; `validate_targets_inner`'s second pass assigns several
  announced targets to that one slot), but `legal_targets_per_slot` returns one entry per
  *requirement*. The first DTO shape was `Vec<Vec<TargetOptionView>>` plus a **collective**
  `(min, max)`, from which a client cannot tell *which* slot the slack belongs to — so the
  obvious one-pick-per-slot reading silently capped `force_of_vigor` (`Complete` by the
  `#[default]` derive, deck-legal, one `UpToN { count: 2 }`) at destroying **one** of its
  "up to two" targets. Fixed by making a slot a struct: `TargetSlotView { min, max,
  candidates }`, each range computed by handing `target_count_range` a one-element slice so
  it cannot drift from the collective one, with `TargetPicker` multi-selecting up to a
  slot's own `max`. **No test could have found it**: no seeded game in the fixture sweep
  deals such a card, so the multi-select branch still ships unexercised and the README says
  so. **Carry: a DTO that flattens a domain concept ("a slot") onto a container shape ("a
  list of candidates") loses whatever the concept carried besides its contents.**
- **The second MEDIUM is the same class as PB-DX3's: a doc comment contradicted by its own
  code.** `action_option_view`'s `# Cost` block claimed the candidate sweep "runs **only**
  for actions that declare at least one target requirement" — while the function's own
  modal branch calls the sweep once per *mode*, and a per-mode-targeting card's
  option-level requirement list is empty **by design**. So the exact actions the sentence
  called free were the ones paying `modes × slots × candidates`. And `queries.rs` asks in
  terms that this be **measured** before a browser polls it; the comment had substituted an
  argument. Measured with a temporary probe: 4 players / seed 9 / turn 17, 12 actions of
  which 1 targeted, 22 candidates → one `decision_view` ≈ **201 µs**, debug build, mean of
  20. **A first draft of the corrected paragraph carried invented numbers ("24 actions, 91
  candidates, under 3 ms") and the probe contradicted every one of them** — which is the
  whole argument for running the probe rather than reasoning.
- **The third MEDIUM: a redaction test whose leak oracle could not fire.**
  `test_target_option_labels_are_seat_redacted` asserted no other seat's hand-card name
  appears in a target label — but target candidates come only from Battlefield / Stack /
  Graveyard (all public), and `redact_hands` rewrites a hidden hand card's `object_id` to 0,
  so no id it collects can key into a hand entry. Deleting `redact_hands` entirely would
  have left it green. Fixed by adding the assertion that **does** bite — every object label
  equals the name the *seat-redacted* `StateViewModel` carries for that id, re-derived from
  the session rather than read off the payload — verified by perturbation (sourcing the
  label from the id instead of `NameIndex` turns it red: `left: "obj-409", right:
  "Vampire"`). The hand-name loop is kept and **relabelled a forward guard** against a
  future widening of `legal_targets_per_slot` into a hidden zone, not evidence that
  redaction works today. The one reachable divergence at this site — a face-down
  battlefield permanent, CR 708.2a — is unfixtured and said so.
- **And the fix cycle's own record overstated that repair, which the re-review caught —
  the exact failure mode this project keeps hitting.** The perturbation cited as proof
  (sourcing the label from the id instead of `NameIndex`) is the *trivial* one. The
  redaction-relevant perturbation is **building `NameIndex` from the omniscient view**, and
  it was then run: `api.rs::seat_view` was edited to do exactly that and **the whole crate
  stayed green — all 23 tests**, including S5's whole-body sweep
  `test_seat_view_over_http_contains_no_other_hand_card_names`. So no behavioural test in
  this crate guarded the chokepoint at all.
  The reason is structural, not a gap in those tests: `NameIndex` is only ever *queried*
  for ids that appear in an action, a target candidate or a combat list, and every one of
  those is in a public zone, so on every id that ever gets labelled the two views **agree**.
  The only construct that separates them is a face-down battlefield permanent (CR 708.2a),
  which no seeded game reaches.
  Closed the way this project closes an unfalsifiable invariant — with a **source gate**:
  `test_production_code_never_builds_an_omniscient_view` scans the production region of
  every `src/*.rs` (comment- and string-blanked by the existing `code_only`, so a doc
  comment naming the symbol neither satisfies nor trips it) for `from_game_state(` and
  `Viewer::Omniscient`, and **was proven to catch the exact edit above** rather than
  assumed to. Its own non-vacuity check went red on first run and taught the batch
  something: the two needles are not in the same position — `from_game_state(` is used for
  real in the test region (the oracle), while `Viewer::Omniscient` appears **nowhere in
  this crate** and is a forward guard whose *mechanism* is pinned instead. play-server
  tests 23 → **24**.
- Four LOWs, all applied: the `{X}` 422 was observed on `casting.rs`'s `x_count == 0`
  fallback path rather than on a real `{X}` card (the seed row and README now say which);
  `StackItemView::source_object_id`'s leak argument did not cover the hidden-zone source
  `redact_stack`'s own doc raises (now does); `BlockerPicker` cannot express CR 509.1b
  "can block an additional creature" while the server deliberately permits it (recorded as
  a client limitation); README limitation numbering and a stale "16 tests" (the pre-S7
  count was 18 — PB-DX4 added two).
- **Still unverifiable headless, and marked so in the README rather than glossed**: every
  DOM and keyboard behaviour — clicking through the picker chain, Escape aborting a chain
  mid-way, `space` being suppressed while a picker is open, `<select>` default rendering in
  the attacker/blocker pickers. There is still no frontend test harness (plan §8 R7), and
  S6's row 2 stands as the proof that a green `npm run build` says nothing about whether a
  component survives a redacted payload.

**S6 handoff (2026-08-01, `scutemob-169`)**

- **Read this first: the session's one real bug was invisible to every gate it had, and the
  fix is in the replay viewer, not here.** `ZoneHand.svelte` keyed its `#each` on
  `card.object_id`. That is right for the omniscient replay viewer and **fatal** for a
  seat-redacted payload: `redact::redact_hands` replaces every unreadable hand card with
  `hidden_placeholder()`, whose `object_id` is **0**, so three bot hands of seven cards each
  arrive with one distinct key apiece. Svelte 5 evaluates `length > keys.size` and calls
  `each_key_duplicate`, which **throws in production as well as DEV**; with no
  `<svelte:boundary>` the throw escapes the effect flush and takes the mount down. **The play
  surface rendered nothing at all** — while `npm run build` was clean at 135 modules and 0
  warnings, the Rust diff was empty, and 4,040 tests were green. Caught in review by
  evaluating Svelte's own condition against the dumped hands (`7 > 1` per bot seat, `7 > 7`
  false for the human's), not by a build and not by a browser. Fixed **in the shared
  component** as `card.hidden ? \`hidden-${i}\` : card.object_id` — the flag the redactor
  sets, not the sentinel 0 — inert for the viewer, and precisely the reason the plan aliases
  the component instead of copying it. `hidden_placeholder` has one call site, so hands are
  the only zone at risk (checked). **Carry into S7: the viewer's components were written
  against an omniscient view model, and every id-uniqueness assumption in them is now also a
  claim about the redacted one.**
- **The play surface has a UI.** `tools/play-server/frontend` — Svelte 5 runes + Vite 7, the
  same versions as the replay viewer's frontend. Eight source files
  (`App.svelte`, `app.css`, `main.js`, `lib/{api,stores}.js`,
  `lib/{PlayApp,ActionBar,EventFeed}.svelte`) and a `package-lock.json`; `npm run build`
  emits `tools/play-server/dist/`, which S5's `ServeDir` fallback already mounts. Build
  clean: 135 modules, 0 warnings.
- **`$viewer` imports the replay viewer's components rather than copying them, and the claim
  is checked.** The alias is `fileURLToPath(new URL('../../replay-viewer/frontend/src/lib',
  import.meta.url))` — absolute at resolve time, because a bare relative alias target
  resolves against the *importing* file and would break for `src/lib/` importers. Evidence
  both ways: `find frontend/src -type f` lists eight files with no `Zone*` and no
  `PhaseIndicator`, while the production CSS bundle contains those components' scoped rules.
  Promotion to `tools/ui-shared/` stays deferred (plan §8 R8).
- **Zero Rust, and the gate is the whole surface rather than the wire files.**
  `git diff main -- crates/ tools/play-server/src tools/play-server/Cargo.toml` is
  **empty** — zero Rust anywhere. The **only** change outside `tools/play-server` is one
  Svelte component, `tools/replay-viewer/frontend/src/lib/ZoneHand.svelte`, and it is the
  review HIGH above; an earlier draft of this bullet claimed an empty `tools/replay-viewer/`
  diff and the fix cycle falsified it in the same commit that introduced the fix.
  PROTOCOL **32** / HASH **69** unmoved; workspace
  `cargo test --all` **4,040 / 0**; `clippy --workspace --all-targets -D warnings`, `cargo
  fmt --check` and `tools/check-defs-fmt.sh` (1,804 defs) all clean. The test count is
  unchanged from the merge base *by construction* — no Rust test target was added and the
  plan explicitly gives this session no frontend test harness (§4 item 7: S5's API tests are
  the automated coverage).
- **The manual checklist was run, not asserted — and that is the part worth carrying.** A
  temporary `#[ignore]`d probe was added to `main.rs`'s existing `mod tests`, driven through
  `tower::ServiceExt::oneshot` (**binding no port**, per plan §7 constraint 1), and
  **removed again**; the frontend was then validated against the dumped `SeatView` payloads
  rather than against a written-down idea of them. Established at the pinned
  `--seed 0 --players 4 --bot heuristic`: a **7-card** opening hand (Island, Mist Intruder,
  Misdirection, Nyxbloom Ancient, Accorder's Shield, Helm of the Host, Swan Song); a land
  drop through `{index: 1, kind: "PlayLand", object_id: 2}` moving hand 8→7 and battlefield
  0→1; 25 `PassPriority` submissions; **10–21 rendered, seat-redacted `EventView` lines per
  response**; **turn 4** reached in 25 submissions. A second run preferring `CastSpell` was
  needed for the stack, because the land-only policy never put anything on it in three turns
  — it produced `zones.stack: [{id: 404, kind: "spell", source_name: "Accorder's Shield"}]`.
  The two steps that genuinely cannot be checked headlessly (launching the binary; keyboard
  and DOM events) are **marked unverifiable in the README**, not glossed.
- **The mulligan `LegalAction`s are unreachable on this surface.** `legal_actions.rs` and
  `local_game.rs::decision_kind_for` both gate `TakeMulligan`/`KeepHand` and
  `DecisionKind::Mulligan` on `is_first_turn_of_game && turn_number == 0`, while
  `setup::build_initial_state` + `GameStateBuilder` leave a fresh table already *in* turn 1
  — `session.rs::is_pregame`'s own doc says the condition is unsatisfiable, and the payload
  agrees (pregame decision `kind: "Priority"`, one option, `Pass priority`). So the UI gates
  its pregame block on `summary.pregame` alone and uses the dedicated
  `POST /api/game/mulligan`. **"Keep this hand" has no server-side representation at all** —
  `take: false` only re-renders and `pregame` is `command_count == 0` — so it is a
  client-side flag, said out loud in `PlayApp.svelte` rather than hidden.
- **The redactor rewrites a hidden card's `object_id` to 0, and click-through must refuse
  those rather than merely fail to match them.** `redact::hidden_placeholder` emits
  `{hidden: true, name: "Hidden card", object_id: 0}`; the playthrough carried **569** such
  entries, every one id 0, while the lowest id any `ActionOptionView` ever carried was 2.
  There is no collision today — but all seven of a bot's hand cards share one id, so a single
  action about object 0 would make all seven of them submit it. Matching on a sentinel is the
  wrong shape whether or not it currently collides. Scope worth knowing: `hidden` is a field
  of `CardInZoneView`, **not** of `PermanentView`, so an opponent's face-down permanent keeps
  its real id and is matched normally — which is right, since an action naming it is about an
  object the seat can point at without knowing what it is.
- **`DeclareAttackers` / `DeclareBlockers` submit an EMPTY set, silently** (review MEDIUM).
  `params.rs` maps default params straight to `Command::DeclareAttackers { attackers: vec![] }`
  — legal, irreversible, and *quieter* than the targeted-spell case, which at least fails
  loudly. The buttons stay enabled (disabling would deadlock a combat where the declaration
  is the only offered action, and CR 508.1 makes "no attackers" legal) but are marked
  `declares none` with a tooltip, and the README says it plainly. S7's pickers are the fix.
- **An activated ability's `{X}` is announced as 0, and the client cannot tell which
  abilities have one** (second review cycle). `params.rs` maps default params to
  `x_value: None`, read as `unwrap_or(0)`; `view.rs::action_needs_x` answers `CastSpell` only
  (README Limitation 5), so `needs_x` is `false` regardless. Reachable and destructive on a
  deck-legal card: **`mirror_entity`** declares no `completeness` field — `Complete` by the
  `#[default]` derive, the same silent-defect generator PB-DX1 and PB-DX3b each hit — and its
  activated ability has `x_count: 1`, so one click makes every creature 0/0 and the board
  dies to SBAs, with no error to read. Annotated `X = 0` **unconditionally** on the kind
  because there is no flag to branch on; the tag goes away when S7 closes Limitation 5.
  **All three silent-degradation paths are the same hole — the client can only send
  `params: {}` — and only the targeted-spell one fails loudly.**
- **Three review LOWs applied**: `jsconfig.json`'s `$viewer/*` path was off by one directory
  (editor-only; `vite.config.js` was right, so the build never noticed); the "omit and take
  the CLI default" rationale did not hold for `players`, pre-seeded to `'4'` and therefore
  overriding a server run with `--players 6` on every New game; and the event feed keyed its
  `#each` on the array index against a front-truncating window, now on a monotonic `seq`
  stamped at append.
- **Two facts recorded for Session 7.** (1) **`ZoneStack` declares `onCardClick` and never
  invokes it** — a dead prop in the viewer, harmless here (no `LegalAction` with an
  `object_id` names a stack object, `view.rs::action_object`) but load-bearing for S7, which
  renders targets on stack items. (2) A **targeted spell cast from this UI fails with a real
  422** — `{"error":"invalid target: expected 1..=1 target(s) but got 0","kind":"rejected"}`,
  observed, not imagined — because `target_slots` is empty until S7 and the client sends
  `params: {}`. Correct S6 behaviour under CR 601.2c; the error strip surfaces it instead of
  swallowing it, and S7's `TargetPicker` is exactly what closes it.
- **One server-side oddity found and deliberately NOT fixed** (fixing it is a Rust diff this
  session's acceptance criteria forbid): `ActionRequest.params` carries a plain
  `#[serde(default)]`, which routes through `ActionParamsDto`'s **derived** `Default` where
  `auto_tap` is `false`, while an explicit `"params": {}` takes the field's
  `#[serde(default = "default_auto_tap")]` and gets `true`. Omitting `params` and sending
  `{}` are therefore not equivalent, which is not what the DTO's doc comment implies. The
  client always sends `{}`, so it is unaffected. **Reasoned from the source, not executed**,
  and nothing in `tools/play-server` names `auto_tap` at all, so nothing pins it either way —
  a one-test job for S7 or S8.

**S5 handoff (2026-08-01, `scutemob-167`)**

- **A full game is now playable over `curl` alone.** `POST /api/game` → `GET /api/game` →
  `POST /api/game/action` → `POST /api/game/mulligan` → `GET /api/healthz`, plus a `ServeDir`
  fallback to `dist/` for S6's frontend. Tests 4,008 → 4,016 → 4,023 → **4,024** across the two
  review fix cycles (+16 in the crate's inline `mod tests`: 15 `oneshot` HTTP tests plus the
  source gate, which is a plain `#[test]` and builds no router). `git diff main -- crates/engine/src
  crates/card-types/src crates/card-defs/src` is **empty**; PROTOCOL **32** / HASH **69**
  unmoved; `crates/simulator` and `crates/view-model` untouched — S5 needed nothing added to
  either, **including its fix cycle**: MEDIUM 1's root cause is `LocalGame::decision_seq`
  restarting at 0, and the fix is an offset in `PlaySession`, not an edit to the simulator.
- **No port is ever bound, and that is now a gate rather than a promise — but the first
  version of the gate cut in the wrong place.** `TcpListener` / `axum::serve` appear only
  inside `async_main`, which no test calls; all 15 HTTP tests drive
  `build_router(state, &PathBuf::from("nonexistent_dist"))` through
  `tower::ServiceExt::oneshot`. `test_no_socket_symbol_appears_in_the_test_region` now walks
  **every `.rs` file** under the crate's `src/` and `tests/` (rooted at `CARGO_MANIFEST_DIR`,
  so it does not depend on the working directory) and fails on any of the four symbols inside
  a test region — line-anchored `#[cfg(test)]` in a `src/` file, the whole file for a
  `tests/` one. Needles are assembled with `concat!` so it does not match its own source.
  **As first shipped it read `main.rs` alone and cut at the first *textual* `#[cfg(test)]`,
  which is the one spelled out in that file's own module doc comment** — the "test region"
  therefore began at a paragraph of prose and the gate passed only because all four needles
  happen to be typed to the left of the marker in that sentence; its non-vacuity guard was
  satisfiable by the same paragraph. Both fixed in fix cycle 2, with three mutation proofs
  run rather than argued. Plan §7 constraint 1 is machine-held crate-wide for S6/S7.
- **Four review findings worth carrying into S6, because the client will encounter all
  four.** (1) The wire `seq` is **not** `LocalGame`'s `seq`: `PlaySession::seq_base` makes it
  monotonic across restarts and mulligans, because without it game B's first decision reused
  game A's `seq: 1` and a stale tab's post was **accepted with 200** (observed — the new
  game's `command_count` moved 0 → 4). (2) A body the extractor rejects is now **400 in the
  JSON envelope**, not axum's bare `text/plain` **422** — the old behaviour collided with
  this crate's own meaning for 422 ("the engine refused it"), so a client-side typo read as
  an engine rejection; and `POST /api/game {"playerz":9}` used to answer **200 with a default
  game** because `Option<T>`'s `FromRequest` is `.ok()`. An **absent** body still means "use
  the CLI defaults". (3) `POST /api/game` — and only it — recovers from a poisoned session
  mutex, so one engine panic no longer costs a process restart on the surface that exists to
  find engine panics. **That recovery had to be made atomic in fix cycle 2**: as first
  written it cleared the poison flag *before* the fallible rebuild, and `session::new_game`
  fails on a client-supplied seed (a colourless commander's deck is padded with Forests,
  which `validate_deck` refuses under CR 903.5c — 7 failing tables in 180 `(players, seed)`
  pairs), so the `?` left the half-mutated session readable at **200** where the unfixed
  code had answered 500. The corrupt session is now `take()`n in the same straight-line
  block that clears the flag. (4) `GameSummary.seed` is the **base** seed; after a mulligan the table
  came from `redeal_seed(seed, seat, count)`, so a reproducible bug report needs
  `seed` + `players` + `bot` + `mulligan_count` — all four are already in every `GameSummary`.
- **The multi-thread runtime flavor is a correctness requirement, not a performance choice.**
  `tokio::task::block_in_place` **panics** on a current-thread runtime — which is exactly what
  a plain `#[tokio::test]` builds. Every async test carries
  `#[tokio::test(flavor = "multi_thread")]`. The 8 MB worker stacks are the separate,
  inherited reason (`tools/replay-viewer/src/main.rs`'s `fn main` (`:50-65`): deep trigger chains overflow
  tokio's 2 MB default in debug builds). Both facts are commented at the runtime builder and
  in `api.rs`'s module doc — **S6/S7 must not "simplify" either one away.**
- **New engine seed, found by a test refusing to lie about itself.** Writing
  `test_post_action_illegal_target_returns_422` against the first castable spell at seed 0
  (`Accorder's Shield`, a `{0}` artifact with no target requirements) returned **200**: the
  engine **accepts a spurious `Player` target on a spell that requires none**, and records it
  on the stack object. The test was rebuilt to drive three deterministic steps to
  `Cast Dispel` ("counter target spell", CR 601.2c), where the target is genuinely refused →
  `Rejected(GameStateError::InvalidTarget)` → 422, with the same params on `PassPriority`
  asserted alongside as the **400** control (`ParamError::UnsupportedParam`, never reaches the
  engine). **The excess-target acceptance is a real engine-side gap, out of scope for
  M11-local, and is FILED as `OOS-M11-5`** (`docs/audits/decision-point-audit.md` §8.1) so the
  next queue re-rank — which enumerates `OOS-*` tokens — actually sees it. Root cause read in
  source: `validate_targets_inner` skips its entire requirement-matching pass when
  `requirements.is_empty()`, an "existence-only" arm added for **aura/bestow** (which declares
  a target while carrying no `TargetRequirement`) and never scoped to it. Zero exposure through
  the bots — `params.rs` only forwards targets a human announced — so it became reachable only
  when S3 gave a human a way to announce targets at all, which is why nine prior batches did
  not see it.
- **Invariant 7 at the HTTP boundary is pinned in both directions.** Omniscient truth is read
  out of band from the session's `GameState`; after excluding the human's own hand and every
  public zone (battlefield, graveyards, command zone per CR 903.6, exile, stack), **20
  distinct other-seat hand card names** remain and the count is asserted **exactly**, so a
  future change cannot quietly empty the set and turn the search into a no-op. Each name is
  searched for in the **raw response body string**, not the parsed `zones.hand` — S4's review
  HIGH ("redaction follows the rendering site, not the zone") applied forward. All seven of
  the human's own names are asserted **present**, so an empty payload fails. Proven by
  mutation, **re-run against the current 16-test module in fix cycle 2**: flipping
  `seat_view` to `Viewer::Omniscient` reddens exactly
  `test_seat_view_over_http_contains_no_other_hand_card_names`, on `"Aggravated Assault"`,
  while the other **fifteen** stay green.
- **Two facts S7 needs.** (1) `mtg_view_model::redact::viewer_may_identify` is `pub(crate)`
  and not re-exported, so a play-server label physically cannot call it — every label goes
  through a `NameIndex` derived from the already-redacted `StateViewModel`, and an unidentified
  id renders `(hidden card)`. **S7's `target_slots` labels must use the same index**, not
  `state.objects()`. (2) `event_view_for` takes **four** params (`ev, state, player_names,
  viewer`), not the plan §3 sketch's three.
- **Known limitations, all deliberate and documented in `tools/play-server/README.md`**: the
  mulligan rebuilds the **whole table** (CR 903.6 makes the command zone public, so a redeal
  is not invisible; and CR 103.5c's per-seat counts cannot be represented) — a per-seat model
  needs each *bot* seat asked, i.e. a new decision channel; `cards_to_bottom` is refused with
  **400** rather than silently discarded, because `handle_keep_hand` checks it against a
  `PlayerState::mulligan_count` a rebuild always leaves at 0; `GET /api/game` calls the
  idempotent `advance()` and consumes `journal_cursor`; `target_slots` / `modes` are empty
  until S7; `needs_x` answers `CastSpell` only; one game per process; and (added by the fix
  cycle) `GameSummary.seed` is the base seed, not the effective one after a mulligan.
- **Second engine/simulator seed, `OOS-M11-6`, found while probing whether `new_game` is
  client-reachably fallible — and it is.** `random_deck` (`crates/simulator/src/deck.rs`)
  applies the CR 903.5c colour-identity filter correctly to the main deck and then **bypasses
  that same filter 37 lines later** when padding to 99 with basics (filter predicate
  `deck.rs:68`, padding loop `deck.rs:105-110`): `basics_for_colors`
  falls back to **Forest** (identity `{Green}`) for a **colourless** commander, so such a deck
  carries ~34 illegal Forests and `validate_deck` — which S2 deliberately routed
  `build_initial_state` through — refuses the whole table. **Measured: 7 failures in a sweep of
  180 `(players, seed)` pairs** (`players: 2, seed: 17` among them), so roughly one
  client-supplied seed in 25 returns a deck-validation failure instead of a game. There are
  **two** Forest fallbacks and the second is dead — the call site's own `if basics.is_empty()`
  arm has a comment saying *"use Wastes (or just any basic)"* and pushes `forest`. **Not a
  one-line fix**: no `wastes.rs` def exists, so it needs either Wastes authored or colourless
  padding drawn from the identity-legal lands already in the pool (prefer the second — no new
  def, no `Complete` flip). **The fuzzer half is CONFIRMED, not suspected** (checked in the
  third audit): `driver.rs` has no deck reference at all, `validate_deck` appears in
  `crates/simulator` only in `setup.rs`, and `bin/fuzzer.rs:296` calls `random_deck` and feeds
  `GameStateBuilder` directly at `:309`+ from the same `all_cards()` pool. So those decks are
  **played** there, not refused — the blast radius is a **silent CR 903.5c deviation in every
  fuzz run that rolls a colourless commander**, not just a play-server 422. The two
  poison-atomicity tests in `tools/play-server` use this bug as their only trigger; closing it
  needs a replacement failure mode (they fail loudly, not vacuously).
- **The no-WebSocket / no-SSE decision is recorded in the crate README with its reasoning**
  (bots act synchronously inside the human's own request, so the server never holds news the
  client is not already waiting on; a second human seat would break that premise; push is
  M10a's problem). `memory/decisions.md` receives it at **S8**, per plan item 8 — deliberately
  **not** written there yet.

**S4 handoff (2026-08-01, `scutemob-165`)**

- **One view-model implementation now feeds both hosts.** `tools/replay-viewer/src/view_model.rs`
  is gone; the file is `crates/view-model/src/lib.rs` (`git mv`, 91% similarity, additive
  changes only). `tools/replay-viewer` is a consumer and **its 15 tests pass unedited**.
  `crates/engine`/`card-types`/`card-defs` diff vs main is **empty**; PROTOCOL 32 / HASH 69
  unmoved. Tests 3,988 → **3,998**.
- **Redaction follows the RENDERING SITE, not the zone — the review's HIGH, and the thing
  most worth carrying into S5-S7.** The first cut redacted `zones.hand`,
  `zones.battlefield` and `zones.exile`, which are the zones CR calls hidden, and stopped.
  Four other sites read `obj.characteristics.name` **raw** — no layer pass, no entitlement
  check — and each can be handed a face-down object: `StackItemView::source_name`,
  `format_target`, `AttackerView::name` (and its planeswalker `target`), `BlockerView::name`.
  A morph creature that attacks *is* on the battlefield, so the battlefield redaction
  "covered" it in the zone sense while `combat.attackers[i].name` printed its name to the
  whole table. `redact_stack` / `redact_combat` now route all four through the
  already-correct `viewer_may_identify`. **S7 populates `ActionOptionView.target_slots` from
  the engine query surface — those labels are a fifth rendering site and must come from the
  seat-redacted view, not from `state.objects()` directly.**
- **"Renders a name" is too narrow a test for a redaction surface — `is_commander` renders a
  boolean and leaks a name.** The re-review's finding, and the seventh site.
  `build_zones_view` derives `PermanentView::is_commander` from the raw `obj.card_id`, and
  CR 903.3 calls the commander designation *"an attribute of the card itself"*, not a
  characteristic — which is precisely why CR 708.2a's face-down override does not touch it
  and why `calculate_characteristics` structurally **cannot**. So a commander cast face down
  for its morph cost comes back with every characteristic correctly blanked and
  `is_commander: true` intact, and since every opponent already knows which card is your
  commander (CR 903.6 — it started in the command zone) that one boolean resolves the
  identity to exactly one card the instant it enters. Now cleared for non-owners; the test
  asserts the omniscient view *does* flag it before asserting the seat view does not.
  `redact.rs`'s module doc now carries the complete site inventory with a disposition for
  each, including the one deliberate non-redaction (`commander_damage_received`, whose inner
  keys are commander names — but a non-zero entry requires that commander to have dealt
  combat damage, at which point CR 903.10a makes the association public in paper too).
- **A single-seat leak scan is a blind leak scan.** Every scan viewed from alice, and alice
  is the one player whose hand card the fixture also puts on the stack, so her own names
  were never needles — which is why the HIGH above passed six whole-document scans. Fixed by
  looping all four seats. With the new redactions disabled the all-seats test fails on
  `seat 2: leaked "Lightning Bolt"` **while all six plan-named tests stay green**.
- **The exhaustive-match gotcha moved with the file.** `stack_kind_info()`
  (`StackObjectKind`) and `format_keyword()` (`KeywordAbility`) now live in
  `crates/view-model/src/lib.rs`. `cargo build --workspace` is still the gate and is now a
  *harder* one: a missed arm breaks a library two binaries depend on, not one binary's
  private module. `memory/gotchas-infra.md` and the session-loaded auto-memory index are
  both updated; several historical docs still cite the replay-viewer path and are stale.
- **The golden snapshot was captured BEFORE the move** (commit `56d44177`), from pristine
  code, then the source file was restored byte-for-byte. That is what makes
  `test_omniscient_view_is_unchanged_for_fixture_state` a regression guard rather than a
  record of whatever the new code happens to do. Compared as `serde_json::Value`, never as
  a string — `StateViewModel` uses `HashMap` and its iteration order is randomized per
  process. It was regenerated exactly once, for the additive `hidden` field; a structural
  diff showed 12 deltas, every one an added `hidden: false`.
- **The leak that mattered was not the one the plan named.** A face-down *battlefield*
  permanent was already safe: `build_zones_view` runs each permanent through
  `calculate_characteristics` and the layer system applies the CR 708.2a override for
  everyone, so the pristine golden already shows `"name": ""`. The face-down *exiled* card
  leaked its printed name, because `objects_in_zone_as_card_views` reads
  `obj.characteristics.name` raw with no layer pass. Both are redacted explicitly so
  Invariant 7 does not silently depend on the layer system continuing to blank the name.
- **A lookup bug can hide inside privacy behaviour — the sharpest thing in this session.**
  `event_view.rs` first rendered a cast spell's name from `SpellCast.stack_object_id`,
  which **never** resolves: `handle_cast_spell` mints `stack_entry_id =
  state.next_object_id()` (`rules/casting.rs:4401`) solely to build the `StackObject` it
  pushes onto `state.stack_objects()` (`:4529`), and that id is never inserted into
  `state.objects()`. Every cast degraded to the name-free fallback "alice casts a spell" —
  never wrong, never present, and **indistinguishable from correct redaction**, which would
  have quietly made S6's event feed useless for the most common action in the game. Fixed
  to `source_object_id` (`:4732`). The three sibling `card_name` call sites were audited the
  same way against their emission sites and were already right (`CardDrawn.new_object_id` →
  `Hand`, `LandPlayed.new_land_id` → `Battlefield`, `CardDiscarded.new_id` → `Graveyard`).
  **Generalisable: in a redacting renderer, a failed id lookup and a deliberate redaction
  produce the same output. Check every id against its emission site, not just its
  entitlement rule.**
- **`event_view_for` takes a 4th parameter** the plan's sketch omits, `player_names:
  &HashMap<PlayerId, String>` — `GameState` carries `PlayerId`s only, so without it every
  line reads `player_2` and the caller must re-render, putting string formatting back
  *outside* the chokepoint. Display names are public. Same deviation class as S3's
  `alt_cost`.
- **Plan §"Hidden-information filtering point" is stale on one premise**: it says
  `GameEvent::private_to()` "does not exist". It does (PB-DP9, `rules/events.rs`), but by
  its own doc it is "a declaration, not an enforcement point" with no consumer, and it is a
  per-*event* verdict that cannot express per-*field* privacy (`CardDrawn` is public; the
  card's identity is not). `event_view.rs` honours it first and then applies per-field
  entitlement, and its module doc says so for M10a.
- **Face-down redaction keys on `obj.owner`, which is conservative rather than strictly
  correct.** CR 708.5a lets a player who *controls* a face-down permanent look at it, so a
  thief is denied a name they are entitled to. Denying too much never leaks; the reverse
  does. Recorded in `redact.rs` for whoever wants the precise version.
- **Escape hatches need the `test-util` feature.** The fixture uses `objects_mut` /
  `players_mut` / `stack_objects_mut` / `combat_mut`, which are `#[cfg(any(test, feature =
  "test-util"))]` per SR-3. They belong in `[dev-dependencies]` only — putting `test-util`
  in `[dependencies]` would break the seal that `cargo build --workspace` enforces.

**S3 handoff (2026-08-01, `scutemob-163`)**

- **The milestone's crux is closed.** The TUI always sent `targets: Vec::new()`, so any
  spell with a `TargetRequirement` was rejected at `casting.rs:3708` — a human literally
  could not cast Lightning Bolt. `test_human_casts_targeted_spell_through_local_game`
  now casts a targeted spell through `LocalGame::submit` and asserts the damage
  **resolved**, picking its target through the new engine query surface end-to-end.
- **Engine half** — new `crates/engine/src/rules/queries.rs` (read-only, 4 fns,
  re-exported from `lib.rs`, **no new public type**). `casting.rs` gains three shared
  helpers extracted verbatim from `handle_cast_spell` (`card_def_target_requirements`,
  `spell_mode_selection`, `per_mode_target_requirements`) so the query and the cast path
  **cannot drift** — that shared extraction, not the query itself, is the load-bearing
  part of plan item 1. `legal_targets_per_slot` delegates one
  `casting::validate_targets_inner` call per candidate, which is what buys
  hexproof/shroud/protection (`casting.rs:6160`) and player-hexproof (`:6114`) for free
  instead of re-deriving them. SR-9a honoured: `tests/rules/queries.rs` **plus** its
  `mod` line in `tests/rules/main.rs`.
- **Signature deviation, argued not assumed:** `spell_target_requirements` takes a 4th
  parameter `alt_cost: Option<AltCostKind>` that the plan's §3 sketch omits.
  `casting_with_overload` (`casting.rs:1163`) and `casting_with_aftermath` (`:533`) are
  **caster-intent** flags derived from the `CastSpell` command, not derivable from state,
  so without it CR 702.96b is unreachable and the named Overload test is unwritable.
  `AltCostKind` is already public → no new public type, wire fingerprint unmoved.
- **A gate-churn trap, avoided.** The first cut checked Overload eligibility by reading
  `KeywordAbility::Overload` from layer-resolved characteristics. That is a *parallel
  re-derivation* of what `casting.rs:1203` establishes with
  `get_overload_cost(...).is_some()`, and it reclassified Overload from SR-5 `Marker` to
  `Handled`, dragging `keyword_registry.rs`, its gate test and
  `docs/sr-5-keyword-catchall-audit.md` along with it. Replaced with the *same call*
  casting makes; the three collateral files reverted. **Lesson: when a new read of a
  keyword forces an SR-5 reclassification, that is a signal you re-derived something
  instead of delegating to it.** (Aftermath genuinely does read its keyword, mirroring
  `casting.rs:533-538`, so `queries.rs` is honestly added to *its* site list.)
- **Simulator half** — new `crates/simulator/src/params.rs` is now the **single**
  `LegalAction` → `Command` mapping table (`random_bot::action_to_command` delegates;
  RNG survives only to fill `attackers`/`blockers`). `hybrid_choices` /
  `phyrexian_life_payments` forwarded **verbatim** from the `LegalAction` (PB-RS2
  precedent — re-deriving is the OOS-RS-2 drift class); an `any_color` `TapForMana` with
  no `chosen_color` is **rejected**, not defaulted to Colorless (PB-EF12, CR 106.1a/b).
  A param announced on an action with no channel for it is rejected rather than silently
  discarded.
- **Bot parity was proven, not asserted.** `mtg-fuzzer --games 50 --seed 424242 --bot
  random` built at pristine and at refactored code: byte-identical per-seed
  Turns/Commands/Winner/Error across all 50 seeds, identical aggregates. Only difference
  is `stack_consistency` violation *line ordering*, which is the known `OOS-M11-3`
  nondeterminism (total violation count identical).
  > **VOIDED as evidence by SIM-3 (`scutemob-177`, 2026-08-02).** Those
  > `stack_consistency` lines were false positives by construction — the check compared
  > `StackObject::id` against the Stack-zone `ObjectId`, two id namespaces CR 400.7
  > guarantees will differ. So "only difference is their line ordering" is a statement
  > about the ordering of noise, and cites `OOS-M11-3` for something that was never
  > evidence of nondeterminism. **The byte-identical Turns/Commands/Winner/Error result
  > stands and is what that bullet's claim actually rests on.**
- **`HumanChoice` is now a struct** (`{ action_index, params }`), not
  `enum HumanChoice::Command(Command)`. `submit` builds the command itself for
  `pending.player`, so a **cross-seat command is structurally unrepresentable** — S1's
  `command_player` runtime guard and its unit test are deleted, exactly as `submit`'s own
  S1 doc comment predicted. The tap-then-cast sequence applies to a **clone** and commits
  only on full success, so a succeeded tap never survives a rejected cast.
- **`OOS-M11-2`'s pool half is CLOSED**: auto-tap now fires only when `params.auto_tap`
  **and** the caster's existing `ManaPool` cannot already cover the cost. The layer-
  resolution half (`mana_solver.rs` reads non-layer-resolved `mana_abilities` at `:35`)
  is **still open** and unowned. `advance()`'s bot-seat auto-tap deliberately still fires
  unconditionally — a bot has no reason to prefer its pool, and touching it would perturb
  the fuzzer parity above.
- **A fixture trap worth carrying into S4-S8:** you cannot pre-fill a player's mana pool
  before `LocalGame::start` and expect it to survive — `start_game` runs through
  Untap/Upkeep and **CR 500.4 empties the pool between steps**. Produce a funded pool
  with a real `TapForMana` submit inside the same step instead.
- Workspace **3,955 → 3,965 / 0** (engine +7, simulator +4, −1 deleted `command_player`
  unit). PROTOCOL **32** / HASH **69** unmoved; diff vs main over
  `crates/engine/src/rules/protocol.rs` and `crates/card-types/` **empty**.
- **S4 is unblocked and stays parallel-safe** with the PB-DX queue: it touches a new
  `crates/view-model` + `tools/replay-viewer`, no engine surface.

**S2 handoff (2026-07-31, `scutemob-161`)**

- Shipped `crates/simulator/src/setup.rs`: `LocalGameConfig` / `DeckSource` / `BotKind` /
  `SetupError` / `build_initial_state` / `redeal`, re-exported from the crate root. One
  `StdRng` seeded from `cfg.seed`, consumed in ascending `PlayerId` order — same seed
  reproduces the same `public_state_hash`. Deck admission runs through the **real**
  `mtg_engine::validate_deck` and refuses on any `DeckViolation` (Architecture Invariant
  9); `start_game`'s `check_all_defs_complete` stays as the independent second line.
  `tools/tui/src/play/app.rs::PlayApp::new` rewired onto it (~55 duplicated lines gone);
  `deck.rs` and `bin/fuzzer.rs` untouched by design. **10 tests** in
  `crates/simulator/tests/setup.rs`; workspace **3,928 → 3,938 / 0**; PROTOCOL 31 / HASH
  68 unmoved; engine + card-types + card-defs diff vs main **empty**.
- **A live Commander bug was found in the lifted logic and FIXED, not seeded.** The old
  TUI setup placed the commander *object* in `ZoneId::Command` but never called
  `GameStateBuilder::player_commander`, so `PlayerState::commander_ids` was **empty** in
  every game it built. That field gates commander tax, the CR 903.9a/704.6d
  command-zone-return SBA, CR 903.10a commander damage, and CR 903.9b's hand/library
  redirects — none of them fired. `mtg-tui`'s play mode had been running non-Commander
  games under a Commander UI. Fixed with two calls to existing public engine API (zero
  engine edits), pinned by `test_setup_registers_commanders_not_just_places_them`.
- **CR cite correction, and it is the reusable lesson.** The session plan's own Session 2
  text cites **CR 103.4** for the seven-card opening hand in items 2 and 7. That is wrong:
  **103.4 is the starting life total** (103.4c = Commander's 40); the seven-card draw and
  the mulligan are both **CR 103.5**, and CR 402.1 restates the draw. Verified against the
  CR via MCP, corrected in six places across `setup.rs` and `tests/setup.rs`. This is the
  *same* stale-cite family as the "CR 103.4b" the PB-DP2 handoff already flagged — the
  plan text was never corrected, so the miscite propagated straight into new code.
  **Anyone working Sessions 3-8 should treat the plan's CR cites as unverified.**
- **`redeal` is a v1 UX path with two honest limitations**, documented in source rather
  than papered over: it rebuilds the whole table, so (a) it re-rolls every seat's
  commander — and the command zone is *public* (CR 903.6), so this is not invisible to
  the other seats; and (b) a single `(seat, mulligan_count)` signature cannot represent a
  partially-decided table, so it discards a hand another seat already kept (CR 103.5:
  "once a player chooses not to take a mulligan, the remaining cards become that player's
  opening hand"). A per-seat mulligan state fixes both and belongs with the Session 5
  play-server pregame flow.
- **Premise honored:** plan §8 R2 was **not** re-filed — `OOS-M11-1` was closed by PB-DP2
  (`scutemob-150`), `handle_take_mulligan` really shuffles, and `redeal` is kept for the
  pregame-UX reason in Q1, not as a correctness workaround.
- **Still open from S1:** `OOS-M11-2` (mana solver ignores the pool, reads
  non-layer-resolved `mana_abilities`) — S3 owns the pool half; the re-rank
  (`scutemob-159`) confirmed its exclusion from the primitive queue. `OOS-M11-3` (fuzzer
  nondeterminism in 150-200+ turn games) untouched.

## Coordinator note (scutemob-186 collect, 2026-08-02)

The adjudication task was conflict-barred from coordination files; its dispositions live in
`docs/audits/mtg-characteristics-recursion-adjudication.md` (§5 queue insertion PB-DX42a/b,
§6 seeds OOS-ADJ-1..7). The v3 queue memo's §4 table has NOT been re-rowed — the next dispatcher
must read adjudication §5 alongside it. OOS-ADJ-3 warns `OOS-DX19-2`'s "613.8b fixpoint" framing
would make a worker build the wrong thing — re-word at dispatch. OOS-ADJ-7 (blood_moon strips
Artifact card type) rides PB-DX27.

## Worker Handoff (PB-DX25b, `scutemob-204`) — a spell you can target is a spell you can retarget

**What was wrong.** `casting.rs::validate_object_satisfies_requirement` opens by resolving the
announced target id through `state.objects` — so the id must name the **CARD** sitting in
`ZoneId::Stack` — and then, for `TargetSpellWithSingleTarget` and
`TargetSpellOrAbilityWithSingleTarget`, looked the stack object up by `so.id == id`, which is a
**stack-entry** id. `handle_cast_spell` mints the two one line apart (`move_object_to_zone(card,
ZoneId::Stack)` then `state.next_object_id()`), both from the one monotone `timestamp_counter`, so
an id lives in exactly one namespace and the comparison **type-checks while being unsatisfiable**.
`is_spell` was always false and `target_count` always 0: `misdirection` and `bolt_bend` are
`Complete`, deck-legal, and could never resolve a legal target. Same defect as `OOS-SIM3-5`, two
functions apart.

**The brief was short by three sites, and a fix obeying it would have been strictly worse than
HEAD.** The dispatch row said "validation-site only, no stored state". But `Effect::ChangeTargets`
— the effect *both* cards use — resolves the same announced id and matches it against stack-entry
ids at three more places. Repairing only the validator produces a cast that passes announcement,
**takes the mana**, and then hits `continue` at resolution: a silent no-op in place of an honest
refusal. `Effect::CopySpellOnStack` is a fourth site (latent — corpus population zero, re-measured
by enumeration) and `Effect::CounterSpell` had open-coded the *correct* rule as a fifth. The
authoritative id space is settled by the offer layer, not by argument:
`queries::legal_targets_per_slot` enumerates object candidates from `state.objects()` alone, so a
card id is the only thing a player or bot can ever announce.

**The fix is structural.** One `state::stack_registry::stack_index_for_announced_target`, beside
PB-DX25's `card_in_stack_zone`, encoding the rule ONCE —
`so.id == announced || (!so.is_copy && card_in_stack_zone(&so.kind) == Some(announced))` — and
consumed by all five sites. Two guards that look incidental and are not:
* **`!so.is_copy` is load-bearing twice.** CR 707.10: `copy.rs` clones the original's `kind`
  wholesale, so a copy's `source_object` names the ORIGINAL's card. Without the guard one card id
  matches the original *and* every copy of it and `position()` silently returns whichever comes
  first. It is also what stops the CR 702.99c cipher-copy exile leak PB-DX25 documented.
* **`is_spell` is KEPT although it is now production-unreachable.** After the repair the helper can
  return a non-spell only via the direct-id clause, which needs an id that is simultaneously a
  `state.objects` row and a stack-entry id — unreachable. So the two requirements are
  behaviourally identical on the real path today. That is the visible shadow of `OOS-DX25b-1`, not
  a new defect, and deleting the guard becomes CR-wrong the day that seed closes. The collapsed-id
  fixture is deliberately kept for the same reason: it is now the only configuration that isolates
  the guard, and its doc says so.

**`stack_card_of` stays uncoupled.** The simulator's exhaustive re-implementation
(`crates/simulator/src/invariants.rs`) is untouched — `check_stack_consistency` exists to catch the
engine getting this classification wrong, and a verifier that reads the engine's own answer goes
silent on exactly the defect it was written for. Zero simulator lines.

**The tests were green while testing a fiction.** `casting.rs`'s `make_test_stack_spell` built
`StackObject { id, kind: Spell { source_object: id } }` — collapsing the two id spaces into a
configuration **no real cast can produce**. Three test files carried that fixture (the two in-src
`casting.rs` tests, `pb_ef11_spell_single_target.rs`, and `pb_ef11`'s Misdirection probe, which
announced a stack-entry id straight into `execute_effect`). All repaired to mint distinct ids, each
proven to discriminate by executed mutation. **`tests/rules/copy_redirect.rs` still has eight more
of the same shape** — including `test_bolt_bend_redirects_single_target_spell`, named after a card
this batch repaired and proving nothing about it. Not repaired (no coverage hole: the real probes
catch the regression), but now disclosed rather than left as a trap.

**Three things worth carrying forward.**
1. **The review's only HIGH was a plan deliverable the implement phase silently dropped** — plan §8
   R2 option (iii) required a wrong-way-round probe for the CR 115.7a redirect, and the execution
   notes then recorded that "the plan scoped this as future work". The plan scoped the **fix** as
   future work and the **probe** as this batch's. Both the probe and the correction shipped.
2. **The reviewer defeated the new R5 gate three ways** — clause-order reversal, a preceding
   statement's `;` landing inside its 150-byte backward window, and a brand-new bare `so.id ==`
   site carrying no `card_in_stack_zone` at all so R5 never looks. The first two are now caught; the
   third is a **permanent structural residual** and is stated as such in the gate's own doc. A gate
   whose doc overclaims its reach is this batch's own subject matter.
3. **The census is complete, and that was verified by the inverse method**, not by agreeing with
   the plan: every `stack_objects` id-comparison across the 18 engine files that mention them,
   classified by provenance. Exactly five raw `so.id ==` comparisons survive outside
   `stack_registry.rs`, every one correctly classified. **No sixth site.** Nothing delegates into
   the `ChangeTargets`/`CopySpellOnStack` arms the way `CounterUnlessPays` delegates into
   `CounterSpell`, so R4's two-arm scope is not blind in the PB-DX25 way.

**Numbers.** Tests **4,452 → 4,469 (+17)**, residual empty, `--workspace --no-fail-fast` to a file.
PROTOCOL **35** / HASH **73** gate-executed and unmoved (fingerprint gate too, not just the version
sentinel). Coverage unmoved **1,133/1,803 = 62.8%**, proven by regeneration; all four card-def edits
comment-only, verified per-line. `crates/simulator/`, `crates/view-model/`, `crates/card-types/`,
`tools/` diffs all empty. Review 1 HIGH / 5 MEDIUM / 6 LOW, all 12 taken.

**Two coordinator scope calls, recorded so they are not re-litigated.** (1) `bolt_bend` **stays
`Complete`** — `completeness` describes the def's fidelity to the printed card, and the def is
faithful; the gap is engine-layer (`OOS-DX20-10` precedent). A demotion would also redden
`pb_dx32_fuzz_output.rs`'s `CORPUS_COMPLETE = 1133` pin and, per `OOS-CARDS2-3`, re-roll every
recorded fuzz seed across five shipped flows — out of all proportion to the finding. (2)
`deflecting_swat`'s requirement was **not widened**; its false comment was corrected in place and
the mismatch filed as `OOS-DX25b-5`.

**Seeds.** `OOS-DX25-3` **CLOSED** — its registry row now carries four corrections to its own
claims (wrong function name; "validation-site only" wrong by three sites; the in-src tests are not
merely negative and fail for a fixture reason, not a rejection reason; `untimely_malfunction`'s
mode 1 fails through the flat/pooled `mode_targets: None` scheme, a more fundamental mechanism than
mode ambiguity). Filed **OOS-DX25b-1..5**. **`OOS-DX25b-3` is LIVE on the same two `Complete`
defs**: this batch is what makes CR 115.7a's unchecked object-target redirect reachable, so a
Misdirected "destroy target creature" now destroys the lowest-`ObjectId` battlefield object —
routinely a basic land — reachable by a human in the browser *and* by the bots, since
`simulator/targeting.rs` routes through the same query. Pinned wrong-way-round for the successor.

**Durable lesson.** *A fixture that collapses two id spaces makes a test green by removing the only
condition under which the code is wrong.* Every test guarding this defect passed, for four months,
because each one hand-built the one state a real cast can never reach. And PB-DX25's enumeration
lesson recurred a third time: the brief, the plan, and the batch's own execution notes were each
short about a different thing.

Full record: `memory/primitives/pb-plan-DX25b.md`, `pb-review-DX25b.md`, and
`pb-DX25b-execution-notes.md` (measurements, the 16-row revert matrix, and §9's corrections to the
plan found by execution).

## Worker Handoff (PB-DX25, `scutemob-203`) — a countered spell is countered, whichever shape it arrived in

**What was wrong.** `Effect::CounterSpell` (`effects/mod.rs`) decided *"does this stack object own
a card in `ZoneId::Stack`?"* by matching the **variant name**. Its `position()` lookup matched
`so.id == id` (the Ward path, CR 702.21a) or `StackObjectKind::Spell { source_object } == id` (the
traditional counter, which targets the **card**) — and nothing else. It then `remove(pos)`ed the
entry **before** matching on the kind, and the match had a `Spell` arm, a combined
`ActivatedAbility | TriggeredAbility` arm, and a `_ =>` catch-all that did nothing. No `is_copy`
check anywhere.

**What the seed and the queue row both got backwards, and the correction is the interesting part.**
`OOS-SIM3-5` ranked (a) — countering a `MutatingCreatureSpell` strands its card — as the live
defect, with (c) as a rider. It is the other way round, and more than that:

* **(a) was never independently reachable.** Ward needs a `GameEvent::PermanentTargeted` naming the
  mutate spell's stack entry, and that event is emitted only for `spell_targets`. A mutate cast's
  target rides in `AdditionalCost::Mutate` and **never enters `spell_targets`** (filed as
  `OOS-DX25-1`), and the SR-36 enumeration measured **0** `Complete` mutate defs declaring a
  spell-level target requirement (roster M3 = 0). So **(a) is what fixing (c) ALONE would have
  created** — a permanent `ZoneId::Stack` leak in place of a silent no-op, reported by
  `stack_consistency` at every subsequent checkpoint for the rest of the game. **A "just fix the
  `position()` lookup" change was strictly worse than HEAD.** This is the single most important
  sequencing fact in the batch and it is why (c) and (a) landed in one commit.
* **(b) is unreachable three ways, not the v3 memo's one.** The ordering argument (`position()`
  returns the lowest index; `copy.rs` pushes the copy above the original), *plus*
  `resolve_effect_target_list_indexed` dropping a dead `DeclaredTarget` — so the window is **empty**,
  not narrow, even under CR 608.2b re-validation — *plus* nothing aiming a counter at a copy at all:
  a copy's stack-entry id is not a `state.objects` key, so `TargetSpell` validation refuses it
  **always**, not merely once the original is gone.
* **(c) is worse than "a silent no-op" sounds.** `TargetSpell` validation resolves the announced id
  through `state.objects` and requires `zone == ZoneId::Stack` — and a mutate spell's card really is
  there. So the engine **offers the target, validates it, takes the mana, and does nothing.**
  Measured exposure: **66** live-wrong pairs.

**What shipped, and why it is structural rather than three patches.** A new engine-side
`crates/engine/src/state/stack_registry.rs::card_in_stack_zone(&StackObjectKind) -> Option<ObjectId>`
— exhaustive over all **27** variants with **no wildcard arm**, `Some` for `Spell` and
`MutatingCreatureSpell` only. Both counter paths consume it: `Effect::CounterSpell` and
`resolution.rs::counter_stack_object`. Adding a 28th card-carrying variant is a **compile error**
until someone classifies it — the same forcing function SR-5 applies to `KeywordAbility`, and it
lives in the engine rather than `card-types` on the `keyword_registry` precedent (which also keeps
the 1,798 card defs `Fresh`). The zone-move moved **out** of the per-kind match and is skipped
entirely when `stack_obj.is_copy` (CR 707.10 — a copy is a spell with no card; `copy.rs` clones the
original's `kind` wholesale, so moving `source_object` would put **someone else's** spell in the
graveyard). A countered copy emits `SpellCountered` with `stack_object_id == source_object_id ==`
its own stack-entry id: CR 707.10 makes the event owed and forbids naming a card id, and the
already-shipped `event_view.rs` fallback renders *"<player>'s spell is countered"* with no renderer
and no wire change.

**The decision worth carrying forward: the verifier was deliberately NOT unified with the thing it
verifies.** The simulator's `invariants::stack_card_of` answers the same question and was the model
for the fix — but it is `check_stack_consistency`'s classification, and that check exists
*specifically* to catch the engine getting this wrong. If the verifier read the engine's own answer
back, a wrong `Some`/`None` would make the check **agree with the defect and go silent**, in exactly
the case it was written for. So there are two implementations on purpose. What keeps them honest:
both are exhaustive with **no wildcard**, so a new variant is a compile error in **both crates
independently** (coverage is machine-synced); the *classification* is deliberately unsynced, so a
disagreement is loud by construction; and one behavioural probe
(`crates/simulator/tests/pb_dx25_counter_on_mutate_is_consistent.rs`) proves they agree on the case
that matters by running a real counter-on-mutate game, rather than by sharing code. Doc
cross-references at both functions say all of this, so the next reader does not "simplify" it.
**Contrast with PB-DX20 deliberately**: there, two *consumers* of one derivation were unified,
because disagreement between them was the defect. A *verifier* is not a consumer.

**The `/review` cycle found 0 HIGH / 6 MEDIUM / 3 LOW + 7 folded notes, all taken — and its three
sharpest findings were this batch's own failure mode recurring inside the batch.**

1. **The plan's "FOUR sites that classify a stack object's card/spell-ness" census was short by
   two, and one of the two was wrong in the same direction as the defect being fixed.**
   `abilities.rs:6736`'s `targeting_is_spell` matched `Spell` alone, gating every CR 601.2c
   "becomes the target of a **spell**" trigger — while `casting.rs:6507`, answering the *identical*
   question one function over, pairs both kinds. Two implementations of "is this a spell",
   disagreeing: verbatim the argument the census was written to make. `casting.rs:7126`'s
   `has_split_second_on_stack` was the sixth, and it is `card_in_stack_zone`'s exact question left
   unconverted. Both fixed; the census corrected to six in the plan and the notes.
2. **The SR-36 roster's `P = 48` was an undercount, and the coordinator had already written it into
   the queue memo.** The enumeration walked `Effect::CounterSpell` and was structurally blind to
   **`Effect::CounterUnlessPays`**, which `effects/mod.rs` delegates *straight into the repaired
   arm* — so `mana_leak`, `mana_tithe` and `make_disappear` (all `Complete`, all carrying
   `TargetSpellWithFilter(TargetFilter::default())`, which is unrestricted field-by-field because
   `TargetController::default() == Any`) were equally live-wrong and equally invisible. It also
   missed counters on activated/triggered abilities and on back faces. Re-measured: **C1 29, C2 24,
   C3 11, P 66.** The first re-measure replaced a grep-derived wrong number (144) with a
   differently wrong one, **with the authority of an SR-36 enumeration behind it**.
3. **T6's advertised non-vacuity did not exist.** `assert_eq!(variants.len(), 27)` compared a
   hand-written `vec!` against itself; a 28th variant classified in the registry would leave it
   green. The property actually lives in `g1_scan_is_not_vacuous`, which counts arms **in source**
   — a different subject, in a different crate target. This is the PB-DX24 durable lesson recurring
   in the batch dispatched immediately after it.

Also from the cycle: a doc cross-reference pointed at a comment **that was never written**
(`stack_registry.rs` → `casting.rs:6503`, the MR-M11-12 class — written now); `SpellCountered`'s
type-level doc was false for two of the three shapes that now emit it; a new **G4** gate was added
over `counter_stack_object`, because criterion 6232's "single classification, **both** paths" half
had been resting on argument plus one test rather than on a machine; and G2 was hardened against the
`use StackObjectKind as K` alias form **the registry itself uses**. One note was **declined with a
reason**: a T4 sub-case for the `unwrap_or(controller)` owner fallback would be misleading, because
`move_object_to_zone` does the identical lookup on the same id moments later, so whenever the
fallback could fire the move has already failed (CR 400.7 fizzle) and the fallback value is never
observable — the plan's claim was narrowed instead of a synthetic probe being written.

**An unclaimed positive the reviewer found.** The `is_copy` guard also closes a CR 702.99c hole
nobody was looking for: `resolution.rs:5418-5430` builds a cipher copy as
`Spell { source_object: <a card in EXILE> }` with `is_copy = true`, so countering one through the
Ward clause would previously have pulled the encoded card **out of exile** into a graveyard.

**Two coordinator-side corrections, recorded because both are the batch's own subject.** A shipped
doc cited **PB-DX9** — an *unshipped* queue entry — as the precedent for keeping
`counter_stack_object`; `git log -S` over the quoted sentence found the real one, **PB-DP9**
(`f33aabe2`, `scutemob-157`). And **CR 701.5 is `Cast`, not `Counter`** — `Counter` is **CR 701.6**,
and the widely-cited "CR 701.5g" does not exist; ~337 sites tree-wide carry the stale number, filed
as `OOS-DX25-6` and corrected only inside the region this batch already edited.

**Measurements.** Tests **4,452 / 0 / 5** (+17 over the **4,435** pre-edit baseline measured on this
branch before any edit). PROTOCOL **35** / HASH **73** gate-executed and unmoved — re-executed after
the `abilities.rs`/`casting.rs` edits, along with the SR-5 `keyword_registry` gate (9/0, unmoved).
Coverage unmoved **1,133/1,803 = 62.8%**, proven by **regeneration** rather than by the empty
card-defs diff the plan would have accepted (criterion 6233 asks for regeneration specifically).
`clippy -D warnings`, `fmt --check` and `tools/check-defs-fmt.sh` all clean. Benches within noise
(`full_turn_4p` 214-215 µs). SR-6 scope empty: **0 lines** in `crates/card-defs/`,
`crates/card-types/`, `crates/view-model/` or `tools/`. Every new probe and gate proven
discriminating by **executing** its revert; the matrix and every failure text are in
`memory/primitives/pb-DX25-execution-notes.md`.

**Seeds.** **OOS-SIM3-5 CLOSED**, its row carrying four corrections to its own claims rather than
having them deleted. Filed **OOS-DX25-1..6**. **Read `OOS-DX25-3` before the next batch**: it is
**LIVE on two `Complete`, deck-legal defs** — `misdirection` and `bolt_bend` can never resolve a
legal target, because `validate_target_requirement` keys the announced id on `state.objects` (the
**card**) and then compares it to `so.id` (a **stack-entry** id), two namespaces minted from one
monotone counter that therefore never intersect. Its in-src tests are **negative** tests and pass
**vacuously**, because the requirement refuses everything. It is the same id-space confusion as this
batch's subject, one function over, and it was found by accident.

**Durable lesson.** *An enumeration is only as wide as the variant list it walks, and an exhaustive
match proves nothing about the callers that never ask it.* The batch built a classification that
cannot silently miss a **variant** — and then shipped a roster that silently missed a **delegating
effect**, a gate whose message described a class it could not see, and two live call sites that
never consulted the classification at all. Exhaustiveness is a property of a match, not of a
program.

## Worker Handoff (PB-DX24, `scutemob-202`) — a zone-scoped ability finally functions in its zone

**What was wrong.** `AbilityDefinition::Triggered` carries a `trigger_zone` field.
`TriggeredAbilityDef` — the runtime shape `build_face_ability_vectors` lowers into — has no home
for it, so 33 of the lowering's 34 trigger arms swallowed it. `nether_traitor` pairs
`WheneverCreatureDies` with `trigger_zone: Some(Graveyard)` and is `Completeness::Complete`, so a
deck-legal card had its graveyard ability **installed on the battlefield object** and functioned
from exactly the wrong zone. CR **113.6m** is the load-bearing rule: the ability's effect moves the
card *out* of the graveyard and its trigger condition does not put it *there*, so it functions only
there.

### Read this before trusting the brief, the queue row, or stage 0

**1. The brief was short by a whole half, and the narrow fix alone would have shipped a card that
fires nowhere.** Both the task brief and the v3 queue row read this as "one line, wire-neutral" —
delete the lowering, done. But `collect_graveyard_carddef_triggers` (`abilities.rs:7112`) had
**one** `fires` arm, `PermanentEnteredBattlefield`, written for Bloodghast's landfall. A
`WheneverCreatureDies` graveyard trigger had **no dispatch path at all**. Suppressing the
battlefield lowering without adding the dispatch would have turned a wrong-zone card into a
silent no-op — and a green test suite would have said nothing, because nothing tested it.
Criterion 6205 demanded *both directions*, which is what forced the discovery.

**2. Stage 0's own arm count was wrong, and the fix cycle caught it twice.** Stage 0 said 40 arms;
stage 3 re-measured **34**; the reviewer independently derived 34. The lowering's doc table now
carries the **counting rule** (36 `for ability in abilities` loops minus the 2 mana/activated
loops), so the next reader re-derives it instead of trusting it. Three published numbers for one
census is the same failure mode PB-DX19 recorded — publish the rule, not the number.

**3. The loss was never uniform, and that is why a single-arm fix would have been wrong.** The
`WheneverPermanentEntersBattlefield` arm had always skipped. The repair does not add 33 more
`continue`s — it extracts the whole trigger-lowering region into `build_face_triggered_abilities`,
whose input is filtered **once** at its single call site through `lowers_onto_the_battlefield` (an
exhaustive match on `TriggerZone`, no wildcard), and **deletes** the old per-arm guard so there is
one mechanism rather than two. A 41st arm cannot re-swallow the field; two source gates fail if one
tries, and the comment-stripping in the structural gate was itself proven load-bearing by executing
both variants (PB-DX32's M8 lesson, applied rather than cited).

### What shipped

- **The lowering half** (`testing/replay_harness.rs`): extraction + filter + the corrected lossy
  table row, which now reads *honoured* rather than *dropped*.
- **The dispatch half** (`rules/abilities.rs`): a `WheneverCreatureDies` arm in
  `collect_graveyard_carddef_triggers` mirroring the battlefield `AnyCreatureDies` arm clause for
  clause — CR 108.4a (a graveyard card has no controller; its **owner** stands in), CR 400.7
  (`exclude_self` compares the **graveyard** id, because the battlefield id can never match one and
  a battlefield-only comparison fails **open, silently**), CR 111.7, CR 603.10a/613.1d — plus a
  CR 603.10a look-back guard (`arrived_in_graveyard_this_batch`) applied to **this arm only**. The
  ETB arm must not gain it: CR 603.10a's list does not include ETB triggers, so Bloodghast arriving
  in the graveyard alongside a land entering still triggers. That asymmetry is written at the guard,
  because it is the one place a future reader will be tempted to "unify" the two arms and be wrong.
- **OOS-DX1-4**: six queue sites moved to `def.effective_abilities(<source>.is_transformed)`; Q5
  re-scoped comment-only.

### The thing worth knowing about OOS-DX1-4's closure

**Its "6 latent queue sites" was right for the wrong reason.** The SR-36 enumeration of
`all_cards()` — never a grep — measured **0** corpus defs carrying any of the seven Q-shapes on a
back face (1,803 defs, 15 with a `back_face`). So all seven are latent, live exposure is zero, and
**every probe is a synthetic `back_face` fixture**. The repair is structural, and the closure says
so rather than implying a live repair.

**Q5 is the interesting one, and both the plan and the review got its rule wrong.** The plan cited
CR 712.2 (which is about DFC face *symbols*). The reviewer corrected it to CR **712.16** and noted
CR **712.15** makes the site reachable after all — but stopped there, so it could only conclude
"unreachable in practice, by engine discipline". CR **712.15a** settles it properly: *"if it's
turned face up, it will have its **front** face up"* — so the one DFC that can reach this site
does so on its front face **by rule**, and reading `def.abilities` there is **CR-correct**, not an
unreachable-case accident. Verified against the rules MCP during the fix cycle.

### The review cycle: 0 HIGH / 6 MEDIUM / 7 LOW — all 13 taken, and two were the coordinator's

The reviewer ran **without a shell** — every finding was derived by reading source. Each was
verified by execution before being applied; none turned out wrong.

- **The two that were mine, not the runner's.** (1) `OOS-DX24-1` asserted "live-wrong on 2
  `Complete` defs" — false: `teysa_karlov` and `drivnod_carnage_dominus` are both
  `Completeness::partial`, so `validate_deck` rejects them. (2) The same row framed the doubler
  defect as *"a NEW instance introduced by this batch"* — wrong about the class: the ETB doubler
  arms have had a graveyard-sourced pairing since PB-35 (Bloodghast). Re-measuring past what the
  review concluded gave a third answer, now in the row: **no pairing is deck-legal on both halves
  in either direction**, so exposure is **zero deck-legal pairings** — not "2 `Complete` defs", and
  not the review's "the deck-legal instance predates this batch" either. The fix is also *smaller*
  than first written: one source-zone conjunct above the `match` covers all four
  `TriggerDoublerFilter` arms, not "every death doubler".
- **A gate that was green while the invariant it claimed to pin was already violated.**
  `test_dx24_is_transformed_true_assignment_has_exactly_one_site` matched only a literal
  `is_transformed = true`, but `face.rs:97` writes a **computed** bool and is how
  `Command::Transform` sets it — the batch's *own* Q3/Q4/Q6 probes assert exactly that. Replaced
  with a runtime probe of the real invariant, and the reviewer's claim ("delete `face.rs:67-69` and
  every PB-DX24 test still passes") was checked by **doing it**: the new probe reddens, the old one
  did not.
- **A plan risk that was never discharged.** Plan §10 risk #2 (look-back slice granularity) was
  silently skipped. Now measured per caller: `sba.rs:97` exact, `resolution.rs:8142` coarse (a whole
  resolution's events), `combat.rs`/`engine.rs` **unaudited and stated as such**. Filed
  `OOS-DX24-7`.

### Numbers

- Tests **4,413 → 4,435 / 0 / 5** (+22). Baseline measured on-branch **before any edit**; final run
  `--workspace --no-fail-fast` to a file, residual list empty.
- **PROTOCOL 35 / HASH 73 gate-executed and unmoved** — no `TriggeredAbilityDef` field, no
  `Command`/`GameEvent`/`Effect` variant. `core keyword_registry` green (run, not reasoned about:
  PB-DX20 and PB-DX23 were each caught by that gate).
- Coverage **unmoved at 1,133 / 1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py`
  to an identical body — *not* by an empty card-defs diff, since this batch mandates a comment-only
  def edit. `tools/check-defs-fmt.sh` clean over 1,803 defs (SR-35).
- Benches within noise: `full_turn_4p` 221.5–223.5 µs, `sba_check` 14.92–14.99 µs,
  `priority_cycle_4p` 24.60–24.83 µs.
- Scope: **0 lines** in `crates/simulator/`, `tools/`, `crates/card-types/`; `crates/card-defs/` is
  one comment-only file.

### Seeds

**CLOSED**: `OOS-DX1-3`, `OOS-DX1-4` — both rows also carry corrections to their **own** original
claims ("latent" was false; "all 34 sites" was wrong twice over).
**FILED**: `OOS-DX24-1..9`. The two to read first:

- **`OOS-DX24-1`** — `doubler_applies_to_trigger` is source-blind in **all four** filter arms.
  Latent (no deck-legal pairing), deferred on the plan's own risk #3, fix is one conjunct.
- **`OOS-DX24-9`** — **LIVE on a `Complete` def.** CR 118.12 makes an optional cost a player
  decision; `MayPayThenEffect` auto-pays. The *class* is the pre-existing DP-19 shape, but this
  batch is what makes `nether_traitor`'s instance reachable at all, so it is live **as of this
  batch**. This is also why the T3 probe had to be re-worded: it cited CR 118.12 while asserting
  pay-when-able — citing the rule the engine deviates from as though it implemented it.

### Durable lesson

**A guard, a gate and a claim each have a subject, and "it passes" only tells you about the subject
it actually has.** Three of this batch's findings are one shape: a gate that scanned for a literal
assignment while the real write was computed; a look-back set whose granularity is decided by
whichever caller hands it a slice; and a seed row whose severity tag named a completeness marker
nobody had read. Each was *true about what it examined* and *wrong about what it was taken to
mean*. The batch's own fix has the same shape and is the reason it is trustworthy: the filter is at
the **call site**, so the thing it must be true of is the one thing it can see.

### For the collector

`memory/primitives/pb-plan-DX24.md` (plan), `pb-DX24-stage0.md` (re-verified premise),
`pb-DX24-execution-notes.md` (measurements, revert matrix, per-caller granularity, bench numbers),
`pb-review-DX24.md` (the review, incl. its "what I checked and found correct" section — read it
before re-auditing anything). Next queue row: **PB-DX25** (v3 rank 7).

## Worker Handoff (PB-DX23, `scutemob-201`) — dredge becomes answerable, by anyone

**What was wrong.** `grep -rn "ChooseDredge" crates/simulator/src/ tools/` returned **zero**: the
engine had `Command::ChooseDredge` and `GameEvent::DredgeChoiceRequired` and a gated handler, and
**nothing could reach any of it**. No `LegalAction::ChooseDredge` existed, so neither a bot nor
the human browser seat could answer a dredge offer.

**The consequence is not a lost option, it is a permanent draw-cadence corruption**, and the
probe measured it rather than arguing it. On a real 2-player `LocalGame`, both bot seats, no state
pokes, `golgari_grave_troll` in `p1`'s graveyard, six turns: **two** `DredgeChoiceRequired` events
fired, **one** card was drawn where **two** were owed, and **one** `PendingDraw` survived to the
halt. Each turn the draw step defers and the *next* turn's draw discharges the stale entry before
deferring the current one — forever one behind, off a library that has had a full turn cycle to be
reordered.

### What shipped

**One derivation, two consumers.** `rules::queries::dredge_options(state, player) ->
Vec<(ObjectId, u32)>` (CR 702.52a/b, sorted by `ObjectId`) is now the only dredge-eligibility scan;
`check_would_draw_replacement` calls it instead of keeping its own copy, and the offer layer
consumes the same function. Re-deriving it in `crates/simulator` would have been the `OOS-RS-2`
drift class. **The SR-5 keyword registry caught what the brief missed** — `queries.rs` is a
`Dredge` handling site and the gate failed until it was declared.

**`LegalAction::ChooseDredge { card: Option<ObjectId>, mill: u32 }`**, emitted as an ORDINARY
priority-window action, mapped in `params.rs`, scored in `heuristic_bot`, labelled in
`view.rs`. Bot and human channels both live.

**`OOS-DX2-2`: the tail of a multi-draw is a DIFFERENT draw.** `perform_remaining_draws`'
hard-coded `offer_dredge: false` is now a parameter, and `resolve_declined_pending_draw` gained
`tail_offers_dredge`.

### Three things worth reading before trusting anything here

**1. The brief's two-site framing was short by one, and the naive flip would have shipped a new
bug.** There are **three** resume sites passing `offer_dredge: false`, not two —
`resolve_pending_draw`'s CR 616.1f re-check is a same-draw site and must stay `false`. More
importantly, an *unconditional* `true` at the tail makes the REOPENED `OOS-DX2-3` **live and
reachable from the corpus's only dredge card**: `perform_one_draw`'s implicit stale-entry
discharge would run a tail that pushes a dredge entry, then control returns to the outer call
which pushes its own — two dredge-originated entries for one player, breaking the one invariant
the discharge does establish. The flag is threaded so that discharge alone passes `false`, and
`test_dx23_implicit_discharge_does_not_mint_a_second_dredge_entry` pins the exact trace.

**Why PB-DP5 §3.3 does not extend to the tail** (this is acceptance criterion 3, and the
distinction is load-bearing): §3.3 argues for `false` because re-offering dredge mid-chain would
restart a CR 616.1 application the player already began *on the same draw event*. That is a claim
about ONE draw. CR 121.2 makes "draw three" three separate draws, and CR 614.11a / 121.6b say the
replacement completes and *then* the sequence resumes — so each resumed draw is its own fresh
"would draw" event.

**2. The brief's UI prescription was one layer off — the PB-DX20 pattern, again.** It asked for
the play-server's blocking-decision UI. The choice lives in the `LegalAction` itself (the
`PayEcho` shape), so the human channel needed **no** `AnswerShapeView` variant, **no**
`ActionParams`/DTO field, **no** picker, and **zero frontend production lines**. Routing it
through the blocking-decision UI would have meant a fourth `BlockingDecision` variant — CR-wrong
(CR 702.52a is "you **may** instead", and the engine deliberately never blocks) and a HASH bump
for an optional decision. The reviewer adjudicated the divergence ACCEPTABLE. T5.1 proves the
human channel end to end over the real router with a NON-DEFAULT answer, and asserts the option
carries no `decision` key — that assertion *is* the pin on this divergence.

**3. The review found the batch's own overclaim, and it was this batch's own failure mode.** The
plan's suppression rule (no offer when nothing is dredge-eligible) was documented as removing the
decline-forever loop **"structurally"**. It does not: the guard is keyed on the GRAVEYARD, while
the entry `handle_choose_dredge` answers is chosen FIFO, and `PendingDraw` carries no origin
discriminator. With a `NeedsChoice`-origin entry queued ahead of an eligible dredge card the
provider offered, the engine answered the wrong entry, the decline re-deferred, and a bot below
the mill margin declined forever. **That is the same shape of claim `OOS-DX2-3` was wrongly closed
on — made inside the batch dispatched to avoid repeating it.** Fixed with a third conjunct that
asks the engine's own question (would declining THIS FIFO entry discharge it?), with its limits
stated: it is a conservative approximation, exact only at queue depth ≤ 1, which is every
reachable case today. `OOS-DX23-8`.

**The reviewer's first suggested fix was declined on precedent**: a `RepeatKey::ChooseDredge` cap
in `heuristic_bot`. PB-DX21 *deleted* exactly that shape — a bot-side repeat cap masking an offer
the provider should never have made. The offer layer is where a bad offer dies (SR-38).

### Numbers

Baseline **4,398 / 0 / 5** re-measured on this branch at `e490153b` before any edit → **4,413 / 0
/ 5**, residual list empty. +15 reconciles exactly: 1 mandatory probe, 7 engine, 6 simulator, 1
play-server. **PROTOCOL 35 / HASH 73 gate-executed and unmoved** — no state field, no wire type.
Coverage unmoved **1,133/1,803 = 62.8%**, card-def diff comment-only, `check-defs-fmt.sh` run
(SR-35). play-server 80/0. The golden dredge script is byte-unchanged. Every recorded fuzz ratchet
was checked explicitly and **none moved** — diagnosed, not assumed: none of the pinned seeds' deals
reaches a dredge offer.

### Seeds

**CLOSED**: `OOS-DX2-5`, `OOS-DX2-2`. **RECORDED, not closed**: `OOS-DX2-7` — now an AUTO-CHOSEN
row in `docs/audits/decision-point-audit.md` §3.1, NON-DSL, 1 `Complete` def reachable, invisible
to `decision_gate` **by construction** (the walk enumerates card-def `Effect`/`Condition` DSL
variants; dredge is a `KeywordAbility`), a fresh `OOS-DP10-9` instance. This batch made the offer
*answerable*; it did not make the auto-discharge stop being an engine-made decision.
**STAYS REOPENED**: `OOS-DX2-3` — not re-closed, and not on a structural argument; the protected
pin is byte-unedited. `OOS-DP5-2` unchanged: answerable, not bounded.

**Filed**: `OOS-DX23-1` (a non-priority-holder's offer is deferred to their next priority window —
CR 117.3d makes that a deferral, never a loss, but the moment is the engine's), `-2`
(`NeedsChoice`-origin entries), `-3` (the TUI has no dredge channel — it hand-builds commands and
never routes through `params.rs`; same family as `OOS-UI2-5`/`OOS-DX6-5`), `-4` (bot dredge policy
is survival-only, no value evaluation), `-6`, `-7` (both method seeds, below), `-8` (the S1
residual). `-5` deliberately NOT filed: it was conditional on a ratchet moving and none did.

### Two method seeds, both found by executing rather than reasoning

**`OOS-DX23-6`** — `cargo build --workspace` does **not** compile test targets, so this project's
standing "the compiler points at every exhaustive match" assumption is false for matches living in
tests. `local_game_playthrough.rs::kind_of` was caught only by `clippy --all-targets`.

**`OOS-DX23-7`** — the doc-reattachment trap, **second occurrence** in this codebase. A doc comment
attaches to the item that follows it, so inserting a function between an existing doc block and its
function silently reassigns the doc. `attack_tax_total` nearly lost its entire doc, including its
load-bearing PB-DX6 "`None` does NOT always mean free" warning. `perform_remaining_draws` carries a
note recording the same trap from PB-DX2's fix cycle.

### Durable lesson

**A guard keyed on one thing cannot police a decision keyed on another.** The suppression rule
asked about the graveyard; the engine answers FIFO. Both were correct about their own subject and
the pair was wrong — and the doc that said otherwise was written by the same reasoning that
produced the guard, which is why only an adversarial read caught it.

### For the collector

Review: `memory/primitives/pb-review-DX23.md` (0 HIGH / 4 MEDIUM / 9 LOW, all 13 taken). Plan:
`memory/primitives/pb-plan-DX23.md`. Every measurement and the full revert matrix:
`memory/primitives/pb-DX23-execution-notes.md`. **Known gap, stated rather than papered over**: no
browser-level (headless Chromium) pass was run; T5.1 covers the same path in-process over the real
router.

## Worker Handoff (PB-DX21, `scutemob-200`) — declaring attackers is once per combat

**What was wrong.** CR 508.1 makes declaring attackers a turn-based action performed **once** per
declare-attackers step. `combat.rs::handle_declare_attackers` guarded on step, active player,
priority holder and per-attacker legality — and on nothing else — and initialised `CombatState`
only when `None`, so a second `Command::DeclareAttackers` in the same combat reran the whole body.

**Three consequences, not the seed's one.** The seed row said a re-declaration "overwrites
`combat.attackers`". It does not: `:745` **inserts** into an `OrdMap`, so declarations *accumulate*
and a repeated same-id entry overwrites only **that creature's attack target, mid-combat**. (2)
`:795-806` pushes a fresh `AttackersDeclared` and re-runs `check_triggers` +
`flush_pending_triggers`, so **every attack trigger re-fires per declaration** — the one a human
hits first. (3) `:759` *assigns* `attackers_declared_this_turn`, clobbering the raid count read by
`Condition::YouAttackedWithNOrMore` on `windbrisk_heights` and `legions_landing`. **A fourth was
found during the batch**: `:818` resets `state.turn.players_passed` on every accepted declaration,
so a re-declaring client holds the CR 117.4 pass-round open **with no attacker changing** — which
is the *empty* declaration's only consequence, and the reason the guard could not key on the map.

**The brief's preferred one-liner would have shipped a new bug — read this before the next batch
that wants to avoid a HASH bump.** The v3 queue brief said "PREFER reading `combat.attackers` over
adding a field", because a marker mirroring `defenders_declared` moves HASH. Refuted three ways,
any one sufficient:

1. **CR 508.1a** — "chooses which creatures … **if any**" — plus **CR 508.8** make an *empty*
   declaration a **completed** action, and the empty declaration is a live shipped client action
   (`params.rs:474` maps default params to `Command::DeclareAttackers { attackers: vec![] }`;
   `play-server/README.md` already called it "irreversible", aspirationally).
2. **CR 508.4 / 506.3** — "put onto the battlefield attacking" inserts **straight into
   `combat.attackers`** at four sites (`effects/mod.rs:1502`, `:6331`, `resolution.rs:6020`,
   `:6480`) with no declaration at all, and CR 508.4c exempts such creatures from declaration
   requirements. An `attackers`-keyed guard would have refused a player's **first, legal**
   declaration in any combat where such a creature entered attacking first.
3. It cannot see consequence (4) at all.

**The durable form of that lesson: a guard keyed on a collection cannot tell "chose nothing" from
"has not chosen", and cannot tell your own writes from someone else's.** `CombatState` gained
`attackers_declared: bool` (`#[serde(default)]`), hashed beside `defenders_declared`; **HASH 72 →
73 computed from the failing gate's own output**, history row APPENDED, **45** sentinel lines across
**44** files re-pinned (the plan predicted 44 — the extras were two bare-`72` spellings and two
split across two lines, which only a full run surfaced). PROTOCOL **35** gate-executed, unmoved.

**What shipped.** `GameStateError::AlreadyDeclaredAttackers(PlayerId)` mirroring
`AlreadyDeclaredBlockers`, carrying a `PlayerId` and nothing else (it reaches a client as 422 text —
Architecture Invariant 7). The guard sits **after** the priority check and **before** the
`CombatState` init and every tax debit, so a refusal is byte-identical (CR 732). The marker is set
on the **success path only**, inside the attacker-insert block and **before** the CR 603.3d
suspended-trigger early return — a declaration that suspends on a trigger target choice must not be
re-declarable. **CR 509.1a is verified covered and deliberately NOT widened** (`combat.rs:1103`).

**The offer layer had to follow, and that is what makes the closure provable.** Deleting
`local_game_playthrough.rs`'s `PolicyState` cap — mandated — would have turned its own
*"a rejection means the offer was wrong"* assertion red, because the policy would re-declare and the
engine would now refuse. So `legal_actions.rs:878` suppresses the `DeclareAttackers` offer once the
marker is set (SR-38). With **both** mitigations deleted *with their mechanism* (the bot's
`RepeatKey::DeclareAttackers` cap too), that test green **with no cap** is the closure proof for
`OOS-M11-9`. Same shape as PB-DX20's `KNOWN_FALSE_OFFERS` deletion.

**Two review findings worth carrying forward.**

- **A card-def comment asserted a defect the card does not have.** The first-draft note in
  `legions_landing.rs` put it in `OOS-DX21-1` and cited CR 508.6 for a claim CR 508.6 does not make.
  Legion's Landing is a **CR 508.3d per-declaration** trigger — evaluating false in a second combat
  where one creature attacked is *correct*. Following the note would have **regressed** the card.
  `OOS-DX21-1` is re-scoped to `windbrisk_heights` alone (turn-scoped, ruling 2007-10-01) with an
  explicit "do not migrate Legion's Landing" warning.
- **Four probes were reading state their failing call never touched.** `process_command` returns
  `Result<(GameState, …), GameStateError>`, so on `Err` Rust discards every mutation the callee
  made. A probe that clones state, expects an `Err`, then reads the *original* passes identically
  whether the guard is at the top of the function or absent. T6's entry-vs-success-path revert did
  not redden until rewritten to call `handle_declare_attackers(&mut state, ..)` directly, and T4's
  CR 117.4 pin was **fully vacuous** until repaired the same way. Filed tree-wide as
  **`OOS-DX21-7`** — a sweep of existing "rejection leaves state unchanged" probes would likely
  find more.

**Also measured, not asserted** (review M7): suppressing an offered action **reindexes every
subsequent `RandomBot` draw** (`random_bot.rs:63` picks uniformly by index), so offer-layer changes
move seeded fixtures that have nothing to do with them. Gate-config rejection rate moved
**31.081‰ → 6.909‰**, wasted-tap share **89% → 92%**, both inside their ratchets and **neither
constant changed**; `pb_dx32_fuzz_output.rs` T4.1/T4.3 unmoved **proven by an executed ablation**,
not observed. Filed as `OOS-DX21-6`.

**Numbers.** Tests **4,398 / 0 / 5** (+10 over the 4,388 pre-edit on-branch baseline), residual list
empty, independently re-run. Coverage unmoved **1,133/1,803 = 62.8%**, proven by byte-identical
report regeneration (two comment-only card-def edits, so an empty defs diff would have proved
nothing). Benches slightly faster and within noise. Golden scripts: exactly two files carry ≥2
`declare_attackers` and both repeats are **cross-turn**, so no script churn and SR-9b green.
Review **0 HIGH / 7 MEDIUM / 8 LOW, all 15 taken**.

**Seeds.** `OOS-M11-9` **CLOSED**. Filed `OOS-DX21-1..7` in `docs/audits/decision-point-audit.md`
§8.1. The two a successor should read first: **`OOS-DX21-4`** (CR 508.8's skip predicate is a
step-END read of `combat.attackers`, so killing your only attacker mid-step skips declare-blockers
*and* combat damage — pre-existing, and the naive `!attackers_declared` fix is **wrong**) and
**`OOS-DX21-2`** (the CR 509.1a twin of the offer hole, deliberately not widened into).

**Artefacts.** Plan `memory/primitives/pb-plan-DX21.md`; stage 0 `pb-DX21-stage0.md`; review
`pb-review-DX21.md`; revert matrix and every measurement `pb-DX21-execution-notes.md`.

---

## Worker Handoff (PB-DX20, `scutemob-198`) — one derivation, two consumers

**What was wrong.** An Aura's CR 303.4a target requirement lives in `KeywordAbility::Enchant`,
which `casting.rs` special-cased. Every offer-side consumer reads
`mtg_engine::spell_target_requirements`, which reaches `card_def_target_requirements` — and that
function sees `AbilityDefinition::Spell.targets` only. An Aura has no `AbilityDefinition::Spell`,
so the list was empty, `target_count_range` was `(0, 0)`, the browser rendered a zero-target
action, and the human's click 422'd. **13 deck-legal `Complete` Auras**, on first contact.

**The queue brief's prescription was one layer off, and this matters for the next batch.** It said
to synthesize the requirement in `crates/simulator/src/legal_actions.rs`. That file is not on the
browser's path: it decides *which actions exist*, not *what they announce*. The fix landed in
`rules/queries.rs`, and `legal_actions.rs` took **zero** lines — as did every production file
under `tools/play-server/` (`view.rs` already read everything through the query). If you are
chasing an offer-side defect, find the consumer before you patch the producer.

### What shipped

* `casting::enchant_target_to_requirement` — a **total** map over all 9 `EnchantTarget` variants,
  exhaustive `match` with **no wildcard arm**, so a future variant is a compile error rather than
  a silent `vec![]`.
* `casting::aura_spell_target_requirements(chars, base)` — synthesizes only when the object is an
  Aura enchantment, `base` is empty, and the keyword is present. Consumed by **both**
  `handle_cast_spell` and `queries::spell_target_requirements`. That is the SIM-1 lesson taken
  literally (`effective_cast_cost` consuming `apply_commander_tax` rather than re-deriving it):
  the two sides are one arithmetic, not two that agree today.
* **No new `TargetRequirement` variant** — the mapping is expressed in existing variants, so
  PROTOCOL/HASH could not move, and they did not (gate-executed: **35 / 72**).
* Bestow (CR 702.103b) gets the same keyword transform query-side, applied to a *local clone* of
  `chars`, so the two derivations cannot drift the day `StubProvider` learns alt-cost casts.
* Reconfigure (`OOS-CARDS1-2`): the `replay_harness.rs` synth site carries CR 702.151a's
  `TargetCreatureWithFilter { controller: You, exclude_self: true }`. The equip repair was **not**
  copied — CR 702.6a has no "another", CR 702.151a does.
* `KNOWN_FALSE_OFFERS` deleted, and the whole excusal mechanism with it. Any refusal in that
  driver is now unconditionally fatal. That, not an assertion, is what proves the closure.

### The three things worth reading before trusting anything here

1. **The CR 303.4a gate was KEPT, deliberately.** It is now a redundant second check, and the
   reason is that `matches_enchant_target` is the **SBA's own** predicate (CR 704.5m). Keeping it
   at cast time is what guarantees cast-time and SBA-time agree — a *different* property from
   "the offer and the cast agree". Trading one for the other would have been a silent regression.
2. **The Reconfigure symptom was worse than its seed row said.** The row reads as an offer-side
   gap. In fact `abilities.rs`'s legacy `AttachEquipment` guard is an `if let Some(..)` over the
   empty `targets` vec, so a zero-target attach **passed validation, paid the mana, and fizzled at
   resolution with no error and no event**. The two pre-existing `mechanics_m_z/reconfigure.rs`
   tests could never have caught it: both accept `Err(_) | Ok(no attachment)`, so they are green
   under either outcome. T5.2/T5.3 are the strict versions; T5.5 is the discriminating one.
3. **The SR-5 keyword registry caught what both targeted test runs missed.** Two implementation
   dispatches each ran their own scoped `cargo test` and both were green; the full-workspace run
   then failed `core::keyword_registry::registry_sites_match_the_source_tree`, because
   `queries.rs` is now an Enchant **handling site** and had not been declared. A query that reads
   a keyword to decide what may be announced *is* behaviour. Targeted runs are not a substitute
   for the workspace run — this is the second batch in a row to learn that from a gate.

### The review cycle: 1 HIGH / 5 MEDIUM / 7 LOW — all 13 taken

The HIGH is **not the primitive**; the reviewer re-derived the 9-variant / 6-field equivalence by
hand and found it exact in both directions. The HIGH is a **card def inside this batch's own 13**:
`imprisoned_in_the_moon` declares `EnchantTarget::Permanent` for a printed *"Enchant creature,
land, or planeswalker"*. It was half-live but **unreachable** before this batch — `Permanent`'s
arm is a bare `true`, and no offer surface read the keyword — and PB-DX20 opens exactly that door,
so the browser now offers artifacts and enchantments and the cast succeeds. **Filed as
`OOS-DX20-10`, not fixed, and the reason is concrete**: `EnchantFilter` has `has_card_type`
(single) and `has_subtypes` (an OR over **sub**types) but no OR over card *types*, so the printed
line is inexpressible today, and adding a field to `EnchantFilter` moves HASH. A wrong-way-round
deviation pin (the PB-DX19 `deviation_animated_nexus_...` precedent) tells the successor to invert
it, and the pin is a **roster**, so a second instance cannot appear silently.

The most useful MEDIUM: **T1 proved less than the plan claimed**. It asserts `offer == cast`, but
post-fix both sides run the *same* synthesized requirement, and the only Aura-specific cross-check
left can only reject and iterates `Target::Object` only — so T1 was blind to strict **narrowing**
and to every **player-side** error. `Permanent → TargetCreature` and `CreatureOrPlaneswalker →
TargetAny` (the exact mistake the plan warns against in bold) would both have shipped green. The
fix is an exact-shape pin over all 9 arms plus expected accepted-candidate sets. **A differential
probe between two consumers of one function proves consistency, not correctness** — that is the
durable lesson, and it generalises to every "the two cannot drift" claim this project makes.

### Numbers

* Tests **4,373 → 4,388 / 0 / 5** (+15), `--workspace --no-fail-fast` to a file, pre-edit baseline
  measured on this branch BEFORE any edit, residual list empty.
* **PROTOCOL 35 / HASH 72** gate-executed and unmoved. **0 card-def lines** (`git diff --numstat
  -- crates/card-defs/` empty); coverage unmoved at **1,133 / 1,803 = 62.8%**, proven by a
  regeneration whose body came back byte-identical apart from its date/sha stamp.
* `clippy -D warnings`, `cargo fmt --check` and `tools/check-defs-fmt.sh` (1,803 defs) all clean.

### Seeds

**CLOSED**: `OOS-CARDS2-4` (HIGH), `OOS-CARDS1-2`. **Narrowed**: `OOS-SIM4-2` (the Aura clause
only), and `OOS-SIM5-4`'s recorded blocker — *"needs an engine query; `get_enchant_target` is
`pub(crate)`"* — is now **stale**, because `queries.rs` answers it and `TargetPlan::Unsatisfiable`
is reachable for Auras for the first time. **Filed**: `OOS-DX20-1..10` in
`docs/audits/decision-point-audit.md`. The one to read first is **`OOS-DX20-10`** (the HIGH);
after that, **`OOS-DX20-7`** — a roster gate on "Activated + `AttachEquipment` ⇒ non-empty
`targets`" would have caught `OOS-M11-10` and `OOS-CARDS1-2` by construction and would catch the
next one.

### For the collector

Branch `feat/pb-dx20-the-offer-layer-cannot-see-a-keyword-carried-target-`. Plan
`memory/primitives/pb-plan-DX20.md`, review `memory/primitives/pb-review-DX20.md`. The v3 queue
memo's §4 row 2 is shipped and can be struck; **next dispatch is PB-DX21** (CR 508.1, attackers
declared without limit, `OOS-M11-9`). No wire change, so no downstream re-pin is owed.

## Worker Handoff (PB-DX32, `scutemob-197`) — the fuzzer's output starts meaning something

**Three seeds closed (`OOS-SIM3-3`, `OOS-SIM3-4`, `OOS-CARDS2-3`), one PARTIAL (`OOS-SIM3-2`).**
Rank 19 of the v3 queue (`memory/primitives/seed-rerank-2026-08-02.md` §4), row 3 of
`docs/mtg-engine-feedback-engineering.md` §2.3 — **promoted from rank 19, user-approved
2026-08-03**. Plan: `memory/primitives/pb-plan-DX32.md`. Full stage-by-stage evidence including
every revert proof: `memory/primitive-wip.md`. Review: `memory/primitives/pb-review-DX32.md`.

### Stage 0 first: every baseline this batch could have quoted was dead

PB-DX22 (`95f53b78`) changed the deal, so no SIM-3 or SIM-5 number survives, and `OOS-DX22-13`
records that several of them were a five-game sample in the first place. Everything was
re-measured at HEAD **before any edit** and committed:
`memory/primitives/pb-dx32-measurement-head-{fuzzer,harness}.txt`.

| measurement at HEAD | value |
|---|---|
| workspace tests, this branch, pre-edit | **4,358 / 0 / 5**, residual empty |
| fuzz violations, 20 games × 200 turns | **426** = 301 `no_orphaned_tokens` + 114 `player_consistency` + 11 `attachment_validity` |
| bot rejections | **542 / 23,613 commands = 22.953‰** (5 games); **1,995 / 94,467 = 21.118‰** (20 games) |
| `RandomBot` wasted taps | **1,986 / 2,641 = 75%** (5 games); **8,423 / 10,720 = 78.6%** (20 games) |
| violations deduped by `(check, description)` | **94 → 20** (4.7×) |
| leaked tokens in the FINAL state | **0**, on 5 seeds and again on all 20 |
| deck pool | `all_cards()` **1,803** / `Complete` **1,133** / commander pool **90** |

**`OOS-SIM3-4`'s "929 of 938" was both stale and a sample.** At HEAD the orphaned-token class is
**70.7%** of the run, and `player_consistency` is a second class at **26.8%** that no seed row
records at anything like that size. Every figure this handoff quotes names its game count.

### What shipped, by criterion

* **(a) SR-38 becomes a run-level invariant.** `GameResult` carries `rejection_count` and a bounded
  `rejections` sample; the fuzzer prints a class histogram and **exits 1** above
  `MAX_BOT_REJECTION_PER_MILLE`. `rejection_count` was already unconditional — only the *sample*
  needed the new `MAX_SAMPLED_REJECTIONS = 8` cap for the journal-off (fuzzer) path, since
  `results` retains every game's `GameResult` and a 256-cap at `--games 1000` would hold 256,000
  cloned `Command`s.
* **(b) the waste instrument is promoted, not copied.** `WasteTally` folds `tap_runs` /
  `wasted_tap_runs` / `wasted_taps` / `total_taps` / `mana_pools_emptied` at the two sites that see
  exactly the journal's command stream — so the streaming fold and `sim5_bot_cast_discipline.rs`'s
  journal walk are provably the same measurement, and `metrics_of` was **kept** as the equivalence
  oracle rather than deleted. `OOS-SIM2-1` is named at the pin.
* **(c) the noise floor.** `check_no_orphaned_tokens` output is split into a `transient_violations`
  bucket at the point of collection; `--stop-on-error` and the crash-report writer key on the hard
  bucket only; counts are deduped by `(check, description)` and printed raw **and** distinct for
  both buckets. Hard violations **426 → 125**, games with ≥1 hard **16/20 → 6/20**, crash files
  **16 → 6**.
* **(d) `OOS-CARDS2-3`.** `CORPUS_DEFS`/`CORPUS_COMPLETE`/`COMMANDER_POOL` pinned exact in both
  directions, the pool recomputed by mirroring `deck.rs`'s own filter clause-for-clause.
* **(e) decision-point runtime coverage.** `crates/simulator/src/decision_coverage.rs` carries
  **ids only**; the roster is kept single-source by a **source gate appended to the existing**
  `decision_gate.rs` (`BASELINE` and `MAX_AUTO_CHOSEN_COMPLETE_UNION = 80` untouched).

### The split is only honest because of what replaces it — read this before widening it

Reclassifying a violation class as non-halting is exactly what SIM-3's `stack_consistency`
withdrawal was about, so the batch bought the right to do it in two ways. First, the **strictly
stronger end-state property** is asserted in the **hard** bucket at both terminal paths
(`invariants::check_no_leaked_tokens` — a new pure-state check, deliberately *not* in `check_all`),
and it measured 0 across all 20 games. Second, **nothing else was reclassified**:
`player_consistency` and `attachment_validity` stay hard, on purpose.

**So criterion (c) is met in its literal wording and not in the colloquial one.**
`--stop-on-error` still halts — now on seed 2's `player_consistency` at turn 123, two games in.
The fuzzer is not yet a clean smoke test; the reason has moved from a false-positive check
(`OOS-SIM3-1`, withdrawn) to a known-transient one (closed here) to an **undiagnosed** one
(`OOS-DX32-1`, filed). That is progress, and stating it as anything more would be false.

### Findings the batch did not go looking for

* **`HeuristicBot` can never leave a tap run open** (`OOS-DX32-5`). It scores `TapForMana` at 0, so
  every tap it makes is an auto-tap prefix inside one atomic sequence. The equivalence probe's
  revert stayed **green** on the plan's own fixture (§7 R8's anticipated failure, hit for real) and
  had to be given a human-`submit()` fixture. Any "0 wasted taps" measured on that bot is weaker
  evidence than it reads as.
* **Every threshold needed two constants** (`OOS-DX32-4`). The debug/25-turn gate and the
  release/200-turn binary measure genuinely different populations, and the gate measures *higher*
  in both cases (31.081‰ vs 21.118‰; 89% vs 78.6%) — which is also the proof the duplication is
  forced rather than a dodge, since an evasive twin loosens for a *lower* number.
* **The SR-38 channel produced its ranked defect list on the first run** (`OOS-DX32-9`):
  `InsufficientMana` (`OOS-SIM6-3`), `InvalidTarget` (`OOS-SIM5-5`), the blocker-refusal family
  (`OOS-SIM5-3`, the largest), and CR 303.4a Aura casts (`OOS-CARDS2-4`). Close any of them and
  ratchet the constant down.
* **Runtime coverage is budget-dependent** (`OOS-DX32-3`): 4 of 5 served rows at the CI gate's
  10×60 debug budget, 5 of 5 at the binary's 20×200 release budget. Both recorded, not just the
  better one.

### The review cycle: 0 HIGH, 8 MEDIUM, 10 LOW — all 18 taken, and one was proven by experiment

The reviewer **had no shell and executed nothing**, and said so; its revert checks were source
inspection. The coordinator closed that gap by executing three things directly, and one of them
found a live hole:

* **`OOS-DX32-6`, the block-comment gate hole.** Wrapping one `UNOBSERVABLE_ROW_IDS` tuple in
  `/* … */` removed it from the **compiled** roster (`ROW_COUNT` 22 → 21) while the source gate's
  `quoted_strings` still found its literals. **The gate stayed green — and so did all 12 probes in
  `pb_dx32_fuzz_output.rs`.** A row silently vanished and nothing in the workspace noticed. Fixed
  (`strip_block_comments` + a raw-count assertion that also catches a duplicated id) and the same
  experiment now fails `left: 21 right: 22`, re-run by the coordinator after the fix. **The open
  half: `strip_line_comments` is used by the other source-reading tests in `decision_gate.rs`, and
  those were not audited for the same hole.** Third appearance of this class in this file's family.
* **T6.1(b) and T4.2 verified genuine** by execution, matching the runner's quoted messages. T4.2's
  revert needed `let _ = state;` as well as `#[allow(unreachable_code)]` — the first attempt failed
  to compile under `-D unused-variables`, which is plan §7 R7's exact class and is worth repeating:
  **a revert whose rebuild failed proves nothing.**
* **M1 — the batch zeroed a diagnostic in its own precedent file.** `local_game_playthrough.rs`
  still split `game.violations()` on `no_orphaned_tokens`, a string that can no longer match, so
  its run report would have printed `0 transient-token reports` forever. Nothing asserted on it, so
  no test went vacuous — but it is the `OOS-DX22-13` "a number's meaning changed silently" class,
  committed in the file the batch cites as its own pattern. Fixed to read
  `transient_violations()`; verified by execution (seeds 7/42 now print 12 and 4).
* **M6 — the thresholds cited the 5-game sample while the 20-game artefact sat in the same commit.**
  Re-quoted from the 20-game run. `MAX_RANDOM_BOT_WASTED_TAP_PCT` **kept at 85** with the real
  headroom (6.4 points over 78.6%, not the ~10 the 75% figure implied) stated as a deliberate
  choice, because the fuzzer is not run-to-run deterministic for long games
  (`OOS-M11-3`/`OOS-DP3-9`) and a single point estimate should not be shaved to the wire.

### For the collector

* Tests **4,373 / 0 / 5** (+15 over the 4,358 pre-edit baseline), full `--workspace --no-fail-fast`
  to a file, residual list **empty** — re-run independently by the coordinator after the fix cycle.
* **PROTOCOL 35 / HASH 72 gate-EXECUTED** (`--test core -- protocol_schema hash_schema`, 38 tests
  green) and unmoved; `PROTOCOL_VERSION = 35` at `protocol.rs:360`, `HASH_SCHEMA_VERSION = 72` at
  `hash.rs:743`.
* **0 wire, 0 engine source, 0 card defs** — `git diff main..HEAD -- crates/engine/src/
  crates/card-defs/ crates/card-types/ crates/view-model/` is EMPTY. Coverage unmoved
  **1,133/1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py` to a body identical
  apart from its own git-sha/date stamp lines, then reverting the churn.
* **`tools/` is exactly one file, `+1 -0`** — a `..Default::default()` in a `#[cfg(test)]`
  construction site. `cargo test -p play-server` 78/0 unmoved.
* The only engine-side file touched is the **test** `crates/engine/tests/core/decision_gate.rs`
  (appended to).
* **`memory/primitives/seed-rerank-2026-08-02.md` is untouched** — striking row 19 is the
  coordinator's at collect.
* Successor candidate: **`OOS-DX32-1`** — diagnose `player_consistency` (is it ever true *at
  rest*?). It is the last thing standing between the fuzzer and a usable smoke test, and it is
  26.8% of a run.

## Worker Handoff (PB-DX22, `scutemob-196`) — the fuzzer becomes a real instrument

**Three seeds closed: `OOS-UI2-1`, `OOS-SIM3-1`, `OOS-SIM1-4`.** Rank 4 of the v3 queue
(`memory/primitives/seed-rerank-2026-08-02.md` §4), row 1 of `docs/mtg-engine-feedback-engineering.md`
§2.1. Plan: `memory/primitives/pb-plan-DX22.md`. Full stage-by-stage evidence:
`memory/primitive-wip.md`. Review: `memory/primitives/pb-review-DX22.md`.

### The mandatory pre-plan measurement, and what it settled

The brief made one measurement mandatory *before* acceptance evidence: does a bot cast its
commander around turn 12-24 through SIM-1's command-zone loop (`legal_actions.rs:675-693`), or
is the offer suppressed? It was run first, at HEAD, and committed as the branch's **first**
commit (`891d346c`, raw output `memory/primitives/pb-dx22-measurement-head.txt`) so the
ordering is checkable rather than claimed.

**Answer: SUPPRESSED, and `OOS-SIM1-4` is the cause.** 5 games / seed 1 / `--max-turns 200`:
`commander_ids` populated **0/4 in every seat of every game**, **zero**
`CommanderCastFromCommandZone` in ~56,800 commands, first `SpellCast` at turns
154/143/151/153/151. The provider's own filter says it — "the zone is NOT the filter;
`commander_ids` is" (CR 903.8; CR 408.1 is why) — and `fuzzer.rs` never populated it. So the
brief's disjunction resolves to its second branch: **SIM-3 did not measure a pre-SIM-1 build**,
and `OOS-SIM1-4` and the missing commander cast are ONE defect. That collapsed the batch's
sizing: no provider change was needed. Seed 2's turn 143 reproduces `OOS-SIM3-1` exactly.

### What shipped

* **`crates/simulator/src/fuzz_setup.rs` (new).** The fuzzer's pregame build, lifted out of
  `src/bin/fuzzer.rs`. **This file exists because Cargo compiles `src/bin/*.rs` as its own
  crate, so no integration test could `use` the fuzzer's state build** — which is exactly how
  `crates/simulator/tests/local_game.rs::build_state` came to be a hand-written copy ("Mirrors
  `mtg-fuzzer::run_single_game`'s builder logic") carrying the identical CR 903.6 defect. Both
  callers now share `place_registered_deck`, which does the placement **and** the registration
  in one `if let`, so they cannot separate again.
* **Deliberately NOT in `setup.rs`.** Every play-server seed pin is a function of `setup.rs`;
  keeping the fuzzer's build in its own file makes "this batch cannot move a play-server pin" a
  property a reviewer checks from the diff's **file list**, not its contents. It held:
  `cargo test -p play-server` 78/0, nothing re-derived, nothing adjusted.
* **CR 103.3 / 903.6 shuffle** off the game's own `StdRng`, interleaved per seat
  (`deck₁, shuffle₁, deck₂, shuffle₂, …`) exactly as `setup.rs` does. There the interleaving is
  load-bearing; here it is free, and the free choice is the one that keeps the two paths the
  same shape.
* **CR 903.6 registration + CR 903.9b `register_commander_zone_replacements`.** Both are
  required and they fail differently: omit the second and CR 903.9a still works (it is an SBA
  keyed on `commander_ids`) while CR 903.9b silently does not exist and any count of
  `CommanderZoneRedirect` reads zero **vacuously**. Proven by isolation, not argued: under a
  revert deleting only that call, P6 reddens and P7 stays green.
* **`tests/local_game.rs` fixed in both halves.** Stage 3 rewired it onto the shared helper; a
  follow-up (`eb60cc80`) found it still lacked the CR 903.9b call — the same half-built
  Commander game one link down — and closed it with its own probe.
* **The fuzzer reports its own census.** A constant-size `MechanicsTally`
  (`local_game.rs`, surfaced by `GameDriver::run_game_with_mechanics`) folded from events
  already in hand, so `record_journal: false` and the fuzzer's memory profile are untouched.
  It is **not** a `GameResult` field: `tools/play-server` constructs one and `tools/` was off
  limits. This exists because the review caught the batch committing its "before" and deleting
  its "after" — see the lesson below.

### The A/B, both sides reproducible from committed code

`--games 20 --seed 1 --max-turns 200 --threads 1 --profile fuzz`. After-side raw output:
`memory/primitives/pb-dx22-measurement-after-fixcycle.txt`. **The before side requires building
the merge base** — the shipped binary cannot produce it.

| | before | after |
|---|---|---|
| `CommanderCastFromCommandZone` (CR 903.8) | **0** (~56,800 commands / 5 games) | **36**, in 16/20 games |
| `CommanderReturnedToCommandZone` (CR 903.9a) | 0 | **13** |
| seats with commander damage (CR 903.10a) | 0 | **16/20 games**, max **31** (past the 21 threshold) |
| `CommanderZoneRedirect` (CR 903.9b) | 0 (no mechanism) | **0** (mechanism exists, no game triggered it — `OOS-DX22-9`) |
| first `SpellCast` turn | 143-154 | **3-29**; library-only **5-29**, median 17 |
| `SpellCast` total | 121 / 5 games | **670** / 20 games |
| violations | 1,519 | **426** (301 `no_orphaned_tokens` / 114 `player_consistency` / 11 `attachment_validity` / **0** `stack_consistency`) |
| wins / errors | 9 / 11 `MaxTurnsReached` | **20 / 0** |
| avg turns · cmds/turn | 191.7 · 58.8 | **103.4 · 45.7** |

### Three deliberate remaining divergences from `setup.rs` — read these before "fixing" one

The fuzz path is **not** a `build_initial_state` replacement and was not made into one:
no opening hand (CR 103.5, **`OOS-DX22-1`**), no `validate_deck` (CR 903.5a/903.4,
**`OOS-DX22-5`**), no `DeckSource`. `OOS-DX22-1` is measured, not assumed: the first
`Command::PlayLand` is turn 1-7 on all 20 seeds, so land supply is not the limiter — a seat
starts with zero cards and draws one per *personal* turn, so it has ~T/4 cards by game turn T
at four seats. That is the reason the band is 3-29 rather than 3-12.

### What the repaired instrument immediately found

**`OOS-DX22-8`** — `attachment_validity`: `Object ObjectId(532) attached to ObjectId(677) which
doesn't exist`, 11 violations across seeds 5, 9 and 15 (the batch first recorded "×3, seed 5"
off the binary's 5-game print cap and under-counted 4×). Repro
`cargo run --profile fuzz --bin mtg-fuzzer -- --replay 5 --players 4 --max-turns 200`;
check `invariants.rs:386`; CR 400.7 / **704.5m** (Aura → graveyard; 704.5n is Equipment, which
unattaches and *stays*). **Pre-existing — 0 engine lines in the branch diff** — and deliberately
unfixed. Transient, one turn per game, and all 20 games still ran to a winner, so it is a live
false-positive candidate of the `OOS-M11-7` SBA-lag family SIM-3 withdrew: classify it before
fixing it. A plausible mechanism this batch uniquely enabled: 13 CR 903.9a returns means
commanders changed zones in fuzz games for the first time, and a CR 400.7 zone change is exactly
the orphaning event.

### Durable lessons

1. **A revert-proof written read-only is a hypothesis** (`OOS-DX22-11`). Two of the plan's ten
   predictions were false when executed: P4's stated revert leaves it green (seeds 1 and 2 draw
   different *decklists*), and P9's reddens 1 of 4 seeds, because a registered commander is cast
   from the **command zone** and so is not gated by library order — Stage 3's fix partially
   masked Stage 2's from that probe. Both gates were left where the plan put them and the real
   discrimination was executed and recorded instead.
2. **A universal negative must be measured over its own denominator.** `bin/fuzzer.rs` prints
   per-violation detail for only the first five offending games, and the batch published
   "not one of 426 is `stack_consistency`" off 94 printed lines. The claim survived the real
   tally — but the sample had projected `player_consistency` at ~1% where it is **27%**. Every
   historical "check X never fired" claim in this project came off that same cap
   (**`OOS-DX22-13`**).
3. **Committing the "before" and deleting the "after" is the same defect the batch exists to
   close.** The headline numbers first came from a scratch `examples/` file that was deleted;
   the review called it, and the repair was to make the fuzzer print them. Re-measured, every
   published number matched to the digit — the instrument was accurate, it was *unreproducible*.
4. **A source gate that greps a file body is satisfied by its own comments.** P11 matched
   `player_commander` in the doc comment explaining the rule, so deleting the real call left it
   green while six behavioural probes reddened. It now strips line comments — safe rather than
   merely stricter, because all four genuine placers were checked for a real call first.
5. `memory/gotchas-infra.md`'s stale-binary trap fired **three times** in this batch: a revert
   that fails to compile under `-D warnings` makes `cargo test` run the *previous* binary and
   report a pass. Check for the `Compiling mtg-simulator` line before trusting any red.

### For the collector

Every recorded fuzz seed predating this merge is dead (**`OOS-DX22-7`**); the docs say "the
PB-DX22 merge, `scutemob-196`" in four places where a merge sha would be better once one exists
(`bin/fuzzer.rs` module doc, `workstream-state.md`'s `--seed 504` annotation, the audit §8.1
banner, feedback-engineering §2.1). `memory/primitives/seed-rerank-2026-08-02.md` is
**untouched by design** — the coordinator strikes the PB-DX22 row at collect, and §2.4's "one
open measurement this task could not settle" is settled by the answer above.

## Worker Handoff (UI-6, `scutemob-194`) — the whole-library search view (G9, CR 701.23a)

**G9 of `memory/playtest-triage-2026-08-02b.md` CLOSED, both halves — the LAST row of its
successor table, so that triage is now fully dispatched.** The playtest said *"only showed legal
basic lands — should be able to view whole library when searching — current view is too
cumbersome — should be a list which you can check"*. **The filter was never the defect**:
`candidates` IS the engine's answer space and `handle_answer_effect_choice` refuses anything
outside it, so widening it would be offering illegal answers (SR-38). What was missing is
CR 701.23a's **look** — *"To search for a card in a zone, look at all cards in that zone (even if
it's a hidden zone)"*. So the look and the pick are now two lists, sent separately and rendered
separately. **0 engine lines and 0 simulator lines** (`git diff main..HEAD -- crates/` empty);
PROTOCOL **35** / HASH **72** gate-executed and unmoved. Tests **4,345 / 0 / 5** full workspace
`--no-fail-fast` to a file (+4 on this branch's own pre-edit baseline of 4,341 — two HTTP probes,
one frontend source gate, and the `/review` cycle's restriction probe; the Invariant-7 gate was
**renamed**, not added), residual list empty. `fmt`, `clippy --workspace --all-targets -D warnings` and `tools/check-defs-fmt.sh`
(1,803 defs) all clean. **0 card-def lines**, coverage untouched at **1,133/1,803 = 62.8%**.

**The Invariant-7 gate went red on purpose and that is the interesting part of this batch.**
`test_ui1_view_rs_reads_game_state_in_exactly_the_two_known_places` is now
`test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places`, and the re-pin is argued
at the pin site in CR terms rather than being a number bump. Three things bound the new read, and
each is a constraint a careless implementation would have missed:

1. **The searcher's own library only.** `player` is `PendingDecision::player`, which
   `api.rs::seat_view` has already filtered to the viewing seat, and the engine's search effect
   builds candidates from `ZoneId::Library(p)` for that same `p`.
2. **Sorted by NAME, never in library order.** CR 701.23a grants a look at the cards; it does not
   grant a look at the shuffle CR 701.23e exists to protect, and Architecture Invariant 7 names
   library *order* explicitly. Sending `Zone::object_ids()` verbatim would leak draw order to the
   seat that just failed to find — a real defect, in the *right* client rather than the wrong one.
3. **CR 121.1: "all cards in that zone" is not always the whole library.** Under an opponent's
   Aven Mindcensor the searcher *"searches the top four cards instead"*, so the entitlement is
   four cards. This was **found by the `/review` cycle, not by the implementation** — the first
   draft enumerated the library unconditionally, which would have shown 89 cards with 85 marked
   "look only". `library_look_cards` now calls the same `apply_search_library_replacement` the
   engine's search path calls and narrows through the same `Zone::top_n`. This makes it the
   **second** place in `view.rs` that restates an engine rule rather than delegating
   (`action_modes` is the first and says so); it is recorded the same way, and every divergence
   is in the narrowing direction.

**Why the gate had to become a needle SET, measured rather than argued.** The new read spells
`.zone(`, not `.objects()` — and with the channel in the tree, `.objects()` in `view.rs`'s
production region is **still exactly 2**. The pre-UI-6 single-needle gate would have stayed
**green** while a new hidden-information channel opened underneath it. That is MR-M11-01's lesson
arriving a second time in the same file, three sessions later. Worse, the *first* revert run
against the two-needle re-pin replaced `state.zone(..)` with `state.zones().get(..)` — the same
channel one accessor over — and the draft went green. So five needles are now pinned at **0**
(`zones()`, `objects_in_zone(`, `player(`, `object(`, `players()`), two of them added by the
`/review` cycle, which pointed out that the first draft closed the *plural* of one needle and the
*singular* of another while leaving each one's opposite number open. Each zero-pin was proven red
by an executed revert that fires it alone. It is still an enumerated set and not a proof about
every raw read; both the gate and `question_card_label`'s doc say so in those terms.

**The new channel got its OWN behavioural gate, and the sibling could not have covered it.**
`test_ui6_a_foreign_seat_never_receives_the_whole_library_look` mirrors the UI-1 scry gate's
construction (move `PlaySession::human`, not the decision — `advance()` refreshes `pending`
straight back), but it exists separately because the scry gate's raw-body needle is the
`looked_at` **key** and a search payload has none. Proven red by executing the revert: deleting
`seat_view`'s `pending.player == human` filter puts seat 1's entire library, **every card named**,
into seat 2's body, and the assertion quotes it.

**The fixture is new, and the reason is worth carrying.** UI-1's `ui1_install` search is Diabolic
Tutor — *unrestricted*, so its candidate set IS the whole library and `all_cards` would be
set-equal to it. **A fixture like that can never exhibit a look-only card**, so it could never
falsify the claim under test. UI-6 uses **Solemn Simulacrum** (`{4}`, colourless, `Complete`,
ETB `SearchLibrary` with `basic_land_filter()`) at `main_deck[0]` of a mono-black deck, plus six
distinct MV≥6 mono-black fillers whose only job is to sit in the library as cards the search
cannot find. At `UI6_SEED` (= `UI1_SEED`) that yields **89 in library / 33 findable / 56
look-only**. Its `may_fail_to_find` is `true`, the opposite of the UI-1 probe's `false`, so
between them both CR 701.23b/d branches are now exercised over HTTP.

**Browser-verified live** (headless Chromium, playwright-core, release server on :3046, seed 116
→ Three Visits at turn 9 — the UI-4 tuple reused): 89 rendered rows, 33 pickable buttons, 56
look-only, a column list scrolling 2082px inside 224px, the look-only row a `DIV` whose forced
click produced **0 POSTs and 0 selection** with Confirm still disabled, and a non-default pick
posting `{"found":97}` against a server default of `10`. 0 `pageerror`s, 0 error strips,
`command_count` 341 → 342. **Path correction for the next worker: the chromium binary is at
`~/.cache/ms-playwright/chromium-1228/chrome-linux64/chrome`**, not the `chrome-linux` the UI-4
handoff records — that path no longer exists and cost a launch failure.

**CR 400.7 trap, for whoever repeats the browser check**: Three Visits puts the found card onto
the battlefield, where it is a **new object** with a new `ObjectId`, so "the clicked id is on the
battlefield" is false even on success. The captured POST body is the discriminator, not the board.

**The `/review` cycle found 6 and all 6 were taken**, one of them a real rules defect (the CR
121.1 restriction above, live-reachable because `aven_mindcensor.rs` declares no `completeness`
field and is therefore `Complete` by the `#[default]` derive — the same generator PB-DX3b and
PB-DX4 both hit). Two more were gates whose message overstated what they asserted: the
`look-tag` needle was satisfiable by the **stylesheet alone** (deleting the `<span>` left it
green), and `library_look_cards` said it "asserted" a premise it only states in prose. The
restriction fix is pinned by `test_ui6_the_look_narrows_with_a_search_restriction` on a second
fixture — seat 2 holds Aven Mindcensor, seed **29**, read off a 300-seed sweep — asserted against
the engine's own `top_n` rather than against the literal `4`, and proven red by revert (89 ids
vs 4).

**Frontend**: `SearchPicker` is a scrollable checkable **list**, not a wrapped button grid — a
fixed left edge is what makes ~99 rows scannable. Rows are the union of `allCards` and
`candidates`, each carrying `pickable = (id in candidates)`; a look-only row is a plain `div`
with a visible `look only` tag, **not a disabled button**, because a disabled control reads as
"not right now" and CR 701.23a's distinction is permanent. `select` and `emit` both re-check
membership (the emit guard explains the refusal in CR terms rather than letting the server's 400
read as "request failed"). A one-click *"hide the N I can't find"* filter is **off by default** —
defaulting it on would restore the exact behaviour that was complained about. Gate:
`test_frontend_search_picker_looks_wider_than_it_picks`, proven red three ways (render candidates
only; accept a look-only id; make the look-only row a disabled button).

### Seeds filed (UI-6)

* **`OOS-UI6-1`** — *The picker opens on a wall of look-only rows.* `all_cards` is
  name-sorted, so in a Swamp/Forest deck every findable card is late in the alphabet and the
  first screen is unpickable cards (observed live: the top 10 rows at seed 116 are `Archetype of
  Endurance` … `Collector Ouphe`, all look-only). The filter box and the "hide the N I can't
  find" toggle each fix it in one action, so this is UX ranking, not correctness. Two candidate
  treatments, both with a cost stated: sort findable-first (loses the single A–Z scan the sort
  exists for), or scroll to the first findable row on open (keeps the sort, costs an effect).
  Deliberately not decided by this batch.
* **`OOS-UI6-2`** — *`all_cards` is filled in at the `SearchLibrary` arm only, and nothing
  gates that.* Any future `PickOne` question gets an empty look list and the client silently
  falls back to candidates-only. That is the safe direction, but a new arm that *should* carry a
  look entitlement would ship without one and no test would notice. A roster gate over the
  `EffectChoiceQuestion` variants routed through `PickOne` would close it — the same shape as
  SR-5's keyword registry.
* **`OOS-UI6-3`** — *The client's graveyard-search union branch is unreachable and therefore
  untested.* `SearchPicker` merges candidates absent from `allCards` because
  `also_search_graveyard` puts graveyard cards in the answer space that are in no library.
  Measured: `finale_of_devastation` is the **only** def with `also_search_graveyard: true`, and
  it is `Completeness::partial`, so `validate_deck` rejects it — the branch cannot fire today.
  Fold into the R7 frontend harness when it exists rather than building a fixture for it.
* **`OOS-UI6-4`** — *The field is named `all_cards`, which overstates it in one case.* It is
  the **library**, narrowed by CR 121.1; a graveyard search's candidates are not in it. The name
  is the triage's own recommendation and the doc is precise, so this is naming, not behaviour —
  `library_cards` or `look_at` would say it. Renaming is a DTO change with a frontend prop to
  match; cheap, and worth doing only alongside another change to this shape.
* **`OOS-UI6-5`** — *The Invariant-7 count gate is still an enumerated needle set.* Seven
  needles now, five of them zero-pins, two of those added because the first draft's own revert
  defeated it with a synonym. A read through an accessor nobody listed stays invisible. The
  durable fix is type-level — a wrapper `view.rs` must go through to reach `GameState` — not more
  needles, and it is the same limitation MR-M11-01 is about.
* **`OOS-UI6-6`** — *`library_look_cards` restates an engine rule and can go stale silently.*
  It is the second such site in `view.rs` (`action_modes` is the first). If the engine's search
  path stops calling `apply_search_library_replacement`, or starts restricting by something other
  than `top_n`, the look narrows wrongly with nothing to catch it. A shared engine query —
  `rules::queries::searchable_library(state, player)` returning the ids the search will actually
  consider — would let both the engine and the view read one implementation. That is an engine
  line, so it was out of scope here.

## Worker Handoff (ENG-2, `scutemob-193`) — targets in the event log (G7, CR 601.2c)

**G7 of `memory/playtest-triage-2026-08-02b.md` CLOSED, event-log half.** Before this batch no
cast/activate/trigger event carried its targets, and a **player**-targeting trigger emitted
nothing at all — the playtester watched a bot's Fell Specter hit them and the feed said only that
a triggered ability went on the stack. One additive `GameEvent::TargetsAnnounced` (discriminant
132) now fires at announcement time from all twelve stack-push sites, and the view-model renders
it. **PROTOCOL 34 → 35, HASH 71 → 72**, both gate-computed from the failing gate's own output on
this branch (the triage's "33" is stale — ENG-1 moved both after it was written). Tests
**4,341 / 0 / 5** full workspace `--no-fail-fast` to a file, residual list **empty**. **0 card-def
lines**; coverage unmoved **1,133/1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py`
to a body byte-identical below its self-dating header, not by an empty diff.

**Shape: option (2) of the triage's three, and the rejections are the useful part.** Option (3)
(the triage's own "cheaper third option" — widen `PermanentTargeted` to cover `Target::Player`)
was evaluated first, as the brief demanded, and rejected on a structural ground, not a taste one:
`PermanentTargeted` is **Ward's dispatch channel**, and `flush_sorted` — the reported defect's own
site — emits none, so widening it **structurally cannot reach the defect**. Option (1) (add a
`targets` field to each of `SpellCast`/`AbilityActivated`/`AbilityTriggered`) was rejected as
unfalsifiable: a forgotten site emits `vec![]`, which is indistinguishable from a genuinely
targetless announcement, so no gate can tell the two apart. Option (2)'s separate event makes
"announced nothing" and "announced no targets" the same observable, and the census gate below is
what keeps the site list honest.

**The census gate is the deliverable that outlives the batch.**
`crates/engine/tests/primitives/pb_eng2_targets_announced.rs::every_announcement_site_is_classified`
enumerates all 26 `SpellCast`/`AbilityActivated`/`AbilityTriggered` push sites from source and
requires each to be classified `ANNOUNCES` or `NEVER_TARGETS` **with a reason inline**. A new
emission site fails the test until someone decides which it is. Part 3 additionally pins that the
`NEVER_TARGETS` sites have not quietly grown targets.

**Where targets can actually come from: 8 sites, not 12.** The only places a `StackObject` acquires
non-empty targets today are two struct literals (`casting.rs:4532`, `engine.rs:3703`) and six
`.targets =` assignments (`abilities.rs:1395/1778/1993/8559/9181/10682`). All eight announce. Four
more sites are wired anyway — `copy.rs:474/699` (cascade, discover) and `resolution.rs:5463/6183`
(cipher-copy, suspend free-cast) — because those four hardcode `targets: vec![]` **unconditionally**
today, which is itself a bug (`OOS-ENG2-3`); wiring them now means the announcement is correct the
day that seed is closed, rather than being a second thing to remember. The ninth target-carrying
construction, `copy.rs:163` (`copy_spell_on_stack`), is deliberately **excluded**: CR 707.10, a
copy of a spell is not *cast*, so there is no CR 601.2c announcement to report.

**Invariant 7 is honoured by reusing the existing chokepoint, not by a new rule.** `private_to()`
stays `None` and `reveals_hidden_info()` needs no arm — correct, and reasoned rather than omitted:
CR 601.2c declares targets as part of putting the object on the stack, and CR 400.2 makes the stack
a public zone, so the *event* is public. The *identity* of an object target may still be private
(CR 708.2, a face-down permanent), which is a per-FIELD verdict a per-EVENT `private_to()` cannot
express — so it is decided in `event_view.rs`'s existing `card_or` gate, which routes
`card_name` → `may_name` → `redact::viewer_may_identify`. Player targets are never redacted.
`crates/view-model/src/tests.rs` proves both directions on a face-down permanent, with a
non-vacuity assertion on the omniscient view so the test cannot pass by rendering nothing.

**Downstream, and the "zero changes expected" claim, confirmed by measurement.** `event_view.rs`
gets the prose arm plus an `event_tier` entry (**neither is compiler-forced** — the tier match is
non-exhaustive, so a new variant silently lands in `Game` and the feed's `stack` filter would never
show it; that class is `OOS-ENG2-7`). `tools/tui/src/play/app.rs` gets an arm (its `_ =>
String::new()` would otherwise drop the line silently — class filed as `OOS-ENG2-8`).
`tools/replay-viewer/frontend/src/lib/eventFormat.js` gets a raw dev-tool line. **`tools/play-server`
carries zero source changes and the play frontend zero changes** — verified, not assumed:
`grep "GameEvent::" tools/play-server/src` returns only doc comments, and the +60 lines in
`main.rs` are entirely inside `mod tests`. The existing `event_view_for` → `EventView` → JSON
pipeline carries the new variant unmodified. `state/hash.rs` is the one arm the compiler demands.

**Rider taken while in the file (§4.5/§4.6).** `GameEvent::TargetsChanged` (CR 115.7) had **no**
`event_view` arm at all and rendered as the bare kind string; it now renders `old → new` through
the same `card_or` gate (`OOS-ENG2-4` filed and closed in the same breath). And three shipped
comments cited **CR 108.1** — the *Oracle-text* rule — for "a player target is public"
(`OOS-ENG2-5`).

**That citation rider was itself wrong once, and the correction is the lesson.** The first
replacement was `CR 102.1 / 115.1 / 400.2`. The `/review` cycle caught it: 102.1 merely defines
what a player *is*, and 400.2 is about whether *cards' faces* are visible in a *zone* — a player is
neither a card nor a zone. Only one rule actually says a player can be a target, and it is the one
in this task's own title: **CR 601.2c**, *"an appropriate object **or player** for each target"*.
Shipped chain is now `CR 601.2c / 400.2` at all **six** sites (the original count of three was also
wrong). **Replacing a wrong citation with a plausible one is not a fix** — verify the replacement
against the CR text, which is what the reviewer did and the implementer did not.

### The crash, and what verifying inherited work actually caught

The first worker process died after committing stage E (PROTOCOL/HASH) but before verification and
close-out. The relaunched worker was told to trust the commits and verify them. The engine work
survived that scrutiny intact — but **every doc-comment count in it was wrong**, and one gate was
weaker than its own comment admitted:

| Finding | What it was | Why it mattered |
|---|---|---|
| The gate's Part 2 was `body.contains("push_target_announcement(")` | Two functions carry **two** announcement sites each (`flush_sorted`: T6 modular, T7 main; `resolve_top_of_stack_inner`: S4 cipher, S5 suspend), so either call alone satisfied it | **T6 has no behavioural probe anywhere in the suite** — deleting it was invisible to all 4,341 tests. Now counts per function; **proven red by executing the revert** (deleting the T6 call fails `left: 1, right: 2`; the old assertion passes on that same tree) |
| `events.rs` SR-4 comment said "8 call sites" | There are 12 | The invariant it asserted was **true at all twelve** — only the count rotted. Restated without a count, pointing at the gate as the thing that keeps it true |
| `event_tier` comment claimed the match "has no `_` arm" | The `_ => EventTier::Game` default is three lines below; the paren was also unclosed | A comment that argues from a false invariant is the PB-DX19 failure mode verbatim — that batch's HIGH survived 4.5 months behind exactly this |
| `FROZEN_HISTORY_PREFIX_DIGEST` in both schema gates | Values moved; no ENG-2 line appended to their running attribution logs | The logs are append-only by convention; a silent value move breaks the audit trail |

**Generalisable**: the crashed worker's *code* was trustworthy and its *prose about the code* was
not. Counts in comments are the first thing to rot and the last thing anyone re-derives.

### Browser verification (live headless Chromium, independently re-run)

Both runs are recorded as ESM task comments. The second was run by the relaunched worker precisely
because the first rested on evidence nobody could re-inspect.

| Run | Seed | Observed in the DOM |
|---|---|---|
| Original (comment 1310) | 12, :3041 | `Scrawling Crawler targets Human-1` ×3, kind `TargetsAnnounced`, tier stack, player Bot-4, immediately after the `AbilityTriggered` line; turn 26, 250 pass clicks, 1,984 feed lines |
| Re-run (comment 1314) | **193193**, :3047, release build | **`Omnath, Locus of the Roil targets Human-1`**, class `feed-line tone-plain tier-stack`, player **Bot-2**, turn 14, 129 pass clicks, 1,416 feed lines, **0 uncaught page errors** |

The re-run is a bot-controlled **triggered** ability (Omnath's ETB, "deals damage to any target")
naming the **human player** — the exact Fell Specter class G7 reports, which emitted nothing at all
before this batch. Two other DOM lines named objects (`Shrieking Drake targets Shrieking Drake`,
`… targets Foundry Street Denizen`), so the sentence is not a fixed string. Corroborated at the
wire level by an HTTP-only drive of the same seed finding the identical line in the human seat
payload's `events` array.

**Recipe for the next batch that needs this** (two things cost the re-run most of its time):
the page boots with **no game** — deal a table through the real pregame controls (fill the two
`input[inputmode="numeric"]`, click `button.primary`), and **"Use the default" in `DiscardPicker`
FILLS the selection, it does not submit** (`DiscardPicker.svelte:78`), so clicking it in a loop
spins forever — click `button.secondary` then `button.confirm`. Driving over HTTP *instead of*
clicking does not work for a feed check: `GET /api/game` drains the event cursor, so an external
driver steals the events the browser was going to render.

### Seeds

Filed: **`OOS-ENG2-1`** (MEDIUM — CR 702.21a: `flush_sorted` emits no `PermanentTargeted`, so
**Ward never fires on a triggered ability**; pinned wrong-way-round by a probe with an instruction
to the successor), **`OOS-ENG2-2`** (MEDIUM — same class, four more sites the recon missed:
`handle_activate_forecast`, `handle_scavenge_card`, the loyalty handler, `flush_sorted`'s modular
arm), **`OOS-ENG2-3`** (MEDIUM — cascade/discover/cipher-copy/suspend free-casts hardcode
`targets: vec![]`, so a free-cast targeted spell reaches the stack with no targets; three of the
four admit it in an in-source comment), **`OOS-ENG2-6`** (LOW — the "cards sections" highlight ask;
a derived `PermanentView` field, no engine change, explicitly out of scope), **`OOS-ENG2-7`** (LOW —
`event_tier` is non-exhaustive by design and nothing asks; proposes a count-only-grows ratchet),
**`OOS-ENG2-8`** (LOW — the TUI's `_ => String::new()` silently drops any new event; mitigated for
this variant, class stands), **`OOS-ENG2-9`** (LOW — the feed now carries two lines per
battlefield-object target, `PermanentTargeted` + `TargetsAnnounced`; the superset proof is recorded
so the follow-up can delete the `PermanentTargeted` prose arm without re-deriving it).

Closed: **`OOS-G7-1`** (this batch, event-log half; the triage's stack half was already REFUTED),
**`OOS-ENG2-4`** and **`OOS-ENG2-5`** (both by their own riders, `-5` twice — see the citation
paragraph above).

**Untouched by design**: `OOS-M11-10` (the loyalty-ability targeting gap) — this batch *announces*
loyalty targets (site A13) but does not touch that seed's substance.

**Successor candidate: `OOS-ENG2-1`/`-2` together.** Ward not firing on any triggered ability is a
game-outcome bug, the two seeds are one mechanism at five sites, and this batch's own census has
already enumerated every site it touches. It will move fuzz and golden parity — budget for that.

**Benches within noise, as predicted**: `full_turn_4p` **221.2 µs** (PB-DX6 pinned 220–222),
`priority_cycle_4p` **24.4 µs**, `sba_check` **14.2 µs**, `full_turn_6p` 351.3 µs, `board_wipe_4p`
107.0 µs. Expected — the helper is one `stack_objects()` scan per *announcement*, not per priority
cycle, and it returns before allocating when the target list is empty.

Full plan and per-stage reasoning: `memory/primitives/pb-plan-ENG2.md`.

## Worker Handoff (ENG-1, `scutemob-191`) — effect-driven discard is a real player choice (G3, CR 701.9b)

**G3 of `memory/playtest-triage-2026-08-02b.md` CLOSED.** `Effect::DiscardCards` used to execute
inline and call `discard_cards`, which picks `min_by_key(|id| id.0)` — the human's leftmost/oldest
card — and moved it. CR 701.9b: *"By default, effects that cause a player to discard a card allow
the affected player to choose which card to discard."* No def in the corpus prints "at random" or
"another player chooses", so the default covers the **entire** live corpus and the violation was
unconditional. It now suspends into a new `EffectChoiceQuestion::Discard` through PB-DP9's
existing suspend-and-replay machinery. **PROTOCOL 33 → 34, HASH 70 → 71**, both gate-computed
from the failing gate's own output. Tests **4,330 / 0 / 5** full workspace (`--workspace
--no-fail-fast` to a file, never tail-piped) against a **pre-edit baseline of 4,317 / 0 / 5
measured on this branch** — +13, being 11 new engine tests and 2 new play-server probes. `fmt`,
`clippy --workspace --all-targets -D warnings` and `tools/check-defs-fmt.sh` (1,803 defs) clean.
Coverage **unmoved at 1,133/1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py`
to a byte-identical body — **0 card-def lines changed in the whole batch**, which is a positive
assertion, not an omission: `fell_specter.rs` was `Complete`, correct and innocent, and the
defect was 100% engine-side.

### The one decision that shaped everything else: the ask lives in the ARM, not in the helper

The dispatch brief reasoned as if the ask went inside `discard_cards`, and concluded that the
full-hand short-circuit is what stops `Effect::WheelHand` double-counting across a suspend/replay.
**That reasoning is replaced.** The ask is in the `Effect::DiscardCards` arm
(`effects/mod.rs:1267`), so:

- **`Effect::WheelHand` cannot suspend, by construction** — it calls the helper directly and the
  helper never asks. Not "because the short-circuit catches it".
- **`Cost::DiscardCard` cannot suspend, by construction**, and this one is not a nicety: that call
  is inside `pay_optional_cost`, on a cost-payment path with **no resolution wrapper to roll back
  to**. An ask there would record a `pending_effect_choice` nothing can discharge — the trap-state
  class `OOS-DP9-14` was filed for. Placement makes "cost discards do not ask" structural rather
  than promised. (CR 701.9c also gives a cost discard rules of its own; it is a *harder* problem
  than a resolution discard, not an easier one.)

Because both guarantees are structural and structural guarantees rot silently,
`test_eng1_wheel_hand_discards_the_whole_hand_exactly_once_and_never_suspends` asserts the
**structure**, not the arithmetic. If a later batch "simplifies" by moving the ask into
`discard_cards`, that test goes red.

### Shape, and why each field is named what it is

`EffectChoiceQuestion::Discard { hand: Vec<ObjectId>, count: u32 }` — the **whole** hand,
ascending, because CR 701.9b restricts nothing, so the whole hand *is* the legal answer space.
`hand`/`count` rather than `candidates` to match `GameEvent::CleanupDiscardChoiceRequired.hand`
and `LegalAction::DiscardToHandSize.count`: the engine's two discard channels should use one
vocabulary.

`EffectChoiceAnswer::Discard { chosen: Vec<ObjectId> }` — **`chosen`, not `discarded`**. The three
sibling answers name a *destination* (`found`, `bottom`/`top`, `graveyard`/`top`) because those
questions are about where cards go. This one is not a partition — the unchosen cards stay in hand
— and a destination name would be actively **wrong**: CR 702.35a sends a chosen Madness card to
**exile**, so at answer time nothing has been discarded and the destination is not yet known.

**One question for all `n` cards, not `n` questions.** Nothing between picks can change the answer
space (no priority during a resolution, CR 608.2; a Madness trigger lands after, CR 603.3), it
matches CR 514.1's cleanup discard, and `DiscardPicker` already renders it. What it forfeits — an
effect whose k-th pick depends on the (k-1)-th — is seeded as `OOS-ENG1-4`.

### The short-circuit, and the two loop exits that are not compile errors

`n == 0 || n >= hand.len()` (which includes the empty hand) short-circuits on **CR 601.2c's
principle**, the same argument the search arm already makes: when the answer space admits exactly
one legal answer the announcement is *determined*, so there is nothing to announce. That is what
keeps a full-hand discard from costing a round trip and from perturbing a fuzz seed.

The `for p in players` loop has **two different exits and neither is a compile error**: `continue`
for the determined case (later seats still get asked) and `return` for the suspension (the whole
pass is discarded; every later seat's question is re-derived by the replay). Getting them
backwards is the easiest way to break this arm, so
`test_eng1_multiplayer_discard_exercises_both_loop_exits` drives both in **one** resolution — and
it also shows the rollback undoes a determined seat's already-applied discard, which is the
property that makes `return` correct.

### The bot/fuzz default is zero-churn, and it is the opposite end of the hand from its sibling

`default_discard_answer` takes the `count` **lowest** ids from an ascending hand — byte-identical
to `min_by_key(|id| id.0)` applied `n` times. No game *outcome* changes in any bot-only game; only
the **command trace** grows an `AnswerEffectChoice`. Note it is the **opposite** of
`rules::turn_actions::default_cleanup_discard`, which takes the `count` **highest**. Both are
faithful reproductions of two auto-picks that genuinely differed (CR 514.1's took `obj_ids.last()`;
CR 701.9b's took `min_by_key`). Do not "unify" them —
`test_eng1_defaults_reproduce_both_pre_batch_picks` pins both in one place and says why.

### Fixtures that moved, enumerated — three, all repaired by ANSWERING, none by weakening

All three drive resolution with a local `pass_all` helper that never pumps blocking decisions, so
the new suspension went unanswered and the resolution appeared to do nothing.

| Test | Card | Why it moved | Change |
|---|---|---|---|
| `casting/x_cost_spells.rs::test_x_cost_spell_basic_mana_payment` | Pull from Tomorrow | draw X then discard 1; post-draw hand of 3 makes `count=1 < hand.len()`, so it asks | new shared `resolve_through_any_discard_choice` answers with the default |
| `casting/x_cost_spells.rs::test_x_cost_effect_amount_xvalue_draw` | Pull from Tomorrow | same | same, **plus** the helper merges the replay's events — the suspension rolls the whole resolution back, so the `CardDrawn` events the test counts now appear on the replay pass, not the first |
| `primitives/pbp_power_of_sacrificed_creature.rs::test_greater_good_draws_by_sacrificed_power_then_discards_three` | Greater Good | discard 3 against a 4-card hand is no longer determined | answers with the default inline (this one reads final zone counts, not events) |

The default reproduces the pre-ENG-1 pick byte for byte, so **every original assertion keeps its
meaning**. Nothing else in the workspace moved: no seeded simulator fixture, no golden script (the
harness's `auto_answer_blocking_decisions` pump already answers any `EffectChoice` with the
default), and no fuzz outcome — the honest prediction there is "no fuzz change **because there is
no fuzz coverage here**" (`OOS-UI2-1`: the fuzzer has never cast a spell), not "no fuzz change
because it is zero-churn".

### The decision-gate yield: 91 → 80, read off the gate, not computed

`decision_site_walk.rs`'s `discard_cards` row flips `AutoChosen` → `Served { by: "ENG-1" }`.
`decision_gate.rs` loses the 11 `BASELINE` entries whose only auto-chosen row was `discard_cards`
and shrinks Izzet Charm to `["counter_unless_pays"]`; `MAX_AUTO_CHOSEN_COMPLETE_UNION` is set to
**80**, the number T6's own panic printed — deliberately **not** `91 − 12`, because the union is
over *defs*, not `(def, row)` pairs. `MIN_BASELINE = 50` clears with 30 headroom and was not
lowered. **Correction to the plan**: it said 13 baseline rows; T9's reconciliation says **12**
(11 solo + Izzet Charm), and the plan's number was off by one.

Worth reading before the next audit: `decision_site_walk.rs:317-326` has carried a **verbatim
statement of this defect** since 2026-07-27 — `why_not_flagged_is_wrong: "CR 701.9b: the affected
player chooses which card, by default; the engine picks the lowest ObjectId"` — green in the suite
the whole time. The audit found it and classified it as expected. That is the corpus-scale form of
the comment-debt failure below.

### Architecture Invariant 7: the first question that names HAND objects

`EffectChoiceQuestion`'s type doc used to say *"Every `ObjectId` in every variant names a card in a
HIDDEN zone — the library."* ENG-1 **falsifies that sentence**, and it is rewritten to state the
two premises separately rather than folding the new one into the old: the three library variants
are entitled by the *effect* (the player may see those ids only because this effect is resolving),
while `Discard` names cards the answerer **already holds**. Same conclusion — `private_to()` stays
`Some(player)` — different, weaker premise, stated so a reviewer can check it. The premise rests on
`entry.player` being enforced in three independent places (the `process_command` admission gate,
`handle_answer_effect_choice` check 2, and the play-server read guard), and the doc names all
three so relaxing one is visibly a leak.

`view.rs` routes hand labels through **`NameIndex`**, not `question_card_label` — these are the
answerer's own cards, already in the seat-redacted view, exactly as the CR 514.1 arm does it.
Routing an owned-hand question through the library channel would enlarge a channel that
`test_ui1_view_rs_reads_game_state_in_exactly_the_two_known_places` counts.

**The new-channel gate exists**: `test_eng1_a_foreign_seats_discard_question_never_reaches_this_payload`,
the hand-zone analogue of the UI-1 gate. Its revert (removing the `pending.player == human` filter
in `api.rs::seat_view`) makes it red, and the leaked payload's candidates render as
`(unknown card)` from the foreign seat — the leak is real and the gate catches it. **The shipped
`GameSummary.seed` HIGH is precisely what a redaction gate checking only the channel it was
written for costs, and a hand is a new channel.**

### The two SILENT plumbing sites — neither is a compile error

`handle_answer_effect_choice` check 4 is a `matches!`, which is not exhaustive: a miss refuses
**every** discard answer with "does not answer question". Check 5 sits before
`_ => unreachable!("variant agreement checked above")`: a miss **panics the engine** on the first
real answer. Both were extended; both are exercised by tests (b) and (f). `api.rs`'s
`validate_decision_params` has a `_ =>` catch-all with the same property — a miss there is a silent
400 on every discard — and it was extended too. `validate_partition` is deliberately **not** reused
for the discard: it is not a partition (the unchosen cards stay in hand) and its message strings
would give a false diagnosis. That is said in a comment so nobody "deduplicates" it later.

### `OOS-ENG1-9` — the batch's biggest discovery, measured and NOT fixed

Building the browser probe reddened its real-name assertion on Faithless Looting, and the cause
generalises: **CR 608.2d's suspend rolls the WHOLE resolution back** (`rules/resolution.rs`, `*state
= restart_point`). For a **draw-then-discard** printing the recorded question names hand objects
that the *restored* state does not contain — the draw was rolled back and CR 400.7 minted new ids
— so every candidate DRAWN IN THAT RESOLUTION renders as the unknown-label placeholder (corrected
by the /review fix cycle: the original wording overstated its own measurement — pre-existing hand
cards render their real names; the probe saw 5 of 7 correct on Faithless Looting, not 0 of 7). The
answer still applies correctly on submission (the replay re-draws deterministically and re-mints
the same ids), so this is a **display gap, not a correctness gap**.

**It is new to this variant, and not by design**: the three library questions name cards that
already existed before the resolution began, so they are immune **by accident**.

**Blast radius, measured, not guessed.** Of the **21** def files that actually carry
`Effect::DiscardCards` (a plain grep says 23 — `reforge_the_soul` AND `nezahal_primal_tide` each
mention it only in a comment explaining that they use `Effect::WheelHand` instead; the first review
cycle caught only the first of those two, correcting the figure a second time here, and derived it
by grep-minus-exceptions rather than an `all_cards()` enumeration, SR-36 — the method, not just the
number, is why it was wrong twice), **14 draw in the same effect**. The number that matters for
playability today is the deck-legal one: of the **12** `Complete` defs, **7 draw** — Chart a
Course, Faithless Looting, Frantic Search, Geier Reach Sanitarium, Greater Good, Izzet Charm, Pull
from Tomorrow — against 5 that do not (Burglar Rat, Consign // Oblivion, Fell Specter, Raiders'
Wake, Sword of Feast and Famine). **A clear majority of the cards a human can actually play, and
the dominant printing, not a corner case** — the loot effect is what "discard" mostly means in
Magic.

**Deferred deliberately, with the reason**: the correct fix is not a discard patch but a general
LKI-for-questions mechanism — capture each candidate's identity at the moment of the ask (where the
objects still exist) onto `PendingEffectChoice`, and widen `BlockingDecision`, `LegalAction` and
the view to carry it. That is a second wire-adjacent surface in a batch already bumping PROTOCOL
and HASH, and it generalises beyond discard to any future question whose answer space is created
mid-resolution. **The coordinator should weigh whether it is the immediate successor**: for those
7 deck-legal cards the human now gets a picker with unlabelled options where they previously got a silent
auto-pick, which is arguably worse for them until this closes. Filed in
`docs/audits/decision-point-audit.md`.

**The /review fix cycle closed the sharpest edge of this, without closing the seed itself**
(review Finding 2): two same-resolution-drawn candidates used to render as two buttons with
IDENTICAL text (`(unknown card)` twice), which read as a redaction bug in the seat's own hand.
`view.rs`'s `PickN` arm now gives each unlabelled candidate a distinguishing placeholder —
`(card drawn this resolution #N)` — so the human can still make a fully informed choice among the
pre-existing hand cards, which is strictly more agency than the pre-ENG-1 silent auto-pick had.
`OOS-ENG1-9` itself (the general LKI-for-questions fix) is still open.

### Comment debt — the thing this batch is a lesson about

`discard_cards`' doc read *"Discard n cards from a player's hand (first by ObjectId,
deterministic)"* — it stated a **placeholder as a design property**. Every one of the ~13 sibling
auto-pick sites the triage census found carries a `deferred to M10+` comment. **This one did not,
which is exactly why the PB-DP decision-point audit's greps missed it and a human found it in a
live game.**

> **A deliberate placeholder that documents its MECHANISM instead of its DEBT is invisible to every
> audit that greps for the debt.**

`discard_cards`' doc now says plainly that its `min_by_key` is the auto-pick path only, and
`Effect::Connive`'s inline comment — the last remaining copy of the exact comment shape that hid
this for a year — now carries `deferred, OOS-ENG1-2` and its CR cite.

### PROTOCOL / HASH, and the sentinel re-pin

PROTOCOL **33 → 34**, fingerprint `2cda8c05…`; **closure type count unchanged at 96** —
`EffectChoiceQuestion`/`EffectChoiceAnswer` have been in the closure since v31, only their declared
shape moved. HASH **70 → 71**, `decl_fingerprint` `ce89c998…` over 129 types, `stream_fingerprint`
`c2845544…`, and both frozen-prefix digests re-pinned to what their gates printed once the v33 and
v70 rows joined the prefix. New history rows appended in both files; **no shipped row edited**.

Sentinels re-pinned **by symbol** across 46 test files. **Two multi-line survivors** —
`pb_dx2_command_gates.rs:1478-1492` and `pb_dp5_pending_draw_choice.rs:1244-1253`, each carrying
both a HASH and a PROTOCOL sentinel split across lines — were invisible to every single-line grep
and were found only by reading the files. That is the exact failure PB-DX5 shipped with.
**Residual list after the pass: EMPTY**, confirmed by execution, not by inspection.

**One surprise worth carrying forward**: `stream_fingerprint_is_pinned` was still **green** before
the version bump even though the new `HashInto` arms already existed — the canonical fixture
carries no `Discard`-shaped `pending_effect_choice`, so the new arms were unexercised and only the
version byte moved the stream. **A hash arm can ship unhashed and unnoticed here**, and `hash.rs`'s
own warning already says the SR-19 gate scans structs only, so an enum arm dropping a field feed
passes every gate green (`OOS-DP9-13`).

### Browser verification (live headless Chromium, real clicks)

Seed **22**, 4 players, heuristic bots, human seat 1. The asking card is **Burglar Rat controlled
by Bot-2** — a bot-controlled `DiscardCards` resolving against the human — reached at step 48 of a
pass-only drive. 28 seeds tried, two hits (seed 22 Burglar Rat, seed 24 Fell Specter). Payload:
shape `PickN`, `answer_field: "effect_choice_answer"`, `chosen_key: "chosen"`, template
`{"Discard":{"chosen":[2]}}`, `default: [2]`, and **all seven candidate labels are real card
names**. Clicked **Elspeth, Storm Slayer (id 3)** — non-default, since the default is id 2, the
lowest `ObjectId`. Server-side proof: Elspeth left the hand and is in the graveyard; **the default
card (id 2, Plains) is still in hand**. Also verified: "Use the default" selects but does **not**
auto-submit (0 POSTs, `command_count` unchanged); "Back" leaves the decision intact at the same
`seq` and re-openable, not wedged; and the console carried exactly **one** message across the whole
session, a 404 for `/favicon.ico` — **no `DataCloneError`, no uncaught exception**, so the UI-4
class does not recur in `DiscardPicker`'s new template branch.

**A trap for the next author of a test like this**: the graveyard entry was id **427, not 3** — CR
400.7 mints a new object on the zone change, so an id-equality assertion across a discard always
reads false. Match by name.

### Seeds filed

- **`OOS-ENG1-1`** — `Cost::DiscardCard` (`effects/mod.rs`, inside `pay_optional_cost`) still
  auto-picks the lowest id. CR 701.9b covers a cost discard too. Excluded **structurally**: a cost
  is paid outside any resolution wrapper, so an ask there records a `pending_effect_choice` nothing
  can discharge, and CR 701.9c adds cost-specific rules an announcement must respect.
- **`OOS-ENG1-2`** — `Effect::Connive`'s inlined discard duplicates the `min_by_key` because it
  needs per-card nonland accounting. Now trivially closable *except* that the nonland counter must
  survive a suspend/replay — a real design question, not a rename.
- **`OOS-ENG1-3`** — no `chooser` field on `Effect::DiscardCards`. **Do not add one.** 21 def files
  carry the effect, 12 are deck-legal `Complete`, and **zero** print "at random" or "another player
  chooses". The two corpus cards that would need it (`gamble.rs`, `grief.rs`) are blocked TODO defs
  carrying no `Effect::DiscardCards` at all, so the field would ship with no reader.
- **`OOS-ENG1-4`** — the one-question shape forfeits a sequenced per-card choice. No current
  printing needs it; filed so a future "discard a card, then discard a card" does not silently
  inherit the wrong shape.
- *(`OOS-ENG1-5` is deliberately unused — the filed set skips it. Noted here per review Finding
  10 so a future reader does not go hunting for a seed that was never filed.)*
- **`OOS-ENG1-6`** — `Effect::MillCards` is the only sibling of the missing `.max(0)` fixed here
  (`resolve_amount(...) as usize` with no clamp wraps a negative to ~1.8e19; `discard_cards`' loop
  has no empty-hand break, so it was an effective hang in release from a legal `EffectAmount`). Not
  fixed — a drive-by in an adjacent arm is how review scope-creep starts. Check whether
  `mill_cards` has an empty-library break before deciding severity.
- **`OOS-ENG1-7`** — `DiscardPicker` submits ascending ids, not click order. CR 608.2f/404.3 make
  discard order a real player payload; shipped ascending because `check_ids` treats the list as a
  set and no card in the corpus reads graveyard order.
- **`OOS-ENG1-8`** — `fable_of_the_mirror_breaker` (`partial`) TODO names this primitive
  (*"DiscardCards has no player-choice bound"*) and is **NOT closed by ENG-1** — chapter II needs
  *optional* + *up-to-N* + a count-driven draw, and this question asks for **exactly** `count`.
  With the variant in the tree, closing it is a min/max widening plus an `EffectAmount` source, not
  a new primitive.
- **`OOS-ENG1-9`** — the draw-then-discard label gap above. **The successor candidate.**
- **`OOS-ENG1-10`** — the second `/review` pass's find: `tools/tui/src/play/input.rs`'s `'r'` key
  still submits the engine's default `EffectChoiceAnswer` verbatim for a discard, with no picker —
  pre-existing and identical to its scry/surveil/search handling (the `OOS-DP7-6`/`OOS-DP8-2`/
  `OOS-DP9-7` family), so NOT a regression, but "effect-driven discard is a real player choice" is
  now true on the browser only. Not fixed. See `tools/play-server/README.md` limitation 28.
- **`OOS-G3-2`** — the "engine picks for the player" census. The triage's list in
  `memory/playtest-triage-2026-08-02b.md` §G3 has never been machine-checked, and
  `decision_site_walk.rs`'s `AutoChosen` rows are the machine-checkable version of it. Reconcile
  the two and make the source comments derive from the table rather than the reverse.

`OOS-G3-1` (the defect itself) is **CLOSED**. `Effect::SacrificePermanents` remains the named
cheapest follow-on and is genuinely cheaper now, but it is a different rule with a **public**
answer space and therefore a different hidden-info argument — its own batch, not a rider.

### Roster-recall gate

TODO sweep over `crates/card-defs/src/defs/`: 27 hits, exactly **1** names this primitive
(`fable_of_the_mirror_breaker`, recorded above as a NOT-a-forced-add with its reason). The other 26
are a different primitive each. **0 forced adds, 0 card-def lines changed.**

### The `/review` cycle — two reviewers, 0 HIGH, 2 MEDIUM, 8 LOW, and **all of them taken**

Full findings: `memory/primitives/pb-review-ENG1.md`. Both reviewers looked specifically for a HIGH
in the four places most likely to hide one — the question-equality determinism premise, the two
loop exits, the three non-compile-error plumbing sites, and the `HashInto` field feeds — and each is
correct. **The MEDIUMs are worth reading even after they are closed**, because one of them is a
standing hole in a gate and the other is a lesson about how a deferral should be shaped.

**MEDIUM 1 — a hash arm can ship unhashed, and the warning that says otherwise is half false.**
`grep 'pending_effect_choice\|EffectChoiceQuestion\|EffectChoiceAnswer'` over
`crates/engine/tests/core/hash_schema.rs` returned **zero**: `canonical_fixture()` had never
populated `pending_effect_choice`, so **all four** arms of both enum impls had been unexercised
since PB-DP9 — not a new hole, an inherited one. `hash.rs`'s own warning claimed the enum impls were
"held by review and by `stream_fingerprint`, nothing else"; the second half was **not true**,
because `stream_fingerprint` is computed over a fixture that never reaches them. Dropping
`count.hash_into(...)` would have made two states differing only in a pending discard's count hash
**identically**, with `cargo test --workspace` fully green — SR-19's gate scans structs only, and
SR-9b's `harness_equivalence` cross-validates green because *both* regimes drop the same field. That
is an undetectable desync in exactly the state M10's network layer most needs to detect one.
**Closed here rather than seeded, and the timing is the whole argument**: closing it re-pins
`stream_fingerprint` on the **v71 row this batch was already writing and which has not shipped to
main**. A successor batch would have paid a HASH 71 → 72 bump plus a 46-file sentinel re-pin for a
test-fixture change — an order of magnitude more, for identical correctness. `canonical_fixture()`
now carries a `Discard`-shaped `pending_effect_choice` (hand of 3, `count: 2`) and a one-entry
bank; new `stream_fingerprint` `923b1ff8…`, **proven by executing the revert** — dropping `count`
turned the gate red printing `1edc655e…`, restored, green.

**MEDIUM 2 — the deferral was right, the placeholder was not.** See the `OOS-ENG1-9` section above:
two same-resolution-drawn candidates rendered as two buttons with *identical* text. Both reviewers
independently endorsed deferring the general fix and both flagged the placeholder as separately
fixable at zero wire cost. **Generalisable: when you defer a fix, the thing you ship in its place
is a deliverable too, and it can have its own defect.**

Of the eight LOWs, three were errors in **my own write-up** and are the ones worth naming, because
they are the batch's own thesis turned on itself: the def-file denominator was wrong **twice**
(23 → 22 → 21; `reforge_the_soul` *and* `nezahal_primal_tide` each mention `Effect::DiscardCards`
only in a comment, and I caught one of the two), and it was wrong because it was derived by
grep-minus-exceptions instead of an `all_cards()` enumeration — **SR-36 exists to say exactly
that**; and the `OOS-ENG1-9` summary overstated its own measurement ("every candidate" where the
evidence said 5 of 7 rendered correctly). The 12-`Complete` and 7-draw figures were right
throughout. Two more LOWs were **comment debt inside the batch whose thesis is comment debt** —
`rules/events.rs` still said the question's ids were library-only, and `view.rs`'s arm comment
asserted "the CANDIDATES are library cards" directly above the arm that disproves it. The last
structural one: `Cost::DiscardCard`'s guarantee — which plan §2.4 calls the *more* dangerous of the
two — had **no named guard**, and test (d) does not cover it (a future batch moving both the ask and
the short-circuit into `discard_cards` leaves (d) green because `WheelHand` passes
`n == hand_size`, while `Cost::DiscardCard` passes `n = 1` against a larger hand and would begin
recording undischargeable entries). `test_eng1_a_cost_discard_never_suspends` now exists, proven red
by an executed revert.

## Worker Handoff (UI-5, `scutemob-190`) — UX polish batch 2: G8, G10, G11, G12, G13

**All five UX rows of `memory/playtest-triage-2026-08-02b.md` closed. Frontend only: 0 engine
lines (`git diff main..HEAD -- crates/` is empty), 0 wire change, PROTOCOL 33 / HASH 70
gate-executed and unmoved.** Tests **4,317 / 0 / 5** full workspace (+4 over SIM-6's 4,313 —
the four new gates), measured with `--workspace --no-fail-fast` to a file. `fmt`, `clippy
--workspace --all-targets -D warnings` and `tools/check-defs-fmt.sh` all clean.

### The one decision the brief asked for up front, made once and applied three times

G11/G12/G13 all land in the `$viewer` components the two surfaces share in place. **The rule:
edit the shared file in place; where the two surfaces genuinely want opposite behaviour,
express the difference as a PROP rather than as a copy.**

| Item | Shared? | Why |
|---|---|---|
| G11 caption | in place, unconditional | the native-`title` collision is identical in the replay viewer — same anchor, same chrome |
| G12 board order | in place, unconditional | pure sibling-block order; the replay viewer has no opposing requirement |
| G13 land stacking | in place, behind `stackLands` (default **false**) | the replay viewer is a step *debugger*: `App.svelte`'s `openCard` opens the object you clicked, and folding five Forests into one chip deletes four of the objects you are stepping to inspect |

A fork of `ZoneBattlefield` would have duplicated 476 lines **including G11's and G12's fixes**
and forked again on the next `PermanentView` field — precisely what `PlayBoard.svelte`'s module
doc says the *leaf* components must not do. That is one rule with one exception criterion, not
three answers in one file.

### What shipped, item by item

**G8 — Concede placement + confirmation.** Out of the action row (filtered from **both** groups,
not just `controls` — dropping it from `controlKinds` alone would have re-shown it mid-play-list),
into the header beside "New game", behind a two-step confirm. Same `option.index`, routed through
`ActionBar.beginExternal` so there is no second code path to the most destructive control on the
surface. **Disabled with a visible reason rather than hidden** — a control that blinks in and out
of the header on every bot turn reads as a bug, or gets hunted for in the one moment it is
dangerous. The reason is rendered as text, not a `title`: **a native tooltip does not open on a
disabled button**, because a disabled control fires no pointer events, so "disabled with a reason"
written as a `title` is a reason nobody can read. Pickers' "Cancel" → **"Back"** at all eight plus
the unknown-shape fallback.

**G10 — mana sources.** A `▸ mana sources (N)` disclosure, collapsed by default, one row per
source *name* with a count (`Tap Mountain for mana ×4`), folded on the server's own label. **Not
hidden**, and the gate asserts *both* sides — collapsed **and** still submitting — because a later
tidy-up that deletes the group would satisfy the playtest note and break every activation cost,
every echo/cumulative-upkeep/recover payment, and every float-ahead-of-a-cost-increase (CR 608.2g).
`OOS-SIM6-3` untouched. Side effect worth knowing: `plays` can now be empty while mana sources
exist, so the empty state says *"No plays available beyond tapping for mana"* instead of lying.

**G11 — tooltip caption.** `cardTooltip` accepts `{name, caption}` and renders the caption inside
the floating div. All nine triage-named sites cleared — **plus roughly ten `title=` on the badges
nested inside those anchors** (`CMD`, `TAP`, `SICK`, `ATT`, counters, keyword abbreviations). Those
were not in the triage and produce the *identical* collision over a smaller hit area on a ~70px
chip; every one existed to expand an abbreviation, so they folded into a second caption line and
lost nothing. Shared `zoneCaption` for the four `CardInZoneView` sites so they cannot drift apart
again — writing the same template four times is how they drifted in the first place.

**G12 — board order.** **Lands moved down, rather than Artifacts/Enchantments moved up** — sliding
A/E up would also have pushed it above Planeswalkers, changing an order nobody complained about.
Result: Creatures, Planeswalkers, Artifacts/Enchantments, Lands, Other. **Artifact lands stay in
the Lands row, deliberately**, documented at the classifier with the one-line reversal named: a
player reads an artifact land as a land — it is what you tap for mana and what CR 305.2 limits —
and nothing here touches `card_types`, so it is still an artifact for Metalcraft and for artifact
removal. Only where the chip is drawn changed.

**G13 — land stacking.** Key is `(name, tapped)` **plus** sorted counters, `attached_to`,
`is_commander`, `is_token`, `summoning_sick`, `damage_marked` — a deliberate superset of what the
land block renders, because the failure mode of a too-narrow key is a silent lie about the board
and of a too-wide key is a chip that does not stack. The `#each` key is the fungibility string and
**not** the representative's `object_id`: tapping one Forest of five moves that permanent into a
different stack, and a key derived from a member that just left would destroy and rebuild a chip
that only changed its count.

**Click path, decided rather than implicit.** The chip nominates `members[0]` — arbitrary *and
immaterial*, since the key already required every member to be indistinguishable, and since tap
state is *in* the key a stack is wholly tapped or wholly untapped, so "first untapped" collapses to
"first". It hands the **whole group** up as a second argument, and `PlayApp.representativeFor`
falls through to a sibling carrying an offered action — the caller is the only party that knows
what the server offered. The extra argument is inert for the replay viewer, whose `openCard(card)`
takes one parameter.

### Gates: four, each proven red by executing a revert

All in `tools/play-server/src/main.rs`, so they run under `cargo test --all` and therefore CI.
Source-level for the standing reason: there is still no frontend test harness (plan §8 R7).

| Gate | Pins |
|---|---|
| `test_frontend_card_elements_carry_no_native_title` | per-**element**, via a tag walk over each `use:cardTooltip` anchor |
| `test_concede_lives_in_the_header_behind_a_confirmation` | out of both action groups; header arm/confirm; same entry point; eight pickers say Back |
| `test_tap_for_mana_is_grouped_and_still_reachable` | collapsed **and** still submits |
| `test_land_stacking_key_is_not_just_the_name` | every field of the key by name; `stackLands` default off; every play-surface instance opts in |

Nine reverts executed, all red, tree green again: `title=` restored on `ZoneHand`; `Concede` back
in `controlKinds`; `CostPicker` Back→Cancel; `concedeArmed` renamed; `manaOpen` default `true`;
the mana row's `onclick` deleted; `p.tapped` dropped from the key; `stackLands` default `true`;
`representativeFor` renamed.

**The G11 gate is the one worth reading, and it is per-element rather than per-file on purpose.**
`title` is fine and useful on a control that is not a tooltip anchor — the Export-report button,
`SeatCard`'s drawer toggle, `StepControls`' whole row — so banning the attribute outright would
have deleted working affordances to fix an unrelated bug. It walks each opening tag carrying
`use:cardTooltip`, tracking `{}` depth and quote state because a Svelte attribute value can
legally contain `>` (`class:pt-damaged={p.damage_marked > 0}`) and stopping at the first `>` would
truncate the tag and read as "no title here". **Its own first run found a bug in itself**: a
component's module doc *names* `use:cardTooltip` in prose, and walking back from there finds the
nearest `<` — `<script` itself, or a `<` comparison operator in code — and reports a tag that does
not exist. Now template-only with HTML comments blanked, and the synthetic non-vacuity case
carries both shapes, so the extractor is proven by execution rather than argued.

### Browser verification — 24/24 live, plus 10/10 on the shared components

Headless Chromium (playwright-core, `/usr/bin/chromium`) against a live `play-server` on **:3045**,
seed **190190**, 4 seats, heuristic bots. Driven over HTTP to turn 23 and stopped **while a
decision was still live** — the first attempt ran to turn 59 and the game was over, which leaves
nothing to concede and no mana source to offer. Stop condition: the human holds ≥4 untapped lands
of one name, a `TapForMana` is offered, and some seat's board carries an artifact/enchantment
*and* lands (an ordering assertion over a board of nothing but lands is vacuous).

| Item | Evidence |
|---|---|
| G11 | 0 elements matching `.permanent-card,.hand-card,.gy-card,.exile-card,.cmd-card,.chip,.stack-item` carry `title`; battlefield hover → `"Legendary Creature — Human Soldier\n2/1 · commander · First Strike"`; hand hover → `"Contagion Clasp\nArtifact"` |
| G12 | `["Creatures (2)","Artifacts/Enchantments (1)","Lands (6)"]` |
| G13 | `Plains×4` untapped, `Swamp×3` **tapped**, `Swamp×2` **untapped**, `Mountain×4` untapped — Swamp and Mountain each render as two chips because each exists in both tap states; clicking the human's own stack acted, `command_count 819 → 820` |
| G10 | `▸ mana sources (3)` collapsed; 0 `kind-TapForMana` in the plays group; expanded → `["Tap Mountain for mana ×3"]` |
| G8 | header shows `New game` / `Export report` / `Concede`; 0 concede in the action row; an **open** `TargetPicker` shows `["Confirm (0/1)","Back"]` and Back submits nothing (`820 → 820`); first click arms `"Concede — end your game? Yes, concede / Keep playing"`; **declining** leaves `game_over=false`, commands `824 → 824`; **confirming really concedes** (`winner Bot-4, 48 turns`); afterwards the button is disabled with `"the game is already over"` |

**Shared components, mounted against a fixture** rather than through the replay viewer's own
binary — `memory/gotchas-infra.md` records that starting that binary from an agent context gets
SIGKILLed (137). A throwaway Vite entry mounted `ZoneBattlefield` **twice on one page**, with and
without `stackLands`, over a 6-Forest fixture (3 plain untapped, 2 tapped, 1 untapped carrying a
charge counter) plus a Sol Ring. Results: viewer mode **6 chips, all count 1**; play mode
**`Forest×3` untapped / `Forest×2` tapped / one lone Forest** — the counter Forest correctly
refusing to merge with its otherwise-identical siblings; artifacts above lands; zero `title`;
caption `"Basic Land — Forest\ncharge counter ×1"` (the badge title that used to be native); and a
stacked chip handing up `[representative_id, group_length] = [1, 3]`. **This is a working
proof-of-concept of the R7 tier-1 harness** and took ~15 minutes; the recipe is: a directory beside
`tools/replay-viewer/frontend/src` containing `index.html` + `main.js` (`mount(Harness, …)`) +
`Harness.svelte` + a `vite.config.js` whose `root` is that directory, built with
`npx vite build --config <dir>/vite.config.js` from the frontend package (so `node_modules`
resolves), then served by `python3 -m http.server`. It was **not** committed — R7 is deferred and
the brief did not ask for it — but the next batch that wants a frontend harness should start here
rather than from scratch. Both production bundles were also rebuilt and both succeed (156 and 142
modules).

### The `/review` cycle found 8 and all 8 were taken — two were real defects, both in G8

1. **MEDIUM — the armed confirmation survived the decision it was armed against.** `local_game.rs`
   appends `Concede` to **every** decision it builds for the human, so a disarm `$effect` keyed on
   `concedeAction` being null essentially never fired. Arm Concede, change your mind, pass priority
   instead — and the red "Yes, concede" bar stayed up, live, across the next decision and the one
   after. **That is the accidental-concede class G8 exists to close, reintroduced by the guard
   meant to prevent it**, and the effect's own doc comment claimed the property the code did not
   have. Reproduced in the browser before the fix (armed, `seq 1446 → 1447`, `stillArmed=true`).
   Now keyed on `$decision?.seq`.
2. **MEDIUM — the header Concede was a silent dead control while a picker chain was open.**
   `beginChain` early-returns on `if (loading || chainOpen)`, and `chainOpen` is `ActionBar`-
   internal, so the button rendered enabled: click "Yes, concede", the bar vanishes, nothing
   happens, no error. **The same silent-dead-button shape UI-4 was dispatched to fix — and the
   shape that made the playtester reach for Concede in the first place.** `ActionBar` gains an
   `onChainOpenChange` push (a method call on a `bind:this` handle is not reactive, and this is
   read inside a `$derived`), and the disabled-reason list gains a fifth entry. Both fixes proven
   by revert: each reverted fix reddens its browser check, 24/24 → 23/24.
3. MEDIUM — stale README and no handoff. Both written (this file; README's Interaction section
   rewritten, and the "one change outside `tools/play-server`" heading generalised).
4. LOW — `position()` floored at the nominal image height when an image is expected. `onEnter`
   assigns `src` and positions synchronously, so on the first frame `offsetHeight` was
   caption-height alone (~30px) and the box could be centred with the image off-screen until the
   first `mousemove`.
5. LOW — `render()` no longer re-assigns an identical `src` on update.
6. LOW — the `ZoneStack`/`onCardClick` doc block had been orphaned by inserting `representativeFor`
   between it and `handleCardClick`. Moved back.
7. LOW — `StateView.svelte` and `CombatView.svelte` still carry card-element `title`s and are
   knowingly out of scope: neither anchors `cardTooltip`, so neither collides, and giving them a
   caption would mean giving them a tooltip (a feature, not this batch's repair). **The exemption
   is now machine-checked** — the gate asserts they are NOT anchors, so the day one grows a
   `use:cardTooltip` it goes red and the per-element ban starts applying to it. That is the only
   honest way to write an exemption down.
8. LOW — gate brittleness. The two array-literal assertions now read the *literals* (whitespace-
   and order-insensitive) rather than whole source lines, and the `stackLands` check became "every
   `<ZoneBattlefield>` instance opts in" with HTML comments blanked first — otherwise the prose
   explaining the prop counts as an opt-in and an added instance that forgot it passes.

### Durable lessons

- **A confirmation step is only as good as the event that disarms it.** The guard was written, was
  documented, and was keyed on the wrong signal, and the wrong signal was one that essentially
  never fires. A two-step confirm whose second step stays live across unrelated decisions is worse
  than no confirm, because it is a live destructive button you have stopped looking at.
- **"Disabled with a reason" written as a `title` is a reason nobody can read** — a disabled
  control fires no pointer events, so the native tooltip never opens. Same lesson as G11, from the
  other direction, and a reviewer will not catch it because the attribute is right there in the
  source.
- **A gate that is worth writing is worth firing at a synthetic offender.** The G11 tag walk was
  wrong on its first run in a way that would have made it green-on-nothing for two of six files;
  the non-vacuity arm caught it in the same minute it was written.
- **Commit before running revert experiments.** A `git checkout -- <file>` used to undo a revert
  also discarded four uncommitted `/review` fixes to the same file. They were reapplied and the
  rebuilt bundle hashed identically (`index-DlGFzzL8.js`), which is how the reapply was verified
  rather than assumed — but the cheap habit is to commit first.

### Seeds

- **`OOS-UI5-1`** — `StateView.svelte:139` (command-zone chip) and `CombatView.svelte:67/79/90`
  (attacker/blocker boxes) carry the native `title` that G11 removed everywhere else. Harmless
  today because neither anchors `cardTooltip`; the gate pins that premise. If either grows a card
  preview, the text must move to a caption first.
- **`OOS-UI5-2`** — land stacking is limited to the Lands group. Creature tokens are the other
  population that arrives in identical multiples (a board of nine Saprolings is nine chips), and
  `PermanentView` carries everything the key would need. Not done here because a creature's chip
  renders P/T, damage and summoning sickness, so the fungibility key has more to say, and because
  combat selection (`AttackerPicker` / `BlockerPicker`) picks per-`object_id` and would need the
  same representative decision made a second time.
- **`OOS-UI5-3`** — `manaSourceRows` folds on the server's rendered **label**, so two different
  cards would merge if `view.rs` ever printed the same sentence for both. It does not today
  (`format!("Tap {} for mana", card(source))` over the card name), and a same-named pair is
  fungible for this purpose anyway — but the fold is on presentation rather than on identity, and
  that is the kind of coupling that is invisible until it is wrong.
- **`OOS-UI5-4`** — the R7 frontend harness remains unbuilt. This batch proved the tier-1 shape
  works in ~15 minutes (recipe above) and then threw it away, which is the right call for a batch
  that was not asked to build it and the wrong outcome to repeat a third time. Every UI batch since
  UI-4 has paid for its absence in source-level gates that cannot prove a component renders.

## Worker Handoff (SIM-6, `scutemob-189`) — activation costs are payable, and the offer stops lying

**G4 CLOSED, both components.** The triage's chain was correct end to end and is
re-verified against HEAD: `LegalAction::ActivateAbility` (`legal_actions.rs:93-102`) had no
cost field; the offer loop (`:883-918`) checked mana/hybrid/Phyrexian/life and never
`ability.cost.sacrifice_filter`; `view.rs`'s `additional_costs_view` early-returned for
anything that was not a `CastSpell`, so `ActionBar`'s cost stage never opened; and
`params.rs:339-345` hardcoded `sacrifice_target: None` / `discard_card: None`. The engine
was innocent throughout — the wire fields have existed since PB-EF1.

**The fix is the UI-2 shape, one command over**: a new `ActivationCostPlan` on the action
(`ActivationSacrificeOption` / `ActivationDiscardOption`), an SR-38 suppression gate when
either eligible set is empty, the choice forwarded through `params.rs` (falling back to the
plan's own default so a *bot* submission is engine-legal), a picker block in
`additional_costs_view`, a validator arm in `api.rs`, and two new props on `CostPicker`.
**0 engine lines** (`git diff main..HEAD -- crates/engine/` empty), **0 wire changes** —
`LegalAction`/`ActionParams` are simulator types. PROTOCOL **33** / HASH **70**
gate-executed and unmoved.

### Three things this batch found that the brief did not predict

1. **The brief's refusal attribution is wrong, and the correction is the useful part.** It
   said "~95 InsufficientMana-on-ActivateAbility + 40 'activation condition not met' … your
   subject is ~80% of all bot command refusals". Re-running the SIM-5 A/B instrument
   (`crates/simulator/tests/sim5_bot_cast_discipline.rs`, seeds 0/7/42, 25 turns) at the
   merge base and printing every rejection class: **not one of the 166 refusals is a
   sacrifice- or discard-cost refusal.** `"sacrifice_target must be Some"` appears **zero**
   times. Those 135 are two *different* SR-38 gaps in the same loop (below). The
   cost-payment channel's refusals never appeared because the *bots* were never reaching
   them — which is exactly why this defect needed a human playtest to surface.

2. **The heuristic bot had to be taught to decline, and a seeded fixture caught it.** With
   the channel open and `ActivateAbility` scored at 40 (vs `PassPriority`'s 1) under a
   2-per-turn repeat cap, a bot ate two of its own creatures per turn, every turn.
   `test_ui3_combat_view_maps_attackers_to_defenders_and_blockers` went red — seed 21 no
   longer reached a declared blocker, because the blockers had been sacrificed. The bot now
   scores an activation whose cost NAMES an object below `PassPriority` (the established
   "0" idiom: below passing, above nothing, so it is still chosen when it is all there is,
   and `params.rs`'s default keeps that command legal). The dispatch brief's own guidance —
   do not teach bots sacrifice strategy; declining is acceptable. `RandomBot` still picks
   these uniformly, so the fuzzer keeps exercising the channel. **Verify this by reverting
   the score, not by reading it**: the UI-3 test is the instrument.

3. **The browser verification found a live 422 of its own, in the same loop.** Driving a
   real game to a Rummaging Goblin discard activation (`{T}`, Discard a card: Draw a card),
   the picker rendered perfectly, the human picked a non-default card, and the POST came
   back **422 — `"object ObjectId(499) has summoning sickness and cannot use abilities with
   {T}"`**. The offer loop mirrored none of the three refusals `handle_activate_ability`
   makes that are knowable from state alone: CR 302.6 summoning sickness, CR 602.5b
   `activation_condition`, CR 118.3 remove-counter. SIM-2 had built exactly this predicate
   for the MANA path (`mana_solver::tap_ability_is_activatable`, OOS-CARDS2-9) and the
   non-mana sibling was never written. It is now
   (`legal_actions::activated_ability_is_activatable`), and **that alone closes the 40
   "activation condition not met" refusals**.

### A/B, measured both ways (instrument: `sim5_bot_cast_discipline.rs`, seeds 0/7/42, 25 turns)

| | merge base | this branch |
|---|---|---|
| total bot command refusals | 30 + 44 + 92 = **166** | 30 + 28 + 55 = **113** |
| `activate: InsufficientMana` | **95** | **62** |
| `activate: "activation condition not met"` | **40** | **0** |
| `activate: sacrifice/discard cost` | **0** | **0** |
| wasted taps / `ManaPoolsEmptied` | 0 / 1 | 0 / 1 (SIM-5's gate holds) |

The 62 residual `InsufficientMana` are **`OOS-SIM6-3`** and are the largest single refusal
class left in the simulator.

### Browser verification — three flows, each with a NON-DEFAULT answer

Seed-scanned `POST /api/game` over 0..400 for a human opening hand holding any of the 37
`Complete` defs with an object-naming activation cost (**hand lives at
`state.zones.hand["Human-1"]`**, not `zones.hand`). **Known-good tuples, handed over so
nobody re-scans**: seed **79** → Yahenni, Undying Partisan; seed **62** → Altar of
Dementia; seed **219** → Rummaging Goblin (discard); seed **282** → Vampiric Rites; seed
**63/70/73/106** → High Market / Spawning Pit / Scavenger Grounds / Viscera Seer. Driver
and playwright scripts were scratchpad-only (~60 lines each, trivially rewritten from this
paragraph).

* **Yahenni (seed 79), activated IN RESPONSE to a bot's Dismember on the stack** — exactly
  the playtest report. The picker offered `Jadar` and `Zombie` and **not Yahenni itself**;
  the prompt read "Sacrifice **another** creature"; picking the non-default `Zombie`
  POSTed `{"cost_sacrifice_target":418}` → **200**; `Zombie` went to the graveyard, `Jadar`
  (the default) did not, the ability resolved above Dismember and Yahenni came back with
  `keywords: ["Haste","Indestructible"]`.
* **Altar of Dementia (seed 62)** — cost stage *then* target stage in one chain, one POST
  carrying `cost_sacrifice_target` **and** `targets` → 200. Archmage Emeritus (power 2)
  sacrificed, the non-default target Bot-3 milled exactly 2, Bot-2 milled 0.
* **Rummaging Goblin (seed 219)** — the discard half. Non-default `Balefire Dragon`
  discarded, goblin tapped, draw resolved. This is the flow that produced finding 3.

No error strip, no `pageerror`, no console error in any of the three.

### Card defs: 8 one-line repairs, and a stale belief that produced them

`yahenni_undying_partisan` was the mandated fix, but it is not alone: **8** activated
abilities print "Sacrifice **another** …" and carried `exclude_self: false`, so all 8 would
have started legally sacrificing themselves the moment this channel opened —
`yahenni_undying_partisan`, `ayara_first_of_locthwain`, `bartolome_del_presidio`,
`razaketh_the_foulblooded`, `umbral_collar_zealot`, `warren_soultrader`, `woe_strider`,
`baron_bertram_graywater`. Coverage is **unmoved at 1,133/1,803 = 62.8%** (regenerated;
only the header date, git SHA and rolling commit log moved).

**Why 8 and not 1**: three defs (`woe_strider`, `wight_of_the_reliquary`,
`vampire_gourmand`) carry notes asserting that "`Cost::Sacrifice` has no 'another' /
exclude-self semantics". **That has been false since PB-EF1** — `TargetFilter.exclude_self`
lowers to `ActivationCost.sacrifice_exclude_self` via `flatten_cost_into`
(`replay_harness.rs:4622`) and `handle_activate_ability` enforces it. Two of those notes
are corrected; the third pair still OMIT their abilities entirely on the stale belief
(`OOS-SIM6-2`). Same shape as PB-DX19's comment: the note, not the code, is why this
survived.

### Seeds filed

* **`OOS-SIM6-1`** (MEDIUM, engine) — `flatten_cost_into` reads only
  `TargetFilter.has_card_type` (**singular**) and ignores `has_card_types` (plural) and
  `colors`. `bartolome_del_presidio` / `umbral_collar_zealot` / `baron_bertram_graywater`
  print "creature **or artifact**" and lower to `SacrificeFilter::Creature`;
  `ayara_first_of_locthwain` prints "another **black** creature" and loses the colour.
  `SacrificeFilter::ArtifactOrCreature` **already exists** and the lowering never emits it.
  The direction is *narrowing* (legal plays refused), so it is not a wrong-game-state bug —
  but it makes three defs' printed text unreachable. Out of scope here only because of the
  0-engine-lines constraint.
* **`OOS-SIM6-2`** (LOW, card defs) — `wight_of_the_reliquary`, `vampire_gourmand` and
  `ruthless_technomancer` omit their sacrifice abilities on the disproved claim above.
  Re-authoring them moves coverage, so it belongs in a card batch, not here.
* **`OOS-SIM6-3`** (HIGH, simulator + human-facing) — **auto-tap covers `CastSpell` and
  nothing else.** `local_game.rs:738` returns `None` for every other command, on both the
  bot path (`advance()`) and the human path (`submit`). `can_afford` offers an activation
  whose cost is solvable *with taps*, the engine charges the *pool*, and the command is
  refused `InsufficientMana`: **62 of the 113 remaining bot refusals**, and a browser human
  activating a mana-cost ability gets a 422 unless they happened to have floating mana.
  This is the largest remaining SR-38 violation on this surface and the obvious successor.
* **`OOS-SIM6-4`** (LOW, simulator) — two engine refusals still unmirrored by the offer
  loop: `forage` (CR 701.61a, `abilities.rs:1235` — needs a Food artifact or three
  graveyard cards; **1 def** in the corpus) and `sacrifice_self` on a source under
  `CantBeSacrificed` (CR 701.21a, `abilities.rs:917`). Both are the same class as the three
  `activated_ability_is_activatable` now covers; neither has measured traffic.
* **`OOS-SIM6-6`** (LOW, latent — filed by the `/review` cycle) — the offer-time
  `activation_condition` evaluation uses `x_value: 0`, because `{X}` is not announced until
  command construction. The engine evaluates the same condition with the command's own
  `x_value` (`abilities.rs:261-271`), so an "Activate only if X is N or more" ability would
  be wrongly **suppressed** — the silent-unplayable direction, not the 422 direction.
  Unreachable today: every `Condition::XValueAtLeast` in the corpus is spell-side. Recorded
  in `activated_ability_is_activatable`'s own doc rather than left to be rediscovered.
* **`OOS-SIM6-5`** (LOW, TUI) — `tools/tui/src/play/input.rs`'s `'e'` key now routes
  through `action_to_command_with_params` (so the costs, modes and hybrid/Phyrexian plans
  are filled), but the TUI still has **no picker** for any of them: it always submits the
  plan's default. A human TUI seat cannot choose which creature to sacrifice.

### What this batch did NOT do, stated plainly

* **Multi-sacrifice is untouched** (`OOS-OS6-1` → PB-DX12). `sacrifice_target` is a single
  `ObjectId` on the wire and stayed one; nothing here reshapes it.
* **No frontend test harness still exists** (R7). The three browser flows were verified by
  hand with playwright-core; nothing automated covers `CostPicker`'s new block. The
  play-server probes cover the *channel* end to end and prove nothing about the component —
  the same limitation UI-2 and UI-4 both recorded.
* **The discard candidate list is the whole hand, unfiltered**, which is what
  `handle_activate_ability` accepts (it checks the zone and nothing else). If a def ever
  needs "discard a *land*", this descriptor has no field for it.

### `/review` cycle — 5 LOW, all 5 taken

The reviewer re-executed every load-bearing gate independently (4,312/0/5, PROTOCOL 33 /
HASH 70, fmt + clippy + defs-fmt, 0 engine lines) and confirmed by three separate reverts
that the suppression gate, the Yahenni `exclude_self` fix and the new activatable mirror
each have a test that goes red without them. All five findings were LOW; all five taken:

1. **The discard channel had no HTTP probe.** The sacrifice half did; the discard half was
   covered only by unit calls and the `params.rs` engine round-trip, so
   `activation_costs_view`'s discard block was verified in a browser by hand and by nothing
   automated. Added `test_sim6_activation_discard_is_answered_over_http` on a new mono-RED
   fixture (Lathliss commander, 99 Mountains, Rummaging Goblin) — deliberately a `{T}`-only
   ability, because an activation that ALSO costs mana fails on this surface for the
   unrelated `OOS-SIM6-3`, which would have made the probe pass or fail for the wrong cause.
   It also pins the CR 302.6 gate incidentally: the offer does not appear on the turn the
   goblin lands.
2. **An `additional_costs` array on an `ActivateAbility` was dropped in silence** — the
   mirror image of a guard this batch had just added in the same function. `params.rs`'s
   activation arm never reads that field and `ActivateAbility` sits inside its consuming
   allowlist, so `first_announced_field` could not catch it either. Now a 400, with a
   both-ways test (and a control that an activation announcing nothing is still accepted).
3. **`OOS-SIM6-6` filed** — see the seed list above.
4. README limitation numbering (the new item was inserted before, not after, item 22).
5. `docs/authoring-status.md` had been regenerated at the batch's first commit rather than
   at HEAD, so its rolling commit block was three commits stale. No count was wrong — no
   card def changed after that commit — but regenerated at HEAD anyway.

### Numbers

Tests **4,313 / 0 / 5** full workspace (+18 over SIM-5's 4,295): 11 simulator (10 for the
channel, 1 for the SR-38 mirror) + 7 play-server. Every suppression gate proven **red by
reverting the gate and watching the assertion fail**, not by inspection. `cargo fmt`,
`tools/check-defs-fmt.sh`, `clippy --workspace --all-targets -D warnings` all clean.

## Worker Handoff (SIM-5, `scutemob-188`) — bots stop wasting mana, and start announcing targets

**G5 CLOSED for its (1)/(2)/(3) halves; (4) DEFERRED with measurements (`OOS-SIM5-4`).**
The triage's chain was correct end to end and is re-verified against HEAD (pre-edit line
numbers, the ones the brief cites): the bot path built `[taps…, cast]` at
`local_game.rs:462-468` and applied them **one at a time** at `:471-472`; on failure `:474-491`
committed the taps, discarded `e`, and passed. The human path has never had that failure mode —
`submit` (`:549`) hands the identical vector to `apply_sequence` (`:700`), whose doc at `:694`
says it exists precisely to stop "a tap-then-cast sequence where the tap succeeded but the cast
was rejected". The cast was rejected because `random_bot::action_to_command` (`:142-193`) built
`ActionParams::default()` and filled only `attackers`/`blockers`, so `params.rs`'s `CastSpell`
arm (`:262`) forwarded `targets: []` and `casting.rs:5931` refused. `HeuristicBot` shares that
function (`heuristic_bot.rs:19`, called at `:346`), so **neither bot had ever cast a targeted
spell**.

### What shipped

* **(1) atomicity, `local_game.rs`** — the bot loop is now one `self.apply_sequence(commands)`
  call. Two deliberate behaviour deltas, both documented at the call site: invariants are
  checked once per *sequence* rather than once per command (the states no longer checked are
  mid-payment ones), and a recorded seed moves **only where a cast is rejected** — per
  `OOS-UI2-1` the fuzzer has never cast at all, so no fuzz seed can reach the changed branch.
* **(3) the refusal is kept** — `RejectedCommand { player, turn, command, error }`, with
  `LocalGame::rejections()` (retained, capped at `MAX_RETAINED_REJECTIONS = 256`) and
  `rejection_count()` (never truncated, so the cap is visible rather than silent). Exported on
  `GET /api/game/report` as `rejections` / `rejection_count` — that endpoint's `journal` records
  applied commands only, which is exactly the limit the triage hit ("the rejected command and
  its error string are unrecoverable").
* **(2) targeting, new `crates/simulator/src/targeting.rs`** — `plan_targets` returns
  `NotTargeted` / `Announce(Vec<Target>)` / `Unsatisfiable`, one target per **mandatory**
  requirement. Every legality decision is delegated to `crates/engine/src/rules/queries.rs`
  (`spell_target_requirements`, `ability_target_requirements`, `legal_targets_per_slot`,
  `target_count_range`); nothing re-derives a targeting rule outside the engine (the `OOS-RS-2`
  drift class). `random_bot::action_to_command` fills `params.targets` from it, and
  `HeuristicBot` inherits it through the shared function.

### Three decisions a successor should not re-litigate blind

1. **Not `Bot::choose_targets`.** The dead trait method takes `&[ObjectId]` and returns
   `Vec<ObjectId>`, so it cannot express `Target::Player` — half of what spells target. It is
   still dead; widening it is `OOS-SIM5-1`'s business, not a legality fix.
2. **Deterministic first-legal candidate, no RNG.** `legal_targets_per_slot` already enumerates
   deterministically (live players in seat order, then objects ascending). Drawing here would
   re-roll every recorded fuzz seed and every seeded play-server fixture for a *strategy* gain,
   and no layer here knows a spell's polarity anyway (removal wants an opponent's creature, a
   pump spell wants its own). Bots therefore target the lowest-`ObjectId` legal candidate,
   which for a `TargetPlayer` slot is often themselves — `OOS-SIM5-1`.
3. **Modes are queried as `spell_default_modes(state, card)`, not `&[]`.** This is the one place
   this module deliberately differs from `view.rs`'s `action_target_requirements`, which passes
   `&[]` because the *human* has not chosen yet. `params.rs` fills a bot's `modes_chosen` with
   exactly that default list, so querying with `&[]` would return `vec![]` for a
   per-mode-targeting card (`queries.rs` divergence 1) and the bot would announce nothing for a
   cast whose command *does* select a mode.

### A/B, measured both ways (instrument: `crates/simulator/tests/sim5_bot_cast_discipline.rs`)

Seeds 0/7/42, 25 turns, four heuristic bots, no human seat; the same journal walk the triage
did on `GET /api/game/report`.

| seed | wasted tap runs | wasted taps | ManaPoolsEmptied | taps | casts | targeted casts |
|------|-----------------|-------------|------------------|------|-------|----------------|
| 0    | 10 → **0**      | 20 → **0**  | 10 → **0**       | 65 → 46 | 17 → 20 | 0 → **2** |
| 7    | 15 → **0**      | 15 → **0**  | 15 → **1**       | 68 → 69 | 23 → 27 | 0 → **4** |
| 42   | 5 → **0**       | 10 → **0**  | 5 → **0**        | 55 → 60 | 19 → 22 | 0 → **1** |

The BEFORE column reproduces the triage's live 1:1 match exactly — `ManaPoolsEmptied` equals
wasted tap runs on all three seeds (10/10, 15/15, 5/5), as the triage measured 18/18.
**The one residual is explained, not waved at**: seed 7 keeps a single `ManaPoolsEmptied` at
T14, and its journal context shows a four-tap run whose cast **succeeded**, part of the
remainder spent on a second cast ~20 commands later and the rest destroyed at the step
boundary — greedy-solver slack (`OOS-SIM2-1`), not a wasted plan. `emptied_pool_context()`
uses a 40-command window for exactly this reason; a 5-command one showed only passes.

**Journal-verified targeted casts by bots** (impossible before this batch): T7 `Glacial Ray` →
player 1 and T18 `Damn` → a permanent (seed 0); T2 `Burst Lightning` → player 1, T3 `Goblin War
Strike` → player 1, T10 `Vandalblast` → a permanent (seed 7); T12 `Doom Blade` → a creature
(seed 42).

### What the recorded rejections immediately revealed (the point of fix (3))

166 refusals across the three seeds, now classifiable instead of inferable:
**~95 `InsufficientMana` on `ActivateAbility`** and **40 `activation condition not met`** — i.e.
the *activation-cost payment channel*, which is **SIM-6's** subject and untouched here; ~25
blocker-declaration refusals (`CrossPlayerBlock`, "the attacking player cannot declare
blockers", CR 508.1d must-attack) — `OOS-SIM5-3`; **4** modal `ActivateAbility` refusals
("requires exactly 1 target(s) for the chosen mode(s)", CR 700.2c) — `ability_target_requirements`
documents that a modal ability's per-mode slice is out of its scope, so a bot cannot announce
for one (`OOS-SIM5-5`); and **1** genuinely unsatisfiable cast (`Victimize`, no creature card in
the graveyard). Cast-side refusals are now ~3% of the total.

### Why fix (4) is deferred rather than shipped (`OOS-SIM5-4`)

Full argument and numbers in `targeting.rs`'s `TargetPlan::Unsatisfiable` doc. In short: the
predicate exists and the filter is short, but it would have suppressed **1 of 166** refusals;
it does **not** cover `OOS-CARDS2-4` (an Aura's restriction is a `KeywordAbility::Enchant`, not
a `TargetRequirement`, and `rules::sba::get_enchant_target`/`matches_enchant_target` are
`pub(crate)` — covering Auras needs an **engine** query this batch may not add); it costs a full
candidate sweep per offered cast per priority window on a path `queries.rs` itself says to
measure and cache first; and shortening the action list re-rolls every recorded fuzz seed and
seeded fixture, since `RandomBot` picks `rng.random_range(0..legal.len())`. Post-(1) an
unsatisfiable offer costs nothing anyway. Scope it as an engine query plus caching.

### Gates (each new gate proven to discriminate by executing a revert, not by assumption)

* `crates/simulator/tests/sim5_bot_cast_discipline.rs` — `seeded_four_bot_game_wastes_no_taps`
  (the A/B instrument; red both on the pre-fix per-command loop **and** on pre-fix zero-target
  params), `bot_announces_a_legal_target_and_the_engine_accepts_the_cast` (a black and a
  colourless creature on board: the bot must pick the non-black one *and* `process_command` must
  accept the command), `plan_targets_reports_an_unsatisfiable_requirement`,
  `a_rejected_bot_cast_commits_no_taps` (no land tapped, no mana floating, no tap in the
  journal, refusal recorded — pinned with a frozen `ZeroTargetCastBot` so it keeps testing
  ATOMICITY even as targeting improves).
* `tools/play-server/src/main.rs` `test_sim5_report_exposes_bot_command_rejections` — asserts
  the two report fields on every iteration (so a dropped field fails even with no rejection) and
  has a non-vacuity floor requiring a real refusal; went red when `record_rejection` was stubbed.
* **0 engine lines** (`git diff main..HEAD --numstat -- crates/engine/` is empty),
  PROTOCOL **33** / HASH **70** unmoved and gate-executed. Workspace suite **4,295 / 0 / 5**
  (+5 = this batch's gates), captured to a file, never tail-piped. `fmt`, `clippy -D warnings`
  and `tools/check-defs-fmt.sh` all clean. **No seeded fixture moved** — the `UI1_SEED`/
  `UI2_SEED`/`SIM1_SEED` pins and the six SEED-0 play-server probes were green untouched, which
  is why nothing in this handoff explains a moved pin.

### Seeds filed

* **`OOS-SIM5-1`** — bot target *choice* is "first legal candidate", and `legal_targets_per_slot`
  lists players before objects **in seat order**, so every player-eligible slot (`TargetPlayer`,
  `TargetAny`, `TargetCreatureOrPlayer`) resolves to **seat 1** — the human's seat in a
  play-server game, and the bot's own seat when the bot is seat 1. **Not a cosmetic seed**: it
  points every bot burn spell at one player, which changes the character of a seeded game and
  not merely its strategic quality. `Bot::choose_targets` is still dead and cannot express
  player targets at all. A real policy (opponent-preferring for removal, self-preferring for
  buffs) needs spell polarity, which is a `HeuristicBot` scoring project.
* **`OOS-SIM5-2`** — `TargetRequirement::UpToN` slots are announced empty (legal: min 0), so a
  bot never uses an optional target.
* **`OOS-SIM5-3`** — ~25 of 166 refusals are blocker declarations the provider offered and the
  engine refused (`CrossPlayerBlock`, "the attacking player cannot declare blockers", CR 508.1d
  must-attack). Pre-existing SR-38 residue in `legal_actions.rs`'s combat surface; now visible
  because rejections are recorded.
* **`OOS-SIM5-4`** — fix (4) deferred; see above and `targeting.rs`.
* **`OOS-SIM5-5`** — a modal **activated ability** with per-mode targets is unannounceable by any
  caller of `ability_target_requirements` (which documents the per-mode slice as out of scope),
  so bots refuse 4× per A/B run. Needs an engine query change, not a simulator one.
* **`OOS-CARDS2-4` unchanged** — Auras still cannot be announced; post-(1) the attempt is a
  harmless no-op that now shows up in `rejections()`.

### The `/review` cycle: 5 PASS, 4 LOW, all 4 taken

The reviewer re-ran everything rather than trusting the numbers — it reverted both fixes in a
scratch tree and reproduced the BEFORE column exactly, then reproduced AFTER on HEAD, then
re-ran the full workspace suite. Four LOW findings, all applied:

1. An in-source A/B summary in `local_game.rs` said "30 wasted taps across 30 tap runs". The
   verified figures are **45 wasted taps across 30 wasted runs, of 82 tap runs in all** — 30 is
   the wasted-*run* count, which is what `ManaPoolsEmptied` matches 1:1. Comment corrected; the
   handoff table and task comment were already right.
2. `record_rejection` retained records regardless of `LocalGameLimits::record_journal`, while
   `driver.rs` sets that flag `false` specifically so the fuzzer retains nothing. Retention is
   now gated on the same flag (the **count** is not gated, so a crash report keeps the number).
3. `OOS-SIM5-1` was under-stated: players are enumerated first *in seat order*, so every
   player-eligible slot resolves to seat 1 for every bot. Seed text strengthened above and in
   `plan_targets`' doc — it changes a seeded game's character, not just its quality.
4. Measured: with targeting kept and only `apply_sequence` reverted, only seed 42 reddens the
   whole-game A/B test, because fix (2) removed nearly all cast-side refusals. So the A/B test
   is **not** the primary atomicity gate — `a_rejected_bot_cast_commits_no_taps` is, and it
   freezes a `ZeroTargetCastBot` into the fixture exactly so it keeps discriminating however
   good targeting becomes. Recorded in that test's doc so a future seed re-pick cannot lose it.

## Worker Handoff (SIM-4, `scutemob-187`) — the mulligan stops re-rolling the table

**G2 CLOSED. CR 103.5: a mulligan permutes a FIXED library-plus-hand multiset.** The
triage's chain was correct end to end; re-verified against HEAD (post-edit line numbers):
`PlayApp.svelte:478` (`Take a mulligan`) → `main.rs:184` (`.route("/game/mulligan", …)`) →
`api.rs:1236` `post_mulligan` → `:1260` `play.mulligan()` → `session.rs:422` →
`session.rs:428` `setup::redeal(&self.cfg, …)` → `setup.rs:503` `redeal` (perturbed seed,
`..cfg.clone()`) → `setup.rs:319` `build_initial_state` → `deck.rs:53`
`commanders[rng.random_range(..)]`. **The load-bearing link is `self.cfg`**: it held
`DeckSource::RandomPerSeat`, a seeded *recipe* in which every card of every seat — the
commander included — is a function of `cfg.seed`, so a perturbed seed re-rolled all four
decklists and all four commanders. CR 903.6 puts the commander in the **public** command
zone, which is why the playtester saw it on three opponents at once.

### The brief's fix was implemented, measured, and replaced — read this before re-proposing it

The brief said: factor `setup.rs`'s deck-resolution block into `resolve_decks(cfg)`, have
`session::new_game` store `DeckSource::Fixed(resolved)`. That was built first. **It reddens
seven tests**, and the reason is structural rather than incidental: `build_initial_state`
draws a seat's deck and then shuffles *that seat* before drawing the next seat's deck, all
off one `StdRng`, so **seat 2's decklist depends on seat 1's shuffle**. Any two-pass
factoring moves the stream, and moving the stream re-rolls every table every existing seed
builds. Measured, not predicted:

* six `tools/play-server` probes that pin card names at `SEED = 0`
  (`test_get_game_returns_seat_view_with_seven_card_hand`,
  `…pass_priority_advances_and_bots_act`, `…no_other_hand_card_names`,
  `test_ui3_combat_view_…`, `test_x_value_is_forwarded_…`, `…illegal_target_returns_422`);
* `local_game_playthrough` seed 1, which landed on a deck holding an **Aura** and died on
  `"engine rejected a just-offered action (CastSpell): Aura spells require exactly one
  target (CR 303.4a)"` — a **pre-existing** engine/legal-action defect the new table merely
  exposed, unfixable inside a 0-engine-lines task and not something to paper over by
  changing the test's seeds. Filed as **OOS-SIM4-2**.

**Shipped instead: `setup::dealt_decks(&state, &cfg)` (`setup.rs:238`)** — read the
decklists that were *actually dealt* back out of the built `GameState` (hand ∪ library, plus
the registered `commander_ids`). `session::new_game` (`session.rs:237`) builds from the
unmodified cfg, then pins the result: `:240` `dealt_decks` → `cfg.decks = Fixed(dealt)`.
This moves **no table at all** (all six SEED-0 pins stayed green, which is the evidence),
and it is the stronger guarantee: the multiset a mulligan permutes is the one the player was
literally dealt, not one re-derived from a config believed to agree. `setup::redeal` needed
**zero** changes — with `Fixed` decks its perturbed seed reaches only the shuffle.

### Gates

* `crates/simulator/tests/setup.rs:392` `test_redeal_preserves_every_seats_deck_and_commander`
  — the pin the brief demanded: for **every** seat, the 100-card multiset and the registered
  commander are identical across a redeal (with a 100-card non-vacuity floor), the
  command-zone *object* is still that commander, and seat 1's hand still changes and is
  still 7. Plus `:505` round trip, `:551` determinism + refusal, `:583` the shape floors.
* `tools/play-server/src/main.rs:7149` (P1, over the real router — two mulligans, all four
  public command zones compared) and `:7232` (P2, direct — the session *holds* `Fixed`, and
  every seat's hidden 100-card multiset survives). **Both proven red by executing the
  revert**: pre-fix P1 reports all four commanders replaced on mulligan 1.
* **The simulator gate alone could never have caught this** and it is worth knowing why:
  `DeckSource::Fixed` was always immune, so a simulator-level test passes whatever the play
  server chooses to store. The defect lived in what the *session kept*. A gate on the
  primitive does not gate the caller's choice of argument.
* `test_redeal_on_an_unresolved_recipe_still_rerolls_the_decks` (`:484`) deliberately pins
  the un-fixed path, so the caller's obligation is visible rather than folklore.

### Deferred, with reasons

* **Per-seat RNG streams** (the brief's optional residual): NOT implemented. Two reasons,
  both concrete. (1) Keying each seat's shuffle on `(seed, pid)` does not isolate anything —
  `redeal` perturbs `cfg.seed`, so every derived stream still moves; real isolation needs
  per-seat mulligan *counts* in the config, i.e. the per-seat pregame model that needs a
  decision channel for bot seats. (2) Re-deriving the shuffle seed at all would move the
  opening hands that `UI1_SEED`/`UI2_SEED`/`SIM1_SEED` pin **by original index** — five
  shipped CR flows rest on those fixtures.
* **OOS-G2-3** (dead `Command::TakeMulligan`/`KeepHand`, turn-0 gate never satisfiable):
  UNCHANGED, out of scope by the brief.

### Seeds filed

* **OOS-SIM4-1** — `setup::redeal` still accepts a `RandomPerSeat` config silently, so G2 is
  prevented by caller discipline, not by construction. `tools/tui/src/play/app.rs:132` builds
  exactly such a config and would reintroduce the defect verbatim the day the TUI grows a
  mulligan (a pointer comment now sits at that construction site). The structural fix is a
  `redeal` that takes the dealt state, or a `DeckSource` that cannot be a recipe past the
  first build.
* **OOS-SIM4-2** — `local_game_playthrough`'s policy submits a just-offered `CastSpell` for
  an **Aura** and the engine rejects it with CR 303.4a. Pre-existing (the seed-1 table only
  changed because of an experiment that was reverted), engine-side, and a genuine
  legal-action bug: an offered action must be applicable. Reproduce by two-passing
  `build_initial_state`'s deck/shuffle loop and running `--test local_game_playthrough`.
* **OOS-SIM4-3** — `dealt_decks` refuses a two-commander seat (CR 903.3 partner/background)
  because `DeckConfig` has one `commander` field. Correct today (nothing builds one), but it
  is the shape that will bite when partner decks arrive.

### Durable lesson

**A limitation documented by its mechanism does not warn anyone; document the consequence.**
Four separate doc blocks (`setup.rs`, `session.rs`, `api.rs`, `PlayApp.svelte`) described the
whole-table rebuild — one even named the commander re-roll — and none said "the players' decks
change". The playtester was the first to say it in those words. All four now do, plus
`tools/play-server/README.md`'s known-limitation 1 and its bug-report reproduction procedure,
which had quietly become **wrong** (rebuilding at the derived seed with the recipe no longer
reproduces a mulliganed table; it now takes base-seed build → `dealt_decks` → rebuild at the
derived seed with `Fixed`).

## Worker Handoff (UI-4, `scutemob-185`) — picker Confirm hotfix

**G1 CONFIRMED IN A BROWSER BEFORE ANY EDIT, and the triage's diagnosis was exactly right.**
Headless Chromium (playwright-core, `~/.cache/ms-playwright/chromium-1228`) against a live
`play-server` on **:3041**, seed 116 driven over HTTP to a Three Visits `SearchLibrary`
decision (the Farhaven Elf class), then three clicks in the browser:

```
[pageerror] DataCloneError: Failed to execute 'structuredClone' on 'Window':
            #<Object> could not be cloned.
    at z (…/assets/index-BIZqizQa.js:3:13218)      <- SearchPicker.emit
    at HTMLButtonElement.O (…:3:13344)             <- confirm()
    at HTMLDivElement.za (…:1:30189)               <- Svelte's delegated handler
```

Picker stayed open, **error-strip count 0, zero POST requests**, `command_count` still 171 and
`seq` still 291 after the click. "Did nothing" was literal.

**Fix**: new `frontend/src/lib/plainClone.svelte.js` (a `$state.snapshot` wrapper) called at all
three sites. `$state.snapshot` over `JSON.parse(JSON.stringify(…))` deliberately: it does not
re-serialize, so it cannot coerce anything the wire put there, and it deep-copies plain objects
too — which matters for the harness proposed below, whose fixtures pass non-reactive values.

**Error surfacing, two mechanisms, both demonstrated live by fault injection** (faults reverted;
`dist` rebuilt clean, bundle hash back to `index-CsOwI2Ah.js`):

| Fault | Path | Rendered |
|---|---|---|
| throw inside `SearchPicker.emit` (guarded) | picker `try` → `onError` → `ActionBar.onPickerError` → `PlayApp` → `stores.reportClientError` → strip | *"Something went wrong in this browser — the game is unchanged / could not submit the search answer: injected UI-4 fault"*; chain closed, 0 POSTs |
| throw inside `SearchPicker.select` (**no** `try`) | `window` `error` listener armed by `main.js` | *"unhandled RangeError in the client: …"* |

The second is the load-bearing one: five pickers have no `try` and never will have one for every
handler, so the `window` listener is the guarantee and the per-picker `catch` is only a better
message. **Svelte 5's `<svelte:boundary>` is not a substitute** — it catches render and effect
errors, not DOM handler ones. Checked, not assumed.

**Gates (2, both proven red by executing a revert, then green again)**, in
`tools/play-server/src/main.rs` so they run under `cargo test --all` and therefore CI:

* `test_frontend_never_structured_clones_reactive_state` — walks every `.svelte`/`.js`/`.css`
  under `frontend/src/` (skipping `node_modules/`, `dist/`), bans `structuredClone(`,
  `.postMessage(` and `indexedDB`. **Proven by reverting `SearchPicker`'s clone.** Four
  non-vacuity arms, because a ban with zero permitted uses is the pinned-empty-roster shape:
  named files **and** a ≥14-file floor on the walk; each picker must *import and call*
  `plainClone` (a picker that stopped copying at all would satisfy the ban while mutating its
  parent's state); the helper must really be `$state.snapshot`; and the matcher is fired at a
  synthetic offending line so a typo in a needle cannot hide.
* `test_frontend_picker_failures_reach_the_error_strip` — pins both mechanisms **and the call
  that arms the second**. **Proven by deleting `main.js`'s `installGlobalErrorReporting()`
  call**, which is the failure mode a "does the module export it?" test would have missed.

**All five CR flows verified end to end, each with a NON-DEFAULT answer** so game state
distinguishes the human's choice from the engine's default — the property the whole defect class
turns on:

| Flow | Setup | Posted | Observed in game state |
|---|---|---|---|
| library search CR 701.23 | seed 116, Three Visits | `{SearchLibrary:{found:24}}` | **Dryad Arbor** on the battlefield, not the default `candidates.first()` Forest |
| scry CR 701.22a | seed 28, Preordain (scry 2) | `{Scry:{bottom:[99],top:[98]}}` | drew **Reverse Engineer**, the *second* card; the default order draws the Island |
| surveil CR 701.25a | seed 28, Consider | `{Surveil:{graveyard:[99],top:[]}}` | **Island in the graveyard** |
| sacrifice CR 118.8 | seed 29, Harrow | `{Sacrifice:{ids:[437],lki:[]}}` | battlefield `402,419,437` → `402,419`; **437 chosen over the server default 402** |
| Squad CR 702.157a | seed 1364, Galadhrim Brigade | `{Squad:{count:1}}` | **token copy #486** beside the real #482; template default is `count: 0` = decline |

Zero `pageerror`s and zero error strips across all five.

**Scope**: 9 source files under `tools/play-server` plus 4 doc files; **0 engine lines**
(`git diff main..HEAD --numstat -- crates/` is EMPTY — no engine *and* no simulator), 0 wire
changes — no `Command`/`GameEvent`/`Effect` variant, so PROTOCOL and HASH are untouched by
construction and were not recomputed. (Reconciliation note for the collector: the implementation
commit is 9 files; the branch total is 12+, the difference being `CLAUDE.md`, the play-server
README and this file.) Workspace:
**4,265 passing / 0 failing / 5 ignored** (`--workspace --no-fail-fast`, captured to a file, not
piped to `tail` — 2026-08-02 lesson), which is +2 on main's 4,263 and those 2 are these gates.
`cargo fmt --check`, `tools/check-defs-fmt.sh` (1,803 defs) and
`clippy --workspace --all-targets -D warnings` all clean.

### The R7 frontend test harness — a concrete proposal, deliberately NOT built here

R7 (`memory/m11-session-plan.md` §8) is the debt this defect collected on, and the triage is right
that it is overdue. It is **not** built here because this task had to be small and go first. What
follows is sized from having actually done both halves by hand today.

**Tier 1 — component tests (vitest + jsdom + `@testing-library/svelte`).** 3 devDeps, an
`npm test` script, a `vitest.config.js` reusing the existing `svelte.config.js`, one spec per
picker (8) ≈ 400-600 lines. **The one rule that makes or breaks it: a fixture MUST wrap the
template in `$state()` before passing it as a prop.** A spec that hands a picker a plain object
would have passed green against the broken code — that is precisely why UI-1's and UI-2's HTTP
probes proved the channel and nothing about the component. Reproduce the *reactivity*, not just
the shape. Write that rule into the harness's own module doc, because it is the only part a
future author can get wrong while believing they have covered the bug.

**Tier 2 — real-browser scenarios (`playwright-core`, ~30 lines of setup).** Exactly the shape
used for today's verification, and worth keeping: drive the game to the target decision **over
HTTP**, then do only the last few clicks in the browser. Cheap, no component framework, and it
catches what jsdom structurally cannot — the `DataCloneError` is real-browser structured-clone
behaviour and a jsdom polyfill may not reproduce it. Tier 1 without Tier 2 could have missed this
exact bug.

**The real cost is fixtures, and it is bigger than the harness.** Reaching a scry / surveil /
Squad decision meant scanning **~2,400 seeds** through `POST /api/game` to find an opening hand
holding the right card, because `session.rs:165` hard-codes `DeckSource::RandomPerSeat`. Two
routes: (a) cheap and immediate — commit the tuples below under `test-data/`; (b) the real fix —
let `POST /api/game` accept a fixed decklist so a scenario names its cards instead of hunting a
seed. Recommend (a) now, (b) when someone touches `session.rs` anyway. **Known-good tuples,
handed over so nobody re-scans**: seed **116** → Three Visits (`PickOne`, 33 candidates incl.
Dryad Arbor as a distinguishable non-default); seed **28** → Preordain *and* Consider, two
Islands (`Partition`, both scry 2 and surveil 1 in one game); seed **29** → Harrow, 3 Forests
(sacrifice-a-land `SacrificeCostView`, no creature needed); seed **1364** → Galadhrim Brigade,
5 Forests (Squad `max_count: 1`). Squad is the scarce one — **1 seed in the first 600**, and
`Ultramarines Honour Guard` at 6 mana is not reachable before the human dies, so use Galadhrim
Brigade. Driver scripts are in this session's scratchpad only, not committed; they are ~150 lines
and trivially rewritten from this paragraph.

**CI note, flagged not fixed**: the workflow is a single Ubuntu **cargo** job. Tier 1 needs an
`npm ci && npm test` step and a Node toolchain in that job. Today's two gates need neither —
that is why they are Rust source gates and not a JS lint.

**What today's gates do NOT cover, stated plainly**: they prove the pattern is absent and the
error wiring exists. They cannot prove a picker *renders*, that a template is read correctly, or
that an answer is *right*. `SearchPicker`'s and `PartitionPicker`'s "# Untested" module sections
are still accurate and were left alone.

**`/review` fix cycle — 5 LOW, all 5 taken rather than deferred** (each was a few lines, and two
were real coverage holes):

1. **The gate had a blind spot exactly the size of the shared component library.** It walked
   `frontend/src/` only, but `vite.config.js` aliases `$viewer` →
   `tools/replay-viewer/frontend/src/lib`, imported **in place** and compiled into *this* bundle.
   The walk now covers both, with its own named-file + ≥8-file floor because a `..`-relative path
   is the arrangement most likely to resolve to nothing after a move. **Proven by appending a
   forbidden call to `cardTooltip.js`** — red, naming that file; restored, green. Zero real hits
   today, so this is coverage, not a repair — but the test's own "this is a class, not three
   sites" claim was overreaching by one directory until now.
2. **The silent bail-outs survived.** All three pickers kept malformed-template guards that
   `return` without reporting. Those are *returns*, not throws, so 6048's literal wording was
   already satisfied — but the **symptom** (click Confirm, nothing happens, no message) is the
   thing this task exists to eliminate, and it should not survive from a second cause. All **six**
   sites now report through `onError` before bailing — `SearchPicker` ×2, `PartitionPicker` ×2,
   `CostPicker` ×2 (the two `!entry` checks, which absorb `fillTemplate`'s own two internal
   `return null` paths). Three `onError?.(` calls per picker: two guards plus the `catch`.
3. **`main.js`'s comment overclaimed.** It said arming the net before mount surfaces "a throw
   during the very first render"; the strip lives inside `ActionBar`, so such a throw sets the
   store and renders nothing. Comment narrowed to what is true.
4. **The weakest gate arm matched prose.** The per-picker check was `contains("onError")`, which
   the prop's own doc comment satisfies — a picker that documented the prop and never called it
   would have passed. Anchored on `onError?.(` instead. **Proven by renaming CostPicker's calls**
   — red; restored, green.
5. Count mismatch between CLAUDE.md and this handoff reconciled (above).

All three picker types re-verified in the browser **after** these edits (search → Dryad Arbor;
surveil → Island to graveyard; sacrifice → 437 over the default 402), so the fix cycle did not
regress the thing the fix cycle was protecting.

**Seed**: `OOS-G1-1` (the structured-clone-on-Svelte-state class) is **CLOSED by this task** —
fixed at all three sites and machine-gated against recurrence. Not filed as open in
`docs/audits/decision-point-audit.md` §8.1 for that reason; the gate is the durable artefact.
`OOS-G8-1`-adjacent note for whoever takes **UI-5**: G8's "Concede was the only live control"
premise is now false — the answer button works — so it reverts to an ordinary UX item, exactly as
the triage predicted.
## Worker Handoff (PB-DX19, `scutemob-184`)

**Scope**: the v3 queue's first dispatch — `OOS-SIM2-6` (HIGH) + `OOS-SIM2-5` fold-in.
Plan: `memory/primitives/pb-plan-DX19.md`. Review: `memory/primitives/pb-review-DX19.md`.
Brief: `memory/primitives/seed-rerank-2026-08-02.md` §4, "Dispatch briefs" → PB-DX19.

**Shipped**: `ee7a55b4` (stage-0 repro), `a0d977e5` (fixes), `79b94a58` (tests + deviation pin).
PROTOCOL **33** / HASH **70** gate-executed, both unmoved. Tests **4,274 / 0 / 5** on branch
(+11 over main's 4,263). `clippy -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh`
all clean. Coverage unmoved, proven by regeneration (see "claims" below).

### What actually broke, and why it took 4.5 months

The cycle is `calculate_characteristics` → `is_effect_active` → `check_static_condition` →
the `YouControlNOrMoreWithFilter` arm → `expect_characteristics` → back. **The seed's own
description of it was wrong in a way that matters.** It reads as a property of *counting
artifacts* with *that permanent on the battlefield*. It is neither: `calculate_characteristics`
calls `is_effect_active` on **every** `state.continuous_effects` entry, whatever object it was
asked about and whatever zone that object is in. So the recursion runs through the **effect**,
not the object. Two probes prove it — one calculating the Archangel's *own* characteristics (it
is not an artifact; the grant can never reach it) and one with Metalcraft **off** (the condition
is *false*) — and both crashed identically pre-fix.

That distinction is the whole story of the 4.5 months. The in-source comment argued termination
from exactly the disproved invariant ("we are checking the types of *other* battlefield
objects"), and then proposed the correct fix as a **performance** note. Anyone who read it came
away reassured. **The comment, not the code, was the defect that survived** — so the batch
rewrote it with the mechanism, and that rewrite is worth more than the one-line code change.

### Durable lessons

1. **A termination argument in a comment is a claim, and claims rot.** This one was never true.
   If a comment says "this recursion is safe because X", the test that proves X is missing.
2. **A test that names a card by string proves nothing about that card.** `static_grants.rs`
   named Indomitable Archangel and then hand-built the effect with `condition: None` — it
   exercised the filter and never the condition, and the condition was the entire defect. Repaired
   to drive the real def through `register_static_continuous_effects`; it now aborts pre-fix where
   the old shape passed. **That is the test to write: one that fails for the reason you claim.**
3. **`as` casts are not checked arithmetic.** `overflow-checks` does not touch them. The one
   OOS-SIM2-5 site no fuzz profile could ever have caught was a `u32 as i32` counter widening that
   wrapped the counter's **sign** in every profile. Its probe fails by *assertion*; the other five
   fail by *panic*. If a hardening pass converts `+=` to `saturating_add` and leaves the `as`
   casts, it has hardened the sites that were already loud and skipped the silent one.
4. **A stack overflow is not a test failure.** It is signal 6, names no test, and takes the binary
   down: `cargo test` printed `running 3 tests` and named exactly one. Filed `OOS-DX19-4` — a
   depth tripwire would have made this a named debug failure in 2026-03.
5. **`cargo fmt` passed a card-def edit that `tools/check-defs-fmt.sh` rejected.** SR-35 caught a
   real one here, not a hypothetical.

### Claims, and how each was actually established

- **The mandatory experiment is decisive, and was run pre-fix at the real pre-fix tree** (not with
  the card's static commented out — the brief's control does not survive the fix landing).
  `mtg-fuzzer --games 15 --seed 1`, `[profile.fuzz]`: **pre-fix** `fatal runtime error: stack
  overflow` → SIGABRT, exit 134, **0 of 15** games completed; **post-fix** 15 completed, 4.6s,
  avg **189** turns, 12 wins / 3 errors. **This closes `OOS-DP3-9` / `OOS-M11-3`'s stack-overflow
  half** — and note the abort was *immediate*, so that row's "game-count- or game-length-dependent"
  reading was an artefact of which decks the seed drew. `OOS-M11-3`'s **determinism** half is
  untouched and stays open.
- **Every pre-fix failure was OBSERVED via an executed revert, and both reverts compiled** (S8's
  lesson: the first revert of that batch did not compile, so it proved nothing). Two independent
  reverts were run: P/T-only (6 probes fail, 3 recursion probes pass) and recursion-only (6 P/T
  probes pass, recursion probe SIGABRTs). The isolation is the point — it shows neither fix is
  carrying the other's evidence.
- **Coverage unmoved was proven by REGENERATION, not by an empty diff.** The card-defs diff is
  *not* empty: the brief mandated the `greymond_avacyns_stalwart` note edit. So the claim rests on
  `tools/authoring-report.py` producing a byte-identical report body (only the self-dating header
  and recent-commits list differ). The generated docs were then reverted, since the numbers moved
  by nothing and committing them is pure churn.

### The mistake this batch made, and what caught it

**The first fix was a HIGH regression, and the tests could not see it.**
`check_static_condition` is a *shared* evaluator with five callers; only `is_effect_active` (inside
`calculate_characteristics`) closes the cycle. Reading `obj.characteristics` unconditionally fixed
that one and broke the other four, all of which CR 613.1d requires to be layer-resolved:
`garruks_uprising`'s `min_power: 4` intervening-if stops firing on a 2/2 with two `+1/+1` counters;
`bloodline_keeper` rejects a changeling (CR 702.73a expands types *inside* the layer loop);
`mox_opal` **over**-counts a face-down manifest (CR 708.2a — printed types are still the hidden
card's, so this one is a false *positive*, the direction nobody thought to look for).

**All 4,274 tests passed through it.** Not because coverage is thin, but because no existing test
put a counter-pumped or type-changed permanent through a condition filter — the fixture creatures
are plain vanilla bears. A green suite is evidence about the scenarios someone thought to write.

The lesson is not "be careful". It is: **when you change a function, enumerate its callers before
you decide what the change means.** The recursion was a property of ONE call path, and the fix was
applied to the function. `rules::layers::characteristics_for_condition` is the repair — a
re-entrancy guard that decides per caller — and because it decides by shape rather than per site, it
also closed `OOS-DX19-1`'s ten siblings, which the leaf-edit fix would have got wrong in the
opposite direction (several are *correct* as layer-resolved on their real paths).

### What the next worker should know

- **The fix has a known, live cost, and it is asserted in the wrong direction on purpose.**
  `blinkmoth_nexus` / `inkmoth_nexus` are `Complete`-by-derive **colourless** lands that animate
  into **artifact** creatures (Layer-4 `AddCardTypes`), so they share a deck pool with the
  Archangel and an animated Nexus no longer feeds Metalcraft — CR 613.1d says it must.
  `deviation_animated_nexus_does_not_count_toward_metalcraft` pins that, and its message tells you
  to **invert** it rather than delete it when `OOS-DX19-2`'s CR 613.8b fixpoint lands.
- **`OOS-DX19-1` is CLOSED, and the second review is why.** The first routing pass claimed the
  closure while three sites were still unconverted — they spell the call
  `expect_characteristics(state, id)` instead of `(state, obj.id)`, so a pattern-replacement walked
  straight past them, and the reviewer reproduced the original SIGABRT through one. **The durable
  lesson: a closure achieved by editing every site you could find is a claim; a closure backed by a
  gate that fails when a site reappears is a fact.** The gate is
  `no_condition_evaluator_resolves_characteristics_directly`, watched failing on a re-introduced
  miss. Ten more `expect_characteristics` sites in
  `check_condition` are the identical shape and are latent **only because of corpus shape** — all
  **57** corpus occurrences of those ten variants were enumerated and classified by field
  position: every one is an `activation_condition`, `unless_condition`, `intervening_if`, or a bare
  `Effect::Conditional`, and **none** is a `ContinuousEffectDef.condition` — which is the only
  field `is_effect_active` reads. (Do not restate this as "all ~98 `condition: Some(..)`
  occurrences are off the layer path"; that is false — 17 of them *are* continuous-effect
  conditions, `indomitable_archangel`'s among them. The claim is about the ten variants
  only.) The next author who writes "as long as you control a legendary creature, …" as a **static**
  reopens a HIGH with no warning. **Do not fix it by converting the ten leaves**: several are
  *correct* as layer-resolved on their real call paths (there is a `// CR 613.1d … Blood Moon`
  comment saying so). It wants a boundary guard.
- **PB-DX22 must still follow this batch**, per the brief's own sequencing note — shuffling the
  fuzzer makes spells castable at ordinary depths. That constraint is now satisfied.


## Worker Handoff (SEED RE-RANK v3, `scutemob-182`) — doc-only

**Deliverable**: `memory/primitives/seed-rerank-2026-08-02.md`. **This is now the authoritative
primitive queue.** `seed-rerank-2026-07-27.md` §4 is banner'd SUPERSEDED; its §1-§3 remain
canonical. `git diff` confined to `memory/`, `docs/`, `CLAUDE.md` — zero source lines.

**The census is twice the brief's estimate, and the reason is a cutoff.** The brief scoped ~40
seeds (DX6 + the 174-181 run). The real post-2026-07-27 population is **80 rows / 79 distinct
IDs**. v2's census closed **2026-07-31**; every PB-DX batch shipped **2026-08-01** — so the
document that ranks PB-DX1..DX18 has never seen the 29 seeds PB-DX1..DX5 filed, nor
`OOS-M11-5..10`. Four of those 29 are live-wrong on deck-legal `Complete` cards.

**No single source is complete, and a future re-rank must run all three.** Pass A
(workstream-state handoffs) misses **20** rows — the PB-DX1..DX4 handoff sections are rotated out
and only the L18 W6 mega-row survives, naming DX1's seeds not at all. Pass B (the 2026-08 archive)
misses `OOS-M11-5` entirely and records almost every filing as an unresolvable range. Pass C
(`docs/audits/decision-point-audit.md` §8.1, the registry) misses **10** — the CARDS-2 family lives
in `memory/card-authoring/cards2-field-fidelity-2026-08-02.md` under
`## 5. Cross-references and seeds` (**§5**, not §7 — that doc has five sections), and only
`OOS-CARDS2-9` is in §8.1. Wildcards resolved, and two written ranges found stale
(`OOS-DX5-1..7` under-reports by one — `OOS-DX5-8` exists and neither narrative mentions it). **UI-1 (`scutemob-174`) filed zero seeds**;
there is no `OOS-UI1-*` family anywhere.

**Next dispatch is PB-DX19, not PB-DX7.** `OOS-SIM2-6` — the registry's only self-declared HIGH —
was walked hop by hop and confirmed: `layers.rs:35`→`:46` `is_effect_active` → `layers.rs:565`
`check_static_condition` → `effects/mod.rs:10259` `expect_characteristics` → `layers.rs:478`
`calculate_characteristics`. **Unconditional**, because the arm evaluates every candidate
permanent *before* the `exclude_self` test at `:10266` and the source is itself a candidate.
`indomitable_archangel.rs` declares **no `completeness` field** → `Complete`, `validate_deck`
accepts it, `random_deck` pools it for any W-identity seat. Result: `stack overflow` → SIGABRT,
not `catch_unwind`-able, so the play-server cannot contain it. **The class is exactly one card**,
measured two ways (17 of 380 `ContinuousEffectDef`s carry a `condition`; exactly one uses a
recursion-capable variant). **The fix is one line** — and `layers.rs:2291`'s
`EffectAmount::PermanentCount` already made that exact choice for that exact reason
(`:2304-2310`). Live **4.5 months** (`d83ac94d` 2026-03-12 / `aa23d26c` 2026-03-23).

**Two lessons worth carrying, both about why it survived.** (1) `effects/mod.rs:10245-10256`
argues termination from the wrong invariant — "we are checking *other* objects" — when the
recursion is on the *effect*, re-collected at `layers.rs:46` on every nested call; it even proposes
the correct fix as a **performance** note. *A safety argument written next to the code it excuses
is not evidence.* (2) `crates/engine/tests/rules/static_grants.rs:711-760` names Indomitable
Archangel and hand-builds the effect with `condition: None` at `:736`. *A test that names the card
while dodging the field is worse than no test — it reads as coverage.* And a landmine:
`greymond_avacyns_stalwart.rs:38-43` instructs a future author to build a second instance.

**Four seeds filed "latent" are live-wrong.** `OOS-DX1-3` (`nether_traitor`),
`OOS-DX2-5`/`-2`/`-7` (`golgari_grave_troll`), `OOS-DX4-2` (`retreat_to_kazandu`), `OOS-DX4-6`
(**all ten Karoo bounce lands** + two more — scope ×7, and the deviation is exploitable *in the
controller's favour*). **The tempting explanation is wrong and the true one is worse.** The
`#[default] Completeness::Complete` derive — PB-DX1's and PB-DX3b's lesson — accounts for **five**
of the eight live-wrong defs this census caught (`golgari_grave_troll`, `retreat_to_kazandu`, the
ten Karoos, `sigil_of_sleep`, `indomitable_archangel`). The other three (`nether_traitor.rs:60`,
`qarsi_sadist`, `voldaren_epicure`) declare `completeness: Completeness::Complete` **explicitly** —
a one-line grep would have found them. So the shared mechanism is not the derive; it is that **the
latency claim was never checked against the corpus at all**, in three cases not even by the
cheapest possible check. Saying "the derive did it" would let a future triage think that grepping
the explicit marker is sufficient diligence, and for three of these eight it would have been.
**965 of 1,803 defs never declare a marker** (re-measured; PB-DX4 said 966 the day before). Filed as
`OOS-RR3-1`. **Binding for every future batch: a latency claim is not verified until the corpus
has been enumerated — over `all_cards()` where possible (SR-36), missing marker treated as
`Complete` — and "no def does X" is not a finding until someone has actually looked.**

**Other findings that moved a rank.** `OOS-CARDS2-4` — the offer layer cannot see a
`KeywordAbility::Enchant`-carried target requirement (`legal_actions.rs` has zero occurrences of
`Enchant(`/`target_min`; `target_count_range` iterates `TargetRequirement` only), so **13
deck-legal `Complete` Aura defs** 422 on first human contact and the only reason the suite is green
is `KNOWN_FALSE_OFFERS`. `OOS-M11-9` — no once-per-combat guard, and the consequences are three,
not one: attackers **accumulate** (`combat.rs:743` inserts into a map), every attack trigger
**re-fires** (`:795-805`), and the raid count is **clobbered** (`:759`); the blocker side already
has the guard (`:1103`). `OOS-UI2-1` and `OOS-SIM3-1` **reconcile arithmetically** — no opening
hand + 34 basics on top ⇒ first non-land at personal draw ~35-40 ⇒ game turn ≈136-156; "never
casts" is `--max-turns 80`, "casts from turn 143" is the default cap. Quote the cap alongside any
fuzz-parity claim.

**Closures: 11 verified in code, and one closed *further* than recorded.** `OOS-UI2-3`'s third
cause was `OOS-M11-2`'s `can_afford` pool-OR-sources split, which SIM-2 also closed
(`legal_actions.rs:1752-1757` is now one `solve_mana_payment_with_pool` call) — so `OOS-M11-2`'s
residue is **cost MODIFIERS + CR 106.12 restricted mana only**, smaller than CLAUDE.md said.
**Three rows are design records, not work** (`OOS-DX5-2`, `OOS-DX5-6`, `OOS-DX6-3`) — ranking them
wastes a slot. **Six merges** recorded, incl. `OOS-SIM1-2` ≡ `OOS-SIM2-7` (literally the same two
TUI lines) and `OOS-CARDS2-11` ⊂ `OOS-CARDS2-8`.

**Two hard sequencing constraints, derived not asserted.** (1) **PB-DX19 must precede PB-DX22** —
shuffling the fuzzer makes spells castable at ordinary depths, turning the Archangel's turn-191
abort into a routine one that will read as a regression caused by PB-DX22. (2) PB-DX22 and every
card-def batch re-roll every recorded seed (`OOS-CARDS2-3`: `random_deck` indexes a corpus-ordered
vector, so correcting a *type line* re-deals every seeded game, and **no gate exists**) — batch the
card-def work and land the pool-size gate first so the re-deal announces itself.

**Not done, deliberately**: the `mtg-fuzzer --games 15 --seed 1` A/B with and without the
Archangel's static (would settle whether `OOS-SIM2-6` is `OOS-DP3-9`/`OOS-M11-3`'s stack-overflow
mechanism — "very likely" is the honest strength until it runs) and the first-`ZoneId::Stack`
re-measure at HEAD (SIM-1's command-zone loop should put a spell on the stack ~120 turns before
SIM-3's turn-143 measurement; either SIM-3 measured a pre-SIM-1 build or something suppresses the
bot offer). Both are assigned into PB-DX19's and PB-DX22's plans respectively. This task was
doc-only and ran no cargo.

## Worker Handoff (SIM-3, `scutemob-177`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (SIM-3) — **F6 CLOSED**; playtest triage is
now **fully closed** (`memory/playtest-triage-2026-08-02.md`: OPEN = none)
**Task**: `scutemob-177`. Branch
`feat/sim-3-stackconsistency-invariant-is-a-false-positive-by-cons`

**The task was re-scoped before it started, and the re-scope was half right.** The
coordinator's own comment said M11-local S8 had already rewritten the check with the same
diagnosis, so the task shrank to "tests + one doc line". Two of those three residuals were
as described (no test module; `docs/mtg-engine-simulator.md` still wrong in prose). The
third was not: `docs/mtg-engine-runtime-integrity.md` was **also** still wrong — S8 had
corrected neither. And the rewrite itself carried a residual false positive.

**Completed**:

1. **The finding: the S8 rewrite classified on `StackObjectKind::Spell` alone, and its
   stated premise for doing so is false.** Its doc block asserted that the four engine
   sites that move an object into `ZoneId::Stack` "all end in that same `Spell` kind".
   `casting.rs::handle_cast_spell` moves the card at `:4399` and *then* branches on
   `cast_with_mutate` at `:4504`, so a **mutate** cast (CR 702.140a / CR 729.2) puts a card
   in the Stack zone under a `MutatingCreatureSpell` kind — and the check reported it as an
   orphan, on every such cast, in a game with nothing wrong with it. **Generalisable, and
   it is the same shape as `OOS-SIM1-3` and `OOS-SIM2`'s fix-cycle finding one and two
   batches earlier: an enumeration is only as complete as the category it names.** Here the
   category was "kinds that obviously put a card on the stack", read off variant names.
   Classification is now `invariants::stack_card_of`, an **exhaustive match over all 27
   variants** — adding a `StackObjectKind` is a compile error until someone classifies it,
   the forcing function SR-5 already applies to `KeywordAbility`.

2. **Two properties the old set comparison could not express, both added and both
   measured.** Property (3) closes **MR-M11-14** (LOW, deferred): no two non-copy stack
   objects may claim the same card, CR 400.7. Its deferral had asked for a measured run
   before widening the check — this batch was already measuring, so it could pay that
   price, and `memory/m11-fix-session-plan.md` is updated to `[x] DONE`. Property (4) is
   order: the Stack zone's contents are the card-owning stack objects' cards read in stack
   order, ability and trigger entries skipped. Structurally guaranteed (`Zone::Ordered`
   inserts at the back only; every entry site pairs the zone move with a
   `stack_objects.push_back`; every removal takes the pair; CR 608.2d suspension restores
   `restart_point` wholesale) — the `/review` verified that argument independently rather
   than accepting the 10 clean runs as proof.

3. **Measured A/B**, old check restored verbatim from `222ff84f^`, same builds, same seeds:
   `local_game_playthrough` seed 1 **720 → 0** (638 + 82 by direction; the test fails on the
   first seed with the old check, passes on all five with the new); `mtg-fuzzer --games 5
   --seed 1 --max-turns 200` **8,781 → 0** (7,575 + 1,206). Every other check byte-identical
   across the A/B (929 `no_orphaned_tokens`, 9 `player_consistency`), so the measurement
   moves this check and nothing else. **8,781 of that run's 9,719 violations — 90.3% — were
   this one check being wrong.**

4. **Ten probes in a new `#[cfg(test)] mod tests`** (the file had none across 306 lines),
   every one watched failing under a deliberate revert: a **9-revert matrix** in which
   R1–R7 each fail exactly one test and R8/R9 cover the over-firing direction. T2 pins the
   pre-S8 check's two-per-spell false positive as a historical record in code.

**Durable lessons**

- **A redaction of a false positive is not the same as a proof of the truth.** S8 removed
  501 false positives and the number went to zero, which read as done; the *reason* it
  went to zero was that no seed in its evidence cast a mutate spell. Zero is not a proof
  when the population is thin.
- **`OOS-UI2-1` is right about the mechanism and wrong about the horizon** —
  **`OOS-SIM3-1`**, and the most reusable thing here. UI-2 measured 5 games × 80 turns, saw
  no non-land in hand, and concluded "every fuzz parity claim in this project's history is
  a claim about a land-only game". At the fuzzer's **default** `--max-turns 200`, **150
  distinct cards reached `ZoneId::Stack` across 5 games, the earliest on turn 143** — the
  basics run out and real spells start resolving. The deck-order defect is real and
  unchanged; its consequence is a **threshold**, not an absolute. Any future fuzz A/B
  should say which side of turn ~140 it lives on.
- **Every "N violations" figure this project has quoted is checkpoint-weighted**
  (**`OOS-SIM3-3`**): `check_all` re-reports a condition that is still true at every
  command. Measured: 929 `no_orphaned_tokens` reports = **183 distinct tokens**; 9
  `player_consistency` reports = **1 condition**. The inflation factor is not constant, so
  those totals are not comparable to each other.

**Seeds filed** (`docs/audits/decision-point-audit.md` §8.1): **OOS-SIM3-1** (the horizon
qualification above), **OOS-SIM3-2** (two of the twelve documented invariant checks have
never been written — legal-action soundness and SBA idempotency — a third is a no-op, and
`runtime-integrity.md`'s parallel list has four that do not exist), **OOS-SIM3-3**
(checkpoint-weighted totals), **OOS-SIM3-4** (`no_orphaned_tokens` is now the next noise
floor at 929 of the 938 remaining, and OOS-M11-7 says they are *expected* — so the fuzzer
still is not a clean smoke test, for a new reason), **OOS-SIM3-5** (`/review`:
`Effect::CounterSpell` drops `MutatingCreatureSpell` into its `_ =>` arm after already
removing the stack object, stranding the card in `ZoneId::Stack` forever; and countering a
**copy** moves the *original's* card. Both are engine defects that will legitimately trip
this check, neither is in this batch's evidence — **read the next one as a real finding,
not a SIM-3 regression**).

**Also updated**: `OOS-DP3-9`'s `stack_consistency` half **WITHDRAWN** with the A/B
attached (its stack-overflow half stands and now has `OOS-SIM2-6` as a named mechanism; its
`crash-reports/.gitignore` rider re-checked and closed — `.gitignore:52`). The
`memory/workstream-state.md` bot-parity bullet that cited `stack_consistency` line ordering
as evidence of `OOS-M11-3` nondeterminism is annotated as **voided** — it was ordering of
noise; the byte-identical Turns/Commands/Winner/Error result is what that claim rests on.

**Gates**: tests **4,247 → 4,257 / 0 / 5** (+10, exactly the probes). `cargo fmt --check` +
`tools/check-defs-fmt.sh` (1,803 defs) clean; `clippy --workspace --all-targets -D
warnings` clean; `cargo build --workspace` clean (the SR-3 seal gate — the probes use the
`test-util` escape hatches and are `#[cfg(test)]`, so the seal holds). **PROTOCOL 33 /
HASH 70 gate-EXECUTED unmoved** (the criterion's "32" is stale, as UI-1/UI-2/SIM-2 also
found); `decision_gate` 18/18. **Zero engine lines** — the only source file in the diff is
`crates/simulator/src/invariants.rs`.

**Not done, deliberately**: `OOS-SIM3-5`'s two engine defects are left unfixed
(`crates/simulator`-only batch). `OOS-SIM3-2`'s two missing checks are marked in both docs,
not written — #10 (legal-action soundness) is the SR-38 property and is close to free, since
`GameDriver` already distinguishes a rejected command from an applied one.
## Worker Handoff (UI-3, `scutemob-180`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (UI-3) — the UX/layout items the triage
filed under **"Not verified (by design)"**, i.e. feature work rather than claims
**Task**: `scutemob-180`. Branch
`feat/ui-3-play-frontend-ux-polish-batch-playtest-notes-layoutinfo`

**Completed** (all five criteria; 0 engine lines — `git diff main -- crates/engine/src
crates/card-types/src crates/card-defs` is empty):

- **AC 6006 — combat display.** The headline is that **nothing was missing from the
  payload**. `StateViewModel::combat` has carried `attackers[].target` and
  `attackers[].blockers[]` since M9.5, seat-redacted by `redact::redact_combat`. The play
  client rendered `$viewer/StateView.svelte`, which **does not include**
  `CombatView.svelte` — the replay viewer composes those two in its own `App.svelte`, so
  the component existed, the data existed, and the two had never been introduced on this
  surface. `PlayApp` now renders it under the stack. **A real defect fell out of doing so**:
  `AttackerView::target`'s doc comment said `"planeswalker:<id>"` and
  `CombatView.svelte::formatTarget` believed it, rendering `PW #{suffix}` — so an attacked
  planeswalker displayed as **`PW #Chandra, Torch of Defiance`** in *both* surfaces.
  `build_combat_view` has always written a name, and `redact_combat` substitutes
  `FACE_DOWN_NAME` — a name — which is only coherent for a name field. Fixed in place in the
  shared component (deliberately: the replay viewer had the identical bug) and the doc
  corrected with it.
- **AC 6007 — event feed.** The feed was sparse **because the renderer was**:
  `event_view_for` had ~11 rendered arms and a `_ =>` catch-all emitting the bare serde
  variant name with no player and no card, and *every single item the playtest asked for*
  (taps, ETB, deaths, exiles, counters, triggers, resolutions, attacks, blocks, damage) fell
  into it. **49 prose arms** added, each identity routed through the `viewer_may_identify`
  gate — no arm reads `state.objects()`. `EventView` gains `tier`
  (`game`/`player`/`card`/`stack`), assigned **server-side** by a match on the variant with a
  documented `_ => Game` default. The client deliberately does **not** classify by `kind`
  substring: `GameEvent` has ~141 variants, and a stale client-side list would silently hide
  a whole class of event behind a filter chip. (It still substring-matches for *tone*, which
  only picks a colour — the distinction is written down.) `EventFeed` gains tier chips with
  live counts and collapsible per-turn sections; **section boundaries come from the
  unfiltered list**, because `TurnStarted` is itself a `game`-tier event and deriving them
  from the filtered one would make every turn heading vanish the moment someone unticked
  "turn".
- **AC 6008 — layout.** New play-local `PlayBoard.svelte` (2×2 battlefield grid via
  `repeat(auto-fit, minmax(22rem, 1fr))`, so four boards lay out 2×2 and two survivors reflow
  to full width **with no code branch on the count**) and `SeatCard.svelte` (command zone
  folded into the player card, expandable details drawer). The shared
  `$viewer/StateView.svelte` is **untouched** and the replay viewer still uses it — every
  requirement here is the opposite of what a step-debugger wants, and a dead player's board
  must keep rendering there because stepping *backwards* past an elimination is the normal
  thing to do. All four "stay in place" requests fall out of **one** arrangement: seat row,
  action bar and stack dock are flex siblings *above* the scrolling body, the own-hand bar is
  a sibling *below* it, and nothing is `position: sticky` (four stacked sticky strips slide
  under each other). **Commander hover-preview**: measured rather than assumed — the command
  zone was the **only** zone in the codebase without `cardTooltip` (hand, battlefield,
  graveyard, exile and stack all had it).
- **AC 6009 — pass-until.** `stores.js::startPassUntil`, entirely client-side: each iteration
  is one ordinary `POST /api/game/action` naming the `PassPriority` option the server already
  offered, so **no server change, no new route, and no recorded seed moves**. It stops on
  cancel, game over, no decision, **a non-`Priority` decision**, no pass offered, a failed
  request, or 400 passes (below the server's own 500-consecutive-pass guard, so it stops
  first and stops visibly) — and it always **says which**. The non-`Priority` stop is the
  load-bearing one: answering a cleanup discard or a trigger's targets with a default is
  precisely the defect UI-1 existed to delete. Predicates are keyed on a **mode object**, so
  the note's fine-grained form is one more entry (`OOS-UI3-3`).
- **AC 6010 — target segmentation.** `TargetOptionView.owner`, derived inside `NameIndex`
  from the **same already-redacted view** every `label` comes from — never from `GameState`,
  and never re-derived client-side, which would be wrong for exactly the case that matters
  (a stolen permanent sits in its *controller's* battlefield map, CR 109.4). `TargetPicker`
  groups by it, human seat first, unlabelled last. Grouping carries **original candidate
  indices**, so what is submitted is byte-identical to before.

**Tests**: **4,247 → 4,253 / 0 failing / 5 ignored** (baseline measured on this branch at
merge-base `f40c9fb9` before any edit — note this is the post-SIM-2/UI-2/CARDS-2 merged tree,
not CLAUDE.md's 4,218 UI-2 branch pin). +4 view-model (tier classification including the
documented default arm, redaction non-leak across three hidden-zone cases, exact prose), +2
play-server HTTP probes. **Every one watched failing under a deliberate revert**, including
two I re-ran independently rather than trusting the implementing agent's report.

**The fixture lesson worth carrying**: the combat probe first ran on `COMBAT_SEED` (6) and
passed while checking almost nothing — that seed offers **one** eligible attacker, so
"attacker → defender" collapsed to "there is a defender" and a bug swapping two attackers'
defenders would have passed. A sweep of `seed` ∈ 0..24 found that **every** seed offers 3
player targets (just CR 506.2) and **only seed 21 offers two eligible attackers**, because at
the turn the first attack becomes available the boards hold a single creature. New pin
`UI3_SPLIT_COMBAT_SEED = 21`, and **the split itself is asserted** (`distinct_defenders >= 2`)
rather than reported, so a re-deal fails loudly instead of leaving a test that still passes
while checking a strictly weaker property. The blocker half is asserted the same way.

**Gates**: PROTOCOL **33** / HASH **70** gate-EXECUTED unmoved (`--test core` hash_schema +
protocol_schema, 53 passing — not predicted); `decision_gate` 18/18; `cargo clippy --workspace
--all-targets -D warnings` clean; `cargo fmt --check` clean; `tools/check-defs-fmt.sh` clean
(1,803 defs); coverage untouched (0 card-def edits).

**S6 method, both ways**: play frontend **151 → 155 modules**, 0 warnings; replay viewer
**142 modules, unchanged**, and its **CSS bundle hash is byte-identical** across the change
(`index-DYVpLGsR.css` before and after) with the JS differing by 10 bytes — exactly the one
deliberate `formatTarget` string and nothing else. Only **one** `$viewer` file was touched
(`CombatView.svelte`); the four S7 pickers other than `TargetPicker` are byte-identical, and
their six `test_ui1_*` HTTP channel probes re-run green.

**Seeds filed** (`docs/audits/decision-point-audit.md` §8.1): **OOS-UI3-1** (nine wrong CR
citations in `events.rs` doc comments, all in the renumbered 701.x keyword-action block,
verified against the CR text — the largest instance of the OOS-DP6-8 rot class yet, and it
survived because every wrong number points at a *real* keyword action); **OOS-UI3-2** (two
event arms under-disclose because the only id they carry is the destination object — a
battlefield bounce is public in paper and renders name-free; needs a wire change);
**OOS-UI3-3** (fine-grained "until Bot-3 end"); **OOS-UI3-4** (no reveal channel on
`CardInZoneView`, so an opponent's seat card can never show a revealed hand card).

**Limitations 21–25** appended to `tools/play-server/README.md`.

**The fix cycle's finding is the one to read**: `/review` caught that **the 2×2 grid was not
2×2**. `repeat(auto-fit, minmax(22rem, 1fr))` packs as many tracks as *fit*, and four boards
need only ~88rem — so on any display wider than that the batch shipped a squeezed **1×4** row
with empty space to the right, which is *verbatim* the complaint the grid exists to answer.
`auto-fit` was chosen because it delivered the dead-player reflow with no code branch, and it
did; it also silently failed the headline requirement on exactly the machines most likely to
run this. **A CSS idiom that solves the requirement you were thinking about can fail the one
you started from, and neither the build nor any test can tell you** — there is no frontend
harness (plan §8 R7), so the only detector was reading it. Corroboration the reviewer found
and I had not: `--cells` was set inline on the grid and consumed by **no CSS rule** — a hook I
wrote for this and never finished, sitting in the file as evidence. Column count is now
computed. Second MEDIUM, same family: `.top-dock` was the **one uncapped sibling** — I capped
`.stack-dock` and `.hand-bar` and missed the container that hosts every picker, so an expanded
drawer plus a segmented `TargetPicker` could squeeze the board to nothing and push the page
into a *document* scrollbar, destroying the "stay in place on scroll" property the whole flex
arrangement exists to provide.

**One review finding was FALSE and was not actioned**, recorded because a future reader will
meet the same claim: the reviewer's sole HIGH said the `phase-end` predicate captures
`ctx.stackDepth` once at run start, so a resolve-then-recast slips through. It had quoted a
collapsed paraphrase and dropped the `ctx.stackDepth = depth` re-baselining line; its own
worked example fails at its step 3. Verified against source before deciding. (That gap **was**
real one commit earlier — I found and fixed it in a self-review pass before the reviewer ran,
which is presumably why it was reading for it.)

**Also worth carrying**: `MAX_EVENTS` was still **500**, chosen when ~11 `GameEvent` variants
rendered as prose; this batch took that to **60**, multiplying lines per turn. Shipping the
feature that makes a cap bite while leaving the cap alone would have truncated the very
history the feature exists to show. Raised to 2000. **A constant tuned against a behaviour is
part of that behaviour's blast radius.**

---

## Worker Handoff (SIM-2, `scutemob-176`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (SIM-2) — **F3 + F4 + F5 CLOSED**
**Task**: `scutemob-176`. Branch
`feat/sim-2-mana-intelligence-batch---residual-auto-tap-solver-cou`
**Full evidence**: `memory/primitives/sim2-mana-intelligence-2026-08-02.md`

**Completed**:
- **F4 — the mana solver counted SOURCES, not MANA.** `produces` was expanded per unit and
  the expansion never read, so Sol Ring was one mana. Both directions were live and a human
  saw both: over-tap (Sol Ring + two Forests for `{2}{G}`, one mana stranded and destroyed by
  CR 500.4) and, worse, **under-offer** — a `{2}` spell with only a Sol Ring untapped solved
  to `None`, so `can_afford` never offered the cast. A tapped source now credits its whole
  production to a running tally and each pip is paid from that tally.
- **F3 — auto-tap was all-or-nothing.** Pool covers the whole cost → tap nothing; anything
  less → solve for the **entire printed cost** with the pool never subtracted.
  `solve_mana_payment_with_pool` subtracts in `ManaPool::can_spend`'s own order and solves the
  residual; the early return is now the residual-is-zero case of the general rule.
  `advance()`'s bot path calls the same helper, and `can_afford` asks the same question once
  instead of a pool shortcut OR a whole-cost solve **with a gap between them** (a player with
  `{G}` floating and one Forest up was told `{1}{G}` was uncastable).
- **F5 — the bot tapped out every empty upkeep.** `TapForMana` 5 → **0**, below
  `PassPriority`. The demote-vs-gate choice is not arbitrary: every mana-consuming action
  already outscored 5, so a tap was only ever *chosen* when it was the sole alternative to
  passing. Scored 0 rather than removed, so it stays choosable when it is all there is.
- **The layer half of `OOS-M11-2`, recorded as theoretical, was live-wrong.** Changing which
  source the solver reaches for reddened the S8 scripted playthrough on seed 42:
  `"object ObjectId(487) has no mana ability at index 0"`. `layers.rs` clears
  `mana_abilities` for a **face-down** permanent (CR 707.2) and the solver read base
  characteristics. The doc block had illustrated the gap with *granted* abilities
  (Cryptolith Rite) and no urgency. Now `calculate_characteristics`, measured free:
  `mtg-fuzzer --games 60 --max-turns 40` is 6.8 s on both sides.
- **`OOS-CARDS2-9`, which existed only in three source comments and was never filed.** Its
  own statement of the fix — "one place: make the solver ask whether the ability is
  activatable" — was right about the affordability half and silent about the **offer** half:
  `legal_actions`'s `TapForMana` loop checked `life_cost` and nothing else, so an unmet
  activation condition and a summoning-sick creature were offered and refused, and the
  play-server driver carried both refusal strings in `KNOWN_FALSE_OFFERS`. One predicate,
  `tap_ability_is_activatable`, now serves both.
- **The bot half of `OOS-M11-8`.** S8 recorded it CLOSED on a fix living only in
  `auto_tap_commands_for` while `advance()` kept its own printed-cost solve. Latent (no
  shipped bot announces X > 0), but open. Closed by there being one function; pinned by
  `t21`, which drives a purpose-built `XBot`.

**Gates**: workspace **4,214 / 0 / 5**; `play-server` 40/40; clippy `-D warnings` clean;
`cargo fmt --check` + `tools/check-defs-fmt.sh` clean (1,803 defs); `cargo build --workspace`
(SR-3 seal) clean. **PROTOCOL 33 / HASH 70 gate-executed, unmoved** — the criterion's
"PROTOCOL 32" was stale, PB-DX6 moved it before this fork. Coverage unmoved: zero card-def
edits, zero completeness flips.

**Diff scope, stated exactly**: `crates/simulator` (3 source + 4 test files) +
`tools/play-server/src/main.rs` (one seed pin) + docs/memory + **one line of
`crates/engine/src/state/keyword_registry.rs`**. That last one is not scope creep and not
optional: SR-5's gate greps the source tree, so the solver's new CR 302.6 branch on
`KeywordAbility::Haste` must be declared or `core::keyword_registry` fails. It is a data
line; PROTOCOL/HASH are unmoved and `git diff main -- crates/engine/src/rules
crates/engine/src/effects crates/card-types crates/card-defs` is empty.

**Fuzzer A/B** (`--games 100 --seed 1 --max-turns 60`, merge base vs branch): **96/100 games
byte-identical**, 4 differ only in command count, violations 0 → 0, every game ends
`MaxTurnsReached(60)` on both sides. The four are the offer set moving.

**Fix cycle (`/review`, Opus)**: 8 findings, all applied. Two were live SR-38 violations the
batch had *asserted away*: (a) CR 605.3 **stax restrictions** — an opponent's Collector Ouphe
or Stony Silence refuses a Sol Ring's tap, and that class was mirrored in neither the solver
nor the offer loop while four comments claimed the mirror of `handle_tap_for_mana` was
complete (same shape as `OOS-SIM1-3`: an enumeration is only as complete as the category it
names — there enum variants, here the rejections inside one function); (b) an SR-36 **scaled**
ability's marker was called a safe under-count, but the engine adds `resolve_amount(..).max(0)`
with no error, so Itlimoc with no creatures out produces nothing while the marker promises one
mana — over-credit, refused cast. Both fixed and pinned (`t22`, `t23`). The other six were
documentation: a "hoisted" claim the same file contradicted two hundred lines later, the
`OOS-M11-2`/`OOS-M11-8` audit rows (criterion 5 asked for exactly this and the first pass
appended seeds without correcting the rows they contradicted), the playtest-triage F3/F4/F5
banners and roll-up, a defs-vs-ability-rows unit error in a population count, and a
discrimination matrix that claimed no test was decorative while having no row for the one
guarding `pick_least_waste`.

**Two engine findings carried out, both out of scope and both worth someone's attention**:
- **`OOS-SIM2-6` (HIGH)** — `calculate_characteristics` recurses without bound through
  `is_effect_active` → `check_static_condition` → `expect_characteristics`, and
  `indomitable_archangel` (`Complete`, deck-legal) makes that unconditional: its metalcraft
  static's activity depends on counting artifacts, which depends on layer-resolved types,
  which depends on its activity. **Hard, unrecoverable crash** (still overflows at
  `ulimit -s 524288`). Reproduce: `mtg-fuzzer --games 1 --seed 504 --max-turns 200` on this
  branch. **DEAD REPRO across the PB-DX22 merge (`scutemob-196`, `95f53b78`)**: that batch shuffles the
  fuzz libraries and registers the commanders, so seed 504 deals a different game and no
  longer reproduces this one. The seed is a pre-merge artefact — see `OOS-DX22-7`; the
  defect itself is closed by PB-DX19 (`451e3517`) regardless.
  Diagnosed by `gdb` backtrace plus a depth probe that named the card. Very likely
  the mechanism behind `OOS-M11-3` / `OOS-DP3-9`, which had the symptom and no cause.
- **`OOS-SIM2-5`** — `layers.rs` P/T arithmetic is unchecked `i32`; Devilish Valet's
  doubling reaches 2^30 and the next doubling panics in debug and **wraps silently in
  release**.

**Seed pin re-derived, for the second time in two days**: `TARGET_SEED` 1 → **13**, by the
rule the pin's own comment states (the pins are a function of the whole corpus *and of the
provider*; SIM-2 changes `can_afford`). Swept 0..24 running the four fixtures against each;
only 13 passes all four. Seed 1 now drives into `OOS-SIM2-5`'s overflow, which is recorded at
the pin so it cannot later read as a property of the fixture.

**Left open, deliberately**: `OOS-SIM2-1` (the solve is greedy — an under-offer is still
possible where source assignment interacts), `OOS-SIM2-2` (20 abilities with their own mana
component are never planned), `OOS-SIM2-3` (a bot still cannot pay an activated ability's
mana cost — `advance()` auto-taps for `CastSpell` only; pre-existing and unchanged in kind),
`OOS-SIM2-4` (SR-36 scaled production and CR 106.6a replacements under-counted — safe
direction), `OOS-SIM2-7` (the two `tools/tui` call sites inherit the production fix but not
the residual). What remains of `OOS-M11-2` after this batch is exactly cost *modifiers* and
CR 106.12 restricted mana.
## Worker Handoff (UI-2, `scutemob-178`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (UI-2) — **F9 CLOSED for `Sacrifice` + `Squad`**
**Task**: `scutemob-178`. Branch
`feat/ui-2-additional-cost-surfacing---sacrifice-squad-offer-descr`

**Completed**:
- **The request wire already existed; the OFFER was blind, and that was the whole
  defect.** `CastSpellData.additional_costs` covers all sixteen cost kinds and
  `ActionParamsDto` deserialized it, so a hand-crafted POST could pay a sacrifice
  before this batch. `StubProvider` simply never read `spell_additional_costs` or
  Squad — zero references — so Life's Legacy was offered on mana affordability alone
  and `casting.rs:3311` then refused it (the human's observed **422**, an SR-38
  violation), and a Squad creature always cast at `count: 0` with the optional cost
  silently lost (CR 702.157a).
- `LegalAction::CastSpell` gains `additional_costs: AdditionalCostPlan`. Eligibility
  mirrors `casting.rs:3300-3369` **gate for gate** — zone, controller,
  `object_cant_be_sacrificed`, then the filter against LAYER-RESOLVED characteristics
  — and deliberately **not** `effects::eligible_sacrifice_targets`, which also checks
  `is_phased_in` and would therefore offer a different set from the one the engine
  validates. `object_cant_be_sacrificed` is re-derived locally because the engine's
  copy is `pub(crate)`; documented as a *necessary* duplicate, explicitly unlike
  `effective_cast_cost`, whose engine copy is public and is consumed.
- **A required cost with nothing eligible suppresses the whole offer**
  (`offerable_cast_plan`, one helper used by BOTH cast loops). That is SR-38 restored,
  and it is F9's actual fix.
- `params.rs` appends the plan's default sacrifice only when the caller announced
  none, so `ActionParams::default()` (every bot) still produces an engine-accepted
  command and a human's choice is never overwritten. Squad is never defaulted —
  absent means declined, which keeps a bot's command byte-identical to the pre-UI-2
  one.
- `ActionOptionView.costs` + `CostPicker.svelte`, inserted between `ValuePrompt` and
  `TargetPicker`. `validate_additional_cost_params` answers **400** for an
  out-of-offer sacrifice id, more than one id, a Squad count above `max_count`, and a
  duplicate entry of either kind; the other fourteen `AdditionalCost` variants
  deliberately fall through to the engine's 422 and the doc says so.

**The card-def repair, which the brief did not anticipate**: `galadhrim_brigade` — the
very card the human tried to Squad — shipped `Complete` and deck-legal carrying
`KeywordAbility::Squad` with **no `AbilityDefinition::Squad { cost }`**, so
`casting.rs::get_squad_cost` returned `None` and *every* non-zero count was refused
with "spell has squad keyword but no squad cost defined". Repaired from the printed
"Squad {1}{G}". `core::ui2_additional_cost_roster` **R3b** now pins that the marker set
and the cost set are the **same set**, in both directions. This is the CARDS-2 shape
again: the knowledge existed per-def and nothing could fail.

**The fix cycle found the sharpest correctness bug**: `effective_cast_cost_with_additional`
**summed** multiple `Squad` entries where `casting.rs` **assigns** (`squad_count = *count`,
so the LAST wins). A two-entry submission therefore made the auto-tap reach for more
mana than the engine charges, the solver found no plan, no taps were issued, and the
engine refused the cast for want of mana — a 422 after a clean offer, which is exactly
the shape this batch exists to delete. Mirrored to last-wins, and duplicates are now
refused at the 400 boundary as well, because the engine resolves the two kinds in
**opposite** directions in silence (Squad last-wins, Sacrifice first-wins via `find_map`).

**The two findings that matter beyond this batch** — both measured, neither UI-2's to
fix:
- **OOS-UI2-1**: **`mtg-fuzzer` has never cast a spell.** `bin/fuzzer.rs` populates its
  libraries through `GameStateBuilder` and **never shuffles them**, while `random_deck`
  appends its ~34 basics LAST and `Zone::Ordered`'s top is the last index. Instrumenting
  the provider over 5 games x 80 turns gave **25,964 hand-card observations and zero
  non-lands**; `build_additional_cost_plan` was reached **0** times in 30 games. UI-2's
  own 360-game A/B came back byte-identical for that reason and is reported as worth
  nothing rather than banked. **Every "fuzz parity" claim in this project's history is a
  claim about a land-only game.**
- **OOS-UI2-2**: `HeuristicBot` scores `TapForMana` 5 against `PassPriority`'s 1, and in
  the upkeep those are the only two actions — so it burns its lands where it cannot
  spend the mana, the pool empties (CR 500.4), and by its own main phase the cast is
  never *offered*. A whole-game bot test therefore passes by never reaching the thing it
  claims to test.

**Numbers**: tests **4,185 -> 4,218 / 0 / 5**. PROTOCOL **33** / HASH **70**
gate-EXECUTED unmoved (the criterion's "PROTOCOL 32" is stale — PB-DX6 moved it before
this fork, the same staleness UI-1 recorded). `decision_gate` 18/18. Coverage unmoved at
**1,133/1,803 = 62.8%**, 0 completeness flips — the Galadhrim repair is an addition to an
already-`Complete` def. `fmt` + `check-defs-fmt.sh` + `clippy -D warnings` clean.

**NOT zero engine lines, and the exception is named**: **9 insertions / 1 deletion in
one file**, `crates/engine/src/state/ability_definition_registry.rs` — one data-only
`sites:` row adding `crates/simulator/src/legal_actions.rs` to `A::Squad`. The SR-15
gate demanded it the moment the provider read the cost-carrying variant; that gate's
`SCAN_ROOTS` includes `crates/simulator/src` **by design** (SR-20), and `A::Bloodrush`
already carries the identical row. `crates/card-types/src` diffs empty.

**Seeds filed** (`docs/audits/decision-point-audit.md` §8.1): **OOS-UI2-1..5** —
the fuzzer's unshuffled libraries; HeuristicBot's upkeep mana burn;
`squad_max_count`'s under-report (capped by playtest triage **F4**, whose test pins
the current wrong value and names the right one); the fourteen unsurfaced
`AdditionalCost` variants; and the TUI receiving the sacrifice default with no picker.

**Also deleted**: `local_game_playthrough.rs`'s `KNOWN_FALSE_OFFERS` register. Its last
entry was F9, its own trailing assertion required deletion once an entry stopped firing,
and the playthrough now asserts `run.error.is_none()` unconditionally — strictly sharper.

## Worker Handoff (CARDS-2, `scutemob-181`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (CARDS-2) — **SR-37 built; F1 + F2 CLOSED**
**Task**: `scutemob-181`. Branch
`feat/cards-2-corpus-field-fidelity-audit-permanent-gate-mana-cost`

**Completed**:
- **A new permanent gate, SR-37**: every `all_cards()` def's printed mana cost, power,
  toughness and type line is diffed against a committed Scryfall fixture. Three pieces —
  `tools/card-field-dump` (enumerates, SR-36), `tools/refresh-card-fidelity-fixture.py`
  (joins `cards.sqlite`, copies **verbatim**), and
  `core::cards2_printed_field_fidelity` (**the only place equality is decided**). The
  fixture is committed because `cards.sqlite` is gitignored and absent in CI; the Python
  does no normalisation on purpose, or the two sides would drift.
- **39 real defects found and repaired** across 31 defs (the gate's raw first run said 51;
  the difference is six false mismatches from its own notation and six more that were the
  design working — see the evidence record's three-column table): 17 mana costs, 5 P/T over 3 defs,
  16 type lines over 16 defs, 1 duplicate card name. **R2 reproduced the playtest-triage
  F2 table exactly, card for card** — first independent confirmation it was reproducible.
- **Boon Satyr (F1) fully repaired**, all four defects incl. the printed "+4/+2" that was
  **never authored** on a def declaring `Complete`. Expressed as two layer-7c statics on
  `EffectFilter::AttachedCreature` — **the shape Rancor already used**; the machinery was
  never missing. T5 proven discriminating **by execution** (revert → the bear stays 2/2).
- **Two more `Complete` defs were implementing a different card's abilities** —
  `backup_agent` (Backup 1 + Lifelink, from another card entirely) and `necron_deathmark`.
  Both repaired, both stayed `Complete`; both were caught because **more than one** printed
  field was wrong, which is the signal for "authored from a misremembered card".
- **Two more `Complete` defs implemented text on NO card at all** — `cyber_conversion`
  ("becomes an artifact + draw a card" for a printed "turn target creature face down")
  and `exalted_angel` (static `Lifelink` for a printed *triggered* "whenever this deals
  damage, gain that much life" — CR 702.15a lifelink cannot be Stifled; the printed clause
  can). Both **honestly demoted** with blocker notes naming the missing primitive
  (**OOS-CARDS2-5/6**), not half-repaired.
- **Zero engine lines**; PROTOCOL/HASH gate-executed unmoved; `decision_gate` 18/18; tests
  **4,185 / 0 / 5** (post-merge with SIM-1). Coverage **1,133/1,803 = 62.8%**, down from 1,137/1,804 — **4
  completeness flips, ALL demotions**. The number went DOWN because the corpus got truer
  (the PB-DX4 pattern): `cyber_conversion` and `exalted_angel` implemented text on no card,
  `braided_net`'s two remaining clauses have no expression (its note first claimed six,
  and a reviewer found four of those to exist), `birchlore_rangers`' mana
  ability has no `Cost` variant.

**Hazards for the next session — read these three:**

0. **A new browser-client defect fell out of the re-derivation: `OOS-CARDS2-4`.** An Aura
   is offered with `target_min: 0` — its target requirement lives in
   `KeywordAbility::Enchant(...)`, which `casting.rs:3720` special-cases (CR 303.4a) and
   the provider never reads — so the engine 422s the cast. **A human clicking any Aura in
   the play client gets an error.** Simulator-only fix; same shape as CARDS-1's equip bug,
   one link earlier in the chain. The S7 test driver now *skips* a refused action, which
   is a workaround in the test and NOT a fix.
1. **The seed pins are a function of the WHOLE CORPUS, not of the completeness markers.**
   Every play-server pin carried the comment "re-read when a batch flips a marker". This
   batch flipped **zero** markers and moved all of them, because
   `simulator/src/deck.rs::random_deck` draws its commander from `Complete` **AND
   Legendary AND Creature** and fills by **colour identity** (computed from the mana
   cost). Measured: commander pool 91 → 90. Correcting a *type line* re-deals every
   seeded game. All the comments now say so. Filed as **OOS-CARDS2-3** (no gate exists).
2. **A fixture predicate broader than the fixture's purpose does not fail when the fixture
   moves — it silently tests something else.** `test_x_value_is_forwarded_to_cast_spell_data`
   had retargeted from a spell onto Deserted Temple's "untap target land", and the failure
   surfaced three assertions later as "the cast is still offered after tapping" (it was
   never a cast). Predicate now says `CastSpell`.
3. **A golden script generated from a card def is not an independent check of that def.**
   Scripts 177 and 164 were written to what the def said, not to the card, and passed for
   two batches while encoding a wrong cost. Script 163 is **retired** — its subject
   (Backup Agent's Backup 1) does not exist.

**Also worth carrying**: the duplicate-name finding (R5) had been written down in
`memory/card-authoring/marker-sweep-2026-07-16.md` **seventeen days earlier**, with the
words "one of the two should be deleted", and nothing happened — because no gate could
fail. `CardRegistry::try_new` rejects a duplicate `CardId` and says nothing about a
duplicate name.

**The fix cycle found the sharpest thing in the batch — read this one.** `tyrranax_rex`,
the gate's own motivating example, shipped `Complete` declaring `KeywordAbility::Ravenous`
— **on no printing of the card** — while omitting haste, Toxic 4 and "can't be countered";
a golden script certified the invented keyword. And that script had already FAILED earlier
in this same batch when the cost was corrected, and was re-baselined by recomputing its
mana pool **without re-reading the oracle** — the exact failure the batch had written down
for scripts 164/177 one commit earlier. Repaired in full (every primitive existed); script
177 retired alongside 163. **The rule: a wrong printed field is reason to re-read the whole
oracle, not to fix the field.** The batch's own "more than one wrong field = misremembered
card" heuristic cannot catch this class, because only one field was wrong.

**A gate-needle gap worth someone's time**: `braided_net` and `windbrisk_heights` both
shipped `Complete` with a printed ability unimplemented and said so in their own comments —
"DSL gap" and "deferred". `completeness_deviation_scan`'s needle set is
`["simplif", "modeled as", "modelled as", "deviation", "approximat"]`, so neither reddened.
Both were also **stale** claims: `Effect::TapPermanent` and
`Condition::YouAttackedWithNOrMore` had both landed since. That is the third and fourth
"not expressible" note this batch found to be false (after `wake_the_dead`'s `x_count` and
`boon_satyr`'s aura static).

**Full evidence record**: `memory/card-authoring/cards2-field-fidelity-2026-08-02.md`
(measurement, every disposition, the four gate-design findings, and seeds
**OOS-CARDS2-1..11** — 7..11 came out of three review fix cycles: **OOS-CARDS2-7** the
`completeness_deviation_scan` needle set has no entry for "DSL gap" or "deferred", the two
phrases the corpus actually uses; **OOS-CARDS2-8** stale "not expressible" notes are a
recurring class, four found false in this batch alone). Gate rationale + refresh procedure:
`docs/engine-invariants.md`
(SR-37).

## Worker Handoff (CARDS-1, `scutemob-179`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (CARDS-1) — **OOS-M11-10 (equip) CLOSED**
**Task**: `scutemob-179`. Branch `feat/cards-1-equip-target-repair-batch---close-oos-m11-10-16-defs`

**Completed**:
- **17 card defs repaired** (not the 16 the seed scoped): every `AbilityDefinition::Activated`
  whose effect is `Effect::AttachEquipment` now declares
  `TargetCreatureWithFilter { controller: You }` (CR 702.6a). **0 engine lines.**
- **Roster re-derived from `all_cards()` per SR-36**, never from the seed's def-source scan. It
  confirmed the seed's counts exactly (17 activated attach sites, 16 empty, `cryptic_coat`'s ETB
  self-attach correctly excluded, 4 prose-only files correctly excluded) — and then broke its
  conclusion, see the lesson below.
- All 17 printed equip lines MCP-verified as plain `Equip {N}`: **no CR 702.6c quality
  restriction anywhere**, so there is no per-def deviation to document.
- New permanent gates: `core::cards1_equip_target_roster` R1–R3 and
  `primitives::cards1_equip_target_repair` T1–T7b (11 tests). Fail-before evidence with verbatim
  pre-fix output: `memory/primitives/cards1-equip-fail-before-2026-08-02.md`.
- Gates: **0 completeness flips** (report body byte-identical, 1,137/1,804 = 63.0%);
  **PROTOCOL 33 / HASH 70 unmoved**, verified by *executing* `core hash_schema` +
  `core protocol_schema`; `decision_gate` 18/18, no pin moves; `cargo fmt --check` and
  `tools/check-defs-fmt.sh` (1,804 defs) both clean.

**Durable lessons** (the reason this handoff is worth reading):
- **The designated reference def was itself wrong.** The seed named `helm_of_the_host` as the one
  def that "declares the `TargetRequirement`" — true, and it was read as "already correct". It
  declared a bare `TargetRequirement::TargetCreature`, dropping CR 702.6a's "you control", so it
  offered opponents' creatures as legal equip targets. *Being the only member with a requirement
  is not the same as being the only member with a correct one.* A batch that trusts its reference
  without re-deriving it inherits the reference's defect — the same shape as PB-DX6's "the brief
  named one arm; all three shared the defect".
- **Two tests written to fail pre-fix passed pre-fix**, and that was information, not noise: the
  legacy `AttachEquipment` special-case in `abilities.rs` *does* validate a **volunteered**
  target. So the defect was never "equip doesn't validate" — it was "**nothing ever asks**".
  That is exactly why the TUI (which volunteered targets) never surfaced this and the browser
  client did on its first human game. The prediction was recorded as wrong rather than smoothed.
- **`OOS-M11-10` names TWO distinct seeds** in `docs/audits/decision-point-audit.md` §8.1 — the
  equip one (closed here) and a still-OPEN loyalty-ability targeting gap filed the same day by
  M11-local S8's close-out. **Every cite of the ID outside that table — CLAUDE.md, the
  milestone-reviews doc, `params.rs`'s in-source comment, and line 183 of this file — means the
  LOYALTY seed.** Both rows are now labelled and a collision note sits under the table.
  Renumbering was declined here: it would rewrite an in-source engine comment, and this batch is
  pinned to zero engine lines. Whoever next touches `params.rs` should renumber the equip row.

**Not done / deferred (deliberate)**:
- **OOS-CARDS1-1** — `darksteel_garrison` has the identical shape for **Fortify** (CR 702.67a).
  Card-def-only and 0 engine lines via
  `TargetPermanentWithFilter { has_card_type: Land, controller: You }` — verified live in
  `casting.rs`, not assumed. Left alone because criterion 6003 required neighbouring attach
  mechanisms be untouched.
- **OOS-CARDS1-2** — **Reconfigure** has it too, but the defective `targets: vec![]` is written in
  *engine* source (`testing/replay_harness.rs`'s `AbilityDefinition::Reconfigure` expansion), so
  zero-engine-lines excluded it by construction. CR 702.151a says "**another** target creature you
  control" — it needs `exclude_self: true`, and copying the equip repair verbatim would be wrong.
- Both rosters are **pinned** by `t7b` (`{"Darksteel Garrison"}`, `{"Lizard Blades"}`), so either
  fix must move a pin in the same change.
- **OOS-CARDS1-3 — the biggest of the three, and it came from the `/review`, not from me.** 21
  Equipment defs print "Equip {N}" and have **no equip ability at all** (`K::Equip` is a
  `KeywordHandling::Marker` that synthesises nothing), **10 of them deck-legal `Complete`**, 9 by
  the `#[default]` derive. That is a larger population than this batch touched and one link
  earlier in the same chain: not "the picker never asks for a target" but "**there is no action
  to pick**". A human can legally deck Umezawa's Jitte or Sword of Feast and Famine today and
  never be offered an equip. Four of the 11 `partial` members already named this gap in their own
  completeness notes — the knowledge existed per-def and had never been aggregated into a seed.
  **Lesson**: R1's exact-17 pin makes a true statement ("all 17 members are correct") that reads
  as a false one ("the equip surface is swept clean"). A roster gate certifies the population it
  enumerates and is silent about the population it does not — and the defs that fall outside it
  are exactly the ones no gate is watching. Whoever takes OOS-CARDS1-3 should also decide whether
  R1 grows a companion pin over marker-only Equipment.

**Hazards for the collector**:
- CLAUDE.md and this file both got a new **appended** section (no existing line grown), per the
  2026-08-02 formatting rule — expect the usual both-sides-edited conflict and take the richer side.
- `docs/authoring-status.md` / `-prev.json` were regenerated to measure flips and then **reverted**,
  because the only delta was the timestamp/SHA header. Do not re-run and commit them.

**Commit prefix used**: `scutemob-179:`

## Worker Handoff (SIM-1, `scutemob-175`)

**Date**: 2026-08-02 (worker session)
**Workstream**: playtest-triage successor track (SIM-1) — **triage F7 CLOSED**
**Task**: `scutemob-175`. Branch `feat/sim-1-commander-castable-from-the-command-zone-legalactionpr`

**Completed** — a human can cast their commander from the command zone. **Zero engine lines**
(`crates/engine/src` + `crates/card-types/src` + `crates/card-defs` diffs all empty and pasted);
PROTOCOL **33** / HASH **70** gate-executed unmoved.

- **The engine was never the problem.** `casting.rs` has supported CR 903.8 since M6 — it derives
  command-zone-ness from the object's zone, admits it past the "not in your hand" gate, gates it
  on `commander_ids`, applies the tax and increments the counter, and emits
  `CommanderCastFromCommandZone`. `StubProvider` simply never looked in the zone, so the browser
  correctly reported that the server had offered nothing. **The frontend was innocent and so was
  the wire**: `params.rs` already forwarded the bare card, and `from_zone` is read *nowhere* in
  the workspace.
- **`effective_cast_cost` — one helper, three call sites.** The brief named one place the tax was
  needed; there were **three**, and `local_game.rs`'s own doc block already described the defect in
  as many words ("Recasting a taxed commander with a pool that covers only the printed cost
  therefore skips tapping and the cast is rejected"). The offer gate, the human `submit` auto-tap
  and the bot `advance()` auto-tap all read the **printed** cost. They now share one helper that
  **consumes `mtg_engine::apply_commander_tax`** rather than re-deriving `generic + 2*tax` — SR-38's
  "only offer what the engine accepts" is only true if the two arithmetics are literally the same
  function. (Contrast `multiply_mana_cost`, a *necessary* duplicate because the engine's copy is
  private. This one is not, so duplicating it would have been a choice, and the wrong one.)
- **The Drannith trap — the finding that would have shipped a fresh SR-38 violation.**
  `casting.rs` rejects **any** non-hand cast while an opponent controls a Drannith Magistrate, and
  `is_cast_restricted_by_stax` says in its own doc that it deliberately does not mirror per-card
  *zone* restrictions. That was harmless for exactly one reason: every offer the provider had ever
  made was a hand cast, and a hand cast always satisfies `zone == Hand(player)`. **Every
  command-zone offer is a non-hand cast**, so without a new mirror the batch would have offered an
  action the engine rejects 100% of the time. `drannith_magistrate.rs` is deck-legal `Complete` by
  the `#[default]` derive. Generalisable: **a guard that is "harmless because unreachable" becomes
  a defect the moment you widen what reaches it — check the reachability argument, not the guard.**
- **Timing is mirrored, not assumed.** A commander is a permanent, so sorcery speed is the *usual*
  answer — but the engine's timing gate is zone-agnostic, so a commander with Flash or under a CR
  601.3b flash grant is legally castable at instant speed. The hand loop's timing block was
  extracted to `can_cast_at_this_time` and is now called by both enumerations, so they cannot drift.
- **Appended after the hand loop on purpose**: `RandomBot` picks by index, so appending leaves every
  pre-existing action's index untouched.

**The regression, and why it is not SIM-1's bug** (the durable lesson of this batch):
`local_game_playthrough` seed 1 halted `InfiniteLoop` at turn 17 having applied exactly 20,000
commands. Diagnosed **by measurement, not by reading code** — a throwaway instrumented copy of the
test printed a per-turn, per-kind histogram: **19,351 of those commands were `DeclareAttackers` in
that single turn.** The cause is the already-open seed **`OOS-M11-9`**: nothing gates "attackers
already declared this combat", so a **vigilant** attacker stays untapped, stays `eligible`, and is
re-offered without limit (CR 508.1 makes it a once-per-combat turn-based action). SIM-1 only made
it *reachable* — seed 1's human commander is `Samut, Voice of Dissent`, which has Vigilance, and
before this branch no commander could ever be cast, so no vigilant commander was ever on the
battlefield to re-declare with. It is the same seed, the same turn range and the same
20,000-command signature the audit already records for the S8 **bot-side** instance.
**The fix location was already decided, in shipped source.** `heuristic_bot.rs` mitigated the
identical loop with a per-combat `RepeatKey` cap and states its reason: put it in the client
"rather than in `StubProvider` … keeps the provider's action list, and therefore every recorded
`mtg-fuzzer` seed, untouched." The scripted human policy is simply the **second client** to need
it, so it got the same cap — reset on the **combat-entry edge**, not the turn number, because
`MR-M11-09` found exactly that regression in the bot (a turn-keyed tally silently disables attacks
in every CR 506.5 extra combat). **No assertion was relaxed.**

**A/B evidence, measured in a separate git worktree at the true merge-base — not reasoned to:**
- **Fuzzer unperturbed.** 60 games, `--seed 42 --max-turns 50 --verbose`: per-game
  `Seed/Turns/Commands/Violations/Error` lines diffed with **zero** differences; the only differing
  line in the entire output is the games/sec throughput counter (58 vs 57), i.e. timing noise.
  This is immunity **by construction** — `fuzzer.rs` never calls `builder.player_commander`, so
  `commander_ids` is empty and the `commander_ids`-gated offer is unreachable there (`OOS-SIM1-4`).
- **Playthrough trajectory essentially unmoved.** Per-seed commands, merge-base → branch:
  1058→1064, 1177→1183, 1164→1172, 1010→1010, 1118→1111 — within ~1% on every seed, with identical
  per-seed action-kind coverage sets.
- **A correction worth carrying**: I first reported these seeds as finishing "below the pre-SIM-1
  baseline". That compared against a **stale comment inside the test file**, written for a
  different `max_commands` config. The measured answer is *unchanged*, which is a stronger result —
  but the lesson is the recurring one here: **a number written in a doc is not a baseline; the
  baseline is what the merge-base actually does when you run it.**
- **Pre-existing failure correctly attributed**: the documented smoke command
  (`--games 100 --seed 42`, default `--max-turns 200`) **stack-overflows on the merge-base too** —
  `OOS-M11-3` / `OOS-DP3-9`, reproduced on pristine code, not SIM-1.

**Seeds filed** (durable rows in `docs/audits/decision-point-audit.md` §8.1, the same table CARDS-1
used): **`OOS-SIM1-1`** (hybrid/Phyrexian commander gated by `can_afford`, not a payment plan —
`CastSpell` has no PB-RS2 channel; note the tax cannot *create* a pip, since `apply_commander_tax`
writes only `generic`), **`OOS-SIM1-2`** (a **fourth** printed-cost auto-tap in `tools/tui`,
outside this batch's scope — which is why `effective_cast_cost` is exported `pub`, so the fix is a
call and not a copy), **`OOS-SIM1-3`** (verified exhaustively against all 9 `GameRestriction`
variants: 7 are cast-relevant, the provider now mirrors 5, and exactly
`MaxNoncreatureSpellsPerTurn` + `MaxNonartifactSpellsPerTurn` remain unmirrored — pre-existing for
hand casts, deliberately not widened), **`OOS-SIM1-4`** (the fuzzer's games are not Commander games
at all: no tax, no CR 903.9a return, no CR 903.10a commander damage is ever fuzzed — deliberately
unfixed, because fixing it moves every recorded seed).

**Scope note the coordinator must record, not swallow**: criterion 5984 requires an HTTP probe and
`tools/play-server` is a **bin** crate with no `lib.rs`, so no `tests/` integration test can reach
`build_router` — every HTTP test in this crate lives in `main.rs`'s `#[cfg(test)] mod tests`. So
criterion 5987's "empty git diff elsewhere" is satisfied as: engine/card-types/card-defs diffs
**empty**, and the `main.rs` diff **proven** test-only by line arithmetic rather than asserted —
the `#[cfg(test)]` cut is at line 207 and the lowest changed line is 3873, so the shipped binary is
behaviourally identical.

**Commit prefix used**: `scutemob-175:`

## Last Handoff

**Date**: 2026-08-13..14 (oversight session #9 — v3 closed out at rank 13, v4 re-rank, v4 rank 1)
**Workstream**: W6 correctness queue (v3 → v4)
**Task**: `scutemob-209` (PB-DX27, merge `bd0a9743`) + `scutemob-210` (PB-DX28, `2bdc3533`) +
`scutemob-211` (PB-DX29, `08c9ef1e`) + `scutemob-212` (v4 re-rank, `1d54f122`) + `scutemob-213`
(PB-DX43, `ba83116a`) + `scutemob-214` (worker-tab skill adoption, `19b81255`, PARTIAL — still
`in_progress`). Each dispatch → monitor → collect, one worker at a time, each user-approved.

**Completed**:
- **PB-DX27/28/29 shipped** (v3 ranks 11/12/13 — v3 queue COMPLETE): stale-blocker-note sweep +
  wrong-oracle register + rider OOS-ADJ-7; `TargetFilter` owner axis + choose-on-resolution
  channel; params.rs loyalty allowlist + all cost-kind surfaces. Details in each Worker Handoff.
- **SEED RE-RANK v4 shipped** (`memory/primitives/seed-rerank-2026-08-14.md` §4 authoritative,
  41 entries; 208 post-v3 seed IDs censused, 61 unrowed; PB-DX42b re-decided on corrected
  grounds; Blood Moon/Urza's Saga flag discharged as OOS-RR4-1/-2/-3).
- **PB-DX43 shipped** (v4 rank 1): CR 305.6/305.7 intrinsic mana derivation — Urborg/Yavimaya/
  Dryad live again; moons deleted their hand-authored grants. Tests **4,753 / 0 / 5**;
  PROTOCOL **37** / HASH **76**; coverage 63.0%.
- **PB-DX28 ran through a mid-batch system reboot** cleanly: disclosed non-compiling WIP
  checkpoint + `pb-DX28-RESUME.md`; the pattern worked and is worth reusing.
- User directive recorded (auto-memory `project_authoring_resumption_gate`): **no rush back to
  authoring** — work the v4 queue until everything is in place; surface the option, user decides.
- `scutemob-214` (ESM-agent-filed): dispatch/crew skills now launch via `esm worker-tab`
  (custom worker prompt preserved verbatim via `--prompt`); criteria 6518/6519 satisfied.

**Not done / deferred**:
- `scutemob-214` criterion 6520 — live split-tab verification — lands at the next real
  `/dispatch` (PB-DX44); the task stays `in_progress` deliberately.

**Next session candidates**:
- **Dispatch PB-DX44** (v4 rank 2, the casts you cannot make) — first use of the new
  `worker-tab` launch; verify + satisfy 6520 and transition `scutemob-214` to done at that point.
- Then DX15a (rank 3), DX45 (rank 4 — the costless-"may" channel, the big authoring unlock),
  DX47 probe (rank 5).

**Hazards** (carrying forward):
- **LAN DNS blips**: `tower` (the dnsmasq host itself) intermittently fails to resolve;
  monitors are IP-pinned (`ESM_URL=http://192.168.1.223:8765`) with a 5-failure quiet
  threshold. Worker-side esm calls still use the name; if a worker reports connection
  errors, pin the same way.
- One completeness flip re-deals every seeded fixture — budget TWO reconciliation passes
  (PB-DX27's lesson, reconfirmed by DX28/DX43 cycles).
- `esm update --force` would clobber the customized dispatch skill — never run it here.

**Commit prefix used**: `scutemob-N:` (workers) / `merge:` / `chore:` (collect chores + flag)

## Previous Handoff (preserved for chain context)

**Date**: 2026-08-11..12 (oversight session #8 — three-batch queue run, v3 ranks 8/9/10)
**Workstream**: W6 correctness queue (v3)
**Task**: `scutemob-206` (PB-DX26, merge `1f2ec5d3`) + `scutemob-207` (PB-DX7, merge
`5e5ab073`) + `scutemob-208` (PB-DX8, merge `fbcf495f`) — each dispatch → monitor → collect
in sequence, single worker at a time, each dispatch explicitly user-approved.

**Completed**:
- **PB-DX26 shipped** (rank 8, equip surface): `OOS-CARDS1-3` + `OOS-CARDS1-1` + `OOS-DX3b-1`
  all CLOSED — 21 equip defs gained the ability their printed line promised, `darksteel_garrison`
  its CR 702.67a fortify target, `guardian_project` its `is_nontoken` flip. 0 engine lines.
  Inverse census found 2 defs the brief's roster was structurally blind to (Quietus Spike,
  Sting — print Equip, carry neither marker nor ability, both Inert). A gate-defeat exercise
  found the 38 authored equip costs checked by NO gate → SR-37 R7 extended to Equip/Fortify,
  which exposed a latent R7 scanner bug (`Equip` matched inside `Equipped`). Review flipped
  `the_reaver_cleaver` down, cancelling `sword_of_body_and_mind`'s flip up — **coverage NET
  UNMOVED at 62.8%**, and the struck queue row records why the ~4-6 flip estimate was wrong
  twice in opposite directions. Tests 4,491 → **4,508**.
- **PB-DX7 shipped** (rank 9, SR-19 gate holes, test-only): `OOS-DP7-11` + `OOS-DP9-13` CLOSED,
  riders `OOS-DP10-1` + `OOS-DP9-10` residual **CLOSED gated-not-deferred**. Scanner key
  normalised on the bare name; enum coverage added (79 enums / 1,252 variants / 1,097 variant
  fields); **zero genuinely-unhashed fields found**, so HASH 74 unmoved. OOS-DP10-1's
  "cross-check by value" was actually a floor check one below live; OOS-DP9-10's residual got
  `unordered_iteration_ratchet.rs`. Review 2 HIGH: the new ratchet counted the literal
  `HashSet<` annotation — the MINORITY spelling (27 vs 54 `::new()`) — its own subject matter
  recurring; both HIGH defeats re-executed red against the fixed gates. Tests → **4,527**.
- **PB-DX8 shipped** (rank 10, oracle-text-vs-DSL cross-check, test-only): `OOS-CARDS2-7`
  FILED (it had no registry row — memo-only) and CLOSED; `OOS-DP10-9` **RECORDED not closed**
  (80-entry frozen baseline of dropped-'may'/'choose' defs, fail-closed proven on
  `lightning_bolt.rs`); rider **PB-DX42a SHIPPED** (adjudication §5.1 marked shipped;
  DX42b keeps rank 13 with the rider's disclosed caveat). Inverse census found `CardFace`
  carries its own `oracle_text` — back/adventure faces were invisible to the first draft,
  fixed structurally. Tests → **4,561**. Seeds `OOS-DX8-1..8` filed.
- Coordinator state-sync gaps caught at collect, both the N4 shape: PB-DX26's W6 row carried a
  stale pre-fix-cycle coverage claim (fixed `0b944806`); PB-DX7's worker appended its handoff
  but left the W6 summary row saying "next PB-DX7" (fixed `79f05d9e`). PB-DX8's worker did
  full state-sync — nothing to reconcile.

**Not done / deferred** (inherited set, mostly unchanged):
- Feedback doc rows 2 (FUZZ-CRASH) / 4 / 5 / 6 / 7 / 8 undispatched; **OOS-DX22-8**
  unclassified; **OOS-DX32-1** undiagnosed; OOS-ADJ-1..7 still not rowed into §8.1
  (ADJ-7 rides PB-DX27); `scutemob-127` still backlog. **PB-DX27 not dispatched**
  (queue-next, offered; session ended at /eot).

**Next session candidates** (highest-yield first):
- **PB-DX27** (rank 11): the corpus-wide stale-blocker-note re-check (`OOS-RR3-2` +
  `OOS-CARDS2-8`, ~67-def machine-checkable surface), with `OOS-ADJ-7`
  (blood_moon/magus strip Artifact from artifact lands) riding it. Re-word OOS-DX19-2 per
  OOS-ADJ-3 before any DX42b dispatch (standing note, still undone).
- **OOS-DX32-1 diagnosis** or **FUZZ-CRASH** (feedback row 2, cheapest row).

**Hazards** (carrying forward):
- **The floor lesson is now FIVE consecutive batches**: DX25 family (three), DX26 (census
  short by 2 — Quietus Spike/Sting carry neither marker nor ability, invisible to both
  grep families), DX8 (`CardFace.oracle_text` — a per-FACE prose field the def-level scan
  missed). The inverse-census acceptance criterion has paid for itself every single time;
  keep it in every PB brief.
- **A gate's own vocabulary is a floor too**: DX7's ratchet counted the minority `HashSet<`
  spelling; DX8's whole subject was needle sets the corpus outgrew. When writing any
  source-scan gate, derive the needle set from the corpus and prove the derivation.
- PB-DX8's 80-entry dropped-decision baseline is a WORKLIST, not a closure — `OOS-DP10-9`
  stays open until the defs are re-authored (later card work, not a PB).
- `tests/rules/copy_redirect.rs` still carries 8 disclosed collapsed-id fixtures;
  `OOS-DX25c-6` stays open (resolution-order self-redirect).
- Coordinator ops: Monitor + stdin-JSON pattern held for all three workers (fixed field
  names: `content`/`timestamp`, not `body`/`created_at`). Workers' collect state-sync is
  still inconsistent batch-to-batch — verify the W6 summary row *and* the queue banner at
  every /collect, even when the handoff landed.

**Commit prefix used**: `scutemob-206:` / `scutemob-207:` / `scutemob-208:` (workers) +
`merge:` + `chore:` (collect reconciliations, eot)


## Prior Handoff (oversight #7 — two user-approved queue inserts, ranks 7b + 7c)

**Date**: 2026-08-06 (oversight session #7 — two user-approved queue inserts, ranks 7b + 7c)
**Workstream**: W6 correctness queue (v3)
**Task**: `scutemob-204` (PB-DX25b, merge `8258e715`) + `scutemob-205` (PB-DX25c, merge
`241d82f9`) — each read-the-seed → insert-row → user-approve → dispatch → collect, in sequence.

**Completed**:
- **PB-DX25b shipped** (rank 7b insert): `OOS-DX25-3` CLOSED — the announced-target id-space
  confusion one function over from PB-DX25's. **The brief's "validation-site only" was short by
  three sites** and obeying it would have shipped a cast that announces then silently does
  nothing at resolution; the fix went structural (`stack_registry::stack_index_for_announced_
  target`, consumed by all five sites, reviewer confirmed no sixth by inverse census). The old
  negative tests were green against a fixture that **collapsed the two id spaces** — a
  configuration no real cast can produce. Tests 4,452 → **4,469** (+17); PROTOCOL 35 / HASH 73
  unmoved, gate-executed.
- **PB-DX25c shipped** (rank 7c insert): `OOS-DX25b-3` CLOSED — CR 115.7a redirect legality.
  New hashed `StackObject.target_requirements` + `rules::retarget::plan_target_change`
  delegate the whole redirect decision to `casting::validate_targets_inner`, closing the filed
  object branch AND an unfiled, independently-reachable player-branch defect. `t9` pin
  inverted (+ `t9b` sibling); fix cycle 2 closed `OOS-DX25c-5` (self-redirect onto own card).
  Tests → **4,491** (+22); **HASH 73 → 74 gate-computed exactly as the insert row predicted**;
  PROTOCOL 35 unmoved. Three review cycles, all 32 findings taken; revert matrix honest at
  16/19 discriminating with 3 recorded UNDISCRIMINATED with reasons.
- Both inserts were recorded as v3 §4 rows (7b, 7c) with pointers repointed BEFORE dispatch,
  each explicitly user-approved — no silent repoint. Both workers did full collect
  state-sync; verified (not assumed) at `/collect` both times.
- Seeds: `OOS-DX25-3`, `OOS-DX25b-3`, `OOS-DX25c-5` CLOSED; `OOS-DX25b-1..5` +
  `OOS-DX25c-1..6` filed (registry grep-checked).

**Not done / deferred** (inherited set, unchanged):
- Feedback doc rows 2 (FUZZ-CRASH) / 4 / 5 / 6 / 7 / 8 undispatched; **OOS-DX22-8**
  unclassified; **OOS-DX32-1** undiagnosed; v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7 not
  rowed into §8.1; `scutemob-127` still backlog. **PB-DX26 not dispatched** (queue-next,
  offered; session ended before the go).

**Next session candidates** (highest-yield first):
- **PB-DX26** (rank 8, equip surface; ~4-6 flips; re-measure the 21/18/10 roster from
  `all_cards()` per v3 §2.7). For the first time in three batches there is **no insert to
  weigh first** — the DX25 family is burned down and none of its residual seeds is live-wrong
  on a `Complete` def.
- **OOS-DX32-1 diagnosis** or **FUZZ-CRASH** (feedback row 2, cheapest row).

**Hazards** (carrying forward):
- **The DX25-family lesson, three times over: a filed site list is a FLOOR, not a census.**
  DX25's census was short by two, DX25b's brief short by three, DX25c's filing missed a whole
  player branch. Writing "the site list is a floor" + an inverse-method census criterion into
  the DX25c brief is what recovered the third one — keep doing that in every PB brief.
- A fixture that collapses two id spaces makes a test green by removing the only condition
  under which the code is wrong; `tests/rules/copy_redirect.rs` still carries **8 disclosed
  instances** of the collapsed-id fixture.
- `OOS-DX25c-6` stays open: a resolving spell's own `StackObject` entry is popped before its
  effect runs (resolution.rs's documented order), so `TargetSpellWithSingleTarget` can never
  observe the actively-resolving spell as a redirect candidate.
- Coordinator ops: `esm` dropped off PATH again mid-session (`export PATH="$HOME/.local/bin:
  $PATH"` per call); and a Monitor that interpolates `esm task get` JSON inline into a python
  `-c` string breaks the moment a worker comment carries quotes — pipe the JSON to python via
  **stdin** (the v2 monitor pattern held for both workers).

**Commit prefix used**: `scutemob-204:` / `scutemob-205:` (workers) + `merge:` + `chore:`
(insert bookkeeping, eot)

## Worker Handoff (UI-1, `scutemob-174`)

**Date**: 2026-08-02 (worker session, `scutemob-174` — UI-1 blocking-decision pickers)
**Workstream**: M11-local maintenance track (`crates/simulator`, `tools/play-server`)
**Task**: `scutemob-174` — branch `feat/ui-1-blocking-decision-payload-channel-pickers-discard-scrys`

**Completed** — playtest-triage **F8** closed on the browser surface:
- **Three layers, one mechanism.** `StubProvider` bakes the engine-accepted default into every
  blocking-decision `LegalAction` (cleanup discard = the `count` highest `ObjectId`s, scry/surveil
  = the identity partition, search = `candidates.first()`) so a *bot* can submit it and always be
  accepted (SR-38). The candidate data rides along so a *human* client can render a choice. The
  view layer threw it away, so the browser drew one bare button that submitted the default.
- `crates/simulator/src/params.rs`: `ActionParams` gains `discard_cards` / `effect_choice_answer`
  / `trigger_targets`; the three arms forward an announced answer and fall back to the same
  default as before; the three variants join the allowlist and `first_announced_field`.
- `tools/play-server/src/view.rs`: `ActionOptionView.decision` — a generic
  `{question, prompt, answer_field, answer}` envelope whose `answer` is one of **four shapes**
  (`Subset` / `PickOne` / `Partition` / `Slots`). `ActionParamsDto` gains the three answer fields.
- `tools/play-server/src/api.rs`: `validate_decision_params` — an answer naming something the
  response never offered is a **400**, not an engine 422.
- Frontend: `DiscardPicker`, `PartitionPicker` (scry AND surveil), `SearchPicker`; `ActionBar`
  gains a `'decision'` stage dispatching on `answer.shape`; `TargetPicker` now hands back the
  grouped `Target[][]` alongside the flat list.
- **Tests 4,124 → 4,136** (+8 params unit, +5 play-server). Zero engine lines (empty `git diff`
  over `crates/engine/src` + `crates/card-types/src`), PROTOCOL **33** / HASH **70** unmoved
  (gate-executed, not predicted).

**Durable lessons this session paid for**:
1. **CR 400.7 defeats id-following assertions.** The scry and search probes' first drafts followed
   an `ObjectId` from the library into the hand. A card that changes zones is a NEW object. Both
   now assert over the **library**, where the ids survive — and the two answers are distinguished
   by *which* card is still in it.
2. **A probe can pass on a printed keyword.** The trigger-target probe's first version asserted
   "the chosen creature has a keyword" and **passed against the un-fixed code**, because Nezumi
   Prowler is printed with Ninjutsu. It now asserts what each creature *gains* against a baseline
   taken before the answer. Every probe here was re-checked by reverting the fix and watching it
   go red; that check is what caught this one.
3. **A generic payload's extension claim is worth its test.** `Slots` was built so OOS-DP8-2 would
   need no rework, and it did not — but the claim only became evidence once a real pair of
   `Complete` cards (Shadow Alley Denizen + Nezumi Prowler) drove it end to end. Every other
   mono-black route was checked and rejected: PB-EF6 retargeted them all to `TargetOpponent`,
   which has exactly one candidate in a 2-player game and is therefore always forced.
4. **A fourth Invariant-7 channel, opened deliberately.** `StateViewModel` models `library_size`
   and no library *contents*, so `NameIndex` answers `(unknown card)` for every scry/search
   candidate. `view::question_card_label` reads the name off `GameState` for ids drawn out of the
   engine's own `EffectChoiceQuestion` — whose `private_to()` already classifies exactly that id
   set as this seat's. MR-M11-01's lesson applies verbatim (*a redaction gate checks the channel
   it was written for*), so it ships with its own gate: `view.rs`'s production code may read the
   raw object table exactly **twice**, and a third read must be deliberate.
5. **`session::new_game` is the deck-injection seam.** `config_for` hard-codes
   `DeckSource::RandomPerSeat`, but `new_game` takes any `LocalGameConfig` and runs the same two
   Invariant-9 gates — so a `#[cfg(test)]` fixture can install a `DeckSource::Fixed` session and
   still drive every request through the real router. One seed (184) serves two different fixture
   decks because the shuffle permutes *positions* and both probe spells sit at `main_deck[0..2]`.

**Fix cycle (Opus review, 1 HIGH + 1 MEDIUM + 4 LOW, all closed)** — and the HIGH is the one
worth carrying:
- `question_card_label`'s doc **cited a gate test that did not exist**
  (`test_ui1_a_bot_seats_effect_choice_never_reaches_the_human_payload`) and said the channel
  "ships with its own gate rather than with an argument". That is this project's own defect
  class — a claim in prose that no test holds — landing on the one subsystem MR-M11-01's lesson
  is about. It was a draft line left behind when the planned behavioural test was replaced by a
  source-count gate; the README and the archive entry both described the real situation
  correctly, which is how it survived self-review.
- Worse, the premise that test would have asserted was **enforced nowhere**. The channel's safety
  argument needs the `EffectChoiceQuestion` to belong to the seat being rendered, and that held
  only by arithmetic on a one-element set (`config_for` hard-codes `human_seats: [HUMAN_SEAT]`).
  A second human seat — the obvious M10a direction — would have rendered seat A's scried library
  cards, **with real names**, into seat B's payload. `api.rs::seat_view` now filters
  `pending.player == human`, and `test_ui1_a_foreign_seats_effect_choice_never_reaches_this_payload`
  holds it two-sidedly. **Generalisable**: when a doc comment says "structural", check that the
  structure is in the code and not in the configuration.
- The new test's own first version mutated `pending.player` and did nothing — every route calls
  `advance()`, which refreshes `pending` straight off `LocalGame`. It moves `PlaySession::human`
  instead. Recorded in the test's doc.
- LOWs: the same doc block said "fourth channel" in its heading and "a fifth" five lines later;
  `question_kind`'s rationale claimed redaction while two functions above it format candidate ids
  into their own 400 bodies (corrected to what it is — message quality); `ActionBar`'s decision
  guard required `currentShape`, so a malformed payload rendered nothing and **skipped the very
  fallback that exists to prevent a dead bar**; the count gate's narrowness (one needle, blind to
  `zones()`/`card_registry()`) is now stated rather than implied.

**Re-review (second pass)**: all 6 confirmed fixed and the new gate confirmed two-sided by
execution — but the fix cycle had left 4 doc defects of its own, **two in the same class it was
convened to fix**. That recurrence is the point, not an aside: writing the correction is a second
chance to assert something no test holds. One was substantive — `seat_view`'s comment justified
dropping a foreign decision with "`submit`'s own `seq` check already refuses to act on it", which
is false (`pending_wire_seq` ignores `human`, and the 409 body discloses the current `seq`, so a
client could learn a hidden decision's seq and submit against it). `post_action` now refuses it
too, and the gate asserts both halves using the seq captured *before* the move. Also corrected:
the gate's own description said it retargets the decision (it retargets the viewer); "asserts
every name is gone" overstated a needle that is the `looked_at` **key** (names are not assertable
— seat 2 legitimately holds Swamps); an unresolvable `[check_ids]` intra-doc link; and both the
code comment and the README now say plainly that this pair is **fail-closed, not M10a-ready** —
`PlaySession::human` is a single `PlayerId`, so a real second human seat would be deadlocked
rather than served, and the missing piece is a per-request viewer.

**Confirmation pass (third)**: all 5 second-cycle findings confirmed fixed, both guard halves
confirmed two-sided **by execution** — and the write half turned out to close a real hole, not a
theoretical one: with `post_action`'s guard deleted the probe gets **HTTP 200 and the other seat's
scry is applied**. No new instance of the "guard that is not there" class; one instance of its
**inverse** — three comments still advertising a `seq`-disclosure channel that the guard's own
placement (above the staleness check) had already closed. Closed the same way as everything else
here: the gate now asserts that a wrong `seq` against a foreign decision answers
`no_pending_decision` rather than `stale_decision`, whose body would carry `expected: <the real
seq>`.

**The through-line of three review rounds, worth more than any one finding**: every round found
prose out of step with the code, in one direction or the other, and every round's fix was the
same — *make a test hold the claim*. A comment that says "structural", "gated", "already
refuses" or "discloses" is an assertion; if no test executes it, it decays at the speed of the
code around it. Two of the three rounds' faults were introduced by the correction to the previous
round.

**Not done / deferred** (all recorded as play-server README limitations 14-17):
- The TUI halves of OOS-DP7-6 / OOS-DP8-2 / OOS-DP9-7 are untouched; those rows are *about* the
  TUI and remain open. OOS-DP9-1 is unchanged and deliberately so — it is about the bot, and the
  bot still submits the default, which is what keeps every recorded fuzzer seed reproducing.
- No picker has an automated test (no frontend harness exists, plan §8 R7).
- `Slots` has no "use the default" button; `PartitionPicker` has no ordering control on the moved
  pile (CR 608.2f says that order is the player's).
- Two pre-existing broken intra-doc links in `tools/play-server/src/view.rs` (`GameSummary::seed`,
  `crate::api::validate_combat_params`) predate this branch and were left alone. CI runs
  fmt/clippy/build/tests, not `cargo doc`, so nothing goes red — noted for whoever wants them.

**Next session candidates**: `scutemob-175` (SIM-1 commander cast) or `scutemob-177` (UI-2
additional costs) — UI-2 is the one UI-1 was meant to pre-shape, and its `CostPicker` should slot
into the same `pickerNeeded` chain.

**Commit prefix used**: `scutemob-174:`


## Prior Handoff (oversight — wave-7 recovery, both collects, playtest triage)

**Date**: 2026-08-02 (oversight session — wave-7 crash recovery, both collects, playtest triage)
**Workstream**: coordinator — W6 (PB-DX6 collect) + M11-local (S8 collect, MILESTONE CLOSED) + triage
**Task**: `scutemob-172`/`173` collected; `scutemob-174..181` created; merges `51878905` + `cb0755bf`

**Completed**:
- Both wave-7 crashed workers restarted per the agreed recovery (173 resumed on its WIP,
  172 fresh from plan `4d367c54`; crashed WIP preserved at `wip/scutemob-172-crash-20260802`).
- **`scutemob-173` COLLECTED (`51878905`) — M11-LOCAL COMPLETE**, on-main 4,097/0.
- **`scutemob-172` COLLECTED (`cb0755bf`) — PB-DX6 SHIPPED**, PROTOCOL 32→**33** / HASH **70**;
  combined S8+DX6 tree measured on main: **4,124 / 0 / 5 ignored**. Both tasks `done` in ESM.
- **OOS-M11-10 filed** (`e4b93ac0`): equip `targets: vec![]` silent fizzle — measured 16 of 17
  real equip activations, 10 `Complete` via the `#[default]` derive.
- **First-human-playtest triage**: every claim in `test-data/bot testing notes.md` verified
  against code — `memory/playtest-triage-2026-08-02.md` (F1–F10, ZERO engine bugs; all
  simulator / play-server / card-defs). Corpus-wide mana-cost audit: **17 wrong costs,
  9 deck-legal `Complete`** (`tyrranax_rex` 3 cheap on a 7-drop).
- **Successor tasks `scutemob-174..181` created** (UI-1 pickers, SIM-1 commander cast, SIM-2 mana
  intelligence, SIM-3 invariant residuals, UI-2 additional costs, CARDS-1 equip batch, UI-3 UX
  polish, CARDS-2 field-fidelity gate). 176/177 carry re-baseline comments — S8 already closed
  OOS-M11-8 and rewrote the false-positive `stack_consistency` check.
- **CLAUDE.md line hygiene** (`fdb872b6`): 12 changelog entries rotated to the monthly archives,
  Current State rewrapped at ~100 chars, formatting rule pinned in the file.

**Not done / deferred**:
- `scutemob-174..181` all in backlog, none dispatched (standing directive: every dispatch needs
  explicit user approval). PB-DX7 (SR-19 gate holes, test-only) next in the W6 queue, undispatched.
- This file (`workstream-state.md`) still has its own mega-lines (the W6 table row is 30k+ chars) —
  same disease CLAUDE.md was cured of; treat in a future chore.

**Next session candidates**:
- Dispatch `scutemob-174` (UI-1 blocking-decision pickers) — biggest agency win, pre-shapes UI-2.
- Then `scutemob-181` (field-fidelity gate) + `scutemob-176` (mana intelligence) — parallel-safe.
- Or PB-DX7 in the W6 lane (disjoint from all of the above).

**Hazards** (carrying forward):
- CLAUDE.md formatting rule is NEW: close-outs append a short delta and rotate detail to the
  monthly archive — never grow an existing line.
- Read the ESM comments on 176/177 before dispatching them (scopes shrank post-S8).
- User's `tools/play-server/frontend/package.json` edit left uncommitted deliberately.

**Commit prefix used**: `merge:` / `chore:` / `scutemob-172:` (worker)


## Prior Worker Handoff (PB-DX6, preserved for chain context)

**Date**: 2026-08-02 (worker session, `scutemob-172`)
**Workstream**: W6 (primitives) — **PB-DX6 SHIPPED**, sixth batch of the PB-DX queue
**Task**: `scutemob-172`. Branch `feat/pb-dx6-the-last-two-unflattened-mana-cost-payment-sites-oos-`, 8 commits.

> **Restart note**: this task crashed mid-implement in a prior session and was redone **from
> scratch** from the plan commit `4d367c54`. The crashed WIP survives at
> `wip/scutemob-172-crash-20260802` for reference only; nothing was cherry-picked from it.
> The redo was staged deliberately (0/A/B/C/D/E/F) with a commit per stage, precisely because
> the previous single-pass attempt ran out of room. That staging is the reusable part.

**Completed**:
- **OOS-RS2-1 and OOS-DP4-1 both CLOSED.** `rules/engine.rs::handle_turn_face_up` paid a **raw**
  `def.mana_cost`, and it is **all three** `TurnFaceUpMethod` arms that share the defective payment
  block — not the `ManaCost` arm the dispatch brief named. `Command::DeclareAttackers` gains the
  two PB-RS2 payment fields, so a hybrid or Phyrexian CR 508.1h attack tax is payable rather than
  rejected.
- **Pre-fix numbers were OBSERVED in both build modes before any line changed**, because plan §2.0
  named a trap purpose-built to produce a plausible false claim. In **debug** — every `cargo test`
  run and all of CI — a manifested Kitchen Finks flip **panics** inside `debug_assert_flattened`
  ("2 hybrid + 0 Phyrexian pip(s) would be paid for free"). The "flips for `{1}`" figure is
  **release-only**, produced by temporarily disabling the guard and reading the pool:
  `{1 colorless, 1 G, 1 W}` → `{0 colorless, 1 G, 1 W}`. **That debug panic is the batch's most
  useful finding**: every test build this project has ever run would have caught the bug, and no
  test ever put a pipped cost through the site.
- **Design (A) shipped on evidence, not taste.** Pips are replicated into the CR 508.1h total and
  the total is flattened **once**. Design (B) — flatten each `cost_per_creature`, then multiply —
  is *rules-wrong* on the Norn's Annex ruling of 2011-06-01 ("that player chooses how to pay each
  cost **individually**"), which (B) structurally cannot express, and it fails in the **quiet**
  direction: it would accept the command and charge a legal-but-not-chosen total.
- **The pip order is copy-major** (`[r1, r2, r1, r2, …]`, never `[r1, r1, …, r2, r2, …]`) so that
  "creature *k*'s pips live at offsets `[k·P, (k+1)·P)`" is true — the only form the ruling or a UI
  can be stated against. Written down in all three required places.
- `unpayable_tax_defenders` → **`x_tax_defenders`**, narrowed to X only; a name asserting
  "unpayable" when hybrid and Phyrexian are now payable is a lying identifier of the class this
  suite keeps re-creating. Message now cites CR 107.3/601.2b and the new **OOS-DX6-1**.
- New read-only **`rules::queries::attack_tax_total`**, because the attack tax is the one payment
  cost a client **cannot** derive — `LegalAction::DeclareAttackers` carries no attacker set. Exactly
  **one** accumulation (`accumulate_attack_tax_total`) serves both it and the validation path.
- **`ManaPool::can_spend`/`spend` stop failing OPEN.** `can_spend` is fail-closed on an unflattened
  residue in every build; `spend` asserts unconditionally. The asymmetry is the argument: a question
  has a truthful conservative answer, an instruction has none, and `spend`'s documented precondition
  is `can_spend`. `Result`-returning signatures were rejected because they **launder an engine bug
  into a rules answer** (every caller would `?` it into `InvalidCommand`) — filed as OOS-DX6-3.
- **PROTOCOL 32 → 33 computed** from the failing gate's own output; the falsifier named in advance
  ("if it passes unchanged, stop") did not occur; closure type count unchanged at 96. **HASH
  confirmed unmoved at 70 by running the gate.** 13 sentinels re-pinned by symbol, then confirmed by
  a full `--workspace --no-fail-fast` run whose residual list was **empty**.
- **0 completeness flips**, pre-committed and held — empty `git diff` over `crates/card-defs`, and a
  coverage regeneration whose body came back byte-identical. Coverage holds at **1,137/1,804 =
  63.0%**; no seeded deck re-dealt, so the play-server pins were never touched.
- Tests 4,066 → **4,099**. clippy / fmt / `tools/check-defs-fmt.sh` (1,804 defs) clean; 210 golden
  scripts green, 0 new skips; benches within noise.

**Hazards for the next worker**:
1. **A batch can silently delete another batch's regression coverage.** PB-DP4's two E1 CR 508.1c
   scoping pins both used a **hybrid** restriction — which stopped being a rejection class the
   moment this batch landed, so E1's fix had lost **all** discriminating power. Found in review,
   verified by reverting E1 and watching them stay green, then moved to `x_count: 1`. **When you
   narrow a rejection class, go find every test that was pinning something else through it.**
2. **The review's HIGH, and the reason to distrust your own new doc comments.** The copy-major
   order-pin test **could not fail** under the permutation it existed to catch — copy- and pip-major
   diverge only when one `add_mana_cost` call has `times > 1` **and** more than one pip, which the
   fixture never produced — while the batch's freshly-written `multiply_mana_cost` doc asserted that
   it could. That is the PB-DX5 "verified: none exist" class, reproduced inside the batch that cites
   it twice. **A test named after an invariant is not evidence that it pins the invariant.** Prove
   discrimination by reverting.
3. **A finding established by reading is a hypothesis.** This batch's reviewer had **no shell** and
   said so per-finding; every finding was re-verified by execution in the fix cycle, and one (the
   TUI site count) was wrong on the numbers. Do not apply an unverified finding.
4. **`tools/tui` still hand-builds `DeclareAttackers` with empty payment vectors** (3 sites), so a
   TUI player facing a pipped attack tax gets a rejection with no way to answer it. Zero exposure
   today (no def carries such a tax) and recorded as **OOS-DX6-5** with in-source comments; the UI
   is M11/M13 work.
5. **`attack_tax_total` returns `None` for an all-X tax**, which is not "no tax". The doc says so
   explicitly and `params.rs` carries the SR-38 note; the real fix needs an X-announcement channel
   (**OOS-DX6-1**).
6. **`OOS-DP4-7` is re-dispositioned, NOT closed.** Do not dedup `add_mana_cost` onto
   `multiply_mana_cost`: the latter is **pip-major**, so the "harmless" dedup would silently
   re-order the tax's pips and re-interpret every `hybrid_choices` vector a client had already
   built — no compile error, and no test failure except the new discriminating fixture.
7. **The SR-31 ratchet gained nothing, deliberately and with the reason checked.**
   `turn_face_up:hybrid` is impossible today because `script_schema.rs`'s `PermanentInitState` has
   no face-down field at all, so the JSON regime cannot build the state;
   `declare_attackers:hybrid` has no honest script because no def produces a pipped tax. Both
   recorded beside `CROSS_VALIDATED_SHAPES` rather than left ambiguous.

**Next**: **PB-DX7** (OOS-DP7-11 + OOS-DP9-13 — the SR-19 gate reports success while checking
nothing; gate integrity, 0 flips, test-only, no wire change). Queue authority:
`memory/primitives/seed-rerank-2026-07-27.md` §4.

---

## Prior Worker Handoff (PB-DX5, preserved for chain context)


## Prior Handoff (wave-7 crash + recovery session, superseded 2026-08-02 — preserved for chain context)

> **ADDENDUM 2026-08-02 (coordinator, post-crash recovery session)** — the crash state below is
> resolved: **`scutemob-173` COLLECTED (merge `51878905`) — M11-LOCAL IS COMPLETE**, on-main
> verified **4,097 / 0** (matches the worker's branch pin exactly); task `done` in ESM.
> **`scutemob-172` (PB-DX6) RESTARTED FRESH** per the agreed recovery: branch reset to plan
> commit `4d367c54`, the unverified 94-file WIP preserved as branch `wip/scutemob-172-crash-20260802`,
> new worker running in the same worktree (1/5 criteria, mid-implement). **The equip finding
> below is FILED**: seed **OOS-M11-10** (`e4b93ac0`, audit §8.1) + repair task `scutemob-179` —
> measured roster is **16 of 17** real equip activations (4 of the 22 grep hits are prose-only,
> 1 is a correct triggered self-attach), 10 of the 16 `Complete` via the `#[default]` derive.
> Also this session: the user's full playtest notes were verified claim-by-claim
> (`memory/playtest-triage-2026-08-02.md`, F1–F10 — **zero engine bugs**, everything is
> simulator/play-server/card-def) and a **corpus-wide mana-cost audit** found **17 wrong costs
> (9 deck-legal `Complete`**, incl. `tyrranax_rex` 3 mana cheap); successor tasks
> **`scutemob-174..181`** created in backlog (pickers, commander cast, mana intelligence,
> invariant fix, additional costs, equip batch, UX polish, field-fidelity gate). S8's merge
> re-baselined 176/177: OOS-M11-8 ({X} auto-tap) is CLOSED in-branch by S8, and S8 already
> rewrote the false-positive `stack_consistency` check (task 177 shrinks to tests + one doc line).
> No dispatch of 174..181 without explicit user approval (standing directive).

**Date**: 2026-08-01..02 (oversight session — parallel two-lane waves; wave 7 lost to a kitty crash; /eot 2026-08-02)
**Workstream**: W6 (PB-DX queue) + M11-local track, run as parallel pairs
**Task**: coordinator chain `scutemob-160..173` (waves 1-6 collected; wave 7 crashed in-flight)

**Completed**:
- **Five waves collected and on-main verified this session** (each pair merged + full workspace run):
  PB-DX2+S3 (**3,988**), PB-DX3+S4 (**4,008**), PB-DX3b+S5 (**4,040** after the seed-pin re-pin
  `b24a9685`), PB-DX4+S6 (**4,048**), PB-DX5+S7 (**4,072**). Main is at `f20823b1` (PB-DX5 merge).
  Detail per batch lives in the entries below and in CLAUDE.md Current State.
- **M11-local S7 SHIPPED** (`scutemob-171`, merge `05849372`) — targeting/combat/X/mode pickers;
  the human can attack, block, and cast targeted/X/modal spells in the browser. CLAUDE.md's
  milestone bullet was a session stale on main and is corrected by this /eot.
- **First human playtest of the browser client happened this session** (frontend `npm install`
  + `npm run build`, `cargo run -p play-server`). It works — and it immediately found a real bug
  (see the equip finding below), which is the whole point of first-playable.
- Stray `/tmp/claude-1000/s8-fuzz-baseline` worktree (left by the S8 worker's fuzz-parity
  comparison) removed; both crashed worktrees WIP-committed so `git status` is clean everywhere.

**Not done / crashed (wave 7 — kitty crash killed both worker sessions)**:
- **`scutemob-172` (PB-DX6, mana-payment flattening)**: died mid-implement. Plan committed
  (`4d367c54`, 1/5 criteria); the 94-file partial implement (mid-PROTOCOL-bump) is preserved as
  WIP `18e89bde` but is **UNVERIFIED — do not build on it**. Agreed recovery: reset branch to
  `4d367c54`, redo implement fresh.
- **`scutemob-173` (M11-local S8, closes the milestone)**: died at **4/5 criteria** with
  substantial verified work committed — scripted-human playthrough (5 seeds), fuzz-parity gate,
  `GET /api/game/report`, Concede + OrderBlockers surfacing, measured test pin **4,092**, seeds
  OOS-M11-7/8/9 handled in-branch. In-flight review-fix edits preserved as WIP `c2013efa`.
  Agreed recovery: fresh worker resumes on the existing commits; only the milestone close-out
  criterion remains. **Collect hazard**: this branch already advances CLAUDE.md past M11-local
  and closes the workstream-state M11 table IN-BRANCH — expect coordination-file conflicts.
- Both tasks remain `in_progress` in ESM with recovery comments attached.

**New finding from the user's playtest (UNFILED — next session should file as an OOS seed)**:
- **Equip is unusable from the browser client, and the root cause is corpus-wide.**
  `accorders_shield.rs` (and ~20 of the 22 `AttachEquipment` defs — `skullclamp`, `lightning_greaves`,
  `swiftfoot_boots`, the swords, etc.) declare the equip `AbilityDefinition::Activated` with
  `targets: vec![]` while the effect reads `EffectTarget::DeclaredTarget { index: 0 }`.
  `abilities.rs:537` has a **legacy special-case** that validates a *volunteered* target
  (`targets.first()`) for `AttachEquipment` but **silently accepts activation with no target** —
  mana is paid, the ability resolves, the attach fizzles. The TUI/old paths volunteered targets;
  S7's browser picker only renders slots from *declared* `TargetRequirement`s → empty → no picker
  → no target submitted → exactly the observed "pay mana, click, nothing happens."
  `crates/simulator` has **zero** equip handling (bots never equip), so no fuzz run ever covered it.
  This is the mirror of OOS-M11-5 (targets accepted without requirements ↔ requirements absent so
  targets never asked). Fix directions to weigh: author a real `TargetRequirement` on the equip
  ability corpus-wide (card-def sweep, likely zero engine lines — the general validation path then
  serves the picker for free); make a no-target `AttachEquipment` activation a **hard rejection**
  (CR 601.2c/702.6a — the ability *requires* a target); and check the two defs that looked
  different (`blade_of_the_bloodchief` is `partial` with equip not even authored; verify
  `blackblade_reforged`).

**Policy change (binding, this session)**:
- **Autonomous wave-chaining RETRACTED by the user.** After wave 1 ("dispatch both") the
  coordinator chained five further waves overnight on the strength of the 2026-07-18
  authorization; the user did not want that. `feedback_queue_autonomous_chaining.md` and the
  MEMORY.md index now record the retraction: **every dispatch — including restarting a crashed
  worker — needs explicit user approval; collect what is in flight, then stop and report.**

**Next session candidates**:
- **Resume `scutemob-173` (S8)** — closest to done; closes M11-local. Fresh worker on the existing
  branch, only the milestone close-out criterion left.
- **Redo `scutemob-172` (PB-DX6)** — reset branch to the plan commit `4d367c54`, fresh implement.
- **File + schedule the equip finding** — could ride PB-DX6's close-out or run as a micro-batch
  (card-def sweep shape, PB-DX3's zero-engine-lines pattern).
- Queue then continues at **PB-DX7** (SR-19 gate integrity) per `seed-rerank-2026-07-27.md` §4.

**Hazards** (carrying forward):
- **kitty remote-control socket loss**: `/tmp/kitty-<pid>` vanished mid-session (likely tmpfiles
  aging at the date rollover) leaving RC unusable while kitty ran; a second detached kitty
  instance (`--listen-on unix:/tmp/kitty-claude-workers`) worked but auto-loads the full session
  config (duplicate tabs). Then kitty itself crashed, killing both wave-7 workers.
- **Two Opus workers + their subagents get heavily API-throttled** — waves 5-7 ran 3-5h wall each;
  single-worker dispatch is materially faster per task.
- **Workers can create throwaway worktrees outside `.worktrees/`** (S8's `/tmp/.../s8-fuzz-baseline`)
  which escape `esm worktree list` hygiene — check `git worktree list` at collect.
- The play-server seed pins re-deal on ANY `Complete`-pool change (precedent `b24a9685`) — now a
  standing coupling between card-def batches and `tools/play-server` tests.

**Commit prefix used**: coordinator `chore:` + `merge:`; worker `scutemob-N:`; crash-preservation `wip:`.

---

**Date**: 2026-08-01 (worker session, `scutemob-170`)
**Workstream**: W6 (primitives) — **PB-DX5 SHIPPED**, fifth batch of the PB-DX queue
**Task**: `scutemob-170`. Branch `feat/pb-dx5-cr-6112c-lock-the-affected-set-of-a-resolution-genera`, 8 commits.

**Completed**:
- **OOS-OS7-2 CLOSED — CR 611.2c is implemented.** `ContinuousEffect` gains
  `affected_set: Option<OrdSet<ObjectId>>`. `Some(set)` = generated by the resolution of a spell
  or ability; `effect_applies_to` answers by **membership alone** and never re-consults `filter`,
  `chars` or `obj_zone`. `None` = generated by a **static** ability (CR 611.3a — genuinely not
  locked in), which keeps the live re-evaluation it always had. Populated at exactly one site,
  `Effect::ApplyContinuousEffect`, via the new `rules::layers::snapshot_affected_set`, called
  before the effect is pushed so `calculate_characteristics` cannot see the effect being created.
- **`is_effect_active` was deliberately NOT changed**, against the dispatch brief and the task's
  own acceptance criterion, which name both functions. It takes no `object_id`, so a per-object
  locked set is not expressible there; and an effect whose locked set is empty is still *active*
  (CR 611.2b describes an outcome, not non-existence). Ruled correct in review. Pinned by
  `test_is_effect_active_is_unchanged_by_the_snapshot`.
- **The dispatch row's roster was wrong twice over — the sixth consecutive batch in this suite
  whose published roster was wrong before it started.** Enumerated from `all_cards()` rather than
  grep: **116** defs generate a resolution-time continuous effect; **38** use a mass filter
  (29 `Complete`, 8 `partial`, 1 `known_wrong`), not "9 defs / 7 `Complete`". The grep conjunction
  missed the entire `CreaturesYouControl*` family (27 defs — Craterhoof Behemoth, Purphoros,
  Mirror Entity, Triumph of the Hordes, Unbreakable Formation) because the filter name does not
  begin with `All`, and it counted `elvish_dreadlord`, whose only `ApplyContinuousEffect` mention
  is inside a **blocker-note string**. Three separate arithmetic slips were then caught inside the
  batch itself — mine (37/28), the plan's (its own table summed to 38/29), and the implement
  phase's test count (+16 vs the true +17) — each by re-measuring rather than re-reading.
- **The batch closed a second, larger defect and did not know it until review (OOS-DX5-7).**
  `effect_applies_to`'s source-relative arms require `state.objects.get(&source_id)` to still
  exist. For an instant or sorcery, `ctx.source` is the spell's card object, which
  `resolve_top_of_stack_inner` moves to the graveyard **after** effects run — a new object under
  CR 400.7. So pre-fix, *Triumph of the Hordes*, *Unbreakable Formation*, *Goblin Surprise* and
  *Return of the Wildspeaker* applied to **nobody at all** the moment they resolved, which is a
  strictly bigger bug than the "newcomer wrongly gets it" the seed described. Verified empirically
  in the fix cycle (membership read reverted, both board creatures observed collapsing to their
  printed power), not inferred. It is also the only mechanism by which the batch's own T12 could
  fail pre-fix, so T12 had been mislabelled about what it demonstrated.
- **Fingerprints computed, not predicted**: `HASH_SCHEMA_VERSION` 69 → **70** (mandatory; the
  field is hashed), append-only history row added, 43 sentinels re-pinned by **symbol** grep —
  two of which the single-line grep could not see and only the full workspace run with
  `--no-fail-fast` caught. `PROTOCOL_VERSION` **confirmed unmoved at 32** by running
  `--test core protocol_schema`, the falsifier the plan named in advance. `ContinuousEffect` is
  outside the SR-8 wire closure; `git diff` over `rules/protocol.rs` is empty. The PB-DX1 lesson
  ("anything reachable from `Characteristics` is PROTOCOL too") was the reason to check, and here
  it did not apply.
- **Yield 0 flips, exactly as pre-committed** (`tools/authoring-report.py`: 1,137/1,804 = 63.0%,
  body byte-identical, only the regenerated-date header moved). This is a pure engine correctness
  fix that makes 29 existing `Complete` defs behave correctly; no marker moved, so the seeded-deck
  re-deal hazard from PB-DX4 did not fire and the play-server seed pins were not touched.
- **One existing test was asserting the bug while citing CR 611.2c as its justification** —
  `pb_ac3_dynamic_pt_counts.rs::test_set_both_dynamic_locked_at_resolution` claimed the rule
  required the *filter membership* to be re-evaluated live while only the *value* stayed locked.
  Inverted with the rule text quoted, renamed to
  `test_611_2c_new_creature_after_resolution_does_not_get_the_locked_value`, and **strengthened**
  (exact `Some(1)`, the newcomer's own printed power) rather than loosened. No assertion anywhere
  in the batch was weakened.
- **Review 0 HIGH / 6 MEDIUM / 6 LOW, all 12 applied.** Every MEDIUM was the same shape: *a claim
  recorded as measured that had been reasoned to*. Two of them put a false statement into engine
  source — `snapshot_affected_set`'s doc block asserted "verified: no Layer-≤4 divergence exists
  in the roster", which asked the wrong question (the divergence comes from any Layer-≤4 effect
  that **writes** the characteristic the filter reads, and `inkmoth_nexus` does exactly that).
  Fixed, and a real test added (animate a Nexus, then activate Mirror Entity), which discriminates.
- **Probes discriminate, and that was verified independently rather than asserted.** With the
  read-site membership block disabled, **8 of the 15** probes fail (mass -1/-1 newcomer,
  Craterhoof newcomer, control-change retention, Jitte, SBA-after-debuff, phased-out exclusion,
  PB-DP9 abort-and-replay, Layer-≤4 divergence) and exactly the 7 that must be insensitive stay
  green (static anthem in **both** directions, `SingleObject` unchanged, `is_effect_active`
  unchanged, CR 400.7 leave-and-return, phase-in).

**Numbers**: tests 4,048 → **4,066** (+18). Benchmarks within ~1% of the merge base
(`full_turn_4p`, `priority_cycle_4p`, `sba_check`, `board_wipe_4p`; the last, flagged as most
likely to move, measured slightly *faster*) — the snapshot runs once per resolution, not per
layer pass. `cargo clippy -D warnings`, `cargo fmt --check` and `tools/check-defs-fmt.sh` (1,804
defs) clean.

**Seeds**: **OOS-DX5-1..7** in `docs/audits/decision-point-audit.md` §8.1. OOS-DX5-6 was filed as
a checked non-finding and **reopened as a real finding by the fix cycle**; OOS-DX5-7 (the
source-retirement class above) was found only by review.

**Durable lesson for the next batch.** Three arithmetic slips and two false "verified" claims all
came from the same move: writing down a number that was derived rather than read. Every one was
caught by re-running the measurement, and none by re-reading the prose. The corollary that cost
the most here is narrower and worth carrying: **a doc comment that says "verified: none exist"
is a dated claim about a question someone chose**, and the question can be wrong even when the
answer to it is right.

**Left for the collector**: `CLAUDE.md` Current State + Last Updated (updated in-branch by this
worker). `main` moved during this session (`scutemob-171`, M11-local S7) — merge base is
`d568615b`; `tools/` is untouched in both directions, so a `git diff main -- tools/` right now
shows S7's work, not this branch's.

**Commit prefix used**: `scutemob-170:`.

---

## Handoff History

### 2026-08-05 (oversight #6 — v3 rank 7, single dispatch) [rotated]
**Date**: 2026-08-05 (oversight session #6 — v3 rank 7, single dispatch)
**Workstream**: W6 correctness queue (v3)
**Task**: `scutemob-203` (PB-DX25, merge `f8ed9618`), dispatched and collected same evening.

**Completed**:
- **PB-DX25 shipped** (rank 7): counter-on-mutate silent no-op closed. Structural fix — a new
  engine-side `state::stack_registry::card_in_stack_zone`, exhaustive over `StackObjectKind`
  with no wildcard, consumed by BOTH counter paths, so a 28th kind is a compile error until
  classified. The simulator's `stack_card_of` deliberately NOT unified with it (a verifier
  reading the engine's own answer goes silent on exactly the defect it exists to catch).
- **The seed and the queue row had the live shape backwards**: (c) was the only live shape;
  (a) was never independently reachable — Ward cannot reach a mutate spell because the mutate
  target rides `AdditionalCost::Mutate` and never enters `spell_targets` (`OOS-DX25-1`) — so
  (a) is what fixing (c) ALONE would have created, a permanent `ZoneId::Stack` leak in place
  of a silent no-op. (b) is unreachable three independent ways. Live-wrong population
  re-measured: **66** pairs, not the row's implied 144. All corrections written into the
  registry row and the v3 §3 row in place.
- Tests **4,435 → 4,452 / 0 / 5**; PROTOCOL **35** / HASH **73** gate-executed and unmoved
  (prediction held); coverage unmoved **1,133/1,803 = 62.8%** proven by regeneration; benches
  within noise (`full_turn_4p` 214-215 µs).
- Review 0 HIGH / 6 MEDIUM / 3 LOW + 7 folded notes, **all taken** — its sharpest findings
  were the batch's own failure mode recurring inside it (a census short by two sites; a roster
  blind to a delegating variant; a non-vacuity assertion comparing a fixture to itself).
- **OOS-SIM3-5 CLOSED**; **OOS-DX25-1..6 filed** (registry grep-checked per the dedup rule).
  Worker did FULL collect state-sync (queue row struck, W6 row, CLAUDE.md delta, registry) —
  verified, not assumed, at `/collect`.

**Not done / deferred** (inherited set, unchanged):
- Feedback doc rows 2 (FUZZ-CRASH) / 4 / 5 / 6 / 7 / 8 undispatched; **OOS-DX22-8**
  unclassified; **OOS-DX32-1** undiagnosed; v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7 not
  rowed into §8.1; `scutemob-127` still backlog.

**Next session candidates** (highest-yield first):
- **Read `OOS-DX25-3` first — LIVE on 2 deck-legal `Complete` defs**: `misdirection` and
  `bolt_bend` can NEVER resolve a legal target (`TargetSpellWithSingleTarget` compares a card
  id to a stack-entry id across disjoint id namespaces; the in-src negative tests pass
  vacuously). Weigh as an insert before PB-DX26.
- **PB-DX26** (rank 8 — the equip surface one link earlier; ~4-6 flips; re-measure the
  21/18/10 roster from `all_cards()` at dispatch per v3 §2.7).
- **OOS-DX32-1 diagnosis** or **FUZZ-CRASH** (feedback row 2, cheapest row).

**Hazards** (carrying forward):
- The three standing #5 hazards (registry-grep dedup rule; Monitor over bash poll loops;
  verify worker state-sync at `/collect`) all held this session — PB-DX25's worker synced
  fully.
- New from PB-DX25: `next_object_id` mints stack-entry ids and card ids from ONE counter, so
  an id lives in exactly one namespace — any `so.id == <card id>` comparison type-checks and
  can never match. `OOS-DX25-3` is a second instance of the same class one function over from
  the seed's. Grep for the pattern before trusting any stack-lookup-by-id.

**Commit prefix used**: `scutemob-203:` (worker) + `merge:` + `chore:` (eot)

### 2026-08-04..05 (oversight #5 — correctness-queue run, ranks 2/3/5/6) [rotated]

**Date**: 2026-08-04..05 (oversight session #5 — correctness-queue run, v3 ranks 2/3/5/6)
**Workstream**: W6 correctness queue (v3)
**Task**: five tasks: `scutemob-199` (OOS-FB1 filing — DUPLICATE of `scutemob-195`, deduped at
`/eot`, see hazards; `e7edcdd1`), `scutemob-198` (PB-DX20, merge `ecd7b119`), `scutemob-200`
(PB-DX21, `e490153b`), `scutemob-201` (PB-DX23, `49958549`), `scutemob-202` (PB-DX24, `7b3d7d58`).

**Completed**:
- **PB-DX20 shipped** (rank 2): offer layer sees keyword-carried target requirements — ONE shared
  derivation (`casting::enchant_target_to_requirement`); 13 `Complete` Auras castable in the
  browser; Reconfigure synth site carries `exclude_self: true` (CR 702.151a); the whole
  `KNOWN_FALSE_OFFERS` excusal mechanism deleted. Brief correction: the "4 no-Enchant Auras" set
  was a grep artefact — the T4 roster gates over `all_cards()` (SR-36).
- **PB-DX21 shipped** (rank 3): CR 508.1 once-per-combat guard. The brief's PREFERRED mechanism
  (read `combat.attackers`) was **refuted three ways** (CR 508.1a "if any" + CR 508.8: an EMPTY
  declaration is a completed action, live via `params.rs:474`) → `CombatState::attackers_declared`
  bool, **HASH 72 → 73 gate-computed**. Both client-side mitigations deleted; 3 discriminating
  probes; refuted advice left standing in the brief with the reasoning.
- **PB-DX23 shipped** (rank 5): `LegalAction::ChooseDredge` end-to-end (bot + browser);
  Grave-Troll draw-cadence probe on a real game; OOS-DX2-2 tail flip with the PB-DP5 §3.3
  distinction argued in the commit; OOS-DX2-7 AUTO-CHOSEN row added; **OOS-DX2-3 stays REOPENED**,
  pin byte-unedited.
- **PB-DX24 shipped** (rank 6): `trigger_zone` honoured structurally at the single lowering call
  site (not 34 per-arm edits); graveyard death dispatch built — beyond the brief, the sweep
  handled only `PermanentEnteredBattlefield`; OOS-DX1-4 six of seven sites fixed, seventh
  re-scoped with reason; both seeds CLOSED with their own row-claim corrections.
- Tests **4,373 → 4,435 / 0 / 5**; full suite re-verified on merged main after EVERY collect
  (4,388 / 4,398 / 4,413 / 4,435, all exit 0); PROTOCOL **35** unmoved throughout; HASH **72 → 73**
  (PB-DX21 only); coverage unmoved **1,133/1,803 = 62.8%**.
- **OOS-FB1 double-filing found and deduplicated at `/eot`**: `scutemob-199` re-filed what
  `scutemob-195` had already filed (stale "NOT filed" CLAUDE.md bullet); nine duplicate rows
  removed, the chain-verified `scutemob-199` set kept with the older set's two unique facts
  folded in; banners corrected in registry + feedback doc + CLAUDE.md.

**Not done / deferred**:
- Feedback doc rows 2 (FUZZ-CRASH) / 4 / 5 / 6 / 7 / 8 still undispatched; **OOS-DX22-8** still
  unclassified; **OOS-DX32-1** still undiagnosed.
- Inherited: v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7 not rowed into §8.1; `scutemob-127`
  still backlog.

**Next session candidates** (highest-yield first):
- **PB-DX25** (rank 7 — `Effect::CounterSpell`'s three stack-object shapes; a countered spell
  resolves anyway, silently). Table-only rank: write the brief at dispatch from the seed rows,
  re-verify premise first.
- **OOS-DX32-1 diagnosis** or **FUZZ-CRASH** (feedback row 2, cheapest row, OOS-DX22-7 feeds it).
- **OOS-ADJ-1..7 rowing into §8.1** (small, closes an inherited deferral) — grep the registry for
  each ID first, per the new dedup rule.

**Hazards** (carrying forward):
- **Seed-filing dedup rule (new, learned the hard way)**: before filing any OOS seed, grep
  `docs/audits/decision-point-audit.md` for the ID — the registry is ground truth; status bullets
  in CLAUDE.md/handoffs lag it (OOS-FB1-1..9 was double-filed exactly this way).
- Monitor tool over bash poll loops for worker watches — bash loops were killed within ~2 min
  repeatedly this session; one persistent Monitor per worker was reliable.
- Workers now do their own collect state-sync inconsistently (DX21/DX24 fully, DX20/DX23
  partially) — `/collect` step 7 must still verify the queue-memo row strike + brief banner.

**Commit prefix used**: `scutemob-N:` (workers/self-task) + `chore:` (collects, eot) + `merge:`

### 2026-08-03..04 (oversight #4 — FEEDBACK-1 + first two feedback-buildout batches) [rotated]

**Date**: 2026-08-03..04 (oversight session #4 — FEEDBACK-1 + the first two feedback-buildout
batches, user-directed "stop after 3 tasks for a check-in")
**Workstream**: W6 correctness queue + feedback-engineering track
**Task**: four tasks dispatched/collected serially: `scutemob-192` (FEEDBACK-1 planning, merge
`d55e74cc`), `scutemob-195` (OOS-FB1 seed filing, coordinator-inline, `9aa4f220`),
`scutemob-196` (PB-DX22, `95f53b78`), `scutemob-197` (PB-DX32, `685aa1c4`).

**Completed**:
- **FEEDBACK-1 shipped** (doc-only): `docs/mtg-engine-feedback-engineering.md` — 14-channel
  inventory, 8-row ranked proposal table, alpha-loop ownership table, 18 from/to corrections.
  Registered in `.claude/docs.yaml` (25 templates) + the CLAUDE.md primary-docs table. Its four
  coordinator-notes (decision gate exists / crash pipeline absent / two rows already queued /
  rejection channel bot-only) are in the ESM task comments (scutemob-183 pattern).
- **OOS-FB1-1..9 filed** into `docs/audits/decision-point-audit.md` §8.1 (`scutemob-195`).
- **PB-DX22 shipped** (v3 rank 4): fuzzer shuffles from the game's seeded RNG + registers
  commanders in both builders (new shared `crates/simulator/src/fuzz_setup.rs`); first-cast
  turn 143-154 → **3-29 band**; CR 903.8/903.9a/903.10a fuzzed for the first time; fuzz games
  END (20 wins / 0 errors vs 9/11 timeouts). The §2.4 open measurement settled: the commander
  offer was SUPPRESSED (empty `commander_ids`), OOS-SIM1-4 the cause. **OOS-UI2-1 / OOS-SIM3-1 /
  OOS-SIM1-4 CLOSED**; OOS-DX22-1..11 filed; every pre-merge fuzz seed dead (OOS-DX22-7); the
  repaired instrument's first real find is **OOS-DX22-8** (attachment_validity transient).
- **PB-DX32 shipped** (v3 rank 19, PROMOTED per feedback doc §2.3): `GameResult` carries the
  SR-38 rejection invariant + promoted waste tally behind measured-at-HEAD ratchets (2.30%
  rejection rate; wasted taps 1,986/2,641); orphan-token noise floor gets the transient/end-state
  treatment; violations deduped by condition; fuzz deck pool size gated (**OOS-CARDS2-3 CLOSED**);
  decision-point runtime coverage counter (reached-vs-ROWS). **OOS-SIM3-3 / OOS-SIM3-4 CLOSED,
  OOS-SIM3-2 PARTIAL**; OOS-DX32-1..10 filed. Review 0 HIGH / 8 MEDIUM / 10 LOW, all 18 taken.
- Tests **4,345 → 4,373 / 0 / 5**; PROTOCOL **35** / HASH **72** unmoved by every batch,
  gate-executed each time; coverage unmoved **1,133/1,803 = 62.8%**.
- **Lean-bullet evaluation gate PASSED** at `/start` (UI-6 reconstructed from its bullet plus one
  pointer-follow) — the lean form stands, no rollback.

**Not done / deferred**:
- Feedback doc rows undispatched: **2 FUZZ-CRASH** (now the cheapest row; OOS-DX22-7 feeds it),
  **4 HTTP-FUZZ** (yield gated on OOS-SIM6-3), **5 R7-HARNESS**, **6 DECK-CHANNEL** (re-rolls
  seeds again — batch with card-def work), **7 CI-POLICY** (needs the OOS-FB1-6 timing
  measurement first), **8 REPORT-LOOP**.
- **OOS-DX22-8** unclassified (classify before fixing — OOS-M11-7 SBA-lag family candidate);
  **OOS-DX32-1** undiagnosed (player_consistency = 26.8% of a run, now what --stop-on-error
  halts on).
- Inherited from oversight #2: v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7 not rowed into
  §8.1; `scutemob-127` still backlog.

**Next session candidates** (highest-yield first):
- **OOS-DX32-1 diagnosis** (PB-DX32's flagged successor) or **FUZZ-CRASH** (feedback row 2).
- **PB-DX20** (standing queue next — 13 `Complete` Auras unplayable in the browser).
- **OOS-SIM6-3** (unlocks HTTP-FUZZ row 4's yield and 62 of 113 residual bot refusals).

**Hazards** (carrying forward):
- `esm task create --criteria` is REPEATABLE, not pipe-separated — a pipe-joined string becomes
  ONE mega-criterion (scutemob-196 shipped that way; workable, avoid). A `backlog` task cannot
  be archived; reuse it rather than recreate.
- Every fuzz baseline pinned before `95f53b78` is dead (OOS-DX22-7) — re-measure at HEAD, never
  quote SIM-3/SIM-5 numbers.

**Commit prefix used**: `scutemob-N:` (workers/self-task) + `chore:` (collects) + `merge:`

### 2026-08-02 (oversight #3 — playtest-triage-2 successor run, rows 2-8) [rotated]


**Date**: 2026-08-02 (oversight session #3 — playtest-triage-2 successor run, rows 2-8)
**Workstream**: playtest-triage-2 successor track (SIM/ENG/UI)
**Task**: seven tasks dispatched serially and collected same-day: `scutemob-187` (SIM-4, merge
`dcb1fe55`), `scutemob-188` (SIM-5, `e185a2ff`), `scutemob-189` (SIM-6, `ee99929d`),
`scutemob-190` (UI-5, `08dc4e6a`), `scutemob-191` (ENG-1, `a3b5e56b`), `scutemob-193` (ENG-2,
`4ab68fdc`), `scutemob-194` (UI-6, `dd5cb47d`). **The triage-2 successor table is COMPLETE (8/8
rows shipped)**; every row ✅-marked in `memory/playtest-triage-2026-08-02b.md`.

**Completed**:
- **G2/G5/G4/G8+G10-13/G3/G7/G9 all CLOSED** — per-batch detail in each Worker Handoff above and
  the lean CLAUDE.md bullets (per the new `memory/decisions.md` 2026-08-02 lean-bullet schema,
  first applied this session; ENG-1/ENG-2/UI-6 workers wrote theirs in-schema unprompted).
- Tests **4,263 → 4,345 / 0 / 5** across the run; PROTOCOL **33 → 35** / HASH **70 → 72** (ENG-1
  and ENG-2, both gate-computed); coverage unmoved **1,133/1,803 = 62.8%**.
- **FEEDBACK-1 created** (`scutemob-192`, backlog): planning task for the alpha feedback-loop
  buildout (HTTP browser-path fuzzer, rejection/waste/decision-point invariants, R7 harness,
  steered decks, CI integration). **Deliberately NOT dispatched — user wants a fresh session.**
- Ceremony decision recorded (`memory/decisions.md` 2026-08-02): lean close-out bullets, cut
  explanation never identifiers, lean dispatch briefs from ENG-2 onward. Evaluation gate = next
  `/start` reconstructing the run from lean bullets.
- Mid-run incident: kitty crashed during ENG-2 (`scutemob-193`); worktree survived with 9 clean
  commits; worker relaunched with a verify-don't-reimplement resume prompt (user-approved) and
  re-ran the browser verification whose evidence died with the crash.

**Not done / deferred**:
- **FEEDBACK-1 (`scutemob-192`) dispatch** — waits for a fresh Claude Code session by user request.
  - **→ DONE 2026-08-03** (oversight #4): dispatched, collected, merge `d55e74cc`, doc-only
    (`docs/mtg-engine-feedback-engineering.md`); handoff lives in ESM task comments
    (scutemob-183 pattern); OOS-FB1-1..9 specified in doc §5 but NOT yet filed.
- Inherited from oversight #2 (see Previous Handoff): v3 §4 not re-rowed with DX42a/b; OOS-ADJ-1..7
  not rowed into `decision-point-audit.md` §8.1; `scutemob-127` still backlog.
- Successor candidates flagged by workers, unranked: **OOS-SIM6-3** (bot/human mana-cost
  activation auto-tap — 62 of 113 residual refusals), **OOS-ENG1-9** (suspend-rollback question
  labels), **OOS-ENG2-1+2** (Ward never fires on a triggered ability).

**Next session candidates** (highest-yield first):
- **Dispatch FEEDBACK-1** (`scutemob-192`) from the fresh session — brief is complete in ESM.
- **PB-DX20** (v3 queue next) or the worker-flagged seeds above once FEEDBACK-1's plan lands.
- Third human playtest — the run closed every functional finding from playtest 2; the success
  criterion adopted for the feedback plan is "playtest 3 triages to UX-only".

**Hazards** (carrying forward):
- All oversight-#2 hazards stand (verbatim working_branch attests; commit brief inputs to main
  pre-dispatch; both-append CLAUDE.md conflicts → union-merge, demote to Prior).
- kitty crash kills all worker tabs but NOT worktrees — recovery = relaunch in the same worktree
  with a resume prompt; check `git log main..HEAD` + `git status` before assuming loss. Worker
  relaunch requires explicit user approval (retraction rule).
- `~/.local/bin` can drop off the coordinator shell PATH after a kitty crash — `export
  PATH="$HOME/.local/bin:$PATH"` per call.

**Commit prefix used**: `scutemob-N:` (workers) / `merge:` / `chore:`


### 2026-08-02 (oversight #2 — OOS pivot: re-rank v3, triage 2, PB-DX19, UI-4, adjudication) [rotated]

**Date**: 2026-08-02 (oversight session #2 — OOS pivot: re-rank v3, triage 2, PB-DX19, UI-4, adjudication)
**Workstream**: W6 correctness queue + playtest-triage-2 track
**Task**: five tasks dispatched and collected same-day: `scutemob-182` (seed re-rank v3, merge
`131716d6`), `scutemob-183` (playtest triage 2, `99aba4a8`), `scutemob-184` (PB-DX19, `451e3517`),
`scutemob-185` (UI-4, `b031d39e`), `scutemob-186` (adjudication, `8b069ae2`).

**Completed**:
- **Queue re-ranked twice with evidence**: v3 memo (`seed-rerank-2026-08-02.md`, PB-DX7..DX41; the
  v2 queue had never seen PB-DX1..DX5's 29 seeds), then adjudication `scutemob-186` inserted
  PB-DX42a (rider on DX8) / PB-DX42b (rank 13) — v3 §4 table NOT re-rowed, read with adjudication §5.
- **OOS-SIM2-6 (only HIGH) + OOS-SIM2-5 CLOSED** (PB-DX19): fuzzer 0/15 SIGABRT → 15/15 completed;
  29 checked-arithmetic edits incl. two sign-wrapping `as i32` casts; known pinned deviation:
  animated Nexus no longer feeds Metalcraft (OOS-ADJ-1/OOS-DX19-2 → PB-DX42b).
- **UI-4 (G1) SHIPPED**: Confirm was dead in all three pickers (`structuredClone` on `$state`
  proxy); five CR flows (search/scry/surveil/sac-costs/Squad) work in a browser for the first time;
  R7 harness proposed with the `$state()` fixture rule; two source gates + `$viewer` scan hole fixed.
- **Playtest triage 2** (`playtest-triage-2026-08-02b.md`, G1-G13): 5 new defects (G1 UI-4 done;
  G2 mulligan re-rolls decks CR 103.5; G3 effect-discard has no decision point; G4 activation-cost
  payment channel absent; G5 non-atomic auto-tap), 1 known limitation (G6=R4), 6 UX items; proposed
  tasks UI-5/UI-6/SIM-4/5/6/ENG-1/2 with sequencing constraints (SIM-5∦SIM-6; ENG-1+2 may merge).
- **Adjudication**: external review's durable architecture CR-correct, its immediate patch has no
  CR warrant (613.8b = timestamp order, never inactivity); deviation measured at 7 deck-legal pairs;
  seeds OOS-ADJ-1..7 (registry-of-record: adjudication §6) incl. OOS-ADJ-7 blood_moon strips
  Artifact card type (ride PB-DX27).
- CLAUDE.md wave-4 rotation completed (CARDS-1/SIM-1 bullets to archive, 711→678 lines); external
  findings doc + testing notes 2 committed (`277e60d7`).

**Not done / deferred**:
- v3 §4 table not re-rowed with DX42a/b (pointer note in this file instead).
- OOS-ADJ-1..7 not rowed into `decision-point-audit.md` §8.1 (adjudication §6 is
  registry-of-record until then).
- Triage-2 successor tasks (SIM-4/5/6, ENG-1/2, UI-5/6) not created in ESM yet.
- Tests full-tree re-measure after UI-4 merge pending (4,281/0/5 at `451e3517`; play-server 57
  green at `b031d39e`; nominal 4,283).

**Next session candidates**:
- **SIM-4** (G2 mulligan deck-swap, ~40-60 lines, needs the deck-unchanged-across-redeal gate) —
  highest user-visible value.
- **PB-DX20** (v3 queue next; re-word OOS-DX19-2 framing per OOS-ADJ-3 before any DX42b dispatch).
- **PB-DX8 + DX42a rider** (test-only gate pair).
- UI-5 UX batch (brief must forbid hiding TapForMana; resolve shared-component question up front).

**Hazards** (carrying forward):
- Attest `working_branch` with the LITERAL string from `esm worktree create` output — a command
  substitution can race and record empty, and an empty attest breaks `esm worktree check/merge`
  (fall back: `git merge-tree --write-tree main <branch>` + manual merge; hit on `scutemob-186`).
- Any input doc a task brief references MUST be committed to main BEFORE dispatch — worktrees
  branch from main and do not see untracked coordinator files (hit on the external findings doc).
- Both-append CLAUDE.md/workstream-state conflicts remain routine in parallel waves: union-merge,
  demote the older bullet to Prior.

**Commit prefix used**: `scutemob-N:` (workers) / `merge:` / `chore:`

---

## PB-DX28 handoff (`scutemob-210`, 2026-08-14) — the untargeted-choice class + the owner axis

**Shipped**: `OOS-DX4-6` and `OOS-DX4-1` both CLOSED. v3 queue rank 12. Tests **4,634 / 0 / 5**,
coverage **1,136/1,803 = 63.0%** (0 flips), **PROTOCOL 36 → 37 / HASH 75 → 76**.
Filed **OOS-DX28-1..10**.

**What shipped**

* `EffectTarget::ChosenObject { zone: ChoiceZone, filter, count, up_to }` — a choose-on-resolution
  channel riding the existing CR 608.2d suspend-and-replay machinery as
  `EffectChoiceQuestion::ChooseObject`. 18 `Complete` defs migrated.
* `TargetFilter.owner: TargetOwner` (CR 108.3) and
  `TriggerCondition::WheneverCreatureDies.owner`, distinct from CR 109.4 control.
* `EffectTarget::DamagedPlayer`, repairing `sword_of_war_and_peace`.

**Durable lessons, in the order they cost the most**

1. **A census is a floor, including your own.** The plan's §0.1 enumerated 17 members by slot
   arithmetic. The batch's own inverse gate then found an 18th (`Connive // Concoct`) *after* the
   roster was pinned. The implement run proposed deferring it on scope-discipline grounds; that
   was reversed, because closing a class while a known deck-legal `Complete` member keeps the
   defective shape closes it on a false premise. **The reversal is what found the R3 hole** — the
   walk enumerated `Triggered`/`Spell`/`Activated` and a split card's half is a `Fuse`, so R3
   could not distinguish a migration it could not SEE from one that had not happened. That hole
   was unreachable for as long as no member used the missing variant, so deferring would have
   shipped it undetectable.
2. **A subtraction-shaped gate cancels.** R4 compared declared slots against `"target"` word
   count. One planted sentence — "becomes the target of a spell" — in the *same ability* supplied
   both sides and kept every row green. ~39 corpus defs already carry such phrasing. Hardened by
   stripping non-slot idioms, and the doc's "the class cannot silently regrow" was **withdrawn**
   rather than left standing: it was stronger than the row supports.
3. **A gate keyed on a def NAME cannot see a change *inside* that def.** Moving a `ChosenObject`
   to an unsupported effect arm kept all five roster rows green; in release the failure is a
   silent resolve-to-empty, because the `debug_assert` that catches it is compiled out. New R5.
4. **Do not transcribe a measurement — print it.** The execution notes' "verbatim" gate output
   quoted two fingerprints that `git log --all -S` proves have never existed in any source file
   here. The shipped bump was fine; the *evidence* for it was fabricated, in the record for the
   one criterion that says the numbers must come from the gates' own output. This is PB-DX8's
   rule, broken one batch after it was written down.
5. **A brief's enforcement-site list has now been short in six consecutive batches.** Here the
   auto-target picker turned out to be two functions, not one.

**Standing hazards for whoever takes the next batch**

* **`OOS-DX28-1`**: a hand-maintained field-set fingerprint (`pb_dx42a`'s `TARGET_FILTER_FIELDS`)
  goes blind corpus-wide on ANY `TargetFilter` field addition — no compile error, and a failure
  message pointing nowhere near the cause. Nothing has enumerated how many such fingerprints
  exist. **If your batch adds a field to a widely-reached struct, expect this class.**
* **`OOS-DX28-9`**: `ChooseObject` produces no decision-coverage row, so the new decision point is
  invisible to `decision_coverage.rs`.
* **`OOS-DX28-10`**: direction 2 (no CR 608.2b fizzle) is proven structurally — `t8` shows the
  trigger carries zero declared targets, so the defect is unreachable by construction — but never
  end to end with a removal spell cast in response.
* **The `[profile.fuzz]` seeds move.** 17 card defs changed shape, so per `OOS-CARDS2-3`'s
  corpus→seed coupling any recorded fuzz seed from before this merge is re-dealt.
* **Next dispatch is NOT mechanical.** The v3 memo's rank 13 is PB-DX42b, but `OOS-DX27-9`
  already recorded that its rank premise (a layer-querying population of exactly 1) is false.
  The order past rank 12 needs re-deciding, not reading off.
