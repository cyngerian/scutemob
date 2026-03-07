//! Player identity and state types.

use im::{OrdMap, Vector};
use serde::{Deserialize, Serialize};

use super::types::ManaColor;

/// Identifies a player in the game. Unique within a game instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u64);

/// Identifies a physical card across zone changes.
///
/// A CardId persists even when the game object changes zones and gets a new
/// ObjectId (per CR 400.7). Used for commander tracking, commander tax, and
/// commander damage — the physical card identity must survive zone changes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CardId(pub String);

/// A player's mana pool (CR 106.4).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaPool {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
}

impl ManaPool {
    pub fn total(&self) -> u32 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless
    }

    pub fn add(&mut self, color: ManaColor, amount: u32) {
        match color {
            ManaColor::White => self.white += amount,
            ManaColor::Blue => self.blue += amount,
            ManaColor::Black => self.black += amount,
            ManaColor::Red => self.red += amount,
            ManaColor::Green => self.green += amount,
            ManaColor::Colorless => self.colorless += amount,
        }
    }

    pub fn empty(&mut self) {
        *self = ManaPool::default();
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

/// Complete state of a single player.
///
/// Commander-specific fields included per CR 903:
/// - `life_total` starts at 40 (CR 903.7)
/// - `commander_tax` tracks additional cost per commander cast from command zone
/// - `commander_damage_received` tracks combat damage per source commander per
///   opponent, nested for partner commander tracking (CR 903.10a)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    pub life_total: i32,
    pub mana_pool: ManaPool,
    /// Additional cost to cast each commander from the command zone.
    /// Key is CardId of the commander card.
    pub commander_tax: OrdMap<CardId, u32>,
    /// Combat damage received from each commander, tracked per opponent.
    /// Outer key: opponent PlayerId. Inner key: CardId of that opponent's commander.
    pub commander_damage_received: OrdMap<PlayerId, OrdMap<CardId, u32>>,
    pub poison_counters: u32,
    pub land_plays_remaining: u32,
    pub has_drawn_for_turn: bool,
    pub has_lost: bool,
    pub has_conceded: bool,
    /// CardIds of this player's commander(s). Supports partner commanders.
    pub commander_ids: Vector<CardId>,
    /// Maximum hand size for cleanup discard (CR 402.2). Default 7.
    pub max_hand_size: usize,
    /// CardId of this player's companion, if any (CR 702.139).
    ///
    /// A companion is a card in the sideboard with the Companion keyword
    /// that meets the deck restriction. Set at game setup before mulligan.
    pub companion: Option<CardId>,
    /// Whether this player has already used the companion special action
    /// (CR 702.139a). Once used, `BringCompanion` is rejected.
    pub companion_used: bool,
    /// Number of mulligans taken by this player during the pregame procedure
    /// (CR 103.5). Used to determine how many cards must go to the bottom of
    /// library on `KeepHand` (CR 103.5c: first mulligan free, subsequent cost
    /// N-1 cards where N is mulligan number).
    pub mulligan_count: u32,
    /// CR 402.2: If true, this player has no maximum hand size and does not
    /// discard to hand size during cleanup (CR 514.1).
    ///
    /// Set by `rules/sba.rs` or `rules/layers.rs` when a permanent with the
    /// `KeywordAbility::NoMaxHandSize` keyword is on the battlefield under this
    /// player's control (e.g. Thought Vessel, Reliquary Tower).
    #[serde(default)]
    pub no_max_hand_size: bool,
    /// Number of cards drawn by this player this turn (CR 121.1).
    ///
    /// Incremented each time `draw_one_card` completes successfully. Reset to 0
    /// at the start of this player's turn in `reset_turn_state`. Used by Sylvan
    /// Library (CC#33) and other effects that track how many cards have been drawn.
    #[serde(default)]
    pub cards_drawn_this_turn: u32,
    /// Number of spells cast by this player this turn (CR 702.40a).
    ///
    /// Incremented by `casting::handle_cast_spell` on each successful cast.
    /// Reset to 0 at the start of this player's turn in `reset_turn_state`.
    /// Used by the storm keyword to count copies (storm count = spells cast before
    /// the storm spell this turn, i.e., `spells_cast_this_turn - 1` at trigger time).
    #[serde(default)]
    pub spells_cast_this_turn: u32,
    /// CR 702.131c: The city's blessing is a designation that has no rules meaning
    /// other than to act as a marker. Once a player gets the city's blessing, they
    /// keep it for the rest of the game (never removed). Set by Ascend checks.
    #[serde(default)]
    pub has_citys_blessing: bool,
    /// Amount of life this player has lost this turn (CR 702.137a, CR 118.4).
    ///
    /// Incremented whenever this player's life total decreases due to damage
    /// or life loss effects. Reset to 0 at the start of each turn in
    /// `reset_turn_state`. Used by Spectacle to check if an opponent
    /// lost life this turn.
    #[serde(default)]
    pub life_lost_this_turn: u32,
    /// Total damage dealt to this player this turn (CR 120.2, CR 702.54a).
    ///
    /// Incremented whenever this player is dealt damage (combat or non-combat,
    /// including infect damage that gives poison instead of life loss).
    /// Reset to 0 at the start of each turn in `reset_turn_state`.
    /// Used by Bloodthirst to check if an opponent was dealt damage this turn
    /// and by Bloodthirst X to determine the total.
    #[serde(default)]
    pub damage_received_this_turn: u32,
}
