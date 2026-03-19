# Workstream Coordination Plan

> **Purpose**: Coordinate parallel workstreams to avoid dependency conflicts,
> minimize context-switching overhead, and maintain a clear path to playable.
>
> **Load this file** at the start of any session to orient on which workstream to advance.
>
> **Strategic review (2026-03-07)**: `docs/mtg-engine-strategic-review.md` restructures
> the path forward. Key impacts on workstreams:
> - M11 (UI) is now independent of M10 (networking) — can start immediately
> - M10 splits into M10a (basic multiplayer) and M10b (resilience/social)
> - M12 (card pipeline) downscoped — agent-based scaling replaces pipeline crate
> - Transform tree DONE (4 abilities) — only Morph tree (5 abilities) remains blocked
> - Web-vs-Tauri decision pending — affects W2 and M11 architecture
>
> **Last updated**: 2026-03-08

---

## The Four Workstreams

| # | Workstream | Goal | Status |
|---|-----------|------|--------|
| **W1** | Ability completion | Implement ALL abilities (P3 + P4) before M10 | ~75 implementable + 11 blocked; see `docs/ability-batch-plan.md` |
| **W2** | TUI & simulator | Playable interactive Commander games for testing | Phase 1 done; playable but limited |
| **W3** | LOW issues remediation | Clear ~68 open LOW issues from code reviews | Plan written; 0 fixes applied |
| **W4** | M10 networking | Centralized WebSocket game server | Stub crate only |
| **W5** | Card authoring | Scale card definitions from 112 → 1,000+ using pipeline | Phase 9 ready; 1,061 cards in worklist |

---

## Dependency Map

```
W3 (LOW fixes)                W1 (Abilities)
  T1 zero-risk ───────┐         │
  T2 cosmetic ────────┤         │ each ability needs:
  T3 behavioral ──────┤         │   card def (BOTTLENECK)
  T4 architectural ───┘         │   game script
        │                       │   coverage doc update
        │                       │
        ▼                       ▼
   Stable engine ◄──────── New keywords/effects
        │                       │
        │                       │
        ▼                       ▼
   W4 (M10 networking)    W2 (TUI & simulator)
        │                       │
        │                 LegalActionProvider
        │                 needs to know about
        │                 new abilities/keywords
        │                       │
        └───────────────────────┘
              Both consume
              engine API
```

### Actual conflicts (few)

| Conflict | Files touched | Risk | Mitigation |
|----------|--------------|------|------------|
| W1 + W3 both edit `sba.rs` | W1 adds keyword SBA checks; W3 fixes MR-M4-11, MR-M4-13 | Merge conflict | Do W3 T1/T2 SBA fixes first, then new abilities |
| W1 + W3 both edit `casting.rs` | W1 adds alt cost keywords; W3 fixes MR-M9.4-11 | Merge conflict | W3 fix is comment-only — do it anytime |
| W1 + W2: new abilities not in LegalActionProvider | W2 bots can't use abilities W1 adds | Functional gap | Batch: update LegalActionProvider after each ability batch, not each ability |
| W1 card authoring bottleneck | `definitions.rs` grows; agent unreliable | Velocity | See "Card Authoring Strategy" below |
| W4 needs `private_to()` in `events.rs` | W1 also adds new event variants | Low risk | `private_to()` is a method on existing events, not a new variant |

### Non-conflicts (safe to parallelize)

| Pair | Why safe |
|------|----------|
| W2 (TUI) + W3 (LOWs) | TUI is in `tools/tui/`; LOWs are in `crates/engine/` — no file overlap |
| W2 (TUI) + W4 (M10) | TUI is a local client; M10 is a server — different crates, complementary |
| W1 (abilities) + W4 (M10) | M10 depends on engine stability, not specific abilities |
| W3 T1 (tests) + anything | Additive-only; zero regression risk by definition |
| W5 (card authoring) + anything | Only writes new files to `cards/defs/`; no file overlap with any other workstream |

---

