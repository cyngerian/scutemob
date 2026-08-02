//! Greedy mana payment solver.
//!
//! Plans a `Vec<Command::TapForMana>` that, together with whatever is already floating,
//! covers a mana cost. Colored pips first (CR 107.4a), then `{C}` (CR 107.4c: only
//! colorless mana pays it), then generic (CR 107.4: any type pays it).
//!
//! # What "greedy" means here, and what it costs
//!
//! Each phase picks one source at a time and never backtracks. That is exact for the
//! shapes this engine's corpus actually contains (fixed-colour lands, two-colour karoo
//! lands, `{C}{C}` rocks) and can in principle mis-plan a board where a *choice* of
//! which flexible source to spend on which pip matters. The failure mode is a `None`
//! (or a larger-than-minimal plan), never an illegal plan — the returned commands are
//! always ones `handle_tap_for_mana` accepts, which is the property `can_afford` and the
//! auto-tapper both depend on (SR-38: never offer what the engine rejects).
//!
//! # SIM-2: production is counted in MANA, not in SOURCES
//!
//! Until 2026-08-02 every phase decremented the remaining pip count by **one per source
//! tapped**, while `produces` was expanded per unit of mana and the expansion then never
//! read (playtest triage F4). Sol Ring (`{C}{C}`) was credited as one mana. Both
//! directions were live and both were observed by a human in the browser client:
//!
//!   * **Over-tap** — Sol Ring + two Forests tapped for `{2}{G}`: four mana produced,
//!     three spent, one stranded, and CR 500.4 destroyed it at the step boundary.
//!   * **Under-offer** — a `{2}` spell with only a Sol Ring untapped solved to `None`,
//!     so `legal_actions::can_afford` never offered the cast at all.
//!
//! A tapped source now contributes its whole production to a running `Floating` tally,
//! and each pip is paid from that tally; a source is only tapped when the tally cannot
//! already cover the pip. That makes the surplus of a multi-mana source spendable on
//! later pips of the same payment instead of vanishing.
//!
//! # SIM-2: the pool half of `OOS-M11-2` is CLOSED, at the solver
//!
//! [`solve_mana_payment_with_pool`] subtracts the player's existing pool from the cost
//! before planning, mirroring `ManaPool::can_spend`'s allocation exactly (colored pips
//! from matching colored mana, `{C}` from colorless, then *everything left over* against
//! generic). [`solve_mana_payment`] — the pool-blind entry point — is retained for the
//! two `tools/tui` callers (`OOS-SIM1-2`) and is now defined as "solve with an empty
//! pool", so the two cannot drift.
//!
//! `LocalGame::auto_tap_commands_for` used to compensate for the pool-blindness itself
//! with an all-or-nothing check — if the pool fully covered the cost it tapped nothing,
//! and otherwise it solved for the ENTIRE printed cost — so two floating mana plus a
//! `{3}` cast tapped three more sources and destroyed the float (triage F3). That check
//! is gone; both it and the bot path in `LocalGame::advance` now call the residual
//! solver.
//!
//! # SIM-2: the layer-resolution half of `OOS-M11-2` is CLOSED too — it was live-wrong
//!
//! Sources used to be gathered from `obj.characteristics.mana_abilities` **raw**. That
//! was recorded here as a known gap with a theoretical example (Cryptolith Rite). The
//! real one is duller and far more common: `layers.rs` clears `mana_abilities` for a
//! **face-down** permanent (CR 707.2), so the solver planned a tap the engine refused
//! with `"has no mana ability at index 0"` — caught by the S8 scripted playthrough on
//! seed 42 the moment this batch changed which source the solver reaches for.
//! [`gather_sources`] now calls `calculate_characteristics`, the same function
//! `StubProvider`'s `TapForMana` offer loop and `handle_tap_for_mana` itself use, and is
//! hoisted so `legal_actions` pays for it once per enumeration rather than once per card.
//!
//! What remains open under `OOS-M11-2` after this batch: cost *modifiers* (no
//! Thalia-style increase, no reduction) and CR 106.12 restricted mana, neither of which
//! lives in this file.
//!
//! # Which taps this solver refuses to plan, and why
//!
//! [`plannable_tap_ability`] is the single gate. It exists because a plan is worthless
//! unless `handle_tap_for_mana` (`rules/mana.rs`) accepts every command in it: the human
//! path applies taps and the cast as one atomic sequence, so a single refused tap fails
//! the whole cast (the `422` a human sees), and the bot path silently falls back to
//! `PassPriority`.

