# Card Authoring Operations Plan

> **Purpose**: Operational runbook for completing all 1,743 card definitions with zero TODOs
> and zero wrong game state. Covers triage, fix, authoring, and audit phases.
>
> **Prerequisite**: PB-22 (deferred cleanup) must be complete before Phase 1 begins.
> All 22 primitive batches (PB-0 through PB-21) plus PB-22 deferred cleanup provide
> the full DSL vocabulary needed for card authoring.
>
> **Created**: 2026-03-21
> **Status**: ACTIVE — infrastructure complete, triage next

---

## Implementation Order

Every task is numbered. Dependencies are explicit. A session picks up at the first
unchecked item and works forward. **Do not skip ahead** — each task assumes prior
tasks are complete.

### Infrastructure (build before any card work)

- [x] **I-1**: Create `card-fix-applicator` agent (`.claude/agents/card-fix-applicator.md`)
- [x] **I-2**: Update `bulk-card-author` agent — add PB-0 through PB-22 DSL patterns,
      new reference cards, expanded known-issue list, increase MCP budget to 30
- [x] **I-3**: Update `card-batch-reviewer` agent — add all HIGH patterns from waves
      001-003, add TODO-validity check, add `etb_tapped` oracle cross-check
- [x] **I-4**: Create `/triage-cards` skill (`.claude/skills/triage-cards/SKILL.md`)
- [x] **I-5**: Create `/author-wave` skill (`.claude/skills/author-wave/SKILL.md`)
- [x] **I-6**: Create `/audit-cards` skill (`.claude/skills/audit-cards/SKILL.md`)

### Phase 0: Triage

- [x] **T-1**: Refresh DSL gap audit — scan all 742 card defs for TODOs, classify each
      against current DSL capabilities (PB-0 through PB-22). Output:
      `memory/card-authoring/dsl-gap-audit-v2.md` (569 TODOs: 143 now-expressible, 96 partial, 313 blocked, 17 stale)
- [x] **T-2**: Re-evaluate 28 blocked sessions in `_authoring_plan.json` — 19 unblocked
      (→ ready), 15 remain blocked. Updated JSON.
- [x] **T-3**: Re-evaluate 6 deferred sessions — all reclassified (4 → ready, 2 → blocked).
- [x] **T-4**: Parse all 73 existing review findings (Phase 1 batches 01-20, Wave 002
      batches 01-38, Wave 003 batches 01-15). 29 HIGH (22 fixed, 3 valid, 2 DSL gap),
      ~60 MEDIUM (~45 fixed, 7 valid, 3 DSL gap). Output:
      `memory/card-authoring/consolidated-fix-list.md`
- [x] **T-5**: Inventory 264 pre-existing defs not in authoring plan — 197 clean,
      21 fixable now, 7 wrong-state (dangerous), 32 still blocked.
- [x] **T-6**: Write triage summary with updated counts. Output:
      `memory/card-authoring/triage-summary.md`
- [x] **T-7**: Commit: `W6-triage: Phase 0 complete` (9a27d9c)

### Phase 1: Fix Existing Card Definitions

Requires: T-7 complete.

- [ ] **F-1**: Apply all HIGH fixes from consolidated fix list (~30 findings, 3 sessions
      of ~10 cards each). Per session: read finding → read def → fix → mark fixed.
      Build after each session. Commit per session:
      `W6-fix: HIGH findings session 1 — <description> (<N> cards)`
- [ ] **F-2**: Apply all MEDIUM fixes from consolidated fix list (~53 findings, 5-6
      sessions of ~10 cards each). Same procedure as F-1. Commit per session:
      `W6-fix: MEDIUM findings session N — <description> (<N> cards)`
- [ ] **F-3**: Apply LOW fixes (opportunistic, batch into 1-2 sessions). Commit:
      `W6-fix: LOW findings — <description> (<N> cards)`
- [ ] **F-4**: Re-author cards whose TODOs are now expressible (from T-1 "now expressible"
      list). Sessions of 8-12 cards. For each card: look up oracle, read reference def,
      implement full ability, remove TODO. Build + test after each session. Commit per
      session: `W6-cards: implement previously blocked abilities (<N> cards)`
      Repeat F-4 until "now expressible" list is exhausted.
- [ ] **F-5**: Review all cards fixed/re-authored in F-1 through F-4 using
      `card-batch-reviewer` (batches of 5, 4 parallel). Any new HIGH/MEDIUM → loop
      back to F-1/F-2 pattern.
- [ ] **F-6**: Verify: `cargo build --workspace && cargo test --all && cargo clippy -- -D warnings`
- [ ] **F-7**: Commit: `W6-fix: phase 1 complete — all existing defs clean`

### Phase 2: Author Remaining Cards

Requires: F-7 complete.

Work through groups in the order below. For each group, follow the per-group workflow
(Section "Per-Group Workflow" below). Check off each group when committed.

**Tier 1: Simple, zero blockers**

