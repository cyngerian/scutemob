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

---

## §3 The enforcement-site census — the brief named TWO of THREE

Both seeds and the queue row describe two sites: cast-time validation (`casting.rs:1379-1412`)
and the CR 702.140b resolution check (`resolution.rs:7495`). Derived at HEAD by grepping for the
predicate's own shape (a non-Human + creature + owner conjunct) rather than for the word "mutate",
there are **three behavioural sites**, and the third is the one that would have broken:

| # | Site | What it decides | Named by the brief? |
|---|---|---|---|
| 1 | `casting.rs:1379-1412` (`handle_cast_spell`) | is this announced mutate target legal | yes |
| 2 | `resolution.rs:7495` (`MutatingCreatureSpell` arm) | is it STILL legal at resolution (CR 702.140b) | yes |
| 3 | **`legal_actions.rs:1662-1675`** (`StubProvider`, `non_human_own`) | **which targets are OFFERED** | **NO** |

Site 3 matters twice over:

* **SR-38.** Half 1 tightens site 1 with hexproof / shroud / protection / "can't be the target of".
  If site 3 keeps its own looser predicate, the engine offers a mutate onto a hexproofed creature
  and then refuses the cast — *a clean offer followed by a guaranteed refusal*, which is the exact
  defect shape PB-DX29 gated Fuse to avoid, PB-DX44 re-created while fixing it, and PB-DX45 shipped
  and had to fix. **This batch would be the fourth.** A batch that took the seeds' two-site list at
  its word ships that defect.
* **It reads `o.characteristics` RAW**, not `expect_characteristics` — so it is blind to the layer
  system. A creature animated into a non-Human, or turned into a Human by a type-changing effect,
  is classified from its printed types. Sites 1 and 2 both read layer-resolved characteristics.
  That divergence is independent of this batch and is filed.

**Not a site**: `crates/simulator/src/targeting.rs` (mutate's target rides on the action, per
PB-DX29 — checked, not assumed) and `tools/play-server/src/view.rs` (labelling only).

---

## §4 A correction to THIS PLAN, made before it shipped

§1.1 item 3 said site 2 (the CR 702.140b re-check) should "delete the four hand-rolled checks and
replace them with `is_target_legal`". **That is a regression, and the plan was wrong.**

`resolution::is_target_legal` (`resolution.rs:8418`) checks, for an object target, exactly one
thing:

```rust
Target::Object(id) => state.objects.get(id)
    .map(|obj| Some(obj.zone) == spell_target.zone_at_cast).unwrap_or(false)
```

Zone, and nothing else. HEAD's mutate block checks battlefield **and** creature-ness **and**
non-Human **and** owner. So "delegate to the shared helper" would have *removed* three checks in the
name of removing duplication — the failure mode is that **the shared thing was weaker than the
duplicated thing**, and "one arithmetic" is only an improvement when the arithmetic that survives is
the RIGHT one.

