// Greymond, Avacyn's Stalwart — {2}{W}{W}, Legendary Creature — Human Soldier 3/4
// As Greymond enters, choose two abilities from among first strike, vigilance, and lifelink.
// Humans you control have each of the chosen abilities.
// As long as you control four or more Humans, Humans you control get +2/+2.
//
// TODO: DSL gap — "As ... enters, choose from among" modal static grant is not expressible.
// No ChooseAbility ETB mechanism exists in the DSL.
// The +2/+2 static also requires a conditional static with a count threshold filter.
// Both abilities are omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("greymond-avacyns-stalwart"),
        name: "Greymond, Avacyn's Stalwart".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "As Greymond, Avacyn's Stalwart enters, choose two abilities from among \
                      first strike, vigilance, and lifelink.\nHumans you control have each of the \
                      chosen abilities.\nAs long as you control four or more Humans, Humans you \
                      control get +2/+2."
            .to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: ETB choose-two modal ability grant (no ChooseAbility ETB in DSL)
            // TODO: Conditional +2/+2 static requiring count_threshold filter
        ],
        completeness: Completeness::inert(
            "Blocked: 'As this enters, choose two abilities from among first strike, vigilance, \
             and lifelink' — no as-enters ability-choice replacement and no layer grant keyed to \
             a chosen ability set. The +2/+2 conditional static IS expressible \
             (Condition::YouControlNOrMoreWithFilter + ContinuousEffectDef.condition) and may be \
             wired — but read this first (PB-DX19, 2026-08-02): until that batch, following this \
             note would have built a SECOND instance of the OOS-SIM2-6 stack-overflow class. It \
             is safe to register now, at one cost: a registered static's condition is evaluated \
             from INSIDE the layer walk, where the filter test reads base characteristics \
             (rules::layers::characteristics_for_condition), so a Human created by another \
             continuous effect's type change is not counted. That is a property of the LAYER \
             PATH, not of the condition — the same condition used as an activation_condition or \
             intervening_if reads layer-resolved characteristics. See OOS-DX19-2 for the CR \
             613.8b-honest fixpoint.",
        ),
        ..Default::default()
    }
}