- [ ] **A-01**: body-only (3 sessions, 55 cards)
- [ ] **A-02**: mana-creature (2 sessions, 19 cards)
- [ ] **A-03**: mana-artifact (3 sessions, 33 cards)
- [ ] **A-04**: mana-other (1 session, 5 cards)
- [ ] **A-05**: cost-reduction (2 sessions, 12 cards)
- [ ] **A-06**: scry-surveil (1 session, 7 cards)
- [ ] **A-07**: lifegain (1 session, 5 cards)
- [ ] **A-08**: lifedrain (1 session, 6 cards)
- [ ] **A-09**: protection (1 session, 1 card)
- [ ] **A-10**: aura (1 session, 6 cards)

**Tier 2: Standard patterns**

- [ ] **A-11**: removal-destroy (4 sessions, 48 cards)
- [ ] **A-12**: removal-exile (1 session, 11 cards)
- [ ] **A-13**: removal-damage-target (2 sessions, 21 cards)
- [ ] **A-14**: removal-damage-each (2 sessions, 17 cards)
- [ ] **A-15**: removal-bounce (1 session, 7 cards)
- [ ] **A-16**: removal-minus (1 session, 4 cards)
- [ ] **A-17**: counter (2 sessions, 15 cards)
- [ ] **A-18**: draw (14 sessions, 161 cards)
- [ ] **A-19**: token-create (13 sessions, 146 cards)
- [ ] **A-20**: pump-buff (3 sessions, 26 cards)
- [ ] **A-21**: counters-plus (5 sessions, 42 cards)
- [ ] **A-22**: equipment (2 sessions, 9 cards)
- [ ] **A-23**: death-trigger (3 sessions, 24 cards)
- [ ] **A-24**: attack-trigger (4 sessions, 33 cards)
- [ ] **A-25**: activated-tap (4 sessions, 25 cards)
- [ ] **A-26**: activated-sacrifice (3 sessions, 19 cards)
- [ ] **A-27**: sacrifice-outlet (1 session, 6 cards)
- [ ] **A-28**: discard-effect (1 session, 7 cards)

**Tier 3: Complex patterns**

- [ ] **A-29**: cant-restriction (3 sessions, 24 cards)
- [ ] **A-30**: untap-phase (2 sessions, 12 cards)
- [ ] **A-31**: land-etb-tapped (8 sessions, 122 cards)
- [ ] **A-32**: land-fetch (4 sessions, 27 cards)
- [ ] **A-33**: graveyard-recursion (2 sessions, 9 cards)
- [ ] **A-34**: opponent-punish (2 sessions, 9 cards)
- [ ] **A-35**: etb-trigger (1 session, 5 cards)
- [ ] **A-36**: static-enchantment (1 session, 6 cards)
- [ ] **A-37**: combat-keyword (11 sessions, 163 cards)
- [ ] **A-38**: modal-choice (13 sessions, 100 cards)
- [ ] **A-39**: tutor (2 sessions, 11 cards)
- [ ] **A-40**: x-spell (1 session, 1 card)
- [ ] **A-41**: exile-play (1 session, 1 card)
- [ ] **A-42**: other (16 sessions, 127 cards)

### Phase 3: Audit and Certification

Requires: All A-* items complete (or all ready sessions exhausted).

- [ ] **X-1**: Full re-scan — every card def file checked for TODOs, empty abilities,
      known-issue patterns, oracle text correctness. Output:
      `memory/card-authoring/audit-report.md`
- [ ] **X-2**: Fix all remaining gaps found in X-1. Implement micro-primitives if they
      unblock 5+ cards. Replace unfixable TODOs with `// KNOWN_GAP: <description>`.
- [ ] **X-3**: Re-scan to verify X-2 resolved everything.
- [ ] **X-4**: Final build + test: `cargo build --workspace && cargo test --all && cargo clippy -- -D warnings`
- [ ] **X-5**: Update documentation: CLAUDE.md, primitive-card-plan.md, workstream-coordination.md
- [ ] **X-6**: Write certification: `memory/card-authoring/audit-certification.md`
- [ ] **X-7**: Commit: `W6-audit: card authoring complete — <N> cards, zero TODOs`

---

## Situation Assessment

### Card Definition Inventory (as of 2026-03-21)

| Metric | Count |
|--------|------:|
| Total card def files | 740 |
| Cards in authoring plan (`_authoring_plan.json`) | 1,636 |
| Plan cards with existing def file | 476 (29%) |
| ...with TODOs (need rework) | 234 |
| ...clean (no TODO) | 242 |
| Plan cards with no def yet | 1,160 (71%) |
| Defs not in authoring plan (pre-existing) | 279 |
| **Total cards needing work** (new + TODO rework) | **~1,394** |

### Authoring Plan Sessions (`_authoring_plan.json`)

| Status | Sessions | Cards |
|--------|---------|------:|
| ready | 149 | 1,387 |
| blocked | 28 | 133 |
| complete | 7 | 92 |
| deferred | 6 | 24 |
| **Total** | **190** | **1,636** |

### Prior Authoring Waves — Incomplete

Three waves were started but **none reached the commit stage**:

