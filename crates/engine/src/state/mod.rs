//! Game state model: zones, objects, players, and the core GameState struct.
//!
//! All state uses `im` persistent data structures for structural sharing,
//! enabling cheap snapshots and deterministic replay.
