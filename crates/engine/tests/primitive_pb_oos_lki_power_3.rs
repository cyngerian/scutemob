//! Tests for PB-OOS-LKI-Power-3: hash `pre_lba_counters` + `pre_lba_power` on four
//! `GameEvent` LBA variants (CR 603.10a).
//!
//! Prior to this batch, `GameEvent::AuraFellOff`, `GameEvent::ObjectExiled`,
//! `GameEvent::PermanentDestroyed`, and `GameEvent::ObjectReturnedToHand` all used `..`
//! in their `HashInto` arms, silently discarding the LKI payloads added by PB-LKI-CC
//! (`pre_lba_counters`) and PB-LKI-Power (`pre_lba_power`). Only `GameEvent::CreatureDied`
//! hashed its symmetric `pre_death_counters` / `pre_death_power` fields.
//!
//! This batch makes the four sibling arms symmetric, closing OOS-LKI-Power-3
//! (pb-retriage-CC.md:621) and bumping HASH_SCHEMA_VERSION 23 → 24.
//!
//! CR 603.10a: "Some triggered abilities look back in time. When such a triggered ability
//!   resolves, it checks characteristics of objects that have since left the battlefield or
//!   another zone."
//! CR 113.7a: "An object's last known information is used only for determining characteristics
//!   of that object if it is no longer in the zone it was in when the triggered ability
//!   resolved."
//!
//! Sub-tests:
//!   α: `GameEvent::AuraFellOff` — three values differing in `pre_lba_power` (None, Some(2),
//!      Some(5)) produce three pairwise-distinct hashes. Also verifies Option tag-byte encoding:
//!      Some(0) ≠ None.
//!   β: `GameEvent::PermanentDestroyed` — same fixture as α (covers ≥2 of the 4 variants per
//!      AC #3940).
//!   γ: `GameEvent::ObjectExiled` — control over `pre_lba_counters` axis: empty OrdMap vs
//!      OrdMap with one (PlusOnePlusOne, 3) entry produces distinct hashes. Proves the counter
//!      axis is also closed by this PB.
//!   δ: `HASH_SCHEMA_VERSION == 24u8` sentinel (collocated in this file so future hash bumps
//!      force a fail here as well as in the sweep files).

use blake3::Hasher;
use im::OrdMap;
use mtg_engine::state::hash::HashInto;
use mtg_engine::state::types::CounterType;
use mtg_engine::{GameEvent, ObjectId, PlayerId, HASH_SCHEMA_VERSION};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Compute the Blake3 hash of a `GameEvent` using the engine's `HashInto` impl.
fn hash_event(ev: &GameEvent) -> [u8; 32] {
    let mut h = Hasher::new();
    ev.hash_into(&mut h);
    *h.finalize().as_bytes()
}

/// Placeholder object IDs used across sub-tests.
const OBJ_A: ObjectId = ObjectId(100);
const OBJ_B: ObjectId = ObjectId(101);
const P1: PlayerId = PlayerId(1);

// ── single combined test ──────────────────────────────────────────────────────

