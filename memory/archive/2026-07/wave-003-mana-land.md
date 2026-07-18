# Wave 003: Mana Lands

**Group**: `mana-land`
**Status**: IN PROGRESS
**Sessions**: 71–77 (7 sessions)
**Total cards**: 92
**Reference card**: `crates/engine/src/cards/defs/command_tower.rs`

## DSL Pattern

Mana lands are the simplest card type — just a tap ability, no ETB replacement:

```rust
AbilityDefinition::Activated {
    cost: Cost::Tap,
    effect: Effect::AddMana { color: ManaColor::X, player: PlayerTarget::Controller },
    timing_restriction: None,
}
```

For two-color (dual) lands:
```rust
effect: Effect::Choose {
    player: PlayerTarget::Controller,
    choices: vec![
        Effect::AddMana { color: ManaColor::Black, player: PlayerTarget::Controller },
        Effect::AddMana { color: ManaColor::Green, player: PlayerTarget::Controller },
    ],
}
```

For "any color" lands (Command Tower style):
```rust
effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller }
```

For color-identity-specific (e.g. Secluded Courtyard, Unclaimed Territory):
```rust
effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller }  // best approximation
```

## Key DSL Gotchas

- `ManaCost` always needs all fields — use `..Default::default()` for zeros
- `*/*` CDA creatures: use `power: None, toughness: None` (not `Some(0)`)
- Lands with abilities beyond simple mana tap may need extra `AbilityDefinition` entries
- Utility lands (e.g. Strip Mine, Ghost Quarter, Wasteland) have activated sacrifice abilities
- Legendary lands (e.g. Minamo, Oboro) may have special tap abilities
- Urza's Saga is a Saga/Land — complex, use `abilities: vec![]` with TODO if needed
- Cavern of Souls: choose creature type at ETB — complex, TODO comment if needed

## Session Breakdown

| Session | Cards | Notes |
|---------|-------|-------|
| 71 | 16 | Cavern of Souls, Boseiju, Strip Mine, Gaea's Cradle, Nykthos, Ghost Quarter, painlands, Wasteland, utility |
| 72 | 16 | Hanweir Battlements, Command Beacon, Phyrexian Tower, Gemstone Caverns, Treasure Vault, channel lands, misc |
| 73 | 16 | Urza's Saga, tribal lands, original duals (Bayou etc.), snow basics, Inkmoth Nexus |
| 74 | 16 | Original duals, filter lands (Graven Cairns etc.), Shivan Reef, Blinkmoth Nexus, utility |
| 75 | 16 | Original duals, filter lands (Twilight Mire etc.), Kher Keep, Voldaren Estate, misc |
| 76 | 4  | Tainted Isle, Access Tunnel, The Seedcore, Gloomlake Verge |
| 77 | 8  | Channel lands (Takenuma etc.), Buried Ruin, Temple of the False God, Inventors' Fair, misc |

## Review Batches

(Filled in as reviews complete)

## Status Log

- [ ] Sessions 71-77 authored
- [ ] Reviews complete
- [ ] HIGH/MEDIUM fixes applied
- [ ] Tests passing
- [ ] Committed
