# Invariant-Driven Audit Methodology

**Date**: 2026-03-27
**Purpose**: A repeatable process for finding systemic bugs in complex rule engines.
Applicable to this MTG engine and generalizable to any system where correctness is
domain-defined, multiple representations of the same data coexist, and inputs can
change the rules the system operates under.

---

## The Problem With Testing Alone

Unit tests verify individual behaviors. Integration tests verify combinations. But
complex engines have a category of bug that neither catches reliably:

**Legal-but-wrong states**: The system is internally consistent — no crashes, no
assertion failures, no structural violations. But it applies a rule incorrectly. The
state is *valid* but *not what the domain says should happen*.

These bugs resist detection because:
- The system doesn't know it's wrong (no error to catch)
- Tests only cover combinations someone thought to write
- The bug may only manifest under specific input combinations that are individually
  well-tested but produce incorrect interactions

Example: A Magic engine correctly implements the layer system AND correctly implements
spell cost modifiers, but reads cost modifiers from the wrong data source (static
definition vs. layer-resolved state). Under normal play, both sources agree. Under
Humility (which removes abilities via the layer system), they diverge — and the engine
produces a legal but incorrect game state.

---

## The Five-Step Process

### Step 1: Document Architectural Invariants

An invariant is a property that must hold everywhere in the system, not just in tested
scenarios. Invariants are not tests — they're *about* the system, at a higher level
than any individual test.

**How to find them:**
- What assumptions does the architecture make that, if violated, produce wrong results?
- What "must always be true" statements appear in design docs, code comments, or
  verbal descriptions of the system?
- Where does the system maintain multiple representations of the same information?
  Each pair of representations implies an invariant: "these two always agree under
  condition X."

**Examples from this project:**
- "All observable characteristics of battlefield permanents flow through the layer system"
- "When an object changes zones, it becomes a new object — old references are dead"
- "Triggered abilities go on the stack in APNAP order"
- "State-based actions are checked simultaneously, not sequentially"

**Examples in other domains:**
- Compiler: "Optimization passes preserve the semantics of the input program"
- Physics engine: "No object moves faster than its maximum velocity after constraint solving"
- Distributed system: "All replicas converge to the same state after quiescence"
- Financial system: "Account balance equals sum of all transactions"

**Format:** Write each invariant as a declarative statement. Keep a living list. When
a new bug is found, ask "what invariant was violated?" and add it if missing.

### Step 2: Classify Each Invariant by Verifiability

Not all invariants can be verified the same way. Classify each:

**A. Greppable violation** — the invariant's violation has a detectable code pattern.
You can search the entire codebase and find every instance.

Examples:
- "All battlefield reads go through the layer system" → violation pattern:
  `card_registry.get()` for battlefield objects without `calculate_characteristics()`
- "Zone changes always use new ObjectId" → violation pattern: old ObjectId used
  after `move_object_to_zone()` call
- "Phased-out permanents don't exist" → violation pattern: battlefield scan without
  `is_phased_in()` check

**B. Scenario-testable** — the invariant can't be grepped, but specific inputs are
known to stress it. Domain experts know which combinations historically break
implementations.

Examples:
- "Layer system applies effects in correct order" → stress inputs: Humility +
  Opalescence (circular dependency), multiple anthems (timestamp ordering)
- "APNAP ordering is correct in multiplayer" → stress input: 4 players with
  simultaneous "lose the game" triggers (Nine Lives combo)

**C. Observable only at runtime** — the invariant can't be grepped or tested with
known inputs. Violations only surface during real usage. The system must explain
itself when something goes wrong.

Examples:
- "The engine never enters an infinite loop" → needs loop detection + timeout
- "Complex three-way card interactions produce correct results" → needs diagnostic
  logging so that when a player reports "this seemed wrong," the event log captures
  enough to diagnose the root cause

### Step 3: Audit Greppable Invariants Exhaustively

For each Class A invariant:

1. **Define the violation pattern** as precisely as possible
2. **Search the entire codebase** — every file, every occurrence
3. **Classify each site**: true violation, false positive (intentional/correct), or
   uncertain (needs manual analysis)
4. **Document every site** in an audit file with location, context, severity, and
   proposed fix
5. **Fix all sites** in a single batch — the fix is usually uniform because the
   violation pattern is uniform

**Key property**: This is *exhaustive*. After the audit, the entire class of bug is
eliminated, not just the instances you happened to think of. New code introducing the
same pattern is caught in code review because the violation pattern is documented.

**Cost**: One audit per invariant. Usually a few hours of searching and a day of
fixing. The ROI is enormous — you're eliminating a bug *class*, not a bug.

### Step 4: Write Adversarial Scenarios for Testable Invariants

For each Class B invariant:

1. **Gather domain knowledge** about what inputs stress the invariant. In MTG, this
   means cards known to break engines (Humility, Blood Moon, Panharmonicon). In
   physics, it's edge cases like zero-velocity objects or simultaneous collisions.
   In compilers, it's pathological nesting or self-referential types.
