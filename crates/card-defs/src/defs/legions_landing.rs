// Legion's Landing // Adanto, the First Fort — {W} DFC Legendary Enchantment //
// Legendary Land (Transform)
// Front: When Legion's Landing enters the battlefield, create a 1/1 white Vampire
//        creature token with lifelink.
//        Whenever you attack with three or more creatures, transform Legion's Landing.
// Back:  Adanto, the First Fort — {T}: Add {W}.
//        {1}{W}, {T}: Create a 1/1 white Vampire creature token with lifelink.
//
// PB-OS6(b): the attack-count gate uses Condition::YouAttackedWithNOrMore(3) inside
// Effect::Conditional, not intervening_if -- build_face_ability_vectors hardcodes
// intervening_if: None for WheneverYouAttack lowering (see pb-plan-OS6.md), so any
// DSL intervening_if on this trigger would silently be dropped. Self-gating inside
// the effect is the proven pattern (mirrors delver_of_secrets.rs / heralds_horn.rs).
use crate::cards::helpers::*;

fn white_vampire_lifelink_token() -> TokenSpec {
    TokenSpec {
        name: "Vampire".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Vampire".to_string())].into_iter().collect(),
        colors: [Color::White].into_iter().collect(),
        supertypes: imbl::OrdSet::new(),
        power: 1,
        toughness: 1,
        count: EffectAmount::Fixed(1),
        keywords: [KeywordAbility::Lifelink].into_iter().collect(),
        tapped: false,
        enters_attacking: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
        ..Default::default()
    }
}

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("legions-landing-adanto-the-first-fort"),
        name: "Legion's Landing".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Enchantment]),
        oracle_text: "When Legion's Landing enters the battlefield, create a 1/1 white Vampire \
                      creature token with lifelink.\nWhenever you attack with three or more \
                      creatures, transform Legion's Landing."
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // CR 603.6/614: "When Legion's Landing enters the battlefield, create a 1/1
            // white Vampire creature token with lifelink."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: white_vampire_lifelink_token(),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 508.1/508.4 (PB-OS6(b)): "Whenever you attack with three or more
            // creatures, transform Legion's Landing." Fires unconditionally on any
            // attack; the count gate self-evaluates inside the effect via
            // Condition::YouAttackedWithNOrMore(3), reading the captured
            // PlayerState.attackers_declared_this_turn. Transforms even if some of
            // those creatures later leave combat (ruling 2017-09-29).
            //
            // PB-DX21 review (finding M3): this card is NOT a member of
            // `OOS-DX21-1` (the windbrisk_heights.rs residual) -- do not migrate
            // it there. This is a CR 508.3d "Whenever [a player] attacks" trigger:
            // it fires PER DECLARATION and its count reads the SAME declaration
            // that fired it (ruling 2017-09-29: "The last ability of Legion's
            // Landing only counts creatures that you declare as attacking
            // creatures" -- a per-declaration, not per-turn, statement; the
            // second ruling's "once you've attacked with three or more creatures"
            // is about creatures leaving combat AFTER that one declaration, not
            // about accumulating count across separate declarations). CR 508.6
            // ("has attacked [a player]") is a boolean per-player predicate with
            // no count or turn-scope content and does not apply to this trigger's
            // gate at all -- an earlier draft of this comment cited it in error.
            // So attacking with 3 in combat 1 (transforms) and then 1 in combat 2
            // (does not re-transform, already transformed; a hypothetical second
            // copy would correctly evaluate false on 1 < 3) is the ENGINE'S
            // CORRECT behaviour for this card's own trigger, not a residual to
            // close. Windbrisk Heights' activation condition, by contrast, is
            // genuinely turn-scoped ("if you attacked with three or more
            // creatures this turn", ruling 2007-10-01, "at any point in the
            // turn") -- that is the sole member of `OOS-DX21-1`. Comment only; no
            // completeness change.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouAttack { filter: None },
                effect: Effect::Conditional {
                    condition: Condition::YouAttackedWithNOrMore(3),
                    if_true: Box::new(Effect::TransformSelf),
                    if_false: Box::new(Effect::Nothing),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Adanto, the First Fort".to_string(),
            mana_cost: None,
            types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
            oracle_text: "{T}: Add {W}.\n{1}{W}, {T}: Create a 1/1 white Vampire creature token \
                          with lifelink."
                .to_string(),
            power: None,
            toughness: None,
            abilities: vec![
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(1, 0, 0, 0, 0, 0),
                    },
                    timing_restriction: None,
                    targets: vec![],
                    activation_condition: None,
                    activation_zone: None,
                    once_per_turn: false,
                    modes: None,
                },
                AbilityDefinition::Activated {
                    cost: Cost::Sequence(vec![
                        Cost::Mana(ManaCost {
                            generic: 1,
                            white: 1,
                            ..Default::default()
                        }),
                        Cost::Tap,
                    ]),
                    effect: Effect::CreateToken {
                        spec: white_vampire_lifelink_token(),
                    },
                    timing_restriction: None,
                    targets: vec![],
                    activation_condition: None,
                    activation_zone: None,
                    once_per_turn: false,
                    modes: None,
                },
            ],
            color_indicator: None,
        }),
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
        completeness: Completeness::Complete,
    }
}
