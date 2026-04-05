// Seasoned Dungeoneer — {3}{W}, Creature — Human Warrior 3/4
// When Seasoned Dungeoneer enters the battlefield, you take the initiative.
// Whenever you attack, target attacking Cleric, Rogue, Warrior, or Wizard gains
// protection from creatures until end of turn. It explores.
//
// CR 725.2: "take the initiative" — sets has_initiative, ventures into Undercity.
// CR 701.12a: Explore — look at top card, put land into hand or put +1/+1 counter.
//
// DSL gaps:
// - "whenever you attack" (fires when controller declares any attacker) has no TriggerCondition
//   variant — WhenAttacks fires when THIS creature attacks, which is used as a close
//   approximation (fires when Seasoned Dungeoneer itself attacks).
// - "target attacking Cleric, Rogue, Warrior, or Wizard gains protection from creatures
//   until end of turn" — targeting by subtype not available in TriggerCondition.
//   Additionally, Effect::GrantProtection targeting a named subtype doesn't exist in DSL.
// - "It explores" — Effect::Explore not yet in DSL.
// Full attack trigger deferred; ETB Take the Initiative is fully represented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("seasoned-dungeoneer"),
        name: "Seasoned Dungeoneer".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Warrior"]),
        oracle_text: "When Seasoned Dungeoneer enters the battlefield, you take the initiative.\nWhenever you attack, target attacking Cleric, Rogue, Warrior, or Wizard gains protection from creatures until end of turn. It explores."
            .to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // CR 725.2: ETB trigger — take the initiative.
            // Taking the initiative sets has_initiative = Some(controller) and immediately
            // ventures into the Undercity (CR 725.2).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::TakeTheInitiative,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Whenever you attack, venture into the dungeon.
            // TODO: Complex effect (protection + explore on targeted creature) deferred.
            // Using WheneverYouAttack + VentureIntoDungeon as partial approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouAttack,
                effect: Effect::VentureIntoDungeon,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
    }
}
