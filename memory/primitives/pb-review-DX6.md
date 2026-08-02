# Primitive Batch Review: PB-DX6 — The Last Two Unflattened Mana-Cost Payment Sites

<!-- last_updated: 2026-08-02 -->

**Date**: 2026-08-02
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-172` · **Branch**: `feat/pb-dx6-the-last-two-unflattened-mana-cost-payment-sites-oos-` (HEAD `dd4a9237`)
**Plan**: `memory/primitives/pb-plan-DX6.md`
**Seeds**: OOS-RS2-1 (turn-face-up), OOS-DP4-1 (attack tax)
**CR Rules verified via MCP**: 107.4e, 107.4f, 118.5, 119.4, 119.4a, 119.4b, 508.1 (a–m),
701.40 (a–h), plus the nine Norn's Annex rulings (2011-06-01)

---

## ⚠ Verification method — read this first

**This review session had no shell/Bash tool available.** I could not run `cargo test`,
`cargo build`, `git diff`, or the revert-and-restore probe-discrimination procedure the task
brief asked for. **Every finding below was established BY READING**, and each finding says so
explicitly. Where a claim could only be settled by execution I say that plainly rather than
asserting a result — which is the same discipline plan §2 imposes on the implementer.

Specifically **not** verified in this pass:
- that the suite is 4,098 / 0 (the batch's claim);
- that clippy / `cargo fmt --check` / `tools/check-defs-fmt.sh` are clean;
- that the 210 golden scripts are green after the new turn-face-up `ManaCostPaid` event
  (plan §13 risk 7 predicts repairs; I could not observe whether any were needed);
- probe discrimination by revert-and-rerun for T1/T3/T10/T11 — analysed by reading only
  (see §"Probe discrimination" below);
- benches;
- `cargo test -p mtg-card-types --release` (plan §6.4 item 3).

A fix cycle that has a shell should re-run those and confirm.

**Engine files reviewed**: `crates/engine/src/rules/combat.rs`,
`crates/engine/src/rules/engine.rs` (`handle_turn_face_up`, `multiply_mana_cost`),
`crates/engine/src/rules/command.rs`, `crates/engine/src/rules/queries.rs`,
`crates/engine/src/rules/protocol.rs`, `crates/engine/src/state/hash.rs`,
`crates/card-types/src/state/player.rs`, `crates/engine/src/testing/replay_harness.rs`,
`crates/engine/src/testing/script_schema.rs`, `crates/simulator/src/{params,legal_actions,random_bot}.rs`,
`tools/tui/src/play/input.rs`, `tools/play-server/src/api.rs`.

**Test files reviewed**: `crates/engine/tests/primitives/pb_dx6_unflattened_payment_sites.rs`
(1,847 lines, all of it), `crates/engine/tests/core/pb_dx6_turn_face_up_and_attack_tax_roster.rs`,
`crates/engine/tests/core/protocol_schema.rs`, `crates/engine/tests/core/bare_lookup_ratchet.rs`,
`crates/engine/tests/scripts/harness_equivalence.rs`,
`crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs` (the pre-existing
tests this batch supersedes), inline `#[cfg(test)]` modules in `player.rs`, `params.rs`,
`random_bot.rs`.

**Card defs reviewed**: `kitchen_finks.rs`, `blade_historian.rs`, `boggart_ram_gang.rs`,
`deathrite_shaman.rs`, `vexing_shusher.rs` — read for `completeness` and `mana_cost` only;
**zero card defs were modified by this batch** (`rg 'PB-DX6' crates/card-defs` → no files).

---

## Verdict: needs-fix

The engine work is **CR-correct and, in the parts I could check by reading, well-built**. Both
payment sites now flatten before paying; the CR 119.4 checks are pre-mutation and correctly
short-circuited on `> 0` per CR 119.4b; the `total != ManaCost::default()` guard is evaluated on
the **pipped** total (plan §13 risk 9 avoided, and T11 discriminates it); design (A) with
copy-major replication is the shape the Norn's Annex rulings actually require and design (B)
would indeed have been rules-wrong; there is **exactly one** accumulation helper shared by the
query and the validation path; `PROTOCOL` 32→33 is append-only with a matching history row and
13 live sentinels all at 33 (including the two multi-line ones PB-DX5 warned about); `HASH`
holds at 70; the residue guard is genuinely fail-closed in `can_spend` and unconditional in
`spend`, with a byte-identical message so the existing `#[should_panic]` tests still match; and
`debug_assert_flattened`'s doc block was rewritten rather than appended to, and is now true.
Zero card defs moved, and the two deck-illegal roster members were not opportunistically
promoted — both pre-commitments held.

**But the design's own declared weakest joint is not actually guarded, and two source doc blocks
say it is.** Plan §13 risk 2/3 required a probe that would fail if the copy-major pip order were
permuted — specifically if a future dedup swapped `add_mana_cost` for `multiply_mana_cost`'s
pip-major shape. The shipped order-pin test (`two_defenders_two_restrictions_attack_tax_pip_order_is_copy_major`)
**cannot fail under that permutation**: in its fixture the copy-major and pip-major totals are
byte-identical, because the only defender with two restrictions has only one attacker.
`multiply_mana_cost`'s freshly-rewritten doc nonetheless asserts the swap would produce "no test
failure unless a probe pins the order (`pb_dx6_unflattened_payment_sites.rs`'s order-pin tests
do)" — a false "this is verified" claim written into engine source, which is precisely the
PB-DX5 failure mode the brief flags. That is the one HIGH.

Beyond it: four pre-existing PB-DP4 tests were left un-reconciled (one now passes on a
Debug-string artifact, two lost all their discriminating power), the batch's own seeds
OOS-DX6-1..4 exist nowhere outside the plan while a **shipped engine error message cites
OOS-DX6-1**, and three doc blocks describe a state of the world the same batch superseded.

**Totals: 1 HIGH / 8 MEDIUM / 6 LOW.**

---

## Engine Change Findings

