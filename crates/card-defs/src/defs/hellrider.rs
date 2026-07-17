// Hellrider — {2}{R}{R}, Creature — Devil 3/3
// Haste
// Whenever a creature you control attacks, Hellrider deals 1 damage to the player or
// planeswalker it's attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hellrider"),
        name: "Hellrider".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Devil"]),
        oracle_text: "Haste\nWhenever a creature you control attacks, Hellrider deals 1 damage to \
                      the player or planeswalker it's attacking."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 508.1m / CR 603.2: "Whenever a creature you control attacks, Hellrider deals 1
            // damage to the player or planeswalker it's attacking."
            // WheneverCreatureYouControlAttacks is supported (PB-N).
            // TODO: DSL gap — the damage target is "the player or planeswalker IT'S attacking"
            // (the combat attack assignment of the triggering creature). No PlayerTarget or
            // EffectTarget variant resolves to the specific attack target of the triggering
            // creature. Implementing with EachOpponent or Controller would produce wrong game
            // state in multiplayer (wrong target, extra damage). Leaving abilities empty per
            // W5 policy until AttackTargetOf(EffectTarget::TriggeringCreature) or equivalent
            // is added to PlayerTarget.
        ],
        completeness: Completeness::partial(
            "DSL gap — the damage target is 'the player or planeswalker IT'S attacking' (the \
             combat attack assignment of the...",
        ),
        ..Default::default()
    }
}
