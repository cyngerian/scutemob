# Primitive Batch Plan: PB-DX25b — the announced-target → stack-entry id space

**Generated**: 2026-08-05
**Primitive**: one shared engine-side resolution of *"which stack object does this **announced
target id** name"*, encoded ONCE in `state::stack_registry`, consumed by every site that takes a
declared target and looks it up in `GameState::stack_objects`.
**Seed**: `OOS-DX25-3` (registry row `docs/audits/decision-point-audit.md:1364`)
**Queue**: `memory/primitives/seed-rerank-2026-08-02.md` §4 row **7b** (INSERTED 2026-08-06,
user-approved)
**CR rules**: 115.7 / 115.7a / 115.7b / 115.7d / 115.7e, 601.2a, 601.2c, 608.2b, 707.10 /
707.10a / 707.10c, 400.7, 115.10 (cited here to **refute** an existing citation)
**Cards affected**: **2 live-wrong `Complete` deck-legal defs repaired** (`misdirection`,
`bolt_bend`); 1 `partial` def partially unblocked (`untimely_malfunction` mode 1); 1 further def
reaches the same code (`deflecting_swat`) with **no observable change** — see §2.4.
**Dependencies**: PB-DX25 (`scutemob-203`) — `state::stack_registry` and its
`card_in_stack_zone` classification are the substrate this batch extends.
**Deferred items from prior PBs**: none inherited that this batch is obliged to take. The
standing undispatched set (feedback rows 2/4/5/6/7/8, `OOS-DX22-8`, `OOS-DX32-1`, v3 §4 not
re-rowed with DX42a/b, `OOS-ADJ-1..7` not rowed into §8.1, `scutemob-127`) is unchanged and out
of scope.

**Baseline to re-measure BEFORE any edit** (do not trust these — re-measure and record):
workspace tests **4,452 / 0 / 5**; PROTOCOL **35**; HASH **73**; coverage **1,133/1,803 =
62.8%**.

---

## §1 Premise re-verification

Every fact in the dispatch brief was independently checked against source at HEAD. Line numbers
are this worktree's (`/home/skydude/projects/scutemob/.worktrees/scutemob-204`).

| # | Brief's claim | Verdict | Evidence |
|---|---|---|---|
| 1 | `validate_object_satisfies_requirement` opens with `state.objects.get(&id).ok_or(ObjectNotFound)?`, so the announced id must be a CARD id | **CONFIRMED**, with two name/line corrections | `crates/engine/src/rules/casting.rs:6417` is the `fn` signature (the brief and the registry row both call it `validate_target_requirement`, which **does not exist** — the real name is `validate_object_satisfies_requirement`); the lookup is at `:6426-6429`, not `:6417`. Registry row `:1364` says `casting.rs:6426` for the *function*; that is the lookup line, not the fn. |
| 2 | `:6476` and `:6502` search the **stack-entry** id space; `casting.rs:4423-4425` mints two distinct ids from one monotone counter | **CONFIRMED, stronger than stated** | `casting.rs:6476` and `:6502` are both `state.stack_objects.iter().find(\|so\| so.id == id)`. `casting.rs:4423` `let (new_card_id, _old_obj) = state.move_object_to_zone(card, ZoneId::Stack)?;` → `state/mod.rs:1305` mints `new_id = self.next_object_id()`. `casting.rs:4425` `let stack_entry_id = state.next_object_id();`. Both draw `self.timestamp_counter += 1` at `state/mod.rs:1012-1015`, so the two ids are **consecutive and never equal**, and the counter is never rewound — so no *later* object can ever occupy a retired stack-entry id either. The comparison is therefore impossible on any real cast: `is_spell` is always `false`, `target_count` always `0`, both requirements always `InvalidTarget`. |
| 3 | `TargetSpell` / `TargetSpellWithFilter` work off `obj.zone == ZoneId::Stack` on the CARD, and are the working precedent | **CONFIRMED, and there is a stronger piece of evidence the brief did not cite** | `casting.rs:6431-6454`. **`casting.rs:6308-6311`** — 170 lines *above* the defect, in the same function's caller — already states the invariant in prose: *"Object targets are always looked up in state.objects. Spells on the stack are also in state.objects (zone == ZoneId::Stack); **StackObject entries in state.stack_objects have separate IDs used internally by the engine, not as targets**."* The defect at `:6476`/`:6502` directly contradicts a comment in its own file. |
| 4 | `queries::legal_targets_per_slot` enumerates candidates from `state.objects()` only, so stack-entry ids are never offered | **CONFIRMED** | `crates/engine/src/rules/queries.rs:230-237` — candidates are `state.objects()` filtered to `Battlefield \| Stack \| Graveyard(_)`, plus players. The doc at `:187-196` explicitly names `TargetSpellOrAbilityWithSingleTarget` / `TargetSpellWithSingleTarget` as requiring `Stack`, i.e. the offer layer already assumes the card-id space. |
| 5 | There is a **second live site**: `effects/mod.rs:7520 Effect::ChangeTargets` at `:7528`, `:7542`, `:7634` | **CONFIRMED — the brief is right and the seed row is wrong.** The registry row `:1364` describes this as validation-site only. | `effects/mod.rs:7524` `resolve_effect_target_list(state, target, ctx)` on `DeclaredTarget { index: 0 }` returns the announced **card** id; `:7528` `state.stack_objects.iter().any(\|s\| s.id == stack_obj_id)` → `false` → `:7530` `continue`. A validation-only fix ships **announce-and-silently-no-op**, which is strictly worse than "cannot be announced" because the mana is spent. Both sites must land in one commit. |
| 6 | The reviewed precedent is `effects/mod.rs:2745` (`Effect::CounterSpell`): `so.id == id \|\| (!so.is_copy && card_in_stack_zone(&so.kind) == Some(id))`, `!so.is_copy` load-bearing (CR 707.10, cipher-copy exile leak) | **CONFIRMED** | `effects/mod.rs:2745-2750`; rationale at `:2771-2786`. |
| 7 | `stack_registry::card_in_stack_zone` is exhaustive with no wildcard; its doc and `casting.rs:6503-6513` both forbid using it to answer *"is this a spell"*; using it for the **lookup** is its purpose; do not weaken those comments | **CONFIRMED** — and I counted the arms: **27**, no wildcard, matching the brief. | `crates/engine/src/state/stack_registry.rs:69-110`; module doc `:25-34` (the "NOT is it a spell" note) and `:36-48` (the deliberate-duplication note w.r.t. the simulator). |
| 8 | The ability half is unreachable for an independent, deeper reason: an activated ability's stack entry gets no `state.objects` entry | **CONFIRMED** | `crates/engine/src/rules/abilities.rs:1381` `let stack_id = state.next_object_id();` → `:1396` `StackObject::trigger_default(stack_id, …)` → `:1415` `push_back`. There is **no** `add_object` / `objects.insert` on that path. So the offer layer cannot enumerate it (fact 4) and the validator's opening `state.objects.get(&id)` rejects it with `ObjectNotFound` before any of the single-target logic runs. **OUT OF SCOPE**, filed as a seed (§8 R1). |
| 9 | Third site `effects/mod.rs:7490 Effect::CopySpellOnStack` (`:7495`), *"only two corpus defs use it (`plumb_the_forbidden` partial, `complete_the_circuit` Complete-by-derive)"* | **SITE CONFIRMED. LIVENESS CLAIM REFUTED — the brief is WRONG, and wrong in an instructive way.** | The site is real: `effects/mod.rs:7495` `state.stack_objects.iter().any(\|s\| s.id == stack_obj_id)` on a resolved announced target, feeding `copy::copy_spell_on_stack` which itself does `.find(\|s\| s.id == stack_object_id)` (`copy.rs:150`). But **neither named def constructs `Effect::CopySpellOnStack`**. `crates/card-defs/src/defs/plumb_the_forbidden.rs` mentions it only inside its `Completeness::partial(...)` **prose** (`:42`); `complete_the_circuit.rs` mentions it only in a **Rust comment** (`:6`) and its two abilities are `Keyword(Convoke)` and `Effect::GrantFlash`. The grep that produced the brief's claim matched English, not code — the SR-36 rule ("enumerate `all_cards()`, never grep source") exactly. **Corpus usage of `Effect::CopySpellOnStack` is believed to be ZERO**; the runner must confirm this by enumeration in R3 (§5.3), not by trusting this paragraph. |
| 10 | `effects/mod.rs:7691` and `~:7960` are already correct and must not be "fixed" | **CONFIRMED** | `:7690-7692` accepts an id present in *either* space (`exists_in_objects \|\| exists_on_stack`) — this is what lets Ward's stack-entry-id target resolve at all, and narrowing it would break Ward. `:7955-7963` (`PlayerTarget::ControllerOf`) tries `state.objects` then falls back to `stack_objects`. Both stay byte-unchanged. |
| 11 | The existing tests hide the bug by fixture construction; `casting.rs:8150 make_test_stack_spell` collapses the id spaces; `casting.rs:8209`, `:8291` and `tests/primitives/pb_ef11_spell_single_target.rs` all pass vacuously | **CONFIRMED, with one correction and one addition** | `casting.rs:8150-8158` builds `StackObject { id, kind: Spell { source_object: id } }` — both spaces collapsed onto one id. Both in-src tests (`:8210`, `:8292`) and `pb_ef11_spell_single_target.rs`'s `build_base_state` (`:197` `make_stack_object(other_id, …)` with `source_object: other_id`) inherit the collapse. **Correction**: they are not *all* negative tests — `pb_ef11`'s `test_spell_single_target_accepts_single_target_spell` (`:238`) is a **positive** test that passes only because of the collapse; it is the single most misleading artefact in the tree, because it reads as proof the primitive works. **Addition**: `pb_ef11`'s `test_misdirection_retargets_single_target_spell` (`:372`) does *not* collapse the spaces — it announces the **stack-entry** id directly into `execute_effect`, i.e. it tests a path no cast can ever produce. It is green, and it is testing a fiction. |

