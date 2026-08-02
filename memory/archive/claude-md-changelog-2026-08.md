# CLAUDE.md changelog archive — 2026-08

> Detail moved out of `CLAUDE.md`'s "Current State" so that section stays a true
> snapshot, per the Recurrence rule in CLAUDE.md's "Changelog & history". Newest first.
> Older entries: `memory/archive/claude-md-changelog-2026-07.md`.

## Rotated "Last Updated" entries (moved from CLAUDE.md Current State, 2026-08-02)

- 2026-08-02 — **UI-1 SHIPPED (`scutemob-174`): the browser client can answer a blocking decision
  instead of echoing the engine's default.** Playtest-triage **F8**, whose three symptoms ("the
  game discards for me, always the cards on the right", "it never asks me to scry", "the tutor
  always fetches the same card") are **one mechanism at three layers**. `StubProvider` bakes the
  engine-accepted default *into* each blocking-decision `LegalAction` — cleanup discard = the
  `count` highest `ObjectId`s, scry/surveil = the identity partition, search = `candidates.first()`
  — and it is right to: that is what lets a **bot** submit it and always be accepted (SR-38, never
  offer an action the engine will refuse). The candidate data rides along on the same action
  precisely so a **human** client can render a picker (`legal_actions.rs`'s own doc says so). The
  view layer threw it away, `ActionParamsDto` had no override channel, and `params.rs` refused the
  three variants — so the frontend found no picker stage and submitted `{}`.
  **All three layers fixed generically.** `ActionParams` gains `discard_cards` /
  `effect_choice_answer` / `trigger_targets`; the three arms forward an announced answer and fall
  back to the same default as before — which is load-bearing, not conservative, because
  `random_bot` reaches them with `ActionParams::default()` and so every recorded fuzz seed's
  outcome is byte-identical (OOS-DP8-1, the constraint PB-DP8/DP9 wrote those arms under).
  `ActionOptionView.decision` is a `{question, prompt, answer_field, answer}` envelope whose
  `answer` is one of **four shapes** — `Subset` (CR 514.1), `PickOne` (CR 701.23a), `Partition`
  (CR 701.22a/701.25a), `Slots` (CR 603.3d). **A client dispatches on the shape, not the
  question**, which is what makes a scry and a surveil share one component (their entire
  difference is `moved_key`/`moved_label`, supplied by the server) and what let **OOS-DP8-2** —
  filed as the identical gap — reuse the CR 601.2c `TargetSlotView` with no new picker, no new
  answer encoding and no new client branch. `PickOne`/`Partition` carry the engine's own default
  answer as a `template` plus the JSON keys to fill, so the client **never spells
  `EffectChoiceAnswer`'s externally-tagged variant name** — `TargetOptionView::value`'s argument
  applied to a second type. `api.rs::validate_decision_params` makes an answer naming something
  the response never offered a **400**, not an engine 422.
  **Four HTTP probes, every one proven to discriminate by execution** (revert the arm, watch it go
  red): a pass-only 4-player seat driven to a CR 514.1 cleanup discard and answered with the
  *lowest* ids against a default that is the highest; Read the Bones' scry and Diabolic Tutor's
  search at a pinned fixed-deck seed, each pinning the observed default as the reproduction before
  driving a different answer; and Shadow Alley Denizen + Nezumi Prowler driving a real CR 603.3d
  announcement. Fixed decks reach the router through `session::new_game`, which `config_for`
  cannot express (`DeckSource::RandomPerSeat` is hard-coded) but which runs the same two
  Invariant-9 gates — so nothing about the HTTP path is stubbed, only the deck.
  **Three drafts of these probes were wrong, and each names a durable hazard.** (1) The scry and
  search probes first followed an `ObjectId` from the library into the hand: **CR 400.7** makes a
  card that changes zones a NEW object, so both now assert over the *library*, where the ids
  survive and the two answers are distinguished by which card is still in it. (2) The
  trigger-target probe first asserted "the chosen creature has a keyword" and **passed against the
  un-fixed code** — Nezumi Prowler is *printed* with Ninjutsu; it now asserts what each creature
  **gains** against a baseline taken before the answer. (3) The first `.objects()` gate read the
  wrong side of the `#[cfg(test)]` cut (`test_region` returns the suffix *from* the marker) and
  counted 0.
  **A fourth Architecture-Invariant-7 channel, opened deliberately and gated on its own terms.**
  `StateViewModel` carries `library_size` and no library *contents*, so `NameIndex` answers
  `(unknown card)` for every scry/search candidate — which is not *safer*, since the ids are
  already on the wire and must be, and makes the picker useless.
  `view::question_card_label` therefore reads the name off `GameState`, but only for ids drawn out
  of the engine's own `EffectChoiceQuestion` — whose `private_to()` already classifies exactly that
  id set as this seat's (CR 701.22a/701.23a/701.25a: the effect *tells* this player to look).
  MR-M11-01's lesson applies verbatim — **a redaction gate checks the channel it was written for**
  — so neither existing gate can see this one (the source gate scans for omniscient *view-model*
  entry points, and the body scan looks for another seat's *hand* names). It ships with a count
  gate instead: `view.rs`'s production code may read the raw object table exactly **twice**, and a
  third read must be a deliberate act.
  **The fix cycle's HIGH is the entry worth re-reading.** `question_card_label`'s doc **cited a
  gate test that did not exist** and said the channel "ships with its own gate rather than with an
  argument" — a claim in prose that no test holds, on the one subsystem MR-M11-01's lesson is
  about. It was a draft line left behind when the planned behavioural test was replaced by the
  source-count gate, and the README and this archive entry both described the real situation
  correctly, which is exactly how it survived self-review: *the wrong version was the one nobody
  re-read.* And the premise that test would have asserted was **enforced nowhere** — the channel
  needs its `EffectChoiceQuestion` to belong to the seat being rendered, and that held only by
  arithmetic on a one-element set (`config_for` hard-codes one human seat), so a second human seat
  would have rendered seat A's scried library cards, **named**, into seat B's payload.
  `api.rs::seat_view` now filters `pending.player == human` and the gate exists and is two-sided.
  **Generalisable: when a doc comment calls an argument "structural", check the structure is in
  the code and not in the configuration.** Four LOWs alongside it, one of which is a live UI trap:
  `ActionBar`'s decision guard required a truthy `currentShape`, so a malformed payload rendered
  nothing at all and **skipped the very unknown-shape fallback that exists to prevent a dead bar**.
  Tests **4,124 → 4,138** on branch (+8 `params.rs` unit, +6 play-server). **Zero engine lines** —
  empty `git diff` over `crates/engine/src` + `crates/card-types/src`. PROTOCOL **33** / HASH
  **70** unmoved, gate-executed rather than predicted. (The task's acceptance criterion said
  "PROTOCOL 32"; 32 → 33 was PB-DX6's bump on the parallel W6 track, which landed before this
  branch forked — the *claim* held, the *number* in the criterion was stale.) Coverage unmoved at
  1,137/1,804 = 63.0%, 0 card-def edits. **Not closed**: the TUI halves of OOS-DP7-6 / OOS-DP8-2 /
  OOS-DP9-7 (those rows are *about* the TUI); OOS-DP9-1 is unchanged and deliberately so — it is
  about the bot. No picker has an automated test; there is no frontend harness (plan §8 R7). Full
  limitation list: `tools/play-server/README.md` 14-17.

- 2026-08-02 — **PB-DX6 SHIPPED (`scutemob-172`): the last two unflattened mana-cost payment sites
  now flatten, and the payment path stops being able to fail OPEN.** PB-RS2 routed three payment
  sites through `ManaCost::flatten_hybrid_phyrexian` and left two standing. (1)
  `rules/engine.rs::handle_turn_face_up` paid a **raw** `def.mana_cost` — and it is **all three**
  `TurnFaceUpMethod` arms that share the defective block, not the `ManaCost` arm the brief named.
  (2) `Command::DeclareAttackers` had no payment-choice fields at all, so PB-DP4 had turned a silent
  free attack into a hard rejection: better, and still wrong. **The pre-fix numbers were OBSERVED in
  both build modes before a line was changed**, because plan §2.0 named a trap purpose-built to
  produce a plausible false claim: in **debug** — every `cargo test` run and all of CI — a
  manifested Kitchen Finks flip **panics** inside `debug_assert_flattened` ("2 hybrid + 0 Phyrexian
  pip(s) would be paid for free"); the "flips for `{1}`" figure is a **release-only** claim,
  produced by temporarily disabling the guard and **reading** the pool: `{1 colorless, 1 G, 1 W}` →
  `{0 colorless, 1 G, 1 W}`, both `{G/W}` pips free. That debug panic is itself the most useful
  thing the batch found: **every test build the project has ever run would have caught this, and no
  test ever put a pipped cost through the site.** **The attack-tax design was chosen on evidence,
  not taste.** Design (A) — replicate pips into the CR 508.1h total, flatten the total once — ships;
  design (B), flatten each restriction's `cost_per_creature` then multiply, is **rules-wrong** on
  the Norn's Annex ruling of 2011-06-01 (*"If a player attacks with more than one creature, that
  player chooses how to pay each cost **individually**"*), which (B) structurally cannot express,
  and it fails in the *quiet* direction — accepting the command and charging a legal-but-not-chosen
  total, the "legal but wrong" class this project ranks as its biggest pre-alpha risk. The pip order
  is **copy-major** (`[r1, r2, r1, r2, r1, r2]`, never `[r1, r1, r1, r2, r2, r2]`) so that "creature
  *k*'s pips live at offsets `[k·P, (k+1)·P)`" is true — the only form the ruling or a UI can be
  stated against — and it is written down in all three places a reader could need it.
  `unpayable_tax_defenders` is renamed `x_tax_defenders` and narrowed to **X only**, because a name
  asserting "unpayable" when hybrid and Phyrexian are now payable is a lying identifier of exactly
  the class this suite keeps re-creating; its message now cites CR 107.3/601.2b and the new
  **OOS-DX6-1**. New read-only `rules::queries::attack_tax_total` exists because the attack-tax cost
  is the one payment cost a client **cannot** derive — `LegalAction::DeclareAttackers` carries no
  attacker set at all — and **exactly one** accumulation (`accumulate_attack_tax_total`) serves both
  it and the validation path, since two copies of the pip order is how this whole seed family
  started. **`ManaPool::can_spend`/`spend` stop failing open.** PB-RS2's own review had written the
  honest description into the doc block — *"In release, this guard fires **NEVER**"* — which
  describes a guard that, compiled out, answers *"yes, you can pay this"* and debits the pool as if
  the pips were not there: a silent **undercharge**, the corrupted-history failure Architecture
  Invariant 9 exists to forbid. The fix is option 2 of four, and **the asymmetry is the argument**:
  `can_spend` is a **question**, for which a truthful conservative answer exists (a cost with an
  unresolved hybrid pip *is not payable as given*), so it returns `false`; `spend` is an
  **instruction**, for which **no truthful execution exists**, and whose documented precondition is
  `can_spend` — so reaching it with a residue is a violated precondition and `assert!` is the
  sanctioned answer. Option 3 (`Result` signatures) was rejected on the decisive ground that it
  **launders an engine bug into a rules answer**: every caller would `?` it into `InvalidCommand`,
  making "the engine has a bug" indistinguishable from "your command was illegal" — the exact
  distinction SR-4's vocabulary exists to preserve, and *quieter* than a `false`, not louder. §6.1's
  existing SR-4/SR-6 argument was accepted whole and its paragraphs kept **verbatim**; only the
  now-false "fires NEVER" paragraph was replaced. **Testability is stated honestly rather than
  overclaimed**: with the `debug_assert!` retained, `can_spend`'s `return false` branch **cannot
  execute in a debug build** and CI builds debug — so the predicate is tested directly in every
  build, `spend`'s panic test loses its `debug_assertions` gate and now proves the **release**
  behaviour of the mutating path in the normal suite, and the `can_spend` branch is observed
  **exactly once by execution** (`cargo test -p mtg-card-types --release`, 12/12), with a comment
  saying plainly that it does not run in CI. **Both fingerprints computed, never predicted**:
  **PROTOCOL 32 → 33** taken from the failing gate's own output, with the falsifier named in advance
  ("if it passes unchanged, the scanner is not seeing the new fields — **stop**, do not bump a
  version to make a green gate greener") not occurring; closure type count **unchanged at 96**,
  since `HybridManaPayment` and `bool` were already reachable via `CastSpellData`/`ActivateAbility`
  — exact precedent, history row `- 27: PB-RS2`. **HASH confirmed unmoved at 70** by running `--test
  core hash_schema`. **0 completeness flips, pre-committed and held**, confirmed by an empty `git
  diff` over `crates/card-defs` and a coverage regeneration whose body came back byte-identical — so
  PB-DX4's seeded-deck re-deal hazard never fired and the play-server seed pins were not touched.
  **The review's HIGH is the durable lesson, and it is uncomfortably on the nose: a batch that warns
  twice about claims-recorded-as-measured shipped one of its own.** The accumulation *is* genuinely
  copy-major — but the order-pin test **could not fail** under the pip-major permutation it existed
  to catch, because copy- and pip-major diverge **only** when a single `add_mana_cost` call has
  `times > 1` **and** more than one pip, and the fixture (1 restriction × 2 attackers, and 2
  restrictions × 1 attacker) never produced one; both orders yielded the identical vector, and the
  "pool drains to exactly zero" argument the test's own doc offered as its real discriminator was
  satisfied identically too. Worse, `multiply_mana_cost`'s **freshly rewritten** doc asserted that a
  dedup would cause "no test failure unless a probe pins the order (pb_dx6's order-pin tests do)" —
  **false as shipped**, the PB-DX5 "verified: none exist" class reproduced inside the batch that
  cites it. Fixed with the **minimum discriminating fixture** — one defender, **two distinct**
  restrictions, two attackers — and proven **by execution in both directions**: swapping the loop
  for `multiply_mana_cost`'s pip-major `flat_map(repeat_n)` left the old test **green** and reddens
  the new one with exactly the predicted "hybrid pip {Green/White} cannot be paid with Red". **The
  second finding is about damage this batch did to somebody else's tests**: PB-DP4's two E1 CR
  508.1c scoping pins both used a *hybrid* restriction, which stopped being a rejection class the
  moment this batch landed — so PB-DP4's E1 fix had silently lost **all** regression coverage.
  Verified rather than assumed (E1's scoping loop reverted; both tests stayed green), then moved to
  `x_count: 1` and the revert repeated, at which point both correctly failed. A third:
  `test_107_4e_hybrid_attack_tax_is_rejected_not_paid_free` was passing on a **`Debug`-string
  artifact** — `ManaCost`'s `Debug` prints the field name, so `contains("hybrid")` was satisfied
  unconditionally by `hybrid: []`. **Review 1 HIGH / 8 MEDIUM / 6 LOW, all 15 applied**, and every
  one **re-verified by execution before being acted on**, because the reviewer had no shell and said
  so per-finding — which is itself the reusable point: a finding established by reading is a
  hypothesis, and one of them (the seed count in the TUI gap) was wrong on the numbers. Tests 4,066
  → **4,099**; benches within noise (`full_turn_4p` 220–222 µs) since the flatten runs once per
  declaration, not per attacker; clippy / `cargo fmt --check` / `tools/check-defs-fmt.sh` (1,804
  defs) clean; 210 golden scripts green, 0 new skips. Durable records:
  `memory/primitives/pb-plan-DX6.md` and `pb-review-DX6.md`; seeds **OOS-DX6-1..5** in
  `docs/audits/decision-point-audit.md` §8.1, with **OOS-DP4-7 re-dispositioned, not closed** (the
  reason the seed lacked: `multiply_mana_cost` is pip-major, so the proposed dedup would silently
  re-order the tax's pips and re-interpret every `hybrid_choices` vector a client had already
  built). **Next: PB-DX7** (OOS-DP7-11 + OOS-DP9-13 — the SR-19 gate reports success while checking
  nothing; gate integrity, 0 flips, test-only).

- 2026-08-01 — **M11-local SHIPPED (`scutemob-173`, S8): the engine is first-playable, and the
  session that closed it found four defects by the simple expedient of playing five games.** A human
  sits at one seat of a 4-player Commander game in a browser against three bots, with no networking,
  across `LocalGame` → `setup.rs` → `crates/view-model` → `tools/play-server`. **Wire-neutral end to
  end** — no new `Command`/`GameEvent`/`Effect` variant in any of the 8 sessions, PROTOCOL **32** /
  HASH **70** unmoved by an empty `git diff` over `crates/engine/src`. **S8's headline is that the
  acceptance test earned its place.** Plan item 1 asked for a scripted-human playthrough on 5 seeds;
  the first run failed, and so did the next three, each for a different real reason. **(1)**
  `invariants::check_stack_consistency` compared **two different id spaces**: `casting.rs` moves a
  cast card into the Stack zone as `ObjectId` *n* and then mints the `StackObject` as *n+1*, so the
  check fired twice per spell and once per ability, always, in games with no defect — **501 spurious
  violations across 500 fuzz games** at the merge base, **0** after rewriting it against
  `StackObjectKind::Spell { source_object }`. That is what OOS-DP3-9's "long games trip
  `stack_consistency`" had been the whole time. **(2)** `mana_solver` held one entry per (permanent
  × mana ability) and marked only the chosen entry spent, so a permanent with two mana abilities was
  planned into two `TapForMana`s and the second was refused — a bug reachable in every game since
  the solver was written, and invisible until a *human* path refused to swallow the engine's error,
  because `advance()`'s bot fallback had always absorbed it as a `PassPriority`. **(3)**
  `HeuristicBot` froze the table twice over, on a free repeatable no-op ability
  (`lightning_greaves`' Equip `{0}`, whose runtime `ActivatedAbility` declares `targets: []` while
  its effect names `DeclaredTarget { index: 0 }`) and on re-declaring the same combat; CR 104.4b
  catches neither, because both are *optional* actions. Fixed with a per-turn preference cap **in
  the bot, not the provider**, so the fuzzer's draw sequence is untouched. **(4)** the playthrough's
  own `max_commands` inherited `GameDriver`'s `max_turns * 200`, which is the fuzzer's ratio for
  games that start with empty hands; a real table runs ~260 commands/turn, so the command valve
  fired before the turn cap. **Two of the four bottom out in the engine and were filed rather than
  fixed, because M11-local makes no engine change**: **OOS-M11-7** (CR 704.3 says SBAs are checked
  whenever a player *would receive* priority; this engine checks them on step entry and at
  resolution, so a Treasure sacrificed for mana sits legally in the graveyard through several passes
  — the playthrough asserts the strictly stronger end-state property that no token leaked at all)
  and **OOS-M11-9** (neither the provider nor `handle_declare_attackers` gates "already declared
  this combat"; CR 508.1 makes it a once-per-combat turn-based action and the engine will accept
  re-declaration without limit). **Plan item 2's premise was stale before the milestone finished,
  and that is the reusable lesson**: it lists Echo / Cumulative Upkeep / Recover as needing new
  `LegalAction` variants, and **PB-DP4 (`scutemob-152`) had shipped all three, with SR-38
  affordability gating, on the same day the plan was written**. Only `OrderBlockers` (CR 509.2) was
  genuinely missing. *A plan item naming missing work is a dated claim; read the code before
  building it.* **`Concede` (CR 104.3a) and `OrderBlockers` are offered to human seats only**,
  appended by `local_game::human_only_actions` and never by `StubProvider` — a bot must not
  auto-concede, and appending to the provider's list re-rolls every `RandomBot` draw downstream of
  it, which is precisely what would have made the R11 gate unmeasurable. **The R11 gate was
  measured, and so was its narrowing**: 500 games at the same seed, merge-base worktree vs branch,
  **0 games differ in turns, commands or outcome**; and it could **not** be run at the plan's
  default `--max-turns 200`, because `mtg-fuzzer` stack-overflows **at the merge base** (OOS-DP3-9,
  reproduced single- and multi-threaded and with a 128 MiB `RUST_MIN_STACK`), so it ran at 40 turns
  and `memory/m11/s8-fuzz-parity.md` says so rather than reporting a number that would imply more
  coverage than it has. **`GET /api/game/report`** ships the Layer-3 repro artefact (`{seed, config,
  PROTOCOL + fingerprint, HASH, final state hash, journal}`) plus an Export button; it is a **pure
  read** (it uses `journal()`, not `take_new_records()`, so an export cannot swallow event lines the
  live feed has not shipped — tested) and it is the **one payload in the crate that is deliberately
  not seat-redacted**, because a redacted repro is not a repro, safe only because M11-local is one
  human + three bots in one process and **explicitly flagged for re-scoping at M10a**. **OOS-M11-8
  closed** (`auto_tap_commands_for` now adds `x_value × mana_cost.x_count`, so a human can cast an
  `{X}` spell) — and the first attempt to prove that test discriminates was itself invalid, because
  clippy `-D warnings` failed the with-the-fix-disabled build and cargo reused the stale binary: **a
  revert-and-rerun proves nothing unless the rebuild succeeded.** Tests 4,072 → **4,097** (+25 —
  4,092 at the implement phase, +5 more in the close-out fix cycle for the milestone review), every
  figure measured rather than carried forward; clippy / `fmt` / `check-defs-fmt.sh` (1,804 defs) /
  `build --workspace` clean. **Next: nothing new is started.** The PB-DX correctness queue continues
  alone (PB-DX6 in flight as `scutemob-172`); the roadmap's next milestone candidate is **M10-pre →
  M10a**, which is *not* to be started without direction.

- 2026-08-01 — **PB-DX5 SHIPPED (`scutemob-170`): CR 611.2c is implemented, and the batch's headline
  is a defect nobody had filed.** `struct ContinuousEffect` stored an `EffectFilter` and no
  affected-object set, and `rules/layers.rs`'s `effect_applies_to` re-evaluated that filter live
  against current state on every characteristics calculation. CR **611.2c** says the opposite: *"If
  a continuous effect generated by the resolution of a spell or ability modifies the characteristics
  or changes the controller of any objects, the set of objects it affects is determined when that
  continuous effect begins. After that point, the set won't change."* The fix is a new
  `affected_set: Option<OrdSet<ObjectId>>`. `Some(set)` means the effect came from a resolution, and
  `effect_applies_to` answers by **membership alone**, deliberately ignoring `chars`, `obj_zone` and
  the live filter — precisely the three things CR 611.2c forbids re-consulting. `None` means a
  **static** ability, which CR **611.3a** says *"isn't 'locked in'; it applies at any given moment
  to whatever its text indicates"*, so statics keep the live behaviour they always had and were
  already correct. Exactly one site populates the field — `Effect::ApplyContinuousEffect`, via the
  new `rules::layers::snapshot_affected_set`, called *before* the effect is pushed so
  `calculate_characteristics` cannot see the effect being created — and the snapshot is computed by
  calling the same `effect_applies_to` predicate the layer system applies, so determination and
  application cannot drift. Every one of the 32 `LayerModification` variants either modifies
  characteristics or is `SetController`, re-derived independently in review, so the lock is
  unconditional with no rules-modifying carve-out to write. **`is_effect_active` was deliberately
  not changed**, against both the dispatch brief and this task's own acceptance criterion, which
  name it alongside `effect_applies_to`: it takes no `object_id`, so a per-object locked set is
  structurally not expressible there, and an effect whose locked set is empty is still *active* — CR
  611.2b describes an outcome, not non-existence. Ruled correct in review and pinned by a test
  rather than left as prose. **The roster the batch was dispatched on was wrong twice over, which
  makes this the sixth consecutive batch in this suite whose published roster was wrong before it
  started.** The brief's "**9** defs, **7** `Complete`" came from a per-file grep conjunction — file
  mentions `ApplyContinuousEffect` *and* file mentions `EffectFilter::All*`. Enumerated instead from
  `all_cards()`, walking each `ApplyContinuousEffect`'s **own** `effect_def.filter`: **116** defs
  generate a resolution-time continuous effect at all, and **38** use a mass filter — **29
  `Complete`**, 8 `partial`, 1 `known_wrong`. The grep missed the entire `CreaturesYouControl*`
  family, 27 defs, for the sole reason that the filter name does not begin with `All` — and that
  family holds the most-played members of the class: **Craterhoof Behemoth**, Purphoros, Mirror
  Entity, Triumph of the Hordes, Unbreakable Formation, Goblin Bushwhacker, Ezuri. Craterhoof is the
  sharpest instance in the corpus: a creature entering after it resolved wrongly received +X/+X
  **and trample**. In the other direction the grep counted `elvish_dreadlord`, whose only occurrence
  of `ApplyContinuousEffect` is inside a **blocker-note string** describing a rewire that was never
  done — it generates no continuous effect at all. **Three separate arithmetic slips were caught
  inside the batch, all by the same move.** The premise phase published 37/28; the plan published
  37/28 while its own table already listed 38 rows summing to 29; the implement phase published
  +16/4,064 when the new roster file contains **two** `#[test]`s and the true figure is +17/4,065.
  Each was found by re-running the measurement, and not one by re-reading the prose — which is the
  entire lesson, since a number that was derived is indistinguishable in a document from one that
  was read. **The batch closed a second and strictly larger defect, and did not know it until review
  — OOS-DX5-7.** `effect_applies_to`'s source-relative arms (`CreaturesControlledBy` and siblings)
  resolve the controller through `state.objects.get(&source_id)` and require the source object to
  still exist. For an instant or sorcery, `ctx.source` is the *spell's card object*, which
  `resolve_top_of_stack_inner` moves to the graveyard **after** effects run — a new object under CR
  400.7, so the old id is dead. Pre-fix, therefore, *Triumph of the Hordes*, *Unbreakable
  Formation*, *Goblin Surprise* and *Return of the Wildspeaker* applied to **nobody at all** from
  the instant they resolved. That is a bigger and more visible bug than the "a newcomer wrongly gets
  it" the seed was filed on, it was invisible to the seed's own framing, and it also revealed that
  the batch's own T12 had been mislabelled about which mechanism it demonstrated. Verified
  empirically in the fix cycle — membership read reverted, both board creatures observed collapsing
  to their printed power — rather than reasoned to. **Both fingerprints were computed, never
  predicted.** `HASH_SCHEMA_VERSION` 69 → **70** is mandatory, since the field is hashed;
  append-only history row added; **43** sentinel assertions re-pinned via the **symbol** grep, two
  of which are multi-line `assert_eq!`s that the single-line pattern structurally cannot see and
  that only a full workspace run with `--no-fail-fast` caught — a reusable finding about that re-pin
  procedure, not a one-off. `PROTOCOL_VERSION` was **confirmed unmoved at 32** by executing `--test
  core protocol_schema`, the falsifier the plan named in advance: `ContinuousEffect` sits outside
  the SR-8 wire closure. PB-DX1's lesson that anything reachable from `Characteristics` is PROTOCOL
  too was the reason to check and, here, did not apply — which is what "gate-compute, do not assume"
  is for in both directions. **Yield 0 flips, exactly as pre-committed**, because this is a pure
  engine correctness fix that makes 29 existing `Complete` defs behave correctly rather than a DSL
  gap; coverage holds at 1,137/1,804 = 63.0% with the report body byte-identical, so PB-DX4's
  seeded-deck re-deal hazard never fired and the play-server seed pins were not touched. **One
  existing test had been asserting the bug while citing CR 611.2c as its own justification** —
  `pb_ac3_dynamic_pt_counts.rs::test_set_both_dynamic_locked_at_resolution` claimed the rule
  required *filter membership* to be re-evaluated continuously while only the *value* stayed locked,
  which is the rule read backwards. Inverted with the rule text quoted, renamed, and
  **strengthened** to an exact `Some(1)` — the newcomer's own printed power — rather than loosened.
  No assertion anywhere in the batch was weakened. **Review 0 HIGH / 6 MEDIUM / 6 LOW, all 12
  applied.** Every MEDIUM was the same shape: *a claim recorded as measured that had been reasoned
  to*. Two of them had written a false claim into engine source — `snapshot_affected_set`'s doc
  block asserted "verified: no Layer-≤4 divergence exists in the roster", which asks the wrong
  question, because the divergence is caused by any Layer-≤4 effect that **writes** the
  characteristic the filter reads, not by mass-filter defs, and `inkmoth_nexus` does exactly that
  (`TypeChange` + `AddCardTypes([Creature])`). Animate a Nexus, then activate Mirror Entity:
  post-fix the Nexus correctly receives the grant. The claim was corrected, the seed reopened from
  "checked non-finding" to an open finding, and a real discriminating test added. The narrow lesson
  worth carrying past this batch: **a doc comment reading "verified: none exist" is a dated claim
  about a question somebody chose, and the question can be wrong even when the answer to it is
  right.** **Probe discrimination was verified by execution rather than asserted**: with the
  read-site membership block disabled, **8 of the 15** probes fail (mass -1/-1 newcomer, Craterhoof
  newcomer, control-change retention, Umezawa's Jitte, SBA-after-debuff, CR 702.26e phased-out
  exclusion, PB-DP9 abort-and-replay, Layer-≤4 divergence) and exactly the 7 that must be
  insensitive stay green (static anthem in **both** directions, `SingleObject` unchanged,
  `is_effect_active` unchanged, CR 400.7 leave-and-return, phase-in). Tests 4,048 → **4,066**;
  benches within ~1% of the merge base with `board_wipe_4p` — flagged in advance as most likely to
  move — measuring slightly *faster*, because the snapshot runs once per resolution and not once per
  layer pass; clippy / `cargo fmt --check` / `tools/check-defs-fmt.sh` (1,804 defs) clean. Durable
  records: `memory/primitives/pb-plan-DX5.md` and `memory/primitives/pb-review-DX5.md`; seeds
  **OOS-DX5-1..8** in `docs/audits/decision-point-audit.md` §8.1. **Next: PB-DX6** (OOS-RS2-1 +
  OOS-DP4-1 — the two mana-cost payment sites PB-RS2 left unflattened: `handle_turn_face_up` pays a
  raw `def.mana_cost` and `can_spend`'s residue guard is `debug_assert`-only, so in release every
  hybrid and Phyrexian pip in a `TurnFaceUpMethod::ManaCost` flip is free — `kitchen_finks` is
  `Complete` with two `{G/W}` pips; and `Command::DeclareAttackers` has no
  `hybrid_choices`/`phyrexian_life_payments` fields at all, so a hybrid attack tax cannot be paid.
  One PROTOCOL bump for the batch, and make the residue guard fail loud).

- 2026-08-01 — **PB-DX4 SHIPPED (`scutemob-168`): the 97-entry decision `BASELINE` is oracle-read,
  and the estimate it was ranked on was wrong in the direction that costs coverage.** PB-DP10 froze
  97 `Complete` defs carrying an engine-made choice into `decision_gate.rs`'s `BASELINE` but
  populated it mechanically; the plan §5.3 class-B/class-D triage was never done. All 97 have now
  been read against MCP printed text, with the roster **parsed out of the const array itself rather
  than taken from prose** — this suite had published a plausible roster and been wrong three times,
  so the 97 entries were resolved to 97 distinct names and 97 unique def files before a single card
  was read. **The split is 84 class-B / 13 class-D.** PB-DP10's closing spot-check found 2 of 5 and
  flagged itself as "a very noisy sample"; it was, overstating the rate roughly fivefold — and the
  queue row's "0 flips" estimate was wrong the other way, because 5 of the 11 could not be fixed and
  had to be demoted. Coverage 1,143 → **1,137**. **Six repaired in place and still `Complete`**:
  `metastatic_evangel` (four separate defects — `{2}{W}` for a printed `{1}{W}`, a missing `Human`
  subtype, a **transposed** 1/3 for a printed 3/1, and no nontoken filter, the last of which its own
  in-def note declared unauthorable because "`is_token` … for ETB trigger matching is silently
  ignored" — **stale**, since PB-AC0 forwards the whole `TargetFilter` as
  `triggering_creature_filter` and `abilities.rs` honours `is_nontoken` on exactly that path);
  `grisly_salvage` and `satyr_wayfinder`, both of which used `Effect::RevealAndRoute` — which routes
  **every** match — for a printed "You **may** put **a** creature or land card … into your hand", so
  a two-mana instant put three to five cards in hand, mandatorily; `sword_of_truth_and_justice`,
  whose bare `TargetCreature` let its own +1/+1 counter be placed on an opponent's creature against
  a printed "a creature you control"; and `radstorm` at `{2}{U}` for a printed `{3}{U}`, which on a
  Storm card compounds into extra copies rather than staying a one-mana discount. **Six demoted,
  each with an oracle citation**: `smugglers_copter` → `known_wrong` (printed "you **may** draw a
  card. If you do, discard a card" as an unconditional `Sequence` on both triggers — the 20th
  instance of audit §5's DP-12 class, where the other 19 were **already** `known_wrong`, so the
  marker rather than the encoding was the whole defect); `contaminant_grafter`,
  `grateful_apparition` and `thrasios_triton_hero` → `partial`; and `shambling_ghast` → `partial`
  **for a defect that fixing it surfaced**. That last one is the batch's sharpest finding. Its three
  alleged deviations all held and were all *fixed* — a phantom `KeywordAbility::Decayed` the printed
  card does not have at all (MCP keywords: `["Treasure"]` only), a permanent `MinusOneMinusOne`
  counter where the card says "-1/-1 **until end of turn**", and a stored `oracle_text` reading
  "When Shambling Ghast **enters**" against the def's own `WhenDies` trigger — but with those gone a
  fourth became visible: the mode-1 target is declared **flat** on the trigger, so it is required
  whichever mode is chosen, and a Ghast dying while no opponent controls a creature yields nothing
  at all (CR 603.3d) where the printed card simply makes a Treasure. `ModeSelection.mode_targets` is
  the CR 601.2c-correct scoping and **every consumer of it lives on the casting path** — nothing on
  the triggered-ability path reads it — so the obvious repair would have silently *dropped* the
  requirement instead of scoping it (**OOS-DX4-2**). **One left `Complete` deliberately**:
  `staff_of_compleation`'s printed "target permanent **you own**" authored as
  `TargetController::You` is real and reachable under any control-change effect, but `TargetFilter`
  has no owner axis at all and the identical deviation ships reviewed-and-allowlisted on
  `nether_traitor` (whose own note names `athreos` and `fecundity` as further members). It was
  allowlisted to match, because demoting the two members that happen to sit inside PB-DP10's 97
  would have reported a corpus class as a pair of cards; the class question is **OOS-DX4-1**. **The
  batch also closed OOS-M11-6, and found it by accident.** Demoting `thrasios_triton_hero` — a
  Legendary Creature, and therefore a member of `random_deck`'s own commander pool — shifted every
  seeded deck in the workspace and landed `setup.rs`'s seed 9001 on Rograkh, Son of Rohgahh, the
  corpus's **only** colourless `Complete` legendary creature (1 of 91), exposing the CR 903.5c
  Forest padding filed a day earlier by M11-local S5. Fixed the way that seed itself preferred — pad
  a colourless deck from the identity-legal colourless pool rather than exclude colourless
  commanders, viability **measured** (40 colourless nonbasic lands + 82 colourless nonlands = 122
  distinct singletons against the 99 a deck needs) — with **both** Forest fallbacks removed, as the
  seed correctly predicted there would be two. The larger half of that finding is that the fuzzer
  feeds `random_deck` straight to `GameStateBuilder` with no validation, so it had been silently
  **playing** illegal decks, not refusing them. Its two coupled play-server fixtures lost their only
  failure trigger exactly as their own maintenance note predicted, and now use an explicit sentinel
  **seed**; a first attempt used a process-global flag that raced with every other test POSTing
  `/api/game` — green under `-p`, red under `--workspace`, twice — which is why the trigger is
  carried by the request. Golden script `baseline/112` **retired**: it tested Decayed on Shambling
  Ghast and cited the *card definition* as its authority ("engine-verified from
  …/shambling_ghast.rs"), which is how a phantom keyword propagated into a golden script that then
  stood as evidence for it — a provenance failure rather than a stale assertion. Coverage was
  checked before retiring, not assumed: CR 702.147a keeps twelve unit tests in
  `mechanics_a_d/decayed.rs`, none referencing the card, and the golden-level gap is filed as
  **OOS-DX4-3** rather than left silent. **Numbers, each re-measured against `all_cards()` rather
  than derived**: coverage 1,143 → **1,137** (63.0%), tests 4,040 → **4,048**,
  `MAX_AUTO_CHOSEN_COMPLETE_UNION` 97 → **91**, deviation floor 661 → **667**, PB-DP8 roster 76 →
  **74**, `scry` 16 → **15**. The union pin moved **twice inside the batch** (97→93→92) because
  Shambling Ghast's fourth defect only appeared after its first three were fixed — which is
  precisely why it was read off the gate rather than computed, and worth carrying: yield can move
  mid-batch. **0 engine lines** — empty `git diff` over the whole of `crates/engine/src` *and*
  `crates/card-types/src` — PROTOCOL 32 / HASH 69 unmoved, clippy / `fmt` /
  `tools/check-defs-fmt.sh` (1,804 defs) clean. **PB-DX3b's parting question is answered and the
  answer is not a handful: 966 of 1,804 def files never mention `completeness` at all (970 before
  this batch)**, so a clear majority of the `Complete` population is `Complete` because nobody wrote
  the field — and **eleven of the thirteen** class-D defs were in that group. Now ratcheted in the
  growth direction. Durable record with per-def oracle citations:
  `memory/primitives/pb-dx4-baseline-triage.md`, which also states plainly what the triage does
  **not** establish — it is a dated claim nothing re-reads, it cannot see a decision the DSL never
  encoded (OOS-DP10-9 stands, and three of the eleven were exactly that class, caught by reading
  oracle text rather than by any gate), and 97 of 1,143 `Complete` defs is not a sample the other
  1,046 can be inferred from. **Next: PB-DX5** (OOS-OS7-2, CR 611.2c affected-set snapshot — 7
  live-wrong `Complete` defs; compute both fingerprints).

- 2026-08-01 — **PB-DX3b SHIPPED (`scutemob-166`): the OOS-DX3-1 insert, and the seed's own triage
  was wrong about one of its cards.** The batch closed the remainder of the `pb-plan-DP6.md:395`
  stale-blocker bucket — **all seven** defs dispositioned explicitly, four fixed and three deferred,
  none silently dropped. `jadar_ghoulcaller_of_nephalia` was a `Complete`, deck-legal def with
  `intervening_if: None` that made a 2/2 decayed Zombie **every** end step; it stays `Complete` and
  is now gated with `Not(YouControlNOrMoreWithFilter{1, Creature + has_keywords[Decayed]})`. **Its
  stored `oracle_text` was wrong, not merely its blocker note** — the field said "if you control no
  tokens named Shambling Ghast" and the note therefore declared a `Condition::NoTokensNamedX` DSL
  gap, chasing a filter the printed card never had (MCP: "if you control no **creatures with
  decayed**"). That is a distinct failure mode from PB-DX3's: not a note that went stale, but a note
  that was **never right**, because the text it was reasoning from was itself wrong. `ophiomancer`
  `partial` → `Complete`, deliberately using `has_subtype: Snake` alone rather than the def's own
  suggested `ControlCreatureWithSubtype`, whose match arm hard-requires `CardType::Creature` while
  CR 205.3 reads "Snakes" as permanents with the subtype. `dwynen_s_elite` `inert` → `Complete`, its
  ability **authored from nothing** — the `inventors_fair` shape from PB-DX3 recurring, and now a
  pattern to expect rather than rediscover: a stale blocker note reads as though the ability were
  present but ungated when it is often absent entirely. **The headline is that the seed
  mis-dispositioned a second live-wrong `Complete` def into its own "genuinely blocked" pile.**
  `emeria_the_sky_ruin` declares **no `completeness` field at all**, so it was `Complete` by the
  `#[default]` derive — deck-legal — and reanimated a creature every upkeep regardless of Plains
  count, while its blocker note asked for a `Condition::YouControlNOrMorePermanentsWithSubtype` that
  `YouControlNOrMoreWithFilter { count: 7, filter: has_subtype Plains }` has expressed all along.
  This is the `aurelia_the_warleader` trap from PB-DX1 hit a **second time in three batches by a
  different route**, which promotes it from an anecdote to a class: **`#[default]
  Completeness::Complete` is a silent-defect generator, and "which defs never declare a marker at
  all?" is a cheap corpus-wide question nobody has asked.** Emeria was gated and given an
  **explicit** `partial`, because the printed "you **may** return" has no free-optional mechanism —
  the search was run rather than assumed (`MayPayThenEffect` requires a `Cost` and a free one always
  trivially pays, producing behaviour byte-identical to the unconditional effect; `MayPayOrElse` and
  `Effect::Choose` are both barred from `Complete` by `effect_choose_gate.rs`, and
  `Effect::Choose`'s own doc says *"do not reach for this to express 'you may X'"*; PB-DP9's
  `pending_effect_choice` channel serves search/scry/surveil only) — the same class as
  **OOS-DP10-8**. The review then found a **spurious `Legendary` supertype** on Emeria as well (MCP
  type line is `Land`; control-tested against Gaea's Cradle → `Legendary Land` and Valakut → `Land`,
  Emeria being in Valakut's nonlegendary Zendikar cycle despite the comma name), so CR 704.5j would
  have wrongly applied — removed in the fix cycle. **Yield: 2 flips up, 1 honest flip down — net
  coverage 1,142 → 1,143, +1 and not +3**, and worth stating plainly because the batch's own plan
  wrote "+2" in a sentence that also correctly listed +2 up and −1 down; the arithmetic slip was
  caught and corrected on closure rather than banked. **0 engine lines** — an empty `git diff` over
  the whole of `crates/engine/src` *and* `crates/card-types/src` — PROTOCOL 32 / HASH 69 unmoved,
  clippy / `fmt` / `tools/check-defs-fmt.sh` (1,804 defs) clean, tests 4,008 → **4,022**. Golden
  script `combat/191` was reconciled by **strengthening**: it had never asserted the Zombie token at
  all and passed whether or not the token existed, its `generation_notes` and open dispute both
  narrating a resolution-time engine gap that B14's card-registry fallback had closed long ago — the
  same vacuous-assertion shape PB-DX3's MEDIUM was about, sitting unnoticed in the golden corpus.
  Fixed by asserting the token, rewriting the prose, and resolving the stale dispute (description
  left verbatim). **Every pre-fix claim in the new test module was observed, not reasoned to** —
  reverted def, instrumented re-run, numbers read, restored — and the three claims that genuinely
  could not be observed (Dwynen's ability did not exist pre-fix) are labelled vacuous rather than
  given a manufactured number; the review verified each fixture could actually produce the number it
  reported, which is precisely the check PB-DX3's T1 had failed. Review 0 HIGH / 5 MEDIUM / 7 LOW,
  **all 12 applied**, all four completeness moves independently ruled justified clause-by-clause
  against MCP oracle text and every ruling. New seed **OOS-DX3b-1**: `guardian_project`'s note is
  half stale too — its `is_nontoken` half is authorable today (PB-AC0 wired
  `triggering_creature_filter` through the creature-ETB dispatch), its name-uniqueness half
  genuinely is not, so the def correctly stays `known_wrong`. Two pinned test floors moved as a
  legitimate consequence of Emeria's marker correction (`decision_gate` 77 → 76,
  `completeness_deviation_scan` 662 → 661), both with derivations inlined; the latter's message text
  had itself been carrying a stale "669" for eight batches. **Next: PB-DX4** (OOS-DP10-8, the
  97-entry `BASELINE` triage) — carry the `#[default]` marker question into it.

- 2026-08-01 — **PB-DX3 SHIPPED (`scutemob-164`): two `partial` blocker notes that had been stale
  for a week and a half are closed, and the batch caught itself committing the same sin.**
  `garruks_uprising` and `inventors_fair` were both marked `partial` on a blocker that no longer
  existed. Both notes named the **runtime** `InterveningIf` enum ("only `ControllerLifeAtLeast` /
  `SourceHadNoCounterOfType`") when the def-level field is `intervening_if: Option<Condition>` — and
  `Condition::YouControlNOrMoreWithFilter { count, filter }` had been sitting in
  `card_definition.rs` with **21** shipped users the whole time. The notes were stale **twice
  over**: the runtime enum they misname now has *three* variants, because PB-DX1 added
  `InterveningIf::CardDef` two batches earlier in this very queue. Their hedge that "the
  trigger-time half remains blocked" was false as well — the variant is in
  `condition_is_queue_time_evaluable`'s true set, PB-DP6 wired the card-def `intervening_if` into
  the ETB queue site (`replacement.rs::queue_carddef_etb_triggers`) and the upkeep sweep
  (`turn_actions.rs`), and PB-DX1 added the resolution-time counterpart, so **both halves of CR
  603.4 have been available since `scutemob-154`**. **The scope was bigger than the seed said in one
  specific way, and it matters for how these rows should be read**: `inventors_fair`'s upkeep
  trigger **did not exist at all**. The seed, the queue row and the def's own note all read as
  though the ability were present but ungated; in fact `abilities` held only the mana ability and
  the search ability, so the batch had to *author* "At the beginning of your upkeep, if you control
  three or more artifacts, you gain 1 life" from scratch. **The activated ability's condition went
  on `activation_condition`, deliberately and not by default** — ruling 2016-09-20 #3 says the
  artifact count is checked *only as you activate it* and never re-checked at resolution, so an
  `Effect::Conditional` wrapper would have been wrong; T10 pins that by dropping to one artifact
  after a legal activation and asserting the search still asks. **10 fail-before probes**, each CR-
  or ruling-cited; T9 drives the search end to end through PB-DP9's `EffectChoiceRequired` /
  `AnswerEffectChoice` channel and asserts the *announced* artifact reaches hand, the un-chosen
  candidate stays in the library, and a real `LibraryShuffled` fires for the printed "then shuffle".
  **The review's single MEDIUM is the durable lesson, and it is uncomfortably on the nose: a batch
  whose entire subject is stale notes wrote a stale note of its own.** The test module recorded a
  pre-fix observation for T1 — "the post-resolution hand count was 1, not 0" — that **could not have
  been observed against T1's own fixture**, which had no library object; drawing from an empty
  library sets `has_lost` and emits `PlayerLost` rather than incrementing the hand, and the
  companion assertion (`hand_count == hand_before`) therefore passed whether or not the bug fired.
  The number had been reasoned to, not read. **The repair was to make the claim checkable and then
  actually check it** — give T1 a real library card, revert `intervening_if` to `None`, re-run, read
  the numbers — rather than to reword the prose; the same standard was then applied to
  T3/T5/T6/T7/T8, all of which held (T5 vacuously, as the plan predicted, since the ability was
  absent). The original claim turned out **right**, which is exactly why it is worth recording: a
  true statement arrived at by inference is indistinguishable, in a document, from one arrived at by
  observation, and only the second survives the next refactor. **Yield exactly as predicted — 2
  flips, 0 engine lines, 0 wire**: PROTOCOL 32 / HASH 69 unmoved with an empty `git diff` over the
  whole of `crates/engine/src` *and* `crates/card-types/src`, not merely `protocol.rs`/`hash.rs`;
  coverage 1,140 → **1,142** (63.2% → 63.3%); tests 3,988 → **3,998**; clippy / `fmt` /
  `tools/check-defs-fmt.sh` (1,804 defs) all clean; PB-DP10's `decision_gate` suite green, so
  neither new `Complete` def introduces an unrecorded engine-made choice. Two smaller honesty items
  were fixed rather than left: `Effect::SearchLibrary`'s `reveal: true` is **inert** (the engine
  destructures `reveal: _`; pre-existing **OOS-DP9-9**) and now carries an in-def comment saying so,
  because a `Complete` marker should not silently cover an unimplemented printed clause; and the one
  line-number cite in the plan that ignored the plan's own "cite by symbol" instruction was
  corrected to a symbol. **New seed OOS-DX3-1, and its first member is live-wrong on a deck-legal
  card.** `pb-plan-DP6.md:395` filed six more defs in this same "the DSL lacks the variant" bucket;
  three are authorable today. **`jadar_ghoulcaller_of_nephalia` is `Completeness::Complete`** —
  deck-legal — with `intervening_if: None`, so it creates a 2/2 Zombie **every** end step
  unconditionally; and its stored `oracle_text` says "if you control no tokens named Shambling
  Ghast" while MCP says the printed text is "if you control no **creatures with decayed**", so its
  blocker note ("`Condition::NoTokensNamedX` does not exist") is chasing a filter the card never
  had. Expressible now as `Not(YouControlNOrMoreWithFilter { count: 1, filter: Creature +
  has_keywords[Decayed] })`, with golden script `combat/191` (whose own generation notes are
  themselves stale) to reconcile. `ophiomancer` (`partial`; its note already says "Blocker stale")
  and `dwynen_s_elite` (`inert`) are two more flips in the same shape. **The generalisation worth
  carrying past these nine defs: a blocker note records what the DSL could express on the day it was
  written, and nothing re-reads it when a later batch adds the variant** — so "blocked on a DSL gap"
  is a *dated* claim, and a corpus-wide re-check of every `TODO: … DSL gap` / `Blocker stale` note
  against the current `Condition` enum is a cheap standing sweep rather than a per-card accident.
  **Next: PB-DX4** (OOS-DP10-8, the 97-entry `BASELINE` triage), with OOS-DX3-1's Jadar half a
  candidate to insert ahead of it.

- 2026-08-01 — **PB-DX2 SHIPPED (`scutemob-162`): `Command::ChooseDredge` no longer mints a free
  card, and a golden script that had been quietly exercising the exploit is fixed.**
  `Command::ChooseDredge` had NO pending-state gate anywhere: `rules/engine.rs`'s admission arm
  checked only `validate_player_exists`, and `handle_choose_dredge` validated the *card* (graveyard,
  `Dredge(n)`, library ≥ n) but never that a draw was actually outstanding — so `card: None` drew a
  free card for **any player, at any time**, and `card: Some(x)` dredged at will. **Fix, design (b)
  exactly as the brief predicted**: reuse the EXISTING `pending_draws: Vector<PendingDraw>` queue
  rather than inventing new state. `perform_one_draw`'s `DredgeAvailable` arm now records a
  `PendingDraw` entry at the offer site; `handle_choose_dredge` requires-and-consumes that entry
  before doing anything else, with a dead-player discharge guard as step 0. **The implement phase
  FOLDED a second offer into an outstanding entry, and the review's HIGH was that this made the
  entry a draw bank** — `remaining` accrued one per draw step with nothing reaping `pending_draws`
  and no timing gate on `ChooseDredge`, so seven turns of unanswered offers could be cashed as seven
  cards in one command during another player's combat, while the doc *this batch wrote* said an
  unanswered offer meant the draw simply never happened. Fix cycle 1 replaced the fold with an
  unconditional **discharge**: `perform_one_draw` now plays out any stale entry for that player, as
  an explicit decline would, at the top of the function before checking what the new draw needs.
  That bounds the queue, conserves every draw, and needed no new state. The standalone
  `draw_card_skipping_dredge` helper is deleted, its call folded directly into the gated decline arm
  — and every one of the 7 other prose references to that now-dead name across `replacement.rs`,
  `effects/mod.rs`, `card-types/state/replacement_effect.rs`, and two test files were updated so `rg
  -n 'draw_card_skipping_dredge' crates/` returns zero, not merely the production code. **A second
  bug, found in planning and not named by the seed, was fixed in the same edit**: `Effect::DrawCards
  { count: 3 }` with a dredge card in the graveyard emitted ONE `DredgeChoiceRequired` per remaining
  draw (three prompts) and drew ZERO cards, because `draw_cards_for_player`'s sequence loop didn't
  know a `DredgeOffered` outcome meant "stop" — CR 614.11a says the sequence stops at the
  replacement and resumes after it completes, exactly like the sibling
  `Deferred`/`LostToEmptyLibrary` outcomes already do. Fixed by adding `DredgeOffered` to the break
  set and extracting `resolve_pending_draw`'s CR 614.11a tail loop into a shared
  `perform_remaining_draws` helper `handle_choose_dredge` now calls too. **Wire-neutrality was a
  hard acceptance criterion (AC 5873), not a fallback**: `PROTOCOL_VERSION` stayed 32 and
  `HASH_SCHEMA_VERSION` stayed 69, confirmed by an EMPTY `git diff` over `rules/protocol.rs` and
  `state/hash.rs` and by the `core` test group's `hash_schema::*`/`protocol_schema::*` suites (36
  tests) all green — no new type, no new `GameState` field, no new `Command`/`GameEvent` variant.
  **Two riders closed in the same batch.** OOS-DP2-1: `handle_keep_hand` validated only the COUNT of
  `cards_to_bottom`, so a malformed or hostile command could bottom a permanent from the
  battlefield, a card from a graveyard, or a card from **another player's hand** — fixed with a
  per-entry `expect_zone(&ZoneId::Hand(player))` membership check plus a duplicate-id rejection,
  both before any mutation (`bare_lookup_ratchet`'s ceiling for `commander.rs` unmoved —
  `expect_zone` is not a bare lookup). OOS-DP9-14: a `pending_effect_choice` whose owner has
  conceded/lost is a trap state nothing outside `handle_concede` clears — closed defensively
  (unreachable through legal commands today, but the residue would be unrecoverable if a future
  admission-gate widening ever let an SBA reach it) with a narrow reap above
  `resolve_top_of_stack`'s entry `debug_assert!`, pinned in both directions by two in-src tests (a
  dead-owner entry is cleared and resolution proceeds; a live-owner entry still trips the assert —
  proving the reap did not swallow the real detector). **OOS-DP7-2 closed as a documentation fix,
  not a behaviour change**: FIVE doc sites needed reconciling, not the two the seed named — the
  third, `events.rs:1354`'s `CleanupDiscardChoiceRequired` doc, cited this exact seed as "not
  implemented" and would have become a NEW lying comment the moment PB-DX2 shipped if left alone.
  `MiracleRevealChoiceRequired`'s "same shape and the same suspicion" was VERIFIED false this batch
  (not merely suspected), and its live CR 702.94a violation (a miracle card need not be the one just
  drawn) is seeded separately as OOS-DX2-1 rather than fixed, because closing it needs a record of
  the just-drawn object — new stored state, a HASH bump, out of scope for a wire-neutral batch.
  **The batch's headline surprise came from actually running the golden corpus, not from reasoning
  about it.** The plan traced golden script `replacement/014_golgari_grave_troll_dredge.json`'s
  action sequence and predicted it would stay green untouched, same as every existing `dredge.rs`
  unit test — and every unit test did stay green, but the golden script did not: its `type:
  turn_based_action, action: draw_card` entry is, and always was, PURELY INFORMATIONAL per
  `script_schema.rs`'s own documented contract (no driver dispatches an engine `Command` off a
  `TurnBasedAction`'s `action` field), and its `initial_state` started already inside the Draw step
  with no step-entry transition ever run — so the script never actually attempted a real draw before
  dredging, and its `choose_dredge` succeeded pre-PB-DX2 purely because nothing gated it. **Fixed,
  not weakened**: the script now starts at Upkeep and a leading `priority_round` (both players pass)
  drives the REAL Upkeep→Draw transition and its CR 504.1 turn-based action, mirroring
  `crates/engine/tests/mechanics_a_d/dredge.rs`'s own `pass_all` unit-test helper exactly; a new
  append-only dispute entry documents the finding with CR citations, and the pre-existing dispute
  record (the original harness-gap fix from 2026-02-26) is untouched. **Card yield exactly as
  predicted**: `all_cards()` + `effective_abilities(both faces)` finds exactly ONE `Complete` dredge
  card in the whole 1,804-def corpus (`golgari_grave_troll.rs`, `Dredge(6)`) — 0 flips, 0 def edits,
  a permanent enumeration gate for one card would be theatre. **The durable lesson is what the
  SECOND review found, and it is the sharpest thing in the batch: the fix cycle's own repair shipped
  a false proof.** Having replaced the fold with the discharge, the runner marked a seed
  (`OOS-DX2-3`, "two `PendingDraw` entries per player") **CLOSED** on the argument that both
  `pending_draws.push_back` sites live inside `perform_one_draw` downstream of the discharge, so a
  second entry is *"structurally impossible — not bounded, but literally zero."* **That is a claim
  about where the pushes are, not when they run.** `resolve_declined_pending_draw` re-enters
  `perform_one_draw`; the inner call's discharge check finds the queue already emptied by its caller
  and skips, but its own `check_would_draw_replacement` can independently return `NeedsChoice` — CR
  616.1f excludes only replacements that were *applied*, not merely offered — and pushes a fresh
  entry, after which the outer call pushes its own. A re-review of the fix cycle caught it, and the
  runner **reproduced it empirically before writing anything** (one extra `draw_card` on the
  existing T19 fixture → `pending_draws().len() == 2`). The seed is **REOPENED**, the corrected
  invariant ("at most one *dredge-originated* entry per player") replaces the false one at all seven
  sites that asserted it — including two FIFO arguments and a termination proof that had been
  *assuming the thing it needed to prove* — and the real count is now pinned by a test rather than
  by prose. **No engine behaviour changed and no wire moved: the record was wrong, not the code**,
  and corpus exposure is zero (no card def registers a `WouldDraw` replacement at all). That a batch
  whose entire subject is doc-vs-code seeds produced a false proof in its own repair is the point
  worth carrying. **Tests 3,955 → 3,978** across implement (+16) and two fix cycles (+3, +4) in this
  worktree (T1-T16 matching the plan's full roster, plus T17/T18/T19 for cross-player and cross-kind
  consumption, the queue-growth reproduction, a discharge-then-`Proceed` event test verified
  non-vacuous by injecting the exact regression, and two probes restoring coverage to
  `handle_choose_dredge`'s `Some`-arm validations; T12 asserts on the error MESSAGE naming the
  duplicate, not `is_err()`, because `[a,a]` already errored pre-fix via `ObjectNotFound` — CR
  400.7, the first move mints a new id — so an `is_err()`-only probe would have been vacuous);
  benches within noise of the merge base (`full_turn_4p` 229.1 µs base vs 219.6-254.8 µs across 3
  runs on the branch, high ambient noise, one run showing an *improvement*). Seeds **OOS-DX2-1..7**
  filed in audit §8.1 (DX2-3 filed, wrongly closed, and reopened within the same batch; DX2-7 filed
  by the re-review — the discharge is itself a new engine-made auto-decline, recorded in no
  decision-point row and *not* outcome-neutral, since a discharged draw takes the library top at the
  later moment); four seeds CLOSED (OOS-DP5-7, OOS-DP7-2, OOS-DP2-1, OOS-DP9-14). **Cite hygiene,
  twice-failed and then fixed structurally**: both stale cites the plan flagged in advance were
  corrected on closure, but the re-review then found **two fresh drifts inside those very
  corrections** (a row claiming a line number had been "re-verified" was already off by two), so the
  affected seeds now cite by **symbol**, not line — a line number in a doc-heavy batch is stale the
  moment the next paragraph is written. **Next: PB-DX3** (OOS-DP6-3 — 2 flips, `garruks_uprising` +
  `inventors_fair`, 0 engine lines).

- 2026-08-01 — **PB-DX1 SHIPPED (`scutemob-160`): the PB-DX queue is open, and Aurelia's
  infinite-combat loop is closed.** `build_face_ability_vectors` (`testing/replay_harness.rs`) — the
  universal card-def → runtime lowering, reached from `rules/face.rs`, `rules/resolution.rs` *and*
  `enrich_spec_from_def`, so not test-only — hardcoded `intervening_if: None` at all **34** push
  sites, and both the queue site and the resolution site read that runtime field. CR 603.4 was
  therefore checked in **neither** place, and `aurelia_the_warleader` — `Complete` by `#[default]`,
  deck-legal, widely played — granted herself unbounded extra combats. **Fix (a), realised as a
  variant** — `InterveningIf::CardDef(Box<Condition>)` — **not a field on `TriggeredAbilityDef`**: a
  field would have forced `None` at ~140 struct literals across 84 files and left a permanent "did
  you read the other field too?" hazard at every read site, whereas the variant costs zero
  construction-site churn, makes an unclassified case a compile error, and repairs all 13 queue call
  sites plus the 1 resolution site at once because every dispatch path already funnels through
  `check_intervening_if`. **Both rejected alternatives were rejected on evidence, and one of the
  arguments is worth keeping.** (b) — a `CardDefETB`-style registry re-read — would have discarded
  `layers::expect_characteristics`, so Humility and Dress Down would have stopped suppressing all 34
  trigger events: **a CR 613.1f regression larger than the bug being fixed**, plus the loss of every
  runtime filter the lowering exists to carry, plus breakage on tokens and copies, which have no
  `card_id`. (c) — index correspondence between the two ability lists — was rejected with evidence
  stronger than the seed's own: `replay_harness.rs:2642`/`:2781` already carry an *"Index-namespace
  fix (2026-07-09)"* comment recording that **this exact trick on this exact function** was the root
  cause of the Monastery Mentor / Leaf-Crowned Visionary filter-bypass bug, and the fix applied then
  was (a). The codebase had already tried (c) here and shipped it as a bug. A three-valued
  `InterveningIfMoment { TriggerTime, TriggerTimeLookBack, Resolution }` (plus a review-added
  `ResolutionLookBack`) classifies the 14 call sites, independently re-derived from the expression
  supplying `source` at each — twice, by two different agents, agreeing 14/14. **PROTOCOL 31 → 32
  AND HASH 68 → 69: the dispatch brief's "HASH only" prediction was half wrong**, because
  `Characteristics` is listed in `protocol_schema.rs`'s `CLOSURE_MUST_CONTAIN` and
  `Characteristics.triggered_abilities: Vec<TriggeredAbilityDef>`, so both types were in the wire
  closure all along — **planning predicted the correction and stated the falsifier in advance**,
  which is the AC-5040 gate-compute discipline working rather than failing. **Two things the brief
  did not contain.** (1) **`once_per_turn` is dropped by the same lowering** at 31 of 34 sites, and
  three `Complete`, deck-legal defs were over-firing — `welcoming_vampire`, `elvish_warmaster`,
  `whispering_wizard`; `elvish_warmaster`'s is a *self-reinforcing cascade*, since the Elf token it
  creates re-qualifies its own trigger. Wire-neutral, found in planning, fixed in the same batch. A
  third dropped field, `trigger_zone`, is seeded (OOS-DX1-3), and a lossy-lowering table is now a
  module comment on the function specifically so a fourth is not discovered the same way. (2) **The
  review's single HIGH was a regression this batch introduced, and it is the durable lesson**:
  `aurelia_the_warleader`'s `Condition::IsFirstCombatPhase` (`!turn.in_extra_combat`) is a **proxy**
  for "attacks for the first time each turn", not a translation — and once PB-DX1 made the condition
  actually *evaluate*, the proxy began **suppressing** a legitimate trigger (her first attack of the
  turn occurring in a later combat granted by Aggravated Assault / Moraug / World at War / Port
  Razer), which is the one direction PB-DP6's hard constraint 3 forbids. The plan had deliberately
  deferred re-authoring her, reasoning it "would change which mechanism T1 exercises"; **the
  reviewer falsified that rationale using the batch's own T12b**, which already drives the identical
  intervening-if × attack × extra-combat shape on a real `Complete` def. Re-authored as
  `once_per_turn: true` with no `intervening_if` — expressible *only* because this batch's own
  `once_per_turn` rider shipped. **Karlach is the genuinely different case and keeps her
  intervening-if**: her printed text says "if it's the first combat phase of the turn" literally.
  **Generalised: fixing an engine gap can convert a dormant def-level approximation into a live bug,
  in the suppressing direction** — and the batch contains a second instance,
  `tatyova_steward_of_tides`'s `ControlAtLeastNOtherLands(6)`, which means 6 where oracle says seven
  (the `other` excludes `ctx.source`, and the source is a Merfolk Druid, not a land); inert while
  the condition was discarded, live afterwards, corrected to `(7)` in-batch. **Yield honesty: 1
  flip, not the brief's 2** — `karlach_fury_of_avernus` `known_wrong` → `Complete` on a four-point
  oracle verification (MCP ruling #11 verbatim: *"Karlach doesn't have to be among the attacking
  creatures"*), while **`tatyova_steward_of_tides` stays `partial`** with two untouched blockers.
  Both named riders **fixed, not deferred**, and **both their §8.1 cites were stale and corrected on
  closure** (`resolution.rs:7369`→`:7564`, `:5351`→`:5494` — the OOS-DP6-8 documentation-rot class,
  caught only because the plan mandated re-verifying every cite): **OOS-DP6-5** gains its
  resolution-time re-check, and **OOS-DP6-9**'s haunt gate turned out worse than filed — the *queue*
  end had never consulted the card def at all, so there was nothing to "ignore". Review 1 HIGH / 5
  MEDIUM / 4 LOW, **all 10 applied**, including two the runner corrected rather than complied with
  (the review's suggested one-line `haunting_target` clear was impossible — `check_triggers` takes
  `&GameState` — so the whole gate moved to `flush_pending_triggers`; and the review named a
  function `flush_sorted` that does not exist). Tests 3,928 → **3,945**; benches within 5%
  (`full_turn_4p` 214.6 → 217.4 µs); seeds **OOS-DX1-1..6** filed in audit §8.1 (DX1-5 filed
  *closed*, DX1-6 half-fixed). **Next: PB-DX2** (`ChooseDredge` has no pending-state gate — a free
  card for any player at any time; wire-neutral), then **PB-DX3** (2 flips, 0 engine lines).

## 2026-08-01 — M11-local COMPLETE (`scutemob-173`); the full session-by-session narrative

The verbatim "Active Milestone" bullet as it stood at milestone close, with the
S1..S7 detail that no longer belongs in a snapshot. The S8 close-out and the durable
lessons are in `memory/workstream-state.md`'s S8 handoff and in
`docs/mtg-engine-roadmap.md`'s M11-local section.

- **Active Milestone**: **M11-local ACTIVE (web-first)** — first-playable track, running **in parallel** with the RS correctness queue below (resumed 2026-07-26; RS4 shipped). Roadmap restructured 2026-07-26 (`scutemob-147`) to apply the 2026-03-07 strategic review: **M11 decoupled from M10** and redefined as **M11-local** (browser client + 1 human + 3 simulator bots, **no networking**; deps M9 + M9.5 only), **M10 split into M10-pre / M10a / M10b**, **M12 downscoped** from a pipeline-crate milestone to a continuous agent-authoring track that gates nothing. **UI stack decided: WEB-FIRST** (user, 2026-07-26, `memory/decisions.md`) — extend the axum + Svelte 5 replay-viewer stack; Tauri v2 is a later *packaging wrapper* option, not a parallel framework. Plan: `memory/m11-session-plan.md` (8 sessions; central call is a **steppable driver**, not a channel-backed `Bot`, because `driver.rs` answers a rejected command with a silent `PassPriority` and every sub-decision is already a field of the returned `Command` — so **no new `Command`/`GameEvent` variant in the whole milestone**). **S1 SHIPPED** (`f2a9647b` + review fixes — `LocalGame` in `crates/simulator/src/local_game.rs`: `advance()` runs bot seats and yields `AwaitingHuman`, and is idempotent while that decision is unanswered; `submit(seq, …)` never falls back to `PassPriority`, leaves state untouched on rejection, and refuses a command naming another seat; `GameDriver::run_game` re-expressed on top of it; 10 tests; `crates/simulator` only, PROTOCOL 27 / HASH 63 unmoved). **S2 SHIPPED** (`scutemob-161` — `crates/simulator/src/setup.rs`: deterministic seeded `build_initial_state` + pregame `redeal`, deck admission through the real `validate_deck`, and a live Commander bug fixed in the lifted TUI setup — `PlayerState::commander_ids` had been **empty** in every game the TUI built, so tax, commander damage and the CR 903.9a zone-return SBA never fired). **S3 SHIPPED** (`scutemob-163`, 2026-08-01 — **the milestone's crux, plan §8 R1, is closed: a human can cast a targeted spell**, proven by a test that picks its target through the new engine query surface and asserts the damage resolved). Engine: new read-only `crates/engine/src/rules/queries.rs` (4 fns, re-exported from `lib.rs`, **no new public type**), plus three shared helpers extracted verbatim from `handle_cast_spell` so the query and the cast path cannot drift; `legal_targets_per_slot` delegates one `validate_targets_inner` call per candidate rather than re-deriving hexproof/shroud/protection. `spell_target_requirements` ships a **4th parameter the plan's sketch omits** (`alt_cost: Option<AltCostKind>`) because Overload and Aftermath are caster-intent flags, not state. Simulator: `params.rs` is now the **single** `LegalAction`→`Command` mapping table (hybrid/Phyrexian plans forwarded verbatim per PB-RS2; `any_color` `TapForMana` without `chosen_color` rejected, not defaulted to Colorless), `HumanChoice` became a struct so a cross-seat command is **structurally unrepresentable**, and auto-tap is conditional on the pool — **`OOS-M11-2`'s pool half CLOSED**, its layer-resolution half still open. Bot parity *measured* (50 fuzzer seeds byte-identical), not asserted. Tests 3,955 → **3,965**; PROTOCOL 32 / HASH 69 unmoved. **S4 SHIPPED** (`scutemob-165` — `crates/view-model` (`mtg-view-model`): a seat view provably cannot leak another hand or any library order). **S5 SHIPPED** (`scutemob-167` — `tools/play-server`, axum on port 3040, the only crate in this milestone with async or IO; 5 routes + `ServeDir`, 16 tests, **no port ever bound and machine-gated crate-wide**; a full game is playable over `curl` alone). **S6 SHIPPED** (`scutemob-169`, 2026-08-01 — `tools/play-server/frontend`: Svelte 5 runes + Vite 7, eight source files, `$viewer` aliased at the replay viewer's `src/lib` so `PhaseIndicator`/`StateView` and the whole `Zone*` tree are compiled **in place, not copied** (checked both ways: eight files in our tree, the viewer's scoped CSS in our bundle). **Zero Rust** — `git diff main` over `crates/` + `tools/play-server/src` + `tools/play-server/Cargo.toml` is empty — **zero Rust anywhere**; the only change outside `tools/play-server` is one Svelte component, `tools/replay-viewer/frontend/src/lib/ZoneHand.svelte` (the review HIGH below); PROTOCOL 32 / HASH 69 unmoved; tests **4,040 / 0**, unchanged from the merge base by construction because the plan gives this session no test target. The manual checklist was **run, not asserted**: a temporary `#[ignore]`d probe in `main.rs`'s existing `mod tests`, driven through `oneshot` binding **no port** and then removed, dumped real `SeatView` payloads — 7-card hand, a land drop moving hand 8→7 and battlefield 0→1, 25 passes, 10–21 redacted event lines per response, a non-empty stack, turn 4 reached — and the two steps that genuinely cannot be checked headlessly (launching the binary; keyboard and DOM events) are marked unverifiable in the README rather than glossed. **The review's HIGH is the durable lesson and every gate this session had was green while it was live**: `ZoneHand.svelte` keyed its `#each` on `card.object_id`, which is right for the omniscient viewer and **fatal** for a seat-redacted payload — `redact::redact_hands` gives every unreadable hand card `object_id: 0`, so three bot hands of seven arrived with one distinct key each, Svelte 5 evaluates `length > keys.size` and calls `each_key_duplicate`, which **throws in production as well as DEV**, and with no `<svelte:boundary>` the throw took the whole mount down — **the play surface rendered nothing at all** while the build was clean, the Rust diff empty and 4,040 tests green. Caught by evaluating Svelte's own condition against the dumped hands (`7 > 1` per bot seat), not by a build and not by a browser; fixed **in the shared component** (`card.hidden ? hidden-i : card.object_id`), which is inert for the viewer and is exactly why the plan aliases the component instead of copying it. Generalisation for S7: **the viewer's components were written against an omniscient view model, and every id-uniqueness assumption in them is now a claim about the redacted one too.** A second review MEDIUM: `DeclareAttackers`/`DeclareBlockers` submit an **empty set** silently (`params.rs` maps default params to `attackers: vec![]`) — legal, irreversible, and quieter than the targeted-spell 422; the buttons stay enabled because disabling them would deadlock a combat, but are marked `declares none`. A third: an **activated ability's `{X}` is announced as 0** and the client cannot tell which abilities have one (`action_needs_x` answers `CastSpell` only), which is reachable and destructive on deck-legal **`mirror_entity`** — no `completeness` field, so `Complete` by the `#[default]` derive again, and `x_count: 1` means one click makes every creature 0/0. All three are the same hole (the client can only send `params: {}`) and only the first fails loudly. Three findings the plan did not contain: the **mulligan `LegalAction`s are unreachable** on this surface (`turn_number == 0` is unsatisfiable — `session.rs::is_pregame` says so, and the payload confirms `kind: "Priority"`), so the UI uses the dedicated route and tracks "keep" client-side because the server records it nowhere; the **redactor rewrites a hidden card's `object_id` to 0** (569 such entries in one playthrough, all id 0, against a lowest real action id of 2), so click-through refuses `hidden` cards rather than matching a sentinel; and **`ZoneStack` declares `onCardClick` and never invokes it** — harmless in S6, load-bearing for S7. A targeted spell cast from this UI fails with a real, observed 422 (`invalid target: expected 1..=1 target(s) but got 0`), which is correct under CR 601.2c until S7 populates `target_slots`.) **Next: S7** (targeting, combat and choice UIs — the first session since S4 to touch Rust again). New seeds from planning/S1: **OOS-M11-1 CLOSED** by PB-DP2 (`scutemob-150`) — `handle_take_mulligan` emitted `GameEvent::LibraryShuffled` while permuting nothing; fixed in-engine with the existing seeded `zone.shuffle()` pattern, no wire change, and widened per audit §7 to also cover `handle_keep_hand`'s top/bottom inversion. Still open and **not yet ranked into the RS queue**: **OOS-M11-2** (`solve_mana_payment` ignores the mana pool and reads non-layer-resolved `mana_abilities`), **OOS-M11-3** (fuzzer not run-to-run deterministic for 150-200+ turn games; reproduces on pristine code; bears on Tier 1 hashing and M10a).

**S8 (`scutemob-173`) closed the milestone.** Its handoff — the scripted playthrough,
the four defects running it uncovered, the two engine-level seeds it filed rather than
fixed (`OOS-M11-7`, `OOS-M11-9`), the stale item-2 premise, the measured fuzz parity,
and `OOS-M11-8`'s closure — is in `memory/workstream-state.md` under
**"S8 handoff — MILESTONE CLOSE"**, and the fuzz evidence in
`memory/m11/s8-fuzz-parity.md`.


## Campaign-queue history (rotated from the Current State campaign bullet, 2026-08-02)

**PB-DX1 SHIPPED** (`scutemob-160`, 2026-08-01 — OOS-DP6-1 + riders OOS-DP6-5/DP6-9 all CLOSED;
PROTOCOL 31→**32** / HASH 68→**69**, *not* the brief's predicted HASH-only; 1 flip (`karlach`),
Aurelia repaired, 3 more defs stop over-firing via the unpredicted `once_per_turn` half; tests
**3,945**). **PB-DX2 SHIPPED** (`scutemob-162`, 2026-08-01 — OOS-DP5-7 + OOS-DP7-2 + riders
OOS-DP2-1/OOS-DP9-14 all CLOSED; `Command::ChooseDredge` gated by requiring-and-consuming a
`PendingDraw` entry (design (b), reused the existing queue — no new type/field/wire variant); a
second CR 614.11a multi-draw sequence bug fixed in the same edit; `handle_keep_hand` gained a
per-entry hand-zone guard; a dead-owner `pending_effect_choice` trap state reaped defensively;
**PROTOCOL 32 / HASH 69 both UNMOVED**, confirmed empty `git diff` over
`rules/protocol.rs`/`state/hash.rs`; 0 flips, 0 def edits — roster is exactly 1 `Complete` card
(`golgari_grave_troll`); a genuine surprise found only by running the full golden corpus:
`replacement/014_golgari_grave_troll_dredge.json` relied on the exact exploit this batch closes (its
`turn_based_action: draw_card` label is purely informational and dispatches no engine Command, so
the script never actually attempted a real draw before dredging) — fixed with a real Upkeep→Draw
`priority_round` transition rather than by weakening the assertion; tests **3,971** in the worker's
worktree). **Fix cycle same day (`scutemob-162`)**: review found the implement-phase "fold guard"
was itself a HIGH — it turned an unanswered offer into an obligation that accumulated without bound
across turns and could be cashed in one command at an arbitrary later moment, out of priority, while
the batch's own new doc denied it. **Fixed by replacing the fold with a discharge**:
`perform_one_draw` now auto-resolves (as an implicit decline) any stale entry for a player the
instant another draw arrives for them, unconditionally, before even checking what the new draw needs
— bounding `pending_draws` to the single most-recently-offered draw's own remainder and, as a
found-not-prescribed side effect, closing **OOS-DX2-3** (two entries per player) completely, since
both push sites are now downstream of the discharge. 7 doc-vs-code MEDIUMs (including a
doc-comment-capture bug that stole `resolve_pending_draw`'s doc block — the exact OOS-DP7-2 failure
mode, reintroduced by the batch closing it) + 1 coverage-hole MEDIUM (`dredge.rs` test 9 had
silently degraded to testing the entry gate instead of what its name claimed) + 7 LOWs all applied.
PROTOCOL 32 / HASH 69 confirmed still unmoved; tests 3,971 → **3,974**. **PB-DX3 SHIPPED**
(`scutemob-164`, 2026-08-01 — **OOS-DP6-3 CLOSED**, exactly as scoped: `garruks_uprising` +
`inventors_fair` both `partial` → **`Complete`**, coverage 1,140 → **1,142** (63.2% → 63.3%), tests
3,988 → **3,998** (+10 probes), **0 engine lines** — an empty `git diff` over the whole of
`crates/engine/src` *and* `crates/card-types/src`, not merely the two wire files — and PROTOCOL 32 /
HASH 69 unmoved. Review 0 HIGH / 1 MEDIUM / 5 LOW, all applied; both flips ruled justified
clause-by-clause against MCP oracle text and all eight rulings. **Three things the queue row did not
contain.** (1) `inventors_fair`'s upkeep trigger **did not exist at all** in the def — the seed and
both blocker notes read as though it were present but ungated, so the batch had to *author* the
ability rather than add its `intervening_if`. (2) The runtime `InterveningIf` enum both notes name
now has **three** variants, not the two they cite: PB-DX1 added `InterveningIf::CardDef` two batches
earlier. The notes were stale twice over, and this queue introduced the second staleness itself. (3)
**The MEDIUM was the batch reproducing its own subject.** The new test module recorded a pre-fix
observation for T1 — "the hand count was 1" — that **could not have been observed** against T1's own
fixture, which had no library object; an empty-library draw sets `has_lost` rather than incrementing
the hand, and the companion assertion passed whether or not the bug fired. Fixed by giving T1 a real
library card and **re-running the pre-fix scenario empirically** — reverting `intervening_if` to
`None` and reading the numbers — not by repairing the prose; the same standard was then applied to
T3/T5/T6/T7/T8, all of which held. The original claim was right; it had simply never been checked
against a fixture where the number meant anything, and that distinction is the lesson.
`Effect::SearchLibrary`'s `reveal: true` is inert (pre-existing **OOS-DP9-9**) and now carries an
in-def comment saying so rather than being silently covered by the `Complete` marker. **New seed
OOS-DX3-1**: six more defs sit in the same `pb-plan-DP6.md:395` stale-blocker bucket and
**`jadar_ghoulcaller_of_nephalia` is a live-wrong `Complete` def** — `intervening_if: None`, so it
makes a 2/2 Zombie **every** end step unconditionally, and its stored `oracle_text` names a
token-name filter the printed card never had (MCP: the real text is "if you control no creatures
with decayed"); expressible today as `Not(YouControlNOrMoreWithFilter{Creature + Decayed})`, with
golden script `combat/191` to reconcile. `ophiomancer` and `dwynen_s_elite` are two more flips in
the same shape.) **PB-DX3b SHIPPED** (`scutemob-166`, 2026-08-01 — the OOS-DX3-1 insert was taken
and **OOS-DX3-1 is CLOSED**: all **seven** remaining defs of the `pb-plan-DP6.md:395` stale-blocker
bucket dispositioned explicitly — four fixed, three deferred with blockers re-affirmed against the
*current* `Condition` enum rather than copied forward. `jadar_ghoulcaller_of_nephalia` stays
`Complete` and is now gated, and its stored **`oracle_text` was wrong**, not merely its blocker note
— the note had been chasing "tokens named Shambling Ghast", a filter the printed card never had.
`ophiomancer` `partial` → `Complete` (using `has_subtype: Snake` alone, deliberately **not** the
def's own suggested `ControlCreatureWithSubtype`, whose arm hard-requires `CardType::Creature` while
CR 205.3 reads "Snakes" = permanents with the subtype). `dwynen_s_elite` `inert` → `Complete`, its
ability **authored from nothing** — the `inventors_fair` shape from PB-DX3 recurring, and now a
pattern worth expecting: a stale blocker note reads as though the ability were present but ungated
when it is often absent entirely. **The headline is that the seed itself mis-dispositioned a
live-wrong `Complete` def**: `emeria_the_sky_ruin` sat in OOS-DX3-1's "genuinely blocked" pile, but
it declares **no `completeness` field at all**, so it was `Complete` by the `#[default]` derive —
deck-legal — and reanimated a creature every upkeep regardless of Plains count. That is the
`aurelia_the_warleader` trap from PB-DX1 hit a second time in three batches by a different route,
which makes **`#[default] Completeness::Complete` a twice-demonstrated silent-defect generator**;
"which defs never declare a marker at all?" is now a cheap corpus-wide question nobody has asked.
Emeria was gated and given an **explicit** `partial` marker for the printed "you **may** return",
which the DSL cannot express (searched: `MayPayThenEffect` needs a `Cost` and a free one always
trivially pays; `MayPayOrElse` and `Effect::Choose` are both barred from `Complete` by
`effect_choose_gate.rs`; PB-DP9's `pending_effect_choice` channel is search/scry/surveil-only) —
same class as OOS-DP10-8. A spurious `Legendary` supertype on Emeria was found and removed in the
fix cycle (MCP type line is `Land`; CR 704.5j would have wrongly applied). **Yield: 2 flips up, 1
honest flip down — net coverage 1,142 → 1,143, +1 not +3**, reported that way because the batch's
own plan said "+2" and that was its own arithmetic slip. **0 engine lines** (empty `git diff` over
the whole of `crates/engine/src` *and* `crates/card-types/src`), PROTOCOL 32 / HASH 69 unmoved,
tests 4,008 → **4,022** (+14 probes). Golden script `combat/191` reconciled by **strengthening**: it
had never asserted the Zombie token at all and passed whether or not the token existed — the same
vacuous-assertion shape PB-DX3's MEDIUM was about, sitting in the corpus. Review 0 HIGH / 5 MEDIUM /
7 LOW, **all 12 applied**; all four completeness moves independently ruled justified
clause-by-clause against MCP oracle text. New seed **OOS-DX3b-1**: `guardian_project`'s note is half
stale too — its `is_nontoken` half is authorable today (PB-AC0 wired `triggering_creature_filter`),
its name-uniqueness half genuinely is not, so the def stays `known_wrong`.) **PB-DX4 SHIPPED**
(`scutemob-168`, 2026-08-01 — **OOS-DP10-8 CLOSED**, and **OOS-M11-6 closed incidentally**. All 97
`BASELINE` entries read against MCP printed text, roster derived from the const array itself rather
than prose (97 → 97 distinct names → 97 unique def files). **Split 84 class-B / 13 class-D —
PB-DP10's 2-of-5 spot-check overstated the D rate roughly fivefold**, and its own "very noisy
sample" caution was right; both spot-check members held. **6 repaired in place, still `Complete`**:
`metastatic_evangel` (four defects at once — `{2}{W}`→`{1}{W}`, missing `Human` subtype, P/T
transposed 1/3→3/1, and a **stale** note claiming `is_token` is ignored on the ETB path when PB-AC0
had made that false — the PB-DX3/DX3b stale-note class reached by a different route),
`grisly_salvage` + `satyr_wayfinder` (`RevealAndRoute` routes ALL matches →
`LookAtTopThenPlace{optional:true}`; printed says "**a** card" and "you **may**"),
`sword_of_truth_and_justice` (bare `TargetCreature` → `controller: You`), `radstorm`
(`{2}{U}`→`{3}{U}`; a Storm card a mana cheap compounds into extra copies). **6 demoted with oracle
citations**: `smugglers_copter` → `known_wrong` (the 20th DP-12 instance where the other 19 already
were — the marker, not the encoding, was the defect), `contaminant_grafter` / `grateful_apparition`
/ `thrasios_triton_hero` → `partial`, and `shambling_ghast` → `partial` **for a defect the fix
surfaced rather than the ones it went after**: its three named deviations (phantom `Decayed`,
permanent `MinusOneMinusOne` counter for a printed "until end of turn", stored `oracle_text` saying
"enters" against `WhenDies`) were all fixed, and the marker is for a fourth — its mode-1 target is
declared flat, so taking the Treasure mode still needs an opponent creature (CR 603.3d).
`mode_targets` is honoured **only on the casting path**, never for triggered abilities, so the
obvious repair would have *dropped* the requirement rather than scoped it (OOS-DX4-2). **1 left
`Complete` deliberately**: `staff_of_compleation`'s "target permanent you own" as
`TargetController::You`, allowlisted to match the shipped `nether_traitor` decision for the
identical owner-vs-controller class — demoting the two members that happen to sit in the 97 would
have reported a corpus class as a pair of cards (OOS-DX4-1). **The batch also closed OOS-M11-6 by
accident**: demoting `thrasios_triton_hero`, a legendary creature and so a member of `random_deck`'s
own commander pool, shifted every seeded deck in the workspace and landed seed 9001 on Rograkh — the
corpus's ONLY colourless `Complete` legendary creature — exposing the CR 903.5c Forest padding that
the fuzzer had been silently *playing*, not merely refusing. Fixed the way the seed preferred (pad
from the identity-legal colourless pool; measured 40 colourless lands + 82 nonlands = 122 singletons
vs 99 needed), **both** Forest fallbacks removed. Golden script `baseline/112` **retired**: it
tested Decayed on a card that does not have it, citing the card def as its authority — provenance
failure, not a stale assertion; CR 702.147a keeps 12 unit tests, and the golden-level gap is
OOS-DX4-3 rather than silence. Coverage 1,143 → **1,137** (63.0%), tests 4,040 → **4,048**,
`BASELINE` 97 → **91** (it moved three times inside the batch, 97→93→92→91, which is why it was read
off the gate rather than computed), deviation floor 661 → **667**, DP8 roster 76 → **74**, `scry` 16
→ **15** — each re-measured against `all_cards()`. 0 engine lines (empty `git diff` over
`crates/engine/src` *and* `crates/card-types/src`), PROTOCOL 32 / HASH 69 unmoved. **PB-DX3b's open
`#[default]` question answered, and bigger than expected: 966 of 1,804 def files never mention
`completeness` at all (970 before this batch)** — a clear majority of the `Complete` population, and
eleven of the thirteen class-D defs were in it; now ratcheted in the growth direction. Durable
record: `memory/primitives/pb-dx4-baseline-triage.md`; seeds OOS-DX4-1..6.) **PB-DX5 SHIPPED**
(`scutemob-170`, 2026-08-01 — **OOS-OS7-2 CLOSED: CR 611.2c is implemented.** `ContinuousEffect`
gains `affected_set: Option<OrdSet<ObjectId>>`; `Some` = resolution-generated, and
`effect_applies_to` answers by **membership alone**, never re-consulting
`filter`/`chars`/`obj_zone`; `None` = static ability (CR 611.3a — genuinely not locked in), which
keeps its live re-evaluation. Populated at exactly one site, `Effect::ApplyContinuousEffect`, via
the new `rules::layers::snapshot_affected_set`, called before the push so
`calculate_characteristics` cannot see the effect being created. **`is_effect_active` deliberately
NOT changed** — against the brief and the task's own criterion, which name both: it takes no
`object_id`, so a per-object set is not expressible there, and an effect with an empty locked set is
still *active* (CR 611.2b describes an outcome, not non-existence); ruled correct in review and
pinned by a test. **The dispatch row's roster was wrong twice over — the sixth consecutive batch in
this suite whose published roster was wrong before it started**: from `all_cards()`, **116** defs
generate a resolution-time continuous effect and **38** use a mass filter (**29 `Complete`**, 8
`partial`, 1 `known_wrong`), not "9 defs / 7 `Complete`". The grep conjunction missed the whole
`CreaturesYouControl*` family — 27 defs, including Craterhoof Behemoth, Purphoros, Mirror Entity,
Triumph of the Hordes, Unbreakable Formation — because the filter name does not begin with `All`,
and it counted `elvish_dreadlord`, whose only `ApplyContinuousEffect` mention is inside a
**blocker-note string**. **Three arithmetic slips were then caught inside the batch itself** — the
premise phase's 37/28, the plan's own table (which already summed to 38/29), and the implement
phase's test count (+16 against a true +17) — each by re-running the measurement, none by re-reading
the prose. **The batch closed a second and larger defect it did not know about until review
(OOS-DX5-7)**: `effect_applies_to`'s source-relative arms require the source object to still exist,
and for an instant or sorcery `ctx.source` is the spell's card, which `resolve_top_of_stack_inner`
moves to the graveyard **after** effects run (CR 400.7, new object) — so pre-fix *Triumph of the
Hordes*, *Unbreakable Formation*, *Goblin Surprise* and *Return of the Wildspeaker* applied to
**nobody at all** the moment they resolved, which is strictly bigger than the "newcomer wrongly gets
it" the seed described; verified empirically in the fix cycle, and it revealed that the batch's own
T12 had been mislabelled about what it demonstrated. **Fingerprints computed, not predicted**: HASH
69 → **70** (mandatory), append-only history row, 43 sentinels re-pinned by **symbol** grep — two of
which the single-line pattern could not see and only a full `--no-fail-fast` workspace run caught;
**PROTOCOL confirmed unmoved at 32** by running the gate, the falsifier the plan named in advance
(`ContinuousEffect` is outside the SR-8 wire closure, so PB-DX1's "anything reachable from
`Characteristics` is PROTOCOL too" was the reason to check and did not apply here). **0 flips,
exactly as pre-committed** — a pure engine fix that makes 29 existing `Complete` defs behave
correctly, so no marker moved and PB-DX4's seeded-deck re-deal hazard never fired. **One existing
test had been asserting the bug while citing CR 611.2c as its justification**
(`pb_ac3_dynamic_pt_counts.rs`, claiming the rule required filter *membership* to stay live while
only the *value* locked); inverted with the rule text quoted and **strengthened** to an exact value,
not loosened — no assertion in the batch was weakened. Review **0 HIGH / 6 MEDIUM / 6 LOW, all 12
applied**; every MEDIUM was the same shape, *a claim recorded as measured that had been reasoned
to*, and two of them had put a false "verified: none exist" into engine source —
`snapshot_affected_set`'s doc block asked the wrong question, since the Layer-≤4 divergence comes
from any effect that **writes** the characteristic the filter reads, and `inkmoth_nexus` does
exactly that (fixed, seed reopened, real test added). Probe discrimination was **verified
independently, not asserted**: with the membership read disabled, **8 of 15** probes fail and
exactly the 7 that must be insensitive stay green. Tests 4,048 → **4,066**; benches within ~1% (the
snapshot runs once per resolution, not per layer pass); seeds **OOS-DX5-1..8**.) **PB-DX6 SHIPPED**
(`scutemob-172`, 2026-08-02 — **OOS-RS2-1 + OOS-DP4-1 both CLOSED**: the last two unflattened
mana-cost payment sites now flatten. `handle_turn_face_up` paid a raw `def.mana_cost`, and it is
**all three** `TurnFaceUpMethod` arms that share the defective block, not the `ManaCost` arm the
brief named. `Command::DeclareAttackers` gains the two PB-RS2 payment fields, with pips **replicated
copy-major into the CR 508.1h total and the total flattened once** — design (B),
flatten-then-multiply, is *rules-wrong* on the Norn's Annex ruling that each cost is chosen
**individually**, and fails in the quiet direction by accepting the command and charging a
legal-but-not-chosen total. `unpayable_tax_defenders` → `x_tax_defenders`, narrowed to X only,
because a name asserting "unpayable" when hybrid and Phyrexian are now payable is a lying
identifier. New read-only `rules::queries::attack_tax_total` — the attack-tax cost is the one
payment cost a client **cannot** derive, since `LegalAction::DeclareAttackers` carries no attacker
set — and **exactly one** accumulation serves both it and the validation path, pinned by a test
asserting the query byte-identical to what the engine charged. `ManaPool::can_spend` is now
fail-**closed** on an unflattened residue in every build and `spend` asserts unconditionally: the
guard PB-RS2's own review described as firing "NEVER" in release was failing **open**, i.e. silently
**undercharging**. **PROTOCOL 32 → 33 computed** from the gate's own output (the falsifier named in
advance — "it passes unchanged" — did not occur; closure type count unchanged at 96); **HASH
confirmed unmoved at 70 by running the gate**. **0 completeness flips, pre-committed and held**
(empty `git diff` over `crates/card-defs`), so no seeded deck re-dealt and the play-server pins were
never touched. Tests 4,066 → **4,099**. **The review's HIGH is the durable lesson and it is on the
nose**: the copy-major order-pin test **could not fail** under the very permutation it existed to
catch (copy- and pip-major diverge only when one `add_mana_cost` call has `times > 1` **and** more
than one pip, which its fixture never produced), while the batch's own freshly-written doc asserted
that it could — the PB-DX5 "verified: none exist" class, reproduced inside the batch that cites it
twice. Fixed with a minimum discriminating fixture (one defender, **two distinct** restrictions, two
attackers) and proven by reverting to pip-major and watching the old test stay green and the new one
redden. **Second finding, and it is about damage this batch did to somebody else's tests**: PB-DP4's
two E1 CR 508.1c scoping pins both used a *hybrid* restriction, which stopped being a rejection
class the moment this batch landed — so E1 had silently lost **all** regression coverage; verified
by reverting E1 and watching them stay green, then moved to `x_count: 1`. Review 1 HIGH / 8 MEDIUM /
6 LOW, **all 15 applied**, each re-verified by execution first because the reviewer had no shell.
Seeds **OOS-DX6-1..5**; OOS-DP4-7 **re-dispositioned, not closed**, with the reason the seed lacked:
`multiply_mana_cost` is pip-major, so the proposed dedup would silently re-order the tax's pips.)
**Next dispatch: PB-DX7** (OOS-DP7-11 + OOS-DP9-13 — the SR-19 gate reports success while checking
nothing; gate integrity, 0 flips, test-only, no wire change).** The queue history below is a record,
not a to-do list. — **OS1..OS3 SHIPPED** (correctness group, `scutemob-116`/`128`/`129`), **OS4
SHIPPED-NARROWED** (`scutemob-130` merge `7ee96913` — `ExileSourceAndReturnTransformed`, PROTOCOL 19
/ HASH 56; 0 flips, fable partial; new seeds OOS-OS4-1/2 — **OOS-OS4-2 face-aware ability gathering
is a correctness candidate ahead of OS5**, may implicate PB-EF5 TransformSelf Completes), **OS4b
SHIPPED** (`scutemob-134` merge `77d411a0` — face-aware ability gathering, wire-neutral;
docent+bloodline verified Complete by execution; **OOS-OS4-2 now FULLY CLOSED** by PB-RS4
(`scutemob-146`) — the 3 surviving CR 712.8d/e residuals tracked as OOS-RS-3 are fixed; OOS-OS4-3
filed), **OS5 SHIPPED** (`scutemob-135` merge `de58b1cc` — relative-count EffectAmount, PROTOCOL 20
/ HASH 57; shared_animosity + goblin_piledriver Complete, OOS-EF4-1 closed), **OS6 SHIPPED**
(`scutemob-136` merge `63ca78ce` — flip-condition sub-batch, PROTOCOL 21 / HASH 58; delver +
legions_landing + thaumatic_compass Complete; OOS-OS6-1 filed), **OS7 SHIPPED** (`scutemob-137`
merge `e2dd4c1f` — defending-player continuous filter, PROTOCOL 22 / HASH 59; silumgar Complete;
OOS-OS7-1/2 filed), **OS8 SHIPPED** (`scutemob-138` merge `38246a6e` — LookAtTopThenPlace + min_cmc,
PROTOCOL 23 / HASH 60; birthing_ritual + growing_rites Complete; OOS-OS8-1/2 filed), **OS9 SHIPPED**
(`scutemob-139` merge `6800d924` — YouControlYourCommander, PROTOCOL 24 / HASH 61; skyhunter
Complete; OOS-OS9-1 filed), **THE PB-OS QUEUE IS COMPLETE — OS1..OS11 + OS4b ALL SHIPPED**
(`scutemob-116`..`141`; final: OS11 merge `bd220b00`, RemoveCounter lowering + filtered-attack
trigger, 6 flips). Coverage **1,135/1,804 = 62.9%**; PROTOCOL **26** / HASH **63**; tests 3560+.
**Rider-seed mini-triage DONE** (`scutemob-142`, merge `6f50b7f7`) — **NEXT QUEUE: PB-RS1..RS11**
(`memory/primitives/rider-seed-triage-2026-07-19.md` §3, correctness-first; 4 new correctness-class
findings — OOS-RS-1 library top/bottom inversion across the reveal/scry family (~47 files), OOS-RS-2
hybrid/Phyrexian pips free in activated costs, OOS-OS9-1 AtBeginningOfCombat sweep gap, OOS-RS-3
partial OS4-2 closure — outrank every previously filed seed; 2 live-wrong on `Complete` cards).
**PB-RS1 SHIPPED** (`scutemob-143` merge `56697a00` — `Zone::top_n` reconciliation across
Scry/Surveil/RevealAndRoute/LookAtTopThenPlace + bottom-writes, camp A CR-confirmed, 41-card roster
repaired, no wire bump; OOS-RS1-1 filed, muxus still gated), **PB-RS2 SHIPPED** (`scutemob-144`
merge `86176ff7` — hybrid/Phyrexian pip payment for activated + mana abilities,
`ActivateAbility`+`TapForMana` schema fields, PROTOCOL 26→**27**; birthing_pod inert→Complete,
OOS-OS8-1 CLOSED; 7 filter lands stop being free, stay `known_wrong`; CR 119.4 holes fixed).
**PB-RS3 SHIPPED** (`scutemob-145` merge `b1c21909` — `begin_combat` card-def sweep; 3 flips incl.
probe-earned goblin_rabblemaster + helm_of_the_host integrity repair; seeds OOS-RS3-1..4 filed,
RS3-1 rankable). Coverage now **1,139/1,804 = 63.1%**; PROTOCOL **27** / HASH **63**. **PB-RS4
SHIPPED** (`scutemob-146` merge `9419d0e9`, 2026-07-26 — face-aware residuals; **OOS-RS-3 CLOSED,
OOS-OS4-2 fully closed**: both `replacement.rs` gathering sites read the active face,
`deregister_face_statics` extended from 1 to all 10 registered families via
`remove_one_registration` + a source-scan parity gate, and a 4th same-root-cause deviation found in
planning — CR 714.3b Saga lore sweep + chapter `ability_index` namespace — fixed; **0 flips** as
predicted, 2 integrity repairs, 17 fail-before/pass-after probes; seeds OOS-RS4-1/2/4 filed;
PROTOCOL 27 / HASH 63 unchanged). **RS QUEUE HELD AGAIN (user directive 2026-07-26): the PB-DP suite
runs first** — PB-DP1..DP10 from `docs/audits/decision-point-audit.md` §8, sequential, tasked out as
`scutemob-149..158`, autonomous chain; **PB-DP1 SHIPPED** (`scutemob-149` merge `f7651bb5`,
2026-07-26 — priority-to-actor CR 117.3c: 14 Group-A sites (6 behaviour flips) + 8 Group-D sites +
entry priority guards on turn_face_up/loyalty/level-up; 19 tests + 15 golden scripts reconciled; no
wire change, 3,721 tests green; seeds OOS-DP1-1..4 filed in audit §8.1); **PB-DP2 SHIPPED**
(`scutemob-150`, 2026-07-26 — the mulligan is no longer a content no-op: `handle_take_mulligan` now
runs a real `timestamp_counter`-seeded `Zone::shuffle` (the `LibraryShuffled` event was a phantom)
and `handle_keep_hand` bottoms with `move_object_to_bottom_of_zone`; CR **103.5/103.5c** — the
suite's "CR 103.4b" cite is stale, that rule is the Vanguard starting life total; **OOS-M11-1
CLOSED**, 4 probes, no wire change (PROTOCOL 27 / HASH 63), tests 3,721 → **3,725**; seeds
OOS-DP2-1..6 filed in audit §8.1); **PB-DP3 SHIPPED** (`scutemob-151`, 2026-07-26 — **mode
announcement is mandatory**, CR **601.2b/700.2a**: the range/duplicate/`min_modes`/`max_modes`
checks were lifted out of the `!modes_chosen.is_empty()` gate so they run whenever the object is
modal. The audit's "mirror the Spree guard" prescription was deliberately not followed — the Spree
guard fires earlier and owns its CR 702.172a message — and the **lift made the yield 40 defs, not
3**: the 3 `Complete` commands (Cryptic/Austere/Incendiary) plus the **37** `min_modes: 1` defs that
had all been accepting an unannounced cast at full price, plus the identical bypass for modal
**activated** abilities in `abilities.rs` (audit §4.2 L214, folded in at zero test cost). Narrow CR
702.120a escalate exemption with the derived count bounds-checked; `resolution.rs`'s `vec![0]`
fallback **retained** — it looks dead and is not: 4 producers build Spell stack objects without
calling `handle_cast_spell` (cascade, discover, cipher, suspend). **0 card-def edits**, PROTOCOL 27
/ HASH 63 unmoved, 8 fail-before probes, tests 3,725 → **3,747**; seeds OOS-DP3-1..9 filed in audit
§8.1; **PB-DP4 SHIPPED** — `scutemob-152` merge `799dcc0a`: attack tax debited colour-correct +
echo/cum-upkeep/recover auto-decline deadline per CR 118.12a, 5 Complete defs righted,
OOS-DP1-1/OOS-RS3-4 closed, tests 3,781; **PB-DP5 SHIPPED** — `scutemob-153` merge `922252f7`:
pending_draws state, 3 emit sites (audit named 2), CR 614.11a multi-draw sequence bug fixed, HASH
63→**64**, tests 3,797, 0 defs register WouldDraw so card yield 0; **PB-DP6 SHIPPED** —
`scutemob-154` merge `d52fe5b6`: intervening-if at queue time, no wire change, tests 3,809,
OOS-DP6-1..10 filed; no-wire block DP1..DP6 complete; **PB-DP7 SHIPPED** — `scutemob-155` merge
`8f890611`: cleanup-discard Command + blocking pending-decision mechanism proven, PROTOCOL 27→**28**
/ HASH 64→**65**, tests 3,837, OOS-DP7-1..12 filed (DP7-11: SR-19 gate skips path-qualified HashInto
— gate-integrity seed); **PB-DP8 SHIPPED** — `scutemob-156` merge `48353a36`: triggered-ability
target choice, CR 603.3d/601.2c; roster **77**, not the audit's 84; 2 live-wrong `Complete` cards
fixed by accident; PROTOCOL 28→**30** / HASH 65→**67**, tests 3,878; OOS-M11-4 CLOSED, seeds
OOS-DP8-1..14; **PB-DP9 SHIPPED** — `scutemob-157`: **search, scry and surveil become CR 608.2d
player choices** — the engine's first *resolution-time* decision channel, built as an
**abort-and-replay** (clone at entry, restore wholesale on suspension, bank the answer, re-run the
resolution from the top) rather than the "resumable effect-list cursor on the stack object" both
`pb-plan-DP7.md` §1.6 and audit §8 prescribed — that design is **impossible**, because
`resolve_top_of_stack` pops the stack object before any effect runs. **ONE `Command`** for all three
effects (CR 608.2d is one rule), PROTOCOL 30→**31** / HASH 67→**68**, roster **69/16/7** not
74/16/8, 0 def edits, tests 3,878 → **3,905**; scry/surveil defaults deliberately FLIP to the
identity; CR 701.22b + CR 400.7 fixed in scope; seeds OOS-DP9-1..12); **PB-DP10 SHIPPED —
`scutemob-158`: THE PB-DP SUITE IS COMPLETE (DP1..DP10, `scutemob-149..158`).** Test-only, and the
*invariant-level* fix the other nine were instances of:
`crates/engine/tests/core/{decision_site_walk,decision_gate}.rs` classify **all 22** decision sites
of audit §3.1 (**4 SERVED** by PB-DP7..DP9 / **15 still AUTO-CHOSEN** / **2 GATED** / **1
NO-DECISION**), freeze the 97 `Complete` defs that still carry an engine-made choice in a name-keyed
`BASELINE`, and redden when a new one arrives (proven end-to-end on a real def, not just
synthetically). **The headline is a gate-integrity finding**: every serde walk in this codebase
before now matched **object keys only** and is blind to a **unit** `Effect` variant
(`to_value(Effect::Proliferate)` is `Value::String`), so a verbatim reuse would have reported **0**
for Proliferate's 25 `Complete` defs while looking green. Two hand-maintained zeros nothing was
holding (`AddManaFilterChoice`, `TheRingTemptsYou`) are now machine-checked. All-rows union **267**
(the §3.1 "277" re-derived — **closes OOS-DP7-7**), still-auto union **97**. PROTOCOL 31 / HASH 68
unmoved, 0 def edits, tests 3,905 → **3,928**. Review 2 HIGH / 6 MEDIUM / 6 LOW all applied; both
HIGHs are about the gate's own honesty — `BASELINE` was populated mechanically and hides two class-D
defs (**OOS-DP10-8**), and the gate can only see a decision the DSL *encoded*, a blind class
strictly worse than the one it records (**OOS-DP10-9**). Seeds OOS-DP10-1..11. **The re-rank is DONE
— `scutemob-159` (2026-07-31), `memory/primitives/seed-rerank-2026-07-27.md`**: 209 `OOS-*` tokens →
207 distinct seeds → 204 real (3 phantoms, one of them newly found: `OOS-RS1-2`); all six declared
closures re-verified against shipped code and **a seventh found that no doc recorded — OOS-RS3-1 was
closed by PB-DP6 while the RS banner went on advertising it as the next insert**; OOS-M11-2's
simulator-side exclusion confirmed; queue re-ranked as **PB-DX1..PB-DX18**. The RS queue and
M11-local are independent tracks and may run concurrently — M11-local touches `crates/simulator` /
`tools/` / a new view-model crate, the RS queue touches `crates/engine` + card defs. Other
candidates: dormant backlog, retired-scripts worklist, or the M10 line (M10-pre → M10a). Completed
chains (EF queue, AC chain, SR-33..38, marker sweep, W-waves, OOS retriage, DOC remediation):
`memory/archive/claude-md-changelog-2026-07.md`.

## Tests-pin history (rotated from the Current State tests bullet, 2026-08-02)

Prior pin: **4,066 passing / 0 failing** on branch `scutemob-170` at PB-DX5 close (+18 over the
**4,048** merge-base baseline at `d568615b`, measured on this branch before any edit. Split: 15 in
`crates/engine/tests/primitives/pb_dx5_affected_set_snapshot.rs`, 1 in-source `#[cfg(test)]` unit
test in `rules/layers.rs` — `snapshot_affected_set` / `candidate_ids_for_filter` are `pub(crate)`
and unreachable from an integration test — and 2 in the new
`crates/engine/tests/core/pb_dx5_continuous_effect_roster.rs`. That last file is why the implement
phase first reported +16/4,064: it contains **two** `#[test]`s, not one, and the arithmetic was
written down rather than run. **HASH 69 → 70** (the new hashed field), append-only history row, 43
sentinels re-pinned by symbol grep — two of them multi-line, invisible to the single-line pattern,
and caught only by a full `--no-fail-fast` workspace run. **PROTOCOL confirmed unmoved at 32** by
executing `--test core protocol_schema`, not by predicting it. Coverage unmoved at **1,137/1,804 =
63.0%** — 0 completeness flips, as pre-committed.) Prior pin: **4,048 passing / 0 failing** on
branch `scutemob-168` at PB-DX4 close (+8 over the **4,040** merge-base baseline measured on this
branch before any edit — note 4,040, not the 4,022 pin below: `scutemob-165`/`166`/`167` all merged
in between. The 8 are `crates/engine/tests/primitives/pb_dx4_baseline_triage.rs`; PROTOCOL 32 / HASH
69 unmoved and the zero-engine gate is an empty `git diff` over `crates/engine/src` +
`crates/card-types/src`. Coverage 1,143 → **1,137** (63.4% → **63.0%**) from 5 completeness
demotions, 0 promotions. Four gate pins moved as a consequence, each re-measured against
`all_cards()` rather than derived: `BASELINE` 97 → 92, `completeness_deviation_scan` floor 661 →
666, PB-DP8 roster 76 → 75, `scry` 16 → 15.) Prior pin: **4,022 passing / 0 failing** on branch
`scutemob-166` at PB-DX3b close (+14 over this branch's true merge-base baseline of **4,008** at
`0eb5a0d4` — note that is *not* the 3,998 pin below: `scutemob-165` (M11-local S4) merged after
`scutemob-164` and brought `crates/view-model/src/tests.rs` with it, worth +10. T1..T12 from
implement, T13/T14 from the fix cycle, all in
`crates/engine/tests/primitives/pb_dx3b_stale_blocker_bucket.rs`; PROTOCOL 32 / HASH 69 unmoved and
the zero-engine gate is an empty `git diff` over `crates/engine/src` + `crates/card-types/src`).
Prior pin: **3,998 passing / 0 failing** on branch `scutemob-164` at PB-DX3 close (+10 over the
3,988 main pin, all in the new `crates/engine/tests/primitives/pb_dx3_stale_blocker_notes.rs`; 31
workspace test binaries; PROTOCOL 32 / HASH 69 unmoved, and the zero-engine gate is an empty `git
diff` over `crates/engine/src` + `crates/card-types/src`). Prior pin: **3,988 passing / 0 failing**
on main at the `scutemob-162`+`scutemob-163` collect (verified post-merge on the combined tree; the
two branches forked from the same 3,955 main pin and were code-disjoint — PB-DX2 branch pin
**3,978** (+23), M11-local S3 branch pin **3,965** (+10: engine +7, simulator +4, −1 for the deleted
`command_player` unit that S3's `HumanChoice` made unreachable); PROTOCOL **32** / HASH **69**
unmoved by both). Prior pin: **3,945** on branch `scutemob-160` at PB-DX1 close (+17 over the 3,928
pin). Before that: **3,928** on main at PB-DP10 collect (merge `16ffcfd0`; suite total +245 from the
3,683 pin at `scutemob-147`) (+18 test-only gate tests over the 3,910 on main at PB-DP9 collect,
merge `d65e7f1e`; branch pin was 3,905 at PB-DP9 close (prior pin 3,878 at PB-DP8 collect, merge
`48353a36`; PB-DP9 adds 23 engine + 4 simulator tests). Prior pin **3,747** on branch `scutemob-151`
at PB-DP3 close (prior pin 3,721 at PB-DP1 collect, merge `f7651bb5`; prior pin 3,683+ at
`scutemob-147`, counted across all 31 workspace test binaries; includes M11-local S1's 6 new
`crates/simulator/tests/local_game.rs` tests; PB-RS4's 17 face-dereg probes merged after the pin) —
**PB-DP2 landed +4 (3,725) and PB-DP3's +22 are on branch `scutemob-151` and land at collect:
3,747** — across the consolidated suites (SR-9a: 297 test binaries → 9 *in `crates/engine`*; the
workspace total is higher because other crates have their own targets); build/clippy/fmt clean
