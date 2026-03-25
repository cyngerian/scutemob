// Battle Squadron — {3}{R}{R}, Creature — Goblin 2/2 (*/* characteristic-defining ability)
// Flying; power and toughness each equal to the number of creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("battle-squadron"),
        name: "Battle Squadron".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Flying\nBattle Squadron's power and toughness are each equal to the number of creatures you control.".to_string(),
        power: None,   // */* CDA — engine SBA skips None toughness; actual P/T set by layer
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 604.3, 613.4a: CDA — P/T each equal to the number of creatures you control.
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::PermanentCount {
                    filter: TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
                    controller: PlayerTarget::Controller,
                },
                toughness: EffectAmount::PermanentCount {
                    filter: TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
                    controller: PlayerTarget::Controller,
                },
            },
        ],
        ..Default::default()
    }
}
