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
                StackObjectKind::CascadeTrigger { source_object, .. } => {
                    ("Cascade: ".to_string(), Some(*source_object))
                }
                StackObjectKind::StormTrigger { source_object, .. } => {
                    ("Storm: ".to_string(), Some(*source_object))
                }
                StackObjectKind::EvokeSacrificeTrigger { source_object } => {
                    ("Evoke sac: ".to_string(), Some(*source_object))
                }
                StackObjectKind::MadnessTrigger { source_object, .. } => {
                    ("Madness: ".to_string(), Some(*source_object))
                }
                StackObjectKind::MiracleTrigger { source_object, .. } => {
                    ("Miracle: ".to_string(), Some(*source_object))
                }
                StackObjectKind::UnearthAbility { source_object } => {
                    ("Unearth: ".to_string(), Some(*source_object))
                }
                StackObjectKind::UnearthTrigger { source_object } => {
                    ("Unearth exile: ".to_string(), Some(*source_object))
                }
                StackObjectKind::ExploitTrigger { source_object } => {
                    ("Exploit: ".to_string(), Some(*source_object))
                }
                StackObjectKind::ModularTrigger { source_object, .. } => {
                    ("Modular: ".to_string(), Some(*source_object))
                }
                StackObjectKind::EvolveTrigger { source_object, .. } => {
                    ("Evolve: ".to_string(), Some(*source_object))
                }
                StackObjectKind::MyriadTrigger { source_object, .. } => {
                    ("Myriad: ".to_string(), Some(*source_object))
                }
                StackObjectKind::SuspendCounterTrigger { suspended_card, .. } => {
                    ("Suspend tick: ".to_string(), Some(*suspended_card))
                }
                StackObjectKind::SuspendCastTrigger { suspended_card, .. } => {
                    ("Suspend cast: ".to_string(), Some(*suspended_card))
                }
                StackObjectKind::HideawayTrigger { source_object, .. } => {
                    ("Hideaway: ".to_string(), Some(*source_object))
                }
                StackObjectKind::PartnerWithTrigger { source_object, .. } => {
                    ("Partner with: ".to_string(), Some(*source_object))
                }
                StackObjectKind::IngestTrigger { source_object, .. } => {
                    ("Ingest: ".to_string(), Some(*source_object))
                }
                StackObjectKind::FlankingTrigger { source_object, .. } => {
                    ("Flanking: ".to_string(), Some(*source_object))
                }
                StackObjectKind::RampageTrigger { source_object, .. } => {
                    ("Rampage: ".to_string(), Some(*source_object))
                }
                StackObjectKind::ProvokeTrigger { source_object, .. } => {
                    ("Provoke: ".to_string(), Some(*source_object))
                }
                StackObjectKind::RenownTrigger { source_object, .. } => {
                    ("Renown: ".to_string(), Some(*source_object))
                }
                StackObjectKind::MeleeTrigger { source_object, .. } => {
                    ("Melee: ".to_string(), Some(*source_object))
                }
                StackObjectKind::PoisonousTrigger { source_object, .. } => {
                    ("Poisonous: ".to_string(), Some(*source_object))
                }
                StackObjectKind::EnlistTrigger { source_object, .. } => {
                    ("Enlist: ".to_string(), Some(*source_object))
                }
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
                StackObjectKind::EncoreSacrificeTrigger { source_object, .. } => {
                    ("Encore sacrifice: ".to_string(), Some(*source_object))
                }
                StackObjectKind::DashReturnTrigger { source_object } => {
                    ("Dash return: ".to_string(), Some(*source_object))
                }
                StackObjectKind::BlitzSacrificeTrigger { source_object } => {
                    ("Blitz sacrifice: ".to_string(), Some(*source_object))
                }
                StackObjectKind::ImpendingCounterTrigger {
                    impending_permanent,
                    ..
                } => ("Impending tick: ".to_string(), Some(*impending_permanent)),
                StackObjectKind::CasualtyTrigger { source_object, .. } => {
                    ("Casualty: ".to_string(), Some(*source_object))
                }
                StackObjectKind::ReplicateTrigger { source_object, .. } => {
                    ("Replicate: ".to_string(), Some(*source_object))
                }
                StackObjectKind::GravestormTrigger { source_object, .. } => {
                    ("Gravestorm: ".to_string(), Some(*source_object))
                }
                StackObjectKind::VanishingCounterTrigger {
                    vanishing_permanent,
                    ..
                } => ("Vanishing tick: ".to_string(), Some(*vanishing_permanent)),
                StackObjectKind::VanishingSacrificeTrigger {
                    vanishing_permanent,
                    ..
                } => (
                    "Vanishing sacrifice: ".to_string(),
                    Some(*vanishing_permanent),
                ),
                StackObjectKind::FadingTrigger {
                    fading_permanent, ..
                } => ("Fading: ".to_string(), Some(*fading_permanent)),
                StackObjectKind::EchoTrigger { echo_permanent, .. } => {
                    ("Echo: ".to_string(), Some(*echo_permanent))
                }
                StackObjectKind::CumulativeUpkeepTrigger { cu_permanent, .. } => {
                    ("Cumulative Upkeep: ".to_string(), Some(*cu_permanent))
                }
                StackObjectKind::RecoverTrigger { recover_card, .. } => {
                    ("Recover: ".to_string(), Some(*recover_card))
                }
                StackObjectKind::ForecastAbility { source_object, .. } => {
                    ("Forecast: ".to_string(), Some(*source_object))
                }
                StackObjectKind::GraftTrigger { source_object, .. } => {
                    ("Graft: ".to_string(), Some(*source_object))
                }
                StackObjectKind::ScavengeAbility { .. } => {
                    // No source_object -- card was already exiled as cost (CR 702.97a).
                    ("Scavenge: ".to_string(), None)
                }
                StackObjectKind::BackupTrigger { source_object, .. } => {
                    ("Backup: ".to_string(), Some(*source_object))
                }
                StackObjectKind::ChampionETBTrigger { source_object, .. } => {
                    ("Champion ETB: ".to_string(), Some(*source_object))
                }
                StackObjectKind::ChampionLTBTrigger { source_object, .. } => {
                    ("Champion LTB: ".to_string(), Some(*source_object))
                }
                StackObjectKind::SoulbondTrigger { source_object, .. } => {
                    ("Soulbond: ".to_string(), Some(*source_object))
                }
                StackObjectKind::RavenousDrawTrigger {
                    ravenous_permanent, ..
                } => ("Ravenous draw: ".to_string(), Some(*ravenous_permanent)),
                StackObjectKind::BloodrushAbility { source_object, .. } => {
                    ("Bloodrush: ".to_string(), Some(*source_object))
                }
                StackObjectKind::SquadTrigger { source_object, .. } => {
                    ("Squad trigger: ".to_string(), Some(*source_object))
                }
                StackObjectKind::OffspringTrigger { source_object, .. } => {
                    ("Offspring trigger: ".to_string(), Some(*source_object))
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
