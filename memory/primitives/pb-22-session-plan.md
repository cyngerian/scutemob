# PB-22: Deferred Cleanup — Session Plan

> Clear the 13 deferred items from PB-0 through PB-21 before Phase 2 card authoring.
> Only 1 item (Tiamat multi-card search) is genuinely M10-blocked; the rest are small
> engine additions that were deferred for momentum, not complexity.

## Impact Summary

| Gap | Existing defs with TODOs | Unauthored cards needing it | Total |
|-----|--------------------------|----------------------------|-------|
| Emblem creation (CR 114) | 1 (Ajani) | ~10 planeswalkers | ~11 |
| Tapped-and-attacking tokens | 3 (Hanweir Garrison/Township, Den of the Bugbear) + 1 (Thousand-Faced Shadow) | ~3 | ~7 |
| Coin flip / d20 | 3 (Mana Crypt, Ancient Brass/Silver Dragon) | ~2 | ~5 |
| Reveal-route (top N → filter → zones) | 4 (Goblin Ringleader, Chaos Warp, Grief, Biting Palm Ninja) | ~5 | ~9 |
| Copy-as-token / become-copy | 4 (Scion, Shifting Woodland, Thousand-Faced Shadow, Thespian's Stage) | ~5 | ~9 |
| Adventure (cast from exile) | 2 (Monster Manual, Lozhan) | ~3 | ~5 |
| Activation condition (control N artifacts, etc.) | 0 | ~5 | ~5 |
| Sorcery-speed activation timing | 0 | ~3 | ~3 |
| Equipment auto-attach on ETB | 0 | ~2 | ~2 |
| Flicker (exile + return) | 0 | ~8 | ~8 |
| Dual-zone search (library OR graveyard) | 0 | ~2 | ~2 |
| Exact mana cost filter on search | 0 | ~1 | ~1 |
| **DEFERRED (M10)**: Tiamat multi-card choice | 0 | 1 | 1 |
| **Total** | ~17 | ~50 | **~67 cards** |

## Session Breakdown

### Session 1: Trivial Wiring (3 items, ~2 hrs)

**Items**: Things where the mechanism exists but isn't wired.

1. **Activation condition** — Add `activation_condition: Option<Condition>` to activated ability def.
   Add `Condition::YouControlNOrMorePermanents { filter: TargetFilter, count: u32 }`.
   Gate in `legal_actions.rs`. Unblocks Inventors' Fair, ~4 other cards.
   - Engine: `card_definition.rs`, `legal_actions.rs`, `hash.rs`
   - Tests: 3 (condition met, not met, changes dynamically)

2. **Exact mana cost filter** — Wire existing `TargetFilter` mana cost field into
   `SearchLibrary` filter evaluation in `effects/mod.rs`. Unblocks Urza's Saga.
   - Engine: `effects/mod.rs` (1-2 lines)
   - Tests: 2 (match, no match)

3. **Sorcery-speed activation timing** — Add `ActivationTiming::SorcerySpeed` enum.
   Add `timing_restriction: Option<ActivationTiming>` to activated ability def.
   Check in `legal_actions.rs`: only offer if main phase + stack empty + active player.
   - Engine: `card_definition.rs`, `legal_actions.rs`, `types.rs`, `hash.rs`
   - Tests: 3 (allowed on main phase, blocked on opponent turn, blocked with stack)

### Session 2: Coin Flip & Random Effects (1 item, ~3 hrs)

**Item**: Coin flip / d20 rolls.

1. **Effect::CoinFlip** — `{ on_heads: Box<Effect>, on_tails: Box<Effect> }`.
   Use deterministic RNG seeded from game state (reproducible replays).
   Add `GameEvent::CoinFlipped { player: PlayerId, result: bool }`.
   - Engine: `effects/mod.rs`, `types.rs`, `hash.rs`
   - Fix card defs: Mana Crypt
   - Tests: 4 (heads path, tails path, event emitted, deterministic replay)

2. **Effect::RollDice** — `{ sides: u32, results: Vec<(RangeInclusive<u32>, Effect)> }`.
   Same deterministic RNG. Add `GameEvent::DiceRolled { player, sides, result }`.
   - Engine: `effects/mod.rs`, `types.rs`, `hash.rs`
   - Fix card defs: Ancient Brass Dragon, Ancient Silver Dragon
   - Tests: 4 (low roll, high roll, event, deterministic)

### Session 3: Reveal-Route & Flicker (2 items, ~4 hrs)

1. **Effect::RevealAndRoute** — `{ count: u32, filter: TargetFilter, matched_dest: Zone, unmatched_dest: Zone }`.
   Reveals top N cards, routes matched cards to one zone, unmatched to another.
   Covers Goblin Ringleader pattern (creatures → hand, rest → bottom) and Chaos Warp
   pattern (reveal top → if permanent, put onto battlefield).
   - Engine: `effects/mod.rs`, `types.rs`, `hash.rs`
   - Fix card defs: Goblin Ringleader, Chaos Warp, Grief, Biting Palm Ninja
   - Tests: 5 (all match, none match, partial match, empty library, event)

2. **Effect::Flicker** — `{ target: EffectTarget, return_tapped: bool }`.
   Exiles target, returns to battlefield (under owner's control by default).
   Verify ETB triggers fire on return. May just be a convenience wrapper around
   `Sequence([ExileObject, MoveZone])` — check if that already works first.
   - Engine: `effects/mod.rs` (or just verify existing mechanism)
   - Tests: 3 (basic flicker, flicker triggers ETB, flicker with tapped)

### Session 4: Tapped-and-Attacking Tokens & Equipment Auto-Attach (2 items, ~3 hrs)

1. **Tapped-and-attacking token creation** — Extend `Effect::CreateToken` with
   `enters_tapped: bool` and `enters_attacking: Option<PlayerTarget>`.
   On token creation, set `tapped = true` and add to attacking creatures list
   (skip declare-attackers for these tokens, per CR 508.8).
   - Engine: `effects/mod.rs`, `combat.rs` (attacking list), `types.rs`, `hash.rs`
   - Fix card defs: Hanweir Garrison, Hanweir Township, Den of the Bugbear
   - Tests: 4 (token enters tapped, token is attacking, trigger fires, blocker assignment)

2. **Equipment auto-attach on ETB** — Add `Effect::AttachToTarget` or extend existing
   equipment attachment to support "enters the battlefield attached to" pattern.
   Uses existing `attached_to` field on GameObject.
   - Engine: `effects/mod.rs` (ETB replacement or triggered sequence)
   - Tests: 2 (auto-attaches, target leaves before resolution)

### Session 5: Copy/Clone Primitives (2 items, ~4 hrs)

1. **Effect::BecomeCopyOf** — Permanent becomes a copy of another permanent until EOT.
   Layer 1 copiable values replacement. Needs `ContinuousEffect` with
   `EffectDuration::UntilEndOfTurn` + `LayerModification::BecomeCopy { source: ObjectId }`.
   Covers Scion of the Ur-Dragon, Thespian's Stage, Shifting Woodland.
   - Engine: `copy.rs`, `layers.rs`, `effects/mod.rs`, `types.rs`
   - Fix card defs: Scion of the Ur-Dragon, Thespian's Stage, Shifting Woodland
   - Tests: 4 (becomes copy, reverts at EOT, copy of copy, layer interaction)

2. **Token copy entering tapped-and-attacking** — Combine token copy + Session 4's
   tapped-and-attacking. `Effect::CreateTokenCopy { source: EffectTarget, enters_tapped: bool, enters_attacking: Option<PlayerTarget> }`.
   Covers Thousand-Faced Shadow.
   - Engine: `effects/mod.rs`, `copy.rs`
   - Fix card defs: Thousand-Faced Shadow
   - Tests: 3 (copy token created, has source characteristics, tapped+attacking)

### Session 6: Emblem Creation (1 item, ~4 hrs)

**Item**: The biggest single unblock — 11 planeswalker cards.

1. **Emblem infrastructure** (CR 114):
   - Add `Zone::CommandZone` emblem placement (emblems live in command zone, CR 114.1)
   - Add `SubType::Emblem` or use existing type system (emblems have no card types but
     have abilities, CR 114.2)
   - Add `Effect::CreateEmblem { abilities: Vec<TriggeredAbilityDef> }` — creates a
     game object in the command zone with the specified abilities
   - Emblems are NOT permanents (CR 114.3) — they're objects in the command zone that
     have triggered/static abilities but can't be interacted with
   - SBA: emblems are never destroyed/exiled (CR 114.4)
   - Engine: `effects/mod.rs`, `types.rs`, `state/game_object.rs`, `hash.rs`
   - Fix card def: Ajani Sleeper Agent (−6 ability)
   - Tests: 5 (emblem created, trigger fires, emblem survives board wipe,
     emblem ability targets correctly, multiple emblems stack)

### Session 7: Adventure & Dual-Zone Search (2 items, ~4 hrs)

1. **Adventure casting** (CR 715):
   - Add `AltCostKind::Adventure` — cast the adventure half as an instant/sorcery
   - On resolution, exile the card instead of graveyard (CR 715.4)
   - From exile, can cast the creature half (CR 715.3d)
   - Need `CardFace`-like structure or reuse existing back_face for adventure half
   - Engine: `casting.rs`, `resolution.rs`, `types.rs`, `legal_actions.rs`
   - Fix card defs: Monster Manual
   - Tests: 5 (cast adventure, exiled after, cast creature from exile, adventure on stack
     is instant/sorcery type, countered adventure goes to graveyard not exile)

2. **Dual-zone search** — Extend `Effect::SearchLibrary` to optionally also search
   graveyard: `search_zones: Vec<Zone>` field (default `[Library]`). Or add
   `Effect::SearchZones { zones, filter, destination }`.
   Covers Finale of Devastation pattern.
   - Engine: `effects/mod.rs`
   - Tests: 3 (library only, graveyard only, both zones)

## Execution Notes

- **Order matters**: Sessions 1-2 are independent. Session 4 must come before Session 5
  (tapped-and-attacking tokens needed for copy-token-attacking). Session 6 (emblems) is
  independent. Session 7 is independent.
- **Parallel-safe**: Sessions 1+2, 3+6, and 4+5→7 are independent chains that could
  theoretically run in parallel worktrees.
- **Test command**: `~/.cargo/bin/cargo test --all` after each session.
- **Build check**: `~/.cargo/bin/cargo build --workspace` (catches replay-viewer/TUI breaks).
- **Commit prefix**: `W6-prim:` for each session.
- **After PB-22**: Update `docs/project-status.md` deferred items table (remove resolved),
  update card health stats, then proceed to Phase 2 card authoring.

## Items NOT in PB-22 (genuinely deferred)

| Item | Reason | Revisit When |
|------|--------|-------------|
| Tiamat multi-card search | Requires M10 interactive player choice | M10 |
| resolution.rs split | Organizational, no card impact | Post-Phase 2 |
| EffectContext refactor | Organizational, no card impact | Post-Phase 2 |