use mtg_engine::{
    Characteristics, Command, GameObject, GameState, HybridMana, ManaAbility, ManaColor, ManaCost,
    ObjectId, PhyrexianMana, PlayerId, ZoneId,
};

/// WUBRG, the order colored pips are paid in. Colorless is deliberately absent — it is a
/// mana *type*, not a colour (CR 106.1b), and has its own phase.
const COLORS: [ManaColor; 5] = [
    ManaColor::White,
    ManaColor::Blue,
    ManaColor::Black,
    ManaColor::Red,
    ManaColor::Green,
];

/// The order surplus mana is spent on generic pips: colorless first, then GRBUW.
/// Mirrors `ManaPool::spend`'s documented generic order so a plan and the engine's own
/// deduction agree about which mana is left over.
const GENERIC_SPEND_ORDER: [ManaColor; 6] = [
    ManaColor::Colorless,
    ManaColor::Green,
    ManaColor::Red,
    ManaColor::Black,
    ManaColor::Blue,
    ManaColor::White,
];

/// A mana source on the battlefield: its ObjectId, ability index, and what it produces.
///
/// One entry per *ability*, so a permanent with two `requires_tap` mana abilities
/// contributes two entries with the same `object_id`. [`spend`] is what keeps those
/// entries from being planned independently — see its doc.
#[derive(Clone, Debug)]
struct ManaSource {
    object_id: ObjectId,
    ability_index: usize,
    /// One entry per unit of mana produced, e.g. `[Colorless, Colorless]` for Sol Ring.
    /// **Empty for an `any_color` source**: CR 111.10a's "add one mana of any color"
    /// produces exactly one mana whose colour is chosen on the activation command
    /// (`rules/mana.rs` step 8 adds `1 × multiplier` of `resolved_color` and ignores
    /// `produces` entirely for such an ability), so the colour is not knowable until the
    /// pip being paid is known. [`ManaSource::output`] is the accessor that folds the two
    /// cases together.
    produces: Vec<ManaColor>,
    any_color: bool,
    tapped: bool,
}

impl ManaSource {
    /// How many mana this source adds to the pool when tapped.
    fn amount(&self) -> u32 {
        if self.any_color {
            1
        } else {
            self.produces.len() as u32
        }
    }

    /// How many of `color` this source can contribute — for an `any_color` source, one of
    /// *any* colour (CR 106.1b: never colorless).
    fn produces_color(&self, color: ManaColor) -> u32 {
        if self.any_color {
            u32::from(color != ManaColor::Colorless)
        } else {
            self.produces.iter().filter(|c| **c == color).count() as u32
        }
    }

    /// The mana this source actually adds, given the colour chosen for an `any_color`
    /// ability.
    fn output(&self, chosen: Option<ManaColor>) -> Vec<ManaColor> {
        match (self.any_color, chosen) {
            (true, Some(c)) => vec![c],
            (true, None) => Vec::new(),
            (false, _) => self.produces.clone(),
        }
    }
}

/// Mark the chosen source spent — **and every other entry for the same permanent**.
///
/// # The bug this closes (M11-local S8, found by the scripted playthrough)
///
/// `sources` holds one entry per (permanent × mana ability), and the three payment
/// phases below previously set `tapped` on the chosen entry alone. A permanent with two
/// `requires_tap` mana abilities therefore stayed selectable through its *other* entry,
/// so the solver could emit two `Command::TapForMana` for the same permanent. The first
/// taps it; the second is refused with `"permanent ObjectId(n) is already tapped"`,
/// because CR 602.2 pays a `{T}` cost by tapping an untapped permanent and it is no
/// longer untapped.
///
/// Observed rather than reasoned to: the S8 playthrough (seed 1, turn 21) submitted a
/// `CastSpell` the game had just offered and got exactly that rejection back. On the bot
/// path the same plan has always been silently absorbed — `LocalGame::advance`'s
/// command-rejected fallback issues `PassPriority` — which is why a bug reachable in
/// every game since the solver was written surfaced only once a *human* path refused to
/// swallow an engine error (S8 item 4).
fn spend(sources: &mut [ManaSource], idx: usize) {
    let object_id = sources[idx].object_id;
    for source in sources.iter_mut() {
        if source.object_id == object_id {
            source.tapped = true;
        }
    }
}

