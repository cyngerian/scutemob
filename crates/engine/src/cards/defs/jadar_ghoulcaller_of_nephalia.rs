// Jadar, Ghoulcaller of Nephalia — {1}{B}, Legendary Creature — Human Wizard 1/1
// At the beginning of your end step, if you control no tokens named Shambling Ghast,
// create a 2/2 black Zombie creature token with decayed.
// (It can't block. When it attacks, sacrifice it at end of combat.)
//
// CR 702.147a: Decayed — can't block; sacrifice at end of combat after attacking.
// CR 603.1: Triggered ability fires at beginning of controller's end step with intervening-if.
// CR 111.10: Token is created with the Decayed keyword in its characteristics.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jadar-ghoulcaller-of-nephalia"),
        name: "Jadar, Ghoulcaller of Nephalia".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Wizard"],
        ),
        oracle_text: "At the beginning of your end step, if you control no tokens named \
Shambling Ghast, create a 2/2 black Zombie creature token with decayed. (It can't block. \
When it attacks, sacrifice it at end of combat.)"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.1: End step trigger with intervening-if (no Shambling Ghast tokens).
            // NOTE: The intervening-if condition (no Shambling Ghast tokens) cannot be expressed
            // in the current Condition DSL — no token-name filter exists yet.
            // Implemented as an unconditional end-step trigger (no intervening_if).
            // This is a known DSL gap: Condition::NoTokensNamedX does not exist.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::CreateToken { spec: zombie_decayed_token_spec(1) },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
    }
}