## Recommended Execution Order

### Phase 0: Stabilize (1-2 sessions, ~3-4 hours)

**Goal**: Clear the zero-risk backlog before adding anything new.

1. **W3 T1: Write missing tests** (10 new + 4 improvements)
   - Zero risk to existing 1033-test baseline
   - Improves safety net for everything that follows
   - See `docs/mtg-engine-low-issues-remediation.md` Phase 1

2. **W3 T1: Delete dead code** (MR-M1-14, MR-M9.5-08)

3. **W1: Finish Hideaway** (steps 5-7: card def, script, coverage update)
   - Currently in review — don't leave WIP hanging

4. **Commit**: `chore: fill LOW test gaps, remove dead code, close Hideaway`

**Why first**: These are the safest possible changes. They make every subsequent
phase safer by expanding test coverage. Finishing Hideaway clears the WIP pipeline.

### Phase 1: Full Ability Coverage (~23-28 sessions)

**Goal**: Implement ALL keyword abilities and ability patterns (P3 + P4) before M10.
~78 implementable abilities organized into 16 batches + Mutate mini-milestone.
See `docs/ability-batch-plan.md` for the full batch plan with per-ability effort
estimates and dependency map.

**Approach**:
1. Manual batch card authoring (Option C) — write 5-8 card defs at batch start
2. Work through batches in order (Batch 0 first, then 1-15 mostly independent)
3. Update LegalActionProvider after every 3-4 batches
4. Only hard dependency: Batch 11 (Modal Choice) before Entwine/Escalate
5. Phasing included in Batch 8 (`phased_out` field already exists — medium effort)
6. Mutate mini-milestone after all 16 batches (~2-3 sessions, new object model)

**Blocked abilities** (5 total — defer to dedicated subsystem milestone):
- Morph tree (5): face-down casting/battlefield subsystem
- ~~Transform tree (4): DFC second face + day/night cycle~~ — **DONE** (Transform mini-milestone 2026-03-08)

**Why this is the bulk of the work**: ~78 abilities at ~30-90 min each = the largest
workstream by far. But each ability is independent and follows established patterns.
Most P4 abilities reuse infrastructure built for P1-P3 (alt-cost casting, combat
triggers, counter placement, ETB patterns).

### Phase 2: TUI & Simulator Hardening (1-2 sessions)

**Goal**: Make TUI games meaningfully playable.

1. **Targeted ability resolution**: LegalActionProvider currently passes empty target
   vecs for activated abilities → most activated abilities fizzle. Fix this.

2. **Blocker declaration UI**: Engine supports full blocking; TUI stubs it. Wire up
   the existing engine code to TUI input.

3. **Attacker targeting**: Allow per-creature attack target selection (currently
   "attack with all" only).

4. **LegalActionProvider expansion**: Add missing action types (flashback, escape,
   evoke, kicker, etc.) — these abilities exist in engine but bots can't use them.

**Why third**: TUI needs abilities to exist first (Phase 1). TUI hardening validates
that abilities actually work in gameplay, which is the whole point.

### Phase 3: W3 T2 + T3 LOWs (1 session)

**Goal**: Harden defensive checks now that all the ability work is done.

1. **W3 T2**: debug_assert additions, silent-default hardening, error name fixes,
   performance micro-optimizations (see remediation doc Phase 2)

2. **W3 T3 (ManaPool only)**: If M10 is next, encapsulate ManaPool.spend() now —
   the network layer needs a clean mana API

3. **Commit**: `fix: harden defensive checks, encapsulate ManaPool`

**Why fourth**: T2/T3 LOWs touch runtime code. Better to do this after abilities are
done so the test suite is as comprehensive as possible.

### Phase 4: M10 Networking (multi-session milestone)

**Goal**: Centralized WebSocket server for multiplayer Commander.

1. Use `/start-milestone 10` to load the roadmap section and create a session plan

2. **One small engine change**: Add `GameEvent::private_to() -> Option<PlayerId>`