Site 2 as shipped is the conjunction: `is_target_legal` (CR 608.2b's own zone sentence) **AND**
`validate_object_satisfies_requirement` re-applied to the requirement recorded on the stack object
at announcement. That is strictly stronger than HEAD — it adds hexproof / shroud / protection gained
*in response*, which CR 608.2b requires and HEAD missed — and never weaker.

**A related engine-wide deviation, found by this batch and deliberately NOT fixed**: because
`is_target_legal` is zone-only, *every* spell in this engine under-checks CR 608.2b at resolution —
a target that stays put but stops satisfying the requirement, or gains hexproof/protection in
response, is still treated as legal. That is not a mutate defect and it is far outside this batch.
Filed, not fixed.

---

## §5 The movement budget, ENUMERATED before Half 2 runs

PB-DX15a's lesson was a budget that was written and never came due; PB-DX48's was one that came due
and was rounded to "no change". This one is enumerated up front so it can be checked either way.

| Surface | Measured population at HEAD | Why it moves |
|---|---|---|
| `HASH_SCHEMA_VERSION` sentinels | **45** (43 spelled `78u8`, 2 spelled `78`) | Half 2's bump. Re-pinned **by symbol**, and BOTH spellings enumerated first — PB-DX45 re-pinned 44 and then found 2 more, because a re-pin is only as wide as the spelling its regex matched. No reverse-order (`78u8, HASH_SCHEMA_VERSION`) spelling exists; checked, not assumed. |
| `PROTOCOL_VERSION` sentinels | **11** (all spelled `39`) | same |
| History rows | 2 (`HASH_SCHEMA_HISTORY`, `PROTOCOL_HISTORY`) | **APPEND** only; no shipped row edited. Both `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned. |
| Golden scripts | **1** — `test-data/generated-scripts/combat/192_mutate_gemrazer.json` | It casts mutate with `mutate_on_top: true` in its command JSON. Half 2 deletes that field, and SR-9c makes script JSON **strict**, so the script must be edited. Its SR-9b per-step fingerprint moves for BOTH halves (Half 1 records targets on the stack object; Half 2 inserts an `AnswerEffectChoice`). |
| `crates/simulator/tests/pb_dx29_mutate_on_top.rs` | 3 tests | M1 (`provider_offers_both_on_top_and_under`) and M2 (`params_forwards_the_actions_on_top_choice`) **invert** — the provider stops offering the pair. M3 (`mutating_under_keeps_the_hosts_characteristics`) is the proof the criterion says must survive; it is **re-homed** onto the resolution-time answer, not deleted. All three disclosed by name as leavers/renames. |
| Mutate offer count | one action per `(target, on_top)` → one per `target` | **Halves.** Any seeded observation that counts offers moves. |
| Coverage | predicted **0 flips, 0 card-def edits** | this batch adds no card-def capability |

**Behavioural identity is preserved for every existing bot game, fuzz seed and golden script** by
`default_effect_choice_answer(MutateOnTop) == { on_top: true }` — the exact recovery of the
pre-batch hard-coded value, the same argument ENG-1 and PB-DX45 made. `replay_harness.rs:402-409`
auto-answers with that default, so only the COMMAND TRACE grows.

---

## §6 PB-DP9's mana-ability obligation: the discharge is NOT an eighth needle

`ask_or_consume_effect_choice`'s CR 605.4a branch fires on `ctx.effect_choice_gate_closed`, which is
set at **exactly one site in the tree** — `rules/mana.rs:880`, inside the `WhenTappedForMana`
triggered-mana-ability branch that calls `execute_effect` directly. `test_dp9_mana_ability_gate`
discharges the skipped obligation by walking card defs for asking-`Effect` variants nested inside a
`WhenTappedForMana` trigger.

**The mutate ask is not an `Effect` variant and cannot be reached from that site.** A mutating
creature spell resolves through `resolve_top_of_stack_inner`'s own `match` arm; a mana ability
resolves outside the stack and never enters it. So the honest discharge is a **statement of
structural unreachability plus a gate on that statement** — not an eighth card-def needle, which
would scan for a variant name that does not exist and therefore measure nothing. A gate that cannot
fail is a comment; this queue has filed that shape three times already.

It also means the gate's own instruction — *"re-derive the list from the
`ask_or_consume_effect_choice` call sites"* — stops being sufficient the moment Half 2 lands, because
one asking site is no longer reachable from a card def at all. That is said in the gate rather than
left for the next reader to trip over.

### §6.1 A stale claim found while checking this, in a file nobody re-checked

`rules/mana.rs:878-879` says, in-source:

> *the skipped obligation is discharged by … `test_dp9_mana_ability_gate` … (**four** asking effects
> now: SearchLibrary, Scry, Surveil, DiscardCards)*

The gate checks **seven**. This is the **same sentence** PB-DX45's `/review` caught one channel
short in `effects/mod.rs` — and that fix corrected the `effects/mod.rs` copy only. **Nobody checked
whether the sentence existed anywhere else.** It does, and this copy is *three* channels stale
(missing `ChosenObject`, `MayPayThenEffect` and `LookAtTopThenPlace`), i.e. it was already wrong when
PB-DX28 shipped and has been wrong through two batches that each corrected its twin.

The durable half is not "someone forgot": it is that **a claim was corrected where it was noticed
rather than where it lived**, so the correction did not generalise. Fixed here, and both copies now
point at the gate's own list as the single source rather than restating it.

---

## §7 Half 3 — the 15-variant audit, its verdicts, and where I overrule it

Full audit: `memory/primitives/pb-DX50-additional-cost-copy-audit.md`. Every sharp claim in it was
**re-verified by the coordinator rather than accepted** (the standing rule that a delegated report is
a claim like any other). All four checked claims held:

* CR 707.10's text, read verbatim from the rules server.
* The `MutatingCreatureSpell` arm contains **zero** `is_copy` checks (`grep -c` over `:7481-7660`),
  while the `Spell` arm guards exactly this at `:819` under a comment saying *"The source_object
  belongs to the original spell and must not be moved by a copy's resolution."*
* `Effect::CopySpellOnStack` has **zero** genuine declarations — both grep hits are comments
  (`plumb_the_forbidden.rs:42`, `complete_the_circuit.rs:6`). **SR-36 for the fifth consecutive
  batch in this queue.**
* The `Gift` (`:619-628`) and `Sacrifice` (`:634-644`) read sites are where the audit says.

### §7.1 The comment's RULE is refuted by the rule it cites

`copy.rs:241-242` invents a choice-vs-cost dichotomy:

> *CR 707.2: Copies copy choices (entwine, escalate, fuse) but not one-shot additional costs
> (sacrifice, discard, squad, offspring, gift, mutate).*

CR 707.10, verbatim: *"A copy of a spell or ability copies both the characteristics of the spell or
ability and all decisions made for it, including modes, targets, the value of X, **and additional or
alternative costs**."* The dichotomy does not exist. What the CR actually says is three separate
things — the copy is treated as having paid; it does not actually pay; and *"if an effect of the copy
refers to objects used to pay its costs, it uses the objects used to pay the costs of the
original."* That last clause makes dropping `Sacrifice` affirmatively wrong.

The comment's **list** also names 6 of the 12 dropped variants, silently omitting `EscapeExile`,
`CollectEvidenceExile`, `Assist`, `Replicate`, `Splice`, `ExileFromHand`.

### §7.2 CR 707.10 settles Half 3 better than the criterion's own wording

The criterion asks for *"a copied mutate spell keeps its Mutate entry with a defined `on_top`
answer."* CR 707.10's third sentence is: **"Choices that are normally made on resolution are not
copied."** Once Half 2 makes `on_top` a resolution-time choice (CR 702.140c), the copy **must not**
inherit it — it makes its own. So the honest shape is: propagate `Mutate { target }` (a target IS
copied, sentence 2), and the `on_top` answer is defined by the copy asking at its own resolution
(sentence 3), which after Half 2 is automatic because the field no longer exists.

### §7.3 Disposition — 15 of 15, and the two I take

| Verdict | Variants |
|---|---|
| **FIX** | `Mutate` (allowlist; CR 707.10 sentence 2) — plus the `is_copy` hole in the mutate resolution arm |
| **FILE** | `Sacrifice` (MEDIUM), `Gift` (MEDIUM), `Squad` (LOW-MED), `Offspring` (LOW), and CR 707.10f/608.3f unimplemented (MEDIUM) |
| **CORRECT-AS-IS** | `Discard`, `EscapeExile`, `CollectEvidenceExile`, `Assist`, `Replicate`, `EscalateModes`, `Splice`, `Entwine`, `Fuse`, `ExileFromHand` |

Every FILE gets a registry row with the defect sentence above it. **No silent skips.**

### §7.4 Where I overrule the audit

It argues the `is_copy` hole must not be patched in isolation because that "would encode *a copy of a
mutate spell does nothing*, which is a third wrong answer", and that CR 707.10f must be decided
first. **I take the fix anyway, and the reason is the reason it is not a third answer.**
`resolution.rs:819` **already** encodes "a copy of a permanent spell does nothing", for every other
permanent spell in the game. Making the mutate arm agree with it is not a new wrong answer, it is the
*same* known deviation applied consistently — while leaving it alone means a resolving copy calls
`move_object_to_zone` on the **original's card**, or merges the original's card into the target.

That is PB-DX24's trade, verbatim: **a no-op is auditable; silently consuming another object's card
is not.** The consistency fix removes a state-corruption path; CR 707.10f is filed, not smuggled in.
