# PB-DX48 — Ward never fires on a triggered ability (OOS-ENG2-1 ≡ OOS-ENG2-2, + OOS-ENG2-3)

Task `scutemob-219`, branch `feat/pb-dx48-ward-never-fires-on-a-triggered-ability-cr-70221a-ta`.
v4 queue rank 6 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 6; re-verification §2.5).

---

## §0 — WIRE PREDICTION, written BEFORE any code changed

**Prediction: PROTOCOL 39 UNMOVED / HASH 78 UNMOVED.**

Committed at `git rev-parse HEAD` = `7eb0b2e0` (merge of `scutemob-218`), with **zero** lines of
`crates/`, `tools/` source changed at the moment of writing (`git status` clean).

**Reason, stated rather than asserted:**

* `GameEvent::PermanentTargeted` already exists (`rules/events.rs:767`, discriminant 69) and is
  already inside the `Command`/`GameEvent`/`Effect` wire closure — it is emitted at three sites
  today. Emitting the **same variant with the same three fields** at more sites adds no type, no
  variant and no field, so `PROTOCOL_SCHEMA_FINGERPRINT` cannot move. This is the identical
  argument PB-DX47 used for a *suppression*, run in the other direction.
* `HASH` hashes **declarations**, not event volume. `state/hash.rs:5484` already hashes
  `GameEvent::PermanentTargeted`'s three fields; nothing about this batch adds a hashed field, a
  hashed struct, or a new enum member. No `GameState` field is added.
* **The one thing that could have moved it, checked explicitly**: the CR 603.3b re-dispatch hook
  (§3) pushes `PendingTrigger`s into the existing `state.pending_triggers` field. That field is
  already hashed and its element type is unchanged, so the *declaration* is unchanged; only the
  runtime population differs, which a schema fingerprint does not see.

**Stop condition (pre-committed):** if either gate moves, STOP and read the bump off the failing
gate's own output rather than inventing one — do not edit a pin to make a prediction true.

## §0b — MOVEMENT BUDGET, written BEFORE any code changed

The ENG-2 handoff warns that this fix "will move fuzz and golden parity — budget for that", and
the v4 memo row repeats it. Recorded here in advance so an EMPTY moved-pin list has to be
*explained* rather than quietly enjoyed (PB-DX15a's lesson: a paid-and-unclaimed budget is
reported, not dropped):

* golden-script assertion paths that count events;
* SR-9b per-step stream fingerprints (`stream_fingerprint_is_pinned`);
* `UI3_SPLIT_COMBAT_SEED` and the other seeded constants in `tools/play-server`;
* fuzz violation counts.

Every movement is to be listed **by NAME with its CR reason**; no assertion is to be weakened to
absorb one.

---

## §1 — Census, re-verified at HEAD by the INVERSE method

Not by trusting the seed's list and not by trusting the memo's. All
`push_target_announcement` call sites, minus the sites that emit
`GameEvent::PermanentTargeted`:

| # | site | dispatch at HEAD (pre-fix) | class |
|---|---|---|---|
| 1 | `casting.rs::handle_cast_spell` | **YES** | emitter |
| 2 | `abilities.rs::handle_activate_ability` | **YES** | emitter |
| 3 | `abilities.rs::handle_activate_bloodrush` | **YES** | emitter |
| 4 | `abilities.rs::handle_activate_forecast` | **no** | **MISSING** |
| 5 | `abilities.rs::flush_sorted` (modular arm, T6) | **no** | **MISSING** |
| 6 | `abilities.rs::flush_sorted` (main arm, T7) | **no** | **MISSING** |
| 7 | `abilities.rs::handle_scavenge_card` | **no** | **MISSING** |
| 8 | `engine.rs::handle_activate_loyalty_ability` | **no** | **MISSING** |
| 9 | `copy.rs::resolve_cascade` | n/a | `targets: vec![]` (`OOS-ENG2-3`) |
| 10 | `copy.rs::resolve_discover` | n/a | `targets: vec![]` (`OOS-ENG2-3`) |
| 11 | `resolution.rs::resolve_top_of_stack_inner` (cipher-copy) | n/a | `targets: vec![]` (`OOS-ENG2-3`) |
| 12 | `resolution.rs::resolve_top_of_stack_inner` (suspend) | n/a | `targets: vec![]` (`OOS-ENG2-3`) |

