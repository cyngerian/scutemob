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