| # | Severity | File:Symbol | Description |
|---|----------|-------------|-------------|
| 1 | **HIGH** | `tests/primitives/pb_dx6_unflattened_payment_sites.rs::two_defenders_..._is_copy_major` + `rules/engine.rs::multiply_mana_cost` (doc) | **The copy-major pip order is not pinned by any test, and two doc blocks claim it is.** The fixture's copy-major and pip-major totals are identical. **Fix:** add a defender with ≥2 distinct restrictions AND ≥2 attackers; correct both doc blocks. |
| 2 | MEDIUM | `tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs::test_107_4e_hybrid_attack_tax_is_rejected_not_paid_free` | **Survives only because `ManaCost`'s `Debug` prints the word "hybrid".** Its doc still describes the superseded unpayable-class regime. **Fix:** reconcile or invert it, as PB-DX5 did to `pb_ac3`. |
| 3 | MEDIUM | same file, `..._does_not_block_attacks_on_other_defenders` / `..._does_not_block_an_empty_declaration` | **PB-DP4's E1 CR 508.1c scoping fix lost every regression pin it had.** Both use a *hybrid* restriction, which is no longer a rejection class at all. **Fix:** switch both to `x_count: 1`. |
| 4 | MEDIUM | `docs/audits/decision-point-audit.md` §8.1 (absent) vs `rules/combat.rs` (cite shipped) | **OOS-DX6-1..4 are filed nowhere**, yet a shipped `InvalidCommand` message and a test both cite `OOS-DX6-1`. OOS-RS2-1/OOS-DP4-1 carry no closure record. **Fix:** file §11's four seeds; mark both closures. |
| 5 | MEDIUM | `rules/command.rs::Command::DeclareAttackers::hybrid_choices` (doc) | Calls `rules::queries::attack_tax_total` **"(forthcoming, a later PB-DX6 stage)"** — it shipped in this batch. **Fix:** delete "forthcoming, a later PB-DX6 stage". |
| 6 | MEDIUM | `pb_dx6_unflattened_payment_sites.rs::observation_b_...` (`#[ignore]` reason) | The stated reproduction recipe **cannot reproduce the recorded numbers on the post-fix tree**. **Fix:** state that the stage-B hunk must also be reverted, or mark the recipe historical. |
| 7 | MEDIUM | `rules/queries.rs::attack_tax_total` (doc + behaviour) | Silently omits X-taxed restrictions; returns `None` for an all-X tax, i.e. "no tax", for the one class the engine hard-rejects. **Fix:** document it; note the SR-38 consequence in `params.rs`. |
| 8 | MEDIUM | `tools/tui/src/play/input.rs` (2 sites) | The stage-E TUI gap is recorded **nowhere in the tree**. **Fix:** add an in-source note at both sites and a seed row. |
| 9 | MEDIUM | `testing/replay_harness.rs` (`hybrid_choice_names` / `phyrexian_life_payment_choices` param docs) | Still say "For `activate_ability` or `tap_for_mana`" only, while `script_schema.rs`'s twin doc was extended. **Fix:** extend both param docs. |
| 10 | LOW | `rules/combat.rs::accumulate_attack_tax_total` (doc) | "a defender excluded here contributes nothing" — *restrictions* are skipped, not defenders. **Fix:** reword. |
| 11 | LOW | `pb_dx6_unflattened_payment_sites.rs` §header | Section header reads "T8/T9"; there is no T9. **Fix:** renumber or add the missing probe. |
| 12 | LOW | `crates/simulator/src/random_bot.rs::choose_action_pays_a_hybrid_attack_tax_on_every_seed` | `if !attacked { continue; }` with no floor — vacuous if no seed attacks. **Fix:** count and assert ≥1. |
| 13 | LOW | `rules/engine.rs::handle_turn_face_up`, `rules/combat.rs` payment block | Non-empty choice vectors are silently ignored when the cost has no pips, while the `Command` doc says over-long vectors are rejected. **Fix:** narrow the doc sentence. |
| 14 | LOW | `tests/core/pb_dx6_..._roster.rs::r2_walk_is_not_vacuous` | The floor proves the abilities exist but never exercises `has_pip` on a morph cost. **Fix:** optionally assert ≥1 morph cost with a non-`None` field. |
| 15 | LOW | `crates/card-types/src/state/player.rs::can_spend_returns_false_..._in_release` (doc) | Cites a result "pasted into the PB-DX6 close-out report" that does not exist yet. **Fix:** paste it at close, or reword. |

---

## Finding Details

### Finding 1 — The copy-major pip order is not pinned, and two doc blocks say it is

