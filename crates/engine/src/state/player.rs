//! Player identity and state types.

use im::{OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};

use super::dungeon::DungeonId;
use super::types::{ManaColor, ProtectionQuality, SubType};
use crate::cards::card_definition::ManaRestriction;

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

/// A single entry of restricted mana in a player's pool (CR 106.12).
///
/// Restricted mana can only be spent on costs that match the restriction.
/// When mana is produced with a restriction, it goes into `ManaPool::restricted`
/// instead of the unrestricted color buckets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestrictedMana {
    pub color: ManaColor,
    pub amount: u32,
    pub restriction: ManaRestriction,
}

/// Context about a spell being cast, used to check mana spending restrictions.
#[derive(Clone, Debug)]
pub struct SpellContext {
    /// Whether the spell is a creature spell.
    pub is_creature: bool,
    /// Creature subtypes of the spell (if any).
    pub subtypes: Vec<SubType>,
}

/// A player's mana pool (CR 106.4).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaPool {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
    /// Restricted mana entries (CR 106.12). Each entry tracks mana that can only
    /// be spent on matching spells/abilities.
    #[serde(default)]
    pub restricted: Vec<RestrictedMana>,
}

impl ManaPool {
    /// Total unrestricted mana in the pool.
    pub fn total(&self) -> u32 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless
    }

    /// Total mana including restricted entries.
    pub fn total_with_restricted(&self) -> u32 {
        self.total() + self.restricted.iter().map(|r| r.amount).sum::<u32>()
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

    /// Add restricted mana to the pool (CR 106.12).
    pub fn add_restricted(&mut self, color: ManaColor, amount: u32, restriction: ManaRestriction) {
        // Merge with existing entry if same color and restriction
        for entry in &mut self.restricted {
            if entry.color == color && entry.restriction == restriction {
                entry.amount += amount;
                return;
            }
        }
        self.restricted.push(RestrictedMana {
            color,
            amount,
            restriction,
        });
    }

    /// Get the amount of restricted mana of a specific color that matches the spell context.
    pub fn restricted_available(&self, color: ManaColor, spell: &SpellContext) -> u32 {
        self.restricted
            .iter()
            .filter(|r| r.color == color && restriction_matches(&r.restriction, spell))
            .map(|r| r.amount)
            .sum()
    }

    /// Spend restricted mana of a specific color that matches the spell context.
    /// Returns the amount actually spent.
    pub fn spend_restricted(
        &mut self,
        color: ManaColor,
        mut amount: u32,
        spell: &SpellContext,
    ) -> u32 {
        let mut spent = 0;
        for entry in &mut self.restricted {
            if amount == 0 {
                break;
            }
            if entry.color == color && restriction_matches(&entry.restriction, spell) {
                let take = amount.min(entry.amount);
                entry.amount -= take;
                amount -= take;
                spent += take;
            }
        }
        // Remove depleted entries
        self.restricted.retain(|r| r.amount > 0);
        spent
    }

    pub fn empty(&mut self) {
        *self = ManaPool::default();
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0 && self.restricted.is_empty()
    }
}

/// Check if a mana restriction matches the spell being cast.
pub fn restriction_matches(restriction: &ManaRestriction, spell: &SpellContext) -> bool {
    match restriction {
        ManaRestriction::CreatureSpellsOnly => spell.is_creature,
        ManaRestriction::SubtypeOnly(st) => spell.subtypes.iter().any(|s| s == st),
        ManaRestriction::SubtypeOrSubtype(a, b) => spell.subtypes.iter().any(|s| s == a || s == b),
        ManaRestriction::ChosenTypeCreaturesOnly => {
            // The chosen type is resolved before calling restriction_matches —
            // if this variant appears, it should have been resolved to SubtypeOnly
            // by the effect executor. As a fallback, treat as creature-only.
            spell.is_creature
        }
        ManaRestriction::ChosenTypeSpellsOnly => {
            // Same: should be resolved before reaching here.
            true
        }
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
    /// CR 702.16b/e: Protection qualities granted to this player by continuous effects.
    ///
    /// Most cards grant protection to permanents, not players. However, some effects
    /// (e.g., Teferi's Protection) can grant a player protection from everything.
    /// When non-empty, spells/abilities from matching sources cannot target this player
    /// (CR 702.16b) and damage from matching sources is prevented (CR 702.16e).
    ///
    /// Populated by continuous effects; empty by default.
    #[serde(default)]
    pub protection_qualities: Vec<ProtectionQuality>,
    /// CR 309.7: Count of dungeons this player has completed.
    ///
    /// Incremented each time a player's dungeon is removed from the game after reaching
    /// the bottommost room (CR 309.7: "A player completes a dungeon as it leaves the game.").
    ///
    /// Used by `Condition::CompletedADungeon` for cards like Nadaar, Selfless Paladin.
    /// Never decreases during a game.
    #[serde(default)]
    pub dungeons_completed: u32,
    /// Set of specific dungeon IDs this player has completed (CR 309.7).
    ///
    /// Used by `Condition::CompletedSpecificDungeon` for cards like Acererak
    /// ("if you haven't completed Tomb of Annihilation").
    #[serde(default)]
    pub dungeons_completed_set: OrdSet<DungeonId>,
    /// CR 701.54c: Number of times the Ring has tempted this player (0-4, capped).
    ///
    /// Determines which ring abilities are active. Once incremented, never decreases.
    /// 0 = the Ring has never tempted this player (no emblem yet).
    /// 1-4 = the ring level; higher levels unlock additional ring abilities.
    #[serde(default)]
    pub ring_level: u8,
    /// CR 701.54a: ObjectId of this player's current ring-bearer creature.
    ///
    /// `None` if the player has no ring-bearer (never tempted, no creatures when
    /// tempted, or ring-bearer left the battlefield / changed control per CR 701.54a).
    /// Stored as `ObjectId` because the designation is lost on zone change (CR 400.7).
    #[serde(default)]
    pub ring_bearer_id: Option<crate::state::game_object::ObjectId>,
}