2. **Write concrete test scenarios** with:
   - Setup (exact initial state)
   - Action (what happens)
   - Expected result (what should happen, with domain rules citation)
   - Subsystems being stressed
3. **Prioritize by likelihood** — how often would a real user encounter this?
4. **Convert to automated tests** as capacity allows

**Key property**: These scenarios come from domain expertise, not random exploration.
The MTG community has 30 years of "this card breaks everything" knowledge. Physics
engines have known failure modes documented in GDC talks. Leverage existing knowledge
about what breaks systems like yours.

**Sources of adversarial knowledge:**
- Community forums (Reddit, Stack Overflow, domain-specific communities)
- Bug trackers of similar projects (MTGO bugs, Forge/XMage issues)
- Domain rules errata and changes (rules that changed imply previous implementations
  were wrong — what else might be wrong the same way?)
- "War stories" from experienced practitioners

### Step 5: Add Observability for Runtime-Only Invariants

For each Class C invariant:

1. **Add diagnostic events** that capture the engine's reasoning, not just its
   conclusions. "Spell cost = 3" is a conclusion. "Spell cost = 3 because base
   cost 2 + Thalia tax 1, layer-resolved: true" is reasoning.
2. **Tag diagnostics** so they're filterable — hidden during normal play, included
   in bug reports, visible in debug/replay mode.
3. **Design for the bug report workflow**: when a user says "this seemed wrong,"
   the diagnostic log should contain enough to reproduce and diagnose without
   needing to set up the exact board state manually.

**Key property**: You're not trying to prevent these bugs — you're accepting they'll
happen and ensuring they're diagnosable. The goal is "one bug report → root cause
identified" instead of "ten bug reports → still guessing."

---

## When to Re-Audit

Audits are not one-and-done. Re-audit when:

- **Major refactors** change the code patterns that the audit relied on
- **New subsystems** are added that might not follow established patterns
- **New invariants** are discovered (usually from a bug that revealed a property
  you didn't know needed to hold)
- **Domain rules change** (like the 2025 Saga rules update) — the invariant may
  still hold, but the implementation may need updating to match the new rule

A lightweight re-audit (re-run the grep patterns, check for new sites) takes 30
minutes and should be part of milestone completion checklists.

---

## Applying This Outside MTG

The methodology is domain-agnostic. The steps are the same; only the invariants,
violation patterns, and adversarial inputs change.

### Game physics engine

| Step | Application |
|------|-------------|
| Invariants | "No interpenetration after solver", "Energy is conserved", "Constraints are satisfied" |
| Greppable violations | Direct position writes that bypass the constraint solver |
| Adversarial scenarios | Zero-mass objects, simultaneous multi-body collisions, objects at rest on slopes |
| Observability | Per-frame solver iteration counts, constraint violation magnitudes, energy delta |

### Compiler optimization passes

| Step | Application |
|------|-------------|
| Invariants | "Each pass preserves input semantics", "No undefined behavior introduced", "Type system is sound" |
| Greppable violations | IR mutations without corresponding verifier calls |
| Adversarial scenarios | Pathological nesting, self-referential types, optimization pass ordering edge cases |
| Observability | Before/after IR dumps per pass, semantic diff tooling |

### Distributed consensus system

| Step | Application |
|------|-------------|
| Invariants | "All replicas converge", "No committed transaction is lost", "Linearizability holds" |
| Greppable violations | State mutations without consensus protocol (direct writes) |
| Adversarial scenarios | Network partitions, leader failure during commit, clock skew |
| Observability | Per-node state hashes, operation logs with vector clocks, divergence detection |

### Financial transaction system

| Step | Application |
|------|-------------|
| Invariants | "Balance = sum of transactions", "No double-spend", "Audit trail is complete" |
| Greppable violations | Balance reads from cache without recomputation, writes without audit log entry |
| Adversarial scenarios | Concurrent withdrawals, partial failure during transfer, timezone boundary transactions |
| Observability | Transaction-level logging with before/after balances, reconciliation reports |

---

## Summary

```
1. Document invariants (properties, not tests)
2. Classify: greppable violation? known stress inputs? runtime-only?
3. Greppable → exhaustive audit (one-time, eliminates the class)
4. Stress inputs → adversarial test suite (from domain expertise)
5. Runtime-only → diagnostic observability (for when bugs surface)
6. Re-audit periodically (new code, new subsystems, domain changes)
```

The key insight is the progression from *infinite search space* ("are there bugs?")
to *finite, searchable space* ("are there bugs of this class?") to *specific sites*
("here are the exact lines"). Each step narrows the space. By the end, you've either
eliminated the bug class, written tests for known stress points, or ensured the system
explains itself when the unknown occurs.

This isn't about achieving zero bugs. It's about making the *category* of remaining
bugs as small and as diagnosable as possible.
