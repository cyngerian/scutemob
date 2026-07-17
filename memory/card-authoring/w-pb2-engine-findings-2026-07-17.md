# W-PB2 engine findings (scutemob-95) — 2026-07-17

Findings surfaced while authoring the W-PB2 wave. Per the task guardrail ("don't
implement engine changes in this wave — mark and file"), these are **filed, not fixed**.
Each names a card that stays non-`Complete` because of it.

## EF-W-PB2-1 — `EffectAmount::PermanentCount` ignores `exclude_self` (MEDIUM)

`effects/mod.rs:6749` — the `PermanentCount` resolver closure filters on zone / phased-in /
controller / `matches_filter` / chosen-subtype / counter-type, but **never applies
`filter.exclude_self`**. Its sibling amount resolvers do: `AttackingCreatureCount`
(`effects/mod.rs:7032`) and `TappedCreatureCount` (`:7066`) both guard `obj.id != ctx.source`.

**Instance**: `eomer_king_of_rohan.rs`. Oracle: "Éomer enters with a +1/+1 counter on it for
each **other** Human you control." The def is correct — `EntersWithCounters` with
`count = PermanentCount{ has_subtype: Human, controller: You, exclude_self: true }` — but the
self-ETB replacement resolves the count with `ctx.source = Éomer` **after** Éomer is already on
the battlefield (moved at `resolution.rs:576`, replacement runs at `:1646`). Éomer is a Human
you control, so it counts itself: a 2/2 with no other Humans enters as a 3/3.

**Éomer is the ONLY def using `PermanentCount + exclude_self: true`** (grep-verified over the
corpus), so the fix is zero-risk to existing cards and changes no existing test's hash.

**Fix (one line)**: add `&& (!filter.exclude_self || obj.id != ctx.source)` to the
`PermanentCount` closure, mirroring `:7032`/`:7066`. Then flip `eomer_king_of_rohan.rs` to
`Complete` and add a regression test asserting the entered counter count with 0 and with N
other Humans. Demoted to `known_wrong` here.

## EF-W-PB2-2 — no opponent-restricted player `TargetRequirement` (MEDIUM)

`TargetRequirement` has `TargetPlayer` (any player) but **no `TargetOpponent`** variant, so
"target opponent …" oracle text cannot be authored without permitting an illegal self-target
(KI-1). Confirmed by author against the DSL and 3 sibling defs already stuck on this exact gap:
`raiders_wake.rs`, `forbidden_orchard.rs`, `ajani_sleeper_agent.rs`.

**Instance**: `shaman_of_the_pack.rs`. The ETB *amount* is now expressible
(`PermanentCount{ has_subtype: Elf, controller: You }`), but "target opponent loses life" is not.
Stays `partial`. Also unblocks the 3 sibling defs above.

**Fix**: add `TargetRequirement::TargetOpponent` (+ its validation arm restricting candidates to
opponents of the source's controller, CR 115.x). A PB-sized task, out of scope for a
card-authoring wave.

## LOW / accepted (non-blocking, cards ship Complete)

- **avenger_of_zendikar** — landfall "you may put a +1/+1 counter" modeled as mandatory (the
  effect is always beneficial; matches the khalni/roster convention for non-interactive "may").
- **access_denied** — `ManaValueOf` sums the countered spell's printed cost and omits a chosen
  `X`, so countering an `{X}` spell undercounts the Thopter tokens. Narrow, pre-existing
  `ManaValueOf` limitation (the blessed primitive for this card per the marker sweep); accepted
  as Complete for the non-X case.
