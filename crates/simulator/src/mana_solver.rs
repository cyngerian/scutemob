//! Greedy mana payment solver.
//!
//! For each colored pip, tap a source that produces that color.
//! For generic, tap any remaining source. Returns a sequence of
//! `TapForMana` commands.
//!
//! # Two things this solver does not do — **`OOS-M11-2`**
//!
//! Both halves are open, and this block exists because
//! [`crate::local_game::LocalGame::auto_tap_commands_for`] cites it by name for the
//! first of them and, until now, cited a sentence that was not here (review
//! MR-M11-12 — the lying-cite class this project keeps finding, one layer out).
//!
//! 1. **It ignores the mana pool entirely.** `solve_mana_payment` plans a payment for
//!    the *whole* cost from untapped sources, as though the pool were empty, so
//!    floating mana is neither spent nor counted. The caller compensates rather than
//!    the solver: `auto_tap_commands_for` checks the existing pool first (M11-local S3
//!    closed that half) and only calls in when the pool cannot already cover the cost.
//!    That keeps a human from over-tapping, and leaves the solver itself unchanged —
//!    so any *other* caller still gets the pool-blind behaviour.
//! 2. **It reads `obj.characteristics.mana_abilities` raw**, not through
//!    `calculate_characteristics`, so a source whose mana abilities are granted,
//!    removed or altered by a continuous effect (CR 613) is mis-planned. This is the
//!    layer-resolution half of `OOS-M11-2` and is still open — M11-local made no engine
//!    change, and the fix belongs with the PB-DX correctness queue.

use mtg_engine::{
    Command, GameState, HybridMana, ManaColor, ManaCost, ObjectId, PhyrexianMana, PlayerId, ZoneId,
};

/// A mana source on the battlefield: its ObjectId, ability index, and what it produces.
///
/// One entry per *ability*, so a permanent with two `requires_tap` mana abilities
/// contributes two entries with the same `object_id`. [`spend`] is what keeps those
/// entries from being planned independently — see its doc.
#[derive(Clone, Debug)]
struct ManaSource {
    object_id: ObjectId,
    ability_index: usize,
    produces: Vec<ManaColor>,
    any_color: bool,
    tapped: bool,
}

/// Mark the chosen source spent — **and every other entry for the same permanent**.
///
/// # The bug this closes (M11-local S8, found by the scripted playthrough)
///
/// `sources` holds one entry per (permanent × mana ability), and the three payment
/// phases below previously set `tapped` on the chosen entry alone. A permanent with two
/// `requires_tap` mana abilities therefore stayed selectable through its *other* entry,
/// so the solver could emit two `Command::TapForMana` for the same permanent. The first
/// taps it; the second is refused with `"permanent ObjectId(n) is already tapped"`,
/// because CR 602.2 pays a `{T}` cost by tapping an untapped permanent and it is no
/// longer untapped.
///
/// Observed rather than reasoned to: the S8 playthrough (seed 1, turn 21) submitted a
/// `CastSpell` the game had just offered and got exactly that rejection back. On the bot
/// path the same plan has always been silently absorbed — `LocalGame::advance`'s
/// command-rejected fallback issues `PassPriority` — which is why a bug reachable in
/// every game since the solver was written surfaced only once a *human* path refused to
/// swallow an engine error (S8 item 4).
fn spend(sources: &mut [ManaSource], idx: usize) {
    let object_id = sources[idx].object_id;
    for source in sources.iter_mut() {
        if source.object_id == object_id {
            source.tapped = true;
        }
    }
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

            // None => can't pay this color.
            let idx = found?;
            spend(&mut sources, idx);
            // PB-EF12 (CR 605.3b): an any_color source can satisfy any colour — choose
            // the exact one needed here. A fixed-colour source carries no choice.
            let chosen_color = if sources[idx].any_color {
                Some(*color)
            } else {
                None
            };
            commands.push(Command::TapForMana {
                player,
                source: sources[idx].object_id,
                ability_index: sources[idx].ability_index,
                chosen_color,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            });
            remaining.pay_colored(*color);
        }
    }

    // Phase 2: pay colorless pips — ONLY colorless mana can pay {C} (CR 107.4c)
    while remaining.colorless > 0 {
        let found = sources
            .iter()
            .position(|s| !s.tapped && s.produces.contains(&ManaColor::Colorless));

        // None => no colorless source available; colored mana cannot pay {C} (CR 107.4c).
        let idx = found?;
        spend(&mut sources, idx);
        // PB-EF12: an any_color source's `produces` is empty (CR 106.1b: colorless is
        // not a legal "any color" choice), so it never matches the `contains(&Colorless)`
        // filter above — this source is always fixed-colour, chosen_color is None.
        commands.push(Command::TapForMana {
            player,
            source: sources[idx].object_id,
            ability_index: sources[idx].ability_index,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        });
        remaining.colorless -= 1;
    }

    // Phase 3: pay generic with any remaining sources
    while remaining.generic > 0 {
        let found = sources.iter().position(|s| !s.tapped);
        // None => no untapped source left to pay the remaining generic cost.
        let idx = found?;
        spend(&mut sources, idx);
        // PB-EF12: a generic pip can be paid with any colour, so an any_color source
        // just needs *a* legal choice — deterministic White, mirroring legal_actions.rs.
        let chosen_color = if sources[idx].any_color {
            Some(ManaColor::White)
        } else {
            None
        };
        commands.push(Command::TapForMana {
            player,
            source: sources[idx].object_id,
            ability_index: sources[idx].ability_index,
            chosen_color,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        });
        remaining.generic -= 1;
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
        let mut tracker = Self {
            white: cost.white,
            blue: cost.blue,
            black: cost.black,
            red: cost.red,
            green: cost.green,
            colorless: cost.colorless,
            generic: cost.generic,
        };

        // Flatten hybrid mana into colored pip requirements (CR 107.4e).
        // Default: pay with the first color (ColorColor) or the specific color (GenericColor).
        // GenericColor ({2/W} etc.) defaults to paying the color pip, not 2 generic.
        for hybrid in &cost.hybrid {
            match hybrid {
                HybridMana::ColorColor(c1, _c2) => {
                    // Default: pay with first color.
                    tracker.add_color(*c1, 1);
                }
                HybridMana::GenericColor(c) => {
                    // Default: pay with the color (not 2 generic).
                    tracker.add_color(*c, 1);
                }
            }
        }

        // Flatten Phyrexian mana into colored pip requirements (CR 107.4f).
        // Default: pay with mana (not 2 life).
        for phyrexian in &cost.phyrexian {
            match phyrexian {
                PhyrexianMana::Single(c) => {
                    tracker.add_color(*c, 1);
                }
                PhyrexianMana::Hybrid(c1, _c2) => {
                    // Default: pay with first color.
                    tracker.add_color(*c1, 1);
                }
            }
        }

        // x_count: X defaults to 0 for the solver (CR 202.3e), so no generic added.

        tracker
    }

    fn add_color(&mut self, color: ManaColor, amount: u32) {
        match color {
            ManaColor::White => self.white += amount,
            ManaColor::Blue => self.blue += amount,
            ManaColor::Black => self.black += amount,
            ManaColor::Red => self.red += amount,
            ManaColor::Green => self.green += amount,
            ManaColor::Colorless => self.colorless += amount,
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
