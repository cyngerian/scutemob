# Engine Audits

Correctness audits targeting systemic bug classes in the rules engine. Each audit
identifies a category of implementation error, lists every affected callsite, and
proposes a uniform fix.

These are distinct from milestone code reviews (which cover new code) and card
batch reviews (which cover card definitions). Audits target cross-cutting issues
that affect multiple subsystems and can only be found by systematic search.

**Methodology**: See [methodology.md](methodology.md) for the invariant-driven audit
process used to find these bugs. The process is domain-agnostic and applicable to any
complex rule engine.

## Audits

| Audit | Bug Class | Sites | Status |
|-------|-----------|-------|--------|
| [layer-bypass-audit.md](layer-bypass-audit.md) | Code reads static CardDefinition instead of layer-resolved state for battlefield objects | 9 HIGH | Open — scheduled for M10 engine correctness pass |
| [stress-test-scenarios.md](stress-test-scenarios.md) | 27 card combinations known to stress MTG engine implementations | 27 scenarios | Proposed — P1 scenarios (S-01 to S-05) scheduled with layer bypass fixes |
| [event-log-diagnosability.md](event-log-diagnosability.md) | Event log lacks "why" information needed to diagnose legal-but-wrong bugs | 3 tiers proposed | Proposed — Phase 1 with M10 layer fixes, Phase 2 with networking, Phase 3 with UI |

## Completed Audits

| Audit | Bug Class | Result |
|-------|-----------|--------|
| Stale ObjectId after zone change | Code uses old ObjectId after `move_object_to_zone` | Clean — all 31 callers use new ID |
| Missing phased-out checks | Battlefield scans without `is_phased_in()` | Clean — systematic enforcement |
| Owner vs Controller confusion | Wrong field for zone context | Clean — correct separation |
| Protection (DEBT) gaps | Missing protection checks on non-targeting effects | Clean — centralized in `protection.rs` |
| APNAP trigger ordering | Triggers not sorted by active-player-next-active-player | Clean — explicit sort in `flush_pending_triggers` |
| "As enters" vs "when enters" | Replacement effects modeled as triggers or vice versa | Clean (1 documented intentional gap) |
