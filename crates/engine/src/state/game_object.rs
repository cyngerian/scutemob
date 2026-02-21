//! Game object types: ObjectId, characteristics, status, and the GameObject struct.

use im::{OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};

use super::player::{CardId, PlayerId};
use super::types::{CardType, Color, CounterType, SubType, SuperType};
use super::zone::ZoneId;

/// Identifies a game object instance. Per CR 400.7, when an object changes
/// zones it becomes a new object with a new ObjectId.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u64);

/// Mana cost of a card or ability (CR 202).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaCost {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
    pub generic: u32,
}

impl ManaCost {
    /// Mana value (formerly "converted mana cost") per CR 202.3.
    pub fn mana_value(&self) -> u32 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless + self.generic
    }
}

/// The observable characteristics of a game object (CR 109.3).
///
/// These are the copiable values of an object — what a copy effect copies.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Characteristics {
    pub name: String,
    pub mana_cost: Option<ManaCost>,
    pub colors: OrdSet<Color>,
    pub color_indicator: Option<OrdSet<Color>>,
    pub supertypes: OrdSet<SuperType>,
    pub card_types: OrdSet<CardType>,
    pub subtypes: OrdSet<SubType>,
    pub rules_text: String,
    pub abilities: Vector<AbilityInstance>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub defense: Option<i32>,
}

/// Status bits for a permanent on the battlefield (CR 110.5).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectStatus {
    pub tapped: bool,
    pub flipped: bool,
    pub face_down: bool,
    pub phased_out: bool,
}

/// An instance of an ability on a game object.
/// Placeholder — will be fully defined in M3/M7.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbilityInstance {
    pub id: u64,
    pub description: String,
}

/// A game object — a card, token, copy, or ability on the stack (CR 109).
///
/// Every card and token in the game is represented as a GameObject with a
/// unique ObjectId. When an object changes zones, it gets a new ObjectId
/// per CR 400.7 ("an object that moves from one zone to another becomes a
/// new object with no memory of, or relation to, its previous existence").
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameObject {
    pub id: ObjectId,
    /// Links back to the physical card identity (survives zone changes).
    pub card_id: Option<CardId>,
    pub characteristics: Characteristics,
    pub controller: PlayerId,
    pub owner: PlayerId,
    pub zone: ZoneId,
    pub status: ObjectStatus,
    pub counters: OrdMap<CounterType, u32>,
    pub attachments: Vector<ObjectId>,
    pub attached_to: Option<ObjectId>,
    pub damage_marked: u32,
    pub is_token: bool,
    /// Timestamp for continuous effect ordering (CR 613.7).
    pub timestamp: u64,
}