**Severity**: HIGH (test-validity; per `memory/conventions.md` "Test-validity MEDIUMs are
fix-phase HIGHs", plus a false "verified" claim in engine source)
**Files**: `crates/engine/tests/primitives/pb_dx6_unflattened_payment_sites.rs::two_defenders_two_restrictions_attack_tax_pip_order_is_copy_major`;
`crates/engine/src/rules/engine.rs::multiply_mana_cost` (doc);
`crates/engine/src/rules/combat.rs::add_mana_cost` (doc, by reference)
**Plan**: §13 risk 2 ("A probe (T10) **must** pin the order explicitly … or a future refactor
will silently permute it") and §13 risk 3 ("T10 is the only thing that would catch it")
**Verified**: by reading (no shell). The reasoning below is a closed algebraic argument over the
two functions' bodies, not an inference from prose.

**Issue.** The implementation is correct: `add_mana_cost` appends `times` intact copies

```rust
for _ in 0..times {
    total.hybrid.extend(addend.hybrid.iter().cloned());
    total.phyrexian.extend(addend.phyrexian.iter().cloned());
}
```

which is copy-major, exactly as all three doc blocks describe, and `accumulate_attack_tax_total`
composes it correctly (per-restriction `times: 1` into a per-defender entry, then
`times: attacker_count` into the total).

The **test** does not discriminate it. Its fixture is:

- P2: **one** restriction `{G/W}`, **two** attackers;
- P3: **two** restrictions `{G/W}` then `{R/W}`, **one** attacker.

Evaluate both orders:

| defender | per-creature `hybrid` | `times` | copy-major result | pip-major (`multiply_mana_cost`'s `flat_map(repeat_n)`) |
|---|---|---|---|---|
| P2 | `[G/W]` | 2 | `[G/W, G/W]` | `[G/W, G/W]` |
| P3 | `[G/W, R/W]` | 1 | `[G/W, R/W]` | `[G/W, R/W]` |

Total under **both**: `[G/W, G/W, G/W, R/W]`. The two orders coincide, because copy-major and
pip-major differ **only** when a single `add_mana_cost` call has `times > 1` **and**
`addend.hybrid.len() > 1` — and no call in this fixture satisfies both. Therefore:

- the `assert_eq!(paid.hybrid, [...])` assertion is satisfied identically under either order;
- the "exact drain to zero in every field" argument the test's own doc offers as the real
  discriminator is also satisfied identically, since the pip-to-index mapping is unchanged;
- **the exact regression plan §13 risk 3 names — a dedup of `add_mana_cost` onto
  `multiply_mana_cost` — would leave this test green.**

The test *does* catch defender-order and within-copy restriction-order permutations (moving the
lone `{R/W}` off index 3 makes a `Red` choice illegal on a `{G/W}` pip, CR 107.4e), which is real
value. It simply does not catch the axis it is named after.

**The doc claims are the sharper half.** `rules/engine.rs::multiply_mana_cost`'s rewritten doc
block states:

> A "harmless" dedup onto this function would silently re-order the attack tax's pips … with no
> compile error and **no test failure unless a probe pins the order
> (`pb_dx6_unflattened_payment_sites.rs`'s order-pin tests do)**.

That parenthetical is false as shipped. It is a dated "verified: this is covered" claim written
into engine source, the class PB-DX5 shipped and its review caught (`snapshot_affected_set`'s
"verified: no Layer-≤4 divergence exists"). The test's own doc block makes a matching false claim
("what actually discriminates copy-major from any alternative interleaving is that the pool is
seeded to the EXACT sum …"), and — notably — the doc *already concedes* the coincidence two
sentences earlier ("would be `[P2-copy1, P2-copy2, P3-r1, P3-r2]` too for this particular
defender/restriction shape by coincidence of P2 having only one restriction") and then reasons
its way past it instead of changing the fixture.

**Failure scenario.** A later batch takes OOS-DP4-7 at face value and expresses `add_mana_cost`
as `multiply_mana_cost` + a field-wise add. The whole workspace stays green. From that moment,
any defender with two or more `CantAttackYouUnlessPay` restrictions attacked by two or more
creatures reinterprets every `hybrid_choices` / `phyrexian_life_payments` vector: with per-creature
pips `[{G/W}, {R/W}]` and 2 attackers, a client that correctly sent `[Green, Red, White, White]`
under copy-major now has index 1 (`Red`) applied to the *second copy of the `{G/W}` pip* → an
`InvalidCommand` naming CR 107.4e for a legal payment, or (with compatible colours) a
**different, legal-but-not-chosen** mana distribution charged — the "legal but wrong" class this
project ranks as its biggest pre-alpha risk. Zero corpus exposure today (R4 is empty), so this is
latent, but the whole point of the roster gate is that R4 will not stay empty.

**Fix (concrete).**
1. Add the discriminating case to the order-pin test — the minimum shape is **one** defender with
   **two distinct** restrictions (e.g. `{G/W}` then `{R/W}`) and **two** attackers. Canonical
   (copy-major) total is `[G/W, R/W, G/W, R/W]`; pip-major is `[G/W, G/W, R/W, R/W]`. Assert the
   `ManaCostPaid` `hybrid` vec directly, and additionally seed the pool so only the copy-major
   choice vector is affordable (e.g. choices `[Green, Red, White, White]` → pool `{G}{R}{W}{W}`;
   under pip-major the same vector demands `Red` on a `{G/W}` pip and errors).
2. Prove the new case is discriminating **by execution**: temporarily swap `add_mana_cost`'s
   replication loop for `flat_map(repeat_n)` shape, confirm the new assertion reddens and the
   existing ones stay green, restore, confirm `git diff` clean. Record the observed failure.
3. Correct `multiply_mana_cost`'s doc parenthetical and the test's doc paragraph so neither
   claims coverage that only exists after step 1. If step 1 is deferred, the docs must say the
   order is **not** currently pinned.
4. While there: the section header says "T8/T9" but only one test exists (Finding 11) — the
   discriminating case is the natural T9.

---

### Finding 2 — A pre-existing PB-DP4 test now passes on a `Debug`-string artifact

**Severity**: MEDIUM (per `memory/conventions.md`, treat as a fix-phase HIGH)
**File**: `crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs::test_107_4e_hybrid_attack_tax_is_rejected_not_paid_free`
**Verified**: by reading.

The test builds a `cost_per_creature: {2/W}` (`HybridMana::GenericColor(White)`) restriction, an
empty pool, and asserts:

```rust
msg.contains("attack tax") && (msg.contains("hybrid") || msg.contains("Phyrexian"))
```

Pre-DX6 that matched the class-rejection string. Post-DX6 the message is

```
attack tax: the attacking player cannot pay the required ManaCost { white: 1, …,
hybrid: [], phyrexian: [], x_count: 0 } for the declared attackers … 0 unrestricted mana available.
```

which contains `"hybrid"` **as a `Debug` field name**. The assertion is now satisfied
unconditionally by any message that `Debug`-prints a `ManaCost`, and the test's doc comment
("a hybrid attack tax is REJECTED, not silently paid for free. Pre-fix: the hybrid pip is
invisible to the field-sum") describes the regime this batch just replaced.

This is the sibling of the very tests PB-DX6 wrote: `historical_observation_c_...` and T3
explicitly assert `!msg.contains("is not payable")`, while this one still narrates the opposite
policy three files away. PB-DX3b reconciled golden `combat/191`, and PB-DX5 inverted
`pb_ac3_dynamic_pt_counts.rs` for exactly this reason; the same standard applies here.

**Fix**: rename/redoc it to what it now pins (an insufficient pool cannot pay a *payable* hybrid
tax), replace the `contains("hybrid")` clause with `contains("cannot pay the required")` plus
`!contains("is not payable")`, and note the PB-DX6 reconciliation with a CR cite. **Do not delete
it** — the scenario is still worth pinning.

---

### Finding 3 — PB-DP4's E1 CR 508.1c scoping fix has lost all its regression coverage

**Severity**: MEDIUM
**File**: same file, `test_107_4e_hybrid_tax_does_not_block_attacks_on_other_defenders` and
`test_107_4e_hybrid_tax_does_not_block_an_empty_declaration`
**CR**: 508.1c
**Verified**: by reading.

Both tests exist to pin PB-DP4's E1 fix — *the rejection must fire only when a declared attacker
actually targets the taxed defender, not on the mere existence of the restriction*. Both use a
**hybrid** `cost_per_creature`. After PB-DX6, a hybrid restriction never enters
`x_tax_defenders`, so the rejection loop those tests guard is never reached for their fixture:
they would pass whether the E1 scoping were present or reverted. Their `is_ok()` assertions are
now guaranteed by the narrowing, not by the scoping.

The `combat.rs` comment block that documents E1 is careful and still correct; only its tests went
hollow. T7 covers "an X tax *does* reject", but nothing now covers "an X tax on a defender no
declared attacker engages must **not** reject" or "an empty declaration is never blocked" — which
is the whole content of E1.

**Fix**: change `cost_per_creature` in both tests to `ManaCost { x_count: 1, ..Default::default() }`
(the only remaining rejection class), keep the assertions, and update the doc comments to say the
class narrowed in PB-DX6. Verify by execution that reverting the `for (_, target) in &attackers`
scoping loop to an unconditional rejection reddens both.

---

### Finding 4 — The batch's own seeds are filed nowhere, and a shipped error message cites one

**Severity**: MEDIUM
**Files**: `docs/audits/decision-point-audit.md` §8.1 (no `OOS-DX6-*` row exists — the table ends
at `OOS-M11-8`); `crates/engine/src/rules/combat.rs` (the X rejection string ships
`"…; see OOS-DX6-1."`); `pb_dx6_unflattened_payment_sites.rs` T7 asserts on that literal
**Plan**: §11
**Verified**: by reading (`rg 'OOS-DX6'` → 3 files: the plan, `combat.rs`, the test file).

Today a player or developer who reads the engine's own rejection message and searches the repo
for `OOS-DX6-1` finds the message that cites it and nothing else. That is the OOS-DP6-8
documentation-rot class the plan warns about twice, created inside the batch that warns about it.
Additionally, neither OOS-RS2-1 nor OOS-DP4-1 carries a closure annotation anywhere in
`docs/audits/decision-point-audit.md`, so the two seeds this batch exists to close still read as
open.

Note also that PB-DP4's own closing `/review` (finding 3, quoted in
`pb_dp4_attack_tax_and_payment_deadline.rs:613-617`) *removed* a literal-seed-id assertion on
the grounds that "a seed id is bookkeeping, not behaviour"; PB-DX6's T7 reintroduces exactly that
coupling. That is defensible (the plan asks for it, and it is the only cheap way to prove the
cite was swapped), but it makes filing the seed a hard prerequisite rather than a nicety.

**Fix**: file OOS-DX6-1 (X has no announcement channel), OOS-DX6-2 (`CantAttackYouUnlessPay` is
player-only ⇒ Norn's Annex not authorable as `Complete`), OOS-DX6-3 (`can_spend`/`spend` as
`Result`), OOS-DX6-4 (`DeclareAttackersData` boxing) in `docs/audits/decision-point-audit.md`
§8.1, re-disposition OOS-DP4-7 with §5.2.5's argument, and add CLOSED annotations to OOS-RS2-1
and OOS-DP4-1. Verify every cite by symbol on closure, per plan §11's own instruction.

---

### Finding 5 — `Command::DeclareAttackers`' doc calls a shipped function "forthcoming"

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/command.rs`, `Command::DeclareAttackers::hybrid_choices` doc
**Verified**: by reading.

> `rules::queries::attack_tax_total` **(forthcoming, a later PB-DX6 stage)** is the supported way
> for a client to obtain the exact cost these choices index

`attack_tax_total` is `pub` in `rules/queries.rs`, re-exported from `lib.rs:33`, and called by
`crates/simulator/src/params.rs` and T10. The doc is a stage-A artefact that stage C invalidated.
Plan §3.2 item 4 required this sentence to state the query **is** the supported way; as shipped it
tells the reader the opposite. Same class as Finding 9.

**Fix**: delete "(forthcoming, a later PB-DX6 stage)".

---

### Finding 6 — Observation B's reproduction recipe can no longer produce its recorded numbers

**Severity**: MEDIUM
**File**: `pb_dx6_unflattened_payment_sites.rs::observation_b_release_figure_pool_debit_kitchen_finks`
(the `#[ignore]` reason string and the OBSERVED block)
**Plan**: §2 step 3 ("re-observe by **reverting just that hunk**")
**Verified**: by reading.

The recorded observation itself is honest — it was taken at stage 0 against an unmodified tree,
and it is correctly labelled release-equivalent, single-run, and outside the standing suite. The
**recipe** is what has gone stale. It says:

> Reproduce by temporarily commenting out the `debug_assert_flattened(cost);` line at the top of
> `ManaPool::can_spend` … then run `cargo test … observation_b -- --ignored --nocapture`.

On the post-fix tree `handle_turn_face_up` flattens `{1}{G/W}{G/W}` *before* `can_spend` is
reached, so removing that guard changes nothing: with `hybrid_choices: vec![]` the flattener
defaults both pips to Green, the pool is `{1}{G}{W}`, green 1 < 2, and the command returns `Err`.
A reader following the recipe gets a `.expect()` panic and no numbers. The `#[ignore]` reason's
own claim that "with the guard PRESENT … this test panics rather than failing an assertion" is
true only by accident (the panic is the `expect`, not the guard).

**Fix**: state that reproducing Observation B now also requires reverting the stage-B flatten
hunk in `handle_turn_face_up` (a `git stash` of that hunk, per §2 step 3), **or** relabel the
whole test `historical_observation_b_…`, mark it permanently non-reproducible on this tree, and
keep the recorded numbers as the record they are. Prefer the second — it is honest and cheap, and
matches the `historical_observation_a/c/d2` treatment the batch already uses.

---

### Finding 7 — `attack_tax_total` silently drops X restrictions, and its doc does not say so

**Severity**: MEDIUM (latent: 0 corpus defs carry an X attack tax)
**File**: `crates/engine/src/rules/queries.rs::attack_tax_total`,
`crates/engine/src/rules/combat.rs::accumulate_attack_tax_total`
**CR**: 107.3, 508.1h; SR-38 ("never offer an action the engine rejects")
**Verified**: by reading.

`accumulate_attack_tax_total` skips any restriction with `x_count > 0`. That is correct for the
*validation* path, because `handle_declare_attackers` rejects the declaration before the total is
used. It is **not** correct for the *advisory* path:

- `attack_tax_total`'s doc block — the one a client is meant to read — never mentions X at all.
- For a defender whose only restriction is an X tax, the query returns **`None`**, whose documented
  meaning is "no tax applies to this declaration". The engine will hard-reject that declaration.
- `crates/simulator/src/params.rs`'s `DeclareAttackers` arm does
  `attack_tax_total(...).and_then(resolve_hybrid_phyrexian_plan).unwrap_or_default()`, so an X tax
  produces empty vectors and a command the engine refuses — an SR-38 violation, latent only
  because R4 is empty.

**Fix**: (a) add a paragraph to `attack_tax_total`'s doc stating that X-carrying restrictions are
excluded from the returned total, that a declaration engaging such a defender will be rejected
regardless of what this function returns, and that `None` therefore does not mean "this
declaration is free"; cite OOS-DX6-1. (b) Add a one-line note at the `params.rs` call site
recording the SR-38 residue. Do **not** widen the signature in this batch — that is OOS-DX6-1's
job.

---

### Finding 8 — The stage-E TUI gap is recorded nowhere

**Severity**: MEDIUM
**File**: `tools/tui/src/play/input.rs` (two `Command::DeclareAttackers` literals, ~`:616` and
`:632`)
**Verified**: by reading (`rg 'PB-DX6' tools/` → no matches).

Both TUI sites hand-build `Command::DeclareAttackers` with `hybrid_choices: vec![]` /
`phyrexian_life_payments: vec![]` rather than routing through
`params.rs::action_to_command_with_params`, which is where the batch put the single CR 508.1h
plan-building site. `crates/simulator/src/random_bot.rs` was explicitly migrated away from exactly
this shape ("which used to hand-construct `Command` with hard-coded empty payment vectors"); the
TUI was not.

**Is recording rather than fixing the right call? Yes.** Corpus exposure is exactly zero (R4 is
pinned empty), the TUI is not on the M11-local web-first track, and routing the TUI through
`action_to_command_with_params` is a behavioural refactor of a tool outside the batch's declared
scope — `memory/conventions.md`'s "implement-phase default-to-defer" applies. **But it is not
recorded anywhere**, which is the half that fails: nothing in the tree tells the next reader that
those two literals are a known, deliberate, latent gap rather than a missed migration.

**Fix**: add a two-line comment at both sites naming PB-DX6, the CR 508.1h total, and
`params.rs::action_to_command_with_params` as the correct route; and file it as a seed row (fold
into OOS-DX6-1's row or give it its own).

---

### Finding 9 — `replay_harness.rs`'s parameter docs were not extended

**Severity**: MEDIUM (doc-vs-code; low blast radius)
**File**: `crates/engine/src/testing/replay_harness.rs`, the `hybrid_choice_names` and
`phyrexian_life_payment_choices` parameter doc comments
**Verified**: by reading.

Both still read "For `activate_ability` or `tap_for_mana` on a source with a hybrid pip in its
activation cost … Empty for non-hybrid costs **or all other action types**." Since this batch,
`"turn_face_up"` and `"declare_attackers"` both consume them, and the *sibling* doc in
`script_schema.rs` was correctly extended ("extended by PB-DX6 §9.3 to `turn_face_up` and
`declare_attackers`"). The two now disagree, and the harness one says "all other action types"
ignore a value the harness in fact forwards.

**Fix**: mirror `script_schema.rs`'s wording into both `replay_harness.rs` parameter docs.

---

### Findings 10–15 (LOW)

**10 — `accumulate_attack_tax_total` doc, X-skip wording.** The doc says "a defender excluded
here contributes nothing to the total regardless of how many creatures attack it." No *defender*
is excluded; individual *restrictions* are. A defender carrying one X restriction and one plain
`{2}` restriction still contributes its `{2}` to the total. Unreachable through
`handle_declare_attackers` (which rejects first) but reachable through the public query.
**Fix**: reword to "restrictions … are skipped; a defender whose *only* restriction carries an X
therefore contributes nothing."

**11 — phantom T9.** The section header reads `── T8/T9 — the order pin …` and only one `#[test]`
follows. Plan §2.1 lists "T8–T11". **Fix**: renumber, or make the Finding-1 discriminating case
T9 (preferred).

**12 — `random_bot` sweep has no non-vacuity floor.** `choose_action_pays_a_hybrid_attack_tax_on_every_seed`
does `if !attacked { continue; }`. If the 80 % branch and the random-subset fallback both declined
on all 20 seeds the test would pass having asserted nothing. Vanishingly unlikely, but this suite's
own standard for pinned-empty/skip-guarded assertions is a floor. **Fix**: count the seeds that
attacked and `assert!(attacked_count >= 1)`.

**13 — choice vectors silently ignored on a pip-free cost.** Both new sites gate the flatten on
`!cost.hybrid.is_empty() || !cost.phyrexian.is_empty()`, so a client that sends
`hybrid_choices: [Green]` against a `{2}` cost has it discarded without error. This exactly mirrors
`abilities.rs::handle_activate_ability` (the plan's declared reference shape), so it is not a new
defect — but both new `Command` doc blocks say "a vector LONGER than the pip count is rejected
with `InvalidCommand` rather than silently ignored", which is untrue for the zero-pip case.
**Fix**: add "(when the cost carries at least one pip; a cost with no pips ignores these fields
entirely, matching `ActivateAbility`)".

**14 — R2's non-vacuity floor.** `r2_walk_is_not_vacuous` proves ≥1 `Morph`/`Megamorph`/`Disguise`
ability exists, which is what plan §8 asked for. It does not prove `roster_r2`'s cost extraction
or `has_pip` work on that path; a broken extraction would leave R2 green and its floor green.
`has_pip` itself is exercised by R1's five members, so the residual risk is only in the
match-arm/`cost` binding — small. **Fix (optional)**: also assert the walk saw ≥1 morph-family
cost object at all.

**15 — an as-yet-unpaid documentary debt.** `can_spend_returns_false_for_unflattened_residue_in_release`'s
doc says its claim "rests on a single recorded manual run of `cargo test -p mtg-card-types
--release`, pasted into the PB-DX6 close-out report". No close-out report exists in the tree
(`memory/primitives/` contains only `pb-plan-DX6.md`). This is an obligation, not yet a false
claim — but it becomes one the moment the batch closes without the paste. **Fix**: run it and
paste the observed result into the close-out, per plan §12.

---

## Answers to the ten specific questions in the brief

**1. CR correctness of both payment sites.** Verified against MCP text for CR 107.4e, 107.4f,
119.4/119.4a/119.4b, 508.1a–m, 701.40a–h, and all nine Norn's Annex rulings. All the plan's
verbatim quotes are accurate. Both sites are correct as written:
- `handle_turn_face_up` flattens unconditionally when the cost carries a pip, **before** the
  `flat_cost.mana_value() > 0` gate; the CR 119.4 check is pre-mutation, written through a
  `combined_life_cost` local as the plan required; the life deduction is a **sibling** of the mana
  gate, not nested; `ManaCostPaid` carries the **original pipped** cost. All three
  `TurnFaceUpMethod` arms funnel through the one block, satisfying CR 701.40c/701.40d.
- `handle_declare_attackers` accumulates the CR 508.1h total, flattens **once** against the
  accumulated total (design A), checks CR 119.4 pre-mutation, checks affordability on the
  flattened cost, and pays after the tapping loop with both events inside `if let Some(ps)`
  (PB-DP4's E6 discipline preserved).
- Design (B) really is rules-wrong: the 2011-06-01 rulings say "that player chooses how to pay
  each cost **individually**" both across creatures and across duplicate restrictions, and
  flatten-then-multiply cannot express either. The plan's argument holds.
- CR 508.1i remains unhonoured (pre-existing OOS-DP4-2), correctly documented and not overclaimed.
- One CR nuance worth stating: CR 508.1h locks the total in at determination. The engine
  determines and pays inside one command, so lock-in is trivially satisfied; nothing here
  regresses it.

**2. Copy-major.** *Is it actually copy-major in `accumulate_attack_tax_total`?* **Yes** — verified
by reading the replication loop and the two-stage composition. *Does it match what the three doc
blocks claim?* **Yes** — `Command::DeclareAttackers`' doc, `add_mana_cost`'s doc and
`accumulate_attack_tax_total`'s doc all state the same order, and it is the order the code
produces. *Is the order genuinely pinned by a test that would fail if permuted?* **Partially, and
not on the axis that matters** — see Finding 1. Defender-order and within-copy restriction-order
permutations are caught; copy-major↔pip-major is not.

**3. Risk 9 and risk 10.** Both correct.
- Risk 9: `combat.rs` reads `let (flat_total, phyrexian_life) = if total != ManaCost::default()`
  — the guard is on the **pipped** `total`, exactly as required, with an in-source comment saying
  so and citing T11. The later payment block gates on `if let Some(tax) = &attack_tax`, which is
  `Some` iff the pipped total is non-default. An all-Phyrexian, all-life tax therefore still
  reaches the payment block with `flat_total == {0}`. T11 discriminates this by reading (if the
  guard were on the flattened total, `attack_tax` would be `None`, no life would be deducted, and
  `assert_eq!(ps.life_total, 18)` would fail against 20).
- Risk 10: both sites guard with `> 0` — `if life > 0` in `combat.rs`, `if combined_life_cost > 0`
  in `engine.rs` — so `phyrexian_life == 0` never reaches the CR 119.4 check. CR 119.4b satisfied.

**4. Exactly one shared accumulation.** **Confirmed by reading.**
`combat::accumulate_attack_tax_total` is the sole definition; `queries::attack_tax_total`
delegates to it (`let total = combat::accumulate_attack_tax_total(state, attackers);`) and only
adds the `ManaCost::default() → None` boundary; `handle_declare_attackers` calls the same
function. `rg 'accumulate_attack_tax_total'` returns hits in exactly two files (`combat.rs`,
`queries.rs`). No second copy of the pip order exists. T10 pins the equality by execution.

**5. Observed-not-reasoned discipline.** Largely honoured, and the build-mode trap in plan §2.0
was **not** fallen into — this is the batch's best piece of process work:
- Observation A records the debug panic verbatim, labelled with build mode, date, and capture
  method (`catch_unwind` downcast).
- Observation B is explicitly labelled release-equivalent, single-run, manual, `#[ignore]`d, and
  "NOT reproduced by the standing suite". The recorded numbers are internally consistent with
  Kitchen Finks' actual encoding (`generic: 1`, two `ColorColor(Green, White)`), which I verified
  against MCP oracle text `{1}{G/W}{G/W}` and against `kitchen_finks.rs`, and with `spend`'s
  documented generic order (colourless first). **Could the fixture have produced the number it
  reports?** Yes — pool `{colorless 1, green 1, white 1}`, raw cost `{generic 1}` → colourless 0,
  green 1, white 1. The claim is producible and consistent. Its *recipe* is now stale (Finding 6).
- Observations C and D(ii) quote the pre-fix `InvalidCommand` strings and explicitly say
  "preserved VERBATIM, not re-executed", which is the right label.
- T6's message is labelled "OBSERVED verbatim … both before and after PB-DX6 stage C" and is
  self-consistent (`generic: 4` from 2 attackers × `{2}`, `1 unrestricted mana available`).
- **No manufactured figure found.** The one un-evidenced "verified" claim is the *coverage* claim
  in `multiply_mana_cost`'s doc (Finding 1), which is exactly the PB-DX5 class the brief warned
  about — a doc comment asserting that a test pins something, where the test does not.

**6. Probe discrimination.** **Not verified by execution — no shell.** By reading:
- **T1** — cases 2/3/5 assert `pool.total() == 0`. Pre-fix, `can_spend`/`spend` saw the raw
  `{generic: 1}`, so 2 green would survive; and in debug the guard panics first. Discriminating
  either way.
- **T3** — case 1 expects `Ok`; pre-fix the declaration was rejected as an unpayable class, so
  `.expect()` panics. Discriminating.
- **T10** — discriminates only *drift between two accumulations*, which is what it claims. It
  cannot discriminate an order change inside the single shared implementation (both sides move
  together). Correct as scoped; do not mistake it for an order pin.
- **T11** — discriminates both the pre-fix class rejection and the risk-9 flattened-guard bug (see
  Q3).
- **T8** — discriminates defender-order and within-copy restriction-order, **not** copy/pip-major
  (Finding 1).
A fix cycle with a shell should confirm at minimum T1 case 2, T3 case 1, T11, and the new
Finding-1 case by revert-and-rerun.

**7. Vacuous assertions.**
- **T7 is non-vacuous** and correctly built. `is_err()` alone would indeed be vacuous (the pre-fix
  code also errored), and T7 instead asserts on text: `contains("OOS-DX6-1")`,
  `!contains("OOS-DP4-1")`, `!contains("hybrid") && !contains("Phyrexian")`, plus X naming. The
  two seed-id clauses and the two negative clauses are all genuinely discriminating against the
  pre-fix message. The `contains('X')` clause on its own would not be (the pre-fix message also
  contained "X"), but it is not load-bearing.
- **R2 and R4** are pinned empty and both carry the mandated non-vacuity floors
  (`r2_walk_is_not_vacuous`: ≥1 morph-family ability; `r4_walk_is_not_vacuous`: R3 non-empty).
  R1 has a floor too (≥1 def with a non-`None` `mana_cost`). Floors present as specified; one is
  slightly weaker than ideal (Finding 14).
- **Two vacuous assertions were found, both in pre-existing tests the batch did not reconcile** —
  Findings 2 and 3.

**8. Doc-vs-code honesty.**
- `debug_assert_flattened`'s doc: **rewritten, not appended to, and now true.** The SR-4/SR-6
  paragraphs are preserved verbatim (still correct); the false "fires NEVER … do not treat this
  guard as load-bearing" paragraph is replaced by an accurate question-vs-instruction account of
  the two callers' asymmetry. The message string in `spend`'s new `assert!` is byte-identical to
  the `debug_assert!`'s, so the existing `#[should_panic(expected = …)]` tests still match; the
  module gate moved to plain `#[cfg(test)]` with the two `can_spend` panic tests individually
  re-gated `#[cfg(debug_assertions)]`, exactly as §6.4 specified. **Clean.**
- `add_mana_cost`'s doc: **rewritten, not appended to.** The assert is correctly narrowed to
  `addend.x_count == 0` and the doc's account of the mechanism is accurate. Its only defect is
  by reference to `multiply_mana_cost`'s doc (Finding 1).
- `multiply_mana_cost`'s doc: rewritten and correct **except** the false "no test failure unless a
  probe pins the order (… `pb_dx6`'s order-pin tests do)" (Finding 1).
- **Three further stale doc blocks**: Findings 5, 9, 10. Plus Finding 6's stale recipe.
So: the two docs the plan singled out were done properly; the batch nonetheless shipped four
newly-stale statements elsewhere.

**9. Scope discipline.** **0 completeness flips confirmed by reading.** `rg 'PB-DX6' crates/card-defs`
returns no files. `deathrite_shaman.rs` is still `Completeness::known_wrong(...)` and
`vexing_shusher.rs` still `Completeness::partial(...)` — **not** opportunistically promoted, as
plan §10 required. `kitchen_finks.rs` and `boggart_ram_gang.rs` carry explicit
`Completeness::Complete`; `blade_historian.rs` declares no `completeness` field at all and is
`Complete` by the `#[default]` derive, which the roster gate's own doc comment records honestly
(plan §0's instruction: note it, do not "fix" it). Since no marker moved, `random_deck`'s
`Complete` pool is unchanged and no seeded deck re-deals — plan §10's stated consequence holds.
*(Caveat: this is a read-level check of the five roster defs plus a `PB-DX6` grep, not a
`git diff --stat -- crates/card-defs`. A fix cycle with a shell should confirm the diff is
empty.)*

**10. The stage-E TUI gap.** Recording rather than fixing is **the right call** (zero corpus
exposure, out of declared scope, `default-to-defer` applies) — but it is **not recorded**
anywhere in the tree, which is half the obligation. See Finding 8.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|--------------|---------|-------|
| 107.4e (hybrid payable either half) | Yes, both sites | Yes | T1 c2/c3/c4/c5, T2, T3 c1/c2/c3, T8 |
| 107.4f (Phyrexian: mana or 2 life) | Yes, both sites | Yes | T4 c1–c5, T5 c1–c4, T11 |
| 107.3 (X must be announced) | Rejection only | Yes | T7 + historical D(ii); OOS-DX6-1 |
| 118.5 (`{0}` cost) | Yes (E7 skip preserved) | Indirect | free-restriction skip untouched |
| 119.4 (life ≥ payment, pre-mutation) | Yes, both sites | Yes | T4 c3 (`actual: 1` proves pre-mutation), T5 c3 |
| 119.4b (0 life always payable) | Yes (`> 0` guards) | Indirect | pip-free paths never enter the guard; no dedicated probe |
| 202.3g (Phyrexian mana value 1) | Relied on | Yes | T4 c5 (raw mv 1 vs flattened 0) |
| 400.7 | n/a | — | no zone change in scope |
| 508.1c (restrictions ⇒ illegal declaration) | Yes, scoped to declared attackers | **Weakened** | T7 covers "X rejects"; the *scoping* half lost its pins — Finding 3 |
| 508.1d (never forced to pay) | Unchanged | Pre-existing | `taxed_defenders` union preserved |
| 508.1g | Unchanged | Pre-existing | enlist/exert |
| 508.1h (total; locked in) | Yes | Yes | T3, T5 c4, T6, T8, T10, T11 |
| 508.1i (mana window) | **No** (pre-existing) | n/a | OOS-DP4-2, correctly documented |
| 508.1j (no partial payments) | Yes | Yes | T6 (unchanged rejection) |
| 601.2b (analogue, X announcement) | Rejection only | Yes | T7 |
| 701.40b (manifest: "pay that cost") | Yes | Yes | T1, T2, historical A |
| 701.40c/d (morph/disguise manifested) | Yes (shared block) | **No direct probe** | R2 pinned empty; latent, correctly labelled |
| 702.37e / 702.168d | Yes (shared block) | **No direct probe** | same — latent |
| 704.5a (0 life ⇒ SBA loss) | Pre-existing | Yes | T4 c4 |
| Norn's Annex rulings (per-cost individual choice) | Yes (design A, copy-major) | **Partially** | T5 c4 covers per-creature; per-restriction ordering is Finding 1 |

---

## Card Def Summary

| Card | Oracle Match (MCP) | Modified? | Completeness moved? | Notes |
|------|--------------------|-----------|---------------------|-------|
| Kitchen Finks | `{1}{G/W}{G/W}` ✓ | No | No (`Complete`, explicit) | T1's fixture; def encoding matches MCP |
| Blade Historian | `{R/W}{R/W}{R/W}{R/W}` ✓ | No | No (`Complete` **by `#[default]`**) | T2 asserts 4 pips / generic 0 — matches MCP exactly; recorded honestly in the roster gate doc |
| Boggart Ram-Gang | `{R/G}{R/G}{R/G}` (T2 fixture) | No | No (`Complete`, explicit) | not independently MCP-checked this pass; T2's pool `{R}{R}{R}` is consistent |
| Deathrite Shaman | — | No | No (`known_wrong`) | correctly **not** promoted (plan §10) |
| Vexing Shusher | — | No | No (`partial`) | correctly **not** promoted (plan §10) |
| Propaganda / Ghostly Prison | `{2}` tax | No | No | R3 pinned set of 2; R4 (pip/X) pinned empty |
| Norn's Annex | `{3}{W/P}{W/P}`, "you **or planeswalkers you control**" | **Not authored** ✓ | n/a | correctly left out; the planeswalker half is a real gap (`CantAttackYouUnlessPay` is player-only) → OOS-DX6-2, **which is not filed** (Finding 4) |

---

## Machine-enforced gates

| Gate | State | Verified how |
|------|-------|--------------|
| SR-8 `PROTOCOL_VERSION` | 32 → **33**; History doc row appended (never edited); `PROTOCOL_HISTORY` row 33 appended; `PROTOCOL_SCHEMA_FINGERPRINT` = `a153b665…97f6` matching the row; `protocol_version_sentinel` at 33; `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned with a dated note in the existing style | reading |
| Sentinel re-pin by symbol | 13 live `assert_eq!(PROTOCOL_VERSION, 33)` sites; `rg 'PROTOCOL_VERSION,\s*3[0-9]'` multiline finds **no straggler at 32**, including the two multi-line forms (`pb_dp5`, `pb_dx2`) PB-DX5's lesson was about | reading (multiline grep) |
| `HASH_SCHEMA_VERSION` | **unmoved at 70** (`hash.rs:712`), as predicted; no `GameState` field added | reading |
| T12 batch sentinel | present, asserts 33 / 70, doc states which gate produced each | reading |
| SR-9a `mod` registration | `tests/core/main.rs:31` and `tests/primitives/main.rs:36` both present | reading |
| SR-36 roster from `all_cards()` | R1–R4 all walk `mtg_engine::all_cards()`; no source greps | reading |
| `bare_lookup_ratchet` | `combat.rs` 15 → **16** (raise, justified in-file with the shared-helper argument); `engine.rs` 22 → **21** (a *reduction*, locked in) | reading |
| SR-31 `CROSS_VALIDATED_SHAPES` | unchanged; the absence of `turn_face_up:hybrid` / `declare_attackers:hybrid` is recorded with a checked reason (`PermanentInitState` genuinely has no face-down field — I confirmed `rg 'face_down' script_schema.rs` returns nothing) | reading |
| Wire-neutrality of the new event | `GameEvent::ManaCostPaid` is an existing variant; no wire change | reading |

---

## Deferred — belongs in a later batch, with seed text

Do **not** take these here.

- **OOS-DX6-1** (as planned) — `Command::DeclareAttackers` cannot announce **X** in an attack tax
  (CR 107.3 / CR 601.2b analogue). Needs an x-announcement channel, not a payment-choice vector.
  Carries the rejection message's citation. **Widen it with Finding 7's residue**:
  `rules::queries::attack_tax_total` returns `None` (documented as "no tax") for an all-X tax and
  omits X restrictions from a mixed total, so `crates/simulator/src/params.rs` builds a command
  the engine refuses — an SR-38 violation, latent only while R4 is empty.
- **OOS-DX6-2** (as planned) — `GameRestriction::CantAttackYouUnlessPay` is **player-only**;
  `combat.rs` scopes it to `AttackTarget::Player` on the Propaganda ruling. Norn's Annex's printed
  "or planeswalkers you control" is therefore inexpressible, so the card is `partial` at best even
  after PB-DX6 and must not be authored to inflate yield.
- **OOS-DX6-3** (as planned) — `ManaPool::can_spend`/`spend` as `Result`-returning (plan §6.2
  option 3). Records the argument against doing it inside a correctness batch (it launders an
  engine bug into a rules answer) and for revisiting it standalone if `ManaPool` grows other
  precondition failures.
- **OOS-DX6-4** (as planned) — boxing `Command::DeclareAttackers` into a `DeclareAttackersData`
  struct with `Default` (SR-10 treatment), deferred so a 320-site semantic refactor does not ride
  along with a correctness fix and make the digest delta un-attributable.
- **OOS-DX6-5 (new, from Finding 8)** — `tools/tui/src/play/input.rs` hand-builds
  `Command::DeclareAttackers` at two sites with empty payment vectors instead of routing through
  `params.rs::action_to_command_with_params`, the single CR 508.1h plan-building site every other
  caller uses (`random_bot.rs` was migrated off exactly this shape by PB-DX6 stage E). Latent —
  0 corpus defs carry a pipped attack tax (PB-DX6 R4 pinned empty) — so a TUI player can never
  currently owe one. Becomes live the moment a card with a hybrid or Phyrexian attack tax is
  authored, at which point the TUI silently sends an empty plan and the engine rejects the
  declaration with no way for the player to answer. Fix is mechanical: route both sites through
  `action_to_command_with_params` with `ActionParams { attackers, .. }`, as `random_bot.rs` now
  does. Filed rather than taken because the TUI is outside PB-DX6's declared scope and
  `memory/conventions.md`'s implement-phase default-to-defer applies.
- **OOS-DP4-7 — re-disposition, do not close** (as planned): plan §5.2.5's copy-major/pip-major
  divergence is a new and stronger reason **not** to dedup `add_mana_cost` onto
  `multiply_mana_cost`. **Add Finding 1's correction to the row**: the claim that a probe would
  catch the dedup is, as shipped, false — record that the row's safety depends on the
  discriminating case being added first.

---

## Fix-cycle checklist (in priority order)

1. **[HIGH]** Add the discriminating copy-major case (one defender, two distinct restrictions,
   two attackers), prove it by revert-and-rerun, and correct `multiply_mana_cost`'s and the
   test's doc claims. (Finding 1)
2. **[MEDIUM]** Reconcile the three pre-existing PB-DP4 attack-tax tests: fix the Debug-artifact
   assertion, and move the two E1 scoping pins to `x_count: 1`. (Findings 2, 3)
3. **[MEDIUM]** File OOS-DX6-1..5 in `docs/audits/decision-point-audit.md` §8.1, re-disposition
   OOS-DP4-7, and mark OOS-RS2-1 / OOS-DP4-1 CLOSED. Verify every cite by symbol. (Finding 4)
4. **[MEDIUM]** Fix the four stale doc statements: `command.rs` "forthcoming" (5), Observation B's
   recipe (6), `attack_tax_total`'s X omission (7), `replay_harness.rs` param docs (9).
5. **[MEDIUM]** Record the TUI gap in-source at both sites. (Finding 8)
6. **[LOW]** Findings 10–15.
7. **Re-run everything this review could not**: full workspace tests, clippy,
   `cargo fmt --check` + `tools/check-defs-fmt.sh`, the 210 golden scripts (plan §13 risk 7),
   `cargo test -p mtg-card-types --release` (§6.4 item 3, paste the result),
   `git diff --stat -- crates/card-defs` (confirm empty), benches, and probe discrimination for
   T1/T3/T10/T11 plus the new Finding-1 case.
