# Task list — scutemob-257 / LL-3 (doc-only, change class 0)

Live task list. Mirrored as an `esm task comment` on scutemob-257 and updated at every
milestone (this build exposes no TaskCreate to workers; the comment plus this file IS the
task list — dispatch hygiene 10).

Legend: `[ ]` open · `[x]` done · `[~]` in progress

## Phase 0 — read the sources

- [x] 0.1 Read `.esm/worker.md` + `.esm/brief.md`
- [x] 0.2 Read `memory/conventions.md` "## Change-class acceptance table" (class 0 row)
- [x] 0.3 Read `docs/mtg-engine-landscape-assessment.md` §2, §9, §10
- [x] 0.4 Read reference `~/projects/scutemob-landscape/phase/.claude/skills/add-engine-effect/SKILL.md`
- [x] 0.5 Read `phase/CLAUDE.md` "Parameterize, don't proliferate" (line 19)
- [x] 0.6 Read `phase/crates/manabrew-compat/CLAUDE.md` rules 1–2
- [x] 0.7 Read `phase/scripts/tilt-wait.sh` exit-code contract (1 vs 3 is load-bearing)
- [x] 0.8 Materialise the two skip-worktree provisioned skills so they can be edited

## Phase 1 — registration-point census (AC 7568 input; site list is a FLOOR, hygiene 6)

- [ ] 1.1 Census every exhaustive `match` over `Effect` across `crates/` + `tools/`
- [ ] 1.2 Same for `AbilityDefinition`
- [ ] 1.3 Same for `KeywordAbility`
- [ ] 1.4 Same for `StackObjectKind`
- [ ] 1.5 Census the DANGEROUS non-exhaustive sites (`_ =>` wildcards that silently swallow)
- [ ] 1.6 Verify every path/line the brief names against HEAD (no stale paths)

## Phase 2 — the checklist file (AC 7568)

- [ ] 2.1 Write `memory/checklists/new-effect-variant.md` — one line per point:
      path · what to add · what silently fails if missed
- [ ] 2.2 Cite it from `.claude/skills/implement-primitive/SKILL.md`
- [ ] 2.3 Cite it from `.claude/agents/primitive-impl-runner.md`
- [ ] 2.4 Re-verify every path in the finished file exists at HEAD (mechanical check)

## Phase 3 — conventions (AC 7569)

- [ ] 3.1 `memory/conventions.md`: "Parameterize, don't proliferate" + sibling-cluster smell
      + one-CR-section boundary (example: "## Type Consolidation Patterns", 2026-03-09)
- [ ] 3.2 `memory/conventions.md`: never write *unsupported* without naming the population searched
- [ ] 3.3 `memory/conventions.md`: classify by payload type, never by concept name
- [ ] 3.4 Each rule carries a one-line **Why** + a scutemob example, in the "Pair-or-demote" voice

## Phase 4 — dispatch worker prompt (AC 7570)

- [ ] 4.1 `.claude/skills/dispatch/SKILL.md` step 8 `--prompt`: the exit-3 rule
- [ ] 4.2 Record it in the notes file as dispatch hygiene LL-3 (coordinator copies to auto-memory)

## Phase 5 — CLAUDE.md (AC 7571)

- [ ] 5.1 One bold line under "## Critical Gotchas": a `completeness:` that is a guess
- [ ] 5.2 Verify ≤ 250 lines, four ESM-guarded headings intact, three Current State keys intact

## Phase 6 — assessment doc (AC 7572)

- [ ] 6.1 §9 table: `scutemob-257` in the Recommend cell of every adopted row

## Phase 7 — acceptance ritual (change class 0 = docs only)

- [ ] 7.1 Prove zero Rust/source files touched (`git diff --name-only` against merge base)
- [ ] 7.2 `esm task satisfy` each criterion the moment it is met (never batched)
- [ ] 7.3 `/review` — HIGH/MEDIUM fixed in-cycle, LOW logged to the notes file
- [ ] 7.4 `memory/primitives/ll-3-execution-notes.md`
- [ ] 7.5 `CHANGELOG.md` entry (≤ 10 lines, newest first)
- [ ] 7.6 Delete scratch, commit by explicit path, signal-ready, end session
