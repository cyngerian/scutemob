// Najeela, the Blade-Blossom — {2}{R}, Legendary Creature — Human Warrior 3/2
// Whenever a Warrior attacks, you may have its controller create a 1/1 white Warrior
// creature token that's tapped and attacking.
// {W}{U}{B}{R}{G}: Untap all attacking creatures. They gain trample, lifelink, and haste
// until end of turn. After this phase, there is an additional combat phase. Activate only
// during combat.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("najeela-the-blade-blossom"),
        name: "Najeela, the Blade-Blossom".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Warrior"]),
        oracle_text: "Whenever a Warrior attacks, you may have its controller create a 1/1 white Warrior creature token that's tapped and attacking.\n{W}{U}{B}{R}{G}: Untap all attacking creatures. They gain trample, lifelink, and haste until end of turn. After this phase, there is an additional combat phase. Activate only during combat.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — "Whenever a Warrior attacks" requires a creature-type-filtered
            // attack trigger (WheneverCreatureWithSubtypeAttacks) that does not exist in the DSL.
            // TODO: DSL gap — the activated ability untaps all attacking creatures, grants
            // trample/lifelink/haste to all attacking creatures, and adds an additional combat
            // phase. ForEach over EachAttackingCreature for untap + multi-keyword grant exists,
            // but the activation requires Cost::Sequence with one of each color mana and
            // TimingRestriction::CombatOnly, which is not a supported timing restriction.
        ],
        ..Default::default()
    }
}