/// Whether this `{T}` mana ability is one the solver may put in a plan — i.e. one
/// `handle_tap_for_mana` will accept right now, for reasons the solver can check without
/// simulating the activation.
///
/// Each arm mirrors a specific rejection in `rules/mana.rs`. **SIM-2 added all of them**;
/// before, the only filters were "untapped" and "requires_tap", so the solver freely
/// planned taps the engine then refused — which on the human path fails the whole atomic
/// tap-and-cast sequence and surfaces as a `422` on a cast the client had just been
/// offered.
///
/// Populations measured by enumerating `all_cards()` (SR-36 — never grep) over the 1,803
/// defs at 2026-08-02: 322 defs have a `{T}` mana ability; 20 carry a mana component, 8 a
/// life component, 13 an activation condition, 0 a counter cost.
fn plannable_tap_ability(
    state: &GameState,
    player: PlayerId,
    obj: &GameObject,
    chars: &Characteristics,
    ability: &ManaAbility,
) -> bool {
    // CR 605.3a / CR 118.3a: the ability's OWN mana component (a Signet's `{1}`, a filter
    // land's `{W/B}`) is paid from the pool at activation, before the mana is produced.
    // Planning that means interleaving a tap plan with a pool state this solver does not
    // model per-step, and crediting the gross production while ignoring the cost would
    // *over*-credit — a Signet would look like two free mana. Refusing to plan them is
    // the conservative direction: it can only under-offer. `OOS-SIM2-2`.
    //
    // **Solver-only**, and the one check that is: `StubProvider` still OFFERS these taps,
    // because a human can perfectly well tap a Signet by hand once they have the `{1}` —
    // refusing to offer it would take a legal play away. The solver declining to *plan*
    // one costs nobody anything.
    if let Some(mana_cost) = &ability.mana_cost {
        if mana_cost.mana_value() > 0
            || !mana_cost.hybrid.is_empty()
            || !mana_cost.phyrexian.is_empty()
        {
            return false;
        }
    }
    tap_ability_is_activatable(state, player, obj, chars, ability)
}