| Wave | Group | Cards | Author | Review | Fix Applied | Committed |
|------|-------|------:|:------:|:------:|:-----------:|:---------:|
| 001: Land ETB-tapped | land-etb-tapped | 82 | DONE | DONE | NO | NO |
| 002: Combat keyword | combat-keyword | 187 | DONE | DONE | NO | NO |
| 003: Mana land | mana-land | 92 | DONE | DONE | NO | NO |

Review findings sitting unfixed:

| Wave | HIGH | MEDIUM | LOW | Total |
|------|-----:|-------:|----:|------:|
| Phase 1 (PB reviews) | 9 | 27 | 5 | 41 |
| Wave 002 | 13 | 17 | 35 | 65 |
| Wave 003 | 8 | 9 | 3 | 20 |
| **Total** | **30** | **53** | **43** | **126** |

### Common HIGH Finding Patterns (from reviews)

| Pattern | Count | Example |
|---------|------:|---------|
| W5 policy violation (partial impl produces wrong game state) | ~10 | Pain lands give free colored mana without self-damage |
| Missing expressible keyword (Convoke, CantBeBlocked, Enchant, etc.) | ~6 | TODO claims gap exists when DSL already supports it |
| Missing supertype (Legendary, Basic, Snow, World) | ~4 | Legendary lands without `SuperType::Legendary` |
| Wrong P/T for `*/*` creatures (`Some(0)` instead of `None`) | ~2 | Dies to SBA before CDA applies |
| Missing dual ability def (Ninjutsu/Mutate keyword + cost) | ~2 | Keyword marker without corresponding cost entry |
| Wrong mana cost (hybrid approximation errors, missing mana) | ~3 | MDFC with {0} cost instead of actual cost |
| Wrong type line (MDFC front including back-face types) | ~2 | Sorcery+Land on MDFC front face |

### DSL Gap Audit Status

The audit at `memory/card-authoring/dsl-gap-audit.md` is **stale** (2026-03-13, pre-PB-10
through PB-22). Primitives added since then include:

| Primitive Batch | DSL Gaps Closed |
|----------------|----------------|
| PB-10 | Return from zone (graveyard targeting) |
| PB-11 | Mana restrictions |
| PB-12 | Complex replacements, trigger doublers, player filters |
| PB-13 | Specialized effects |
| PB-14 | Planeswalker support (loyalty abilities) |
| PB-15 | Saga/Class |
| PB-16 | Meld |
| PB-17 | Library search filters |
| PB-18 | Stax / restrictions |
| PB-19 | Board wipes |
| PB-20 | Extra combat |
| PB-21 | Fight / Bite |
| PB-22 S1 | Activation conditions, mana cost filter, sorcery-speed |
| PB-22 S2 | Coin flip, d20 rolls |
| PB-22 S3 | Reveal-route, flicker |
| PB-22 S4 | Tapped-attacking tokens, equipment auto-attach |
| PB-22 S5 | Copy/clone primitives |
| PB-22 S6 | Emblem creation |

Many of the 418 TODO cards and 28 blocked sessions may now be fully expressible.

---

## Phase 0: Triage (do first, before any authoring)

**Goal**: Establish ground truth. Know exactly what the DSL can and cannot express today.
Know exactly which cards need fixes, which need authoring, and which are truly blocked.

### Step 0.1: Refresh the DSL Gap Audit

Re-scan all 740 card def files. For each file with a TODO comment, determine whether
the TODO is still valid given PB-0 through PB-22.

**Procedure**:
1. Grep all `TODO` lines from `crates/engine/src/cards/defs/*.rs`
2. For each TODO, classify:
   - **Now expressible**: The DSL primitive exists. Card should be re-authored.
   - **Still blocked**: No DSL support. Document what's missing.
   - **Stale/wrong**: TODO claims something is missing that already existed (see HIGH findings above).
3. Cross-reference against `helpers.rs` exports and the full `Effect`, `AbilityDefinition`,
   `TriggerCondition`, `TargetRequirement` enums to verify classifications
4. Produce updated gap audit at `memory/card-authoring/dsl-gap-audit-v2.md`

**Output**: A table mapping each gap bucket to:
- Number of cards affected
- Whether the DSL now covers it (yes/no/partial)
- If partial, what's still missing
- Estimated effort to implement the remaining gap (if any)

### Step 0.2: Re-evaluate Blocked Sessions

The 28 blocked sessions in `_authoring_plan.json` have no recorded block reason.

**Procedure**:
1. For each blocked session, read the card list and oracle text
2. For each card, check whether its abilities are now expressible in the DSL
3. Classify sessions as:
   - **Unblocked**: All cards expressible. Move to ready.
   - **Partially blocked**: Some cards expressible, some not. Split or note.
   - **Still blocked**: Core mechanic missing. Document what's needed.
4. Update `_authoring_plan.json` session statuses

**Output**: Updated `_authoring_plan.json` with corrected statuses. A summary table
of what remains truly blocked and why.

### Step 0.3: Re-evaluate Deferred Sessions

The 6 deferred sessions (24 cards) need the same treatment as blocked sessions.

### Step 0.4: Tally and Classify Existing Review Findings

