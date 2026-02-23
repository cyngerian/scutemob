//! Zone types and zone storage (CR 400).
//!
//! MTG has seven zone types. Some are per-player (library, hand, graveyard,
//! command), others are shared (battlefield, stack, exile). Zones are either
//! ordered (position matters: library, graveyard, stack) or unordered.

use im::{OrdSet, Vector};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::game_object::ObjectId;
use super::player::PlayerId;

/// Zone types as described in CR 400.1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ZoneType {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Stack,
    Exile,
    Command,
}

/// Identifies a specific zone instance. Per-player zones encode the owner.
///
/// This enum makes invalid states unrepresentable — you can't accidentally
/// reference "player 3's battlefield" because the battlefield has no player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ZoneId {
    Library(PlayerId),
    Hand(PlayerId),
    Battlefield,
    Graveyard(PlayerId),
    Stack,
    Exile,
    Command(PlayerId),
}

impl ZoneId {
    pub fn zone_type(&self) -> ZoneType {
        match self {
            ZoneId::Library(_) => ZoneType::Library,
            ZoneId::Hand(_) => ZoneType::Hand,
            ZoneId::Battlefield => ZoneType::Battlefield,
            ZoneId::Graveyard(_) => ZoneType::Graveyard,
            ZoneId::Stack => ZoneType::Stack,
            ZoneId::Exile => ZoneType::Exile,
            ZoneId::Command(_) => ZoneType::Command,
        }
    }

    pub fn owner(&self) -> Option<PlayerId> {
        match self {
            ZoneId::Library(p) | ZoneId::Hand(p) | ZoneId::Graveyard(p) | ZoneId::Command(p) => {
                Some(*p)
            }
            ZoneId::Battlefield | ZoneId::Stack | ZoneId::Exile => None,
        }
    }

    /// Whether this zone type uses ordered storage (position matters).
    pub fn is_ordered(&self) -> bool {
        matches!(
            self,
            ZoneId::Library(_) | ZoneId::Graveyard(_) | ZoneId::Stack
        )
    }
}

/// A zone containing game objects.
///
/// Ordered zones (Library, Graveyard, Stack) use `Vector<ObjectId>` where
/// position matters. Unordered zones (Hand, Battlefield, Exile, Command) use
/// `OrdSet<ObjectId>` for deterministic iteration without positional semantics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Zone {
    /// Library, Graveyard, Stack — order matters.
    Ordered(Vector<ObjectId>),
    /// Hand, Battlefield, Exile, Command — order doesn't matter for game rules.
    Unordered(OrdSet<ObjectId>),
}

impl Zone {
    pub fn new_ordered() -> Self {
        Zone::Ordered(Vector::new())
    }

    pub fn new_unordered() -> Self {
        Zone::Unordered(OrdSet::new())
    }

    /// Create a zone with the appropriate storage type for the given ZoneId.
    pub fn for_zone_id(zone_id: &ZoneId) -> Self {
        if zone_id.is_ordered() {
            Zone::new_ordered()
        } else {
            Zone::new_unordered()
        }
    }

    pub fn contains(&self, id: &ObjectId) -> bool {
        match self {
            Zone::Ordered(v) => v.contains(id),
            Zone::Unordered(s) => s.contains(id),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Zone::Ordered(v) => v.len(),
            Zone::Unordered(s) => s.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Add an object to this zone. For ordered zones, appends to the end.
    pub fn insert(&mut self, id: ObjectId) {
        match self {
            Zone::Ordered(v) => v.push_back(id),
            Zone::Unordered(s) => {
                s.insert(id);
            }
        }
    }

    /// Remove an object from this zone. Returns true if it was present.
    pub fn remove(&mut self, id: &ObjectId) -> bool {
        match self {
            Zone::Ordered(v) => {
                if let Some(pos) = v.iter().position(|x| x == id) {
                    v.remove(pos);
                    true
                } else {
                    false
                }
            }
            Zone::Unordered(s) => s.remove(id).is_some(),
        }
    }

    /// Returns all object IDs in this zone in a consistent order.
    pub fn object_ids(&self) -> Vec<ObjectId> {
        match self {
            Zone::Ordered(v) => v.iter().copied().collect(),
            Zone::Unordered(s) => s.iter().copied().collect(),
        }
    }

    /// Shuffle this zone using the provided RNG. Only meaningful for ordered zones.
    /// Uses Fisher-Yates shuffle for uniform distribution.
    pub fn shuffle(&mut self, rng: &mut impl Rng) {
        if let Zone::Ordered(v) = self {
            let mut items: Vec<ObjectId> = v.iter().copied().collect();
            for i in (1..items.len()).rev() {
                let j = rng.gen_range(0..=i);
                items.swap(i, j);
            }
            *v = Vector::from(items);
        }
    }

    /// Insert an object at a specific position (only for ordered zones).
    /// For unordered zones, just inserts normally.
    pub fn insert_at(&mut self, index: usize, id: ObjectId) {
        match self {
            Zone::Ordered(v) => v.insert(index, id),
            Zone::Unordered(s) => {
                s.insert(id);
            }
        }
    }

    /// Get the top object (last element) of an ordered zone.
    pub fn top(&self) -> Option<ObjectId> {
        match self {
            Zone::Ordered(v) => v.last().copied(),
            Zone::Unordered(_) => None,
        }
    }

    /// Insert an object at the front (position 0) of an ordered zone.
    ///
    /// For ordered zones this places the object at the "bottom" (the end
    /// furthest from the top, which is the last element). Used by cascade
    /// to put exiled cards on the bottom of the library (CR 702.85a).
    /// For unordered zones, behaves identically to `insert`.
    pub fn push_front(&mut self, id: ObjectId) {
        match self {
            Zone::Ordered(v) => v.insert(0, id),
            Zone::Unordered(s) => {
                s.insert(id);
            }
        }
    }
}