**Two further findings not in the brief** (see §2 for the full census):

* **F-A — `deflecting_swat` also reaches the `ChangeTargets` site**, via `TargetRequirement::TargetSpell`
  (`crates/card-defs/src/defs/deflecting_swat.rs:39`), which *does* validate today (fact 3). It is
  therefore a `Complete`, deck-legal def that can announce a target and then silently do nothing.
  Its observable behaviour is nevertheless **unchanged** by this batch, because its
  `must_change: false` makes `effects/mod.rs:7533-7537` `continue` anyway — two different
  `continue`s, same outcome. Recorded so nobody claims it as a flip and nobody is surprised when
  it stays a no-op (§8 R4).
* **F-B — `CR 115.10` is mis-cited in two places** as the authority for self-targeting
  prevention. See §4.4.

---

## §2 The census: every site that matches an *announced target id* against *stack-entry ids*

### 2.1 Method (so the next reader can re-derive it, not trust it)

PB-DX25's durable lesson is *"an enumeration is only as wide as the variant list it walks"* — its
own plan's census was short by two. This census is built **backwards from the id's provenance**,
not from a `grep '\.id =='`:

1. An *announced target id* can enter the engine at exactly two places:
   `Command::CastSpell.targets` → `casting::validate_targets_positional` → `SpellTarget`
   (`casting.rs:6187`+), and `Command::ActivateAbility.targets` → the same helper. It is stored on
   `StackObject.targets` and read back at resolution.
2. At resolution the **only** reader of `StackObject.targets` that yields an `ObjectId` to effect
   code is `effects::resolve_effect_target_list` / `_indexed`
   (`effects/mod.rs:7657` / `:7669`). Therefore: **enumerate every caller of those two functions**
   (`Grep resolve_effect_target_list` in `crates/engine/src` → 48 call sites, all in
   `effects/mod.rs`), then ask of each whether the resulting `ResolvedTarget::Object(id)` is fed to
   a `state.stack_objects` lookup.
3. Cross-check by grepping `stack_objects` in `effects/mod.rs` (6 regions) and in every other
   engine file (`rules/casting.rs`, `rules/resolution.rs`, `rules/copy.rs`, `rules/abilities.rs`,
   `rules/events.rs`, `rules/sba.rs`, `rules/engine.rs`, `state/*`) and classifying each hit by
   **where its id came from**. A site whose id is minted internally (`stack_id`,
   `targeting_stack_id`, `original_stack_id`, `stack_object_id` from a `pop_back`) is NOT a member
   of this class and must not be "fixed".
4. Additionally sweep the two `TargetRequirement` variants' own validation arms in `casting.rs`,
   which are announced-id consumers that never go through `resolve_effect_target_list` at all
   (this is the class step 2 alone would miss — the same shape as PB-DX25's missed
   `abilities.rs:6736`).

### 2.2 IN CLASS — must change (4 lookups, 3 regions, 2 files)

| # | File:line | Expression | Id's provenance | Consequence at HEAD |
|---|---|---|---|---|
| C1 | `rules/casting.rs:6476` | `stack_objects.iter().find(\|so\| so.id == id)` | announced (`TargetSpellOrAbilityWithSingleTarget`) | `target_count` always 0 → `InvalidTarget` always. `bolt_bend`, `untimely_malfunction` mode 1 |
| C2 | `rules/casting.rs:6502` | `stack_objects.iter().find(\|so\| so.id == id)` | announced (`TargetSpellWithSingleTarget`) | `is_spell` always false → `InvalidTarget` always. `misdirection` |
| C3 | `effects/mod.rs:7528` + `:7542` + `:7634` | `.any(\|s\| s.id == …)`, `.find(…)`, `.iter_mut().find(…)` | announced, via `resolve_effect_target_list` at `:7524` | `Effect::ChangeTargets` silently `continue`s. **Live for every card in C1/C2 the moment C1/C2 are fixed** |
| C4 | `effects/mod.rs:7495` | `.any(\|s\| s.id == stack_obj_id)`, then `copy::copy_spell_on_stack(state, stack_obj_id, …)` (`copy.rs:150` repeats the same comparison) | announced, via `resolve_effect_target_list` at `:7491` | `Effect::CopySpellOnStack` silently creates no copies. **Believed latent — 0 corpus defs (§1 fact 9); confirm by enumeration** |

### 2.3 ALREADY CORRECT — must not change