Read all review finding files in `memory/card-authoring/review-*.md`.

**Procedure**:
1. Parse all findings (Phase 1 batches 01-20, Wave 002 batches 01-38, Wave 003 batches 01-15)
2. For each HIGH or MEDIUM finding, classify:
   - **Still valid**: Needs fixing.
   - **Superseded by PB work**: A primitive batch already fixed this card.
   - **Now expressible**: The TODO that caused the finding is now implementable.
   - **Already fixed**: Check if the card def file was updated since the review.
3. Produce a consolidated fix list with per-card actions

**Output**: `memory/card-authoring/consolidated-fix-list.md` — one entry per card that
needs work, with the specific action needed and the review batch that found it.

### Step 0.5: Inventory Pre-existing Defs Not in Plan

279 card defs exist but are not in the authoring plan. These are pre-existing hand-authored
and Phase 1 template cards.

**Procedure**:
1. List all 279 files
2. Check each for TODOs
3. For those with TODOs, classify per Step 0.1 (now expressible vs still blocked)
4. Any with wrong game state get added to the fix list

**Output**: Append to the consolidated fix list. Update total card counts.

### Triage Deliverables

At the end of Phase 0, we have:
- [ ] `memory/card-authoring/dsl-gap-audit-v2.md` — refreshed gap audit
- [ ] `_authoring_plan.json` — updated session statuses (blocked → ready where applicable)
- [ ] `memory/card-authoring/consolidated-fix-list.md` — every card needing fixes, with actions
- [ ] `memory/card-authoring/triage-summary.md` — executive summary with updated counts:
  - Cards that can be fully authored today
  - Cards with fixable TODOs (DSL now covers them)
  - Cards with valid TODOs (DSL still lacks something)
  - Truly blocked cards (and what blocks them)

---

## Phase 1: Fix Existing Card Definitions

**Goal**: Every card def that exists today is correct — no wrong game state, no stale TODOs,
no missing expressible abilities.

### Step 1.1: Apply HIGH Fixes from Reviews

Work through the consolidated fix list, HIGH severity first. These are cards that
produce incorrect game state.

**Common HIGH fix patterns** (from review analysis):

| Fix Type | Action | Cards |
|----------|--------|------:|
| Pain land free mana | Remove colored mana ability, leave only {C} + TODO | ~8 |
| Missing activation restriction | Remove unrestricted ability, leave TODO | ~4 |
| Expressible keyword marked as TODO | Add the keyword (Convoke, CantBeBlocked, Enchant, etc.) | ~6 |
| Missing supertype | Add Legendary/Basic/Snow/World | ~4 |
| Wrong P/T for `*/*` | Change `Some(0)` to `None` | ~2 |
| Missing dual def (keyword + cost) | Add Ninjutsu/Mutate cost AbilityDef | ~2 |
| Wrong mana cost | Fix ManaCost fields | ~3 |
| Wrong MDFC types | Remove back-face types from front face | ~2 |

**Procedure per fix session**:
1. Take 10-15 cards from the consolidated fix list (HIGH only)
2. For each card:
   a. Read the review finding
   b. Read the current card def
   c. Look up oracle text via MCP if needed
   d. Apply the fix
3. `cargo build --lib -p mtg-engine` after all fixes
4. `cargo test --all`
5. Mark fixed in consolidated fix list
6. Commit: `W6-cards: fix HIGH findings — <brief description> (<N> cards)`

**Estimated sessions**: 3-4 (30 HIGH findings, ~10 per session)

### Step 1.2: Apply MEDIUM Fixes from Reviews

Same procedure as HIGH, working through MEDIUM findings.

**Common MEDIUM fix patterns**:
- Wrong trigger filter (overbroad `WheneverCreatureDies` → should be `vec![]` per W5)
- Missing target controller filter (`TargetPermanent` → `TargetPermanentWithFilter`)
- Placeholder effects (`GainLife(0)`) that should be `vec![]`
- Wrong `mana_pool` argument order
- Subtype ordering errors

**Estimated sessions**: 5-6 (53 MEDIUM findings, ~10 per session)

### Step 1.3: Re-author Cards with Now-Expressible TODOs

From the triage, some cards have TODOs that are now expressible. These need their
abilities filled in, not just metadata fixes.

**Procedure per session**:
1. Take 8-12 cards from the "now expressible" list
2. For each card:
   a. Look up oracle text via MCP
   b. Read a reference card def using the same DSL pattern
   c. Implement the full ability using the new primitives
   d. Remove the TODO comment
3. `cargo build --lib -p mtg-engine`
4. `cargo test --all`
5. Commit: `W6-cards: implement previously blocked abilities (<N> cards)`

**Estimated sessions**: Depends on triage results. Could be 10-30+ sessions if many
of the 234 TODO cards are now expressible.

### Step 1.4: Review Fixed Cards

After all fixes are applied, run the `card-batch-reviewer` agent on the fixed cards
to verify correctness.

**Procedure**:
1. Batch fixed cards into groups of 5
2. Run `card-batch-reviewer` on each batch
3. Any new HIGH/MEDIUM findings go back to the fix queue
4. Iterate until clean