/// The subset of [`plannable_tap_ability`] that is about **legality right now**, shared
/// with `StubProvider`'s `TapForMana` offer loop.
///
/// Two callers, one predicate, deliberately: **OOS-CARDS2-9** is the same defect appearing
/// at both — "the provider's affordability check counts mana abilities it could not legally
/// activate ... the fix is one place: make the solver ask whether the ability is
/// *activatable*, not whether its source is untapped". `legal_actions.rs` had the *offer*
/// half of it (it checked `life_cost` per SG-1 and nothing else, so an unmet
/// `activation_condition` and a summoning-sick creature were both offered and both
/// refused), and this file had the affordability half. Splitting the fix would have left
/// the play-server driver's `KNOWN_FALSE_OFFERS` list still describing a live bug.
pub(crate) fn tap_ability_is_activatable(
    state: &GameState,
    player: PlayerId,
    obj: &GameObject,
    chars: &Characteristics,
    ability: &ManaAbility,
) -> bool {
    // CR 119.4 / CR 118.3b (rules/mana.rs step 5b): a horizon land's "Pay 1 life" is
    // rejected outright when the player cannot pay. Mirrors `legal_actions.rs`'s SG-1
    // check on the offer side, including its `>=` (CR 119.4 permits paying down to 0).
    if ability.life_cost > 0 {
        let life = state.player(player).map(|p| p.life_total).unwrap_or(0);
        if life < ability.life_cost as i32 {
            return false;
        }
    }
    // CR 602.2c / CR 118.3 (rules/mana.rs step 5b2): a counter cost with too few counters
    // present. No def in the corpus lowers to this today; the check is here because its
    // absence is invisible until the first one does.
    if let Some((counter, count)) = &ability.remove_counter {
        if obj.counters.get(counter).copied().unwrap_or(0) < *count {
            return false;
        }
    }
    // CR 602.5b (SR-37, rules/mana.rs step 5c): "Activate only if ..." — Mox Opal's
    // metalcraft, Tainted Field's Swamp. Evaluated with the same `check_condition` the
    // engine uses, so the two cannot disagree.
    if let Some(condition) = &ability.activation_condition {
        let ctx = mtg_engine::effects::EffectContext::new(player, obj.id, vec![]);
        if !mtg_engine::effects::check_condition(state, condition, &ctx) {
            return false;
        }
    }
    // CR 302.6 / CR 702.10 (rules/mana.rs step 6): a summoning-sick creature cannot pay a
    // `{T}` cost. This is the arm with real traffic — a mana dork played this turn was
    // credited by `can_afford`, the cast was offered, and the auto-tap plan was then
    // refused. Read from the LAYER-RESOLVED characteristics, exactly as `rules/mana.rs`
    // does, so an animated land is summoning-sick and layer-granted haste (Fervor) is
    // seen (CR 613.1d/613.1f).
    if ability.requires_tap
        && obj.has_summoning_sickness
        && chars.card_types.contains(&mtg_engine::CardType::Creature)
        && !chars.keywords.contains(&mtg_engine::KeywordAbility::Haste)
    {
        return false;
    }
    true
}

/// Gather this player's untapped, plannable `{T}` mana sources, in battlefield order.
///
/// # CR 613.1f: layer-resolved, and what that costs
///
/// Until SIM-2 this read `obj.characteristics.mana_abilities` **raw** — the
/// layer-resolution half of `OOS-M11-2`, which was a documented known gap and turned out
/// to be live-wrong rather than theoretical. `layers.rs` clears `mana_abilities`
/// outright for a **face-down** permanent (CR 707.2 — morph/manifest/cloak), so a
/// face-down mana source was planned from its base characteristics and
/// `handle_tap_for_mana`, which reads `expect_characteristics`, answered `"object
/// ObjectId(487) has no mana ability at index 0"`. Found by
/// `test_s8_scripted_human_playthrough_is_clean_on_five_seeds` (seed 42, turn 21) the
/// first time this batch changed which source the solver reaches for — the defect
/// predates SIM-2, the exposure did not.
///
/// The same call also picks up granted mana abilities (Cryptolith Rite, Chromatic
/// Lantern) and removals (Humility), which `StubProvider`'s own `TapForMana` offer loop
/// has always resolved — so the offer list and the payment plan now read the same
/// characteristics, which is the property that makes them impossible to disagree.
///
/// **Cost, measured rather than assumed**: `can_afford` calls this once per castable
/// card per `legal_actions` enumeration, so the layer system's work in the provider is
/// multiplied by roughly the hand size. `mtg-fuzzer --games 60 --threads 1
/// --max-turns 40` reports **6.8 s before and 6.8 s after** on seeds 1 and 7 (two runs
/// each), i.e. inside noise, which is why this function is not hoisted out of the solve
/// and handed a pre-gathered list — a real complication for an unmeasurable saving.
fn gather_sources(state: &GameState, player: PlayerId) -> Vec<ManaSource> {
    let mut sources: Vec<ManaSource> = Vec::new();
    for obj in state.objects_in_zone(&ZoneId::Battlefield) {
        if obj.controller != player || obj.status.tapped {
            continue;
        }
        let chars = mtg_engine::rules::layers::calculate_characteristics(state, obj.id)
            .unwrap_or_else(|| obj.characteristics.clone());
        for (idx, ability) in chars.mana_abilities.iter().enumerate() {
            if !ability.requires_tap {
                continue;
            }
            if !plannable_tap_ability(state, player, obj, &chars, ability) {
                continue;
            }
            let mut produces = Vec::new();
            for (color, &amount) in ability.produces.iter() {
                for _ in 0..amount {
                    produces.push(*color);
                }
            }
            sources.push(ManaSource {
                object_id: obj.id,
                ability_index: idx,
                produces,
                any_color: ability.any_color,
                tapped: false,
            });
        }
    }
    sources
}