| File:line | Why it is right |
|---|---|
| `effects/mod.rs:2745` (`Effect::CounterSpell`) | Already carries the full rule. It is *re-expressed* through the new helper in §3, which is a refactor, not a behaviour change. `Effect::CounterUnlessPays` (`effects/mod.rs:4411`) delegates into this same arm and is covered for free — PB-DX25's review found this delegation the hard way; do not re-discover it. |
| `effects/mod.rs:7690-7692` | Deliberately accepts **both** spaces (`exists_in_objects \|\| exists_on_stack`). Ward's trigger target is a stack-entry id, and this is the only reason it resolves. Narrowing it breaks Ward. |
| `effects/mod.rs:7955-7963` (`PlayerTarget::ControllerOf`) | objects-then-stack fallback, same reason. |
| `rules/resolution.rs:8337` (`counter_stack_object`) | Its parameter **is** a stack-entry id by contract, supplied by its (test-only) callers. Routing it through the announced-target helper would widen it to also match card ids — a semantic change on a `pub` API, on a path this batch has no business touching. **Explicitly declined**; see §3.4. |
| `rules/copy.rs:150`, `copy.rs:289` (`create_storm_copies`) | Genuine stack-entry ids from storm / casualty (`TriggerData::CasualtyCopy { original_stack_id }`, `resolution.rs:2562`) / replicate. Only the C4 caller passes an announced id, so C4 is fixed at the **call site**, not inside `copy.rs`. |
| `rules/abilities.rs:6747` (`collect_permanent_becomes_target_triggers`) | `targeting_stack_id` is minted at `abilities.rs:1381` / `casting.rs:4425`. Internal id. Its `is_spell` two-variant `matches!` is CR-correct and must stay (CR 707.10). |
| `rules/events.rs:1594`, `rules/resolution.rs:8337` (`pop_back` path), `casting.rs:306` (miracle `revealed_card` scan), `casting.rs:7135` (`has_split_second_on_stack`, already routed through `card_in_stack_zone`) | All internal ids or already-registry-driven. |

### 2.4 Corpus reach (recon — **must be re-measured by enumeration**, §5.3)

| Def | Requirement / effect | `completeness` | Status at HEAD | After PB-DX25b |
|---|---|---|---|---|
| `misdirection` | `TargetSpellWithSingleTarget` + `ChangeTargets{must_change:true}` | `Complete` (derive) | **live-wrong** — every cast with a target is refused | works (player-target case; see §8 R2) |
| `bolt_bend` | `TargetSpellOrAbilityWithSingleTarget` + `ChangeTargets{must_change:true}` | `Complete` (derive) | **live-wrong** — same | works for **spells**; abilities remain unreachable (§8 R1) |
| `untimely_malfunction` mode 1 | `TargetSpellOrAbilityWithSingleTarget` + `ChangeTargets` inside `ModeSelection.modes[1]` | `partial` (for an unrelated mode-2 gap) | mode 1 refused | mode 1 should work — **verify the modal target index**, §8 R5 |
| `deflecting_swat` | `TargetSpell` + `ChangeTargets{must_change:false}` | `Complete` (derive) | announces fine, resolves to nothing | **still resolves to nothing** (`must_change:false` fallback) — F-A |
| any `Effect::CopySpellOnStack` def | — | — | believed **none** | n/a |

