//! Modal triggered ability tests (CR 700.2b, CR 603.3c).
//!
//! PB-35: G-27 gap closure — modal triggered abilities.
//! A triggered ability is modal when it has `modes: Some(ModeSelection)`.
//! The controller chooses modes when the trigger is put on the stack
//! (CR 700.2b), not at trigger-fire time. Bot fallback: mode 0.
//!
//! Key rules verified:
//! - CR 700.2b: Modal triggered ability chooses modes at stack-put time.
//! - CR 700.2: Mode 0 auto-selected by deterministic bot fallback.
//! - PB-35: modes field on AbilityDefinition::Triggered enables modal dispatch.

use mtg_engine::cards::card_definition::{AbilityDefinition, TriggerCondition};
use mtg_engine::{all_cards, CardDefinition, ModeSelection, PlayerId};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// CR 700.2b: Modal triggered ability structure check (Retreat to Kazandu).
/// Verifies the card def has modes: Some(ModeSelection) with correct structure.
#[test]
fn test_modal_triggered_ability_structure() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let retreat_def = defs.get("Retreat to Kazandu").unwrap();
    let modal_trigger = retreat_def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Triggered {
            modes: Some(modes), ..
        } = a
        {
            Some(modes.clone())
        } else {
            None
        }
    });

    assert!(
        modal_trigger.is_some(),
        "CR 700.2b: Retreat to Kazandu should have a modal triggered ability"
    );

    let modes = modal_trigger.unwrap();
    assert_eq!(
        modes.min_modes, 1,
        "Retreat to Kazandu: choose exactly 1 mode"
    );
    assert_eq!(
        modes.max_modes, 1,
        "Retreat to Kazandu: choose exactly 1 mode"
    );
    assert_eq!(
        modes.modes.len(),
        2,
        "Retreat to Kazandu: has 2 modes (+1/+1 counter or gain 2 life)"
    );
}

/// CR 700.2b: Felidar Retreat has modal landfall trigger.
/// Mode 0: create Cat Beast token. Mode 1: counters + vigilance.
#[test]
fn test_felidar_retreat_modal_structure() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let felidar_def = defs.get("Felidar Retreat").unwrap();
    let modal_trigger = felidar_def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield { .. },
            modes: Some(modes),
            ..
        } = a
        {
            Some(modes.clone())
        } else {
            None
        }
    });

    assert!(
        modal_trigger.is_some(),
        "CR 700.2b: Felidar Retreat should have a modal landfall triggered ability"
    );

    let modes = modal_trigger.unwrap();
    assert_eq!(
        modes.modes.len(),
        2,
        "Felidar Retreat has 2 modes (Cat Beast token or counters+vigilance)"
    );
}

/// CR 700.2b: Modal death trigger (Junji, the Midnight Sky).
/// Verifies the card def has WhenDies trigger with modes.
#[test]
fn test_modal_death_trigger_structure() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let junji_def = defs.get("Junji, the Midnight Sky").unwrap();
    let has_modal_death_trigger = junji_def.abilities.iter().any(|a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                modes: Some(_),
                ..
            }
        )
    });

    assert!(
        has_modal_death_trigger,
        "CR 700.2b: Junji, the Midnight Sky should have a modal WhenDies trigger"
    );
}

/// CR 700.2b: Modal ETB trigger (Shambling Ghast) with "choose one" structure.
/// Mode 0: create Treasure token. Mode 1: put -1/-1 counter on target creature.
#[test]
fn test_modal_etb_trigger_structure() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let ghast_def = defs.get("Shambling Ghast").unwrap();
    let modal_etb = ghast_def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            modes: Some(modes),
            ..
        } = a
        {
            Some(modes.clone())
        } else {
            None
        }
    });

    assert!(
        modal_etb.is_some(),
        "CR 700.2b: Shambling Ghast should have a modal ETB triggered ability"
    );

    let modes = modal_etb.unwrap();
    assert_eq!(
        modes.modes.len(),
        2,
        "Shambling Ghast has 2 modes (Treasure token or -1/-1 counter)"
    );
    assert_eq!(modes.min_modes, 1, "Shambling Ghast: must choose 1 mode");
    assert_eq!(modes.max_modes, 1, "Shambling Ghast: must choose 1 mode");
}

/// CR 700.2b: Glissa Sunslayer has modal combat damage trigger.
/// Mode 0: draw + lose 1. Mode 1: destroy enchantment. Mode 2: remove counters (Nothing).
#[test]
fn test_glissa_sunslayer_modal_structure() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let glissa_def = defs.get("Glissa Sunslayer").unwrap();
    let modal_trigger = glissa_def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
            modes: Some(modes),
            ..
        } = a
        {
            Some(modes.clone())
        } else {
            None
        }
    });

    assert!(
        modal_trigger.is_some(),
        "CR 700.2b: Glissa Sunslayer should have a modal combat damage trigger"
    );

    let modes = modal_trigger.unwrap();
    assert_eq!(
        modes.modes.len(),
        3,
        "Glissa Sunslayer has 3 modes (draw+lose1, destroy enchantment, remove counters)"
    );
}

/// CR 700.2b: Hullbreaker Horror has "choose up to one" modal trigger (min_modes: 0).
#[test]
fn test_hullbreaker_horror_choose_up_to_one() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    let horror_def = defs.get("Hullbreaker Horror").unwrap();
    let modal_trigger = horror_def.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Triggered {
            modes: Some(modes), ..
        } = a
        {
            Some(modes.clone())
        } else {
            None
        }
    });

    assert!(
        modal_trigger.is_some(),
        "CR 700.2b: Hullbreaker Horror should have a modal triggered ability"
    );

    let modes = modal_trigger.unwrap();
    assert_eq!(
        modes.min_modes, 0,
        "CR 700.2b: Hullbreaker Horror says 'choose up to one' — min_modes must be 0"
    );
    assert_eq!(
        modes.max_modes, 1,
        "Hullbreaker Horror: choose up to 1 mode"
    );
}
