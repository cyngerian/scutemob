//! Greedy mana payment solver.
//!
//! For each colored pip, tap a source that produces that color.
//! For generic, tap any remaining source. Returns a sequence of
//! `TapForMana` commands.

use mtg_engine::{Command, GameState, ManaColor, ManaCost, ObjectId, PlayerId, ZoneId};

/// A mana source on the battlefield: its ObjectId, ability index, and what it produces.
#[derive(Clone, Debug)]
struct ManaSource {
    object_id: ObjectId,
    ability_index: usize,
    produces: Vec<ManaColor>,
    any_color: bool,
    tapped: bool,
}

/// Attempt to solve a mana payment greedily. Returns `TapForMana` commands
/// if a solution is found, or `None` if the cost can't be paid.
pub fn solve_mana_payment(
    state: &GameState,
    player: PlayerId,
    cost: &ManaCost,
) -> Option<Vec<Command>> {
    // Gather untapped mana sources controlled by this player
    let mut sources: Vec<ManaSource> = Vec::new();

    for obj in state.objects_in_zone(&ZoneId::Battlefield) {
        if obj.controller != player || obj.status.tapped {
            continue;
        }
        for (idx, ability) in obj.characteristics.mana_abilities.iter().enumerate() {
            if !ability.requires_tap {
                continue;
            }
            let mut produces = Vec::new();
            for (color, &amount) in ability.produces.iter() {
                for _ in 0..amount {
                    produces.push(*color);
                }
            }
            sources.push(ManaSource {
                object_id: obj.id,
                ability_index: idx,
                produces,
                any_color: ability.any_color,
                tapped: false,
            });
        }
    }

    let mut commands = Vec::new();
    let mut remaining = PipTracker::from_cost(cost);

    // Phase 1: pay colored pips with exact-match sources first
    for color in &[
        ManaColor::White,
        ManaColor::Blue,
        ManaColor::Black,
        ManaColor::Red,
        ManaColor::Green,
    ] {
        while remaining.colored(*color) > 0 {
            // Find a source producing this color (prefer non-any-color first)
            let found = sources
                .iter()
                .position(|s| !s.tapped && !s.any_color && s.produces.contains(color));
            let found = found.or_else(|| {
                sources
                    .iter()
                    .position(|s| !s.tapped && (s.produces.contains(color) || s.any_color))
            });

            if let Some(idx) = found {
                sources[idx].tapped = true;
                commands.push(Command::TapForMana {
                    player,
                    source: sources[idx].object_id,
                    ability_index: sources[idx].ability_index,
                });
                remaining.pay_colored(*color);
            } else {
                return None; // Can't pay this color
            }
        }
    }

    // Phase 2: pay colorless pips — ONLY colorless mana can pay {C} (CR 107.4c)
    while remaining.colorless > 0 {
        let found = sources
            .iter()
            .position(|s| !s.tapped && s.produces.contains(&ManaColor::Colorless));

        if let Some(idx) = found {
            sources[idx].tapped = true;
            commands.push(Command::TapForMana {
                player,
                source: sources[idx].object_id,
                ability_index: sources[idx].ability_index,
            });
            remaining.colorless -= 1;
        } else {
            return None; // No colorless source available — colored mana cannot pay {C}
        }
    }

    // Phase 3: pay generic with any remaining sources
    while remaining.generic > 0 {
        let found = sources.iter().position(|s| !s.tapped);
        if let Some(idx) = found {
            sources[idx].tapped = true;
            commands.push(Command::TapForMana {
                player,
                source: sources[idx].object_id,
                ability_index: sources[idx].ability_index,
            });
            remaining.generic -= 1;
        } else {
            return None;
        }
    }

    Some(commands)
}

/// Track remaining mana pips to pay.
struct PipTracker {
    white: u32,
    blue: u32,
    black: u32,
    red: u32,
    green: u32,
    colorless: u32,
    generic: u32,
}

impl PipTracker {
    fn from_cost(cost: &ManaCost) -> Self {
        Self {
            white: cost.white,
            blue: cost.blue,
            black: cost.black,
            red: cost.red,
            green: cost.green,
            colorless: cost.colorless,
            generic: cost.generic,
        }
    }

    fn colored(&self, color: ManaColor) -> u32 {
        match color {
            ManaColor::White => self.white,
            ManaColor::Blue => self.blue,
            ManaColor::Black => self.black,
            ManaColor::Red => self.red,
            ManaColor::Green => self.green,
            ManaColor::Colorless => self.colorless,
        }
    }

    fn pay_colored(&mut self, color: ManaColor) {
        match color {
            ManaColor::White => self.white = self.white.saturating_sub(1),
            ManaColor::Blue => self.blue = self.blue.saturating_sub(1),
            ManaColor::Black => self.black = self.black.saturating_sub(1),
            ManaColor::Red => self.red = self.red.saturating_sub(1),
            ManaColor::Green => self.green = self.green.saturating_sub(1),
            ManaColor::Colorless => self.colorless = self.colorless.saturating_sub(1),
        }
    }
}