3. **All other work is in `crates/network/`** — no engine changes needed

4. TUI becomes a local test client for the server (Phase 2 work pays off)

**Why last**: M10 is the biggest lift and benefits from maximum engine stability.
All LOWs that affect M10 (ManaPool, determinism) should be resolved first.

---

## Card Authoring Strategy

The `card-definition-author` agent fails ~57% of the time (4/7 in batch 5).
Common failure modes:

1. **Doesn't call Edit** — researches the card, drafts the definition, then stops
   without inserting it
2. **Uses wrong struct syntax** — `TriggeredAbilityDef` (tuple struct) instead of
   `AbilityDefinition::Triggered { trigger_condition, effect, intervening_if }`
3. **Invents DSL variants** that don't exist
4. **Misses qualified paths** — bare `TokenSpec` instead of `super::card_definition::TokenSpec`

### Immediate fixes

**Option A: Manual authoring with template** (recommended short-term)

When the agent fails, add the card manually. Process:
1. `lookup_card` for oracle text
2. Find a similar existing definition in `definitions.rs`
3. Copy-modify the existing definition
4. Time per card: ~5-10 minutes for simple cards, ~15 for complex

**Option B: Improve the agent prompt** (medium-term)

The agent prompt at `.claude/agents/card-definition-author.md` could be improved:
- Add explicit "you MUST call Edit" instruction in Step 4
- Add negative examples of the struct syntax mistake
- Add a "verify your Edit was applied" Step 4b
- Increase `maxTurns` from 12 to 15 (gives it retry room)

**Option C: Batch card authoring sessions** (recommended for velocity)

Instead of one card per agent invocation, do manual batch sessions:
1. List the 5-8 cards needed for the next ability batch
2. Look up all oracle texts in parallel
3. Write all definitions in one Edit session
4. Run `cargo test --all` once at the end

This avoids agent overhead entirely and is faster than fixing the agent for
the ~70-90 cards needed.

### Card deficit analysis

| Need | Count | Notes |
|------|-------|-------|
| Full ability coverage (16 batches, ~75 abilities) | ~70-90 cards | 1-2 showcase cards per ability |
| TUI gameplay richness | ~10-15 cards | More creatures, removal, draw for interesting games (overlap with above) |
| M10 testing (multiplayer scenarios) | ~5-10 cards | Cards that exercise hidden info (overlap with above) |
| **Total new cards needed** | **~70-90 cards** | On top of existing 112; many overlap with gameplay/testing needs |

At ~10 min/card manual authoring, that's ~12-15 hours spread across sessions.
Each batch session writes 5-8 cards at batch start (~50-80 min per batch).

---

## TUI vs Replay Viewer: Complementary, Not Competing

| Dimension | TUI Play Mode | Replay Viewer |
|-----------|--------------|---------------|
| **Purpose** | Play live games (human + bots) | Step through recorded scripts |
| **Input** | Human decisions in real-time | Pre-recorded JSON scripts |
| **Value** | Validates gameplay feel, finds UX bugs | Validates engine correctness, regression testing |
| **Audience** | Developer playing the game | Developer debugging the engine |
| **State** | Mutable, evolving | Immutable, pre-computed |
| **Technology** | Terminal (ratatui) | Web (axum + Svelte 5) |
| **Dependencies** | Engine + Simulator | Engine + Replay Harness |

**Keep both.** They serve different needs:
- Replay viewer catches **engine bugs** (wrong state transitions)
- TUI catches **gameplay bugs** (abilities that are technically correct but feel wrong)
- Replay viewer is **deterministic** (same script = same result)
- TUI is **exploratory** (random bots find unexpected states)

**Future synergy**: TUI could export game logs as replay scripts → replay viewer
could step through games that were played live. This is a natural extension,
not urgent.

---

## Session Dispatch Guide

Use this table to decide what to work on at the start of each session.

