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
            // can't block it." No ContinuousEffectDef Modification variant for power-based
            // blocking restriction exists. Needs a new Modification::CantBlockIfPowerLessThan
            // (or similar) in continuous_effect.rs before this can be expressed.
            AbilityDefinition::Keyword(KeywordAbility::Megamorph),
            AbilityDefinition::Megamorph { cost: ManaCost { generic: 1, green: 1, ..Default::default() } },
            // TODO: Triggered ability — "When this creature is turned face up, return target
            // card from your graveyard to your hand." TriggerCondition::WhenTurnedFaceUp
            // exists, but there is no TargetRequirement variant for targeting a card in a
            // specific zone (graveyard), and no Effect variant for moving a chosen graveyard
            // card to hand. Needs TargetRequirement::TargetCardInGraveyard and
            // Effect::ReturnFromGraveyard (or Effect::MoveZone wired to graveyard targets)
            // before this trigger can be expressed. Tracked as W5 DSL gap: return_from_graveyard.
        ],
        ..Default::default()
    }
}
