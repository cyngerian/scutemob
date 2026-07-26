# Primitive WIP — PB-DP4 (DP-10 attack tax never debited · DP-11 echo/CU/recover never enforce the "otherwise") · PLAN

<!-- last_updated: 2026-07-26 -->

> Previous occupant: **PB-DP3 (DP-4, modal mode announcement) — SHIPPED** `scutemob-151`,
> merge `3b04bd17`. Its record lives in `docs/audits/decision-point-audit.md` §5 DP-4 / §8,
> `memory/primitives/pb-plan-DP3.md` + `pb-review-DP3.md`, and the CLAUDE.md changelog entry.

- **PB**: PB-DP4 — two "the cost is checked but never collected" bugs of the same shape.
  - **DP-10** (CR **508.1g**): the Propaganda/Ghostly Prison attack tax is *inspected* once
    (`rules/combat.rs:250-253`) and **never debited** anywhere in the 600-line handler. Float
    the mana, attack free, keep the mana. The cost's colour is also flattened to a generic
    `u32` total (`:218-227`), so a coloured tax can be paid with the wrong colours.
  - **DP-11** (CR **702.30a** / **702.24a** / **702.59a**): echo, cumulative upkeep and
    recover. `resolution.rs:2785` asserts "The game pauses until a `Command::PayEcho` is
    received"; **no code implements that pause.** The three `pending_*` vectors are read only
    by their own handlers, by `state/hash.rs` and by `replay_harness.rs` — never by priority,
    SBA or step advancement (verified by grep, below). Pass priority and the permanent is
    neither paid for nor sacrificed. Compounding: none of the three has a `LegalAction`, so a
    bot / M11-local seat never sends the command at all.
- **Task**: `scutemob-152`
- **Branch**: `feat/pb-dp4-costs-checked-but-never-collected-propaganda-attack-t`
- **Class**: CORRECTNESS (Tier 1, class **D** both). Rank 4 of the PB-DP suite.
- **Phase**: implement — plan APPROVED by the coordinator 2026-07-26. Two load-bearing claims
  spot-verified independently before approval: (a) `resolution.rs:7768-7772` really does clear
  `players_passed` and grant priority to the active player at the end of every resolution, so
  the owing player is guaranteed a window (and Change 2d's deletions are identity writes for
  echo/CU); (b) `handle_all_passed`'s two branches at `engine.rs:1694-1711` / `:1711+` really
  are disjoint and the non-empty branch returns early, so the sweep cannot fire in the call
  that created the entry. **One scope note for the reviewer**: Change 1c
  (`has_uncosted_attack_target`, CR 508.1d, closes OOS-RS3-4) fixes a *pre-existing* deadlock
  that PB-DP4 does not create. It is accepted into scope because it is the same
  hang-the-game failure class that hard constraint (b) forbids for DP-11, and because it is
  directly adjacent to the code Change 1a rewrites — but it is the one item in this PB that a
  reviewer could reasonably call scope creep, and it should be judged on that basis rather
  than waved through.
- **Binding spec**: `docs/audits/decision-point-audit.md`
  - §4.5 combat table, **line 266** — "Attack cost (Propaganda) | 508.1g | **D** | `rules/combat.rs:248-263`"
  - §4.11 cleanup table, **line 393** — "Echo / cumulative upkeep / recover pay-or-sacrifice | 702.30a / 702.24a / 702.59a | **A** plumbing, **D** enforcement"
  - §5 **line 442** (DP-10 row), **line 443** (DP-11 row)
  - §8 **line 573** (PB-DP4 row) — *"Wire: **none** if the "otherwise" is applied at
    resolution rather than gated on priority"*
  - §8.1 **line 599** (**OOS-DP1-1**) — echo/CU/recover `handle_pay_*` write
    `priority_holder = Some(active_player)` at *resolution* time, when no player holds
    priority. PB-DP1 correctly left it alone (comment-only). *"The write is a bodge standing
    in for the payment pause DP-11 says was never implemented. Correct fix is the pause
    itself, owned by **PB-DP4**."* — **this seed is in scope for this PB.**
  - §9 recommendation 3 (**line ~709**) and recommendation 6 (**line ~734**) — the M11-local
    consequence: `advance()` should yield `AwaitingHuman` for a non-empty pending-payment
    vector, and the three commands currently have no `LegalAction` at all.
- **Plan file**: `memory/primitives/pb-plan-DP4.md`
- **Review file**: `memory/primitives/pb-review-DP4.md`

## Coordinator pre-survey (a hypothesis for the planner to falsify, **not** a fact base)

> The PB-DP3 wip file records that three of its five pre-survey "facts" were wrong in both
> directions. Treat every bullet below as something to verify, and correct it in the plan.

**DP-10 sites and API**

