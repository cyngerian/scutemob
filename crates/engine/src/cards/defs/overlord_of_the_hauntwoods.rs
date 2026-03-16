// Overlord of the Hauntwoods — {3}{G}{G}, Enchantment Creature — Avatar Horror 6/5
// Impending 4—{1}{G}{G}: enters with 4 time counters, not a creature until last removed.
// Whenever this permanent enters or attacks, create a tapped colorless Everywhere land token
// (all basic land types + all five mana abilities).
use crate::cards::helpers::*;

fn everywhere_token_spec() -> TokenSpec {
    // CR 111.10 / card ruling 2024-09-20: The Everywhere token has all basic land types
    // (Plains, Island, Swamp, Mountain, Forest) and each land type's mana ability.
    // It does NOT have the Basic supertype. It is colorless and enters tapped.
    TokenSpec {
        name: "Everywhere".to_string(),
        power: 0,
        toughness: 0,
        colors: OrdSet::new(),
        supertypes: OrdSet::new(),
        card_types: [CardType::Land].into_iter().collect(),
        subtypes: [
            SubType("Plains".to_string()),
            SubType("Island".to_string()),
            SubType("Swamp".to_string()),
            SubType("Mountain".to_string()),
            SubType("Forest".to_string()),
        ]
        .into_iter()
        .collect(),
        keywords: OrdSet::new(),
        count: 1,
        tapped: true,
        mana_color: None,
        mana_abilities: vec![
            ManaAbility::tap_for(ManaColor::White),
            ManaAbility::tap_for(ManaColor::Blue),
            ManaAbility::tap_for(ManaColor::Black),
            ManaAbility::tap_for(ManaColor::Red),
            ManaAbility::tap_for(ManaColor::Green),
        ],
        activated_abilities: vec![],
    }
}

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("overlord-of-the-hauntwoods"),
        name: "Overlord of the Hauntwoods".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Enchantment, CardType::Creature],
            &["Avatar", "Horror"],
        ),
        oracle_text: "Impending 4—{1}{G}{G} (If you cast this spell for its impending cost, \
it enters with four time counters and isn't a creature until the last is removed. At the \
beginning of your end step, remove a time counter from it.)\n\
Whenever this permanent enters or attacks, create a tapped colorless land token named \
Everywhere that is every basic land type."
            .to_string(),
        abilities: vec![
            // CR 702.176: Impending keyword for presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Impending),
            // CR 702.176: Full Impending definition — impending 4—{1}{G}{G}.
            AbilityDefinition::Impending {
                cost: ManaCost { generic: 1, green: 2, ..Default::default() },
                count: 4,
            },
            // "Whenever this permanent enters" — ETB trigger.
            // Note: The DSL has no combined "enters or attacks" trigger variant.
            // Modeled as two separate triggers (WhenEntersBattlefield + WhenAttacks),
            // both producing the same Everywhere token effect.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken { spec: everywhere_token_spec() },
                intervening_if: None,
                targets: vec![],
            },
            // "Whenever this permanent attacks" — attack trigger.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken { spec: everywhere_token_spec() },
                intervening_if: None,
                targets: vec![],
            },
        ],
        power: Some(6),
        toughness: Some(5),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
