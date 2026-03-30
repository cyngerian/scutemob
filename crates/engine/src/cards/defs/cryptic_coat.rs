// Cryptic Coat — {2}{U}, Artifact — Equipment
// When this Equipment enters, cloak the top card of your library, then attach this
// Equipment to it. (To cloak a card, put it onto the battlefield face down as a 2/2
// creature with ward {2}. Turn it face up any time for its mana cost if it's a
// creature card.)
// Equipped creature gets +1/+0 and can't be blocked.
// {1}{U}: Return this Equipment to its owner's hand.
//
// All abilities fully represented: ETB Cloak + auto-attach, static grants (+1/+0,
// can't be blocked) via AttachedCreature filter, and bounce-self activated ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cryptic-coat"),
        name: "Cryptic Coat".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "When this Equipment enters, cloak the top card of your library, \
then attach this Equipment to it. (To cloak a card, put it onto the battlefield face \
down as a 2/2 creature with ward {2}. Turn it face up any time for its mana cost if \
it's a creature card.)\nEquipped creature gets +1/+0 and can't be blocked.\n\
{1}{U}: Return this Equipment to its owner's hand."
            .to_string(),
        abilities: vec![
            // CR 701.58a: ETB trigger — cloak the top card, then attach this Equipment to it.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::Cloak { player: PlayerTarget::Controller },
                    Effect::AttachEquipment {
                        equipment: EffectTarget::Source,
                        target: EffectTarget::LastCreatedPermanent,
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 611.3a / Layer 7c: "Equipped creature gets +1/+0."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 509.1 / Layer 6: "Equipped creature can't be blocked."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::CantBeBlocked),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // {1}{U}: Return this Equipment to its owner's hand.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, blue: 1, ..Default::default() }),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::Source)),
                    },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