/// Pick the untapped source that wastes the least: the largest useful production that
/// still fits inside `need`, and failing that the smallest useful production of all.
/// Ties break toward the earliest source, so the plan is a deterministic function of
/// battlefield order.
///
/// This is the "prefer small producers" rule from the SIM-2 brief, stated as the thing it
/// is actually for. A single `{1}` never taps the Sol Ring while a Forest is up (nothing
/// larger fits, so the Forest wins outright), and a `{2}` takes the Sol Ring alone rather
/// than two Forests — a plain ascending-size order gets the first case right and the
/// second wrong, tapping two sources and stranding a mana exactly as the pre-fix code did.
fn pick_least_waste(
    sources: &[ManaSource],
    need: u32,
    useful: impl Fn(&ManaSource) -> u32,
) -> Option<usize> {
    let mut best_fit: Option<(u32, usize)> = None;
    let mut smallest: Option<(u32, usize)> = None;
    for (idx, source) in sources.iter().enumerate() {
        if source.tapped {
            continue;
        }
        let value = useful(source);
        if value == 0 {
            continue;
        }
        if value <= need && best_fit.is_none_or(|(best, _)| value > best) {
            best_fit = Some((value, idx));
        }
        if smallest.is_none_or(|(small, _)| value < small) {
            smallest = Some((value, idx));
        }
    }
    best_fit.or(smallest).map(|(_, idx)| idx)
}

/// Mana produced but not yet assigned to a pip, during one solve.
#[derive(Default, Debug)]
struct Floating {
    white: u32,
    blue: u32,
    black: u32,
    red: u32,
    green: u32,
    colorless: u32,
}

impl Floating {
    fn get_mut(&mut self, color: ManaColor) -> &mut u32 {
        match color {
            ManaColor::White => &mut self.white,
            ManaColor::Blue => &mut self.blue,
            ManaColor::Black => &mut self.black,
            ManaColor::Red => &mut self.red,
            ManaColor::Green => &mut self.green,
            ManaColor::Colorless => &mut self.colorless,
        }
    }

    fn add_all(&mut self, produced: &[ManaColor]) {
        for color in produced {
            *self.get_mut(*color) += 1;
        }
    }

    /// Spend one mana of exactly `color`; `false` if there is none.
    fn take(&mut self, color: ManaColor) -> bool {
        let slot = self.get_mut(color);
        if *slot == 0 {
            return false;
        }
        *slot -= 1;
        true
    }

    /// Spend one mana of any type against a generic pip (CR 107.4).
    fn take_any(&mut self) -> bool {
        for color in GENERIC_SPEND_ORDER {
            if self.take(color) {
                return true;
            }
        }
        false
    }
}

/// Attempt to solve a mana payment greedily, **ignoring the player's mana pool**.
///
/// Equivalent to [`solve_mana_payment_with_pool`] against an empty pool. Retained as the
/// entry point the two `tools/tui` auto-tap call sites use (`OOS-SIM1-2`); everything
/// inside `crates/simulator` goes through the pool-aware form.
pub fn solve_mana_payment(
    state: &GameState,
    player: PlayerId,
    cost: &ManaCost,
) -> Option<Vec<Command>> {
    let sources = gather_sources(state, player);
    solve_tracker(&sources, player, PipTracker::from_cost(cost))
}

