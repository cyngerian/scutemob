//! Card registry: maps CardId → CardDefinition for runtime effect lookup.
//!
//! The registry is static data — it never changes during a game. It is wrapped
//! in `Arc` so `GameState` can hold a reference without paying clone costs.
//!
//! Test code uses `CardRegistry::empty()`. Game code loads definitions at startup.
use super::card_definition::CardDefinition;
use crate::state::CardId;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
/// Why a set of `CardDefinition`s could not form a registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// Two definitions claim the same `CardId`.
    ///
    /// Previously the second definition silently overwrote the first (a `HashMap`
    /// collision), so a typo'd `cid(...)` in a new def could disable an unrelated
    /// card's abilities with no diagnostic anywhere.
    DuplicateCardId {
        card_id: CardId,
        first_name: String,
        second_name: String,
    },
}
impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::DuplicateCardId {
                card_id,
                first_name,
                second_name,
            } => write!(
                f,
                "duplicate CardId {:?}: registered by both {first_name:?} and {second_name:?}. \
                 Each card definition must declare a unique cid(...).",
                card_id.0
            ),
        }
    }
}
impl std::error::Error for RegistryError {}
/// A lookup table from card identity to card behavior definition.
///
/// Stored as `Arc<CardRegistry>` inside `GameState` so it is shared
/// cheaply across state snapshots (structural sharing).
#[derive(Clone, Debug, Default)]
pub struct CardRegistry {
    definitions: HashMap<CardId, CardDefinition>,
}
impl CardRegistry {
    /// An empty registry. Use in tests that don't rely on card effects.
    pub fn empty() -> Arc<Self> {
        Arc::new(Self::default())
    }
    /// Create a registry pre-populated with the given definitions.
    ///
    /// # Panics
    ///
    /// Panics if two definitions share a `CardId`. A duplicate is always a
    /// programming error in the card definitions themselves, and it is not
    /// recoverable at runtime — the registry is built once at startup. Callers
    /// that want to report the collision instead of aborting use [`Self::try_new`].
    pub fn new(definitions: impl IntoIterator<Item = CardDefinition>) -> Arc<Self> {
        match Self::try_new(definitions) {
            Ok(registry) => registry,
            Err(e) => panic!("CardRegistry::new: {e}"),
        }
    }
    /// Create a registry, returning [`RegistryError::DuplicateCardId`] rather than
    /// panicking when two definitions claim the same `CardId`.
    pub fn try_new(
        definitions: impl IntoIterator<Item = CardDefinition>,
    ) -> Result<Arc<Self>, RegistryError> {
        let mut map: HashMap<CardId, CardDefinition> = HashMap::new();
        for def in definitions {
            if let Some(first) = map.get(&def.card_id) {
                return Err(RegistryError::DuplicateCardId {
                    card_id: def.card_id.clone(),
                    first_name: first.name.clone(),
                    second_name: def.name.clone(),
                });
            }
            map.insert(def.card_id.clone(), def);
        }
        Ok(Arc::new(Self { definitions: map }))
    }
    /// Look up a card definition by its CardId.
    pub fn get(&self, card_id: CardId) -> Option<&CardDefinition> {
        self.definitions.get(&card_id)
    }
    /// Returns the number of registered card definitions.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}
