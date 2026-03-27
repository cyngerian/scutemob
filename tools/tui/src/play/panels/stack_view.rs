//! Stack view — shows items on the stack when non-empty.

use ratatui::prelude::*;
use ratatui::widgets::*;

use mtg_engine::StackObjectKind;

use crate::play::app::PlayApp;
use crate::play::panels::card_detail::card_color;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    if app.state.stack_objects.is_empty() {
        let empty = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<Line> = app
        .state
        .stack_objects
        .iter()
        .rev()
        .enumerate()
        .map(|(i, so)| {
            let (label, source_id) = match &so.kind {
                StackObjectKind::Spell { source_object } => ("".to_string(), Some(*source_object)),
                StackObjectKind::ActivatedAbility { source_object, .. } => {
                    ("Activated: ".to_string(), Some(*source_object))
                }
                StackObjectKind::TriggeredAbility { source_object, .. } => {
                    ("Triggered: ".to_string(), Some(*source_object))
                }
                // CascadeTrigger: migrated to KeywordTrigger
                // StormTrigger: migrated to KeywordTrigger
                // EvokeSacrificeTrigger: migrated to KeywordTrigger
                StackObjectKind::MadnessTrigger { source_object, .. } => {
                    ("Madness: ".to_string(), Some(*source_object))
                }
                StackObjectKind::MiracleTrigger { source_object, .. } => {
                    ("Miracle: ".to_string(), Some(*source_object))
                }
                StackObjectKind::UnearthAbility { source_object } => {
                    ("Unearth: ".to_string(), Some(*source_object))
                }
                // UnearthTrigger: migrated to KeywordTrigger
                // ExploitTrigger: migrated to KeywordTrigger
                // ModularTrigger: migrated to KeywordTrigger
                // EvolveTrigger: migrated to KeywordTrigger
                // MyriadTrigger: migrated to KeywordTrigger
                StackObjectKind::SuspendCounterTrigger { suspended_card, .. } => {
                    ("Suspend tick: ".to_string(), Some(*suspended_card))
                }
                StackObjectKind::SuspendCastTrigger { suspended_card, .. } => {
                    ("Suspend cast: ".to_string(), Some(*suspended_card))
                }
                // HideawayTrigger: migrated to KeywordTrigger
                // PartnerWithTrigger: migrated to KeywordTrigger
                // IngestTrigger: migrated to KeywordTrigger
                // FlankingTrigger, RampageTrigger, ProvokeTrigger, RenownTrigger,
                // MeleeTrigger, PoisonousTrigger, EnlistTrigger: migrated to KeywordTrigger
                StackObjectKind::NinjutsuAbility { source_object, .. } => {
                    ("Ninjutsu: ".to_string(), Some(*source_object))
                }
                StackObjectKind::EmbalmAbility { .. } => {
                    // No source_object -- card was already exiled as cost (CR 702.128a).
                    ("Embalm: ".to_string(), None)
                }
                StackObjectKind::EternalizeAbility { source_name, .. } => {
                    // No source_object -- card was already exiled as cost (CR 702.129a).
                    (format!("Eternalize: {}", source_name), None)
                }
                StackObjectKind::EncoreAbility { .. } => {
                    // No source_object -- card was already exiled as cost (CR 702.141a).
                    ("Encore: ".to_string(), None)
                }
                // EncoreSacrificeTrigger: migrated to KeywordTrigger
                // DashReturnTrigger: migrated to KeywordTrigger
                // BlitzSacrificeTrigger: migrated to KeywordTrigger
                // ImpendingCounterTrigger: migrated to KeywordTrigger
                // CasualtyTrigger: migrated to KeywordTrigger
                // ReplicateTrigger: migrated to KeywordTrigger
                // GravestormTrigger: migrated to KeywordTrigger
                // VanishingCounterTrigger and VanishingSacrificeTrigger: migrated to KeywordTrigger
                // FadingTrigger: migrated to KeywordTrigger
                // EchoTrigger: migrated to KeywordTrigger
                // CumulativeUpkeepTrigger: migrated to KeywordTrigger
                // RecoverTrigger: migrated to KeywordTrigger
                StackObjectKind::ForecastAbility { source_object, .. } => {
                    ("Forecast: ".to_string(), Some(*source_object))
                }
                // GraftTrigger: migrated to KeywordTrigger
                StackObjectKind::ScavengeAbility { .. } => {
                    // No source_object -- card was already exiled as cost (CR 702.97a).
                    ("Scavenge: ".to_string(), None)
                }
                // BackupTrigger: migrated to KeywordTrigger
                // ChampionETBTrigger: migrated to KeywordTrigger
                // ChampionLTBTrigger: migrated to KeywordTrigger
                // SoulbondTrigger: migrated to KeywordTrigger
                // RavenousDrawTrigger: migrated to KeywordTrigger
                StackObjectKind::BloodrushAbility { source_object, .. } => {
                    ("Bloodrush: ".to_string(), Some(*source_object))
                }
                // SquadTrigger: migrated to KeywordTrigger
                // OffspringTrigger: migrated to KeywordTrigger
                // GiftETBTrigger: migrated to KeywordTrigger
                StackObjectKind::SaddleAbility { source_object } => {
                    ("Saddle: ".to_string(), Some(*source_object))
                }
                // CipherTrigger: migrated to KeywordTrigger
                // HauntExileTrigger: migrated to KeywordTrigger
                // HauntedCreatureDiesTrigger: migrated to KeywordTrigger
                StackObjectKind::MutatingCreatureSpell { source_object, .. } => {
                    ("Mutating: ".to_string(), Some(*source_object))
                }
                StackObjectKind::TransformTrigger { permanent, .. } => {
                    ("Transform trigger: ".to_string(), Some(*permanent))
                }
                StackObjectKind::CraftAbility { exiled_source, .. } => {
                    ("Craft: ".to_string(), Some(*exiled_source))
                }
                StackObjectKind::DayboundTransformTrigger { permanent } => {
                    ("Daybound/Nightbound: ".to_string(), Some(*permanent))
                }
                StackObjectKind::TurnFaceUpTrigger { permanent, .. } => {
                    ("Turned Face Up: ".to_string(), Some(*permanent))
                }
                StackObjectKind::KeywordTrigger {
                    source_object,
                    keyword,
                    data,
                } => {
                    let permanent = match data {
                        mtg_engine::state::stack::TriggerData::CounterRemoval { permanent }
                        | mtg_engine::state::stack::TriggerData::CounterSacrifice { permanent }
                        | mtg_engine::state::stack::TriggerData::UpkeepCost { permanent, .. } => {
                            Some(*permanent)
                        }
                        _ => Some(*source_object),
                    };
                    (format!("{:?} trigger: ", keyword), permanent)
                }
                // CR 309.4c: Room ability — no source_object (dungeon is in command zone).
                StackObjectKind::RoomAbility { dungeon, room, .. } => {
                    (format!("Room {:?}[{}]: ", dungeon, room), None)
                }
                // CR 701.54c: Ring-bearer triggered ability.
                StackObjectKind::RingAbility { source_object, .. } => {
                    ("Ring: ".to_string(), Some(*source_object))
                }
                StackObjectKind::LoyaltyAbility { source_object, .. } => {
                    ("Loyalty: ".to_string(), Some(*source_object))
                }
                // CR 716.2a: Class level-up activated ability.
                StackObjectKind::ClassLevelAbility {
                    source_object,
                    target_level,
                } => (
                    format!("Level Up (→{}): ", target_level),
                    Some(*source_object),
                ),
                // CR 603.7: Delayed trigger fires — return/sacrifice/exile action.
                StackObjectKind::DelayedActionTrigger { source_object, .. } => {
                    ("Delayed trigger: ".to_string(), Some(*source_object))
                }
            };

            let (name, name_color) = source_id
                .and_then(|id| app.state.object(id).ok())
                .map(|obj| {
                    (
                        obj.characteristics.name.clone(),
                        card_color(&obj.characteristics),
                    )
                })
                .unwrap_or_else(|| ("???".to_string(), Color::Gray));

            Line::from(vec![
                Span::styled(
                    format!("[{}] {}", i + 1, label),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    name,
                    Style::default().fg(name_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" (P{})", so.controller.0),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect();

    let stack = Paragraph::new(items)
        .block(
            Block::default()
                .title(" Stack ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(stack, area);
}