| If you have... | Work on | Load files |
|---------------|---------|------------|
| < 1 hour | W3 T1 tests (pick 2-3) | `docs/mtg-engine-low-issues-remediation.md` |
| 1-2 hours | One ability (plan → implement → review → fix) | `/implement-ability` |
| 2-3 hours | Card authoring batch + ability batch | `memory/conventions.md`, `memory/gotchas-infra.md` |
| 3-4 hours | TUI hardening (LegalActionProvider + blocker UI) | `tools/tui/src/play/`, `crates/simulator/src/` |
| Half day | W3 T2+T3 LOWs | `docs/mtg-engine-low-issues-remediation.md` |
| Full day | M10 session | `/start-milestone 10` |
| WIP exists (`ability-wip.md`) | Finish the WIP ability first | `/implement-ability` |

### Progress checkboxes

Track progress across sessions by checking these off:

#### Phase 0: Stabilize
- [x] W3 T1: 10 new tests written
- [x] W3 T1: 4 existing tests improved
- [x] W3 T1: Dead code removed (MR-M1-14, MR-M9.5-08)
- [x] W1: Hideaway steps 5-7 complete (card def, script, coverage)
- [x] Committed

#### Phase 1: Full Ability Coverage (see `docs/ability-batch-plan.md` for per-ability tracking)
- [x] Batch 0: P3 stragglers (Hideaway close, Overload, Bolster, Adapt, Partner With)
- [x] Batch 1: Evasion & simple keywords (Shadow, Horsemanship, Skulk, Devoid, Decayed, Ingest)
- [x] Batch 2: Combat triggers — blocking (Flanking, Bushido, Rampage, Provoke, Afflict, Renown, Training)
- [x] Batch 3: Combat modifiers & Ninjutsu (Melee, Enlist, Poisonous, Toxic, Ninjutsu)
- [x] Batch 4: Alt-cast graveyard (Retrace, Jump-Start, Aftermath, Embalm, Eternalize, Encore)
- [x] Batch 5: Alt-cast hand/exile (Dash, Blitz, Plot, Prototype, Impending)
- [x] Batch 6: Cost modification (Bargain, Emerge, Spectacle, Surge, Casualty, Assist)
- [x] Batch 7: Spell modifiers (Replicate, Gravestorm, Overload, Cleave, Splice, Entwine*, Escalate*)
- [x] Batch 8: Upkeep, time & Phasing (Vanishing, Fading, Echo, Cumulative Upkeep, Recover, Forecast, **Phasing**)
- [x] Batch 9: Counter & growth (Graft, Scavenge, Outlast, Amplify, Bloodthirst, Amass)
- [x] Batch 10: ETB/dies patterns (Devour, Backup, Champion, Totem Armor, Living Metal, Soulbond, Fortify)
- [x] Batch 11: Modal choice + deps (Modal Choice system, Tribute, Fabricate, Fuse, Spree)
- [x] Batch 12: Ability words (Enrage, Alliance, Corrupted, Ravenous, Bloodrush)
- [x] Batch 13: Newer set mechanics (Discover, Suspect, Collect Evidence, Forage, Squad, Offspring, Gift, Saddle)
- [x] Batch 14: Niche & encoding (Cipher, Haunt, Reconfigure, Blood/Treasure/Decayed tokens)
- [x] Batch 15: Commander variants (Friends Forever, Choose a Background, Doctor's Companion)
- [x] Mutate mini-milestone (merged-permanent model, CastWithMutate, zone-change splitting)
- [x] Morph mini-milestone (face-down model, TurnFaceUp command, Manifest/Cloak effects, Disguise ward{2})
- [x] LegalActionProvider updated (4 update points: after batches 3, 6, 10, 14; Morph/TurnFaceUp in Morph session)
- [x] Batch 16: Venture/Dungeon (CR 309, 701.49) — 4 dungeons, dungeon zone infra, venture marker, room triggers, SBA 704.5t, initiative (CR 725) — 4 sessions
- [x] Batch 16: The Ring Tempts You (CR 701.54) — ring bearer designation, 4-level temptation, layer grants, EOC sacrifice, combat triggers — 1 session

#### Phase 2: TUI Hardening
- [ ] Targeted ability resolution working
- [ ] Blocker declaration UI working
- [ ] Attacker targeting per-creature working
- [ ] LegalActionProvider handles flashback/escape/evoke/kicker
- [ ] TUI can play a full 10-turn game without crashes

#### Phase 3: LOW Hardening
- [x] W3 T2: debug_assert additions (MR-M1-16, MR-M1-17, MR-M4-11)
- [x] W3 T2: Silent-default hardening (MR-M2-09, MR-M3-11, MR-M3-12, MR-M9-17)
- [x] W3 T2: Performance micro-optimizations (MR-M4-10, MR-M5-06, MR-M9.4-09, MR-M9.5-11)
- [ ] W3 T3: ManaPool::spend() encapsulated
- [ ] W3-LC S1: Layer correctness audit — classify 69 base-characteristic reads (see `memory/w3-layer-audit.md`)
- [ ] W3-LC S2: Fix HIGH sites (abilities.rs, resolution.rs) + Humility interaction tests
- [ ] W3-LC S3: Fix MEDIUM sites (effects/mod.rs, replacement.rs, sba.rs)
- [ ] W3-LC S4: Fix LOW sites + regression test
- [ ] Committed (T3 + LC pending)

#### Phase 4: M10
- [ ] Session plan created (`/start-milestone 10`)
- [ ] `GameEvent::private_to()` added
- [ ] WebSocket server framework
- [ ] Room/lobby management
- [ ] Command validation + event filtering
- [ ] State history + rewind/pause
- [ ] Reconnection
- [ ] Integration tests (hidden info, 6-player)

#### Phase 5: W6 Primitive + Card Authoring (active — see `docs/primitive-card-plan.md`)
- [x] PB-0 through PB-8 complete (ManaCost with damage, conditional ETB tapped, shockland ETB,
  sacrifice cost, targeted abilities, static grants, count scaling, cost reduction statics)
- [x] PB-9: Hybrid mana & X costs (7 cards)
- [x] **PB-9.5: Architecture cleanup** — trigger flush discipline (engine.rs) + test file
  CardDefinition defaults migration. Do immediately after PB-9. See `docs/primitive-card-plan.md`.
- [ ] PB-10 through PB-13: Return from zone, mana restrictions, complex replacements, specialized
- [ ] PB-14 through PB-16: Planeswalker support, Saga/Class, Meld
- [ ] PB-17 through PB-21: Library search, stax, board wipes, extra combat, fight/bite
- [ ] Phase 2: Author ~1,025 remaining cards (bulk authoring sessions)
- [ ] Phase 3: Final audit — zero TODOs, zero wrong game state

#### Deferred architectural work (post-card-authoring, before M10)
- [ ] `resolution.rs` split into per-mechanic files (7,460 lines — organizational only)
- [ ] `EffectContext` refactor: extract cast-time flags into `CastMetadata` struct
- These do NOT affect card authoring. Safe to defer until all 1,743 cards are complete.

---

## Rules of Engagement

1. **Never leave a WIP ability hanging.** Check `memory/ability-wip.md` at session
   start. If something is in-progress, finish it before starting new work.

2. **Batch card definitions.** Don't invoke the card-definition-author agent one
   card at a time. Either fix the agent or do manual batch sessions.

3. **Test after every phase, not every change.** Run `cargo test --all` at phase
   boundaries, not after every file edit. Exception: W3 T3 behavioral changes —
   test after each individual change.

4. **Don't start M10 until Phases 0-1 are done.** M10 benefits from maximum test
   coverage and engine stability. The 14 T1 tests and ability completion both
   contribute to this.

5. **Update this file** when phases complete. Check off boxes, update status table,
   note any new dependencies discovered.

6. **One workstream per session.** Context-switching between workstreams within a
   single session is the primary source of coordination overhead. Pick one
   workstream, advance it, commit, done.
