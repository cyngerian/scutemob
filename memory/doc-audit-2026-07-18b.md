# Documentation Audit #2 (fresh) — 2026-07-18 evening

Independent full re-audit, deliberately unanchored from the DOC-1..8 remediation: three
parallel lanes (docs/+docs.yaml; memory/+state files; cross-reference/skills/agents), each
instructed to treat prior audit artifacts as corpus-to-be-audited, never authority. Ground
truth pulled from machine sources at HEAD `badbe129`: PROTOCOL 18, HASH 55, 1,798 defs,
17 agents, 27 skills, PB-OS1..OS3 collected, OS4 next.

## Verdict in one paragraph

The DOC-1..8 remediation held: all banners accurate, docs.yaml valid (all 21 entries, all
triggers resolve), the reference graph structurally sound (zero dead paths in CLAUDE.md
routing, engine-invariants citations, or MEMORY.md links), archives clean and README'd,
§3 compliant. What the fresh audit found instead is a **new class**: the "what's next"
coordination state went stale — partly *because* the remediation interlude interrupted the
normal collect/eot bookkeeping chain — and **skill wiring to retired docs**, a surface audit
#1 never inspected (it checked skills existed; it did not read their contents).

## Findings

### Theme 1 — Stale "what's next" state (HIGH, blocks PB-OS4 dispatch)

- **N1** `CLAUDE.md:17` (Active Milestone bullet) says "PB-OS1 SHIPPED … next PB-OS2 —
  paused for DOC-1..8" while `CLAUDE.md:53` correctly says OS1..OS3 complete, DOC
  remediation complete, next OS4. The leading Current State line is two generations stale
  and self-contradictory with the same section.
- **N2** `memory/workstream-state.md` is a full generation stale: W6 row says "**No active
  queue**", Last Handoff narrates only the EF queue; zero knowledge of the OOS retriage or
  PB-OS1..OS3. Self-declares source of truth; a fresh session obeying it misjudges state.
- **N3** Auto-memory `MEMORY.md:51` carries the same "no active queue" claim (numbers
  otherwise correct).
- **N4** `memory/primitives/oos-retriage-plan-2026-07-18.md` never marked **PB-OS1**
  shipped: still "RECOMMENDED FIRST DISPATCH" (:191), un-struck row (:343), live §4 spec —
  while OS2/OS3 carry ✅ banners. **Re-dispatch risk** for the dispatch loop. Root cause:
  OS1 was collected during the DOC interlude and its plan-closure step was skipped; the
  OS2/OS3 workers closed their own banners, nobody backfilled OS1.
- **N5** `memory/primitive-wip.md` not reset after OS3 (phase close-out/DONE);
  `/implement-primitive` no-args would surface finished OS3 and halt instead of starting OS4.
- **Process root cause**: `/collect` updates CLAUDE.md:53 + the archive changelog but not
  line 17, workstream-state, or the plan banner of a PB collected out-of-band; `/eot` (which
  rotates workstream-state) has not run since the EF rotation.

### Theme 2 — Skills wired to retired docs (MEDIUM; missed by audit #1 and DOC-2)

DOC-2 bannered the stale docs but nothing re-wired the skills that read/WRITE them:

- **S1** `/implement-primitive` SKILL.md reads *and writes* retired `docs/project-status.md`
  (lines 75, 87, 109, 278-280) and reads batch specs from historical
  `docs/primitive-card-plan.md` (86, 114, 203, 216). Active queue actually lives in
  `oos-retriage-plan-2026-07-18.md`.
- **S2** `/start-work` SKILL.md is built entirely on the retired W1–W6 model: reads
  project-status (27, 68, 75, 89, 95), workstream-coordination (42), ability-batch-plan
  (63), primitive-card-plan (68, 74, 96). Decide: rewire or retire the skill.
- **S3** `/audit-cards` SKILL.md writes retired docs (152-154: project-status Card Health,
  workstream-coordination Phase 5) and reads historical card-authoring-operations.
- **S4** Auto-memory `MEMORY.md:36,109` still routes progress tracking through
  `docs/project-status.md` (only caveats one section) — contradicts full retirement.

### Theme 3 — Content drift, smaller (MEDIUM→INFO)

- **D1** `docs/engine-invariants.md:81` "**HASH_SCHEMA_VERSION is now 37**" — actual 55;
  "is now" phrasing invites stale-current misread (rephrase "was 37 when SR-7 landed").
  Same doc cites pre-SR-9a bare paths `tests/keyword_registry.rs` etc. (:54,:76,:124) that
  its own SR-9a entry says were abolished (modules exist; invocation form wrong).