**12 = 3 + 5 + 4. The seed's five-site census is EXACT and COMPLETE.** After three
consecutive batches in which the filed site list was a floor (PB-DX25/25b/25c, and
again in PB-DX44 and PB-DX47), this one reproduces without correction — worth
recording because the discipline is only credible if the exceptions are reported too.

The four `OOS-ENG2-3` sites' emptiness was verified individually (`targets: vec![]`
in the two `copy.rs` struct literals; `StackObject::trigger_default` for the two
`resolution.rs` sites), not inferred from the in-source comments that claim it.

## §2 — What the seeds do NOT say, and it is the whole batch

**Emitting the event is necessary and not sufficient at the two sites the seed is
named after.** `rules/engine.rs::check_and_flush_triggers` ran
`check_triggers(state, events)` and only THEN `flush_pending_triggers(state)`,
appending the flush's events afterwards. Nothing ever re-read the events a flush
itself produced. So a `PermanentTargeted` emitted from `flush_sorted` would have
been read by nothing, the Ward `PendingTrigger` would never have been created, and
the batch's headline site would have had a behavioural delta of **zero** while
shipping a diff that looks exactly like a fix.

**The design was wrong TWICE before it was right, and both corrections came from
execution or from reading, never from argument.**

*Wrong design 1 — a hook inside `flush_sorted`, defeated BY EXECUTION.* The first
implementation queued and placed the becomes-target wave at `flush_sorted`'s own tail
(plus a queue-only call at its suspend return). It works, and it is wrong:
`Command::ChooseTriggerTargets`'s arm calls `check_and_flush_triggers` over
`resume_trigger_flush`'s returned events, which contain that same
`PermanentTargeted` — so the trigger was collected twice and **Ward fired twice**
(two `AbilityTriggered`, ward stack objects 8 *and* 9, observed on a running probe).
That is why every probe in this batch asserts a COUNT rather than presence: a
`>= 1` assertion passes on the broken design.

*Wrong design 2 — the fixpoint in `check_and_flush_triggers`, caught by READING the
other five callers.* It is the caller that most commands go through, the full suite
was green, and the end-to-end probe passed. It is still short: `rules/resolution.rs`'s
post-resolution sweep does its own `check_triggers` and THEN calls
`flush_pending_triggers` with nothing after it, and **`Command::PassPriority` never
calls `check_and_flush_triggers` at all**. So a triggered ability placed during a
spell's *resolution* — the ordinary way a targeted ETB trigger reaches the stack —
would still have dispatched nothing, with the emission in place and every test green.
The six flush sites do not agree on whether they sweep afterwards, and two of them
(`resume_trigger_flush`, `drop_departed_trigger_flush`) bypass `flush_pending_triggers`
entirely.

**Shipped design**: the wave loop lives in `abilities::flush_pending_triggers`, which
wraps the old body (now `flush_pending_triggers_once`) — the one function all six
flush sites go through. `check_and_flush_triggers` is restored to a single pass;
a second loop there would re-scan events the flush already dispatched, which is
wrong design 1 again one layer up. **The dispatch is a property of flushing, not of
remembering to sweep afterwards.**

**Wrong design 2 was then confirmed wrong by EXECUTION, not left as an argument.**
A probe was built for the exact shape it misses: an ARTIFACT source (so it is not its
own `TargetCreature` candidate) whose ETB trigger targets the opponent's ward
creature, which is the ONLY creature on the battlefield — so
`forced_trigger_target_answer` answers the CR 603.3d slot without suspending
(CR 601.2c: one legal answer is not a choice), and the trigger is placed by
`resolution.rs`'s post-resolution flush during `Command::PassPriority`. Measured both
ways:

| tree | `PermanentTargeted` emitted | ward trigger on stack |
|---|---|---|
| HEAD | **1** | **1** |
| `flush_pending_triggers` reverted to `flush_pending_triggers_once` | **1** | **0** |

**The emission happens and nothing dispatches it.** That is the entire difference
between the shipped fix and a diff that reads like one, and it is why every probe in
this batch asserts the ward trigger reaching the STACK rather than the event being
present — a probe asserting only on `PermanentTargeted` is GREEN under that revert.

