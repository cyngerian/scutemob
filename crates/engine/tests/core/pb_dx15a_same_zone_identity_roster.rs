//! PB-DX15a (CR 400.7): the corpus roster gate for the **same-zone move** class — every
//! `Complete`, deck-legal card definition whose printed effect can ask the engine to move a
//! card into the zone it is already in.
//!
//! MEASUREMENT SCAFFOLD — populations are printed, not yet pinned.

use mtg_engine::{all_cards, CardDefinition, Completeness};
use serde_json::Value;
use std::collections::BTreeSet;

fn complete_defs() -> Vec<CardDefinition> {
    all_cards()
        .into_iter()
        .filter(|d| d.completeness == Completeness::Complete)
        .collect()
}

fn variant_key(v: &Value) -> Option<(&str, &Value)> {
    match v {
        Value::Object(m) if m.len() == 1 => {
            let (k, val) = m.iter().next().unwrap();
            Some((k.as_str(), val))
        }
        _ => None,
    }
}

fn is_library_zone_target(v: &Value) -> bool {
    matches!(variant_key(v), Some(("Library", payload)) if payload.get("position").is_some())
}

fn walk<F: FnMut(&Value)>(v: &Value, f: &mut F) {
    f(v);
    match v {
        Value::Object(m) => {
            for c in m.values() {
                walk(c, f);
            }
        }
        Value::Array(items) => {
            for c in items {
                walk(c, f);
            }
        }
        _ => {}
    }
}

fn population(pred: impl Fn(&Value) -> bool) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for def in complete_defs() {
        let json = serde_json::to_value(&def).expect("serializes");
        let mut hit = false;
        walk(&json, &mut |node| {
            if pred(node) {
                hit = true;
            }
        });
        if hit {
            out.insert(def.name.clone());
        }
    }
    out
}

#[test]
fn measure() {
    let fam_a = population(|v| match variant_key(v) {
        Some(("RevealAndRoute", p)) => p.get("unmatched_dest").map(is_library_zone_target) == Some(true),
        Some(("LookAtTopThenPlace", p)) => p.get("rest_to").map(is_library_zone_target) == Some(true),
        _ => false,
    });
    let fam_b = population(|v| match variant_key(v) {
        Some(("SearchLibrary", p)) => p.get("destination").map(is_library_zone_target) == Some(true),
        _ => false,
    });
    let fam_c = population(|v| matches!(variant_key(v), Some(("Hideaway", _))));
    let fam_d = population(|v| matches!(variant_key(v), Some(("PartnerWith", _))));
    eprintln!("FAM_A ({}) = {:?}", fam_a.len(), fam_a);
    eprintln!("FAM_B ({}) = {:?}", fam_b.len(), fam_b);
    eprintln!("FAM_C ({}) = {:?}", fam_c.len(), fam_c);
    eprintln!("FAM_D ({}) = {:?}", fam_d.len(), fam_d);
    let mut u = BTreeSet::new();
    u.extend(fam_a);
    u.extend(fam_b);
    u.extend(fam_c);
    u.extend(fam_d);
    eprintln!("UNION ({}) = {:?}", u.len(), u);
}
