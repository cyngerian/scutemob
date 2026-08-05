# Primitive Batch Plan: PB-DX25 — `Effect::CounterSpell`'s three stack-object shapes

**Generated**: 2026-08-05
**Task**: `scutemob-203` · **Branch**: `feat/pb-dx25-effectcounterspells-three-stack-object-shapes-counte`
**Queue**: `memory/primitives/seed-rerank-2026-08-02.md` §4 rank **7**
**Seed**: `OOS-SIM3-5` (`docs/audits/decision-point-audit.md:1281`), read through the binding
correction in `seed-rerank-2026-08-02.md` §1c (`:242`)
**Baseline at HEAD**: `PROTOCOL_VERSION` **35** (`rules/protocol.rs:360`), `HASH_SCHEMA_VERSION`
**73** (`state/hash.rs:757`). Test count to be re-measured on this branch **before any edit**.
**Predicted wire consequence**: **NONE** — PROTOCOL **35** / HASH **73** predicted unmoved. This is
a *prediction to be gate-executed*, never a fact. §7.
**Dependencies**: none unshipped. Touches ground laid by SIM-3 (`stack_card_of`,
`check_stack_consistency`) and PB-DX22 (the fuzzer that can now reach a cast at all).
**Deferred items from prior PBs claimed here**: none. `OOS-SIM3-5` is explicitly filed
"correctness (engine), out of scope" for a simulator-only batch; this batch is that scope.

---

## 0. What was verified at HEAD, and what was assumed

Everything in §1–§4 below was read at HEAD in this worktree unless marked **ASSUMED** or
**DERIVED**. The runner must re-verify each cited line and **flag drift** rather than route
around it.

### 0.1 Verified by reading source

| claim | site | verdict |
|---|---|---|
| The `Effect::CounterSpell` arm is `effects/mod.rs:2721-2808` | `effects/mod.rs` | ✅ as briefed |
| `position()` matches `so.id == id` **or** `Spell { source_object } == id`, and no other kind | `:2732-2738` | ✅ |
| `remove(pos)` happens **before** the per-kind match | `:2751` vs `:2753` | ✅ |
| The match has `Spell`, `ActivatedAbility \| TriggeredAbility`, and `_ => {}` at `:2800-2803` | `:2753-2804` | ✅ |
| **No `is_copy` check anywhere in the arm** | `:2721-2808` | ✅ |
| `MutatingCreatureSpell { source_object, target }` is discriminant 59; `source_object` is the card in `ZoneId::Stack` | `card-types/src/state/stack.rs:852-872` | ✅ |
| `copy.rs` clones `kind` wholesale and sets `is_copy: true` | `rules/copy.rs:159-165` | ✅ |
| `resolution.rs::counter_stack_object` handles `Spell \| MutatingCreatureSpell` together, is exhaustive (no wildcard), and has **no `is_copy` guard** | `resolution.rs:8305-8430` | ✅ |
| `counter_stack_object` has **zero production callers** | in-source note `:8424-8427`; confirmed by repo-wide grep — the only callers are `crates/engine/tests/core/resolution.rs:630` and `:711` | ✅ (its doc-comment claim at `:8304`, *"Used by: the fizzle rule (M3-D), counterspell effects (M3-D/E)"*, is **stale**) |
| `invariants::stack_card_of` is the exhaustive per-kind model, 27 variants, no wildcard | `crates/simulator/src/invariants.rs:134-176` | ✅ |
| `next_object_id` **increments** `timestamp_counter`, so a stack-entry id is globally unique and is never a `GameObject` key | `state/mod.rs:1011-1014` | ✅ — load-bearing for §4.3 |
| `fizzle_move_object_to_zone` returns `None` on `ObjectNotFound` (CR 400.7 fizzle) and mints a **new** `ObjectId` on success | `state/diagnostics.rs:281-303` | ✅ |

### 0.2 Verified, and **not** in the brief — these change the batch's framing

**F1. The mutate target is not a spell target in this engine.** `AdditionalCost::Mutate { target,
on_top }` is consumed at `casting.rs:133-136` and never enters `spell_targets`
(`casting.rs:4528-4545` reads it separately). `battlefield_targets` — the list that drives
`GameEvent::PermanentTargeted`, and therefore Ward — is built *from `spell_targets` only*
(`casting.rs:4426-4434`, emitted at `:4768-4776`). **Consequence: Ward never fires on a mutate
cast's mutate target.** CR 702.140a says the mutate spell *"targets a non-Human creature"*, so
this is itself a defect — see §9 `OOS-DX25-1`. It is **out of scope** and is the reason shape (a)
is not independently live (§2.2).