- **D2** `docs/mtg-engine-network-security.md`: living design doc, in CLAUDE.md index, but
  absent from docs.yaml AND its documented exclusion list, and carries no marker — invisible
  to both tracking modes. `docs/authoring-status-guide.md` (self-declared hand-written +
  evolving) likewise unmarked/untracked. `docs/sr-24-lki-capture-cost.md` is the one SR
  record without a marker.
- **D3** `docs/authoring-status.md` stamp: generated from `e991b237` on the PB-EF12 feature
  branch — numbers still match (1,798 / 62.1%) but provenance points at a non-main commit;
  regenerate on main post-collect.
- **D4** `docs/mtg-engine-corner-case-audit.md` last audited 2026-03-08 — a docs.yaml
  living ledger un-refreshed through the entire EF/OS campaign. `docs/mtg-engine-roadmap.md`
  gives no signal M9.5 shipped (design intent: CLAUDE.md tracks it; one pointer line would
  fix). `type-consolidation.md` vs `milestone-reviews.md:2142` disagree 5-clusters/8-sessions
  vs 4/6 (both historical, nil impact).
- **D5** `memory/card-authoring/sr34-engine-findings-2026-07-17.md` presents SF-8/SF-9 as
  open; both shipped in SR-36. `campaign-plan-2026-05-16.md` §0 "Next action" still names
  the completed EF queue as active — no mention of PB-OS.
- **D6** `workstream-state.md:3-4` header cites removed `/start-session`/`/end-session`;
  `.claude/skills/eot/SKILL.md:103` routes to `memory/MEMORY.md` — a repo path that doesn't
  exist (the real file is auto-memory; implement-ability:253 spells it correctly).
- **D7** One uncommitted file on main: `memory/doc-audit-2026-07-18.md` (closure addendum).
  Not a worker-branch hazard (worktrees branch from committed HEAD) but §8/B.7 forbids
  `git add -A` around it and Gate B stop-flags it. Commit it.

### Confirmed healthy (no action)

Banners all accurate with existing successors; project-status retirement note accurate;
docs.yaml shape+triggers valid; archive READMEs reconcile with plan and disk; §3 rewrite
coherent, memory/ compliant; primitives stale-duplicate WIP correctly annotated and all ~30
skill/agent refs point at the live file; EF-era findings files carry correct ✅ CLOSED
banners; ef-batch-plan §10 closures for EF-EF1-A and OOS-EF6-1 correct; agents clean on
post-SR-6 crate layout and post-SR-9a test layout; .mcp.json/CLAUDE.local.md/settings
consistent; `ability-wip.md` IDLE banner exemplary.

## Comparison with audit #1

Audit #1's fixes: **verified durable**. What #1 missed and #2 caught: (a) skill *content*
wiring to retired docs (S1–S3) — #1 only verified skill existence; (b) the coordination-state
freshness break (N1–N5) — largely *created* by the remediation interlude itself interrupting
collect-time bookkeeping; (c) engine-invariants extraction leftovers (D1). Lesson for the
audit method: any future doc audit must read skill/agent bodies, and any pause of a work
queue must include a state-resync step on resume.

## Task set

| ID | Pri | Task |
|----|-----|------|
| DOCB-1 | HIGH | **State resync before OS4 dispatch** (coordinator-inline, one sitting): fix CLAUDE.md:17; rotate workstream-state.md with a real OOS-retriage+OS1..3 handoff; fix MEMORY.md:51 + :36/:109; mark OS1 ✅ SHIPPED in oos-retriage-plan (banner + strike row + close §4); reset primitive-wip.md to idle/OS4-next; regenerate authoring-report on main; commit the doc-audit addendum + this report |
| DOCB-2 | MED | **Rewire skills off retired docs**: /implement-primitive + /audit-cards onto live sources (oos plan, authoring-status, workstream-state); decide /start-work fate (rewire vs retire — W-model is frozen); add a collect/resume state-sync step so an interrupted queue can't strand plan banners again |
| DOCB-3 | LOW | **Polish batch**: engine-invariants "is now 37"→past-tense + SR-9a-form test paths; network-security + authoring-status-guide markers & docs.yaml entries (or documented exclusion); sr-24 marker; roadmap current-milestone pointer line; eot SKILL.md auto-memory path; workstream-state header skill names; campaign-plan §0 repoint; sr34 findings ✅ closure banners; corner-case-audit refresh or dated-defer note |

Sequencing: DOCB-1 is a hard gate before dispatching PB-OS4 (N4 is a live re-dispatch
hazard). DOCB-2 before the next time any of those three skills is invoked. DOCB-3 anytime.