/// Solve for the cost **that remains after the player's existing mana pool is applied**
/// (SIM-2, the pool half of `OOS-M11-2`).
///
/// The subtraction mirrors `ManaPool::can_spend` exactly — colored pips from matching
/// colored mana, `{C}` from colorless mana only (CR 107.4c), then every mana left over
/// against generic (CR 107.4) — so a cost this returns `Some(vec![])` for is a cost the
/// engine agrees the pool already covers, and a plan it returns leaves the pool plus the
/// tapped mana exactly sufficient.
///
/// CR 106.12 restricted mana is invisible here, as it is to `can_spend(cost, None)`; that
/// remains part of the open surviving half of `OOS-M11-2`.
pub fn solve_mana_payment_with_pool(
    state: &GameState,
    player: PlayerId,
    cost: &ManaCost,
) -> Option<Vec<Command>> {
    let mut tracker = PipTracker::from_cost(cost);
    if let Ok(player_state) = state.player(player) {
        tracker.subtract_pool(&player_state.mana_pool);
    }
    solve_tracker(&gather_sources(state, player), player, tracker)
}

/// The solver proper. `remaining` is already flattened (CR 107.4e/107.4f) and already
/// net of whatever the caller decided is available.
fn solve_tracker(
    sources: &[ManaSource],
    player: PlayerId,
    mut remaining: PipTracker,
) -> Option<Vec<Command>> {
    let mut sources = sources.to_vec();
    let mut commands = Vec::new();
    let mut floating = Floating::default();

    // Phase 1: colored pips (CR 107.4a). A fixed-colour source is preferred over an
    // any-colour one, which is kept back for a pip nothing else can pay.
    for color in COLORS {
        while remaining.colored(color) > 0 {
            if floating.take(color) {
                remaining.pay_colored(color);
                continue;
            }
            let need = remaining.colored(color);
            let idx = pick_least_waste(&sources, need, |s| {
                if s.any_color {
                    0
                } else {
                    s.produces_color(color)
                }
            })
            .or_else(|| pick_least_waste(&sources, need, |s| s.produces_color(color)))?;
            tap(
                &mut sources,
                idx,
                player,
                Some(color),
                &mut floating,
                &mut commands,
            );
        }
    }

    // Phase 2: `{C}` pips — only colorless mana pays them (CR 107.4c). An any-colour
    // source can never help (CR 106.1b: colorless is not a colour), which
    // `produces_color` encodes, so no fallback pass is needed here.
    while remaining.colorless > 0 {
        if floating.take(ManaColor::Colorless) {
            remaining.colorless -= 1;
            continue;
        }
        let need = remaining.colorless;
        let idx = pick_least_waste(&sources, need, |s| s.produces_color(ManaColor::Colorless))?;
        tap(
            &mut sources,
            idx,
            player,
            None,
            &mut floating,
            &mut commands,
        );
    }

    // Phase 3: generic pips — any mana pays them (CR 107.4), so the source is chosen by
    // total production and an any-colour source contributes one mana of a deterministic
    // White (mirroring `legal_actions.rs`'s pick).
    while remaining.generic > 0 {
        if floating.take_any() {
            remaining.generic -= 1;
            continue;
        }
        let need = remaining.generic;
        let idx = pick_least_waste(&sources, need, |s| s.amount())?;
        let chosen = if sources[idx].any_color {
            Some(ManaColor::White)
        } else {
            None
        };
        tap(
            &mut sources,
            idx,
            player,
            chosen,
            &mut floating,
            &mut commands,
        );
    }

    Some(commands)
}

/// Emit the `TapForMana` for `sources[idx]`, mark the whole permanent spent, and credit
/// everything it produces to `floating`.
///
/// `chosen_color` is `Some` **only** for an `any_color` ability (PB-EF12 / CR 605.3b: the
/// colour is chosen on the activation command, and `handle_tap_for_mana` rejects a
/// `chosen_color` supplied for a fixed-colour ability outright), so the caller's colour
/// hint is dropped for a fixed source.
fn tap(
    sources: &mut [ManaSource],
    idx: usize,
    player: PlayerId,
    color_hint: Option<ManaColor>,
    floating: &mut Floating,
    commands: &mut Vec<Command>,
) {
    spend(sources, idx);
    let source = &sources[idx];
    let chosen_color = if source.any_color { color_hint } else { None };
    floating.add_all(&source.output(chosen_color));
    commands.push(Command::TapForMana {
        player,
        source: source.object_id,
        ability_index: source.ability_index,
        chosen_color,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    });
}

