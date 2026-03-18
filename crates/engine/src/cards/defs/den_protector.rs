// Den Protector — {1}{G}, Creature — Human Warrior 2/1
// Creatures with power less than this creature's power can't block it.
// Megamorph {1}{G}
// When this creature is turned face up, return target card from your graveyard to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("den-protector"),
        name: "Den Protector".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Human", "Warrior"]),
        oracle_text:
            "Creatures with power less than this creature's power can't block it.\n\
             Megamorph {1}{G} (You may cast this card face down as a 2/2 creature for {3}. \
             Turn it face up any time for its megamorph cost and put a +1/+1 counter on it.)\n\
             When this creature is turned face up, return target card from your graveyard to your hand."
                .to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // TODO: Static evasion — "Creatures with power less than this creature's power
            // can't block it." Needs Modification::CantBlockIfPowerLessThan or similar.
            AbilityDefinition::Keyword(KeywordAbility::Megamorph),
            AbilityDefinition::Megamorph { cost: ManaCost { generic: 1, green: 1, ..Default::default() } },
            // CR 603.1: When turned face up, return target card from your GY to hand.
            // Note: "target card" — no type restriction (any card in your GY).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenTurnedFaceUp,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter::default())],
            },
        ],
        ..Default::default()
    }
}