The durable half is not "we found a bug in our own patch". It is that a green full
suite and a passing end-to-end probe were both satisfied by wrong design 2 — the
thing that caught it was enumerating the OTHER callers of the function being changed,
which is the same enumeration discipline this queue has been paying for since
PB-DX25. And the *reason* the end-to-end probe was satisfied is worth stating
separately: it drove `Command::ChooseTriggerTargets`, because a two-creature board
makes the trigger's target a real choice. **The probe was more interactive than the
common case, and that is what made it weaker** — the ordinary board has one legal
target, takes the forced-answer path, and never reaches the arm the probe exercised.

## §3 — Wire: prediction CONFIRMED

`PROTOCOL_VERSION` **39** (`rules/protocol.rs:427`) and `HASH_SCHEMA_VERSION` **78**
(`state/hash.rs:886`), both **UNMOVED**, gate-executed: `core::hash_schema` and
`core::protocol_schema` green, including `history_is_append_only` and
`frozen_prefix_is_pinned`. No pin edited, no history row appended, because none was
owed. The prediction and its reasoning were committed at `43fc20ab`, before a line of
source changed.

## §4 — Movement budget: the list is EMPTY, and here is the measured reason

The ENG-2 handoff and the v4 memo row both said *"it will move fuzz and golden
parity — budget for that."* The full-workspace suite over the engine change alone
went **4,873 → 4,873, zero regressions**. An empty list is only honest if its reason
is measured, so:

1. **The SR-9b per-step fingerprint is structurally blind to new events.**
   `harness_equivalence.rs::fingerprint` is `public_state_hash()` plus each seat's
   `private_state_hash(pid)` — a hash of GAME STATE. So is
   `hash_schema::stream_fingerprint_is_pinned`, which hashes the canonical fixture's
   state. Adding an event moves neither. What WOULD move them is a Ward trigger
   actually firing, because that changes the stack and can counter a spell.
2. **No fixture in the tree makes one fire on a new site.** 30 of the 271 golden
   scripts mention Ward; the one that exercises it
   (`stack/055_ward_counters_lightning_bolt.json`) targets through
   `handle_cast_spell`, which was already an emitter and is behaviourally unchanged.
   `etb-triggers/177_ravenous_tyrranax_rex_draw.json` has a Ward creature and an ETB
   trigger, but that trigger draws a card and targets nothing.
3. **The three pre-existing emitter sites are byte-identical after Part A**, which
   is what confines any movement to the five new sites in the first place. Part A
   folds three hand-rolled loops into one helper with the identical predicate and the
   identical emission order (`TargetsAnnounced` first, then the `PermanentTargeted`
   events); the only textual behavioural difference is bloodrush's push becoming
   conditional on `zone_at_cast == Some(Battlefield)`, and that is inert because
   `check_triggers`'s own arm already required `zone == Battlefield`.

### §4b — Except that the FUZZ half of the budget DID come due, and here it is by name

The paragraphs above are about the TEST SUITE, and they hold. The fuzzer is a separate
measurement and it was taken separately: the identical invocation
(`cargo run --profile fuzz --bin mtg-fuzzer -- --games 20 --seed 1 --max-turns 200`)
on the merge base `7eb0b2e0` in an isolated `git worktree` with its own
`CARGO_TARGET_DIR`, and on this branch. **The two trees differ in exactly one thing —
the CR 702.21a dispatch — so any delta is attributable to it by construction.**