- `rules/combat.rs:~199-263` is the whole tax block. It builds
  `HashMap<PlayerId, u32> tax_per_attacker` by summing `generic + white + blue + black + red +
  green + colorless` of `cost_per_creature`, counts attackers per taxed defender, sums a
  `total_tax: u32`, compares it against `ps.mana_pool.total_with_restricted()`, and **returns
  `Ok` without touching the pool**. The in-code comment openly states the deferral:
  *"Interactive payment is deferred to post-alpha (requires a new DeclareAttackers command
  field)."* That premise needs testing — a full auto-debit needs no new command field.
- Payment API already exists and is what the echo handler uses:
  `casting::can_pay_cost(pool, &cost)` / `casting::pay_cost(&mut pool, &cost)`, over
  `ManaPool::can_spend` / `ManaPool::spend` (`crates/card-types/src/state/player.rs:148,177`).
  `spend` takes an `Option<&SpellContext>` for restricted mana (CR 106.12) — an attack tax is
  **not** a spell, so what happens to restricted mana here is a real design question, not a
  detail: `total_with_restricted()` currently *counts* restricted mana toward affordability.
- `ManaPool::spend`/`can_spend` `debug_assert_flattened(cost)`: a hybrid/Phyrexian
  `cost_per_creature` must be flattened first (PB-RS2 precedent, CR 107.4e/f).
- Three defs carry `GameRestriction::CantAttackYouUnlessPay`: `propaganda.rs`,
  `ghostly_prison.rs`, `goblin_rabblemaster.rs` (the last only mentions it in a comment —
  confirm). Both real ones are `{2}` generic, so **expect 0 card-def edits**; the colour bug
  is latent, not live-wrong, and the tests must create the coloured case synthetically.
- ⚠ **Known adjacent hazard, already documented in-corpus**:
  `crates/card-defs/src/defs/goblin_rabblemaster.rs:35-52` carries a long accepted-limitation
  note from the PB-RS3 review — `combat.rs:421-424`'s must-attack "able" test never reads
  `CantAttackYouUnlessPay`, so a forced attacker + an unpayable tax on every viable opponent
  is a genuine deadlock. Per the 2014-07-18 Rabblemaster ruling and **CR 508.1d**, *"if
  there's a cost associated with having a creature attack, you're not forced to pay that
  cost."* Making the tax a real debit does not create this, but it does make it matter more.
  Decide explicitly: fix it here, or restate it as a seed. Do not silently inherit it.

**DP-11 sites and the "no pause" claim**

- Verified by grep — the only readers of the three vectors outside their own handlers are
  `state/hash.rs:7736-7748` (hashing), `state/builder.rs:337-339` (init),
  `state/mod.rs:535-549,774-792` (accessors + escape hatches) and
  `testing/replay_harness.rs:912`. **Nothing** in `rules/priority.rs`,
  `rules/engine.rs::handle_all_passed`, `rules/turn_structure.rs` or `rules/sba.rs` consults
  them. The audit's claim holds.
- Producers: `rules/resolution.rs` — echo `:2800-2845`, cumulative upkeep `:2846-2900`,
  recover `:2901-2960`. Each checks the CR 400.7 still-in-zone condition, emits
  `*PaymentRequired`, pushes the pending entry, and emits `AbilityResolved`.
- Consumers: `rules/engine.rs` — `handle_pay_echo:590`, `handle_pay_cumulative_upkeep:779`,
  `handle_pay_recover:1013`. Each removes the pending entry, and already implements **both**
  branches (`pay: true` ⇒ debit + keep; `pay: false` ⇒ sacrifice / exile, bypassing
  indestructible per CR 701.21a). **The consequence logic exists and is believed correct — the
  missing piece is only that nothing ever calls it.** Prefer reusing these handlers over
  writing a second copy of the consequence.
- Design question the plan must settle, with the §8 "no wire change" constraint binding:
  1. **Gate advancement** — refuse to leave the priority round / step while a pending payment
     is outstanding. Most CR-faithful, but a fuzzer or a script that never sends `Pay*`
     **deadlocks**, which is strictly worse than today's free survival. If chosen, it needs a
     forced-resolution backstop.
  2. **Auto-resolve at the advancement boundary** — when the game would leave the point where
     the payment was created, treat an unanswered payment as declined and run the existing
     `pay: false` path (or auto-pay if trivially affordable). No deadlock, no wire change; the
     deviation is that a player may hold priority with the payment outstanding.
  3. **Resolve inside trigger resolution** — never push a pending entry; decide immediately.
     Zero deadlock risk, but it deletes the player's agency and makes `Command::PayEcho`
     unreachable, which contradicts acceptance criterion 3.
  The audit's own wording ("applied at resolution rather than gating priority") leans away
  from (1). Pick one, argue it against CR, and state the deviation explicitly.
- **OOS-DP1-1 is in scope**: whatever mechanism is chosen must remove the need for the three
  `priority_holder = Some(active_player)` bodges in the `handle_pay_*` handlers, or explain
  why they survive.
