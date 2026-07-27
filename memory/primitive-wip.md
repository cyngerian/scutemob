# Primitive WIP — PB-DP10 (decision-gate widening: stop the 277-def figure growing silently)

<!-- last_updated: 2026-07-27 -->

> Previous occupant: **PB-DP9 (DP-7 / DP-8 / DP-9: search, scry, surveil player choice) — SHIPPED**
> `scutemob-157`, merge `d65e7f1e`, PROTOCOL 30 → **31**, HASH 67 → **68**, tests **3,910** on main.
> Its WIP file is preserved verbatim at `memory/primitives/pb-wip-DP9-archive.md` (this file is
> rewritten wholesale by each `/implement-primitive` run); its plan/review are
> `memory/primitives/pb-plan-DP9.md` / `pb-review-DP9.md`, and its seeds are in
> `docs/audits/decision-point-audit.md` §8.1.

- **PB**: PB-DP10 — the **invariant-level** fix for the whole PB-DP suite. Audit §8's last row.
- **Task**: `scutemob-158`
- **Branch**: `feat/pb-dp10-widen-the-decision-gate-stop-the-277-def-engine-gues`
- **Class**: GATE / INVARIANT (test-only). Rank 10 of the PB-DP suite; **closes it**.
- **Phase**: implement
- **Plan**: `memory/primitives/pb-plan-DP10.md`
- **Review file**: `memory/primitives/pb-review-DP10.md`
- **Baseline**: PROTOCOL **31**, HASH **68**, tests **3,910** (main at merge `d65e7f1e`)
- **Hard constraint**: **NO engine change, NO wire change.** PROTOCOL 31 / HASH 68 must be
  unmoved and `crates/engine/src/` / `crates/card-types/src/` must be untouched. Card-def edits
  are allowed only if a completeness marker/note is itself the deliverable, and each one must be
  argued from oracle text. If the work appears to require an engine change, **stop and
  re-scope** (task brief, explicit).

## The problem

`crates/engine/tests/core/effect_choose_gate.rs` (SR-33/34/37/38 + PB-EF12) bars exactly
**three** DSL variants from `Complete` — `Effect::Choose`, `Effect::MayPayOrElse`,
`Effect::AddManaChoice` (plus the any-color family). Audit §3.1 counts **twenty-one** decision
sites across **277 of 1,139** effectively-`Complete` defs (24.3%) where the engine makes a
player's choice for them. Seventeen of those rows the gate does not name at all, so the figure
grows silently with every card authored. DP-INV (audit §1) is the invariant; the gate is
narrower than the invariant, and PB-DP10 closes that difference *at the corpus level* — it
cannot close it at the engine level, because that is what PB-DP1..DP9 were for and what the
still-open rows (DP-13/14/16/17/18/19/20/25/26/31) remain for.

## Acceptance criteria (ESM `scutemob-158`)

1. **5554** — a machine gate enumerates every def containing an engine-made choice, fails on
   unmarked new instances, and its count reconciles against §3.1's magnitude with discrepancies
   explained.
2. **5555** — decision classes fixed by PB-DP7..DP9 are distinguished from still-auto-chosen
   classes in the gate/marker taxonomy.
3. **5556** — no engine or wire change; PROTOCOL / HASH untouched; gate runs inside
   `cargo test --all`.
4. **5557** — audit PB-DP10 row updated + suite marked complete in §8; §10 re-audit triggers
   updated where the gate mechanizes them.

## Known assets to reuse (do not re-invent)

- **`pb_dp9_effect_choice.rs`'s `roster` module** — a *structurally complete* serde walk of the
  serialized `CardDefinition` (`contains_variant` / `collect`). It exists because PB-DP9's fix
  cycle found a hand-written walk had skipped `AbilityDefinition::{Spell,Triggered,Activated}::modes`,
  `{SagaChapter,LoyaltyAbility}`, split-card halves and `Effect::CoinFlip`. **A hand-written
  tree walk is a reachability claim** (audit §8, PB-DP9 row) — reuse the serde walk, and if it
  must be shared across test targets, share it rather than copying it.
- **`effect_choose_gate.rs`'s `def_uses` / `count_key_occurrences`** — same technique, older,
  plus the served-vs-unserved refinement (`registers_any_color_mana_ability`), which is the
  precedent for criterion 5555: *the same variant can be served on one path and a stub on
  another*, so a variant-name predicate alone is not a decision-class predicate.
- **`Completeness`** (`crates/card-types/src/cards/`) — SR-2's marker, and note that adding a
  variant to it is a `card-types` change, i.e. **inside the wire closure** — check the hash gate
  before assuming a new marker variant is free.

## Step Checklist

- [x] 1. Decide gate-widening vs new marker (plan) — **gate-side name-keyed baseline + union
      ratchet**; a new `Completeness` variant is wire-free (the WIP's own caution falsified,
      plan §1.1) but rejected on Architecture-Invariant-9 grounds and by the hard constraint.
      **Planning's headline finding**: the serde walk this batch inherits is **blind to unit
      variants** (`Effect::Proliferate`, `Effect::TheRingTemptsYou` serialize as bare JSON
      *strings*, and all three existing walks match object keys only), so a verbatim reuse
      would report 0 for Proliferate's ~25 `Complete` defs while looking green — plan §2.1, T2.
- [ ] 2. Enumerate the decision-site taxonomy: served (DP7..DP9) / still-auto-chosen / gated
- [ ] 3. Implement the gate + fail-closed allowlist
- [ ] 4. Non-vacuity probes, both directions, including the nesting case
- [ ] 5. §3.1 reconciliation, printed and explained
- [ ] 6. Build / test / clippy / fmt + `tools/check-defs-fmt.sh`; wire-neutrality proof
- [ ] 7. Audit §8 / §5 / §10 / §8.1 updates
