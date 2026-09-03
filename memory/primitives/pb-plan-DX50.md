# PB-DX50 — the mutate surface: target legality (CR 702.140a) and CR 702.140c timing

**Task**: `scutemob-221` | **v4 queue rank 8** (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 8)
**Seeds**: `OOS-DX25-1` (registry `docs/audits/decision-point-audit.md:1362`) +
`OOS-DX29-2` (`:1423`).

---

## §0 STAGE 0 — written BEFORE any code changed

Everything in this section was committed before a single production line moved. It is here so
the batch's own predictions can be **refuted by its own gates** rather than reconciled after.

### §0.1 Pre-edit baseline (AC 7305)

`cargo test --workspace --no-fail-fast` to a file, on this branch, before any edit:

* **4,941 passing / 0 failing / 5 ignored**, **58** result-producing targets, residual list empty.
* This **reproduces PB-DX49's close pin exactly** (4,941 / 0 / 5, 58 targets).
* Test-NAME set captured to `baseline-names.txt` for the NAME-itemised delta.

### §0.2 The CR read, verbatim, and the finding that changes the design

`mcp__mtg-rules__get_rule 702.140` with children, read in full. The load-bearing sentence is
**CR 702.140b**:

> As a mutating creature spell begins resolving, **if its target is illegal, it ceases to be a
> mutating creature spell and continues resolving as a creature spell** and will be put onto the
> battlefield under the control of the spell's controller.

**This is an explicit EXCEPTION to CR 608.2b.** A mutating creature spell has exactly one target;
under CR 608.2b alone, an illegal target would mean *all* its targets are illegal and the spell
would not resolve. CR 702.140b overrides that: the spell resolves anyway, as an ordinary creature
spell.

**Consequence for this batch, and it is the whole reason Half 1 is not a two-line change.**
`OOS-DX25-1`'s prescription is "route the mutate target into `spell_targets`". Doing exactly that
and nothing else would hand the target to the generic CR 608.2b fizzle gate and **regress a
behaviour the engine currently gets right**. The seed does not say this; the queue row does not say
this; only reading CR 702.140b says it.

**Why it does not regress as shipped, stated as a structural fact rather than a hope**: the CR
608.2b fizzle gate lives inside the `StackObjectKind::Spell` match arm of
`resolution::resolve_top_of_stack_inner` (`resolution.rs:194-266`), and
`StackObjectKind::MutatingCreatureSpell` is a **disjoint arm** (`:7482`) with no fizzle gate of its
own. So the exception holds by the shape of the `match`, not by any check. That is exactly the kind
of load-bearing accident a later batch deletes by "unifying the two arms", so it is **pinned by a
test**, not left to the reader.

CR 702.140c (the timing half) and CR 702.140e (why it is load-bearing) are quoted at their sites.

### §0.3 Per-half WIRE PREDICTIONS — committed before code, gate-computed after

Current: **PROTOCOL 39 / HASH 78**
(`rules/protocol.rs:427`, `state/hash.rs:886`).

| Half | Prediction | Reason (stated, not asserted) |
|---|---|---|
| **Half 1 — target legality** | **NO bump on either fingerprint** | The mutate target is injected into the `StackObject`'s EXISTING `targets: Vec<SpellTarget>` and `target_requirements` fields, and the requirement is a `TargetRequirement::TargetCreatureWithFilter(TargetFilter{..})` built from fields that already exist (`exclude_subtypes`, `owner: TargetOwner::You` — PB-DX28 shipped the owner axis). **No type, no variant, no field is added anywhere.** Both fingerprints hash/serialize *declarations*, and no declaration moves. |
| **Half 2 — CR 702.140c timing** | **PROTOCOL 39 → 40 and HASH 78 → 79, ONE bump each** | Adds `EffectChoiceQuestion::MutateOnTop` and `EffectChoiceAnswer::MutateOnTop` (new variants on types reachable from `GameEvent::EffectChoiceRequired` → PROTOCOL, and from `GameState.pending_effect_choice`/`effect_choice_answers` → HASH); **and REMOVES the `on_top` field from `AdditionalCost::Mutate`**, which is reachable from `Command::CastSpell` (PROTOCOL) and from `StackObject.additional_costs` (HASH). Either edit alone moves both; together they still move each exactly once. |
| **Whole PB** | **PROTOCOL 39 → 40, HASH 78 → 79** | At most ONE bump each, as the criterion requires. |