- Existing tests that will constrain the design (read them before designing):
  `crates/engine/tests/mechanics_e_l/echo.rs`,
  `crates/engine/tests/mechanics_a_d/cumulative_upkeep.rs`,
  `crates/engine/tests/mechanics_m_z/recover.rs`,
  `crates/engine/tests/rules/restrictions.rs` (attack tax),
  and the golden script `test-data/generated-scripts/stack/153_recover_grim_harvest.json`.
- Affected defs: echo — `mogg_war_marshal.rs`, `avalanche_riders.rs`; cumulative upkeep —
  `tombstone_stairwell.rs` (`partial`), `mystic_remora.rs` (`known_wrong`); recover —
  `grim_harvest.rs`, `bala_ged_recovery.rs`. Check whether any completeness marker can be
  *upgraded* by this fix, and whether any `Complete` card is live-wrong today.

**Acceptance criteria (ESM `scutemob-152`)**

1. (5527) Attack tax actually debited with correct colours on declaration; tests cite CR
   508.1g; declaring without payable tax is rejected.
2. (5528) Echo / CU / recover: failing to pay reaches the CR-mandated consequence
   (sacrifice / exile per 702.59a); the permanent can no longer survive unpaid by passing
   priority; tests cite CR 702.30a / 702.24a / 702.59a.
3. (5529) `LegalActionProvider` exposes the pay/decline choice for all three payment kinds;
   bots make legal choices.
4. (5530) `cargo test --all`, clippy, `cargo fmt --check` **and** `tools/check-defs-fmt.sh`
   clean; **no wire change (PROTOCOL 27 / HASH 63)** or a documented re-scope; audit
   DP-10/DP-11 rows + the PB-DP4 row updated.

**Hard constraints**

- **No new `Command` or `GameEvent` variant and no new field on either.** PROTOCOL 27 /
  HASH 63 must be unmoved. `PayEcho`, `PayCumulativeUpkeep`, `PayRecover` and the three
  `*PaymentRequired` events all already exist. New `LegalAction` variants are
  simulator-internal and are **not** a wire change (PB-RS2 / PB-DP3 precedent).
  If the design genuinely requires a wire change, **stop and re-scope** in a task comment
  rather than bumping the constants unilaterally.
- SR-4: any new silent-failure site in `effects/mod.rs` / `rules/resolution.rs` must pick a
  side (`expect_*` vs `lki_*`).
- `crates/simulator`, `tools/tui` and `tools/replay-viewer` have exhaustive matches that break
  on new enum variants — `cargo build --workspace` after every phase.
- `state/mod.rs` is sealed `pub(crate)` (SR-3); the pending vectors already have accessors and
  `_mut` escape hatches — prefer the accessors, and do not widen the seal.

## Runner progress log

- [x] Change 1a — `combat.rs` attack-tax block rewritten as bound
      `(Option<ManaCost>, BTreeSet<PlayerId>)` expression, hybrid/Phyrexian/X rejected,
      `BTreeMap` determinism, affordability via `casting::can_pay_cost` (`spell: None`).
      `cargo check -p mtg-engine` clean.
- [x] Change 1b — debit + `GameEvent::ManaCostPaid` inserted after the enlist-tap loop,
      before "Record attackers in combat state".
- [x] Change 1c — `has_uncosted_attack_target` helper added; both goad and
      `MustAttackEachCombat` `no_legal_target` computations replaced with calls to it.
      Closes OOS-RS3-4.
- [x] Change 1d — `add_mana_cost` helper added (rejects hybrid/Phyrexian/X via
      `debug_assert!`), kept separate from `engine.rs::multiply_mana_cost` (OOS-DP4-7).
- [x] Change 2a — `force_resolve_overdue_payments` added in `engine.rs` after
      `handle_pay_recover`. Reads the pending vectors only; does not name
      `KeywordAbility::Echo`/`::CumulativeUpkeep`/`::Recover` (registry-gate safe).
- [x] Change 2b — hooked into `handle_all_passed`'s stack-EMPTY branch, before
      `empty_all_mana_pools`. Guard is `!payment_events.is_empty()`; extra-round branch
      returns (does not fall through).
- [x] Change 2c — recover decline branch now uses `expect_move_object_to_zone` (infallible).
- [x] Change 2d — deleted all three `priority_holder = Some(active)` / `players_passed =
      OrdSet::new()` bodges (echo, CU, recover), replaced with explanatory comments.
      Closes OOS-DP1-1.
- [x] Change 2e — CR 119.4 life-total gate added to `CumulativeUpkeepCost::Life` pay arm.
- [x] Change 2f — comment corrections in `resolution.rs` (3 sites), `state/mod.rs` (3
      fields), `card-types/src/state/player.rs` (2 sites, CR 106.12 -> 106.6).
- `cargo check --workspace` clean after Change 2.
- Card-def comment edit: `goblin_rabblemaster.rs` accepted-limitation paragraph replaced
  with a one-line "CLOSED by PB-DP4" note (the one card-def edit this PB makes, per plan §9).