**Predicted `completeness` flips: 0.** `misdirection` and `bolt_bend` are already `Complete`; this
batch makes the marker *true* rather than changing it. Coverage must be **proven unmoved by
regenerating `tools/authoring-report.py` to a byte-identical body**, not by an empty card-defs
diff (PB-DX25's own standard).

---

## §3 The structural fix

### 3.1 The new primitive

**File**: `crates/engine/src/state/stack_registry.rs` (alongside `card_in_stack_zone`, which it
consumes).

```rust
/// The index in `stack_objects` of the stack object an **announced target id**
/// names, if any (CR 601.2c).
///
/// Two disjoint id spaces meet here, and the whole point of this function is that
/// the decision is made once:
///
/// * a **card** id — `casting.rs::handle_cast_spell` moves the card into
///   `ZoneId::Stack` (CR 601.2a) with a fresh `ObjectId` (CR 400.7), and that is
///   the id the offer layer (`rules::queries::legal_targets_per_slot`) enumerates
///   and the player announces (CR 601.2c). `casting.rs:6308-6311` states this
///   invariant in prose.
/// * a **stack-entry** id — `state.next_object_id()`, one line later. It is never
///   in `state.objects`, so it can never be announced by a player, but it IS
///   passed as a target by engine-internal triggers (Ward, CR 702.21a, via
///   `PermanentTargeted`/`targeting_stack_id`).
///
/// Both are minted from the one monotone `timestamp_counter`
/// (`state/mod.rs:1012-1015`), so an id lives in exactly one of them and a bare
/// `so.id == announced` type-checks while being unsatisfiable — `OOS-SIM3-5` and
/// `OOS-DX25-3` are the same defect, two functions apart.
///
/// CR 707.10: a COPY of a spell owns no card of its own. `copy.rs` clones the
/// ORIGINAL's `kind` wholesale, so a copy's `source_object` names the ORIGINAL's
/// card; without the `!so.is_copy` guard a single card id would match BOTH the
/// original and every copy of it, and `position` would silently return whichever
/// came first. The guard is therefore load-bearing twice over: for
/// disambiguation here, and for the CR 702.99c cipher-copy exile leak documented
/// at `Effect::CounterSpell`'s call site.
///
/// **This is not "is it a spell".** See this module's header: a copy IS a spell
/// (CR 707.10) and is deliberately NOT findable here. A consequence, stated
/// rather than hidden: a copy of a spell can never be the announced target of
/// Misdirection/Bolt Bend (`OOS-DX25b-2`).
pub fn stack_index_for_announced_target(
    stack_objects: &imbl::Vector<super::stack::StackObject>,
    announced: ObjectId,
) -> Option<usize>
```

Body — the rule, written exactly once:

```rust
stack_objects.iter().position(|so| {
    so.id == announced
        || (!so.is_copy && card_in_stack_zone(&so.kind) == Some(announced))
})
```

**Why an index and not `Option<&StackObject>`**: `Effect::CounterSpell` needs
`state.stack_objects.remove(pos)`; `Effect::ChangeTargets` needs a shared read *and* an
`iter_mut()` write; `casting.rs` needs a shared read. An index serves all three with one
function. Two functions (`_index` + `_ref`) would be two places a future edit could diverge, which
is the thing this batch exists to stop.

**Why a `&Vector<StackObject>` parameter and not `&GameState`**: keeps `stack_registry` a pure
classification module with no dependency on `GameState`, matching `card_in_stack_zone`. The extra
`imbl` import is `use super::stack::StackObject;` plus `imbl::Vector`.

### 3.2 Consumers (all four, in ONE commit)

| Site | Edit |
|---|---|
| `effects/mod.rs:2745` `Effect::CounterSpell` | Replace the open-coded `position(...)` closure with `crate::state::stack_registry::stack_index_for_announced_target(&state.stack_objects, id)`. The explanatory comment block `:2732-2744` is **kept and extended** (it now points at the shared helper for the rule and keeps the CR 702.21a / 707.10 prose). `Effect::CounterUnlessPays` is covered by delegation. |
| `casting.rs:6476` (C1) | `let stack_obj = stack_index_for_announced_target(&state.stack_objects, id).map(\|i\| &state.stack_objects[i]);` |
| `casting.rs:6502` (C2) | Same. The `is_spell` two-variant `matches!` at `:6514-6519` **stays**, and its comment block `:6503-6513` is **extended, not weakened** — see §3.3. |
| `effects/mod.rs:7524` region (C3) | Resolve the index **once** at the top of the loop body: `let Some(pos) = stack_index_for_announced_target(&state.stack_objects, stack_obj_id) else { continue };`. Then `:7528`'s `is_on_stack` bool disappears, `:7542`'s `find` becomes `state.stack_objects[pos].clone()`, and `:7634`'s `iter_mut().find` becomes `state.stack_objects.get_mut(pos)`. **The `TargetsChanged` event must keep naming the STACK-ENTRY id** (`state.stack_objects[pos].id`), not the announced card id — `GameEvent::TargetsChanged.stack_object_id` is documented as a stack-object id and the view-model/replay consumers read it as one. This is a behaviour *correction* on a path that never fired, not a new field. |
| `effects/mod.rs:7491` region (C4) | Same index resolution; pass `state.stack_objects[pos].id` (the real stack-entry id) into `copy::copy_spell_on_stack`, **not** the announced id. `copy.rs` is untouched. |

**C4 — argued, not asserted.** PB-DX25's precedent was to preserve unrelated behaviour verbatim
and file a seed (`OOS-DX25-4`). That precedent does **not** transfer here, and the distinction is
worth stating: `OOS-DX25-4` was about *widening an observable event stream* on paths unrelated to
the defect — a change whose blast radius is other subsystems' outputs. C4 is a **pure widening of
a lookup on the identical defect class**: the `so.id ==` clause survives verbatim, so every id
that resolves today still resolves, and the only new behaviour is that an id which previously
matched nothing now matches the thing it names. Nothing observable can regress. Against that,
leaving C4 alone means shipping a batch whose whole thesis is "the rule is encoded once" while
knowingly leaving one of the four sites open-coded and wrong — which is precisely how
`resolution.rs::counter_stack_object` inherited PB-DX25's defect a second time and had to be
retro-fixed inside that same batch. **Include C4.** Because its corpus population is (believed)
zero, its probe is synthetic and it earns no `completeness` flip; say so in the test's own doc so
nobody reads the probe as evidence of card yield.

### 3.3 What the `is_spell` guard becomes, stated honestly

After the fix, `stack_index_for_announced_target` can return a **non-spell** stack object only via
the `so.id == announced` clause — which requires the announced id to be simultaneously a live
`state.objects` entry (the validator's opening `?`) and a stack-entry id. §1 fact 2 shows that
configuration is **unreachable in production**. Therefore:

* On every real cast, C2's `is_spell` check is **always true** — the guard is live code that can
  only fire on a hand-built fixture.
* Consequently `TargetSpellWithSingleTarget` and `TargetSpellOrAbilityWithSingleTarget` become
  **behaviourally identical on the production path**. That is not a defect introduced here: it is
  the visible shadow of §8 R1 (abilities are not announceable at all). Bolt Bend is under-powered,
  never over-permissive.
* **Do not delete the guard.** It is CR-correct, it is the only thing distinguishing the two
  variants, and it becomes load-bearing the day R1 is closed. Extend the comment at
  `casting.rs:6503-6513` with: (a) that the lookup now goes through
  `stack_index_for_announced_target` while the *classification* stays a local `matches!`, (b) that
  the guard is reachable only through the direct-id clause today, naming `OOS-DX25b-1`.
* Correspondingly, **do not delete the collapsed-id fixture** in `casting.rs`'s in-src test — it is
  now the only configuration that isolates this guard. §5.2 makes that explicit rather than
  letting a future reader "clean it up".

### 3.4 What is deliberately NOT unified

* **`mtg_simulator::invariants::stack_card_of`** (`crates/simulator/src/invariants.rs:151`) stays
  an independent, exhaustive re-implementation. `stack_registry.rs:36-48` states the reason and it
  applies unchanged: `check_stack_consistency` exists to catch the engine getting this
  classification wrong, and a verifier that reads the engine's answer goes silent on exactly the
  defect it was written for. **Zero simulator-source lines in this batch.**
* **`resolution.rs::counter_stack_object`** — see §2.3. Its parameter is a stack-entry id by
  contract; widening its lookup would erase the very distinction this batch draws. Its
  *classification* already routes through `card_in_stack_zone` (PB-DX25); its *lookup* stays
  `so.id ==`.
* **`copy.rs::copy_spell_on_stack`** — its other three callers pass genuine stack ids; fix the one
  bad caller, not the callee.

### 3.5 What a future change is forced to do

* A **28th `StackObjectKind`** is still a compile error in `card_in_stack_zone` until classified
  (unchanged, PB-DX25's forcing function) — and now that classification automatically governs
  announced-target lookup at four sites instead of two.
* A **new consumer** that takes a declared target and looks it up on the stack is caught by the new
  source gate R4 (§5.3), which forbids a bare `s.id ==` comparison inside the gated arms and
  requires a call to the helper. The gate cannot see a *brand-new* arm elsewhere in the file; that
  residual is stated in R4's own doc comment rather than papered over.

---

## §4 CR grounding

### 4.1 CR 115.7 — changing targets (verbatim, MCP)

> **115.7.** Some effects allow a player to change the target(s) of a spell or ability, and other
> effects allow a player to choose new targets for a spell or ability.
>
> **115.7a** If an effect allows a player to "change the target(s)" of a spell or ability, each
> target can be changed only to another legal target. If a target can't be changed to another legal
> target, the original target is unchanged, even if the original target is itself illegal by then.
> If all the targets aren't changed to other legal targets, none of them are changed.
>
> **115.7b** If an effect allows a player to "change a target" of a spell or ability, the process
> described in rule 115.7a is followed, except that only one of those targets may be changed
> (rather than all of them or none of them).
>
> **115.7c** … "change any targets" … any number of those targets may be changed …
>
> **115.7d** If an effect allows a player to "choose new targets" for a spell or ability, the
> player may leave any number of the targets unchanged, even if those targets would be illegal. If
> the player chooses to change some or all of the targets, the new targets must be legal and must
> not cause any unchanged targets to become illegal.
>
> **115.7e** When changing targets or choosing new targets for a spell or ability, only the final
> set of targets is evaluated to determine whether the change is legal.
>
> **115.7f** A spell or ability may "divide" or "distribute" an effect … the original division
> can't be changed.

Engine mapping: `must_change: true` = 115.7a (Misdirection, Bolt Bend, Untimely Malfunction mode
1); `must_change: false` = 115.7d (Deflecting Swat). **115.7a's "only to another legal target" is
NOT implemented for object targets** — `effects/mod.rs:7591-7626` picks the smallest `ObjectId` in
the recorded `zone_at_cast` with no requirement check, self-documented as a KNOWN LIMITATION at
`:7596-7601`. This batch makes that limitation *reachable for the first time* — see §8 R2. The
player-target branch (`:7555-7590`) does check CR 115.7a liveness (`has_lost`) and is sound.

### 4.2 CR 601.2a / 601.2c — announcement

> **601.2a** To propose the casting of a spell, a player first moves that card (or that copy of a
> card) from where it is to the stack. It becomes the topmost object on the stack. …
>
> **601.2c** The player announces their choice of an appropriate object or player for each target
> the spell requires. … The chosen objects and/or players each become a target of that spell. …

Engine mapping: the announced object id is a `state.objects` id — the invariant already written
down at `casting.rs:6308-6311` and enforced by `queries::legal_targets_per_slot`. **Pre-existing
CR deviation worth recording, not fixing here**: the engine validates targets *before* moving the
card to the stack (validation `casting.rs:6417`, move `casting.rs:4423`), inverting 601.2a/601.2c's
order. The only place it is observable in this batch is self-targeting (§4.4), where it produces
the correct answer for the wrong reason.

### 4.3 CR 608.2b — the resolution re-check

> **608.2b** If the spell or ability specifies targets, it checks whether the targets are still
> legal. A target that's no longer in the zone it was in when it was targeted is illegal. … If all
> its targets, for every instance of the word "target," are now illegal, the spell or ability
> doesn't resolve. It's removed from the stack and, if it's a spell, put into its owner's
> graveyard. …

Engine mapping: `resolution.rs:8281 is_target_legal` compares `state.objects[id].zone` against
`SpellTarget.zone_at_cast`, which `casting.rs:6345` records as `Some(obj.zone)` = `Some(Stack)` for
a spell target. So a victim spell that resolves or is countered before Misdirection does mints a
new `ObjectId` (CR 400.7) and the old id vanishes → `unwrap_or(false)` → fizzle. **Correct by
construction, and untested on this path until now** — §5.1 T4.

### 4.4 CR 707.10 / 400.7, and one citation this tree gets wrong

> **707.10** … A copy of a spell is itself a spell, even though it has no spell card associated
> with it. A copy of an ability is itself an ability.
> **707.10a** If a copy of a spell is in a zone other than the stack, it ceases to exist. …
> **400.7** An object that moves from one zone to another becomes a new object with no memory of,
> or relation to, its previous existence. …

**Mis-citation found (record it; do not silently fix behaviour).** `casting.rs:8282` and
`crates/engine/tests/primitives/pb_ef11_spell_single_target.rs:300` both cite **"CR 115.10"** as
the authority for self-targeting prevention. CR 115.10 (looked up via MCP) reads:

> **115.10.** Spells and abilities can affect objects and players they don't target. In general,
> those objects and players aren't chosen until the spell or ability resolves. …
> **115.10a** Just because an object or player is being affected by a spell or ability doesn't make
> that object or player a target …

That is the *affects-vs-targets* rule and has nothing to do with self-targeting. The correct
grounding is **CR 601.2a + 601.2c + 115.7a**: at the moment targets are announced the spell being
cast has chosen no targets yet, so it does not have "a single target" and is not an appropriate
object. Misdirection's own 2004-10-04 ruling — *"You can't make a spell which is on the stack
target itself"* — is about the **deflected** spell, not about Misdirection. **Action**: correct
both citations in place (comment-only, zero behaviour), same class as `OOS-DX25-6`'s phantom "CR
701.5g". Do **not** change the guard.

### 4.5 Rulings worth encoding as tests (MCP, authoritative for edge cases only — CR governs)

* Bolt Bend, 2024-11-08: *"If a spell or ability targets the same player or object multiple times,
  you can't target it with Bolt Bend."* → covered by `targets.len() != 1` (a duplicated target is
  two entries). Worth one probe.
* Bolt Bend, 2024-11-08: *"If a spell or ability targets multiple things, you can't target it …
  even if all but one of those targets have become illegal."* → covered; the count is structural,
  not legality-filtered. Already probed by `pb_ef11`'s Test 2.
* Misdirection, 2004-10-04: *"This does not check if the current target is legal. It just checks if
  the spell has a single target."* → the engine matches: `:6526` counts, it does not validate.
* Misdirection, 2004-10-04: *"You choose the spell to target on announcement, but you pick the new
  target for that spell on resolution."* → exactly the two-phase shape this batch repairs.

---

## §5 Test plan

New files:
* `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs` (+ `mod` line in
  `crates/engine/tests/primitives/main.rs`, SR-9a — a dropped `mod` line silently deletes coverage)
* `crates/engine/tests/core/pb_dx25b_announced_target_roster.rs` (+ `mod` line in
  `crates/engine/tests/core/main.rs`)

Modified test files: `crates/engine/src/rules/casting.rs` (in-src `mod tests`),
`crates/engine/tests/primitives/pb_ef11_spell_single_target.rs`,
`crates/engine/tests/core/pb_dx25_stack_registry_roster.rs` (G2 — see §7, this WILL go red).

### 5.1 Positive probes — real `Command::CastSpell`, real resolution (AC 6297)

Hard constraints, stated because the batch will be judged on them: **no direct call to
`validate_object_satisfies_requirement`** (it is private anyway from an integration test), and **no
hand-built `StackObject` whose `id` equals its `source_object`**. Every fixture spell reaches the
stack by being cast.

**T1 — `misdirection` announces AND resolves.** 3 players. Registry: the real
`cards::defs::misdirection::card()` plus one purpose-built victim instant
(`Effect::DealDamage { target: DeclaredTarget{0}, amount: Fixed(3) }`,
`targets: vec![TargetRequirement::TargetAny]`).
1. p2 casts Victim targeting **p3** via `Command::CastSpell`. Assert it lands on the stack.
2. Capture `victim_card_id` = the `ZoneId::Stack` object named "…Victim…" from `state.objects()`,
   and `victim_entry_id` = the `StackObject`'s own `id`. **Assert `victim_card_id != victim_entry_id`
   in the test itself** — this is the non-vacuity anchor: it proves the fixture did not collapse
   the spaces, and it is a `#[test]`-visible statement of the whole defect.
3. p1 casts `misdirection` announcing `Target::Object(victim_card_id)`. **Assert `Ok`.** *(This is
   the assertion that is red at HEAD.)*
4. Assert, before resolution, `state.stack_objects()` for the victim still shows
   `targets[0].target == Target::Player(p3)`.
5. Resolve Misdirection by real priority passes (`Command::PassPriority` from each player in APNAP
   order until the stack shrinks). **Observables that prove the resolve half:**
   * a `GameEvent::TargetsChanged { stack_object_id, old_targets, new_targets }` is emitted, with
     `stack_object_id == victim_entry_id` (**not** `victim_card_id` — pins §3.2's event-id
     decision), `old_targets[0] == Player(p3)`, `new_targets[0] == Player(p1)`;
   * the victim `StackObject` in `state.stack_objects()` now has `targets[0] == Player(p1)`.
6. Resolve the victim too and assert **p1's life total dropped by 3 and p3's is unchanged**. This
   is the end-to-end observable; an event-only assertion would pass against a fix that emitted the
   event and forgot the `iter_mut` write at `:7634`.

**T2 — `bolt_bend` announces AND resolves against a spell.** Same shape with
`cards::defs::bolt_bend::card()` and `TargetSpellOrAbilityWithSingleTarget`. Same six steps, same
life-total observable.

**T3 — the ability half does NOT work, pinned wrong-way-round.** Activate a targeted activated
ability (any fixture permanent with `AbilityDefinition::ActivatedAbility` carrying one
`TargetRequirement::TargetCreature`/`TargetAny`), then:
* assert `rules::queries::legal_targets_per_slot(state, p1, bolt_bend_id,
  &[TargetSpellOrAbilityWithSingleTarget])[0]` contains the victim **spell**'s card id and does
  **not** contain the ability's stack-entry id nor the ability's source permanent;
* assert casting `bolt_bend` at the ability's stack-entry id fails with
  `GameStateError::ObjectNotFound` (the `state.objects.get(&id)?` at `casting.rs:6426`);
* assert casting it at the ability's **source permanent** fails with `InvalidTarget` (zone check).
Message names **`OOS-DX25b-1`** and tells the reader this is a recorded deviation, and that closing
it needs a `Target::StackObject` id space (wire change), not a tweak here.

**T4 — CR 608.2b fizzle on the newly reachable path** (highest-effort probe; keep it). p2 casts
Victim at p3; p1 casts `misdirection` at the victim's card; p3 casts a counter (real
`cards::defs::counterspell` or equivalent) at the victim's card. LIFO: the counter resolves first
and removes the victim's card from `ZoneId::Stack` (new `ObjectId`, CR 400.7). Then Misdirection
resolves: assert a `GameEvent::SpellFizzled` naming Misdirection, **no** `TargetsChanged`, and
Misdirection's card in a graveyard. If assembling three casts proves too brittle, the *fallback* is
to counter the victim via `Effect::CounterSpell` executed directly — but say so in the test doc; do
not silently downgrade.

**T5 — CR 707.10: a copy of a spell is not announceable, pinned wrong-way-round.** Put a real copy
on the stack (`copy::copy_spell_on_stack` on a genuinely cast spell), then assert
`legal_targets_per_slot` for `TargetSpellWithSingleTarget` contains the **original**'s card id and
that a cast announcing the copy's stack-entry id fails. Message names **`OOS-DX25b-2`** and quotes
CR 707.10 ("a copy of a spell is itself a spell") so the deviation is legible.

**T6 — Bolt Bend's duplicate-target ruling.** Victim spell with the same player targeted twice
(`targets.len() == 2`) → `bolt_bend` cast is refused. Cites the 2024-11-08 ruling. Cheap, and it
pins the count guard on the *repaired* id space.

**T7 — the Ward clause still works (regression guard for the `so.id ==` half).** Ward is the only
production consumer of the direct-id clause. Drive a real Ward trigger (an existing Ward fixture
exists in the tree — reuse it rather than build one) and assert the ward `CounterSpell` still finds
its target through `stack_index_for_announced_target`. Without this, a "simplification" that drops
the `so.id ==` clause would pass every other probe in this file.

### 5.2 Non-vacuity repairs (AC 6298) — with the exact mutation that proves each discriminates

| Test | Repair | Mutation that must redden it |
|---|---|---|
| `casting.rs:8150 make_test_stack_spell` | Signature becomes `make_test_stack_spell(id, source_object, controller, targets)`; `kind: Spell { source_object }`. Both existing callers updated. | (helper — proven via its callers) |
| `casting.rs:8210 test_target_spell_single_target_self_targeting_prevented` | Mint `let entry_id = state.next_object_id();` (in-src test, `pub(crate)` is reachable) and build the entry as `make_test_stack_spell(entry_id, spell_id, p(1), …)` so `entry_id != spell_id`. Assert that in the test. The existing `result_no_self.is_ok()` assertion is now the discriminator. | Revert C2's lookup to `state.stack_objects.iter().find(\|so\| so.id == id)` → `result_no_self` becomes `Err(InvalidTarget("… has 0 targets …"))`. Confirmed red-at-HEAD by construction. |
| `casting.rs:8292 test_target_spell_with_single_target_self_and_kind_check` | **Two sub-cases, each labelled with what it isolates.** (i) *distinct ids* — the spell half, as above, discriminates C2's lookup. (ii) *collapsed ids* — **keep the existing `ability_state` block with `id == ability_stack_id`**, relabelled: this is now the ONLY configuration in the tree that reaches the `is_spell` guard with a non-spell actually found, and §3.3 explains why that configuration is unreachable in production. Add a third sub-case: an ActivatedAbility entry with a **distinct** id, asserting rejection with the same "is not a spell" message but for the *not-found* reason — documented as such, and explicitly documented as **not** discriminating `is_spell`. | (i) as above. (ii) delete the `is_spell` guard → collapsed sub-case returns `Ok` (count is 1). **Sub-case (iii) is documented as non-discriminating for `is_spell`; do not claim otherwise.** |
| `tests/primitives/pb_ef11_spell_single_target.rs::build_base_state` | `let entry_id = test_util::next_object_id(&mut state);` and `make_stack_object(entry_id, p2, kind_with_source_object(other_id), …)`. Return `(state, test_spell_id, other_id, entry_id)`; callers use `other_id` as the announced target. | `test_spell_single_target_accepts_single_target_spell` (`:238`) is red at HEAD after this repair — **this is the headline non-vacuity proof and must be executed and recorded.** Test 2 (`:269`) still discriminates the count guard. Test 3 (`:287`) no longer discriminates `is_spell` once ids are distinct; **say so in its doc and point at `casting.rs`'s collapsed sub-case**, do not leave the old claim standing (conventions.md: aspirationally-wrong comments are correctness hazards). |
| `pb_ef11 … :372 test_misdirection_retargets_single_target_spell` | Currently announces a **stack-entry** id into `execute_effect` — a path no cast can produce, so it is green while testing a fiction (§1 fact 11 addition). Rebuild it to place a real victim **card** object in `ZoneId::Stack` (`ObjectSpec::card(...).in_zone(ZoneId::Stack)`) and announce that id, with the stack entry carrying a distinct id and `source_object: victim_card_id`. | Revert C3's lookup at `:7528`/`:7542` → the effect `continue`s, no `TargetsChanged`, `bolt.targets[0]` unchanged. Red at HEAD after the repair. |
| `pb_ef11 … :331 test_spell_single_target_hash_discriminant` | No repair — but it hard-asserts `HASH_SCHEMA_VERSION == 73`. Leave it; §6 predicts no bump. If it moves, that is a signal, not a chore. | n/a |

**Rule for the runner**: every mutation in this table is to be **executed**, the rebuild confirmed
(`Compiling mtg-engine` present in captured output — a stale binary faking a pass is the PB-DX32
R7 class), the failure text recorded verbatim in
`memory/primitives/pb-DX25b-execution-notes.md`, and the revert restored with `git diff` confirmed
clean before the next one.

### 5.3 The roster / gate file (SR-36 idiom — enumerate `all_cards()`, never grep source)

`crates/engine/tests/core/pb_dx25b_announced_target_roster.rs`, modelled on
`crates/engine/tests/core/pb_dx25_stack_registry_roster.rs` (read it first; reuse its
`strip_comments` / `extract_match_arm_body` / `extract_function_body` idiom, including **both**
line- and block-comment stripping — PB-DX32 M8).

* **R1 — requirement roster.** Enumerate `all_cards()`; for each def walk **both faces**'
  `abilities` and each `AbilityDefinition`'s `targets` (and `ModeSelection.mode_targets` when
  present) for the two variants. Pin the exact NAME sets. Recon says
  `{Misdirection}` and `{Bolt Bend, Untimely Malfunction}` — **re-measure; do not hard-code from
  this plan.** Non-vacuity floor `all_cards().len() >= 1_700` asserted **in the same test**
  (PB-DX24 R2 lesson: a broken enumeration must not make an empty roster look correct).