/// Track remaining mana pips to pay.
struct PipTracker {
    white: u32,
    blue: u32,
    black: u32,
    red: u32,
    green: u32,
    colorless: u32,
    generic: u32,
}

impl PipTracker {
    fn from_cost(cost: &ManaCost) -> Self {
        let mut tracker = Self {
            white: cost.white,
            blue: cost.blue,
            black: cost.black,
            red: cost.red,
            green: cost.green,
            colorless: cost.colorless,
            generic: cost.generic,
        };

        // Flatten hybrid mana into colored pip requirements (CR 107.4e).
        // Default: pay with the first color (ColorColor) or the specific color (GenericColor).
        // GenericColor ({2/W} etc.) defaults to paying the color pip, not 2 generic.
        for hybrid in &cost.hybrid {
            match hybrid {
                HybridMana::ColorColor(c1, _c2) => {
                    // Default: pay with first color.
                    tracker.add_color(*c1, 1);
                }
                HybridMana::GenericColor(c) => {
                    // Default: pay with the color (not 2 generic).
                    tracker.add_color(*c, 1);
                }
            }
        }

        // Flatten Phyrexian mana into colored pip requirements (CR 107.4f).
        // Default: pay with mana (not 2 life).
        for phyrexian in &cost.phyrexian {
            match phyrexian {
                PhyrexianMana::Single(c) => {
                    tracker.add_color(*c, 1);
                }
                PhyrexianMana::Hybrid(c1, _c2) => {
                    // Default: pay with first color.
                    tracker.add_color(*c1, 1);
                }
            }
        }

        // x_count: X defaults to 0 for the solver (CR 202.3e), so no generic added.
        // Callers that have an announced X fold `x_value * x_count` into `generic`
        // themselves (CR 107.3 / 601.2b) — `LocalGame::auto_tap_commands_for` does.

        tracker
    }

    /// Deduct what the player's existing pool already covers, in `ManaPool::can_spend`'s
    /// own order: each colored pip from mana of that colour, `{C}` from colorless mana,
    /// and only then the entire remainder of the pool against generic (CR 107.4).
    ///
    /// Saturating throughout: a pool larger than the cost leaves a zero tracker, which
    /// `solve_tracker` answers with an empty plan.
    fn subtract_pool(&mut self, pool: &mtg_engine::ManaPool) {
        let mut leftover = 0u32;
        for color in GENERIC_SPEND_ORDER {
            let have = pool.get(color);
            let owed = self.colored(color);
            let paid = have.min(owed);
            self.pay_colored_n(color, paid);
            leftover += have - paid;
        }
        self.generic = self.generic.saturating_sub(leftover);
    }

    fn add_color(&mut self, color: ManaColor, amount: u32) {
        match color {
            ManaColor::White => self.white += amount,
            ManaColor::Blue => self.blue += amount,
            ManaColor::Black => self.black += amount,
            ManaColor::Red => self.red += amount,
            ManaColor::Green => self.green += amount,
            ManaColor::Colorless => self.colorless += amount,
        }
    }

    fn colored(&self, color: ManaColor) -> u32 {
        match color {
            ManaColor::White => self.white,
            ManaColor::Blue => self.blue,
            ManaColor::Black => self.black,
            ManaColor::Red => self.red,
            ManaColor::Green => self.green,
            ManaColor::Colorless => self.colorless,
        }
    }

    fn pay_colored(&mut self, color: ManaColor) {
        self.pay_colored_n(color, 1);
    }

    fn pay_colored_n(&mut self, color: ManaColor, amount: u32) {
        match color {
            ManaColor::White => self.white = self.white.saturating_sub(amount),
            ManaColor::Blue => self.blue = self.blue.saturating_sub(amount),
            ManaColor::Black => self.black = self.black.saturating_sub(amount),
            ManaColor::Red => self.red = self.red.saturating_sub(amount),
            ManaColor::Green => self.green = self.green.saturating_sub(amount),
            ManaColor::Colorless => self.colorless = self.colorless.saturating_sub(amount),
        }
    }
}
