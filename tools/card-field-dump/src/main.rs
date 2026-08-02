//! Dump every `all_cards()` definition's *printed* fields as TSV on stdout.
//!
//! This is the corpus half of the CARDS-2 field-fidelity audit (`docs/engine-invariants.md`,
//! SR-37). It exists so the fixture-refresh script has an authoritative list of what the
//! corpus actually contains — enumerated from `all_cards()`, never grepped out of the def
//! sources (SR-36).
//!
//! Columns (tab-separated, one row per definition, sorted by name):
//!
//! ```text
//! name  completeness  mana_cost  power  toughness  supertypes  card_types  subtypes
//! ```
//!
//! `-` means "the field is `None`". `mana_cost` is rendered in a canonical, order-free
//! notation: `{X}`… then generic, then WUBRG, then `{C}`, then sorted hybrid and Phyrexian
//! pips. Type components are `|`-joined and sorted.
//!
//! **This renderer is diagnostic, not authoritative.** The gate that actually decides
//! whether a def matches its printed card is
//! `crates/engine/tests/core/cards2_printed_field_fidelity.rs`, which parses the committed
//! fixture's raw Scryfall strings and compares *structurally*. If the two ever disagree,
//! the gate is right and this file is wrong.
//!
//! Usage: `cargo run -p card-field-dump > /tmp/corpus.tsv`

use mtg_card_types::cards::card_definition::CardDefinition;
use mtg_card_types::state::game_object::{HybridMana, ManaCost, PhyrexianMana};
use mtg_card_types::state::ManaColor;

fn color_letter(c: &ManaColor) -> &'static str {
    use ManaColor as C;
    match c {
        C::White => "W",
        C::Blue => "U",
        C::Black => "B",
        C::Red => "R",
        C::Green => "G",
        C::Colorless => "C",
    }
}

/// Canonical, order-free rendering of a `ManaCost`.
fn render_cost(cost: &ManaCost) -> String {
    let mut out = String::new();
    for _ in 0..cost.x_count {
        out.push_str("{X}");
    }
    if cost.generic > 0 {
        out.push_str(&format!("{{{}}}", cost.generic));
    }
    for (n, sym) in [
        (cost.white, "W"),
        (cost.blue, "U"),
        (cost.black, "B"),
        (cost.red, "R"),
        (cost.green, "G"),
        (cost.colorless, "C"),
    ] {
        for _ in 0..n {
            out.push_str(&format!("{{{}}}", sym));
        }
    }
    let mut hybrids: Vec<String> = cost
        .hybrid
        .iter()
        .map(|h| match h {
            HybridMana::ColorColor(a, b) => format!("{{{}/{}}}", color_letter(a), color_letter(b)),
            HybridMana::GenericColor(a) => format!("{{2/{}}}", color_letter(a)),
        })
        .collect();
    hybrids.sort();
    let mut phyrexian: Vec<String> = cost
        .phyrexian
        .iter()
        .map(|p| match p {
            PhyrexianMana::Single(a) => format!("{{{}/P}}", color_letter(a)),
            PhyrexianMana::Hybrid(a, b) => {
                format!("{{{}/{}/P}}", color_letter(a), color_letter(b))
            }
        })
        .collect();
    phyrexian.sort();
    for s in hybrids.into_iter().chain(phyrexian) {
        out.push_str(&s);
    }
    if out.is_empty() {
        "{0}".to_string()
    } else {
        out
    }
}

fn render_types(def: &CardDefinition) -> (String, String, String) {
    let mut supers: Vec<String> = def
        .types
        .supertypes
        .iter()
        .map(|s| format!("{:?}", s))
        .collect();
    supers.sort();
    let mut cards: Vec<String> = def
        .types
        .card_types
        .iter()
        .map(|s| format!("{:?}", s))
        .collect();
    cards.sort();
    let mut subs: Vec<String> = def.types.subtypes.iter().map(|s| s.0.clone()).collect();
    subs.sort();
    (supers.join("|"), cards.join("|"), subs.join("|"))
}

fn main() {
    let mut defs = mtg_card_defs::all_cards();
    defs.sort_by(|a, b| a.name.cmp(&b.name));
    println!("name\tcompleteness\tmana_cost\tpower\ttoughness\tsupertypes\tcard_types\tsubtypes");
    for def in &defs {
        let cost = def
            .mana_cost
            .as_ref()
            .map(render_cost)
            .unwrap_or_else(|| "-".to_string());
        let power = def
            .power
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());
        let toughness = def
            .toughness
            .map(|t| t.to_string())
            .unwrap_or_else(|| "-".to_string());
        let (supers, cards, subs) = render_types(def);
        println!(
            "{}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}",
            def.name, def.completeness, cost, power, toughness, supers, cards, subs
        );
    }
}