**Type-count predictions** (the half a fingerprint prediction from the variant list alone would
miss — PB-DX28's lesson): the PROTOCOL closure is predicted **UNCHANGED in type count**, because
`EffectChoiceQuestion` and `EffectChoiceAnswer` are already members and `bool`/`ObjectId` are
already reachable; the new variants add no NEW type. Same for HASH. **Both counts will be read off
the failing gates' own output and never invented.**

**Stop-condition** (PB-DX44's discipline): if either gate moves in a way the half selector above
does not explain — or does **not** move at all — stop and re-derive rather than re-pinning.

### §0.4 Movement budget, predicted

* **State-hash VALUES** move for any game containing a mutate cast: a `MutatingCreatureSpell`'s
  `StackObject` now carries a non-empty `targets` and `target_requirements`. This is a value change,
  not a schema change. Golden scripts / SR-9b per-step fingerprints / seeded pins that contain a
  mutate cast are budgeted to move. **Predicted population: zero or near-zero**, because the fuzz
  and golden corpora rarely reach a mutate cast — to be MEASURED, not assumed.
* **Offer-count** moves: `legal_actions.rs` currently emits one `CastWithMutate` per
  `(target, on_top)` pair; after Half 2 it emits one per `target`, **halving** the mutate offer
  count. Seeded observations that count offers are budgeted to move.
* **Coverage**: predicted **0 flips, 0 card-def edits** — this batch adds no card-def capability.

---

## §1 Design

### §1.1 Half 1 — one arithmetic, three consumers

New `casting::mutate_target_requirement() -> TargetRequirement`, the exact analogue of PB-DX20's
`enchant_target_to_requirement`:

```
TargetRequirement::TargetCreatureWithFilter(TargetFilter {
    exclude_subtypes: vec![SubType("Human")],   // CR 702.140a "non-Human"
    owner: TargetOwner::You,                    // CR 702.140a "same owner as this spell"
    ..Default::default()                        // TargetCreature* already implies creature+battlefield
})
```

Consumed by, and by nothing else:

1. **Cast-time validation** (`handle_cast_spell`): the four hand-rolled checks at
   `casting.rs:1379-1412` (zone / creature / non-Human / owner) are **deleted** and replaced by the
   shared `validate_object_satisfies_requirement`. That is what adds hexproof / shroud / protection
   / "can't be the target of" — none of which the hand-rolled block had.
2. **Announcement**: the target is APPENDED to `spell_targets` as a `SpellTarget` with
   `zone_at_cast: Some(Battlefield)`, so `rules::events::push_target_announcement` — already called
   on every cast — emits `TargetsAnnounced` **and** `PermanentTargeted`, and PB-DX48's Ward dispatch
   fires with no new code at all.
3. **CR 702.140b re-validation** (`resolution.rs:7495`): the four hand-rolled checks there are
   **deleted** and replaced by `is_target_legal` over the stack object's own targets.

**APPEND, not prepend.** `DeclaredTarget { index }` is a positional index into the spell's own
declared targets. Appending leaves every existing index unchanged. Pinned by a roster row.

**`TargetOwner::You` reproduces HEAD's behaviour and is NOT what CR 702.140a says.** The rule says
*"the same owner as **this spell**"*; `TargetOwner::You` means "the same owner as the **caster**".
The two diverge only when a player casts a spell they do not own (Gonti, Bolas's Citadel off another
library). HEAD already used the caster (`target_obj.owner != player`), so this is behaviour-preserving
— and the divergence is a **pre-existing latent defect**, filed rather than silently fixed, because
fixing it needs a `TargetOwner` variant and therefore a wire bump this batch has already spent.

### §1.2 Half 2 — the choice at resolution time

`EffectChoiceQuestion::MutateOnTop` carries no candidate set: CR 702.140c offers exactly two
answers. It is the `PayOptionalCost` shape (PB-DX45's fresh precedent), not the `ChooseObject` shape.

`ask_or_consume_effect_choice` is private to `effects/mod.rs` and takes an `&EffectContext` — but it
reads only two things off it (`effect_choice_gate_closed` and `source`). It is split into a core
function plus two thin callers so `resolution.rs`'s `MutatingCreatureSpell` arm can ask on the same
channel **without a second implementation**.

**PB-DP9's obligations, discharged in writing** (each is a real obligation, not a checklist tick):

* **Determinism** — the replay re-runs the WHOLE resolution. The mutate arm's pre-ask statements
  must be a deterministic function of the state. To be audited, not assumed.
* **The mana-ability gate** — `test_dp9_mana_ability_gate` asserts no `Complete` def puts an asking
  channel inside a mana ability. It needs a **seventh** needle. *This is the exact gate PB-DX45's
  `/review` caught one channel short, under a comment that said FIVE while the engine asked six.*
  The needle is added AND revert-proven, and the comment is re-derived from the call sites rather
  than appended to.
* **Default answer** — `default_effect_choice_answer` must return the pre-batch behaviour so bots,
  the fuzzer and every existing golden script stay behaviourally identical. HEAD's `unwrap_or(false)`
  at `resolution.rs:7493` means the *pre-batch* default for a copy was `false`; the *cast* default
  was whatever the client sent. The default is `on_top: true` (the ordinary case, and what
  `params.rs` hard-coded before PB-DX29 B3), with the divergence for copies stated.

### §1.3 Half 3 — the copy path, all 15 variants

`AdditionalCost` has **15** variants (counted off the declaration, `card-types/src/state/types.rs:248`):
`Sacrifice, Discard, EscapeExile, CollectEvidenceExile, Assist, Replicate, Squad, EscalateModes,
Splice, Entwine, Fuse, Offspring, Gift, Mutate, ExileFromHand`. `copy.rs:249-251` propagates 3.
Each of the other 12 gets a per-variant written disposition, and anything left unfixed gets a
registry row — **not a silent skip**.

Note the shape of the mutate answer: after Half 2, `on_top` is not in `AdditionalCost::Mutate` at
all, so "a copied mutate spell resolves with `on_top` defaulting to `false`, the opposite of the
cast-time value" **ceases to be expressible**. The copy asks at resolution like the original. That is
the honest closure, and it is a consequence of Half 2 rather than a patch to `copy.rs`.

---

## §2 Deliberate non-goals

* CR 702.140a's "same owner as this SPELL" (see §1.1) — filed, not fixed.
* CR 702.140f (effects referring to the mutating spell refer to the mutated permanent) — untouched.
* The `permanent_targeted_events` one-event-per-SLOT reading (PB-DX48 deliberately preserved it) —
  untouched.