* **R2 — `Effect::ChangeTargets` roster.** Same enumeration, walking the whole effect tree
  including `ModeSelection.modes`. Recon says
  `{Misdirection, Bolt Bend, Untimely Malfunction, Deflecting Swat}` — note Deflecting Swat, which
  the brief's site analysis missed. Message states that `must_change: false` remains a documented
  no-op (F-A) so a future reader does not read membership as "works".
* **R3 — `Effect::CopySpellOnStack` roster, expected EMPTY.** An empty pin needs a **walker-liveness
  control**, not just a corpus floor — PB-DX25's T6 advertised non-vacuity while comparing a
  hand-written fixture to itself, and this is the same trap. Assert in the same test that the
  identical walker returns a **non-empty** set for a control effect known to be common
  (e.g. `Effect::DrawCards`). Without that control an empty answer is indistinguishable from a
  broken walk.
* **Walker construction — mandated, with the trap named.** Do **not** implement R2/R3 as
  `format!("{:?}", def).contains("CopySpellOnStack")` on the raw def:
  `plumb_the_forbidden`'s `Completeness::partial(...)` **prose literally contains the string
  `Effect::CopySpellOnStack`** (`crates/card-defs/src/defs/plumb_the_forbidden.rs:42`), so a naive
  Debug scan false-positives on exactly the card the dispatch brief wrongly believed used it. Use
  either (a) a structural recursive walk over `Effect`, or (b) a Debug scan over a **sanitized**
  clone with `oracle_text` cleared on both faces and `completeness` set to `Complete`. Option (b)
  is preferred: it is total over the effect tree by construction and immune to a new recursive
  `Effect` variant, which a hand-written walker with a `_ => {}` arm is not. Whichever is chosen,
  the choice and its blind spot go in the file's doc comment.