### Phase 1 Deliverables

- [ ] All 30 HIGH findings fixed and verified
- [ ] All 53 MEDIUM findings fixed and verified
- [ ] All LOW findings addressed where trivial
- [ ] All now-expressible TODOs implemented
- [ ] All fixed cards re-reviewed and clean
- [ ] `memory/card-authoring/consolidated-fix-list.md` fully checked off

---

## Phase 2: Author Remaining Cards

**Goal**: Write card definitions for every card in the authoring plan that doesn't have one.

### Authoring Plan Groups

The `_authoring_plan.json` organizes 1,636 cards into 43 groups across 190 sessions.
After triage updates, the ready sessions should increase from 149 to ~170+.

| Group | Ready Sessions | Ready Cards | Blocked Sessions | Blocked Cards |
|-------|-----------|-------------|-------------|---------------|
| combat-keyword | 11 | 163 | 2 | 23 |
| draw | 14 | 161 | 1 | 6 |
| token-create | 13 | 146 | 1 | 9 |
| other | 16 | 127 | 1 | 4 |
| land-etb-tapped | 8 | 122 | 1 | 16 |
| modal-choice | 13 | 100 | 1 | 5 |
| body-only | 3 | 55 | 0 | 0 |
| removal-destroy | 4 | 48 | 1 | 8 |
| counters-plus | 5 | 42 | 1 | 4 |
| mana-artifact | 3 | 33 | 1 | 1 |
| attack-trigger | 4 | 33 | 1 | 1 |
| land-fetch | 4 | 27 | 3 | 18 |
| pump-buff | 3 | 26 | 1 | 1 |
| activated-tap | 4 | 25 | 1 | 2 |
| death-trigger | 3 | 24 | 1 | 10 |
| cant-restriction | 3 | 24 | 1 | 1 |
| removal-damage-target | 2 | 21 | 1 | 2 |
| mana-creature | 2 | 19 | 0 | 0 |
| activated-sacrifice | 3 | 19 | 0 | 0 |
| removal-damage-each | 2 | 17 | 0 | 0 |
| counter | 2 | 15 | 1 | 1 |
| untap-phase | 2 | 12 | 1 | 1 |
| cost-reduction | 2 | 12 | 0 | 0 |
| removal-exile | 1 | 11 | 1 | 1 |
| tutor | 2 | 11 | 0 | 0 |
| graveyard-recursion | 2 | 9 | 1 | 8 |
| equipment | 2 | 9 | 1 | 2 |
| opponent-punish | 2 | 9 | 1 | 3 |
| removal-bounce | 1 | 7 | 1 | 3 |
| discard-effect | 1 | 7 | 0 | 0 |
| scry-surveil | 1 | 7 | 0 | 0 |
| aura | 1 | 6 | 0 | 0 |
| sacrifice-outlet | 1 | 6 | 0 | 0 |
| static-enchantment | 1 | 6 | 1 | 2 |
| lifedrain | 1 | 6 | 0 | 0 |
| mana-other | 1 | 5 | 0 | 0 |
| etb-trigger | 1 | 5 | 1 | 1 |
| lifegain | 1 | 5 | 0 | 0 |
| removal-minus | 1 | 4 | 0 | 0 |
| protection | 1 | 1 | 0 | 0 |
| x-spell | 1 | 1 | 0 | 0 |
| exile-play | 1 | 1 | 0 | 0 |

Note: `mana-land` (92 cards, 7 sessions) shows as "complete" in the plan — these were
authored in Wave 003 but have unfixed review findings (handled in Phase 1).

### Authoring Order

Author groups in this order, prioritized by:
1. **Simplest first** (validates DSL, builds velocity)
2. **Highest card count** (maximum progress per session)
3. **No blockers** (groups with 0 blocked sessions first)

**Tier 1: Simple, high-volume, zero blockers** (body-only, mana-creature, cost-reduction, etc.)
**Tier 2: Standard patterns** (removal, draw, token-create, counters, pump)
**Tier 3: Complex patterns** (modal-choice, land-fetch, graveyard-recursion, other)

Detailed ordering:

| Order | Group | Sessions | Cards | Notes |
|------:|-------|---------|------:|-------|
| 1 | body-only | 3 | 55 | Vanilla/keyword-only — simplest possible |
| 2 | mana-creature | 2 | 19 | Tap-for-mana creatures |
| 3 | mana-artifact | 3 | 33 | Sol Ring variants |
| 4 | mana-other | 1 | 5 | Misc mana producers |
| 5 | cost-reduction | 2 | 12 | Static cost modifiers |
| 6 | scry-surveil | 1 | 7 | Simple ETB effects |
| 7 | lifegain | 1 | 5 | Simple ETB/trigger |
| 8 | lifedrain | 1 | 6 | Drain patterns |
| 9 | protection | 1 | 1 | Single card |
| 10 | aura | 1 | 6 | Enchant + grant |
| 11 | removal-destroy | 4 | 48 | Destroy target |
| 12 | removal-exile | 1 | 11 | Exile target |
| 13 | removal-damage-target | 2 | 21 | Bolt variants |
| 14 | removal-damage-each | 2 | 17 | Board damage |
| 15 | removal-bounce | 1 | 7 | Return to hand |
| 16 | removal-minus | 1 | 4 | -N/-N effects |
| 17 | counter | 2 | 15 | Counterspells |
| 18 | draw | 14 | 161 | Card advantage |
| 19 | token-create | 13 | 146 | Token generators |
| 20 | pump-buff | 3 | 26 | P/T modification |
| 21 | counters-plus | 5 | 42 | +1/+1 counter manipulation |
| 22 | equipment | 2 | 9 | Equip + grant |
| 23 | death-trigger | 3 | 24 | Aristocrat patterns |
| 24 | attack-trigger | 4 | 33 | Combat triggers |
| 25 | activated-tap | 4 | 25 | {T}: effect |
| 26 | activated-sacrifice | 3 | 19 | Sac outlets |
| 27 | sacrifice-outlet | 1 | 6 | Dedicated sac outlets |
| 28 | discard-effect | 1 | 7 | Discard spells |
| 29 | cant-restriction | 3 | 24 | Restriction statics |
| 30 | untap-phase | 2 | 12 | Untap triggers |
| 31 | land-etb-tapped | 8 | 122 | Conditional ETB |
| 32 | land-fetch | 4 | 27 | Fetch + search |
| 33 | graveyard-recursion | 2 | 9 | Return from GY |
| 34 | opponent-punish | 2 | 9 | Punisher effects |
| 35 | etb-trigger | 1 | 5 | ETB patterns |
| 36 | static-enchantment | 1 | 6 | Static enchantments |
| 37 | combat-keyword | 11 | 163 | Complex keyword creatures |
| 38 | modal-choice | 13 | 100 | Modal spells |
| 39 | tutor | 2 | 11 | Search library |
| 40 | x-spell | 1 | 1 | X-cost spells |
| 41 | exile-play | 1 | 1 | Exile-then-play |
| 42 | other | 16 | 127 | Uncategorized |

### Per-Group Workflow

Each group follows this cycle:

```
1. Pre-check: Are all sessions in this group ready? If blocked, skip or split.
2. Author: Run bulk-card-author agent on each session (2-3 agents in parallel).
3. Build: cargo build --lib -p mtg-engine (catch compile errors).
4. Review: Run card-batch-reviewer agent on authored cards (batches of 5, 4 parallel).
5. Fix: Apply all HIGH and MEDIUM findings.
6. Re-review: Re-review fixed cards if any HIGH findings existed.
7. Build + Test: cargo build --workspace && cargo test --all && cargo clippy -- -D warnings
8. Commit: W6-cards: author <group> (<N> cards)
```

### Parallel Execution Strategy

- Run 2-3 `bulk-card-author` agents simultaneously per group
- Run 4-5 `card-batch-reviewer` agents simultaneously
- Groups with no dependency can overlap: while group N is in review, group N+1 can
  start authoring
- **Do not start the next group's fix phase until the current group is committed.**
  Overlapping fixes risk merge conflicts in the same card files.

### Handling Blocked Sessions

After triage (Phase 0), some sessions will remain blocked. For each:

1. **If 1-2 cards are blocked in an otherwise ready session**: Author the ready cards,
   leave blocked cards with `abilities: vec![]` and a precise TODO documenting the
   missing DSL primitive. Mark the card in a tracking list for Phase 3 follow-up.

2. **If the entire session is blocked**: Skip it. Add to the Phase 3 backlog with
   the specific blocker documented.

3. **If a new micro-primitive would unblock 5+ cards**: Consider implementing it
   inline (small engine change + card fixes), same as PB-22 sessions did. Document
   the primitive addition and test it.

### Phase 2 Deliverables

- [ ] All ready sessions authored (149+ sessions, 1,387+ cards)
- [ ] All authored cards reviewed and clean (0 unfixed HIGH/MEDIUM)
- [ ] All groups committed
- [ ] Tracking list of remaining blocked cards with specific blockers

---

## Phase 3: Audit and Certification

**Goal**: Zero TODOs. Zero wrong game state. Every card in the 1,743-card universe
has a complete, correct definition.

### Step 3.1: Full Re-scan

Scan every card def file for:
- `TODO` comments
- `abilities: vec![]` (should only be vanilla creatures)
- Patterns from the known-issue list (KI-1 through KI-10)
- `etb_tapped` correctness (compare against oracle text)
- Mana cost correctness
- Type line correctness

**Tool**: Automated script that checks each file against its oracle text via MCP.
More rigorous than the batch reviewer — checks every field, not just a sample.

### Step 3.2: Fix Remaining Gaps

For cards that still have TODOs:
1. If the DSL covers it: implement the ability, remove the TODO.
2. If the DSL doesn't cover it: evaluate whether a micro-primitive is worth adding.
   If the gap affects only 1-2 cards and is genuinely complex, document it as a
   known limitation with a `// KNOWN_GAP: <description>` comment (replacing TODO).