/// CR 603.10a, CR 113.7a — mechanical safety net for PB-OOS-LKI-Power-3.
///
/// Verifies that all four `GameEvent` LBA variants fold `pre_lba_counters` and
/// `pre_lba_power` into the state hash after HASH 23 → 24.
#[test]
fn test_pb_oos_lki_power_3_lba_variants_hash_pre_lba_fields() {
    // ── δ: HASH_SCHEMA_VERSION sentinel ──────────────────────────────────────
    //
    // Collocated here so any future HASH bump fails in this dedicated file in
    // addition to the per-PB sweep files. If this assertion fails, update this
    // file's sentinel AND add a new history entry in state/hash.rs.
    assert_eq!(
        HASH_SCHEMA_VERSION, 32u8,
        "BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (GameEvent::CreatureDied.pre_death_characteristics: Option<Characteristics>, CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers). If you bumped again, update this test and state/hash.rs history."
    );

    // ── α: GameEvent::AuraFellOff — pre_lba_power axis + Option tag-byte ─────
    //
    // CR 603.10a: a SelfLeavesBattlefield trigger on an Aura uses LKI power
    // from just before the Aura left the battlefield.
    // Four cases: None, Some(0), Some(2), Some(5).
    // None vs Some(0) discriminates the Option tag-byte (tag 0 = None, tag 1 = Some).
    // Some(2) vs Some(5) discriminates distinct payload values.

    let aura_none = GameEvent::AuraFellOff {
        object_id: OBJ_A,
        new_grave_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: None,
    };
    let aura_some_zero = GameEvent::AuraFellOff {
        object_id: OBJ_A,
        new_grave_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: Some(0),
    };
    let aura_some_two = GameEvent::AuraFellOff {
        object_id: OBJ_A,
        new_grave_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: Some(2),
    };
    let aura_some_five = GameEvent::AuraFellOff {
        object_id: OBJ_A,
        new_grave_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: Some(5),
    };

    // Option tag-byte: None ≠ Some(0).
    assert_ne!(
        hash_event(&aura_none),
        hash_event(&aura_some_zero),
        "AuraFellOff: pre_lba_power None vs Some(0) must produce distinct hashes \
         (Option tag-byte encoding — 0=None, 1=Some). Failure means pre_lba_power \
         is not hashed or the tag byte is missing."
    );
    // Value discrimination: Some(2) ≠ Some(5).
    assert_ne!(
        hash_event(&aura_some_two),
        hash_event(&aura_some_five),
        "AuraFellOff: pre_lba_power Some(2) vs Some(5) must produce distinct hashes \
         (payload value discrimination)."
    );
    // Three-way: None ≠ Some(2).
    assert_ne!(
        hash_event(&aura_none),
        hash_event(&aura_some_two),
        "AuraFellOff: pre_lba_power None vs Some(2) must produce distinct hashes."
    );

    // ── β: GameEvent::PermanentDestroyed — same pre_lba_power axis ───────────
    //
    // CR 603.10a: a SelfLeavesBattlefield trigger on a non-creature permanent
    // (e.g. enchantment destroyed by a spell) uses LKI power if the permanent
    // had a power (Layer 4 animation edge cases). Same Option tag-byte canary.

    let perm_none = GameEvent::PermanentDestroyed {
        object_id: OBJ_A,
        new_grave_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: None,
    };
    let perm_some_zero = GameEvent::PermanentDestroyed {
        object_id: OBJ_A,
        new_grave_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: Some(0),
    };
    let perm_some_four = GameEvent::PermanentDestroyed {
        object_id: OBJ_A,
        new_grave_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: Some(4),
    };

    assert_ne!(
        hash_event(&perm_none),
        hash_event(&perm_some_zero),
        "PermanentDestroyed: pre_lba_power None vs Some(0) must produce distinct hashes \
         (Option tag-byte encoding). Failure means pre_lba_power is not hashed."
    );
    assert_ne!(
        hash_event(&perm_some_zero),
        hash_event(&perm_some_four),
        "PermanentDestroyed: pre_lba_power Some(0) vs Some(4) must produce distinct hashes."
    );
    assert_ne!(
        hash_event(&perm_none),
        hash_event(&perm_some_four),
        "PermanentDestroyed: pre_lba_power None vs Some(4) must produce distinct hashes."
    );

    // ── γ: GameEvent::ObjectExiled — pre_lba_counters axis ───────────────────
    //
    // CR 603.10a / CR 122.2: counters cease to exist when a permanent leaves
    // the battlefield (CR 122.2), but LKI snapshots capture them before the
    // zone change. Two events that differ only in pre_lba_counters must hash
    // differently, proving the counter loop in the ObjectExiled arm is active.

    let mut counters_with_pp1 = OrdMap::new();
    counters_with_pp1.insert(CounterType::PlusOnePlusOne, 3u32);

    let exiled_no_counters = GameEvent::ObjectExiled {
        player: P1,
        object_id: OBJ_A,
        new_exile_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: None,
    };
    let exiled_with_pp1_counters = GameEvent::ObjectExiled {
        player: P1,
        object_id: OBJ_A,
        new_exile_id: OBJ_B,
        pre_lba_counters: counters_with_pp1,
        pre_lba_power: None,
    };

    assert_ne!(
        hash_event(&exiled_no_counters),
        hash_event(&exiled_with_pp1_counters),
        "ObjectExiled: pre_lba_counters empty OrdMap vs OrdMap{{PlusOnePlusOne: 3}} must \
         produce distinct hashes. Failure means pre_lba_counters is not hashed in the \
         ObjectExiled arm (counter axis of OOS-LKI-Power-3 not closed)."
    );

    // Bonus: verify that ObjectReturnedToHand also participates (the fourth variant).
    // Two events differing only in pre_lba_power must hash differently.
    let returned_no_power = GameEvent::ObjectReturnedToHand {
        player: P1,
        object_id: OBJ_A,
        new_hand_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: None,
    };
    let returned_with_power = GameEvent::ObjectReturnedToHand {
        player: P1,
        object_id: OBJ_A,
        new_hand_id: OBJ_B,
        pre_lba_counters: OrdMap::new(),
        pre_lba_power: Some(3),
    };

    assert_ne!(
        hash_event(&returned_no_power),
        hash_event(&returned_with_power),
        "ObjectReturnedToHand: pre_lba_power None vs Some(3) must produce distinct hashes. \
         Failure means pre_lba_power is not hashed in the ObjectReturnedToHand arm."
    );
}