* **R4 — source gate over the two `effects/mod.rs` arms.** After comment-stripping, the
  `Effect::ChangeTargets` and `Effect::CopySpellOnStack` arm bodies must each (a) contain
  `stack_index_for_announced_target` at least once, and (b) contain **zero** occurrences of
  `stack_objects.iter()` / `stack_objects.iter_mut()`. Plus a size floor per arm so a collapsed
  extraction cannot pass vacuously. Doc must state the residual honestly: **this gate sees only the
  arms it names**; a brand-new arm elsewhere in the file is invisible to it, exactly as PB-DX25's
  G2 was blind to `resolution.rs` until its review added G4.
* **R5 — the helper has no second implementation.** Scan `crates/engine/src/` (comment-stripped)
  for the literal rule shape `card_in_stack_zone(` appearing in the same expression as `so.id ==` /
  `s.id ==` outside `state/stack_registry.rs`; assert zero. This is what stops the next author
  re-open-coding it.

### 5.4 Existing tests expected to move

* `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs::g2_counter_spell_arm_does_not_reclassify_by_kind`
  **will go RED** — see §7 and the mandatory handling there.
* Golden scripts: none expected (no corpus def in the 208 approved scripts uses these
  requirements — verify with `SCRIPT_FILTER` rather than assuming). Do **not** start the
  replay-viewer HTTP server to check (gotchas-infra: SIGKILL 137).

