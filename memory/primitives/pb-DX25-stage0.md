# PB-DX25 — Stage 0: premise re-verification at HEAD (`cdbc542f`)

Worker: `scutemob-203`. Branch `feat/pb-dx25-effectcounterspells-three-stack-object-shapes-counte`.
Everything below was read or executed on this branch **before any edit**.

## Baseline (mandatory, pre-edit)

`cargo test --workspace --no-fail-fast` to a file:
**4,435 passed / 0 failed / 5 ignored** across 44 `test result` lines, exit 0.
Identical to CLAUDE.md's PB-DX24 pin, as expected (this branch forked from that merge).

## The premise holds. Site-by-site.

| Claim in the brief | Verified? | Evidence |
|---|---|---|
| `Effect::CounterSpell` at `effects/mod.rs:2721-2808` | yes | read |
| `position()` matches `so.id == id` **or** `Spell { source_object } == id` only | yes | `:2732-2738` — the `matches!` names `StackObjectKind::Spell` and nothing else |
| entry `remove(pos)`ed **before** the kind match | yes | `:2751` |
| match arms: `Spell`, `ActivatedAbility \| TriggeredAbility`, `_ =>` no-op | yes | `:2753`, `:2784`, `:2800-2803` |
| no `is_copy` check anywhere in the arm | yes | grep of the arm |
| `copy.rs` clones `kind` wholesale, sets `is_copy: true` | yes | `rules/copy.rs:150-175` — `kind: original.kind.clone()` |
| `MutatingCreatureSpell.source_object` is the card in `ZoneId::Stack` | yes | `card-types/src/state/stack.rs:852-870` (disc 59), and `casting.rs:4525-4530` picks the kind *after* the one `move_object_to_zone(card, Stack)` |
| only `Spell` and `MutatingCreatureSpell` carry a card | yes | matches the simulator's independent classification, `crates/simulator/src/invariants.rs:134-175` |

## What makes shape (c) reachable — the piece the brief did not state

`validate_target` for `TargetRequirement::TargetSpell` (`rules/casting.rs:6425-6453`) resolves the
target id through **`state.objects`** and requires `obj.zone == ZoneId::Stack`. So the id a counter
spell carries is the **CARD** on the stack, never the stack-object id.

- For a mutate spell that card **is** in `ZoneId::Stack`, so **target validation ACCEPTS it** and
  then `Effect::CounterSpell`'s `position()` finds nothing → the counter resolves and does
  *nothing*, silently. Shape (c) is live at the *offer* layer too, not just internally: the engine
  offers the target, takes the mana, and no-ops.
- A **copy**'s stack-object id is not in `state.objects` at all, so `TargetSpell` validation fails
  `ObjectNotFound` — which is exactly why v3 §1c's "(b) is unreachable at HEAD" is correct, and it
  is correct for a *second*, independent reason beyond the `position()`-finds-the-original-first
  one the memo gives. Recorded as a seed: **a copy of a spell can never be the target of a
  counterspell today**, though CR 707.10 makes it a spell.

## A second counter path exists and the brief does not mention it

`rules/resolution.rs::counter_stack_object` (`:8307-8400`) is a separate entry point (fizzle rule +
some effects). Its match **is** exhaustive and **does** pair `Spell | MutatingCreatureSpell`, so
shape (a) does not exist there. It has **no `is_copy` guard**, so shape (b)'s zone-move defect does
appear to exist there — reachability to be established during implementation.

## CR numbering correction (affects this batch's own criterion text)

The MCP rules server (authoritative per `memory/gotchas-rules.md`) at HEAD says:

- **CR 701.5 is `Cast`.** **CR 701.6 is `Counter`** — 701.6a: "To counter a spell or ability means
  to cancel it, removing it from the stack. It doesn't resolve and none of its effects occur. A
  countered spell is put into its owner's graveyard."
- CR 702.21 is Ward ✓ and CR 702.140 is Mutate ✓ (so the CR file is not globally offset; 701 shifted).

The acceptance criterion, `effects/mod.rs:2725`, `resolution.rs:8298`, `events.rs:159` and ~337
other sites in the tree cite **CR 701.5 for countering**. New probes in this batch cite **CR 701.6a**
(Architecture Invariant 8). The repo-wide re-cite is a doc-only sweep far outside this batch —
filed as a seed, with only the lines this batch already edits corrected in place.
