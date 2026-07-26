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
- **Phase**: fix cycle COMPLETE 2026-07-26 — review closed 2026-07-26 (0 HIGH / 5 MEDIUM / 13 LOW, verdict "ship after fixes"; see `memory/primitives/pb-review-DP4.md`). All 5 MEDIUMs fixed (T1/T2/T3/E1/E2); of 13 LOWs, 8 fixed and 5 declined-with-reason (3 folded into new seeds OOS-DP4-10/11/12 for the coordinator to file). See "Fix cycle complete (runner close-out)" at the end of this file for the full summary; the review file's own "## Fix cycle (runner)" section has the per-finding detail. Implementation complete 2026-07-26 (see "Implementation complete
  (runner close-out)" earlier in this file). Two load-bearing claims
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
- [x] Change 3a — three new `LegalAction` variants (`PayEcho`, `PayCumulativeUpkeep`,
      `PayRecover`) appended to `crates/simulator/src/legal_actions.rs` after
      `CastMorphFaceDown`.
- [x] Change 3b — `StubProvider::legal_actions` enumerates all three, appended after
      `PassPriority` (not early-returning, so `TapForMana` stays available per CR 608.2g).
      `pay: false` always offered; `pay: true` gated via `casting::can_pay_cost` (echo,
      recover) or a new private `multiply_mana_cost` mirroring `engine.rs`'s (CU mana) /
      the CR 119.4 life check (CU life). `life_total` hoisted earlier in the function to
      be usable by the new block (it was previously computed later).
- [x] Change 3c — `random_bot.rs::action_to_command` given the three new arms (compile
      error until added — the match has no catchall, confirmed working as the gate).
- [x] Change 3d — no change to `local_game.rs` (confirmed: the three actions arrive
      through the existing `PendingDecision` / `DecisionKind::Priority` path).
- [x] Change 3e — no TUI keybinding added (out of scope, per plan's "runner's call, but
      no more than this"; criterion 5529 names `LegalActionProvider` and the bots, not
      the TUI).

## Implementation complete (runner close-out)

**Summary**: All of §3 Changes 1a-1d, 2a-2f, 3a-3e implemented as specified. 22 new
engine tests (`crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs`,
registered in `primitives/main.rs`) + 8 new simulator tests
(`crates/simulator/src/legal_actions.rs`'s `mod tests`, plus 1 in the same module for
`action_to_command`). Tests: **3,747 → 3,777** (+30 = 22 + 8), 0 failing.
`cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --check`, `tools/check-defs-fmt.sh` all clean. `PROTOCOL_VERSION == 27`,
`HASH_SCHEMA_VERSION == 63` — read directly from source, confirmed unmoved, not edited.

### §4.7 negative-space clause — 2 un-enumerated sites hit, both fixed minimally

1. **`crates/engine/tests/core/bare_lookup_ratchet.rs::bare_lookup_counts_are_pinned`
   (SR-25 ratchet), `combat.rs` ceiling.** Change 1c's `has_uncosted_attack_target`
   deduplicates the two copy-pasted `has_cant_attack_owner` bare
   `state.restrictions.iter().any(|r| ... state.objects.get(&r.source) ...)` lookups (the
   goad block and the `MustAttackEachCombat` block) into one site, so `combat.rs`'s count
   dropped from the pinned 16 to 15. The gate fails on ANY change, up or down (it is a
   ratchet, not a ceiling-only check), with the "down" message asking to lower the ceiling
   to lock in the gain. Fixed: `SWEPT_FILES` entry changed to `("src/rules/combat.rs", 15)`
   with a dated changelog comment, matching the file's own established convention.
