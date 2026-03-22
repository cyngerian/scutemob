//! Card registry: maps CardId → CardDefinition for runtime effect lookup.
//!
//! The registry is static data — it never changes during a game. It is wrapped
//! in `Arc` so `GameState` can hold a reference without paying clone costs.
//!
//! Test code uses `CardRegistry::empty()`. Game code loads definitions at startup.
use super::card_definition::CardDefinition;
use crate::state::CardId;
use std::collections::HashMap;
use std::sync::Arc;
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
    pub fn new(definitions: impl IntoIterator<Item = CardDefinition>) -> Arc<Self> {
        let map: HashMap<CardId, CardDefinition> = definitions
            .into_iter()
            .map(|d| (d.card_id.clone(), d))
            .collect();
        Arc::new(Self { definitions: map })
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
