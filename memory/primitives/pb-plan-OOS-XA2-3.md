# OOS-XA2-3: `is_nontoken` target-side enforcement audit — Plan & Conclusion

**Task**: scutemob-30
**Branch**: feat/oos-xa2-3-isnontoken-target-side-enforcement-audit-fix-if-ga
**Date**: 2026-05-15
**Carryforward of**: OOS-XA-3 (filed by PB-XA scutemob-24), OOS-XA2-3 (filed by PB-XA2)
**Outcome**: **0-yield audit — NO engine change** (valid signal-ready, OOS-XS-E-1 precedent)

---

## Question

Is `TargetFilter.is_nontoken` (`card_definition.rs:2581-2590`) enforced at the
TARGET-VALIDATION path (`casting::validate_object_satisfies_requirement`) and the
TRIGGER auto-target picker (`abilities.rs`), or only at the effect-resolution path
(`effects/mod.rs:2683`)? If a real target-side consumer exists with missing
enforcement, the PB-XA pattern must be applied. If none exists, this closes
audit-only.

## Audit step 1 — `is_nontoken` usage across card defs

`grep -rn 'is_nontoken' crates/engine/src/cards/defs/` →
**exactly one occurrence**, in `accursed_marauder.rs`:

```rust
// Accursed Marauder — "When this enters, each player sacrifices a nontoken creature."
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenEntersBattlefield,
    effect: Effect::SacrificePermanents {
        player: PlayerTarget::EachPlayer,
        count: EffectAmount::Fixed(1),
        filter: Some(TargetFilter {
            has_card_type: Some(CardType::Creature),
            is_nontoken: true,           // <-- the only is_nontoken: true in the codebase
            ..Default::default()
        }),
    },
    intervening_if: None,
    targets: vec![],                     // <-- NOT a targeted trigger
    modes: None,
    trigger_zone: None,
}
```

**Classification**: This `is_nontoken: true` sits inside an **effect-side
`EffectFilter`** — the `filter` field of `Effect::SacrificePermanents`. It is
**NOT** inside any `TargetRequirement::Target*WithFilter` block. The trigger's
`targets` vec is empty; the sacrifice is a non-targeted "each player sacrifices"
choice resolved at effect time, not a target chosen at cast/trigger time.

`is_token: true` (the mirror field) is used in **zero** card defs.

**Audit step 1 conclusion: there is no target-side `is_nontoken` consumer in the
current card defs. The single consumer is effect-side.**

## Audit step 2 — enforcement at the target-validate path & auto-target picker

`grep -rn 'is_nontoken' crates/engine/src/` (non-defs) → field definition at
`card_definition.rs:2590`, hash at `state/hash.rs:4436`, and exactly one
behavioral consumer: `effects/mod.rs:2683`.

### Target-validate path — `validate_object_satisfies_requirement` (`casting.rs:5707-5797`)

`TargetCreatureWithFilter` and `TargetPermanentWithFilter` check:
`matches_filter`, `passes_controller`, `passes_self` (exclude_self),
`passes_combat_role` (is_attacking/is_blocking), `passes_tapped`,
`passes_untapped`. **No `is_nontoken` and no `is_token` check.**

### Trigger auto-target picker (`abilities.rs:6780-6853`)

Same battlefield-object filter set: `matches_filter`, `ctrl_ok`, `passes_self`,
`passes_combat_role`, `passes_tapped`, `passes_untapped`. **No `is_nontoken` and
no `is_token` check.**

### Effect-resolution path (`effects/mod.rs:2683`) — the ONE consumer

`Effect::SacrificePermanents` checks the runtime GameObject field explicitly:

```rust
// `is_nontoken: true` means "must NOT be a token" (nontoken permanent).
if tf.is_nontoken && obj.is_token {
    return false;
}
```

This correctly enforces Accursed Marauder. `is_nontoken` is a runtime
`GameObject` field, NOT a `Characteristics` field, so `matches_filter()` cannot
see it — it is (correctly, per the field doc-comment at `card_definition.rs:2586-2588`)
checked explicitly at the one site that uses it.

**Audit step 2 conclusion: enforcement is ABSENT at the target-validate path and
auto-target picker, but UNREACHABLE — there is no target-side consumer to
enforce. The only consumer (effect-side SacrificePermanents) IS correctly
enforced.**

## Decision — IMPLEMENT or CLOSE

Criterion 3969 conditional: a fix is required IFF audit step 1 finds a
target-side `is_nontoken` consumer AND audit step 2 shows enforcement absent.

- Audit step 1: **no target-side consumer** (the sole `is_nontoken: true` is
  effect-side).
- → The `AND` is not satisfied. **No engine fix.**

**Decision: CLOSE audit-only. 0-yield.** This mirrors the OOS-XS-E-1 precedent
(dies-side audit closed with no engine change because no consumer existed).

The "absent at validate path" finding is real but harmless: it is a latent gap,
not an active one. The field doc-comment at `card_definition.rs:2586-2588`
already warns that `is_nontoken` "MUST be checked explicitly at each call site
that uses it — it will be silently ignored by `matches_filter()`." No current
call site (target-side) uses it. If a future card def introduces a target-side
`is_nontoken` filter (oracle "target nontoken creature"), the implementing PB
must add the `!filter.is_nontoken || !state.objects.get(&id).is_some_and(|o| o.is_token)`
guard to both `Target*WithFilter` arms in `casting.rs` and `abilities.rs` — the
identical pattern PB-XA2 used for `is_tapped`/`is_untapped`. This is a known,
documented latent gap, not a shipped bug, and matches how `is_token` itself is
currently handled (also no target-side consumer).

## HASH

No HASH bump. `is_nontoken` already exists on `TargetFilter` and is already
hashed (`state/hash.rs:4436`). No schema change.

## Gates

`cargo test/clippy/fmt/build --workspace` run as final verification — no engine
change means these confirm the tree is still clean (no regression introduced by
the doc-only edits to `pb-retriage-CC.md` / new plan & review-skip files).