2. **Same gate, `engine.rs` ceiling.** Went from 22 to 24 (up, not down) — two new bare
   `.players.get(` sites: (a) the CR 119.4 life-cost gate (Change 2e) reads
   `state.players.get(&player).ok_or(GameStateError::PlayerNotFound(player))?.life_total`,
   the identical idiom the sibling `CumulativeUpkeepCost::Mana` arm a few lines above
   already uses for `.mana_pool`; (b) the boundary-sweep hook (Change 2b) reads
   `state.players.get(&active).map(|p| !p.has_lost && !p.has_conceded).unwrap_or(false)`
   to decide who gets priority for the extra round — a verbatim copy of `enter_step`'s
   existing `is_alive` predicate read a few dozen lines below in the same file. Both are
   NONSWALLOW predicate reads exactly matching the ratchet's own documented residue
   class (module doc: "predicate reads... where a departed object legitimately answers the
   predicate `false`"), not new silent-failure patterns. Fixed: `SWEPT_FILES` entry raised
   to `("src/rules/engine.rs", 24)` with a dated changelog comment classifying both sites
   (not a blind ceiling bump — each site is named and justified against an existing
   sibling idiom in the same file).

No other un-enumerated site fired: `cargo build --workspace` caught no TUI/replay-viewer
match-arm gaps (§4.6's prediction of "no change" held); `keyword_registry` and
`ability_definition_registry` gates passed clean on the first run (the sweep and the
provider never name `KeywordAbility::Echo`/`::CumulativeUpkeep`/`::Recover` — §4.5's
"lesson PB-DP3 paid for" was heeded); `crates/simulator/tests/local_game.rs` (9 tests,
not edited) stayed green; the golden scripts `stack/152` and `stack/153` both still show
"1 of 271 discovered scripts ran and passed" with no diff needed.

**One test-infrastructure change beyond the plan's enumerated files, self-contained to
test-only build config:** `crates/simulator/Cargo.toml` gained a
`[dev-dependencies] mtg-engine = { path = "../engine", features = ["test-util"] }` line.
The 8 new simulator tests need the `pending_*_payments_mut()` / `players_mut()` /
`turn_mut()` `GameState` escape hatches to seed payment scenarios directly (mirroring the
pattern `crates/engine/tests/mechanics_e_l/echo.rs` etc. already use), but those hatches
are gated `#[cfg(any(test, feature = "test-util"))]` on the whole `impl GameState` block,
and `crates/simulator`'s existing `mtg-engine` dependency doesn't carry that feature. This
is the exact same self-dependency trick `crates/engine/Cargo.toml` already uses for its
own integration tests (`[dev-dependencies] mtg-engine = { path = ".", features =
["test-util"] }`) — copied verbatim to `crates/simulator`, resolver-2-scoped so it only
activates for `cargo test -p mtg-simulator` builds, never the normal library/fuzzer-binary
build. This is a build-config-only change (zero engine behavior, zero wire surface); it
was not enumerated in the plan (which assumed simulator tests would build payment
scenarios through the full cast/resolve chain) and is reported here per §4.7 rather than
applied silently.

### Fail-before / pass-after verification — actual observed pre-fix behaviour

Both `crates/engine/src/rules/combat.rs` and `crates/engine/src/rules/engine.rs` were
temporarily reverted to their pre-PB-DP4 committed state (`git show 5c463339~1:<path>`),
the full `pb_dp4_attack_tax_and_payment_deadline.rs` suite was run against that pre-fix
pair, then both files were restored byte-identical (`git diff` empty after restore) and
the suite re-run to confirm 22/22 green again. Pre-fix run: **16 of 22 failed**, 6 passed.
Every failure matches its plan-predicted pre-fix behaviour exactly:

| test | plan probe | pre-fix observed |
|---|---|---|
| `test_508_1j_attack_tax_is_debited_from_the_pool` | #1 | `Ok`; `mana_pool.total() == 2` (not 0) — assertion `left: 2, right: 0` |
| `test_508_1h_attack_tax_colour_is_not_flattened_to_generic` | #2 | `Ok` (expected `Err`) — the `{W}{W}` restriction was satisfied by `{C}{C}` |
| `test_508_1j_coloured_attack_tax_paid_with_correct_colours` | #2b | `Ok`; `mana_pool.white == 2` (not 0) — declaration succeeded but nothing was spent |
| `test_106_6_restricted_mana_cannot_pay_an_attack_tax` | #3 | `Ok` (expected `Err`) — `total_with_restricted()` counted the restricted mana as affordable |
| `test_508_1h_attack_tax_sums_per_defender_and_per_attacker` | #4 | `Ok`; `mana_pool.total() == 6` (not 0) — assertion `left: 6, right: 0` |
| `test_107_4e_hybrid_attack_tax_is_rejected_not_paid_free` | #6 | `Ok` (expected `Err`) — the hybrid pip was invisible to the old field-sum, tax computed as 0 |
| `test_508_1d_must_attack_creature_is_not_forced_to_pay_an_attack_tax` | #7 | empty declaration returned `Err("...must attack each combat if able (CR 508.1d)")` — the deadlock itself |
| `test_508_1d_goaded_creature_is_not_forced_to_pay_an_attack_tax` | #8 | empty declaration returned `Err("Goaded creature ... must attack (CR 701.15b)")` — same deadlock shape |
| `test_702_30a_unanswered_echo_is_sacrificed_at_the_round_boundary` | #11 | permanent still on battlefield after the boundary pass; no sacrifice |
| `test_702_24a_unanswered_cumulative_upkeep_is_sacrificed_at_the_round_boundary` | #12 | permanent still on battlefield; no sacrifice |
| `test_702_59a_unanswered_recover_card_is_exiled_at_the_round_boundary` | #13 | card still in graveyard, not exiled |
| `test_101_4_multiple_outstanding_payments_resolve_in_apnap_order` | #16 | neither payment resolved (both `CreatureDied` and `RecoverDeclined` absent from the event stream) |
| `test_dp11_answering_a_payment_does_not_reassign_priority` | #17 (OOS-DP1-1) | `priority_holder` became `Some(PlayerId(1))` (the active player), not the pre-existing `Some(PlayerId(2))` — the exact bodge the seed describes |
| `test_119_4_cumulative_upkeep_life_cost_beyond_life_total_is_rejected` | #18 | `PayCumulativeUpkeep { pay: true }` returned `Ok` (expected `Err(InsufficientLife)`) — no affordability check existed on the `Life` arm |
| `test_702_24b_two_cumulative_upkeep_instances_both_reach_the_boundary` | (not on the mandatory list, but exercises the same mechanism) | permanent survives; both entries stranded |
| `test_dp11_boundary_sweep_does_not_deadlock_the_priority_round` | (see note below) | step had already advanced to `Draw` at the point my test asserts it should still be `Upkeep` — the plan characterized this probe as a "vacuous pre-fix guard"; as *implemented* it is a genuine fail-before probe (see next paragraph) |

**Deviation from the plan's guard/probe split, noted per instructions**: the plan's §7.2
table and §8 checklist list probe #15
(`test_dp11_boundary_sweep_does_not_deadlock_the_priority_round`) as one of "the 6
regression guards" that should pass both before and after. As implemented, this test adds
an intermediate assertion the plan's one-line probe description didn't specify — that the
step is *still* `Upkeep` immediately after the boundary-crossing pass (i.e., the sweep
re-grants priority in the same step rather than falling through to an advance) — before
doing one more round and checking the terminating advance to `Draw`. That intermediate
assertion is false pre-fix (there is no sweep, so the very first boundary-crossing pass
already advances to `Draw`), so the test as written is a genuine fail-before probe, not a
vacuous guard. This is a strictly stronger test than the plan's minimal description (it
additionally pins that the sweep doesn't prematurely advance), not a deviation in
intent.
**[Fix cycle correction, T8]**: the paragraph below originally miscounted this split as
"5 guards + 15 fail-before probes" (= 20, not 22, and inconsistent with this section's own
"16 of 22 failed, 6 passed" two paragraphs above). The correct split, re-derived from the
before/after run above (16 rows in that table = 16 fail-before probes; 22 − 16 = 6 passed
pre-fix = 6 guards): **6 guards (§7.1 #5, 9, 10; §7.2 #14, 18b, 20) + 16 fail-before
probes**. The sixth guard is `test_119_4b_cumulative_upkeep_zero_life_cost_is_always_payable`
— pre-fix, the `Life` arm had no affordability check at all, so a `total_life == 0` payment
already succeeded unconditionally; there was nothing for Change 2e to fix in that specific
case, so it is a guard (passes identically both sides), not a probe. The 6 true guards
(`test_508_1c_planeswalker_attack_is_not_taxed`,
`test_508_1d_must_attack_still_forced_when_an_untaxed_opponent_exists`,
`test_508_1d_must_attack_still_forced_when_only_an_opponent_planeswalker_is_untaxed`,
`test_702_30a_echo_paid_before_the_boundary_still_survives`,
`test_608_2g_mana_ability_during_the_payment_window_still_funds_the_payment`,
`test_119_4b_cumulative_upkeep_zero_life_cost_is_always_payable`) all passed identically
pre- and post-fix, confirmed in the same before/after run.

### Fuzzer smoke test (§7.4, record-only, not a gate)

`cargo run --release --bin mtg-fuzzer -- --games 5 --seed 1`: Wins 4, Draws 0, Errors 1,
avg turns 193.6. All violations are `[stack_consistency]` (pre-existing, OOS-DP3-9 class —
reproduces on `main` and here). No new `InvalidCommand` rejection mentioning "attack tax",
"echo", "upkeep", or "recover" appeared in the output; per the plan's own caveat,
`driver.rs`'s silent `PassPriority` fallback on a rejected command means a bot-side
regression here would be invisible anyway — the 30 unit/integration tests above are the
real gate, not this run.

### Deviations from the plan otherwise

None beyond the two negative-space items and the guard/probe accounting note above. Every
named file, line range, and CR citation in §3/§4 matched the source as read; no design
decision in §0/§2.0 was revisited.

### Remaining / deferred (unchanged from the plan, not filed by this runner)

Seeds OOS-DP4-1 through OOS-DP4-9 (§9) and the audit bookkeeping (§10) are explicitly
**not** filed/edited by this runner per the coordinator's instruction — that is close-out
bookkeeping for after the review cycle. `docs/audits/decision-point-audit.md`,
`memory/primitives/rider-seed-triage-2026-07-19.md` (OOS-RS3-4 status marker), and
`CLAUDE.md` "Current State" / "Last Updated" are all untouched by this session.

## Fix cycle complete (runner close-out)

**Summary**: applied every finding in `memory/primitives/pb-review-DP4.md`. All 5 MEDIUMs
fixed: **T1** (APNAP test inverted so it discriminates per-player-outer-loop from
kind-grouped-globally — verified by a temporary reversal experiment that made the
strengthened assertion fail, then reverted byte-identical), **T2** (OOS-DP1-1 probe now
seeds and asserts a non-empty `players_passed`, closing the vacuous half), **T3** (new
`test_dp11_all_no_op_sweep_falls_through_and_advances` pins the guard on plan risk 4's
highest-consequence failure shape), **E1** (hybrid/Phyrexian/X attack-tax rejection rescoped
to only fire when a declared attacker targets the unpayably-taxed defender; two new tests),
**E2** (`engine.rs`'s two new bare `.players.get(` sites converted to `state.player(..)?`
and `state.expect_player(..)`; `bare_lookup_ratchet.rs` ceiling restored to 22).

**Accuracy note on the LOW count itself**: the review's verdict banner says "13 LOW", but
the Engine Change Findings + Test Findings tables actually list **17** LOW rows (E3–E13 = 11,
T4–T9 = 6). This runner dispositioned all 17, not just 13 — the discrepancy is in the
review's own header count (the same class of self-inconsistency T8 caught elsewhere in this
file), left for the coordinator to correct in the review file's banner if desired; this
runner did not edit that banner line since the instruction was to work the findings, not
audit the review's own arithmetic.

Of the 17 LOWs: **9 fixed cleanly** (E3, E4, E5, E6, E7, T5, T7, T8, T9), **1 fixed AND
folded into a new seed** (E9 — the wording is corrected everywhere, and the postponability
consequence it describes is also filed as a seed since it is a real, undischarged behavior,
not just a documentation gap), **6 declined with a stated reason** (E8, E11, T4, T6, plus
E10 and E12 which are declined-and-folded into new seeds), and **1 needs no fix** (E13 — the
review's own verdict already calls it "a judgement note, not a defect" with "Fix: none
required"). **3 new seeds drafted** for the coordinator to file against
`docs/audits/decision-point-audit.md` §8.1 (not filed directly by this runner, per
instruction): **OOS-DP4-10** (folds E10, `ActiveRestriction.controller` staleness),
**OOS-DP4-11** (folds E12, forced-decline `ChooseReplacement` dead-end), **OOS-DP4-12**
(folds E9's underlying consequence — the DP-11 deadline can be postponed indefinitely by
keeping the stack non-empty). Full per-finding disposition table and seed text:
`memory/primitives/pb-review-DP4.md` § "Fix cycle (runner)".

**Files touched in the fix cycle** (beyond the implement-phase diff):
- `crates/engine/src/rules/combat.rs` — E1/E7 restructure of the attack-tax block (scoped
  hybrid/Phyrexian/X rejection, `{0}`-cost skip), E5 (error message), E6 (event-push
  placement).
- `crates/engine/src/rules/engine.rs` — E2 (two bare-lookup conversions), E3 (Change 2c
  comment), E4 (Change 2d comments, echo + CU), E9 (deadline-boundary wording, 2 sites).
- `crates/engine/src/rules/resolution.rs` — E9 (3 producer comment blocks), T9 (2 "pause"
  reworded to "queue").
- `crates/engine/src/state/mod.rs` — E9 (3 `pending_*_payments` field doc comments).
- `crates/simulator/src/legal_actions.rs` — T7 (CU life-cost provider short-circuit) + 1
  new test.
- `crates/engine/tests/core/bare_lookup_ratchet.rs` — E2 (ceiling restored to 22, changelog
  comment rewritten).
- `crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs` — T1 (inverted
  APNAP test), T2 (non-vacuous pass-set assertion), T3 (new all-no-op probe), E1 (2 new
  tests), T5 (strengthened assertion), plus the `test_106_6_...` message-format update
  required by the E5 fix.
- `memory/primitives/pb-review-DP4.md` — new "## Fix cycle (runner)" section (this file's
  companion).
- `memory/primitive-wip.md` — T8 (guard/probe accounting correction in the implement-phase
  close-out section) + this section.

**Test count**: 3,747 (parent-branch `main` pin) → 3,777 (implement-phase close) → **3,781**
(fix cycle: +4 new tests — T3's fall-through probe, E1's 2 scoping tests, T7's provider
parity test; T1/T2/T5 strengthened existing tests without adding new ones). `cargo test
--all`: **3,781 passed, 0 failed.** `cargo clippy --workspace --all-targets -- -D warnings`:
clean. `cargo build --workspace`: clean. `cargo fmt --check`: clean. `tools/check-defs-fmt.sh`:
clean (1,804 defs, no new TODOs in any of the 8 cards this PB's plan named as affected).

**Wire check (re-confirmed post-fix-cycle)**: `PROTOCOL_VERSION == 27`
(`crates/engine/src/rules/protocol.rs:260`), `HASH_SCHEMA_VERSION == 63`
(`crates/engine/src/state/hash.rs:578`) — both read directly from source, unmoved. No new
`Command` / `GameEvent` / `GameState` variant or field introduced in the fix cycle. SR-3
seal on `state/mod.rs` not widened (only doc comments changed there). No
`KeywordAbility::Echo` / `::CumulativeUpkeep` / `::Recover` named in executable code in the
sweep or `crates/simulator/src` (unchanged from implement phase — no new dispatch logic was
added in the fix cycle that could have reintroduced this).

**Deviations from the review's prescriptions**: none. Every MEDIUM fix matches its
"Fix:" directive; every LOW is either fixed as directed or declined with a reason recorded
in the review file's per-finding table (not silently dropped). The one piece of judgement
exercised: E5's fix statement said "state the total and the shortfall" — the runner
interpreted "shortfall" as "the two comparison quantities" (required cost, available
unrestricted mana) rather than a literal numeric difference, because a literal difference
would be misleading in the colour-mismatch case (required total mana value can equal
available total while still being unpayable due to colour) — this is explained in the
fix's own code comment and the review's E5 row.

**Not filed by this runner** (coordinator close-out, per standing instruction): the 3 new
seed drafts above, the audit rows this PB's own §10 checklist names, and any `CLAUDE.md`
/ `docs/audits/decision-point-audit.md` edits.
