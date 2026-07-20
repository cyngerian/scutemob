# Primitive WIP — PB-RS2 (OOS-RS-2 + OOS-OS8-1) · PLAN PHASE

<!-- last_updated: 2026-07-20 -->

- **PB**: PB-RS2 — activated-cost hybrid/Phyrexian pip payment (every such pip is free today)
- **Task**: `scutemob-144`
- **Branch**: `feat/pb-rs2-activated-cost-hybridphyrexian-pip-payment-every-such`
- **Class**: CORRECTNESS, LIVE (silent undercharge on 7 shipped filter lands; Invariant #9)
- **Phase**: plan
- **Binding spec**: `memory/primitives/rider-seed-triage-2026-07-19.md` §2.2 (chain notes) + §3 (R2 row)
- **Plan file**: `memory/primitives/pb-plan-RS2.md`
- **Review file**: `memory/primitives/pb-review-RS2.md`
- **Wire expectation**: **PROTOCOL bump EXPECTED and machine-forced** (SR-8) — `Command::ActivateAbility`
  gains fields. HASH: expected unchanged unless a hashed struct moves; any movement must be justified
  in the plan, not silently re-pinned.
- **Sequencing constraint**: **do NOT batch with R6** (independent-verification collision flag,
  triage §3 sequencing note).

## The chain (from triage §2.2 — verify each hop before acting)

1. `casting.rs:3990-3991` flattens hybrid/phyrexian **before** payment; life deducted `:4015-4021`.
   **Cast path is correct.**
2. `abilities.rs:748-758` gates on `resolved_cost.mana_value() > 0`, then calls `can_spend`/`spend`
   on the **raw** cost. **No flatten.**
3. `player.rs:148-175`, `:185-206` — `can_spend`/`spend` read only six colors + generic.
   **`cost.hybrid` and `cost.phyrexian` are never read.**
4. `game_object.rs:133-153` — `mana_value()` *does* count hybrid/phyrexian, so a pure `{B/R}` cost
   has mv=1, passes the `> 0` gate, then `can_spend` sees an all-zero cost → always true;
   `spend` deducts nothing.
5. `command.rs:78-102` — `Command::ActivateAbility` has **no** `hybrid_choices` /
   `phyrexian_life_payments` fields (they exist only on `CastSpell`, `command.rs:643`). The player
   cannot *express* the choice. **Schema gap, not just a missing flatten call.**

## Steps

- [x] 0. Step-0 probe written FIRST at
      `crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs` (registered in
      `crates/engine/tests/primitives/main.rs`), BEFORE any production edit. Two probes:
      `probe_hybrid_pip_is_currently_free_activated_ability` (abilities.rs path) and
      `probe_hybrid_pip_is_currently_free_mana_ability` (mana.rs path, Graven Cairns). Ran against
      pre-fix HEAD:
      ```
      $ ~/.cargo/bin/cargo test -p mtg-engine --test primitives probe_hybrid_pip -- --nocapture
      running 2 tests
      test pb_rs2_activated_pip_payment::probe_hybrid_pip_is_currently_free_activated_ability ... ok
      test pb_rs2_activated_pip_payment::probe_hybrid_pip_is_currently_free_mana_ability ... ok
      test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 689 filtered out; finished in 0.02s
      ```
      Both PASS pre-fix (asserting `Ok(_)`), confirming: (a) a `{B/R}` stack-using activated
      ability activates for free with an empty pool, and (b) Graven Cairns's `{B/R},{T}` filter
      ability produces 2 mana from an empty pool (live shipped-card bug, mana.rs path — confirms
      §0.2's correction that this PB is NOT just an `abilities.rs` fix). Confirms the plan's premise
      exactly; no re-scope needed. Probes will be inverted to `Err(InsufficientMana)`-asserting
      permanent regressions (renamed per plan §9.1) once the fix lands, and the file will be
      expanded with the full §9.2-9.6 mandatory test suite.
- [ ] 1. `Command::ActivateAbility` gains hybrid/Phyrexian payment-choice fields mirroring
      `CastSpell`. PROTOCOL bump machine-forced; justified in plan. HASH unchanged or justified.
- [ ] 2. Extract the `casting.rs` flatten logic into a shared helper — **no second open-coded copy**.
- [ ] 3. Thread choices through `handle_activate_ability` into the payment path (mana + Phyrexian
      life deduction, mirroring `casting.rs:4015-4021`).
- [ ] 4. `can_spend`/`spend` fail loud on non-empty hybrid/phyrexian residue, SR-4-classified
      diagnostics (`expect_*` vs `lki_*`). Test proving an unflattened cost cannot silently pass.
- [ ] 5. Simulator `LegalActionProvider` + harness `translate_player_action` handle the new fields
      (no illegal/lethal bot suggestions); extend SR-31 equivalence coverage.
      `cargo build --workspace` verifies replay-viewer/TUI exhaustive matches.
- [ ] 6. Card dispositions — 7 filter lands (`twilight_mire`, `graven_cairns`, `sunken_ruins`,
      `flooded_grove`, `rugged_prairie`, `fetid_heath`, `cascade_bluffs`) charge their real costs;
      per-card disposition documented (Complete flip **or** honest remaining-blocker note — they are
      `known_wrong` for an unrelated fixed-mode simplification, `graven_cairns.rs:49-52`).
- [ ] 7. `birthing_pod` authorable and flipped; `drivnod_carnage_dominus.rs:43-44`'s false
      "already expressible (PB-9)" claim corrected.
- [ ] 8. Mandatory tests: hybrid both-halves, Phyrexian mana-vs-life (CR citations, Invariant #8),
      residue guard, filter-land regression.
- [ ] 9. Gates: `cargo build --workspace`, `cargo test --all`, `cargo clippy --all-targets -D
      warnings`, `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35).
- [ ] 10. `primitive-impl-reviewer` pass; disposition every finding.

## Prior state

PB-RS1 SHIPPED (`scutemob-143`, merge `56697a00`) — library top/bottom reconciliation. The R1..R11
ranked queue lives in `memory/primitives/rider-seed-triage-2026-07-19.md` §3.