| metric | before (`7eb0b2e0`) | after | delta |
|---|---|---|---|
| games completed / wins / draws / errors | 20 / 20 / 0 / 0 | 20 / 20 / 0 / 0 | **0** |
| avg turns per game | 122.3 | 122.3 | **0** |
| HARD violations, raw / distinct | 185 / 13 | 185 / 13 | **0** |
| — `player_consistency` | 138 in games [2,3,6,8,9,14,17,19] | identical | **0** |
| — `attachment_validity` | 47 in games [3,10,13,16,18,20] | identical | **0** |
| TRANSIENT `no_orphaned_tokens`, raw / distinct | 273 / 62 | **275 / 63** | **+2 / +1** |
| `SpellCast` census | 905 | 902 | −3 |
| `LandPlayed` census | 938 | 937 | −1 |
| casts with ≥1 announced target (CR 601.2c) | 890 total / 106 | 887 / 105 | −3 / −1 |
| `triggered_targets` decision points | 70 | 69 | −1 |
| rejections / commands | 2677 / 113366 (23.614 ‰) | 2697 / 113377 (23.788 ‰) | +20 / +11 |
| — of which, **seed 12** | 69 / 6328 | **89 / 6339** | **+20 / +11** |
| tap runs (wasted) | 6404 (5154) | 6407 (5157) | +3 |

**Read it as ONE divergence, not eleven.** Every rejection the run gained is in a
single game (seed 12: +20 of +20), and every other moved figure is downstream of that
game taking a different path from some turn onward — three fewer casts, one fewer
land, one more checkpoint at which a token was still un-reaped. `no_orphaned_tokens`
is the class PB-DX32 classified TRANSIENT precisely because it is checkpoint-weighted
and self-clearing at the next SBA sweep; **no HARD count moved at all, and no game
changed outcome.**