3. Target: zero `TODO` comments in all card def files.

### Step 3.3: Cross-validation

Run `cargo test --all` and `cargo clippy -- -D warnings` one final time.

For each card, verify:
- Card can be included in a deck (deck builder accepts it)
- Card can be cast (if applicable)
- ETB triggers fire (if applicable)
- Keywords function (if applicable)

This is done via the existing test suite + targeted spot-checks on complex cards.

### Step 3.4: Update Documentation

- Update `CLAUDE.md` current state with final card counts
- Update `docs/primitive-card-plan.md` Phase 2/3 completion
- Update `docs/workstream-coordination.md` Phase 5 checkboxes
- Archive `memory/card-authoring/consolidated-fix-list.md` as complete
- Write `memory/card-authoring/audit-certification.md` with final stats

### Phase 3 Deliverables

- [ ] Zero `TODO` comments in card defs (or all remaining are `KNOWN_GAP` with justification)
- [ ] Zero `abilities: vec![]` on non-vanilla cards
- [ ] All tests passing
- [ ] All documentation updated
- [ ] Audit certification written

---

## Infrastructure: Skills and Agents

### Existing Agents (confirmed working)

| Agent | Model | Purpose | Status |
|-------|-------|---------|--------|
| `bulk-card-author` | Sonnet | Write 8-20 card defs per session from `_authoring_plan.json` | Working, needs prompt updates |
| `card-batch-reviewer` | Opus | Review 5 cards against oracle text | Working |
| `card-definition-author` | Sonnet | Author a single card | Working |

### Agent Updates Needed

**`bulk-card-author`**:
- Update DSL Quick Reference to include all PB-0 through PB-22 primitives
- Add references for new patterns: coin flip, flicker, reveal-route, copy/clone,
  emblems, activation conditions, sorcery-speed timing
- Update known-issue patterns list with findings from waves 001-003
- Add rule: when re-authoring a card with existing `abilities: vec![]`, always
  check if the ability is now expressible before leaving as TODO
- Increase MCP budget from 20 to 30 (larger sessions with complex cards)

**`card-batch-reviewer`**:
- Update known-issue patterns with all HIGH patterns from waves 001-003
- Add check: "Does the TODO claim a DSL gap that actually exists?" (catches KI-pattern
  where TODO is wrong about what the DSL supports)
- Add check: verify `etb_tapped` matches oracle text for all lands

### New Agent: `card-fix-applicator`

**Purpose**: Read review findings from `memory/card-authoring/review-*.md` files,
apply corrections to card def files, verify build.

**Model**: Sonnet
**Tools**: Read, Edit, Write, Grep, Glob, Bash, mcp__mtg-rules__lookup_card

**Workflow**:
1. Read the consolidated fix list or a specific review batch file
2. For each finding:
   a. Read the card def file
   b. Look up oracle text if needed
   c. Apply the fix (Edit tool for targeted changes)
   d. Mark as fixed
3. `cargo build --lib -p mtg-engine`
4. Report fixed/failed

This agent fills the critical gap — reviews produce findings but nothing applies them.

### New Skill: `/author-wave`

**Purpose**: Orchestrate the full author-review-fix-commit cycle for one group.

**Workflow**:
1. Read `_authoring_plan.json` to find ready sessions for the specified group
2. Check which cards already have defs (skip those unless skeleton)
3. Launch `bulk-card-author` agents (2-3 parallel) for the group's sessions
4. Wait for all author agents to complete
5. `cargo build --lib -p mtg-engine` to catch compile errors
6. Launch `card-batch-reviewer` agents (4-5 parallel) on authored cards
7. Wait for all reviewers to complete
8. Parse findings: if HIGH/MEDIUM exist, launch `card-fix-applicator`
9. If fixes were applied, re-review the fixed cards
10. Final build + test
11. Report summary: cards authored, findings, fixes applied

**State file**: `memory/card-authoring/wave-progress.md` — tracks which groups
are complete, in-progress, or pending.

### New Skill: `/triage-cards`

**Purpose**: Execute Phase 0 triage steps.

**Workflow**:
1. Scan all card defs for TODOs, classify against current DSL capabilities
2. Re-evaluate blocked/deferred sessions in `_authoring_plan.json`
3. Parse all existing review findings
4. Cross-reference to produce consolidated fix list
5. Generate triage summary with updated counts

### New Skill: `/audit-cards`

**Purpose**: Execute Phase 3 audit.

**Workflow**:
1. Scan every card def file for TODOs, empty abilities, known-issue patterns
2. For each card with issues, look up oracle text and verify correctness
3. Produce audit report with per-card status
4. Generate certification document

---

## Tracking and Coordination

### Files

| File | Purpose | Phase |
|------|---------|-------|
| `_authoring_plan.json` | Card list, groups, sessions, statuses | All |
| `memory/card-authoring/dsl-gap-audit-v2.md` | Refreshed DSL gap analysis | 0 |
| `memory/card-authoring/consolidated-fix-list.md` | Every card needing fixes | 0, 1 |
| `memory/card-authoring/triage-summary.md` | Executive summary from triage | 0 |
| `memory/card-authoring/wave-progress.md` | Per-group author/review/fix/commit status | 2 |
| `memory/card-authoring/audit-certification.md` | Final audit results | 3 |
| `memory/card-authoring/review-*.md` | Per-batch review findings (existing) | 0, 1 |