**F2. `copy_spell_on_stack` emits no `PermanentTargeted`** (grep over `rules/copy.rs`: zero hits).
So Ward never fires on a copy either. CR 702.21a + CR 707.10 (a copy is put on the stack *with*
the original's targets) say it should. §9 `OOS-DX25-2`. Out of scope, and the third independent
reason shape (b) is unreachable.

**F3. There are FOUR sites in the workspace that answer "does this stack object own a card in
`ZoneId::Stack`?", three of them right and one wrong, and none shares a mechanism.**

| # | site | answer for `MutatingCreatureSpell` | correct? |
|---|---|---|---|
| 1 | `effects/mod.rs:2737` (`position()` + the per-kind match) | no | **WRONG — this batch** |
| 2 | `resolution.rs:8318-8319` (`counter_stack_object`) | yes | right |
| 3 | `casting.rs:6504-6509` (`is_spell` for `TargetSpellWithSingleTarget`) | yes | right — **but it answers a DIFFERENT question**, see §3.4 |
| 4 | `crates/simulator/src/invariants.rs:141` (`stack_card_of`) | yes | right |

Three agreeing implementations is not a mechanism; it is three chances to be right and one to be
wrong, and the one that was wrong is the one nobody had a test for. This table is the argument
for §3.1.

**F4. Two CR citations in the code under edit are wrong**, of the `OOS-UI3-1` renumbering-rot
class. `effects/mod.rs:2725` says *"CR 701.5: Counter target spell on the stack"* and `:2744` cites
*"CR 701.5g"*. Verified via MCP: **CR 701.5 is "Cast"** and has exactly two subrules (701.5a,
701.5b); there is no 701.5g. The keyword action is **CR 701.6, "Counter"**. `resolution.rs:8298`
/`:8304` and `rules/events.rs:159` carry the same wrong number; `view-model/src/event_view.rs:785`
already has it right. Comment-only correction, §3.6.

### 0.3 Corpus reconnaissance (grep-derived — **must be replaced by an `all_cards()` enumeration**, SR-36)

Recorded so the runner knows what to expect, **not** as the roster. SR-36: enumerate the corpus,
never grep source.

* **Mutate defs: 8.** `gemrazer`, `sea_dasher_octopus`, `brokkos_apex_of_forever`, `vulpikeet`,
  `necropanther`, `glowstone_recluse` carry **no explicit `completeness` field** or an explicit
  `Complete`, i.e. **6 `Complete`**; `mindleecher` is `partial`, `nethroi_apex_of_death` is
  `partial`. Matches the queue row's "6 `Complete` mutate defs" exactly.
* **Defs carrying `Effect::CounterSpell`: 24** — matches the queue row. Explicitly non-`Complete`:
  `memory_lapse` (partial), `mana_drain` (partial), `flare_of_denial` (partial),
  `transcendent_dragon` (partial), `arcane_denial` (known_wrong), `pyroblast` (known_wrong)
  → **18 `Complete`** by derive.
* **The queue's "6 × 24" is an overcount.** A counter can only reach a mutate spell if its target
  requirement admits a **creature** spell. `TargetRequirement::TargetSpell` (unrestricted) appears
  on `counterspell`, `saw_it_coming`, `abjure`, `force_of_will`, `rewind`, `access_denied`,
  `cryptic_command` (mode 0), `archmages_charm` (mode 0) — **8 `Complete`**.
  `red_elemental_blast`'s `TargetSpellWithFilter(blue)` adds a 9th for the blue mutate defs.
  Every "noncreature"/"instant"/"mana value 1" filter (`negate`, `dovins_veto`, `dispel`,
  `swan_song`, `stubborn_denial`, `force_of_negation`, `fierce_guardianship`,
  `an_offer_you_cant_refuse`, `mental_misstep`) excludes a mutate spell.
  **Estimated live-wrong pairs: ~6 × 8 = 48, not 144.** The runner replaces this with a measured
  number in the execution notes and **corrects the queue row and the seed row in place**.
* **Colour identity is not a constraint on the pairing.** These are two different players' decks
  in a 4-player game; the mutate deck and the counter deck need not share an identity. Do not
  filter the roster by colour.
* **The exact shape-(c) probe pair: `gemrazer` × `counterspell`.** Gemrazer is explicitly
  `Completeness::Complete` (`gemrazer.rs:74`), Mutate `{1}{G}{G}`, and declares **no** spell-level
  target requirement, so the fixture has one moving part. Counterspell is `Complete` by derive
  (`counterspell.rs:23`, `..Default::default()`), `TargetRequirement::TargetSpell`,
  `exile_instead: false`, `cant_be_countered: false`. `crates/engine/tests/mechanics_m_z/mutate.rs:
  192-266` already contains the exact cast-for-mutate command shape — **read it before writing the
  fixture, do not reinvent it**.

---

## 1. CR basis (MCP, verbatim)

### CR 701.6 — Counter

> **701.6a** To counter a spell or ability means to cancel it, removing it from the stack. It
> doesn't resolve and none of its effects occur. **A countered spell is put into its owner's
> graveyard.**
>
> **701.6b** The player who cast a countered spell or activated a countered ability doesn't get a
> "refund" of any costs that were paid.

*Derivation:* "put into its owner's graveyard" is a statement about a **card**. A stack object with
no card has nothing to put anywhere; a stack object with a card must have that card moved, and the
current code decides which is which by variant name.

### CR 702.140 — Mutate

> **702.140a** … "Mutate [cost]" means "You may pay [cost] rather than pay this spell's mana cost.
> If you do, **it becomes a mutating creature spell and targets a non-Human creature with the same
> owner as this spell**." Casting a spell using its mutate ability follows the rules for paying
> alternative costs …
>
> **702.140b** As a mutating creature spell begins resolving, if its target is illegal, it ceases
> to be a mutating creature spell and continues resolving as a creature spell …
>
> **702.140c** As a mutating creature spell resolves, if its target is legal, it doesn't enter the
> battlefield. Rather, it merges with the target creature … (see rule 729) …

*Derivations, both load-bearing:*
1. A mutating creature spell **is a creature spell**, and CR 701.6a puts no restriction on what
   kind of spell may be countered. So an ordinary "counter target spell" **must** be able to
   counter it, and the countered card **must** go to its owner's graveyard. That is shape (c).
2. "**targets** a non-Human creature" — the mutate target is a target, which is what makes F1 a
   defect rather than a modelling choice.

### CR 707.10 — Copies

> … A copy of a spell or ability is controlled by the player under whose control it was put on the
> stack. **A copy of a spell is itself a spell, even though it has no spell card associated with
> it.** …
>
> **707.10a** If a copy of a spell is in a zone other than the stack, it ceases to exist. … These
> are state-based actions.
>
> **707.10b** A copy of an ability has the same source as the original ability. …

*Derivations:* (i) a copy is a **spell**, so it may be countered and countering it is a real game
action worth an event; (ii) it has **no card**, so no card may be moved and no card id may be
named; (iii) 707.10b is why a copy of an *ability* still legitimately names its source object —
the copy special case must be scoped to card-owning kinds only (§4.3).

### CR 729.2 — Merging

> **729.2** To merge an object with a permanent, place that object on top of or under that
> permanent. …
> **729.2b** As an object merges with a permanent, that object leaves its previous zone …

*Derivation:* the mutate card's only two exits from `ZoneId::Stack` are merge-on-resolution
(729.2b) and counter (701.6a). The engine implements the first (`resolution.rs:7372`) and, for
this kind, not the second — which is why the card is stranded.

### CR 702.21 — Ward

> **702.21a** Ward is a triggered ability. Ward [cost] means "Whenever this permanent becomes the
> target of a spell or ability an opponent controls, counter that spell or ability unless that
> player pays [cost]."

*Derivation:* Ward's counter names the **stack entry** (`abilities.rs:4605-4633` tags the pending
trigger with `targeting_stack_id`; `abilities.rs:8400-8405` sets it as the trigger's target), which
is why the `so.id == id` clause exists and must be kept.

### CR 400.7 — Object identity

> **400.7** An object that moves from one zone to another becomes a new object with no memory of,
> or relation to, its previous existence.

*Derivation:* `fizzle_move_object_to_zone` returns a **new** `ObjectId`. Every assertion in §6 about
"the card is in the graveyard" must locate it by NAME or through the emitted event's
`source_object_id` — never by the pre-counter id.

---

## 2. The three shapes, re-derived at HEAD

### 2.1 (c) — LIVE. An ordinary counter on a mutate spell is a silent no-op.

`TargetRequirement::TargetSpell` validation (`casting.rs:6430-6453`) requires the announced target
to be an object in `state.objects` with `zone == ZoneId::Stack` — i.e. **the card**. A mutate
cast puts its card there exactly as a plain cast does (`casting.rs:4423`, one
`move_object_to_zone(card, ZoneId::Stack)` before the `cast_with_mutate` branch). So the target is
announced, validated, and **legal**; the spell is on the stack; the mana is paid; and at resolution
`position()` returns `None`, because its second clause matches `Spell` alone. **Nothing happens and
nothing is reported.**

No divergence is produced today: the stack entry and its card both survive, so
`check_stack_consistency` stays quiet. This is the worst of the three from a player's seat and the
quietest from an instrument's — a countered spell resolves anyway, silently.

### 2.2 (a) — the stranding. **Not independently live today; it is what fixing (c) alone would create.**

The seed files (a) as a Ward-path defect. At HEAD the Ward path cannot reach a
`MutatingCreatureSpell` at all: Ward needs a `GameEvent::PermanentTargeted` naming the mutate
spell's stack entry, and that event is emitted only for `spell_targets` (F1), which for a mutate
cast is **empty** for every corpus mutate def (none declares a spell-level target requirement —
`gemrazer.rs`, `vulpikeet.rs` read; the remaining four to be confirmed by the §5 enumeration).

So (a) is reachable today only through a synthetic fixture — **and it becomes reachable through the
ordinary counter path the instant (c)'s lookup is fixed.** Fixing the `position()` lookup without
fixing the zone-move converts a silent no-op into a permanent `ZoneId::Stack` leak that
`stack_consistency` reports at every subsequent checkpoint, for the rest of the game.

> **This is the single most important sequencing fact in the batch. (c) and (a) must land in one
> commit. A "just fix the lookup" change is strictly worse than HEAD.**

### 2.3 (b) — UNREACHABLE at HEAD, by three independent mechanisms. Fix it anyway.

§1c of the v3 memo gives one reason. Two more were found here; all three must be re-verified by the
runner, and the seed row corrected to carry all three:

1. **Order.** `copy.rs` pushes the copy **above** the original (later index; `copy.rs:132-133`).
   `position()` returns the **first** match, i.e. the lowest index. With the original present, a
   card-id lookup always lands on the original. (§1c's reason.)
2. **The dead-id filter.** `resolve_effect_target_list_indexed`
   (`effects/mod.rs:7620-7642`) returns a `DeclaredTarget` only if the id still exists in
   `state.objects` **or** names a live stack entry. If the original left the stack, its Stack-zone
   card got a new id under CR 400.7 and the announced id is dead — the target resolves to the empty
   vector and the effect never runs. So the window in which the copy could be found by the
   original's card id is empty, not merely narrow. (Stronger than §1c, which argued from
   `TargetSpell` announce-time validation; that argument is also true and covers announcement,
   while this one covers CR 608.2b re-validation at resolution — e.g. a multi-target
   `cryptic_command` that does not fizzle.)
3. **Nothing aims a counter at a copy.** `TargetSpell` cannot name a copy (a copy's stack-entry id
   is not a `GameObject` key, so `casting.rs:6426` returns `ObjectNotFound`), and Ward never fires
   on a copy (F2).

**Fix it anyway**, in the same edit: it is one `if`, it is CR-mandated (707.10), and the batch is
adding `MutatingCreatureSpell` to the card-owning set — which widens the population of kinds whose
copies would be mis-moved from one to two. Its probe is **synthetic and must say so in its doc
comment and in its failure message.**

---

## 3. The design

### 3.1 The classification — **one engine-side registry, `crates/engine/src/state/stack_registry.rs`**

**New file**, declared `pub mod stack_registry;` in `crates/engine/src/state/mod.rs` beside
`pub mod keyword_registry;` (`state/mod.rs:10`).

```rust
/// The card this stack object owns in `ZoneId::Stack`, if it owns one.
///
/// Exhaustive over every `StackObjectKind` with **no wildcard arm**, deliberately:
/// adding a variant is a compile error here until someone decides which side it is
/// on. Same forcing function SR-5 applies to `KeywordAbility`
/// (`state::keyword_registry::handling`).
///
/// **This is not "is it a spell".** A copy of a spell IS a spell (CR 707.10) and owns
/// no card; `casting.rs`'s `is_spell` check for `TargetSpellWithSingleTarget` asks the
/// other question and must NOT be re-expressed through this function (§3.4).
///
/// **Deliberately duplicated** by `mtg_simulator::invariants::stack_card_of` (§3.2).
pub fn card_in_stack_zone(kind: &StackObjectKind) -> Option<ObjectId>
```

Returns `Some(*source_object)` for `Spell` and `MutatingCreatureSpell` (CR 601.2c / 702.140a /
729.2 — one `move_object_to_zone(card, Stack)` at `casting.rs:4423`, then a `cast_with_mutate`
branch that only picks the kind), and `None` for the other 25 variants, listed one per line with
the CR note that says where their source stays.

**Why the engine and not an inherent method on `StackObjectKind` in `crates/card-types`.** Three
reasons, in order of weight:

1. **Project precedent, and it is exactly this shape.** `KeywordAbility` lives in `card-types`;
   its forced classification lives in `crates/engine/src/state/keyword_registry.rs`. SR-5's whole
   text describes the same problem. Following it costs nothing and puts the second registry where
   a reader already looks for the first.
2. **It is an engine fact, not a type fact.** Which kinds own a Stack-zone card is decided by what
   `casting.rs` does, not by the enum's shape. The classification's justification is a set of
   engine call sites; it should live where those can be cited by relative path.
3. **SR-6 build hygiene.** `crates/card-defs` depends on `card-types`; touching `card-types`
   rebuilds all 1,798 defs, touching the engine does not. Not a violation — a real cost with no
   compensating benefit here, because (per §3.2) the simulator is **not** going to consume it.

The compile-forcing property is identical either way: an exhaustive match with no wildcard fails to
compile on a new variant wherever it lives.

### 3.2 Should `invariants::stack_card_of` delegate to it? — **No. Keep them separate.**

**Argument for delegating.** One function, one truth; the duplication is ~30 lines of near-identical
match arms that must be maintained twice; a drift between them is currently caught by nothing at
compile time.

**Argument against, which wins.** `check_stack_consistency` exists to catch the engine getting the
classification wrong — that is literally its history note (`invariants.rs:112-133` records that the
S8 rewrite guessed from variant names and produced a false positive on every mutate cast). If the
verifier reads the engine's own answer, then an engine misclassification makes the check agree with
the defect and go **silent**. A wrong `Some`/`None` would be invisible in exactly the case the check
was written for. That is not a hypothetical: it is the failure mode this batch is fixing, one layer
up.

**Decision: keep two implementations.** What keeps them in sync:

* **Coverage** is machine-enforced on both sides independently: both are exhaustive with **no
  wildcard**, so a 28th variant is a compile error in *both* crates. Neither can silently miss a
  variant. This is the only sync property that matters for the failure mode "someone added a kind
  and forgot".
* **Classification** is deliberately **not** synced. If they disagree, `check_stack_consistency`
  fires — which is the designed behaviour, not a gap. A disagreement is loud by construction.
* **A behavioural cross-check, not a structural one**: the simulator probe in §6 runs a real
  counter-on-mutate game and asserts zero `stack_consistency` violations. That proves the two agree
  *on the case that matters* without coupling them.
* **Doc cross-references at both functions**, each naming the other by path and stating that the
  duplication is deliberate and why. `invariants.rs`'s `t8` doc comment
  (`invariants.rs:833-844`) gains one sentence: the engine's `card_in_stack_zone` is a second,
  independent classification and this test's discrimination is over *this* one.

### 3.3 The `position()` lookup rewrite (shape (c))

```rust
let pos = state.stack_objects.iter().position(|so| {
    // (i) CR 702.21a — the Ward path: the trigger's target is the stack ENTRY's own id.
    so.id == id
        // (ii) CR 601.2c / 608.2b — the traditional counter: the announced target is the
        //      CARD in `ZoneId::Stack` (that is what `TargetSpell` validates,
        //      `casting.rs:6430-6440`). Every card-owning kind, not just `Spell`.
        //      CR 707.10: a COPY has no card of its own — `copy.rs:162` clones the
        //      original's `kind`, so a copy's `source_object` names the ORIGINAL's card
        //      and must never make the copy findable by it.
        || (!so.is_copy
            && crate::state::stack_registry::card_in_stack_zone(&so.kind) == Some(id))
});
```

**Search order and precedence, stated:** `position()` scans bottom-of-stack upward and returns the
first match. Clause (i) can match at most one entry (stack-entry ids are unique). Clause (ii) can in
principle match several entries only if two non-copy stack objects claim the same card, which CR
400.7 makes impossible (`invariants.rs:278-280`, property (3)). So the two clauses cannot race, and
the scan direction is not load-bearing — but it is **kept as `position()` rather than changed to
`rposition()`**, because changing it would be an unmotivated behaviour change on the one path
(clause (ii) against a copy) this batch is closing by other means.

**Why today's behaviour for plain `Spell` is preserved exactly.**
`card_in_stack_zone(Spell { source_object }) == Some(source_object)`, so for a non-copy `Spell`
clause (ii) is character-for-character today's second clause. The only behavioural deltas are:
`MutatingCreatureSpell` becomes findable (the fix), and a **copy** stops being findable by the
original's card id (unreachable today, §2.3 — so a zero-behaviour delta in practice, and the
differential probe T3's non-vacuity half proves the fixture could have reached it).

### 3.4 What must **not** be unified: `casting.rs:6502-6509`

`validate_target_requirement`'s `is_spell` check for `TargetSpellWithSingleTarget` /
`TargetSpellOrAbilityWithSingleTarget` answers *"is this stack object a spell?"*. A copy of a spell
**is** a spell (CR 707.10) and owns no card. Re-expressing that check as
`card_in_stack_zone(..).is_some()` would make copies illegal targets for "target spell", which is
CR-wrong. **Leave it alone.** A comment at `stack_registry`'s doc says so; a comment at
`casting.rs:6503` names the registry and says why it is not used there.

(While reading that site the runner will notice a separate live defect — §9 `OOS-DX25-3`. **Do not
fix it here.**)

### 3.5 The zone-move rewrite (shapes (a) and (b))

Replace the whole `match stack_obj.kind { … }` at `:2753-2804` with:

```rust
let stack_obj = state.stack_objects.remove(pos);
let controller = stack_obj.controller;

// CR 701.6a: "A countered spell is put into its owner's graveyard." Which stack
// objects own a card is decided ONCE, in `state::stack_registry` — never per-kind here.
let card_owned = crate::state::stack_registry::card_in_stack_zone(&stack_obj.kind);
// CR 707.10 / 707.10a: a copy is a spell with no card. It has nothing to move and
// simply ceases to exist on leaving the stack. `copy.rs:162` clones the ORIGINAL's
// `kind`, so moving `source_object` here would put someone else's spell in the
// graveyard.
let card_to_move = if stack_obj.is_copy { None } else { card_owned };

if let Some(source_object) = card_to_move {
    // ── unchanged from the pre-PB-DX25 `Spell` arm, verbatim ──
    let owner = state.objects.get(&source_object).map(|o| o.owner).unwrap_or(controller);
    let destination = if *exile_instead
        || stack_obj.cast_with_flashback     // CR 702.34a
        || stack_obj.cast_with_jump_start    // CR 702.133a
    { ZoneId::Exile } else { ZoneId::Graveyard(owner) };
    if let Some((new_id, _)) = state.fizzle_move_object_to_zone(source_object, destination) {
        events.push(GameEvent::SpellCountered {
            player: controller,
            stack_object_id: stack_obj.id,
            source_object_id: new_id,
        });
    }
} else {
    let named = if card_owned.is_some() {
        // A COPY of a card-owning kind. CR 707.10: no card, so no card id may be
        // named — least of all the original's. Its own stack-entry id is used;
        // `next_object_id` (`state/mod.rs:1011-1014`) is monotone and never a
        // `GameObject` key, so `event_view.rs:786-796`'s `card_name` lookup returns
        // None and the line renders "<player>'s spell is countered" — which is
        // exactly what happened. See §4.3.
        Some(stack_obj.id)
    } else {
        match &stack_obj.kind {
            // CR 701.6a: countering an ability removes it from the stack; the source
            // stays where it is. CR 707.10b: a copy of an ability has the SAME source,
            // so this arm is correct for ability copies too.
            StackObjectKind::ActivatedAbility { source_object, .. }
            | StackObjectKind::TriggeredAbility { source_object, .. } => Some(*source_object),
            // Every other ability/trigger kind: NO event, exactly as before PB-DX25.
            // This wildcard is a DIAGNOSTICS omission, not a state one — the card
            // decision above has no wildcard and cannot lose a card. Widening the
            // event to every kind is `OOS-DX25-4`, deliberately not taken here.
            _ => None,
        }
    };
    if let Some(source_object_id) = named {
        events.push(GameEvent::SpellCountered {
            player: controller,
            stack_object_id: stack_obj.id,
            source_object_id,
        });
    }
}
```

**Every behaviour the brief names is preserved and is individually probed (§6 T4/T5):**
`exile_instead`, `cast_with_flashback` (CR 702.34a), `cast_with_jump_start` (CR 702.133a), the
`unwrap_or(controller)` owner fallback, `source_object_id: new_id` (the **post-move** id, CR 400.7),
`ctx.countered_spell_controller` set **before** the `cant_be_countered` check (EF-W-MISS-1 / the An
Offer ruling — untouched, it lives above the removal at `:2745`).

**Note on the widened `countered_spell_controller`.** Because clause (ii) now finds a mutate spell,
`ctx.countered_spell_controller` gets set for a mutate spell where previously it did not. That is
**correct and intended** — a Swan Song-shaped "its controller creates …" rider must fire off a
countered mutating creature spell exactly as off any other spell — but it is a behaviour delta
beyond the removal itself and must be recorded in the execution notes.

### 3.6 `resolution.rs::counter_stack_object` — fixed in the same edit, and here is why

It has **zero production callers** (§0.1). It is nevertheless changed, for the reason PB-DP9 gave
about this exact function's tail (`resolution.rs:8421-8427` — *"routed through the shared helper so
a future caller does not inherit a shipped deadlock"*): it is `pub`, it is API, and leaving one of
two counter paths carrying the known-wrong shape is precisely how a future caller inherits a
shipped defect.

Three changes:

1. Collapse the `Spell | MutatingCreatureSpell` arm and the 20-variant OR-list into
   `match card_in_stack_zone(&stack_obj.kind) { Some(card) => …, None => … }`. **This deliberately
   gives up the per-variant compile-forcing at this site** — that is the point: the forcing moves
   to the registry, so the engine has ONE classification instead of two that happen to agree
   (F3). **Move the whole `None`-arm comment block verbatim** (the per-keyword "if countered by
   Stifle …" notes at `:8374-8412`); it is the most valuable prose in the function and must not be
   lost to the refactor.
2. Add the `is_copy` guard (CR 707.10), same shape as §3.5.
3. **Correct the stale doc at `:8298-8304`**: it claims *"Used by: the fizzle rule (M3-D),
   counterspell effects (M3-D/E)"*. Neither is true; both counter effects go through
   `effects/mod.rs`, and the fizzle rule does not call it. State what it actually is: a `pub` API
   with no production caller, two test callers, kept as the second counter path.

### 3.7 The CR-citation corrections (comment-only, F4)

`effects/mod.rs:2725` and `:2744`, `resolution.rs:8298`/`:8304`, `rules/events.rs:159`:
**CR 701.5 → CR 701.6**, and delete the non-existent "CR 701.5g" (the An Offer note's real warrant
is the effect's own printed wording plus CR 701.6a, not a subrule). Cite `OOS-UI3-1` as the class.
0 behaviour, 0 wire.

---

## 4. The three questions the brief asks answered in one place

**4.1 Where the classification lives.** `crates/engine/src/state/stack_registry.rs`,
`pub fn card_in_stack_zone`. Engine, not `card-types`, on the `keyword_registry` precedent (§3.1).

**4.2 Whether the simulator delegates.** **No** — the verifier must not read the thing it verifies
(§3.2). Sync is: compile-forced coverage on both sides, deliberately unsynced classification, a
behavioural cross-check probe, and a doc cross-reference at each function.

**4.3 The `SpellCountered` payload for a countered copy.** **Emit it**, with
`stack_object_id == source_object_id == stack_obj.id`. Derivation:

* CR 707.10 — *"A copy of a spell is itself a spell"* — so a counter of a copy is a real,
  observable game action and suppressing the event hides it from the log and from any future
  "whenever a spell is countered" reader. Emitting nothing was the pre-PB-DX25 behaviour only by
  accident (`_ => {}`), not by decision.
* CR 707.10 — *"even though it has no spell card associated with it"* — so no card id may be
  named, and the **original's** card id is the one value that is definitely wrong.
* The copy's own stack-entry id is the honest identity: it names the object that was countered, it
  is monotone-unique (`state/mod.rs:1011-1014`), and it is never a `GameObject` key, so
  `event_view.rs:786-796`'s `card_name()` returns `None` and the already-shipped fallback renders
  *"<player>'s spell is countered"* — the correct sentence, with no renderer change and no wire
  change.
* Free bonus, stated so it is not later mistaken for coincidence: `stack_object_id ==
  source_object_id` becomes a machine-detectable "this was a copy" marker on the existing wire.
* **Scoped to card-owning kinds only** (`card_owned.is_some()`), because CR 707.10b gives a copy of
  an *ability* the same source as the original, so an ability copy must keep naming
  `source_object`.

---

## 5. Corpus roster (SR-36 — enumerate `all_cards()`, never grep)

**Stage-1 work, before any probe is written.** Write the enumeration as the roster gate itself
(§6 G3), not as a throwaway, and record every count including zeros in
`memory/primitives/pb-DX25-execution-notes.md`.

| # | population | how to derive from `all_cards()` | grep estimate (§0.3) — **to be replaced** |
|---|---|---|---|
| M1 | defs carrying `AbilityDefinition::Keyword(KeywordAbility::Mutate)` | scan `abilities` (and `back_face`) | 8 |
| M2 | M1 ∩ `completeness.is_complete()` | | **6** |
| M3 | M2 that declare **any** spell-level `TargetRequirement` | walk `AbilityDefinition::Spell { targets }` | **expected 0** — this is what makes shape (a) corpus-unreachable via Ward (§2.2); a non-zero count is a finding |
| C1 | defs whose `abilities` contain `Effect::CounterSpell` anywhere (incl. inside `Modal`) | recursive `Effect` walk | 24 |
| C2 | C1 ∩ `is_complete()` | | 18 |
| C3 | C2 whose counter target requirement is unrestricted `TargetRequirement::TargetSpell` | syntactic — no filter evaluation | **8** |
| P | **live-wrong pairs** = \|M2\| × \|C3\| | | **~48** (the queue row's "6 × 24 = 144" is an overcount and must be corrected in place, in both `seed-rerank-2026-08-02.md` §4 row 7 and the `OOS-SIM3-5` row) |

C3 is deliberately the *syntactic* subset. Counters with a `TargetSpellWithFilter` that admits a
creature spell (`red_elemental_blast`, blue) add to P but require evaluating `matches_filter`
against a synthetic creature-spell `Characteristics`; record the extra count as a note rather than
pinning it, and say so.

**Non-vacuity floor on every pin**: `all_cards().len() >= 1_700` asserted in the same test, so a
broken enumeration cannot make an empty roster look correct (the PB-DX24 R2 lesson).

**Card-def edits planned: NONE.** Coverage predicted unmoved at **1,133/1,803 = 62.8%** and proven
by an **empty** `git diff -- crates/card-defs/`, not by regeneration.

---

## 6. Probes, with the exact revert that must make each fail

**SR-9a**: two new files, each needing a `mod` line. `crates/engine/tests/primitives/main.rs` — add
`mod pb_dx25_counterspell_stack_shapes;` after `mod pb_dx24_trigger_zone_and_index_spaces;` (`:37`).
`crates/engine/tests/core/main.rs` — add `mod pb_dx25_stack_registry_roster;` after
`mod pb_dx24_trigger_zone_roster;` (`:32`). Never a top-level `tests/*.rs`.

**Architecture Invariant 8**: every test cites its CR in the doc comment *and* in the failure
message.

**Every revert below must be watched failing by EXECUTING it**, with the rebuild confirmed in the
captured output (a stale binary that "passes" is the recurring R7 class), then restored with
`git diff` confirmed clean before the next.

### File A — `crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs`

| id | test | asserts | CR | **revert that must redden it** |
|---|---|---|---|---|
| **T1** | `test_dx25_counterspell_counters_a_mutate_spell` — **shape (c), REAL corpus cards, `gemrazer` × `counterspell`** | p1 casts Gemrazer for its mutate cost onto a Wolf (`mutate.rs:192-266`'s command shape); p2 casts Counterspell targeting the **card in `ZoneId::Stack`**; drain the stack. Then: the `MutatingCreatureSpell` entry is gone; **`ZoneId::Stack` is empty**; a Gemrazer object exists in `Graveyard(p1)` under a **new** id (found by name, CR 400.7); exactly one `SpellCountered` whose `source_object_id` equals that new id; the Wolf has **no** `merged_components` (CR 729.2 did not happen) | 701.6a / 702.140a / 400.7 | Restore the `Spell { source_object } == id`-only clause in `position()` → the counter finds nothing, Gemrazer resolves and merges, `Stack` non-empty |
| **T2** | `test_dx25_ward_path_counter_on_a_mutate_spell_moves_the_card` — **shape (a), SYNTHETIC, and its doc must say why** | A `MutatingCreatureSpell` on the stack is countered through the **`so.id == id`** clause (the Ward shape). Assert entry removed **AND** card in the graveyard **AND `ZoneId::Stack` empty**. Doc comment records the §2.2 measurement: no `Complete` mutate def declares a spell-level target (roster M3 = 0), so no corpus Ward can reach this today. Build it via a real Ward creature if a synthetic mutate def with a spell-level target can be made to emit `PermanentTargeted`; **read `crates/engine/tests/mechanics_m_z/ward.rs:136-260` first**, and fall back to a hand-built trigger stack object carrying `Effect::CounterSpell` with `targets: vec![SpellTarget { target: Target::Object(stack_entry_id), .. }]` only if the first route does not work. Record which route was used | 702.21a / 701.6a | Restore the `_ => {}` catch-all in place of the `card_to_move` branch → the entry is removed and the card is **stranded in `ZoneId::Stack`**; T2's "Stack is empty" assertion reddens |
| **T3** | `test_dx25_countering_a_copy_moves_no_card` — **shape (b), SYNTHETIC, and its doc must say why** | Cast a spell; `copy_spell_on_stack` it; counter the **copy** via its stack-entry id. Assert: the copy's entry is gone; the **original's** card is still in `ZoneId::Stack` under its original id; the original's stack entry is untouched; exactly one `SpellCountered` with `stack_object_id == source_object_id == copy_id` (§4.3). **Non-vacuity, same test**: countering the **original** in a sibling fixture DOES move the card — so the fixture is proven capable of moving one. Doc records §2.3's three unreachability mechanisms | 707.10 / 707.10a / 707.10b | Delete the `if stack_obj.is_copy { None }` guard → the **original's** card moves to the graveyard and the original's stack entry dangles |
| **T4** | `test_dx25_countered_spell_destination_is_preserved` | Three sub-cases on a plain `Spell`: `exile_instead: true` → Exile; `cast_with_flashback: true` → Exile; neither → `Graveyard(owner)`, with `owner ≠ controller` so the owner lookup is discriminating. Fourth sub-case: pin that a `MutatingCreatureSpell` structurally cannot carry `cast_with_flashback` (mutually exclusive alternative costs, `casting.rs:2527`) — asserted, not exercised | 702.34a / 702.133a / 701.6a | Hard-code `ZoneId::Graveyard(owner)` for the destination → the two Exile sub-cases redden |
| **T5** | `test_dx25_uncounterable_mutate_spell_still_sets_the_controller` | A `cant_be_countered` mutate spell: the entry stays on the stack, the card stays in `ZoneId::Stack`, and `ctx.countered_spell_controller` is still set (EF-W-MISS-1 / An Offer, CR 701.6a + the effect's own wording). Newly reachable — before this batch the mutate spell was never found, so this line never ran for one | 101.2 / 101.6 / 701.6a | Move the `countered_spell_controller` assignment below the `cant_be_countered` check → red |
| **T6** | `test_dx25_stack_registry_classifies_every_kind` | One constructed `StackObjectKind` per variant; `card_in_stack_zone` returns `Some` for exactly `Spell` and `MutatingCreatureSpell`. **Non-vacuity**: the roster length is asserted equal to the measured variant count (27 at HEAD — re-measure), so a 28th variant that is classified but not probed reddens | 601.2c / 702.140a / 729.2 | Classify `NinjutsuAbility` as `Some(*source_object)` → red naming the variant |
| **T7** | `test_dx25_both_engine_counter_paths_agree` | Run the same two fixtures (a `MutatingCreatureSpell`, and a copy) through `resolution::counter_stack_object` and assert the same end state as T1/T3. This is the only pin on a `pub` function with no production caller | 701.6a / 707.10 | Delete the `is_copy` guard from `counter_stack_object` → red |

### File B — `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs`

| id | gate | asserts | **revert** |
|---|---|---|---|
| **G1** | source gate — the registry has no wildcard | Read `crates/engine/src/state/stack_registry.rs`; **strip line AND block comments** (PB-DX32 M8: a `/* … */` wrap defeated a line-comment-only scanner while every probe stayed green — prove the stripping is load-bearing by executing **both** revert variants); take `card_in_stack_zone`'s body by brace matching; assert it contains no `_ =>` and no `_ |`. Message: *"a new `StackObjectKind` must be classified here, not defaulted — `Effect::CounterSpell` and `counter_stack_object` both drive their zone-move off this answer"* | add `_ => None,` (and separately, a `/* */`-wrapped variant) |
| **G2** | source gate — the counter arm does not re-classify | Read `effects/mod.rs`; take the `Effect::CounterSpell` arm by brace matching; assert (a) it calls `card_in_stack_zone` at least twice (lookup + move), (b) `fizzle_move_object_to_zone` appears **exactly once** in it, and (c) the literals `StackObjectKind::Spell` and `StackObjectKind::MutatingCreatureSpell` appear **zero** times in it. Message: *"the zone-move is driven off `state::stack_registry`, never off a per-kind match — do not add an arm, extend the registry"* | restore the per-kind `StackObjectKind::Spell { source_object } =>` arm |
| **G3** | roster gate (SR-36) | §5's M1/M2/M3/C1/C2/C3 pinned by **card NAME** where the set is small enough and by count otherwise, each with the `all_cards().len() >= 1_700` non-vacuity floor. Message names `OOS-SIM3-5` and tells a future author that a new mutate def or a new unrestricted counter def widens the class | change a pinned constant by 1 |

### File C — `crates/simulator/tests/pb_dx25_counter_on_mutate_is_consistent.rs`

**The direct closure proof for `OOS-SIM3-5`'s headline claim.** A real two-player game
(`GameStateBuilder` + `process_command`, or `LocalGame` with scripted submissions) in which p2
counters p1's Gemrazer mutate cast. Run `invariants::check_all` after every command; assert
**zero** `stack_consistency` violations across the whole game **and** zero in the terminal state.

**Non-vacuity, in the same file and mandatory**: a hand-built state with a card in `ZoneId::Stack`
claimed by no stack object **does** produce a `stack_consistency` violation — so a green result
distinguishes "clean" from "the check is broken". (Without this half the probe is satisfiable by a
`check_all` that returns nothing.)

**Revert**: restore the `_ => {}` catch-all in `effects/mod.rs` → the counter strands the card and
this probe reddens at every checkpoint after the counter, with the message
*"Object … in the Stack zone is not claimed by any stack object"*.

### Acceptance criterion 6232

The brief names a gate for acceptance criterion **6232**. Its text is not in this worktree — the
runner must read it (`esm task show scutemob-203`) and map it explicitly before Stage 6, recording
the mapping. **Best inference from the batch subject**: it is the "one engine-side per-kind
classification, exhaustive, no wildcard, so a new card-carrying variant is a compile error until
classified" criterion, in which case it is satisfied by **G1 + G2 + T6** together — G1 for the
no-wildcard property, G2 for single-consumption, T6 for the classification's content. **Confirm;
do not assume.**

### Test count expectation

Roughly **+11 to +15** `#[test]` functions (T1–T7 with sub-cases possibly split, G1–G3, the
simulator probe's two halves). The number is **measured against the pre-edit baseline on this
branch**, never predicted.

---

## 7. Wire prediction, stated as falsifiable

> No enum gains a variant. No struct gains a field. `GameEvent::SpellCountered`'s shape is
> unchanged (only *which values* flow into it change, for one previously-silent case).
> `StackObjectKind`, `StackObject`, `Effect`, `Command` are all untouched. The new
> `stack_registry` module is a **function**, and a function is not in either closure.
> **Therefore `core protocol_schema` and `core hash_schema` must both stay green at
> PROTOCOL 35 / HASH 73.**
>
> **Falsifier**: if either gate reddens, this batch changed a type in a closure without the plan
> noticing. **Stop, read the failure text, report it.** Never hand-edit a constant; both numbers
> are read out of the gate's own output.

**Any design that would move either number should be rejected outright here** — nothing in §3
requires it, and a fingerprint bump to fix one match arm's internals would be evidence the design
drifted.

---

## 8. SR gates and invariants to check explicitly

* **SR-8 (wire)** — §7. Gate-execute `--test core protocol_schema` and `--test core hash_schema`.
* **SR-5 (keyword registry)** — **run `cargo test -p mtg-engine --test core keyword_registry` and
  report the result.** `KeywordAbility::Ward`'s declared `sites` include
  `crates/engine/src/effects/mod.rs`, which this batch edits; the gate asserts **set equality**
  between declared sites and scanned source, in both directions. Predicted **unmoved** (the file is
  already listed and no `KeywordAbility::` literal is added or removed), but PB-DX20 and PB-DX23
  were *each* caught by this gate finding a handling site their brief had missed. If it reddens,
  the design touched a keyword's behaviour and the plan is wrong — stop and report.
* **SR-4 (silent failures)** — the `_ => None` in §3.5's `else` branch is a **diagnostics**
  omission on a branch that provably cannot lose a card; its comment must say exactly that and
  name `OOS-DX25-4`. The `fizzle_move_object_to_zone` call keeps the existing `lki_*` side of the
  classification (it already is the fizzle helper). No new `expect_*` is required.
* **SR-3 (sealed state)** — no new mutation path; `GameState` fields are read, not exposed.
  `cargo build --workspace` is the gate.
* **SR-6** — `crates/card-defs` and `crates/card-types` must both be **untouched**;
  `git diff main..HEAD --numstat -- crates/card-defs/ crates/card-types/` **EMPTY**. This is the
  measurable form of §3.1's placement decision.
* **SR-9a** — two `mod` lines added, no top-level `tests/*.rs`.
* **SR-35** — no card-def edit is planned, so `tools/check-defs-fmt.sh` should be a no-op; run it
  anyway.
* **SR-36** — §5's roster is enumerated from `all_cards()`. The §0.3 grep numbers are
  reconnaissance and must be **replaced**, not confirmed.
* **SR-9b/9c (golden scripts)** — a `Complete` card's behaviour changes, so a golden script can
  move. No mutate+counter script was found by the planner; the full-workspace run is the arbiter.
  If a script moves, **report the diff before reconciling**, and reconcile by *strengthening*
  (the PB-DX3b precedent), never by weakening.
* **Architecture Invariant 1** — no IO added to the engine. G1/G2 read files, but they are tests
  and follow `core/decision_gate.rs`'s existing `read_ct` idiom.
* **Exhaustive-match sweep** — **expected empty, and that is a claim to verify.** No variant is
  added, so none of these needs a new arm; confirm each is unchanged: `state/hash.rs:4174`
  (`StackObjectKind`), `crates/view-model/src/lib.rs:646` (`stack_kind_info`),
  `tools/tui/src/play/panels/stack_view.rs`, `rules/protocol.rs`. And
  `git diff main..HEAD --numstat -- tools/ crates/view-model/` must be **EMPTY**.

---

## 9. Hazards (from `memory/gotchas-rules.md` / `gotchas-infra.md`, filtered to what applies)

1. **CR 400.7 / object identity — the #1 hazard here.** `fizzle_move_object_to_zone` mints a **new
   `ObjectId`**. Every "the card is in the graveyard" assertion must find it by NAME or through the
   event's `source_object_id`; the pre-counter id is dead and looking it up returns `None` with no
   error. Half of shape (b)'s unreachability rests on exactly this (§2.3 reason 2).
2. **Never rewind `state.timestamp_counter` in a fixture** (`gotchas-infra.md` Testing). It is the
   same counter `next_object_id` uses; rewinding it makes the next object collide with an existing
   one, silently. §4.3's "a stack-entry id is never a `GameObject` key" property depends on the
   counter being monotone — a fixture that rewinds it would break the payload design as well as
   the state.
3. **`pass_all_four` resolves exactly ONE stack item per call** (`gotchas-infra.md`). T1 has a
   mutate spell **and** a Counterspell on the stack; use a drain loop, not a fixed number of
   passes.
4. **Mutate fixture specifics** (`gotchas-infra.md` "Mutate Gotchas"): the target and the
   over/under choice are announced at cast time via `AdditionalCost::Mutate { target, on_top }`,
   not at resolution; mutate **preserves the target's `ObjectId`** on a successful merge, so T1's
   negative ("the merge did not happen") must be asserted on `merged_components`, not on an id
   change.
5. **`ObjectSpec::card()` creates naked objects** — call `enrich_spec_from_def()`; and
   `ObjectSpec::card + with_types([Creature])` leaves `toughness: None`, which SBAs skip.
6. **Sorcery-speed casts need an empty stack (CR 307.1)** — the Wolf must be on the battlefield via
   the builder, not cast, or T1's sequencing gets long.
7. **Do not run the replay-viewer HTTP server for validation** (`gotchas-infra.md`) — SIGKILL 137.
8. **`cargo test --workspace --no-fail-fast` output goes to a FILE, never `| tail`** — the
   2026-08-02 lesson (a tail pipe hid a compile failure and faked a green run).
9. **Two probes measuring one thing.** T1 and the File C simulator probe both die if the zone-move
   is reverted. That is deliberate: T1 is behavioural (did the right card move?), File C is the
   instrument closure (does `stack_consistency` stay quiet?). Keep both; note the overlap so a
   future reader does not delete one as redundant.

---

## 10. Stage list

**Stage 0 — re-verify, do not re-derive.** Re-read every line cited in §0.1 at HEAD and flag drift.
Additionally establish and record in `memory/primitives/pb-DX25-execution-notes.md`:
* the exact `StackObjectKind` variant count (planner counted **27**);
* whether shape (b) is reachable, by *executing* a probe that tries all three routes of §2.3
  against unmodified HEAD (this is evidence, not argument);
* the workspace test baseline **before any edit**, `--workspace --no-fail-fast` to a file;
* PROTOCOL / HASH read from source **and** confirmed by executing their gates.

**Stage 1 — the corpus enumeration (§5).** Write G3 first; it is the roster and the measurement in
one. Correct the queue row and the seed row's "6 × 24" in place with the measured number.

**Stage 2 — fail-before.** Write **T1** and the **File C** simulator probe against unmodified HEAD
and **watch both fail**. Capture the failure text verbatim — it is the historical record that the
bug was real, and File C's failure is the measured form of "a countered spell resolves anyway".
(T1 fails at HEAD because nothing happens; File C's *positive* half passes at HEAD because there is
no divergence yet — assert T1's shape there instead, i.e. that the mutate resolved despite being
countered. Record this asymmetry honestly.)

**Stage 3 — the registry (§3.1) + T6 + G1.** New module, `pub mod` line, exhaustive classification.
Verify `cargo build --workspace` and that `crates/card-defs` stays `Fresh`.

**Stage 4 — the counter arm (§3.3 + §3.5), atomically.** Both halves in one commit — §2.2. Then
T1–T5 and G2. Full-workspace run: **no pre-existing test may redden**; if one does, it was
asserting the defect — report it before changing it.

**Stage 5 — the second counter path (§3.6) + T7**, plus the §3.7 CR-citation corrections and the
§3.2 doc cross-references (both functions, and `invariants.rs`'s `t8` doc).

**Stage 6 — the gates.** G1/G2/G3 reverts executed, including the **block-comment** variant for G1.
Map and satisfy acceptance criterion 6232 (§6).

**Stage 7 — close-out.**
* `cargo build --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`;
  `cargo fmt --check` **and** `tools/check-defs-fmt.sh`.
* `cargo test --workspace --no-fail-fast` **to a file**; record the count against the Stage-0
  baseline; residual list must be empty.
* `--test core protocol_schema` / `--test core hash_schema` / `--test core keyword_registry` — all
  gate-**executed**, numbers read out of the run.
* `cargo test -p mtg-simulator`, `cargo test -p play-server` — both unmoved.
* Scope: `git diff main..HEAD --numstat -- crates/card-defs/ crates/card-types/ crates/view-model/
  tools/` must be **EMPTY**. Coverage unmoved **1,133/1,803 = 62.8%**, proven by the empty
  card-defs diff (no regeneration needed — no def is edited).
* Update `docs/audits/decision-point-audit.md`'s `OOS-SIM3-5` row: **CLOSED**, with its own
  corrections recorded rather than deleted — (c) was the live shape and (a) the rider, not the
  reverse; (a) becomes reachable only once (c) is fixed; (b) is unreachable three ways; the
  "6 × 24" is an overcount.
* File the §11 seeds.

---

## 11. Findings to FILE, not fix

Each was found while planning and is out of scope. The runner verifies, files, and does **not** fix.

| id | finding | evidence |
|---|---|---|
| **OOS-DX25-1** | **The mutate target is not modelled as a target.** CR 702.140a: a mutating creature spell *"targets a non-Human creature"*. The engine carries it in `AdditionalCost::Mutate` (`casting.rs:133-136`) and never puts it in `spell_targets`, so it is invisible to `GameEvent::PermanentTargeted` (⇒ **Ward never fires on a mutate**, CR 702.21a), to hexproof/shroud/protection target legality, and to CR 608.2b resolution-time re-validation. CR 702.140b's own "if its target is illegal" branch exists at `resolution.rs:7372`, reading the `kind`'s `target` field directly rather than the target list — self-consistent, and disjoint from every other targeting rule in the engine | `casting.rs:133-136`, `:4426-4434`, `:4528-4545`, `:4768-4776` |
| **OOS-DX25-2** | **Ward never triggers on a copy.** `copy_spell_on_stack` emits no `PermanentTargeted` (zero hits in `rules/copy.rs`). CR 707.10 puts the copy on the stack *with* the original's targets, so the permanent becomes the target of a new spell and CR 702.21a should fire | grep over `rules/copy.rs` |
| **OOS-DX25-3** | **`TargetSpellWithSingleTarget` and `TargetSpellOrAbilityWithSingleTarget` can never be satisfied — the same id-space confusion this batch is about, one function over.** `casting.rs:6426` requires `id` to be a `state.objects` key (i.e. the **card**), then `:6476`/`:6502` look the stack object up by `so.id == id` (a **stack-entry** id). The two namespaces both count from small integers, so the comparison type-checks and never matches: `is_spell` is always false and `target_count` is always 0, so both requirements always return `InvalidTarget`. Misdirection-shaped cards (PB-EF11's stated unblock) cannot work. The in-src tests at `:8188-8321` are **negative** tests ("self-targeting must be rejected"), which pass vacuously. **Verify with one positive probe at Stage 0; do not fix.** | `casting.rs:6420-6523`, `:8188-8386` |
| **OOS-DX25-4** | **`Effect::CounterSpell` emits `SpellCountered` for only 2 of the 25 ability kinds.** A Stifle-shaped counter of a `KeywordTrigger`, `RoomAbility`, `LoyaltyAbility`, `DelayedActionTrigger`, … removes the entry and reports nothing. Preserved verbatim by PB-DX25 (§3.5) because widening it changes an observable event stream on paths unrelated to the three shapes. Fix shape: a sibling `source_of(&kind) -> Option<ObjectId>` in `state::stack_registry` | `effects/mod.rs:2784-2803` |
| **OOS-DX25-5** | **`counter_stack_object` is a `pub` API with no production caller and a doc comment that claims two callers it does not have.** Corrected in-batch (§3.6); filed so the *question* — keep it, or delete it and its two tests — is asked once rather than re-discovered | `resolution.rs:8298-8304`, `:8424-8427` |
| **OOS-DX25-6** | **CR 701.5 vs 701.6 rot in the engine's counter comments** (F4). Fixed in-batch as comments; filed so the wider `OOS-UI3-1` sweep knows this family was touched | `effects/mod.rs:2725`/`:2744`, `resolution.rs:8298`/`:8304`, `rules/events.rs:159` |

---

## 12. Risks

1. **The fail-before asymmetry (Stage 2).** At HEAD the File C simulator probe's *headline*
   assertion (zero `stack_consistency` violations) is **already green**, because shape (c) produces
   no divergence — the spell simply resolves. Do not read that as "the bug is not there"; assert
   the *behavioural* fact at HEAD instead (the countered spell merged anyway) and record both.
   A probe that is green before and after proves nothing.
2. **T2's fixture route is unresolved.** The Ward route needs a mutate spell with a spell-level
   battlefield target, which the corpus does not have (roster M3). If a synthetic def cannot be
   made to emit `PermanentTargeted`, fall back to the hand-built trigger stack object — and say
   which route was used, because the two differ in how much of the real Ward path they exercise.
3. **`counter_stack_object`'s refactor loses per-variant compile-forcing at that site.** Deliberate
   (§3.6) and the forcing moves to the registry — but if the runner finds a reason the OR-list
   carries information the registry does not, **stop and report** rather than silently keeping two
   classifications.
4. **`countered_spell_controller` now fires for mutate spells** (§3.5). A Swan Song / An Offer
   probe against a mutate spell is not in the §6 list; if the full-workspace run surfaces an
   interaction, treat it as news.
5. **A golden script or an event-log assertion may move** on the newly-emitted copy event or the
   newly-successful mutate counter. Report the diff before reconciling; strengthen, never weaken.
6. **Scope creep toward `OOS-DX25-1`.** Modelling the mutate target as a real target is the
   natural next thought while writing T2 and it is a much larger batch (it touches
   `spell_targets`, Ward, CR 608.2b re-validation, and probably the wire). **Do not start it.**