**Mechanism, stated as an inference with its premise rather than as a fact.** The
divergence is caused by the dispatch (the trees' only difference). It is almost
certainly a **Ward** trigger firing: `PermanentTargeted`'s handler dispatches exactly
two families, Ward and `PermanentBecomesTarget`, and the census in §1 measures the
latter at **zero** deck-legal `Complete` members while Ward has three. **The specific
card is UNIDENTIFIED** and that is stated rather than guessed: the fuzzer writes no
per-game journal in batch mode and `--replay` prints no card names, so naming it would
cost instrumentation this batch did not build.

**Correction to the v4 memo §2.8 while re-measuring.** That section recorded
HARD **106** across 7/20 and TRANSIENT **226** across 12/20 for this exact invocation.
Neither reproduces: the merge base gives **185** across 13/20 and **273** across 14/20.
That drift is entirely pre-PB-DX48 (§2.8's census closed 2026-08-14, before PB-DX43,
PB-DX44, PB-DX15a, PB-DX45 and PB-DX47 all shipped) and is recorded here so the next
batch re-measures rather than trusting the memo's figures — which is the memo's own
rule, applied to the memo.

So: the test-suite half of the budget was paid and unclaimed; the fuzz half came due,
and it came due for **+2 raw / +1 distinct transient reports and twenty rejections in
one game of twenty**. Reported rather than quietly enjoyed, and reported rather than
rounded to "no change".

## §5 — AC 7252's "ward cost paid" branch is UNREACHABLE at HEAD. Stated, not skipped.

The criterion asks for both CR 702.21a outcomes. Only one exists in this engine, and
the measurement is the useful part:

* `effects/mod.rs`'s `Effect::MayPayOrElse` arm destructures `or_else` and **discards
  `cost` and `payer`**, under a comment saying *"M9+: interactive choice to pay or
  not. For M7, don't pay → apply or_else."* Ward's builder-synthesized trigger
  (`state/builder.rs:405-450`) is built on that variant, so the ward cost is never
  offered to anyone and the targeting spell or ability is countered unconditionally.
  `mechanics_m_z/ward.rs`'s own module doc says so.
* **Blast radius of fixing it, measured: ZERO deck-legal `Complete` card defs use
  `Effect::MayPayOrElse`.** All 15 defs naming it are `known_wrong` (6), `partial` (5)
  or `inert` (4), and 12 of the 15 name it only inside a `// TODO` explaining why they
  cannot use it. Ward is the variant's only live consumer.
* **And that is precisely why it is a separate batch, not a rider here.** Routing it
  onto PB-DX45's shipped CR 608.2d channel means reusing
  `EffectChoiceQuestion::PayOptionalCost`, whose `default_effect_choice_answer` returns
  **`pay: true`** — deliberately, because that recovers `MayPayThenEffect`'s pre-PB-DX45
  auto-pay. For `MayPayOrElse` the pre-batch behaviour is auto-**decline**, so parity
  needs `pay: false`; the two cannot share one default without a distinguishing field
  or a second question variant, and either is a **wire bump** — contradicting this
  batch's own gate-confirmed `PROTOCOL 39 / HASH 78 UNMOVED` and moving every ward
  golden script and all eight `mechanics_m_z/ward.rs` tests in the process.

Filed as **`OOS-DX48-2`**. What is exercised instead is the two-sided discrimination
CR 702.21a itself provides: Ward fires exactly once when an **opponent's** ability
targets the permanent, and **not at all** when its own controller's does
(CR 702.21a's "an opponent controls").

## §6 — The dispatch map, enumerated rather than assumed

Emitting `PermanentTargeted` at a site only helps if something scans it. There are
three ways an emission reaches `check_triggers`, and the batch is only complete
because all three were enumerated. **Two of them were found by reading the callers
after the first design was already green on the full suite.**

| emission site | how its event reaches `check_triggers` | verified |
|---|---|---|
| `casting.rs::handle_cast_spell` | the command arm's `check_and_flush_triggers` | pre-existing |
| `abilities.rs::handle_activate_ability` | same | pre-existing |
| `abilities.rs::handle_activate_bloodrush` | same | pre-existing |
| `abilities.rs::handle_activate_forecast` | same (`Command::ActivateForecast`'s arm) | ✓ read at HEAD |
| `abilities.rs::handle_scavenge_card` | same (`Command::ScavengeCard`'s arm) | ✓ read at HEAD |
| `engine.rs::handle_activate_loyalty_ability` | same (`Command::ActivateLoyaltyAbility`'s arm) | ✓ read at HEAD |
| `abilities.rs::flush_sorted` ×2, via `flush_pending_triggers` | `dispatch_becomes_target_waves`, called by `flush_pending_triggers` | ✓ NEW |
| `abilities.rs::flush_sorted` ×2, via `drop_departed_trigger_flush` from `handle_concede` (CR 800.4d) | `dispatch_becomes_target_waves`, called explicitly there | ✓ NEW |
| `abilities.rs::flush_sorted` ×2, via `resume_trigger_flush` | `Command::ChooseTriggerTargets`'s own `check_and_flush_triggers` | pre-existing sweep |
| the four `OOS-ENG2-3` free-cast sites | n/a — `targets: vec![]`, nothing to emit | structurally |

**`resume_trigger_flush` is deliberately NOT given the wave loop**, and the reason is
the batch's own defect: its events are already swept, so adding it would dispatch the
same event twice. That asymmetry is what leaves `OOS-DX48-3` open — the
`ChooseTriggerTargets` sweep is guarded on `pending_trigger_targets.is_none()`, so a
batch that suspends a SECOND time hands its middle section to a caller that never
scans it. Filed rather than hidden, and narrowed to that one path only after the other
two were closed.

**The three `check_and_flush_triggers`-covered new sites were read at HEAD, not
assumed.** "The handler returns events and something sweeps them" is exactly the kind
of claim this batch exists to punish: `Command::PassPriority` and `Command::Concede`
both look like they should sweep and neither does.

## §7 — Test suite (§3a engine-side probes) and revert matrix

`crates/engine/tests/primitives/pb_dx48_ward_dispatch.rs` (new; `mod` added to
`tests/primitives/main.rs` between `pb_dx47_modal_trigger_mode_zero` and
`pb_dx4_baseline_triage`, correct ASCII order) implements plan §3a's t1/t1b/t2/t3/t4/
t5/t6/t7/t8. Plan §3d's pin inversion is in
`crates/engine/tests/primitives/pb_eng2_targets_announced.rs`.

**The file targets the FINAL architecture (`8c703988` → the later split of
`dispatch_becomes_target_waves` out of `flush_pending_triggers`), not the first draft
(`72c0770c`) the plan was written against.** The engine moved twice more while this
file was being written (the wave loop relocating from `rules/engine.rs::
check_and_flush_triggers` into `abilities::flush_pending_triggers`, then that function
splitting into `flush_pending_triggers` + `dispatch_becomes_target_waves` so
`handle_concede`'s CR 800.4d resume could call the wave loop too). None of the 12
tests in this file needed to change across that relocation — they assert observable
behaviour (event counts, stack contents, resolution effects), never which internal
function does the dispatching — which is itself evidence the probes are testing the
right thing.

**`t1` is the coordinator-provided resolution-path probe** (a self-ETB triggered
ability on an ARTIFACT source, targeting the only creature on the board so its CR
603.3d slot is FORCED rather than suspended, driven through `Command::CastSpell` +
two `Command::PassPriority`s so the trigger is placed by `resolution.rs`'s
post-resolution sweep). Adopted with the fixture byte-identical to the measured
original, renamed, given a real doc comment, and extended past the coordinator's
`PermanentTargeted`/stack-count pin to a full resolution: pass twice more, assert the
stack drains to empty and the ward creature takes zero damage (its own `DealDamage`
never ran). `t1b` is its non-vacuity partner (own-controller case) — it asserts the
`PermanentTargeted` count is still 1 (Part A's predicate is controller-agnostic) while
the ward `AbilityTriggered` count is 0 (CR 702.21a's "an opponent controls"), so a
bare "ward fired zero times" claim can't pass vacuously under a total-emission
failure.

**One structural correction made while writing t2/t3/t4/t5/t8c, not anticipated by the
plan or the coordinator's message**: `Effect::CounterSpell`'s `SpellCountered` event
is emitted for exactly two `StackObjectKind` variants — `ActivatedAbility` and
`TriggeredAbility` (see `effects/mod.rs`'s own comment: "every other ability/trigger
kind: NO event, exactly as before PB-DX25 — a DIAGNOSTICS omission, not a state one").
`ForecastAbility`, `ScavengeAbility`, `LoyaltyAbility`, `KeywordTrigger` (Modular) and
`BloodrushAbility` are all OUTSIDE that pair, so Ward silently removing them from the
stack emits no `SpellCountered` at all — proven by running the first draft of these
five tests (which asserted on that event) and watching all five fail with the stack
already empty and no `AbilityResolved`/`SpellCountered` pair to find. Fixed by
asserting on STATE instead: the stack drains to empty (the ward trigger resolved and
its `CounterSpell` removed the other entry) plus a resolution-effect check specific to
each kind (zero damage for forecast, zero +1/+1 counters for scavenge and modular,
controller unchanged for loyalty, power unchanged for bloodrush). This is disclosed
here because it means five of this file's own resolution assertions are NOT what a
naive reading of `ward.rs`'s existing tests would predict, and a future reader
grepping for `SpellCountered` in a new Ward probe will hit the same wall.

**t6 (real corpus Ward defs) also goes through the resolution path**, using a
synthetic `T48 Trigger Bear` (`TargetRequirement::TargetCreatureWithFilter(opponent)`
so the ward creature is the only legal candidate, forcing the answer) cast against
each of `Adrix and Nev, Twincasters`, `Miirym, Sentinel Wyrm` and `Tyrranax Rex`,
built through `enrich_spec_from_def` per the plan's §2 verification. The Ward keyword
cost is asserted directly off `characteristics.keywords` (2, 2, 4) to prove the probe
reads the real def rather than a stand-in.

**t7 (wave bound / no cascade)** asserts the structural claim from a LIVE stack
object rather than a comment: the ward trigger's own `SpellTarget.zone_at_cast` is
`None` (it targets the targeting stack object, never a battlefield permanent), plus a
total-event count of exactly 1 `PermanentTargeted` for the whole command (proving no
wave-2 fired from the ward trigger's own placement).

**Plan §3d.** The existing "DEVIATION PIN (`OOS-ENG2-1`)" comment on
`test_eng2_fell_specter_etb_announces_its_target` was WRONG about what a fix could
ever change: Fell Specter's ETB targets `TargetRequirement::TargetOpponent`, a
PLAYER, and `GameEvent::PermanentTargeted` carries only an `ObjectId` — the negative
assertion could never flip at PB-DX48 or any other batch. Rewritten as a CR 702.21a
SCOPE pin (a player target structurally cannot raise `PermanentTargeted`) with the
correction stated in the comment, and a new positive sibling added in the same file,
`test_eng2_hyrax_tower_scout_etb_object_target_emits_permanent_targeted` — a real,
deck-legal corpus def ("When this creature enters, untap target creature.") reaching
the SAME `flush_sorted` T7 site with an OBJECT target, which now emits exactly one
`PermanentTargeted` where it emitted none before PB-DX48. This is the discriminator
the original pin was asking a successor to produce. Disclosed as a rename/addition in
the delta, not netted out: `test_eng2_fell_specter_etb_announces_its_target`'s claim
text changed (its assertion boolean did not), and one new test was added.

### Revert matrix — 3 rows, all executed, engine restored byte-for-byte after each
(`git checkout crates/engine/src`, verified clean and green before and after)

| row | what was reverted | reddened | stayed green (why, disclosed) |
|---|---|---|---|
| **R-A** — no emission | `rules/events.rs::push_target_announcement`: replaced `events.extend(permanent_targeted_events(..))` with `let _ = permanent_targeted_events(..);` (Part A disabled) | ALL 12 probes in `pb_dx48_ward_dispatch.rs` (t1, t1b, t2, t3, t4, t5, t6, t7, t8a, t8b, t8c) **and** the new `pb_eng2` sibling, **plus** the pre-existing `pb_eng2_targets_announced::test_eng2_activated_ability_targeting_a_permanent_announces` | — (nothing stayed green; this is total dispatch failure) |
| **R-B** — single wave (the DECISIVE row, matches the coordinator's own measurement verbatim) | `abilities.rs::flush_pending_triggers`: body replaced with a bare call to `flush_pending_triggers_once(state)`, dropping the call to `dispatch_becomes_target_waves` | **t1** (resolution-path), **t5** (Modular/SBA path), **t6** (real-def resolution path) — all three go through a flush with NO subsequent outer `check_and_flush_triggers` re-scan, so the wave loop living inside `flush_pending_triggers` is the ONLY thing that can dispatch Ward for them. Each fails on the `ward AbilityTriggered` count (1 expected, 0 observed) — `PermanentTargeted` emission is untouched, confirming the coordinator's point that a probe asserting only on the emission event would have stayed GREEN here | t1b, t2, t3, t4, t7, t8a, t8b, t8c — each of these goes through a `Command` handler (`ActivateForecast`/`ScavengeCard`/`ActivateLoyaltyAbility`/`ActivateAbility`/`CastSpell`/`ActivateBloodrush`) whose OWN `check_and_flush_triggers` call independently scans the already-populated `events` slice and queues Ward directly — the wave loop inside `flush_pending_triggers` is never the only path for these, so this revert cannot discriminate them. Disclosed per-probe in the test file's doc comments, not left implicit. |
| **R-C** — double-scan (reproduces finding #2, the rejected first design that fired Ward twice) | `dispatch_becomes_target_waves`: removed the `scanned = events.len();` cursor advance, so every wave re-includes wave 0's original `PermanentTargeted` slice forever | **t1, t5, t6** — same three as R-B, but this time the failure is a `debug_assert!` PANIC at `MAX_BECOMES_TARGET_WAVES` (16 consecutive re-dispatches of the same event) rather than an assertion mismatch, because the broken cursor causes Ward to be re-queued every wave until the bound trips. Confirms the count assertions (`ward_ability_triggered_count == 1`, not `>= 1`) are load-bearing against the batch's own documented double-dispatch defect. | t1b, t2, t3, t4, t7, t8a, t8b, t8c — same reason as R-B: their Ward trigger is queued once by the outer scan and the (broken) wave loop finds nothing to re-scan for them, so it never re-enters. |

**No UNDISCRIMINATED row.** Every probe in the file is reddened by at least one of the
three reverts (all 12 by R-A alone); `t1`, `t5` and `t6` are additionally reddened by
R-B and R-C, which is disclosed as a STRENGTH (they are this file's only probes that
exercise the wave-loop relocation directly) rather than a gap. `t1b` is discriminated
by R-A only (its own claim — `PermanentTargeted` count is 1 while the ward trigger
count is 0 — depends on emission, not on the wave mechanism), and that asymmetry is
stated in its own doc comment.

Full test file: `cargo test -p mtg-engine --test primitives -- pb_dx48 pb_eng2` — 21
tests, all green at HEAD; `cargo clippy -p mtg-engine --test primitives -- -D
warnings` clean; `cargo fmt --check` clean on both touched files.
