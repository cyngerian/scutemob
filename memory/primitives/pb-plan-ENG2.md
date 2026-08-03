# Primitive Batch Plan: ENG-2 — targets in the event log (an announcement-time target event)

**Generated**: 2026-08-02
**Primitive**: one new `GameEvent` variant, `TargetsAnnounced`, emitted at CR 601.2c /
602.2b / 603.3d announcement time from a single shared helper, plus one viewer-gated
`event_view` arm.
**CR Rules**: 601.2, 601.2c, 602.2b, 603.3d, 115.1, 400.2, 102.1, 708.2, 702.21a, 707.10
**Seeds**: `OOS-G7-1` (no `GameEvent` carries targets) — G7 of
`memory/playtest-triage-2026-08-02b.md`, row 7 of its successor table.
**Task**: `scutemob-193` · **Branch**:
`feat/eng-2-targets-in-the-event-log-an-announcement-time-target-e`
**Cards affected**: 0 card-def lines. Coverage must be unmoved
(**1,133/1,803 = 62.8%**) and proven unmoved by regenerating `tools/authoring-report.py`
to a byte-identical body, not by an empty diff.
**Dependencies**: PB-DP8 (`SpellTarget` is already on the wire), PB-DP9, ENG-1
(this branch's baseline).
**Deferred items from prior PBs carried here**: none taken. Three adjacent gaps found
during recon are **filed, not fixed** — see §10.

**Baselines measured on this branch before any edit**: **PROTOCOL 34**
(`rules/protocol.rs::PROTOCOL_VERSION`), **HASH 71**
(`state/hash.rs::HASH_SCHEMA_VERSION`). The triage's "PROTOCOL currently 33" is **stale** —
ENG-1 (merge `a3b5e56b`) moved both after the triage was written. Both new values are read
from the failing gates' own output; §8 forbids predicting them.

---

## 1. Verified premises — each recon point confirmed or corrected

Cited by symbol. Line numbers appear only as navigation aids and are snapshots
(OOS-DP6-8 class); the runner re-locates by symbol.

### 1.1 — CONFIRMED, with one addition

`rules/events.rs` declares, verbatim:

```rust
SpellCast        { player: PlayerId, stack_object_id: ObjectId, source_object_id: ObjectId }
AbilityActivated { player: PlayerId, source_object_id: ObjectId, stack_object_id: ObjectId }
AbilityTriggered { controller: PlayerId, source_object_id: ObjectId, stack_object_id: ObjectId }
```

None carries targets. `GameEvent::TargetsChanged { stack_object_id, old_targets:
Vec<SpellTarget>, new_targets: Vec<SpellTarget> }` proves a `Vec<SpellTarget>` payload is
wire-legal in an event, and `state/hash.rs`'s `impl HashInto for GameEvent` already hashes
`Vec<SpellTarget>` at the `TargetsChanged` arm (discriminant 126) — so **no new `HashInto`
impl is needed for the payload type**, only a new arm.

**Addition the recon did not state**: `SpellTarget` entered the SR-8 wire closure at
PROTOCOL v29 (PB-DP8, via `TriggerTargetOption`) — see the `- 29:` History line in
`rules/protocol.rs`. It is already reachable from both wire frames, so this batch's closure
**type count does not change** (stays 96); only `GameEvent`'s declared shape moves.

### 1.2 — CONFIRMED with a correction

`GameEvent::PermanentTargeted { target_id, targeting_stack_id, targeting_controller }` is
emitted from **exactly three** sites, and the exhaustive grep
(`GameEvent::(SpellCast|AbilityActivated|AbilityTriggered|PermanentTargeted)` over
`crates/engine/src`) confirms there is no fourth:

| Site | Filter |
|---|---|
| `rules/casting.rs::handle_cast_spell` (after `GameEvent::SpellCast`) | `battlefield_targets` — `Target::Object` **and** `zone_at_cast == Some(ZoneId::Battlefield)` |
| `rules/abilities.rs::handle_activate_ability` | `battlefield_targets` — same filter |
| `rules/abilities.rs::handle_activate_bloodrush` | **no filter at all** — one unconditional push of the single validated target |

**CORRECTION**: the recon says "each filtered to `Target::Object` with `zone_at_cast ==
Some(ZoneId::Battlefield)` (the `battlefield_targets` vecs)". That is true of **two** of the
three. The bloodrush handler builds no `battlefield_targets` vec and pushes
`PermanentTargeted` unconditionally; it is safe today only because its own validation
guarantees a battlefield creature. The correction matters to option (3): the three sites do
not share a filter, so "widen the filter" is three separate edits with three separate
correctness arguments, not one.

Consumption is confirmed: `rules/abilities.rs::check_triggers` has a
`GameEvent::PermanentTargeted { target_id, targeting_stack_id, targeting_controller }` match
arm that re-checks battlefield + `is_phased_in()` + `obj.controller != targeting_controller`,
collects `TriggerEvent::SelfBecomesTargetByOpponent`, and tags each collected trigger with
`t.targeting_stack_id`. Hash special treatment confirmed: `state/hash.rs`, discriminant
**69**, with a cross-reference at the module's discriminant-47 note.

### 1.3 — CONFIRMED, and wider than stated

`rules/abilities.rs::flush_sorted` sets `stack_obj.targets = trigger_targets.clone()` and
then pushes `GameEvent::AbilityTriggered` with **no** `PermanentTargeted`. Verified by
reading the function; verified negatively by the exhaustive grep (the only
`PermanentTargeted` pushes are the three in §1.2).

This is a **pre-existing Ward gap** (CR 702.21a: "Whenever this permanent becomes the target
of a spell **or ability** an opponent controls…"). **DO NOT FIX IT IN THIS BATCH** — filed
as `OOS-ENG2-1` (§10). Fixing it changes Ward behaviour on live games and would move fuzz
and golden-script parity.

**Wider than the recon states**: three *more* target-carrying announcement sites also emit no
`PermanentTargeted`, so Ward silently does not fire for them either —
`handle_activate_forecast` (`stack_obj.targets = spell_targets`),
`handle_scavenge_card` (`stack_obj.targets = vec![SpellTarget{…}]`), and the loyalty-ability
handler in `rules/engine.rs` (`targets: spell_targets` in the `StackObject` literal).
The modular arm of `flush_sorted` is a fifth. Filed as `OOS-ENG2-2`; also not fixed here.

### 1.4 — CONFIRMED

`crates/view-model/src/event_view.rs`:

- `EventView` doc: *"There is deliberately no payload field: anything the client could need
  must have been rendered into `text` by code that consulted the viewer, otherwise it is a
  path around Invariant 7."* Confirmed verbatim.
- `event_view_for` is Architecture Invariant 7 chokepoint #2 (stated in its own doc).
- `card_or(id, fallback)` wraps `card_name`, which is gated by
  `redact::viewer_may_identify`; `name(pid) = player_display_name(pid, player_names)` is
  unconditional.
- `viewer_may_identify` denies (a) any id absent from `state.objects()` (CR 400.7),
  (b) `obj.status.face_down && obj.owner != seat` (CR 708.2), (c) `ZoneId::Hand(p)` /
  `ZoneId::Library(p)` where `p != seat` (CR 402.1 / 401.2). Everything else is allowed.
- The `PermanentTargeted` arm renders `"{name(targeting_controller)} targets
  {card_or(target_id, \"a permanent\")}"`. Confirmed.
- The `SpellCast` / `AbilityActivated` / `AbilityTriggered` arms render no target because
  they have none. The `AbilityTriggered` arm produces the literal line the playtester read:
  `"A triggered ability of {n} goes on the stack"`. Confirmed.

**Two additions the recon did not state, both load-bearing for the work plan:**

1. `event_tier` is **deliberately non-exhaustive** (`_ => EventTier::Game`, pinned by
   `test_event_tier_defaults_to_game_for_an_unclassified_variant`). A new variant therefore
   **compiles without any edit here** and silently lands in the wrong tier with a bare-kind
   line. The tier list is a required edit that the compiler will not demand. Filed as
   `OOS-ENG2-7`.
2. `event_kind` is derived from the serde discriminant, not a hand-written match, so it needs
   **no** edit.

### 1.5 — CONFIRMED

`rules/protocol.rs::PROTOCOL_VERSION = 34` (History tail is `- 34: ENG-1`);
`state/hash.rs::HASH_SCHEMA_VERSION = 71` (History tail is `- 71: ENG-1`). The triage's
"PROTOCOL currently 33" is stale. Both `PROTOCOL_HISTORY` and `HASH_SCHEMA_HISTORY` tails
match.

### 1.6 — Two premises the recon asserted that are WRONG

**(a) "`abilities.rs` ×1 more `SpellCast`" — REFUTED.** `rules/abilities.rs` contains
`GameEvent::SpellCast {` once, inside `check_triggers`, as a **match arm consuming** the
event (CR 603.2 `WhenYouCastThisSpell` dispatch). There is no `events.push(GameEvent::SpellCast`
anywhere in `abilities.rs`. Likewise `abilities.rs`'s `GameEvent::PermanentTargeted {` is the
Ward **consumer** arm, not a fourth emitter.

**(b) `rules/engine.rs` was omitted from the recon's site list entirely — and one of its two
sites carries targets.** `rules/engine.rs` pushes `GameEvent::AbilityActivated` twice: once
from the loyalty-ability handler, whose `StackObject` literal sets `targets: spell_targets`
built from the command's declared targets (this is `OOS-M11-10` territory), and once from
`handle_level_up_class`, whose literal sets `targets: vec![]`. A plan built from the recon's
list would have shipped **planeswalker loyalty abilities announcing nothing** — the single
most visible targeted action in a Commander game after spells.

**(c) A third, minor one.** The recon's "and 5 keyword-trigger sites in `casting.rs` for
storm/gravestorm/cascade/casualty/replicate" is correct, and all five are structurally
target-free (`StackObject::trigger_default` sets `targets: vec![]` and none overrides it).

---

## 2. The three-option adjudication

CR text below is quoted from the mtg-rules MCP, not paraphrased.

> **CR 601.2c** — "The player announces their choice of an appropriate object or player for
> each target the spell requires. … The chosen objects and/or players each become a target of
> that spell. (Any abilities that trigger when those objects and/or players become the target
> of a spell trigger at this point; they'll wait to be put on the stack until the spell has
> finished being cast.)"
>
> **CR 602.2b** — "The remainder of the process for activating an ability is identical to the
> process for casting a spell listed in rules 601.2b–i. Those rules apply to activating an
> ability just as they apply to casting a spell."
>
> **CR 603.3d** — "The remainder of the process for putting a triggered ability on the stack
> is identical to the process for casting a spell listed in rules 601.2c–d."
>
> **CR 115.1** — "Some spells and abilities require their controller to choose one or more
> targets for them. The targets are object(s) and/or player(s) the spell or ability will
> affect. These targets are declared as part of the process of putting the spell or ability
> on the stack. The targets can't be changed except by another spell or ability that
> explicitly says it can do so."
>
> **CR 400.2** — "Public zones are zones in which all players can see the cards' faces,
> except for those cards that some rule or effect specifically allow to be face down.
> Graveyard, battlefield, stack, exile, ante, and command are public zones. Hidden zones are
> zones in which not all players can be expected to see the cards' faces. Library and hand are
> hidden zones…"
>
> **CR 102.1** — "A player is one of the people in the game. The active player is the player
> whose turn it is. The other players are nonactive players."
>
> **CR 708.2 / 708.2a** — "Face-down spells and face-down permanents have no characteristics
> other than those listed by the ability or rules that allowed the spell or permanent to be
> face down. … it becomes a 2/2 face-down creature with no text, no name, no subtypes, and no
> mana cost."
>
> **CR 702.21a** — "Ward is a triggered ability. Ward [cost] means 'Whenever this permanent
> becomes the target of a spell or ability an opponent controls, counter that spell or ability
> unless that player pays [cost].'"
>
> **CR 707.10** — "To copy a spell, activated ability, or triggered ability means to put a
> copy of it onto the stack; **a copy of a spell isn't cast** and a copy of an activated
> ability isn't activated. A copy of a spell or ability copies both the characteristics of the
> spell or ability **and all decisions made for it, including modes, targets, the value of X,
> and additional or alternative costs.**"

**A citation correction that must ride in this batch.** Three shipped in-source comments cite
**CR 108.1** for "a player target is always public" —
`view-model/src/redact.rs`'s module table row for `format_target`,
`redact.rs`'s `redact_stack` loop, and `event_view.rs`'s `attack_target` closure.
**CR 108.1 is the Oracle-text rule** ("Use the Oracle card reference when determining a
card's wording"), not a public-information rule; the MCP lookup above is decisive. The
correct chain is **CR 102.1 + CR 115.1 + CR 400.2**: a player is a person in the game and no
rule makes their identity hidden; targets are declared as part of putting the object on the
stack; the stack is a public zone. This plan uses the correct chain, and §5 rides the
three-comment fix (comment-only, zero behaviour) so ENG-2 does not propagate a bad cite into
a fourth site.

### Option (3) — widen `PermanentTargeted` into a `Targeted` event covering `Target::Player`

**VERDICT: REJECT.** The dispatching worker's two grounds both hold, and recon adds a third.

1. **It structurally cannot reach the reported defect.** Verified in §1.3: `flush_sorted`
   emits no `PermanentTargeted`, so a Fell Specter ETB would still render nothing after
   option (3) shipped. To reach it you must add an emission at the trigger flush, and
   `check_triggers` dispatches Ward off exactly that event — so the display fix would
   silently start firing Ward on triggered abilities. That is CR-correct per CR 702.21a
   ("spell **or ability**"), which is precisely why it is *not* a display change: it changes
   game outcomes, and it moves fuzz and golden-script parity. **Option (3) is a correctness
   batch wearing a display batch's clothes.**
2. **It couples the display channel to the Ward dispatch channel.** `PermanentTargeted` is
   load-bearing: its consumer re-checks battlefield membership, phasing and opponent-control
   and then writes `targeting_stack_id` onto each collected trigger so the Ward
   `CounterSpell` effect can locate the stack object. Widening the payload to carry
   `Target::Player` forces the consumer to destructure and skip players; a mistake there
   fires Ward on a player target or drops a real one. It also has a hash contract
   (discriminant 69) with a cross-referenced note.
3. **It is not "one variant, and reuses the existing arm" — recon §1.2 refutes the premise.**
   The three emitters do not share a filter (bloodrush has none), so widening is three edits
   with three arguments. And after all that, option (2) is *also* one variant, touches no
   consumer at all, and needs no change to any shipped emission.

### Option (1) — add `targets` to each of the three announcement variants

**VERDICT: REJECT** — for a stronger reason than the brief gives.

The brief's reason (churn) is real and measurable: **27 production emission sites** (§3) must
each be edited, of which **19** would pass `vec![]`. But churn alone is a weak argument in a
codebase that routinely edits 30 sites.

The decisive reason is different: **option (1) makes the defect it fixes unfalsifiable
per-site.** With a `targets` field on `SpellCast`, a site that forgets to populate it emits
`targets: vec![]`, which is byte-identical to a spell that legitimately has no targets. No
gate can distinguish them — not a grep, not a source walk, not a runtime assertion — because
the two states are the same state. That is exactly the failure mode this batch exists to
remove, re-created in a form no future audit can see. Option (2)'s presence/absence of a whole
event is greppable, gate-able (§4) and testable (§7(e)).

Secondary: it changes three shipped wire payloads, so any struct-literal construction breaks
(`crates/view-model/src/tests.rs` builds a `GameEvent::SpellCast { … }` literal; the ~150
`matches!(e, GameEvent::AbilityTriggered { .. })` test sites survive on `..`).

### Option (2) — one additive `TargetsAnnounced` at announcement time

**VERDICT: TAKE.**

- Additive: no shipped variant's payload changes, so every existing `matches!`, destructure
  and struct literal in the workspace is untouched.
- One new `HashInto` arm (discriminant **132**, the next free one — the `GameEvent` arm ends
  at 131/`EffectChoiceRequired` and the match is exhaustive with no `_`), reusing the
  `Vec<SpellTarget>` feed `TargetsChanged` already established.
- One new `event_view` arm + one entry in the `event_tier` Stack list.
- Downstream is exhaustive-match additions only, and there are exactly two exhaustive matches
  in the workspace (§6).
- It reaches all three announcement kinds uniformly through **one** construction site (§4),
  which is the property that makes the batch gate-able.
- It records what was **announced**, which is what CR 601.2c describes — a moment, not a
  state. See the rejection of option (4) for why that distinction is load-bearing.

### Option (4) — considered and rejected: derive it in the view model, zero engine change

Not in the brief. It is the genuinely cheapest option and somebody will propose it, so it is
adjudicated here rather than left to be re-discovered.

`event_view_for` already receives `&GameState`. On seeing `SpellCast { stack_object_id }` it
could look the stack object up in `state.stack_objects()` and render its `targets` — **zero
engine lines, zero wire change, PROTOCOL and HASH unmoved.**

**REJECT, on a timing argument that is fatal.** The `state` an `EventView` is rendered
against is the *current* state, not the state at announcement:
`tools/play-server/src/api.rs`'s seat-view builder maps `event_view_for(ev, state, …)` over
the whole event slice **after** the command batch has been applied, and
`tools/play-server/src/session.rs`'s own doc says the `EventView`s are built against
`self.game.state()`. So:

- a spell that was announced and then **resolved inside the same batch** has no stack object
  left (CR 608.2m), and the line renders empty — reintroducing the defect for exactly the
  fast bot turns the playtester complained about;
- a spell whose targets were **changed** by CR 115.7 (`Effect::ChangeTargets`) would have its
  *announcement* line retroactively rewritten to the new targets, which is a false statement
  about history and directly breaks Architecture Invariant 4 ("Events are the single source
  of truth for what happened").

An event log must record what was announced. A derived view can only report what is true now.

---

## 3. Exhaustive emission-site inventory — the batch's correctness surface

Derived from `Grep "GameEvent::(SpellCast|AbilityActivated|AbilityTriggered|PermanentTargeted)"`
over `crates/engine/src` (the complete, unpaginated result), then reading each site and its
enclosing function. **27 production emission sites** of the three announcement variants, in
**5 files**. The two `abilities.rs` and one `hash.rs` hits that are *match arms* are excluded
and named in §1.6(a).

"Targets?" means: can the `StackObject` this site just pushed ever carry a non-empty
`targets` vec?

### 3.1 `SpellCast` — 5 sites

| # | File / function | Targets? | Where they come from |
|---|---|---|---|
| S1 | `rules/casting.rs::handle_cast_spell` | **YES** | `spell_targets` from `validate_targets_with_source` / `validate_targets_positional` (CR 601.2c), **moved** into `StackObject { targets: spell_targets }` ~200 lines before the emission. See §4.2. |
| S2 | `rules/copy.rs` cascade free-cast (`resolve_cascade`) | no | `StackObject` literal hardcodes `targets: vec![]`. CR 702.85a free-cast. **Latent gap** — a cascaded targeted spell is put on the stack with no targets at all; filed `OOS-ENG2-3`, not fixed here. |
| S3 | `rules/copy.rs` discover free-cast (`resolve_discover`) | no | Same: literal `targets: vec![]`. CR 701.57a. Same latent gap. |
| S4 | `rules/resolution.rs` cipher-copy arm (`StackObjectKind::KeywordTrigger { keyword: Cipher, .. }`) | no | `StackObject::trigger_default` ⇒ `targets: vec![]`; the site's own comment says *"MVP: Cast the copy without selecting targets (no target selection for targeted copies -- deferred)."* |
| S5 | `rules/resolution.rs` `StackObjectKind::SuspendCastTrigger` arm | no | `trigger_default` ⇒ `targets: vec![]`. CR 702.62a. Same latent gap. |

### 3.2 `AbilityActivated` — 15 sites

| # | File / function | Targets? | Where they come from |
|---|---|---|---|
| A1 | `rules/abilities.rs::handle_activate_ability` | **YES** | `stack_obj.targets = spell_targets` (CR 602.2b → 601.2c). Also emits `PermanentTargeted`. |
| A2 | `rules/abilities.rs::handle_cycle_card` | no | `trigger_default`; cycling (CR 702.29a) has no target. |
| A3 | `rules/abilities.rs::handle_activate_forecast` | **YES** | `stack_obj.targets = spell_targets`. Emits **no** `PermanentTargeted` — `OOS-ENG2-2`. |
| A4 | `rules/abilities.rs::handle_activate_bloodrush` | **YES** | `stack_obj.targets = vec![SpellTarget { target: Target::Object(target), … }]`, exactly one. Emits `PermanentTargeted` unfiltered. |
| A5 | `rules/abilities.rs::handle_unearth_card` | no | `trigger_default`, never overridden. |
| A6 | `rules/abilities.rs::handle_ninjutsu` | no | `trigger_default`. |
| A7 | `rules/abilities.rs::handle_embalm_card` | no | `trigger_default`. |
| A8 | `rules/abilities.rs::handle_eternalize_card` | no | `trigger_default`. |
| A9 | `rules/abilities.rs::handle_encore_card` | no | `trigger_default`. |
| A10 | `rules/abilities.rs::handle_crew_vehicle` | no | `trigger_default`. CR 702.122a — crew targets nothing. |
| A11 | `rules/abilities.rs::handle_saddle_mount` | no | `trigger_default`. CR 702.171a. |
| A12 | `rules/abilities.rs::handle_scavenge_card` | **YES** | `stack_obj.targets = vec![SpellTarget { target: Target::Object(target_creature), zone_at_cast: Some(Battlefield) }]`. No `PermanentTargeted` — `OOS-ENG2-2`. |
| A13 | `rules/engine.rs` loyalty-ability handler | **YES** | `StackObject { …, targets: spell_targets, … }` built from the command's `targets` (CR 606.1 → 602.2b → 601.2c). No `PermanentTargeted` — `OOS-ENG2-2`, and this is the site the recon omitted entirely. |
| A14 | `rules/engine.rs::handle_level_up_class` | no | `StackObject` literal, `targets: vec![]`. CR 716.2a. |

*(That is 14 rows for 15 grep hits: `handle_activate_ability` and `handle_level_up_class`
each hit once; the count reconciles because `rules/abilities.rs` has 12 `AbilityActivated`
pushes and `rules/engine.rs` has 2 — 14 total. **Corrected count: 14 `AbilityActivated`
sites, not 15.** The runner must re-derive this number from the gate's own scan (§4.3) and
pin whatever it finds, never this prose.)*

### 3.3 `AbilityTriggered` — 7 sites

| # | File / function | Targets? | Where they come from |
|---|---|---|---|
| T1 | `rules/casting.rs::handle_cast_spell` — Storm (CR 702.40a) | no | `trigger_default`. |
| T2 | `rules/casting.rs::handle_cast_spell` — Gravestorm (CR 702.69a) | no | `trigger_default`. |
| T3 | `rules/casting.rs::handle_cast_spell` — Cascade (CR 702.85a) | no | `trigger_default`. |
| T4 | `rules/casting.rs::handle_cast_spell` — Casualty (CR 702.153a) | no | `trigger_default`. |
| T5 | `rules/casting.rs::handle_cast_spell` — Replicate (CR 702.56a) | no | `trigger_default`. |
| T6 | `rules/abilities.rs::flush_sorted` — `PendingTriggerKind::Modular` arm | **YES** | `stack_obj.targets = modular_targets` (one artifact creature, CR 702.43a). |
| T7 | `rules/abilities.rs::flush_sorted` — main flush | **YES** | `stack_obj.targets = trigger_targets.clone()` (CR 603.3d; the PB-DP8 answer or the engine default). **This is the reported defect's site.** |

**Announcement reachability of T7**: `flush_sorted` is the single funnel. Both
`flush_pending_triggers` and `resume_trigger_flush` (the PB-DP8 suspend/answer path) call it,
so instrumenting `flush_sorted` covers both the auto-default and the human-answered paths
with one edit.

### 3.4 Roll-up

**26 emission sites** (5 + 14 + 7), of which **8 can carry targets**: S1, A1, A3, A4, A12,
A13, T6, T7. The other 18 are structurally target-free today, each for a stated reason above.

**The five files are the closed set**: `rules/casting.rs`, `rules/abilities.rs`,
`rules/copy.rs`, `rules/resolution.rs`, `rules/engine.rs`. §4.3's second check pins that.

---

## 4. The primitive

### 4.1 The new variant

**File**: `crates/engine/src/rules/events.rs`, appended after `EffectChoiceRequired`
(the current tail).

```rust
/// CR 601.2c / 602.2b / 603.3d: the targets chosen for a spell or ability as it
/// is put on the stack.
///
/// Emitted **only when `targets` is non-empty** — CR 115.1 makes targeting
/// optional ("*Some* spells and abilities require their controller to choose one
/// or more targets"), and an empty announcement is noise in every game.
///
/// `source_object_id` is the same object the sibling announcement event names —
/// `SpellCast.source_object_id` (the card's new object in `ZoneId::Stack`),
/// `AbilityActivated.source_object_id` / `AbilityTriggered.source_object_id`
/// (the source permanent). It is NEVER the stack-object id: a `StackObject` id
/// is not in `state.objects()`, so `event_view` could never name it (see the
/// `SpellCast` note in `crates/view-model/src/event_view.rs`).
///
/// Targets are public: CR 115.1 declares them as part of putting the object on
/// the stack, and the stack is a public zone (CR 400.2). `private_to()` is
/// therefore `None`. The *identity* of an object target may still be private
/// (CR 708.2 — a face-down permanent), which is a per-FIELD verdict the
/// per-EVENT `private_to()` cannot express; `event_view`'s `card_or` gate is
/// where that is decided.
TargetsAnnounced {
    /// CR 601.2f / 602.2b / 603.3a: the controller of the spell or ability.
    controller: PlayerId,
    /// The object whose spell/ability this is, in a zone where it can be named.
    source_object_id: ObjectId,
    /// The `StackObject` this announcement belongs to.
    stack_object_id: ObjectId,
    /// CR 601.2c: the announced targets, in declaration order.
    targets: Vec<crate::state::targeting::SpellTarget>,
},
```

**Field-name rationale (a deviation from the brief, stated deliberately)**: the brief writes
`controller`, and that is kept even though `SpellCast`/`AbilityActivated` say `player` —
`controller` is the CR-accurate word for all three kinds (CR 601.2f, 602.2b, 603.3a) and this
one event serves all three.

**No kind discriminator field.** The rendering (§4.4) needs none, and adding one would create
a second, differently-shaped claim about information the sibling event already carries.

`private_to()` and `reveals_hidden_info()` need **no new arm** — both have `_ =>` defaults
(`None` / `false`) that are correct here. Record the decision in the plan's commit message;
do not add a redundant arm.

### 4.2 The single shared helper

**File**: `crates/engine/src/rules/events.rs` (beside the variant it constructs — one file to
read to understand the contract).

```rust
/// CR 601.2c / 602.2b / 603.3d — build the announcement for whatever the stack
/// object at `stack_object_id` actually carries. `None` when it carries nothing.
pub(crate) fn announce_targets(
    state: &GameState,
    controller: PlayerId,
    source_object_id: ObjectId,
    stack_object_id: ObjectId,
) -> Option<GameEvent>;

/// The one-line call form used at every site.
pub(crate) fn push_target_announcement(
    state: &GameState,
    events: &mut Vec<GameEvent>,
    controller: PlayerId,
    source_object_id: ObjectId,
    stack_object_id: ObjectId,
);
```

**Why the helper reads the stack object rather than taking a `Vec<SpellTarget>` argument.**
Three reasons, in order of weight:

1. **It makes the event and the stack view incapable of disagreeing.** `StackItemView.targets`
   (view-model) is built from `stack_object.targets`. If the announcement is built from the
   same field, "what the log says was targeted" and "what the stack shows is targeted" are the
   same data by construction — the exact discrepancy G7 is about.
2. **S1 has already moved its `spell_targets`.** In `handle_cast_spell`, `spell_targets` is
   *moved* into the `StackObject` literal roughly 200 lines before the `SpellCast` push. A
   by-value helper would force a defensive `.clone()` at the top of the function, kept alive
   across two hundred lines of unrelated cost-payment code — a maintenance hazard that a
   future edit will get wrong. Every one of the 8 announcing sites pushes its `StackObject`
   **before** its announcement event (verified individually), so the lookup always succeeds.
3. **It makes the gate simple**: one call shape, one argument list, greppable.

**Fallback behaviour**: if the stack object is not found (which the 8 call sites make
unreachable), return `None` and route the surprise through
`state::diagnostics`'s `expect_*` family, not the `lki_*` family — a missing stack object one
line after pushing it is an engine bug, not an LKI fizzle (SR-4).

### 4.3 The machine-checkable gate

**File**: `crates/engine/tests/primitives/pb_eng2_targets_announced.rs`
**Test**: `every_announcement_site_is_classified`
**Precedent**: `no_condition_evaluator_resolves_characteristics_directly` (PB-DX19,
`crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs`) — a source walk that
brace-matches function bodies out of `CARGO_MANIFEST_DIR`-relative source and asserts a
property of each.

**What it walks.** The five files of §3.4, read as strings from
`concat!(env!("CARGO_MANIFEST_DIR"), "/src/rules/<file>.rs")`.

**Part 1 — the site census (catches a new site in a known file).**
For each file, scan comment-stripped lines (`line.split("//").next()`) for
`events.push(` on the same line as one of
`GameEvent::SpellCast {`, `GameEvent::AbilityActivated {`, `GameEvent::AbilityTriggered {`
(matching the optional `crate::rules::events::` prefix). For each hit, walk backward to the
nearest line whose trimmed-left form starts with `fn `, `pub fn `, `pub(crate) fn ` or
`async fn ` **at column 0** and take the function name; disambiguate repeats within one
function with an occurrence index. Build the sorted key set
`"<file>::<fn>#<n> -> <Variant>"` and `assert_eq!` it against a pinned
`EXPECTED_SITES: [&str; N]` const, one line each, **each carrying an inline
`// targets: yes|no — <reason>` comment**.

Red when: a new emission site appears, an existing one is deleted, or one moves function.
The failure message tells the author to classify the new site as `ANNOUNCES` or
`NEVER_TARGETS` and to say why.

**Part 2 — the helper is called (catches a classified site that forgot the helper).**
For each key in the `ANNOUNCES` subset (S1, A1, A3, A4, A12, A13, T6, T7 — the runner
re-derives this from Part 1's scan, not from this prose), brace-match the enclosing function
body and assert it contains `push_target_announcement(`.

**Part 3 — a `NEVER_TARGETS` site has not quietly grown targets.**
For each key in the `NEVER_TARGETS` subset, assert its enclosing function body contains none
of `stack_obj.targets =`, `targets: spell_targets`, `targets: vec![SpellTarget`. A site that
starts carrying targets reddens here and must be reclassified into `ANNOUNCES`.

**Part 4 — the file set is closed (catches a new site in a SIXTH file).**
Walk **all** of `crates/engine/src/` recursively for the same three-variant `events.push`
pattern and assert the set of *files* containing one equals the pinned five. Without this,
Parts 1–3 are blind to a brand-new module — which is the shape of failure PB-DX19's
pattern-replacement actually hit.

**Non-vacuity (the SR-5 "assert the denominator" rule).**
- assert the raw scan found `>= 26` hits before any classification;
- assert every `EXPECTED_SITES` entry was matched by a real hit (no stale entries);
- assert `ANNOUNCES` is non-empty and `NEVER_TARGETS` is non-empty;
- assert all five brace-matched bodies in Part 4's file set are non-empty.

**Proven red by revert**: delete the `push_target_announcement(` call from
`handle_activate_forecast` ⇒ Part 2 red. Add a sixth-file emission ⇒ Part 4 red. These two
reverts are mandatory evidence (§7).

### 4.4 The `event_view` arm

**File**: `crates/view-model/src/event_view.rs`, in the "Stack tier" block after the
`AbilityTriggered` arm.

```
Some(n) => "{n} targets {t1}, {t2}, …"
None    => "{name(controller)} targets {t1}, {t2}, …"
```

where each `ti` is:
- `Target::Player(pid)` ⇒ `name(pid)` — **unconditional**. CR 102.1 + CR 115.1 + CR 400.2 (see
  the citation correction in §2); there is no rule that makes a player's identity hidden, and
  `player_display_name` is already used unconditionally by every other arm.
- `Target::Object(id)` ⇒ `card_or(*id, "a permanent")` — through the entitlement gate. CR
  708.2: a face-down permanent has no name to reveal. CR 402.1 / 401.2 cover the (rare)
  hidden-zone target.

**Deviation from the brief's suggested strings, stated and justified**: the brief writes
`"alice's spell targets a creature"` for the unnameable-source case. This plan renders
`"alice targets a creature"` instead, because that is **byte-identical to the shipped
`PermanentTargeted` arm's own fallback** — one sentence shape for one concept, and no new
prose form for a reader to learn. The nameable case, `"Fell Specter targets bob"`, matches
the brief exactly.

Fallback noun: `"a permanent"` matches the existing `PermanentTargeted` arm. It is
technically imprecise for a graveyard or stack target, but it is the shipped word for this
concept and consistency beats precision in a name-free degradation. (If the reviewer prefers
`"an object"`, that is a defensible change — but change **both** arms or neither.)

`player:` field ⇒ `Some(controller)`. `tier` ⇒ **`EventTier::Stack`**, which requires an
explicit entry in `event_tier`'s Stack list — **the compiler will not ask for it** (§1.4).

**A measured cost this batch accepts, rather than hides.** For a cast or activation targeting
a battlefield permanent, the feed now carries **two** lines about the same target:
`PermanentTargeted` ("alice targets Grizzly Bears", Card tier) and `TargetsAnnounced`
("Lightning Bolt targets Grizzly Bears", Stack tier). The three `PermanentTargeted` emitters
(§1.2) are a **strict subset** of the 8 announcing sites, and its battlefield-object filter is
a strict subset of the announced targets — so every `PermanentTargeted` is accompanied by a
`TargetsAnnounced` naming the same target. Deleting the `PermanentTargeted` **prose** arm is
therefore safe and is the right follow-up, but it is a display deletion with browser
verification attached and MR-M11-01's lesson is that such a change must not ride inside
another batch. Mitigation available today: UI-3's tier filter puts the two lines in different
buckets. **Filed as `OOS-ENG2-9` with the superset proof, so the follow-up does not have to
re-derive it.**

### 4.5 Rider taken: the `TargetsChanged` arm

`GameEvent::TargetsChanged` (CR 115.7, emitted by `Effect::ChangeTargets`) has **no**
`event_view` arm — it falls to the kind-only floor and renders the bare string
`"TargetsChanged"`, tiered `Game`. That is the same defect class G7 reports, in the one
target-bearing event that already existed. Add a prose arm rendering
`"{controller-less} targets change: {old} → {new}"` — the runner should read the variant's
fields (it has no `controller`, only `stack_object_id` + `old_targets` + `new_targets`) and
pick the sentence — with every object target through `card_or` and every player target
through `name`, and add it to the `event_tier` **Stack** list.

**Cost**: zero wire, zero PROTOCOL, zero HASH, one arm + one tier entry. **Justification**:
it is the same rendering helper, the same gate, and the same review; splitting it out would
cost a whole second batch for one match arm.

### 4.6 Rider taken: the CR 108.1 citation fix

Three comment-only edits (§2): `redact.rs`'s module table row, `redact.rs`'s `redact_stack`
loop comment, `event_view.rs`'s `attack_target` comment. Replace `CR 108.1` with
`CR 102.1 / 115.1 / 400.2`. Zero behaviour. Do **not** add a fourth site with the bad cite.

---

## 5. Copy semantics (CR 707.10)

**Decision: copies do NOT announce. The two `copy.rs` `SpellCast` sites are not copies, and
they announce nothing today because they carry no targets.**

Three distinct things get conflated here; the recon's framing (#5, "decide whether `copy.rs`'s
`SpellCast` sites announce") assumes they are copies. They are not.

1. **The actual copy path** is `rules/copy.rs::copy_spell_on_stack`, which builds
   `StackObject { targets: original.targets.clone(), is_copy: true, … }` — CR 707.10 faithfully
   ("A copy of a spell or ability copies … **all decisions made for it, including … targets**")
   — and returns `GameEvent::SpellCopied`, **not** `SpellCast`. It is therefore not an
   announcement site at all and appears nowhere in §3. **It must not announce**: CR 707.10
   says in terms that "a copy of a spell isn't cast", so there is no CR 601.2c announcement to
   report. The copy's targets are already visible on the stack via `StackItemView.targets`
   (the half of G7 the triage correctly **REFUTED**).
2. **The two `copy.rs` `SpellCast` sites (S2, S3)** are the cascade (CR 702.85a) and discover
   (CR 701.57a) **free-casts**. Those *are* casts — CR 601.2 applies — so they are legitimate
   announcement sites and the helper is wired into them for correctness-under-change. They
   emit nothing today because both hardcode `targets: vec![]`, which is itself a latent gap
   (`OOS-ENG2-3`). **Routing them through the helper now means that when `OOS-ENG2-3` is
   closed, the announcement comes for free and cannot be forgotten.**

   *Correction to §3.4's classification for the gate*: S2/S3/S4/S5 sit in `NEVER_TARGETS`
   because Part 3 asserts they do not set targets — which is exactly the assertion that
   reddens when `OOS-ENG2-3` is fixed, forcing reclassification into `ANNOUNCES`. That is the
   gate working. The runner may instead put S2/S3/S4/S5 in `ANNOUNCES` and call the helper
   unconditionally; both are defensible. **Pick one, and write the reason at the const.**
3. **Storm / gravestorm / casualty / replicate copies** are created at *resolution* of their
   keyword trigger, through `copy_spell_on_stack` — case 1. Their `AbilityTriggered` events
   (T1–T5) are the *trigger going on the stack*, and those triggers have no targets.

---

## 6. Downstream inventory

Determined by reading each crate, not by assumption.

### 6.1 Compile-breaking (exhaustive matches over `GameEvent`) — 1 site

| File | Match | Action |
|---|---|---|
| `crates/engine/src/state/hash.rs` | `impl HashInto for GameEvent` — **exhaustive**, no `_` arm; ends at discriminant 131 (`EffectChoiceRequired`) | Add arm, discriminant **132**: `132u8`, then `controller`, `source_object_id`, `stack_object_id`, `targets` (`Vec<SpellTarget>` already has `HashInto`, established at the `TargetsChanged` arm, disc 126) |

That is the **only** exhaustive match on `GameEvent` in the workspace. Everything else has a
catch-all, which is the real hazard: the batch's downstream work is mostly things the
compiler will not ask for.

### 6.2 Silent-default sites the compiler will NOT flag — 4 sites

| File | Mechanism | Action | Required? |
|---|---|---|---|
| `crates/view-model/src/event_view.rs` — the rendering match | `_ => (kind.clone(), None)` | Add the prose arm (§4.4) | **YES** — without it the line reads `"TargetsAnnounced"` |
| `crates/view-model/src/event_view.rs` — `event_tier` | `_ => EventTier::Game` | Add to the Stack list | **YES** — without it the line lands in the wrong bucket |
| `tools/tui/src/play/app.rs::format_event` | `_ => String::new()` | Add an arm mirroring the view-model prose (the TUI has no redaction layer and is omniscient by design) | recommended — the TUI is the second human-facing surface; without it the event is invisible there (`OOS-ENG2-8`) |
| `tools/replay-viewer/frontend/src/lib/eventFormat.js::formatEvent` | `switch (key) { … default: … }` | Optional arm; this renders raw serialized `GameEvent` for the omniscient dev tool | optional |

### 6.3 Confirmed-zero surfaces

- **`tools/play-server/src/` — ZERO source change. CONFIRMED, not assumed.**
  `Grep "GameEvent::"` over `tools/play-server/src` returns **four hits, all inside doc
  comments**. `api.rs`'s seat-view builder does
  `.filter_map(|ev| event_view_for(ev, state, &session.names, viewer))` and nothing else;
  `view.rs` imports `EventView` as an opaque DTO. The triage's claim holds.
- **`tools/play-server/frontend/` — ZERO change. CONFIRMED.** `EventFeed.svelte` renders
  `ev.text`, filters on `ev.tier` (with a documented `'game'` default for a missing tier), and
  styles via `tone(ev.kind)`, a substring matcher with a fallback. Its own header states the
  rule ("`tier` is assigned in `event_view`… explicitly not matched on `kind` here"). Adding
  `TargetsAnnounced` to `tone()` for colour is optional polish, not a requirement.
- **`crates/simulator/` — ZERO change. CONFIRMED.** `Grep "GameEvent::"` returns two hits,
  both `matches!(e, GameEvent::ManaPoolsEmptied)` in one test file.
- **`crates/engine/src/testing/replay_harness.rs` — ZERO change.** Its four
  `SpellCast`/`PermanentTargeted` hits are prose comments. There is no string-keyed event
  dispatch anywhere in the workspace: `Grep '"SpellCast"|"AbilityTriggered"|…'` over the whole
  tree returns exactly one hit, an assertion in `crates/view-model/src/tests.rs`. `event_kind`
  is serde-derived, so no name table needs an entry.
- **`docs/mtg-engine-game-scripts.md` / SR-9c `check_assertions`** — no new assertion path is
  introduced, so no golden script changes and no partition move.

### 6.4 Version constants

| File | Action |
|---|---|
| `crates/engine/src/rules/protocol.rs` | `PROTOCOL_VERSION` 34 → read-from-gate; new `- N:` History line; **append** a `ProtocolEpoch` row; set `PROTOCOL_SCHEMA_FINGERPRINT` to the same value |
| `crates/engine/tests/core/protocol_schema.rs` | `protocol_version_sentinel` + `FROZEN_HISTORY_PREFIX_DIGEST` |
| `crates/engine/src/state/hash.rs` | `HASH_SCHEMA_VERSION` 71 → read-from-gate; new `- N:` History line; **append** a `HashSchemaEpoch` row with both recomputed fingerprints |
| `crates/engine/tests/core/hash_schema.rs` | `hash_schema_version_sentinel` + `FROZEN_HISTORY_PREFIX_DIGEST` |
| ~56+ `assert_eq!(HASH_SCHEMA_VERSION, 71u8)` / `assert_eq!(PROTOCOL_VERSION, 34)` sentinels across `crates/engine/tests/` | Re-pin **by symbol**, never by count — see §8 |

---

## 7. Test plan

New file: `crates/engine/tests/primitives/pb_eng2_targets_announced.rs`, registered in the
existing `crates/engine/tests/primitives/` group module (SR-9a: **never** add a top-level
`tests/*.rs`; a dropped `mod` line silently deletes coverage).

Card fixtures are named below as *candidates verified by reading their defs*. The runner must
confirm each is `Completeness::Complete` by **enumerating `all_cards()`** (SR-36), not by
grepping — the roster test in §7.7 is the vehicle.

### (a) A spell cast targeting a **player**

**Fixture**: `lightning_bolt.rs` — `AbilityDefinition::Spell` with
`targets: vec![TargetRequirement::TargetAny]`, `Complete` by the `..Default::default()`
derive. `TargetAny` admits `Target::Player`.

`test_eng2_spell_cast_targeting_a_player_announces` — cast Bolt at p2 through
`process_command`; assert the event slice contains, in order, `SpellCast { player: p1, .. }`
then `TargetsAnnounced { controller: p1, targets, .. }` with `targets == [SpellTarget {
target: Target::Player(p2), zone_at_cast: None }]`. **Proves**: the class the reported defect
belongs to (a *player* target, which `PermanentTargeted` cannot express at all).
**Red by revert**: remove `push_target_announcement` from `handle_cast_spell`.

### (b) An activated ability targeting a **battlefield object**

**Fixture**: `rogues_passage.rs` — ability index 1 is
`{4},{T}: target creature can't be blocked`, `targets: vec![TargetRequirement::TargetCreature]`,
`Complete` by derive. Its ability index 0 is a mana ability and is *not* an announcement site,
which makes the def double as a negative control.

`test_eng2_activated_ability_targeting_a_permanent_announces` — activate index 1 at a
creature; assert `AbilityActivated` **and** `TargetsAnnounced` **and** `PermanentTargeted`
all present, and that `TargetsAnnounced.targets[0].target == Target::Object(creature)` and
`PermanentTargeted.target_id == creature`. **Proves**: the two channels agree, i.e. the
display event does not contradict the Ward event.
**Red by revert**: remove the helper call from `handle_activate_ability`.

### (c) A **triggered** ability targeting a player — the Fell Specter class, the actual defect

**Fixture**: `fell_specter.rs` — `Complete` by derive;
`AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield,
targets: vec![TargetRequirement::TargetOpponent], effect: Effect::DiscardCards { player:
DeclaredTarget { index: 0 }, .. } }`. Verified by reading the def and by MCP oracle lookup
("When this creature enters, target opponent discards a card").

`test_eng2_fell_specter_etb_announces_its_target` — put Fell Specter onto the battlefield
under p1 in a 4-player state, flush triggers. Because PB-DP8 makes the CR 603.3d target
choice blocking, the flush **suspends**; answer it with the engine's own default via the
`answer_pending_trigger_targets` helper pattern established in
`crates/engine/tests/primitives/pb_dp8_trigger_target_choice.rs` (reuse it — do not
reimplement). Assert the resumed event slice contains `AbilityTriggered` **and**
`TargetsAnnounced { controller: p1, source_object_id: <the Specter>, targets: [Target::Player(pX)] }`.

Also assert, as an explicit **negative**, that the slice contains **no**
`GameEvent::PermanentTargeted` — pinning `OOS-ENG2-1` (§10) as a *recorded deviation*, so the
batch that closes it must come here and invert this assertion. (The PB-DX19
`deviation_animated_nexus_does_not_count_toward_metalcraft` precedent: pin the wrongness
wrong-way-round with an instruction to the successor.)

Then render it: `event_view_for(&targets_announced_event, &state, &names,
Viewer::Seat(human))` and assert `text == "Fell Specter targets <bob>"`. **This is the
end-to-end proof that the human playtester's complaint is closed.**
**Red by revert**: remove the helper call from `flush_sorted`'s main arm — and note that
reverting only the `event_view` arm produces `"TargetsAnnounced"`, which is a *different*
red and should also be executed once, because it is the failure the compiler cannot catch.

**Also assert the PB-DP8 human-answer path**: drive a **non-default** answer through
`Command::ChooseTriggerTargets` (a 4-player state has ≥2 opponents, so `TargetOpponent` has a
real choice) and assert `TargetsAnnounced` names the seat the human picked, not the default.
Without this, the test cannot distinguish "the engine announced" from "the engine announced
its own default".

### (d) Object-target redaction, proven **both** ways

**Location**: extend the existing table in
`crates/view-model/src/tests.rs::test_event_view_does_not_leak_a_card_moved_into_a_hidden_zone`.
That table already carries a non-vacuity step (`Viewer::Omniscient` must name the card first)
and the golden fixture already contains a **face-down "Exalted Angel" owned by bob** on the
battlefield — exactly the CR 708.2 case.

New case: `TargetsAnnounced { controller: alice, source_object_id: <Shock, in alice's
graveyard — nameable by everyone>, stack_object_id: ObjectId(9_00x), targets: vec![SpellTarget
{ target: Target::Object(<the face-down Angel>), zone_at_cast: Some(ZoneId::Battlefield) }] }`

- `Viewer::Omniscient` ⇒ text contains `"Exalted Angel"` (non-vacuity);
- `Viewer::Seat(bob)` (owner, entitled) ⇒ `"Shock targets Exalted Angel"`;
- `Viewer::Seat(alice)` (not entitled) ⇒ `"Shock targets a permanent"`.

Second new case, in the same style, for the **player** half: a `TargetsAnnounced` whose target
is `Target::Player(bob)` renders `"… targets bob"` for **every** seat including a foreign one
— CR 102.1 / 115.1 / 400.2. A test that only proves redaction happens, and never proves it
does *not* happen where CR says it must not, is half a test.

Third: add `TargetsAnnounced` to
`crates/view-model/src/tests.rs`'s tier table (the `(GameEvent, EventTier, &str)` cases),
asserting `EventTier::Stack` and the wire string `"stack"` — this is the only thing that
catches §6.2's silent-default hazard.

### (e) A non-targeting announcement emits **nothing**

`test_eng2_a_nontargeting_cast_announces_nothing` — cast a creature spell with no targets
(any vanilla creature in the golden fixture; `grizzly_bears` is present in the view-model
fixture and the engine test helpers build creatures directly) and assert the slice contains
`SpellCast` and **zero** `TargetsAnnounced`.

`test_eng2_a_nontargeting_activation_announces_nothing` — activate `rogues_passage`'s mana
ability (index 0): assert no `TargetsAnnounced` (and, as a control, that a mana ability emits
no `AbilityActivated` either — CR 605.3, mana abilities do not use the stack).

**These two are the tests that give the "emitted only when non-empty" clause teeth.** Without
them the clause is a comment.

### (f) The hash arm's own bytes

`test_eng2_targets_announced_hashes_its_targets` — direct `HashInto` unit test, following the
established precedent in
`crates/engine/tests/primitives/primitive_pb_oos_lki_power_3.rs` (`fn hash_event(ev) { let mut
h = Hasher::new(); ev.hash_into(&mut h); … }`). Three `TargetsAnnounced` values differing only
in `targets` (`[]` is unreachable in practice but hashes; `[Player(p1)]`; `[Player(p2)]`;
`[Object(o1)]`) must be pairwise distinct.

**Why this test is mandatory and cannot be replaced by the SR-17 stream fingerprint.** The
`stream_fingerprint` is computed over `canonical_fixture()`, a `GameState`. `GameEvent`
reaches the hash stream only via `PendingTrigger.triggering_event: Option<GameEvent>`, and a
`TargetsAnnounced` will never be a trigger's triggering event. So — unlike ENG-1, whose review
Finding 1 forced the fixture to carry a `Discard` question — **there is no way to put this
arm's bytes inside `stream_fingerprint`**, and it will move only by the v40
version-sentinel mechanism. The `decl_fingerprint` *does* move (a new variant inside the
`GameState` serde closure, since `history: Vector<GameEvent>` puts `GameEvent` in
`CLOSURE_MUST_CONTAIN`). **Exact precedent: HASH v58 / PB-OS6, whose History line records
`GameEvent` gaining `RemovedFromCombat` with "decl_fingerprint MOVES … stream_fingerprint
moves per the v40 mechanism."** Write that reasoning into the new `- N:` History line so the
next reader does not hunt for a fixture that cannot exist.

### (g) The gate's own reverts

Two mandatory executed reverts for §4.3, each run red then restored:
1. delete `push_target_announcement(` from `handle_activate_forecast` ⇒ Part 2 red;
2. add a throwaway `events.push(GameEvent::SpellCast { … })` in a sixth engine file
   (e.g. `rules/combat.rs`) ⇒ Part 4 red.

### (h) The play-server probe — the UI-4/SIM-6 lesson

`tools/play-server/src/main.rs` — one HTTP probe asserting a `TargetsAnnounced` line reaches
the seat payload for a targeted cast, i.e. that the browser really receives
`{"kind":"TargetsAnnounced","tier":"stack","text":"… targets …"}`. This is a **test-only**
addition; it does not contradict §6.3's zero-source-change finding. UI-4 and SIM-6 both
demonstrated that a wire proven below the browser is not proven at the browser; ENG-2's
deliverable is a line a human reads.

### (i) Roster / non-vacuity gate

`test_eng2_announcement_roster` — enumerate `all_cards()` (SR-36) and count
`Completeness::Complete` defs that declare a non-empty `targets` on any
`AbilityDefinition::{Spell, Activated, Triggered}` across front / `back_face` /
`adventure_face`. `assert!(roster.len() >= <floor>)` with a `>=`, not an `==` (the authoring
campaign grows continuously; PB-DP8's roster test is the model, including its `println!` of
the sorted names). Choose the floor from the measured value minus a margin, and say so at the
assertion.

### Test-count expectation

Roughly **+14 to +18** engine/view-model tests plus **+1** play-server probe. The runner
measures the real number with `--workspace --no-fail-fast` **to a file** — never `| tail`
(2026-08-02: a tail pipe hid a compile failure and faked a green run).

---

## 8. Version-bump ritual

**Both values are read from the failing gates' own output. Neither is predicted.** CARDS-1
found a dispatch brief's "PROTOCOL 32" already stale; this plan's own recon found the
triage's "PROTOCOL currently 33" stale. The prediction *"PROTOCOL 34 → 35, HASH 71 → 72"* is
written here only so a mismatch is noticed — if the gate says something else, the gate wins
and the plan is wrong.

### 8.1 PROTOCOL — the procedure quoted from `rules/protocol.rs`'s own doc block

> "To change the wire protocol, in one commit:
>  1. bump `PROTOCOL_VERSION` and add its `- N:` History line above;
>  2. **append** a new row here whose `fingerprint` is the recomputed digest (read it from the
>     `protocol_schema.rs` failure text) and set `PROTOCOL_SCHEMA_FINGERPRINT` to the same
>     value;
>  3. update the `protocol_version_sentinel` and the FROZEN prefix digest in
>     `protocol_schema.rs`.
>
> Never edit an existing row."

Run `cargo test -p mtg-engine --test core protocol_schema` **first**, read the digest out of
the failure text, then perform 1–3.

The `- N:` History line must say: `GameEvent` (a wire frame) gains `TargetsAnnounced
{ controller, source_object_id, stack_object_id, targets: Vec<SpellTarget> }`. **The closure's
type count is unchanged (96)** — `SpellTarget` entered the closure at v29 (PB-DP8) and
`PlayerId`/`ObjectId` long before; only `GameEvent`'s declared shape moves.

### 8.2 HASH — the procedure quoted from `state/hash.rs`'s `HASH_SCHEMA_HISTORY` doc block

> "This table is **append-only**. To change the hash schema:
>  1. bump `HASH_SCHEMA_VERSION` and add its `- N:` History line above;
>  2. **append** a new row here with the two new fingerprints (read them from the
>     `hash_schema.rs` failure message);
>  3. update the `HASH_SCHEMA_VERSION` sentinels the suite still carries.
>
> Never edit an existing row."

Run `cargo test -p mtg-engine --test core hash_schema` first; read **both** digests
(`decl_fingerprint` and `stream_fingerprint`) from the failure text.

The `- N:` History line must record the §7(f) reasoning: `decl_fingerprint` MOVES (a new
variant on `GameEvent`, which is inside the `GameState` serde closure via
`history: Vector<GameEvent>`, a `CLOSURE_MUST_CONTAIN` entry); `stream_fingerprint` moves
**per the v40 mechanism only** — the fixture cannot exercise the new arm's own bytes because
`GameEvent` reaches the hash stream only through `PendingTrigger.triggering_event` and a
`TargetsAnnounced` is never a triggering event. Cite HASH v58 / PB-OS6 as precedent. **Do not
repeat ENG-1's review Finding 1 mistake by hunting for a fixture slot that does not exist —
add the §7(f) direct `HashInto` test instead, and say in the History line that that is where
the arm's bytes are proven.**

### 8.3 The sentinel re-pin — by symbol, never by count

There are **56+** `assert_eq!(HASH_SCHEMA_VERSION, 71u8)` and `assert_eq!(PROTOCOL_VERSION,
34)` sentinels scattered across `crates/engine/tests/`. PB-DX5 and PB-DX6 both found this the
expensive part and both found stragglers.

1. `Grep` for the **symbols** `HASH_SCHEMA_VERSION` and `PROTOCOL_VERSION` across the whole
   worktree — not for the literals `71` / `34`, which miss multi-line assertion forms and
   `71u8` vs `71` spelling variants (both spellings exist today:
   `pb_dx6_unflattened_payment_sites.rs` uses `71`, everything else `71u8`).
2. Re-pin every hit.
3. **Confirm by executing a full `cargo test --workspace --no-fail-fast` with output captured
   to a file** and grepping that file for residual sentinel failures — PB-DX6's residual list
   came back empty, PB-DX5's did not, so the run is evidence and the grep is not.

### 8.4 Coverage must be proven unmoved

Zero card-def lines change. Prove it by regenerating `tools/authoring-report.py` and
confirming a **byte-identical** report body, in addition to an empty
`git diff -- crates/card-defs`.

---

## 9. Explicitly out of scope

1. **The "cards sections" highlight ask.** The notes' *"targeting should be part of the stack
   and cards sections"*: the stack half is **REFUTED** (already implemented —
   `StackItemView.targets` is built, redacted per-target and rendered by `ZoneStack.svelte`).
   The cards half is a derived `PermanentView` field computed from `state.stack_objects()`,
   **needs no engine change at all**, and is UI work. Filed as `OOS-ENG2-6`. Do not build it.
2. **The "events hard to parse" feed redesign.** Dispositioned in the triage as *"partially
   addressed, no separate task"* — UI-3 shipped the 3-tier feed against it. If legibility is
   still poor after ENG-2 lands, it needs a **fresh observation**, not a re-file of that line.
3. **Ward on triggered abilities (CR 702.21a).** `OOS-ENG2-1` / `OOS-ENG2-2`. Changes game
   outcomes; moves fuzz and golden parity. §7(c) pins the current wrong behaviour
   wrong-way-round with an instruction to the successor.
4. **`OOS-ENG2-3`** — cascade / discover / cipher-copy / suspend free-casts put spells on the
   stack with `targets: vec![]` unconditionally.
5. **Deleting the `PermanentTargeted` prose arm** (`OOS-ENG2-9`), for the MR-M11-01 reason.
6. **`OOS-M11-10`** — the loyalty-ability targeting gap. This batch *announces* loyalty targets
   (A13) but does not touch that seed's substance.

---

## 10. Seeds to file

| ID | Finding | Severity |
|---|---|---|
| `OOS-ENG2-1` | CR 702.21a: `flush_sorted` emits no `PermanentTargeted`, so **Ward never fires on a triggered ability**. Pinned wrong-way-round by §7(c). | MEDIUM |
| `OOS-ENG2-2` | Same class, four more sites the recon missed: `handle_activate_forecast`, `handle_scavenge_card`, `rules/engine.rs`'s loyalty handler, and `flush_sorted`'s modular arm all carry targets and emit no `PermanentTargeted`. Ward silently does not fire for any of them. | MEDIUM |
| `OOS-ENG2-3` | CR 601.2c: cascade (`copy.rs`), discover (`copy.rs`), cipher-copy and suspend free-cast (`resolution.rs`) all hardcode `targets: vec![]`, so a free-cast targeted spell goes on the stack with no targets. Three of the four admit it in an in-source comment. | MEDIUM |
| `OOS-ENG2-4` | `GameEvent::TargetsChanged` (CR 115.7) had no `event_view` arm and rendered as the bare kind string. **CLOSED by the §4.5 rider.** | — |
| `OOS-ENG2-5` | Three shipped comments cite **CR 108.1** (the Oracle-text rule) for "a player target is public". **CLOSED by the §4.6 rider**; correct chain is CR 102.1 / 115.1 / 400.2. | — |
| `OOS-ENG2-6` | The "cards sections" ask — a derived `PermanentView` "currently targeted" field. No engine change needed. | LOW |
| `OOS-ENG2-7` | `event_tier` is non-exhaustive by design, so every new `GameEvent` variant silently lands in the `Game` tier with a bare-kind line and nothing asks. Propose (do not build) a ratchet: assert the *classified* variant count only ever grows. | LOW |
| `OOS-ENG2-8` | `tools/tui/src/play/app.rs::format_event`'s `_ => String::new()` silently drops any new event from the TUI play log. **Mitigated for this variant by §6.2; the class stands.** | LOW |
| `OOS-ENG2-9` | The feed now carries two lines per battlefield-object target (`PermanentTargeted` + `TargetsAnnounced`). Superset proof recorded in §4.4 so the follow-up can delete the `PermanentTargeted` prose arm without re-deriving it. | LOW |

---

## 11. Verification checklist

- [ ] Baselines re-measured on the branch **before any edit** and recorded (PROTOCOL, HASH,
      full-workspace test count to a file)
- [ ] `TargetsAnnounced` added to `rules/events.rs`; `private_to()` / `reveals_hidden_info()`
      decisions recorded (no arm added, defaults are correct)
- [ ] `announce_targets` / `push_target_announcement` added; **all 8 announcing sites** call it
- [ ] All 26 emission sites classified in the gate's `EXPECTED_SITES`, each with a reason
- [ ] `every_announcement_site_is_classified` passes, **and both mandatory reverts executed red**
- [ ] `state/hash.rs` arm added, discriminant 132
- [ ] `event_view` prose arm + `event_tier` Stack entry (the compiler asks for neither)
- [ ] `TargetsChanged` rider arm + tier entry (§4.5)
- [ ] CR 108.1 citation rider (§4.6), three comments
- [ ] TUI `format_event` arm (§6.2)
- [ ] Tests (a)–(i) written; each named revert executed red then restored
- [ ] PROTOCOL bumped by the §8.1 ritual, digest read from the gate
- [ ] HASH bumped by the §8.2 ritual, both digests read from the gate, History line carries
      the §7(f) reasoning and the PB-OS6/v58 precedent
- [ ] Every `HASH_SCHEMA_VERSION` / `PROTOCOL_VERSION` sentinel re-pinned **by symbol**, then
      confirmed by a full `--workspace --no-fail-fast` run captured **to a file**
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35)
- [ ] `cargo build --workspace` (the SR-3 seal gate, and the thing that catches a missed
      `KeywordAbility`/`StackObjectKind` match — not applicable here, but run it)
- [ ] Coverage proven unmoved by a byte-identical `tools/authoring-report.py` regeneration
- [ ] Play-server probe (§7(h)) green; ideally one live browser confirmation that a bot's
      targeted trigger now names its target in the feed
- [ ] Benches re-run (`full_turn_4p`, `priority_cycle_4p`) — expect noise-level; the helper is
      one stack lookup per announcement, not per priority cycle
- [ ] Seeds `OOS-ENG2-1..9` filed; `OOS-G7-1` closed with the stack-half refutation restated

---

## 12. Risks & edge cases

1. **Event ordering.** The plan emits `TargetsAnnounced` **immediately after** its sibling
   announcement event. At S1, A1 and A4 that inserts it *between* the announcement and the
   `PermanentTargeted` loop. Ward is unaffected (`check_triggers` scans the whole slice, not
   an index), but any test asserting index adjacency of `SpellCast` and `PermanentTargeted`
   would redden. **Run the full suite and, if one reddens, record it as a finding rather than
   silently reordering the emission** — the ordering choice is "announce, then detail", and it
   should survive a test that merely encoded the old adjacency.
2. **Golden-script and fuzz parity.** Adding an event to every targeted cast/activation/trigger
   changes event-slice lengths everywhere. Golden scripts assert on state and named
   assertions, not slice indices (SR-9c), so the expected blast radius is zero — but
   `OOS-UI2-1` warns that the fuzzer has never cast a spell, so **fuzz parity is not evidence
   about this batch** and must not be cited as such.
3. **The stack lookup at S1.** `handle_cast_spell` pushes its `StackObject` roughly 100 lines
   before the emission, with mutation of `state` in between (commander tax, face-down flags,
   `spells_cast_this_turn`). None of it touches `stack_objects`, verified — but the helper
   must be called with `stack_entry_id`, not `new_card_id`, and swapping the two silently
   yields `None` at every cast. Test (a) discriminates.
4. **The `event_tier` silent default.** The single most likely way this batch ships subtly
   wrong: everything compiles, the line renders, and it lands in the `Game` tier next to the
   turn markers. Only the §7(d) tier-table case catches it.
5. **`Vec<SpellTarget>` ordering.** CR 601.2c announcements are positional (PB-AC4's
   `validate_targets_positional` depends on it), so the announcement must preserve declaration
   order. It does by construction (`stack_obj.targets` is the same vec), but the rendering
   must **not** sort for readability.
6. **A target in a hidden zone.** `zone_at_cast` can in principle be `Hand`/`Library`. No
   printed card targets there, but `card_or` handles it correctly via `viewer_may_identify`,
   and `private_to()` correctly stays `None` because the *event* is public even when a *name*
   inside it is redacted — the per-field/per-event distinction `event_view.rs`'s module doc
   already draws.
7. **PB-DP8 interaction.** A suspended trigger batch announces on the *resumed* flush, so the
   `TargetsAnnounced` for a Fell Specter arrives in the events returned by
   `Command::ChooseTriggerTargets`, not by the command that caused the ETB. §7(c) must assert
   against the resumed slice, or it will assert against an empty one and pass for the wrong
   reason.
8. **ENG-1 interaction.** Fell Specter's *resolution* now suspends again for
   `EffectChoiceQuestion::Discard` (ENG-1, CR 701.9b). §7(c) only needs the announcement, so it
   should stop before resolution — but a test that drives the trigger to resolution will hit a
   second blocking decision and must answer it.
9. **Feed noise.** §4.4's duplicate-line cost, mitigated by the tier split and filed as
   `OOS-ENG2-9`. It is the one place this batch makes the feed marginally busier while making
   it correct.
