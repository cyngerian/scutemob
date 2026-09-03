# `AdditionalCost` propagation to spell copies — per-variant audit

**Scope**: `crates/engine/src/rules/copy.rs:243-255` (the 3-variant allowlist inside
`copy_spell_on_stack`). Read-only audit; no source file was modified.
**Date**: 2026-09-03. **Tree**: worktree `scutemob-221`, branch
`feat/pb-dx50-the-mutate-surface-target-legality-mutate-is-a-targe`.

---

## Summary

`AdditionalCost` has **15** variants, verified off the declaration at
`crates/card-types/src/state/types.rs:248-295` (variant heads at `:259 Sacrifice`,
`:264 Discard`, `:266 EscapeExile`, `:268 CollectEvidenceExile`, `:270 Assist`,
`:272 Replicate`, `:275 Squad`, `:277 EscalateModes`, `:279 Splice`, `:281 Entwine`,
`:283 Fuse`, `:285 Offspring`, `:287 Gift`, `:289 Mutate`, `:294 ExileFromHand`).
Three are propagated to a copy; **twelve are dropped**.

**The comment's stated rule is refuted by the CR it cites.** `copy.rs:241-242` says
*"CR 707.2: Copies copy choices (entwine, escalate, fuse) but not one-shot additional
costs (sacrifice, discard, squad, offspring, gift, mutate)."* CR 707.10 says the
opposite in as many words: *"A copy of a spell or ability copies both the
characteristics of the spell or ability and all decisions made for it, including
modes, targets, the value of X, **and additional or alternative costs**."* CR 707.2's
own example list already includes an optional additional cost — *"whether it was
**kicked**"* — so "additional cost" is not a category the CR excludes. The correct rule
is three-part and the comment states none of it: (i) the copy is **treated as having
had** the same additional and alternative costs paid; (ii) the copy does **not
actually pay** them; (iii) *"If an effect of the copy refers to objects used to pay its
costs, it uses the objects used to pay the costs of the original spell or ability"*
(CR 707.10) — so a cost-payment object record must be **propagated**, not zeroed. The
only genuine CR exclusions are CR 707.10's *"Choices that are normally made on
resolution are not copied"* (which is Mutate's `on_top`, CR 702.140c) and CR 707.2's
*"text-changing effects... are not copied"* (which is Splice, CR 702.47c). So the
allowlist is right about Entwine/Escalate/Fuse and right about Splice, and it is right
about the rest only by accident.

**The comment's list is also incomplete.** It names **six** of the twelve dropped
variants and omits six: `EscapeExile`, `CollectEvidenceExile`, `Assist`, `Replicate`,
`Splice`, `ExileFromHand`. A reader checking the comment against the code cannot tell
whether the six unnamed ones were considered.

**Nothing here is live today, and the reason is a hard bound worth writing down.**
The corpus's *only* spell-copy sources are **six deck-legal `Complete` defs**, all of
them **self-copying instants or sorceries**: `empty_the_warrens.rs:44`,
`radstorm.rs:22`, `flusterstorm.rs:20` (Storm), `follow_the_bodies.rs:21`
(Gravestorm), `train_of_thought.rs:20` (Replicate), `make_disappear.rs:22`
(Casualty(1)) — none carries an explicit `completeness:` field, so all six are
`Complete` by the `#[default]` derive. **`Effect::CopySpellOnStack`
(`card_definition.rs:2475`) has ZERO genuine declarations in 1,803 defs**: the only
two files whose text contains the string are `plumb_the_forbidden.rs:42` (inside a
`Completeness::partial(...)` note) and `complete_the_circuit.rs:6` (a `//` comment).
**No card in this corpus can copy another card's spell.** So the reachable
question per variant is only ever "does one of those six declare it?", and the answer
is yes exactly once (Casualty → `AdditionalCost::Sacrifice`, behaviourally inert).