### Commit Convention

| Phase | Prefix | Example |
|-------|--------|---------|
| Triage | `W6-triage:` | `W6-triage: refresh DSL gap audit, reclassify 28 blocked sessions` |
| Fix | `W6-fix:` | `W6-fix: apply 10 HIGH findings — pain lands, supertypes` |
| Author | `W6-cards:` | `W6-cards: author body-only group (55 cards)` |
| Audit | `W6-audit:` | `W6-audit: implement remaining TODOs, certify zero gaps` |

### Workstream

All card authoring work is **W6: Primitive + Card Authoring**. Claim with
`/start-work W6` before starting. The PB-22 subunit claim will be released when
PB-22 finishes; subsequent work claims `W6-cards` or `W6-triage` or `W6-audit`.

### Estimated Total Effort

| Phase | Sessions | Cards Touched |
|-------|---------|--------------|
| Phase 0: Triage | 2-3 | 0 (research only) |
| Phase 1: Fix existing | 15-25 | ~360 (fixes + re-authors) |
| Phase 2: Author remaining | ~149 | ~1,160 (new defs) |
| Phase 3: Audit | 3-5 | varies (fixes only) |
| **Total** | **~170-182** | **~1,520** |

With parallel execution (2-3 author agents, 4-5 reviewer agents), calendar time
is significantly less than session count. Author sessions take ~15-25 minutes each.
Review sessions take ~10-15 minutes each. Fix sessions take ~20-30 minutes each.

---

## Appendix A: Authoring Plan JSON Schema

```json
{
  "generated": "ISO date string",
  "summary": {
    "total_cards": 1636,
    "already_authored": 107,
    "ready": 1471,
    "blocked": 141,
    "deferred": 24,
    "ready_sessions": 155,
    "blocked_sessions": 29,
    "groups": 43
  },
  "sessions": [
    {
      "session_id": 1,
      "group_id": "land-etb-tapped",
      "group_label": "Lands -- ETB Tapped",
      "status": "ready|blocked|complete|deferred",
      "card_count": 16,
      "cards": [
        {
          "name": "Woodland Cemetery",
          "types": ["Land"],
          "keywords": [],
          "mana_cost": "",
          "deck_count": 3,
          "edhrec_inclusion": 38221,
          "priority_score": 388.2,
          "source": "both",
          "oracle_text": "This land enters tapped unless..."
        }
      ]
    }
  ]
}
```

## Appendix B: Previously Authored Wave Status (Pre-Phase-0)

### Wave 001: Land ETB-Tapped (82 cards)

- **Author**: 9 sessions, all complete (2026-03-12)
- **Review**: 17 batches, all complete
- **Findings**: 2 HIGH, 46 MEDIUM, 22 LOW
- **Fix**: NOT STARTED
- **Review files**: `memory/card-authoring/review-phase1-batch-{01..20}.md` (some overlap)
- **Wave file**: `memory/card-authoring/wave-001-land-etb-tapped.md`
- **Issue**: Review dates from before PB-2 (conditional ETB tapped) and PB-3 (shockland ETB).
  Many MEDIUM findings about missing ETB-tapped may now be expressible. Re-evaluate in triage.

### Wave 002: Combat Keyword (187 cards)

- **Author**: 14 sessions, all complete (2026-03-12)
- **Review**: 38 batches, all complete
- **Findings**: 13 HIGH, 17 MEDIUM, 35 LOW
- **Fix**: NOT STARTED
- **Review files**: `memory/card-authoring/review-wave-002-batch-{01..38}.md`
- **Wave file**: `memory/card-authoring/wave-002-combat-keyword.md`
- **Issue**: Many HIGH findings are about expressible keywords incorrectly left as TODO.
  These should be quick fixes.

### Wave 003: Mana Land (92 cards)

- **Author**: 7 sessions, all complete (marked in `_authoring_plan.json`)
- **Review**: 15 batches, all complete
- **Findings**: 8 HIGH, 9 MEDIUM, 3 LOW
- **Fix**: NOT STARTED
- **Review files**: `memory/card-authoring/review-wave-003-batch-{01..15}.md`
- **Wave file**: `memory/card-authoring/wave-003-mana-land.md`
- **Issue**: Pain land free-mana violations (W5 policy). Activation restriction violations.
  Most are simple fixes (remove wrong ability, leave TODO or implement with PB-22 primitives).

### Phase 1 PB Reviews (misc cards fixed during primitive batches)

- **Review**: 20 batches
- **Findings**: 9 HIGH, 27 MEDIUM, 5 LOW
- **Fix**: NOT STARTED (some may have been fixed inline during PB work)
- **Review files**: `memory/card-authoring/review-phase1-batch-{01..20}.md`
- **Issue**: Some findings may be stale if PB work already fixed the card. Verify in triage.
