//! Zone types and zone storage (CR 400).
//!
//! MTG has seven zone types. Some are per-player (library, hand, graveyard,
//! command), others are shared (battlefield, stack, exile). Zones are either
//! ordered (position matters: library, graveyard, stack) or unordered.
use super::game_object::ObjectId;
use super::player::PlayerId;
use imbl::{OrdSet, Vector};
use serde::{Deserialize, Serialize};
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
    /// CR 103.3 / 701.24 — shuffle this zone with a **pinned**, in-tree algorithm. Only
    /// meaningful for ordered zones.
    ///
    /// # Why this does not take an `impl Rng` (PB-DX18, `OOS-DP2-4`)
    ///
    /// It used to, and every caller handed it `rand::rngs::StdRng::seed_from_u64(seed)`.
    /// That left **two** independent channels through which a `rand` version bump could
    /// silently re-permute every seeded shuffle in the engine — opening libraries
    /// included — with no fingerprint gate able to see it, because
    /// `PROTOCOL_SCHEMA_FINGERPRINT` and `HASH_SCHEMA_FINGERPRINT` digest type
    /// declarations, not shuffle output:
    ///
    /// 1. **The generator.** `StdRng` is explicitly not algorithm-stable across `rand`
    ///    major versions. This is the channel `OOS-DP2-4`'s addendum names.
    /// 2. **The index draw.** `Rng::random_range` is equally unpinned by `rand`'s own
    ///    stability policy, so pinning only the generator would leave the identical
    ///    defect one layer down. The seed names one mechanism; there are two.
    ///
    /// Both are removed by depending on no external RNG at all. The generator is
    /// SplitMix64 (Steele/Lea/Flood 2014, a fully specified 64-bit mix) and the bounded
    /// draw is rejection sampling on a power-of-two mask, so the permutation is a pure
    /// function of `(seed, len)` written here and nowhere else.
    ///
    /// `state::zone::pinned_rng_tests::shuffle_pinned_is_a_pinned_permutation` fixes the output for
    /// a known seed, so a future edit to either half is a red test rather than a silent
    /// re-deal.
    pub fn shuffle_pinned(&mut self, seed: u64) {
        if let Zone::Ordered(v) = self {
            let mut items: Vec<ObjectId> = v.iter().copied().collect();
            let mut rng = PinnedRng::new(seed);
            // Fisher-Yates, high-to-low (CR 103.3: any permutation, uniformly).
            for i in (1..items.len()).rev() {
                let j = rng.below((i + 1) as u64) as usize;
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
    /// Get the top `n` objects of an ordered zone, ordered from the top down.
    ///
    /// Index 0 of the returned vector is the topmost card — the same card
    /// `Zone::top()` returns and the same card a draw takes (CR 121.1).
    /// Because ordered zones store the top at the LAST index, this walks the
    /// backing vector in reverse.
    ///
    /// Returns fewer than `n` entries if the zone is smaller (CR 401.7-adjacent:
    /// callers must tolerate a short read). Returns empty for unordered zones,
    /// matching `top()`.
    pub fn top_n(&self, n: usize) -> Vec<ObjectId> {
        match self {
            Zone::Ordered(v) => v.iter().rev().take(n).copied().collect(),
            Zone::Unordered(_) => Vec::new(),
        }
    }
    /// Insert an object at the front (position 0) of an ordered zone.
    ///
    /// For ordered zones this places the object at index 0, which is the
    /// **bottom** — ordered zones store the top at the **last** index
    /// (`Zone::top()` is `v.last()`), so index 0 is the end furthest from
    /// the top. Used by cascade to put exiled cards on the bottom of the
    /// library (CR 702.85a). For unordered zones, behaves identically to
    /// `insert`.
    pub fn push_front(&mut self, id: ObjectId) {
        match self {
            Zone::Ordered(v) => v.insert(0, id),
            Zone::Unordered(s) => {
                s.insert(id);
            }
        }
    }
    /// CR 400.7 / CR 401.4 (PB-DP9): permute cards **within** this zone without
    /// any of them changing zones.
    ///
    /// `to_top` is top-first (`to_top[0]` finishes as the zone's top card);
    /// `to_bottom` is also top-first among the bottomed cards, so
    /// `to_bottom.last()` finishes bottom-most. Every id in either slice must
    /// already be in this zone; ids not named are left where they are, keeping
    /// their relative order.
    ///
    /// **Why this exists instead of `move_object_to_zone`.** Both of
    /// `GameState`'s move helpers mint a fresh `ObjectId` unconditionally
    /// (CR 400.7's "new object" rule), which is wrong for a card that never
    /// leaves its zone: scry-to-bottom used to renumber every scried card and
    /// consume `timestamp_counter` values (the shuffle/coin-flip seed source)
    /// doing it. Seed OOS-DP9-11 sweeps the other same-zone callers.
    ///
    /// No-op for unordered zones (they have no order to permute).
    pub fn reposition_within(&mut self, to_top: &[ObjectId], to_bottom: &[ObjectId]) {
        let Zone::Ordered(v) = self else {
            return;
        };
        let named: Vec<ObjectId> = to_top.iter().chain(to_bottom.iter()).copied().collect();
        // PB-DP9 fix-cycle Finding 9 (LOW): the "already in this zone"
        // precondition above is real -- an id that is NOT present is silently
        // INSERTED by the rebuild below, conjuring a phantom entry. The engine's
        // two callers cannot violate it (both partition a `Zone::top_n` list the
        // engine itself produced, and `validate_partition` re-checks the wire
        // answer against it), so this is an engine-bug assertion (SR-4), not a
        // runtime rejection.
        debug_assert!(
            named.iter().all(|id| v.contains(id)),
            "Zone::reposition_within: {:?} names ids not in this zone (have {:?})",
            named,
            v
        );
        // Everything not named keeps its position and relative order.
        let rest: imbl::Vector<ObjectId> =
            v.iter().copied().filter(|id| !named.contains(id)).collect();
        let mut out: imbl::Vector<ObjectId> = imbl::Vector::new();
        // Bottom end first (index 0 is the bottom). `to_bottom` is top-first, so
        // reverse it: its last entry must end up at index 0.
        for id in to_bottom.iter().rev() {
            out.push_back(*id);
        }
        out.append(rest);
        // Top end last (the final index is the top). `to_top` is top-first, so
        // reverse it: `to_top[0]` must end up last.
        for id in to_top.iter().rev() {
            out.push_back(*id);
        }
        *v = out;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn ordered(ids: &[u64]) -> Zone {
        Zone::Ordered(ids.iter().map(|&n| ObjectId(n)).collect())
    }

    #[test]
    /// CR 121.1: `top_n(1)` must agree with `top()` -- both identify the same
    /// card as "the top" of the library.
    fn test_top_n_agrees_with_top() {
        let z = ordered(&[1, 2, 3]);
        assert_eq!(z.top_n(1), z.top().into_iter().collect::<Vec<_>>());
    }

    #[test]
    /// Index 0 of `top_n`'s result is the topmost card; the vector is ordered
    /// top-down (CR 121.1).
    fn test_top_n_orders_top_first() {
        let z = ordered(&[1, 2, 3]);
        // Vector storage: [1, 2, 3] -- last element (3) is the top.
        assert_eq!(z.top_n(3), vec![ObjectId(3), ObjectId(2), ObjectId(1)]);
    }

    #[test]
    /// `n > len` must saturate to `len` entries, not panic or pad.
    fn test_top_n_over_length_saturates() {
        let z = ordered(&[1, 2]);
        assert_eq!(z.top_n(5), vec![ObjectId(2), ObjectId(1)]);
    }

    #[test]
    /// Unordered zones return empty, consistent with `top()` returning `None`.
    fn test_top_n_unordered_is_empty() {
        let z = Zone::Unordered(OrdSet::unit(ObjectId(1)));
        assert!(z.top_n(1).is_empty());
    }
}

/// The engine's pinned pseudo-random generator (PB-DX18, `OOS-DP2-4`).
///
/// SplitMix64: `state += GOLDEN; z = state; z ^= z >> 30; z *= C1; z ^= z >> 27;
/// z *= C2; z ^= z >> 31`. Chosen because it is a *fully specified* algorithm that fits
/// in this file — the point is not statistical quality (Fisher-Yates over a library needs
/// very little) but that the permutation cannot change underneath the engine when a
/// dependency is upgraded.
///
/// Deliberately NOT a `rand::RngCore` implementation: implementing that trait would let a
/// caller reach the permutation through `rand`'s own helpers again, which is the channel
/// this type exists to close.
#[derive(Clone, Debug)]
pub struct PinnedRng {
    state: u64,
}

impl PinnedRng {
    pub fn new(seed: u64) -> Self {
        PinnedRng { state: seed }
    }

    /// The next 64 bits of the stream.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform value in `0..bound`, by rejection sampling on a power-of-two mask.
    ///
    /// Rejection (rather than `% bound`) is what makes the draw *unbiased*; the mask
    /// keeps the expected number of rejections below one. `bound == 0` is a caller error
    /// and returns 0 rather than looping forever — the only caller passes `i + 1` with
    /// `i >= 1`.
    pub fn below(&mut self, bound: u64) -> u64 {
        if bound <= 1 {
            return 0;
        }
        let mask = u64::MAX >> (bound - 1).leading_zeros();
        loop {
            let v = self.next_u64() & mask;
            if v < bound {
                return v;
            }
        }
    }
}

#[cfg(test)]
mod pinned_rng_tests {
    use super::*;

    #[test]
    /// PB-DX18 (`OOS-DP2-4`) — the permutation is PINNED, not merely deterministic.
    ///
    /// A "deterministic given the same seed" test passes just as happily after a
    /// dependency bump silently re-permutes everything, which is exactly the failure
    /// `OOS-DP2-4`'s addendum describes. This fixes the actual output.
    fn shuffle_pinned_is_a_pinned_permutation() {
        let mut z = Zone::Ordered(Vector::from(
            (0..10u64).map(ObjectId).collect::<Vec<ObjectId>>(),
        ));
        z.shuffle_pinned(42);
        let got: Vec<u64> = z.object_ids().into_iter().map(|o| o.0).collect();
        assert_eq!(
            got,
            // OBSERVED by executing this algorithm, never transcribed from a prediction
            // (PB-DX27's rule). The first draft of this row guessed a value and the test
            // refuted it on its first run, which is the whole reason the row exists.
            vec![0, 8, 9, 1, 6, 7, 4, 2, 3, 5],
            "the shuffle for seed 42 over 0..10 is pinned; if this moved, EVERY seeded \
             fixture in the tree has been re-dealt and that must be a deliberate, \
             measured decision (OOS-DX21-6)"
        );
    }

    #[test]
    /// A shuffle is a PERMUTATION: same multiset, same length, every element once.
    fn shuffle_pinned_preserves_the_multiset() {
        for seed in 0..32u64 {
            let mut z = Zone::Ordered(Vector::from(
                (0..25u64).map(ObjectId).collect::<Vec<ObjectId>>(),
            ));
            z.shuffle_pinned(seed);
            let mut got: Vec<u64> = z.object_ids().into_iter().map(|o| o.0).collect();
            assert_eq!(got.len(), 25);
            got.sort_unstable();
            assert_eq!(got, (0..25u64).collect::<Vec<u64>>(), "seed {seed}");
        }
    }

    #[test]
    /// NON-VACUITY: different seeds really do give different permutations, and the
    /// shuffle is not the identity.
    fn shuffle_pinned_is_not_the_identity_and_varies_with_the_seed() {
        let identity: Vec<u64> = (0..25).collect();
        let mut seen = std::collections::HashSet::new();
        for seed in 0..16u64 {
            let mut z = Zone::Ordered(Vector::from(
                (0..25u64).map(ObjectId).collect::<Vec<ObjectId>>(),
            ));
            z.shuffle_pinned(seed);
            let got: Vec<u64> = z.object_ids().into_iter().map(|o| o.0).collect();
            assert_ne!(
                got, identity,
                "seed {seed} produced the identity permutation"
            );
            seen.insert(got);
        }
        assert_eq!(
            seen.len(),
            16,
            "16 seeds must give 16 distinct permutations"
        );
    }

    #[test]
    /// `below` is uniform enough that no value in range is unreachable, and never
    /// returns a value at or above the bound.
    fn below_stays_in_range_and_reaches_every_value() {
        let mut rng = PinnedRng::new(7);
        let mut seen = [false; 6];
        for _ in 0..2000 {
            let v = rng.below(6);
            assert!(v < 6);
            seen[v as usize] = true;
        }
        assert!(
            seen.iter().all(|s| *s),
            "every value in 0..6 must be reachable"
        );
    }
}