**A second, independent bound.** CR 707.10f / CR 608.3f — *"Some effects copy a
permanent spell. As that copy resolves, it ceases being a copy of a spell and becomes
a token permanent"* — is **unimplemented**. `grep -rn "608.3f\|707.10f"
crates/engine/src crates/card-types/src --include=*.rs` returns **nothing**, and
`resolution.rs:819-822` makes a resolving copy of a permanent spell a pure no-op (it
emits `SpellResolved` and creates nothing). Every read site in the permanent-ETB
branch is therefore unreachable for a copy: `Squad` (`resolution.rs:872`), `Offspring`
(`:881`), `Gift`-on-permanent (`:886`) and Devour's `Sacrifice { ids }` (`:1473`) are
all inside the `} else if is_permanent {` arm opened at `resolution.rs:823`.

**Not a regression from RC-1.** `git log -S` on `copy.rs` shows `squad_count`,
`offspring_paid`, `gift_was_given`, `gift_opponent`, `mutate_target`, `mutate_on_top`
and `devour_sacrifices` all existed as `StackObject` fields before commit `16a3cec3`
("chore: type consolidation complete — RC-1 through RC-4") folded them into
`additional_costs`. `git show 16a3cec3 -- crates/engine/src/rules/copy.rs` shows the
removed lines were `squad_count: 0,` / `offspring_paid: false,` / `gift_was_given:
false,` / `gift_opponent: None,` / `mutate_target: None,` / `mutate_on_top: false,` /
`devour_sacrifices: vec![],` — i.e. **hardcoded defaults, not propagations**. RC-1
changed the defect's shape (seven explicit zeroed fields became one filter) and not
its behaviour. A future batch must not blame RC-1; the gap predates it.

**Two live-code findings that are not about the allowlist at all** and are the largest
things this audit found: the Mutate arm at `resolution.rs:7481-7620` never consults
`is_copy` (§14 below), and CR 707.10f is missing entirely (§16).

---

## Disposition table (all 15 variants)

| # | Variant (`types.rs`) | CR governing | Current | CR-correct? | Read at resolution? | Deck-legal `Complete` producers | Verdict |
|---|---|---|---|---|---|---|---|
| 1 | `Sacrifice` `:259` | 707.10 ("objects used to pay its costs") | dropped | **No** | **Yes** — `resolution.rs:636-643` (LKI → `ctx.sacrificed_creature_lki`, copy-reachable); `:1473` (Devour, permanent-only) | 13 spell-sac defs + `predator_dragon` (Devour); 0 copyable | **FILE (MEDIUM)** |
| 2 | `Discard` `:264` | 707.10 | dropped | No (in principle) | **No** — only `casting.rs:96` (validation) and `hash.rs:4835` | `flame_jab` (Retrace), `radical_idea` (Jump-Start) — 2 Complete | **CORRECT-AS-IS (unobservable)** |
| 3 | `EscapeExile` `:266` | 702.138 / 707.10 | dropped | No (in principle) | **No** — only `casting.rs:99`, `hash.rs:4842` | **0 Complete** (4 declare; all partial/known_wrong) | **CORRECT-AS-IS (unobservable)** |
| 4 | `CollectEvidenceExile` `:268` | 701.59c (linked ability) | dropped | **Yes** | **No** — only `casting.rs:102`, `hash.rs:4849` | **0 Complete** (`crimestopper_sprite` is partial) | **CORRECT-AS-IS** — the linked-ability bit rides `evidence_collected`, propagated at `copy.rs:239` |
| 5 | `Assist` `:270` | 702.132a (payment-rules modification) | dropped | Yes | **No** — only `casting.rs:105`, `hash.rs:4856` | `huddle_up` (1) | **CORRECT-AS-IS** |
| 6 | `Replicate` `:272` | 702.56a | dropped | Yes (defensively) | **No** — the trigger carries its own `copy_count` in `TriggerData::SpellCopy` (`resolution.rs:2694-2703`) | `train_of_thought` (1) | **CORRECT-AS-IS** |
| 7 | `Squad` `:275` | 702.157a ("if its squad cost was paid") | dropped | **No** | Yes, but **unreachable** — `resolution.rs:872` is inside the `else if is_permanent` arm (`:823`) | `galadhrim_brigade`, `ultramarines_honour_guard` (2) | **FILE (LOW-MEDIUM)**, gated on §16 |
| 8 | `EscalateModes` `:277` | 702.120a | **propagated** | **Yes** | Yes — `resolution.rs:542-548` | 0 Complete (both escalate defs partial) | **CORRECT-AS-IS** |
| 9 | `Splice` `:279` | 702.47c + 707.2 | dropped | **Yes** | **No** — only `casting.rs:118`, `hash.rs:4873`; the operative line is `spliced_effects: vec![]` at `copy.rs:229` | `glacial_ray` (1) | **CORRECT-AS-IS** (comment does not state the reason — NIT) |
| 10 | `Entwine` `:281` | 702.42b | **propagated** | **Yes** | Yes — `resolution.rs:527-529` | `goblin_war_party` (1) | **CORRECT-AS-IS** |
| 11 | `Fuse` `:283` | 702.102d | **propagated** | **Yes** | Yes — `resolution.rs:312-314` | `turn`, `wear_tear` (2) | **CORRECT-AS-IS** |
| 12 | `Offspring` `:285` | 702.175a | dropped | **No** | Yes, but **unreachable** — `resolution.rs:881` is in the permanent arm | **0 Complete** (`flowerfoot_swordmaster` is partial) | **FILE (LOW)**, gated on §16 |
| 13 | `Gift` `:287` | 702.174b | dropped | **No** | **Yes** — `resolution.rs:619-628` + `:645+` (instant/sorcery, **copy-reachable**); `:886` (permanent, unreachable) | `nocturnal_hunger` (1, an Instant) | **FILE (MEDIUM)** — highest-severity of the dropped set |
| 14 | `Mutate` `:289` | 702.140a (alt cost) / 702.140c (resolution choice) / 707.10 | dropped | **Split — see §14** | **Yes** — `resolution.rs:7488-7493`, `unwrap_or(false)` | 6 (`brokkos_apex_of_forever`, `gemrazer`, `glowstone_recluse`, `necropanther`, `sea_dasher_octopus`, `vulpikeet`) | **FILE (MEDIUM-HIGH)** — but the payload defect is the cloned `kind`, not the dropped variant |
| 15 | `ExileFromHand` `:294` | 118.9 / 707.10 | dropped | No (in principle) | **No** — only `casting.rs:137`, `hash.rs:4892` | `force_of_negation`, `force_of_vigor`, `force_of_will`, `misdirection` (4) | **CORRECT-AS-IS (unobservable)** |

Census method for the "producers" column: comment-**and**-string-stripped scan of all
1,803 files in `crates/card-defs/src/defs/` (excluding `mod.rs`), matching
fully-qualified variant paths only; completeness from
`completeness\s*:\s*Completeness\s*::\s*(\w+)` on the stripped source with absence ⇒
`Complete` (the `#[default]`). The classifier returns **1,137 Complete / 1,803**,
byte-matching `docs/authoring-status.md`. String-body stripping is load-bearing: several
defs embed the printed keyword in an `oracle_text:` literal, which a naive grep counts
as a declaration (SR-36).

---

## §1 — `Sacrifice`: **FILE (MEDIUM)**

> **Defect sentence.** `copy.rs:243-255` drops `AdditionalCost::Sacrifice { ids, lki }`
> from a spell copy, so `resolution.rs:636-643` reads an empty LKI vector and sets
> `ctx.sacrificed_creature_lki = vec![]`; CR 707.10's *"If an effect of the copy refers
> to objects used to pay its costs, it uses the objects used to pay the costs of the
> original spell or ability"* requires the copy to see the original's sacrificed
> creature, so a copy of a Life's Legacy / Momentous Fall / Eldritch Evolution-shaped
> spell draws or fetches off **0** power instead of the sacrificed creature's, silently
> and with no diagnostic.

One-line fix: add `AdditionalCost::Sacrifice { .. }` to the allowlist at
`copy.rs:249-252`. (The `lki` payload is already exactly the LKI snapshot CR 707.10's
last sentence describes — `types.rs:249-258` documents it as captured *before*
`move_object_to_zone`.)

Severity MEDIUM, not HIGH, because it is **latent**: 13 deck-legal `Complete` defs
carry `SpellAdditionalCost::Sacrifice*` (`abjure`, `altar_of_bone`,
`corrupted_conviction`, `crop_rotation`, `culling_the_weak`, `deadly_dispute`,
`diabolic_intent`, `eldritch_evolution`, `goblin_grenade`, `harrow`, `lifes_legacy`,
`momentous_fall`, `village_rites`) and **none of them is copyable** — see the bound in
the summary. The one variant instance that *does* reach a live copy today is
`make_disappear`'s Casualty sacrifice (`make_disappear.rs:22`), whose spell effect is
`CounterUnlessPays` and never reads LKI, so the drop is observationally inert there.
Devour's read site (`resolution.rs:1468-1477`, `predator_dragon`) is unreachable for a
copy for the §16 reason.

**Do not close this by propagating `Sacrifice` and calling Casualty fixed.** `copy.rs:218-221`
deliberately sets `was_casualty_paid: false`, and that disambiguator (`types.rs:244-247`)
is what stops a copied Casualty spell from re-triggering its own copy. Propagating
`Sacrifice` does not disturb it — but a probe must assert `was_casualty_paid == false`
on the copy alongside the LKI assertion, or a later batch will "simplify" the two.

## §7 — `Squad`: **FILE (LOW-MEDIUM)**

> **Defect sentence.** `copy.rs:243-255` drops `AdditionalCost::Squad { count }` from a
> spell copy, so `resolution.rs:869-878` reads `.unwrap_or(0)` and the permanent that a
> copied squad creature spell would produce enters with `squad_count = 0`; CR 702.157a
> makes the ETB trigger conditional on *"if its squad cost was paid"*, and CR 707.2
> names *"whether it was kicked"* — the same optional-additional-cost family — as a
> copied choice, so the copy should enter having had squad paid the same number of
> times. Currently unreachable, because CR 707.10f is unimplemented
> (`resolution.rs:819-822`) and a copy of a permanent spell produces no permanent at all.

One-line fix: add `AdditionalCost::Squad { .. }` to the allowlist. 2 deck-legal
`Complete` producers (`galadhrim_brigade`, `ultramarines_honour_guard`), 0 copyable.
**Blocked-on**: §16. Fixing this alone changes nothing observable and should be filed as
a rider on the 707.10f seed rather than shipped as a standalone with no red-before test.

## §12 — `Offspring`: **FILE (LOW)**

> **Defect sentence.** `copy.rs:243-255` drops `AdditionalCost::Offspring`, so
> `resolution.rs:879-883` sets `obj.offspring_paid = false` on the permanent a copied
> offspring creature spell would produce, contradicting CR 702.175a's *"if its offspring
> cost was paid"*. Unreachable twice over: CR 707.10f is unimplemented, and the corpus
> has **0** deck-legal `Complete` offspring defs (`flowerfoot_swordmaster` is `partial`).

One-line fix: add `AdditionalCost::Offspring` to the allowlist. Lowest priority of the
FILE set; it has neither a reachable path nor a deck-legal producer.

## §13 — `Gift`: **FILE (MEDIUM)**

> **Defect sentence.** `copy.rs:243-255` drops `AdditionalCost::Gift { opponent }`, so
> `resolution.rs:619-626` sets `ctx.gift_was_given = false` and `ctx.gift_opponent =
> None` on a spell copy; CR 702.174a makes gift *"you may choose an opponent"* — a
> decision, not a payment, and therefore copied under both CR 707.2 and CR 707.10 — so a
> copy of a gift instant or sorcery skips its CR 702.174b gift effect entirely and
> `Condition::GiftWasGiven` reads false, where the original read true.

One-line fix: add `AdditionalCost::Gift { .. }` to the allowlist.

**This is the sharpest of the dropped set, for two reasons.** First, it is the only
dropped variant whose read site is on the **copy-reachable** path:
`resolution.rs:619-628` and the CR 702.174j early-effect block at `:645+` run for
copies (the `is_copy` branch at `:819` is downstream of effect execution and its own
comment says *"Copy resolves: execute the effect, then emit SpellResolved"*), unlike
Squad/Offspring/Devour which sit in the permanent-ETB arm. Second, the in-source comment
lists Gift among *"one-shot additional costs"* alongside sacrifice and discard, which is
wrong on the comment's own dichotomy: **choosing an opponent consumes nothing**. It is
the same shape as `was_bargained` (`copy.rs:212-214`) and `was_surged` (`:215-217`),
both of which the same function propagates *with a CR 707.2 citation for exactly this
reason* — three adjacent fields, one rule, two opposite treatments.

Latent: 1 deck-legal `Complete` producer, `nocturnal_hunger` (an Instant), and it is not
copyable by anything in the corpus.

## §14 — `Mutate`: **FILE (MEDIUM-HIGH)** — and the dropped variant is the smaller half

Two separable defects. The registry's framing ("loses its `Mutate` entry entirely and
resolves with `on_top` defaulting to `false` — the opposite of the cast-time value") is
**verified true** at `resolution.rs:7486-7493`:

```
let mutate_on_top = stack_obj.additional_costs.iter()
    .find_map(|c| match c { AdditionalCost::Mutate { on_top, .. } => Some(*on_top), _ => None })
    .unwrap_or(false);
```

**(a) The `on_top` half is the LESS serious one, and the CR partly excuses it.**
CR 702.140c: *"As a mutating creature spell resolves, if its target is legal... **The
spell's controller chooses whether the spell is put on top of the creature or on the
bottom**."* That makes `on_top` a choice *made on resolution*, and CR 707.10 says
*"Choices that are normally made on resolution are not copied."* So dropping the
original's value is defensible. What is **not** defensible is `unwrap_or(false)`:
CR 702.140c gives the copy's controller a choice, and the engine substitutes a silent
constant. This engine hoists the choice to cast time (PB-DX29's
`LegalAction::CastWithMutate { on_top }`), which is a stated deviation for the cast
path; the copy path inherits the deviation *and* loses the choice.

**(b) The serious half is the cloned `kind`, and no registry row names it.**
`copy.rs:163` is `kind: original.kind.clone()`, so a copy of a mutating creature spell
is a `StackObjectKind::MutatingCreatureSpell { source_object, target }` whose
`source_object` is the **original's card ObjectId**. The mutate resolution arm
(`resolution.rs:7481-7620`) **never consults `is_copy`** — `grep -n "is_copy"
crates/engine/src/rules/resolution.rs` returns `:231, :819, :2030, :2066, :2072, :5506,
:5512, :5520, :5522, :8488, :8654` and **nothing in 7481-7620**. Both of its branches
therefore act on the original's card:

- illegal-target branch, `resolution.rs:7515`:
  `state.move_object_to_zone(source_object, ZoneId::Battlefield)?` — the copy puts the
  **original's card** onto the battlefield, and the original then resolves against a
  dead `ObjectId` (CR 400.7).
- legal-target branch, `resolution.rs:7536-7553`: builds a `MergedComponent` from
  `state.objects.get(&source_object)` and merges the **original's card** onto the target.

This is the exact hazard the two sites that *do* check `is_copy` were written against:
`resolution.rs:816-818` (*"The source_object belongs to the original spell and must not
be moved by a copy's resolution"*) and `resolution.rs:8485-8488` (*"`copy.rs` clones the
ORIGINAL's `kind` wholesale, so moving `source_object` here would put someone else's
spell in the graveyard"*). The mutate arm is the third such site and it was missed.

> **Defect sentence.** `rules::copy::copy_spell_on_stack` clones `original.kind`
> (`copy.rs:163`), so a copy of a mutating creature spell is a
> `MutatingCreatureSpell { source_object, .. }` naming the **original's** card, and the
> mutate resolution arm (`resolution.rs:7481-7620`) — unlike `resolution.rs:819` and
> `resolution.rs:8488`, which both guard on `is_copy` for this exact reason — moves or
> merges that card unconditionally (`:7515`, `:7536-7553`), so a resolving copy consumes
> the original spell's card and the original then resolves against a dead `ObjectId`;
> separately, `resolution.rs:7488-7493` silently substitutes `on_top = false` where
> CR 702.140c gives the copy's controller a choice.

Blast radius: **6** deck-legal `Complete` mutate defs (`brokkos_apex_of_forever`,
`gemrazer`, `glowstone_recluse`, `necropanther`, `sea_dasher_octopus`, `vulpikeet`);
**0** are reachable, because no copy source in the corpus can target a creature spell.
CR 707.10f additionally says a copy of a permanent spell becomes a token as it
resolves — which this engine does not do at all (§16) — so a CR-correct engine would
never take the merge branch for a copy in the first place. **A fix must decide 707.10f
first**; patching `is_copy` into the mutate arm in isolation would encode "a copy of a
mutate spell does nothing", which is a third wrong answer.

**MEDIUM-HIGH rather than HIGH** because it is unreachable today. Two facts must hold
for that to stay true, and both should be re-measured by whoever takes this: (i)
`Effect::CopySpellOnStack` still has zero genuine declarations, and (ii) no def with a
Storm/Gravestorm/Replicate/Casualty keyword is a creature spell.

## §16 — The bound both Squad/Offspring/Gift-permanent and Mutate rest on: **FILE (MEDIUM)**

Not an `AdditionalCost` finding, but it is what makes four of the rows above latent, and
it is not currently seeded anywhere I could find.

> **Defect sentence.** CR 707.10f / CR 608.3f — *"Some effects copy a permanent spell.
> As that copy resolves, it ceases being a copy of a spell and becomes a token
> permanent"* — is unimplemented: `resolution.rs:819-822` handles `stack_obj.is_copy`
> by emitting `SpellResolved` and nothing else, so a resolving copy of a creature,
> artifact, enchantment or planeswalker spell produces **no permanent at all**, and
> `grep -rn "608.3f\|707.10f" crates/engine/src crates/card-types/src --include=*.rs`
> returns zero hits, so nothing in the tree even records the gap.

Currently unreachable for the same corpus reason (no copy source can target a permanent
spell), so LOW blast radius but MEDIUM as a correctness hole — it is the sole reason
four of the twelve dropped variants cannot be demonstrated wrong by a test, and any
future card that copies a permanent spell makes all of them live at once.

## NIT — the comment itself

`copy.rs:241-242` should be rewritten whether or not any variant is added, because a
comment asserting a property nothing enforces is the defect this project keeps filing.
Three specific corrections: its **rule** is refuted by CR 707.10 (*"and additional or
alternative costs"*) and by CR 707.2's own *"whether it was kicked"*; its **list** names
6 of the 12 dropped variants and silently omits `EscapeExile`,
`CollectEvidenceExile`, `Assist`, `Replicate`, `Splice`, `ExileFromHand`; and it gives
no reason for the two drops that are genuinely CR-correct — Splice (CR 702.47c makes it
a text-changing effect, and CR 707.2 excludes text-changing effects) and Mutate's
`on_top` (CR 702.140c makes it a resolution choice, and CR 707.10 excludes those). The
allowlist as written is right about three variants for the right reason, right about two
more for a reason it does not state, and right about the remaining seven only because
nothing reads them.

## UNMEASURED

- **Whether any *test* fixture constructs a copy of a permanent spell** and would redden
  under a 707.10f implementation. Would be measured by grepping
  `crates/engine/tests/` and `crates/simulator/tests/` for `SpellCopied` /
  `copy_spell_on_stack` assertions against a creature-spell subject.
- **Whether the six copy-source defs' printed text actually restricts them to
  instants/sorceries in a way `casting.rs` enforces**, or whether the restriction is
  incidental to those cards' own type lines. This audit measured the *corpus*
  population, not the *engine's* enforcement of copy-target legality. Would be measured
  by reading the `TargetFilter` on the Storm/Gravestorm/Replicate/Casualty trigger paths
  in `resolution.rs:2631-2745` — they take a bare `original_stack_id` with no type
  filter, so the restriction appears to be entirely a property of which cards exist.