---

## §6 Wire and hash analysis

**Prediction**: no new or changed `Command`, `GameEvent`, `Effect`, `TargetRequirement`,
`StackObjectKind`, `StackObject` field, or any other serialized shape. `stack_index_for_announced_target`
is a free function over existing types. Therefore **PROTOCOL 35 unmoved** and **HASH 73 unmoved**.

**This must be gate-EXECUTED, never predicted** — PB-DX20/21/23/24/25 all state this and PB-DX21's
prediction was the one that failed. Run and record:

```
cargo test -p mtg-engine --test core hash_schema
cargo test -p mtg-engine --test core protocol_schema
```

Named gates that decide it:
* `core::hash_schema::hash_schema_version_sentinel` — `assert_eq!(HASH_SCHEMA_VERSION, 73)`
  (`crates/engine/tests/core/hash_schema.rs:1249-1255`).
* `core::protocol_schema::protocol_version_sentinel` — `assert_eq!(PROTOCOL_VERSION, 35)`
  (`crates/engine/tests/core/protocol_schema.rs:876-879`).
* `core::protocol_schema::protocol_schema_fingerprint_is_pinned` (`:850`) — the digest gate; this is
  the one that catches a shape change the version sentinel would sleep through.
* `core::protocol_schema::protocol_closure_is_not_vacuous_and_is_bounded` (`:792`) and the tail-row
  agreement test (`:989`) run in the same target.

Also expected unmoved and to be executed: `cargo test -p play-server` (0 production lines in
`tools/`), and `git diff main..HEAD --numstat -- crates/simulator/ crates/view-model/
crates/card-types/ tools/` should be **EMPTY**. `crates/card-defs/` should be empty or
comment-only; if comment-only, coverage must be proven unmoved by **regeneration**
(`tools/authoring-report.py` → byte-identical body), not by the diff.

---

## §7 Revert matrix

Each row: the production edit, the exact mutation to apply, and the **named** test that must go
red. Every one is to be executed with the rebuild confirmed, and the verbatim failure text recorded
in `memory/primitives/pb-DX25b-execution-notes.md`.

| # | Production edit | Mutation | Test that reddens |
|---|---|---|---|
| V1 | `stack_registry::stack_index_for_announced_target` — the `card_in_stack_zone(...) == Some(announced)` clause | delete the clause (leave `so.id == announced`) | `pb_dx25b…::t1_misdirection_announces_and_resolves` (cast returns `Err`); also `pb_ef11…::test_spell_single_target_accepts_single_target_spell` |
| V2 | same helper — the `!so.is_copy` guard | delete the guard | `pb_dx25b…::t5_copy_is_not_announceable` flips to finding the copy; **and** PB-DX25's own `pb_dx25_counterspell_stack_shapes` copy probes must be re-run to confirm they still discriminate (this guard is shared now) |
| V3 | same helper — the `so.id == announced` clause | delete the clause | `pb_dx25b…::t7_ward_still_finds_its_target` |
| V4 | `casting.rs:6476` (C1) routed through the helper | restore `stack_objects.iter().find(\|so\| so.id == id)` | `pb_dx25b…::t2_bolt_bend_announces_and_resolves` |
| V5 | `casting.rs:6502` (C2) routed through the helper | restore the old `find` | `casting.rs`'s repaired in-src `test_target_spell_single_target_self_targeting_prevented` (spell half) + `pb_ef11…::test_spell_single_target_accepts_single_target_spell` |
| V6 | `casting.rs:6514` `is_spell` guard (unchanged, but now proven) | delete the guard | `casting.rs`'s in-src collapsed sub-case (ii) of `test_target_spell_with_single_target_self_and_kind_check` |
| V7 | `effects/mod.rs:7528`/`:7542` (C3 read half) | restore `.any(\|s\| s.id == …)` | `pb_dx25b…::t1…` life-total assertion; `pb_ef11…::test_misdirection_retargets_single_target_spell` |
| V8 | `effects/mod.rs:7634` (C3 write half) | restore `iter_mut().find(\|s\| s.id == stack_obj_id)` | `pb_dx25b…::t1…` — victim `StackObject.targets` unchanged **while** `TargetsChanged` still fires. This is the mutation that proves the event-only assertion is insufficient; record both. |
| V9 | `effects/mod.rs:7495` + the `copy_spell_on_stack` argument (C4) | restore `.any(\|s\| s.id == …)` and pass the announced id | `pb_dx25b…::t8_copy_spell_on_stack_finds_its_target` (synthetic — no corpus def; say so in the test doc) |
| V10 | `TargetsChanged.stack_object_id` now names the stack-entry id | emit the announced card id instead | `pb_dx25b…::t1…`'s `stack_object_id == victim_entry_id` assertion |
| V11 | R4 source gate | insert `state.stack_objects.iter().any(\|s\| s.id == x)` into the `ChangeTargets` arm | R4 |
| V12 | R4 gate's comment-stripping | wrap a required `stack_index_for_announced_target` call in `/* */` | R4 (must still redden — the PB-DX32 M8 block-comment class) |
| V13 | R3's walker-liveness control | make the walker return `BTreeSet::new()` unconditionally | R3 (the `DrawCards` control assertion) — proves the empty pin is not self-satisfying |
| V14 | R1 non-vacuity floor | (n/a — assert it discriminates by pinning a NAME set one member short) | R1 |

**Mandatory before/after A/B, executed:**
`git stash` the whole batch and run `cargo test -p mtg-engine --test primitives
pb_dx25b` → expect a **large** number of failures (the entire positive-probe file). Restore. This
is the batch's headline evidence: *at HEAD, zero of the positive probes pass.*

---

## §8 Risks, and what this batch does NOT deliver

**R1 — the ability half of Bolt Bend / Untimely Malfunction still does not work, and this batch
must not claim otherwise.** An activated (or triggered) ability's stack entry is minted at
`abilities.rs:1381` and never enters `state.objects`, so (a) `queries::legal_targets_per_slot`
cannot enumerate it and (b) `casting.rs:6426`'s `state.objects.get(&id).ok_or(ObjectNotFound)?`
rejects it before any single-target logic runs. Closing it requires a **new target id space** — a
`Target::StackObject(ObjectId)` variant or equivalent — which is a `Command`/`GameEvent` shape
change (PROTOCOL bump), plus offer-layer, view-model and frontend work. **File `OOS-DX25b-1`.**
Enforcement: T3 pins it wrong-way-round, and **no comment, test name, doc line or card-def note in
this batch may say Bolt Bend's "or ability" half works.** The runner should grep its own diff for
"or ability" before committing.

**R2 — this batch makes a pre-existing wrong-answer path REACHABLE for the first time, and that is
its single biggest hazard.** `effects/mod.rs:7591-7626`'s object-target redirect picks the
**smallest `ObjectId` in the recorded `zone_at_cast`** with **no** check that the new object
satisfies the original spell's `TargetRequirement` — CR 115.7a's "another **legal** target". It is
self-documented as a KNOWN LIMITATION at `:7596-7601` and has been unreachable, because nothing
could announce a target. After PB-DX25b, Misdirection pointed at a "destroy target creature" spell
can redirect it onto a land, a hexproof creature, or the caster's own permanent. Options
considered:
* *(i) add a legality filter* — needs the victim spell's `TargetRequirement`, which `StackObject`
  does not carry. That is a new primitive (a stored requirement list, hashed → HASH bump) and is a
  batch of its own. **Rejected as out of scope**, on conventions.md's implement-phase
  default-to-defer rule.
* *(ii) restrict this batch to player targets* — cannot be enforced; the player chooses the victim.
* **(iii) ship, and pin the deviation wrong-way-round.** Chosen. Positive probes T1/T2 use **player**
  targets (the branch that IS CR-correct, including 115.7a liveness). One additional probe asserts
  the object-target branch's illegal redirect **as the current behaviour**, cites CR 115.7a, names
  **`OOS-DX25b-3`**, and tells the successor batch to invert it — the `blinkmoth_nexus` pattern
  PB-DX19 established.
Say this plainly in the close-out: *the cards go from "cannot be played" to "playable, with a known
CR 115.7a deviation on object targets."* That is a real improvement and an honest one; do not round
it up.

**R3 — a copy of a spell is not an announceable target (CR 707.10).** The `!so.is_copy` guard is
required for disambiguation (one card id would otherwise match the original and all its copies) and
for PB-DX25's cipher-exile hole. Consequence: copies are unreachable as Misdirection/Bolt Bend
targets. Strictly smaller than the HEAD deviation (nothing was reachable); pinned by T5; filed as
**`OOS-DX25b-2`**. Note it compounds with `OOS-DX25-2` (copies raise no `PermanentTargeted`) — the
copy population is invisible to targeting in several independent ways.

**R4 — `deflecting_swat` gains nothing.** `must_change: false` → CR 115.7d "choose new targets" →
`effects/mod.rs:7533-7537` deterministically leaves everything unchanged. It is `Complete` by derive
and its printed line is a no-op. Pre-existing (M9.4 "interactive choice deferred"), **not** caused
or fixed here. R2's roster message must say so, and the close-out must not count it. Consider
filing **`OOS-DX25b-4`** so the "choose new targets" family has a row of its own.

**R5 — modal target indexing on `untimely_malfunction` is unverified.** Mode 1's effect is
`DeclaredTarget { index: 1 }` against a **pooled** three-requirement list, and
`casting::validate_targets_positional` plus the modal slicing at `abilities.rs:433-458` /
`OOS-SIM5-5` are a known-rough area. Verify by probe whether announcing only mode 1's target lands
at `ctx.targets[1]` or `ctx.targets[0]`; if it lands at 0, mode 1 is still broken after this batch.
The card is `partial`, so either way it is not a flip — but **do not claim it works without a
probe.**

**R6 — PB-DX25's G2 gate goes RED and must be re-aimed deliberately, not silently.**
`crates/engine/tests/core/pb_dx25_stack_registry_roster.rs:233-240` asserts
`body.matches("card_in_stack_zone").count() >= 2` inside the `Effect::CounterSpell` arm (lookup +
zone-move). Routing the lookup through `stack_index_for_announced_target` drops that count to **1**.
The correct handling is to re-aim the gate at the *new* invariant — `>= 1` occurrence of
`card_in_stack_zone` (the zone-move) **and** `>= 1` occurrence of
`stack_index_for_announced_target` (the lookup) — keeping the forbidden-literal assertions
(`StackObjectKind::Spell {`, `K::Spell {`, …) byte-unchanged, and then **re-proving discrimination
by executing both reverts**. Weakening it to `>= 1 card_in_stack_zone` alone would silently retire
half a shipped gate. Record this in the execution notes as a deliberate gate edit with its own
revert proof.

**R7 — the `is_spell` guard becomes production-unreachable.** §3.3. Not a defect; a consequence of
R1. The risk is that a future reader deletes it as dead code and thereby deletes the only
distinction between the two `TargetRequirement` variants right before R1 is closed. Mitigated by:
the extended comment at `casting.rs:6503-6513`, the retained collapsed-id sub-case, and R1's seed
naming the guard.

**R8 — census completeness.** §2's method walks `resolve_effect_target_list`'s callers plus the two
validator arms. It does **not** cover a hypothetical future site that reads
`StackObject.targets` directly and does its own lookup. R5 (§5.3) is the machine that catches the
*shape*; nothing catches a site that invents a third way. Stated rather than glossed — PB-DX25's
own lesson is that an enumeration is only as wide as the list it walks.

**R9 — test-suite delta.** Expect roughly +18 to +25 tests (7-8 positive probes, 5 roster/gate
tests, plus the repaired existing ones which are modifications, not additions). Measure the
pre-edit baseline on the branch **before any edit** and report the delta against that, not against
CLAUDE.md's 4,452.

---

## §9 Verification checklist

- [ ] Pre-edit baseline measured on this branch: `cargo test --workspace --no-fail-fast` to a
      **file** (never `| tail` — a tail pipe hid a compile failure and faked a green run on
      2026-08-02), residual list recorded
- [ ] `cargo check -p mtg-engine` clean after each stage
- [ ] All four in-class sites (C1-C4) routed through `stack_index_for_announced_target`, in ONE commit
- [ ] The three already-correct sites (§2.3) byte-unchanged — prove with `git diff`
- [ ] `crates/simulator/` diff **EMPTY**; `stack_card_of` untouched
- [ ] Every revert in §7 executed, rebuild confirmed, failure text recorded, revert restored clean
- [ ] The `git stash` A/B recorded (zero positive probes pass at HEAD)
- [ ] PB-DX25's G2 gate re-aimed with its own revert proof (R6)
- [ ] `mod` lines added to `tests/primitives/main.rs` and `tests/core/main.rs` (SR-9a)
- [ ] `cargo test --workspace --no-fail-fast` to a file; residual list empty
- [ ] `cargo test -p mtg-engine --test core hash_schema` / `--test core protocol_schema` — PROTOCOL
      **35** / HASH **73** **executed**, not predicted
- [ ] `cargo test -p play-server` unmoved
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks none of the
      1,803 defs and still exits 0)
- [ ] `cargo build --workspace`
- [ ] Coverage unmoved, proven by regenerating `tools/authoring-report.py` to a byte-identical body
- [ ] Benches spot-checked (`full_turn_4p`, `priority_cycle_4p`) — the helper runs once per
      announced target, not per priority pass; a regression would mean the lookup landed in a hot loop
- [ ] `OOS-DX25-3` CLOSED in `docs/audits/decision-point-audit.md`, with its own claim corrections
      (the function name `validate_target_requirement` does not exist; the defect is **two** live
      sites, not validation-only; the "vacuous negative tests" description is incomplete — a
      *positive* test is vacuous too)
- [ ] `OOS-DX25b-1..4` filed — **grep the registry for each ID first** (dispatch hygiene 5: status
      bullets lag the registry and IDs have been double-filed)
- [ ] `memory/primitives/pb-DX25b-execution-notes.md` written: revert matrix results, measurements,
      the A/B, every roster population including the zeros
- [ ] v3 queue row 7b struck; CLAUDE.md delta appended as a NEW short bullet (never grow a line);
      `memory/workstream-state.md` handoff
