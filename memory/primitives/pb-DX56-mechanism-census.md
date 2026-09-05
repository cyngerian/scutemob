# PB-DX56 — mechanism census for the two live fuzz violation classes

Read-only investigation, `scutemob-235` worktree, HEAD of
`feat/pb-dx56-make-the-two-live-fuzz-violations-diagnosable-then-d`.
No `git` command was run. No `cargo` command was run — **every claim below is from
reading source or from the CR text quoted verbatim from the rules server**, so nothing
here is backed by execution unless the fuzz figures in the dispatch brief are treated as
such. Each statement is tagged **ESTABLISHED** (I read the lines that make it true) or
**INFERENCE** (it follows from lines I read, but I did not watch it happen).

Two premises in the brief are **corrected** below; both are flagged inline with the line
that refutes them (§1.2 note, §2.3 note).

---

## Q1 — the attachment mechanism

### 1.1 Every production site that removes an entry from `state.objects`

`rg -n "objects\.remove" crates/engine/src/` returns six hits; two are
`state.stack_objects` (a different map) and are excluded. The four real ones:

| # | Site | What it removes | Fixes up other objects' `attached_to`? | Fixes up other objects' `attachments`? |
|---|------|-----------------|----------------------------------------|----------------------------------------|
| 1 | `state/mod.rs:1632` (`GameState::move_object_to_zone`) | the departing object's OLD id | **NO** | **NO** |
| 2 | `state/mod.rs:2146` (`GameState::move_object_to_bottom_of_zone`) | same | **NO** | **NO** |
| 3 | `rules/sba.rs:467` (`check_token_sbas`, CR 704.5d) | a token found off the battlefield | **NO** | **NO** |
| 4 | `effects/mod.rs:5775` (CR 701.42a meld) | the two phantom exile intermediaries | **NO** | **NO** |
| 5 | `rules/resolution.rs:8051` (CR 729.2b mutate) | the mutating spell's `source_object` | **NO** | **NO** |

(Five rows, not four — `resolution.rs:8051` and `effects/mod.rs:5775` are both real.)

**ESTABLISHED for sites 1 and 2.** I read both function bodies end to end
(`state/mod.rs:1577-2057` and `:2100-2305`). The *only* cross-object fix-ups either one
performs are:

* CR 702.95e soulbond — `state/mod.rs:1759-1765` and `:2263-2269`: `if let Some(partner_id)
  = old_object.paired_with { ... partner.paired_with = None }`.
* MR-M8-16 replacement-effect GC — `state/mod.rs:2050-2056`:
  `self.replacement_effects.retain(...)` for `WhileSourceOnBattlefield` effects sourced on
  the departed id.

The brief is right that the NEW object is minted with `attached_to: None` and
`attachments: Vector::new()` (`state/mod.rs:1646` / `:1657` and `:2160` / `:2157`, plus
the CR 729.3 split components at `:1839` and the CR 712.4a meld component at `:1961`).
**Neither function touches the *other* side of the relationship at all** — there is no
`attached_to` and no `attachments` write anywhere in either body other than the fresh
`None`/`Vector::new()` initialisers on the objects being *created*. So the instant
`state/mod.rs:1632` executes, every battlefield object whose `attached_to == Some(old_id)`
is dangling, and every object whose `attachments` contains `old_id` holds a dead entry.

**ESTABLISHED for sites 3, 4, 5.** I read each:

* `sba.rs:459-469` — removes the token from its zone and from `state.objects`, pushes
  `TokenCeasedToExist`. No attachment handling. (In practice a token reaching this arm has
  already been through `move_object_to_zone`, so its battlefield id was already retired at
  site 1; this arm retires the *graveyard* id, which nothing is attached to.)
* `effects/mod.rs:5771-5780` — removes `exiled_source_id` / `exiled_partner_id` from
  `state.objects` and from the Exile zone set. Exile objects, so nothing on the
  battlefield is attached to them. **INFERENCE** that this is therefore harmless.
* `resolution.rs:8046-8051` — removes the mutating spell's `source_object` from its zone
  and from `state.objects`. That object was on the stack (CR 729.2b), so again nothing on
  the battlefield is attached to it. **INFERENCE** that this is harmless.

**So: the entire dangle supply is site 1 and site 2 — i.e. every zone change of an
attached-to permanent.** ESTABLISHED.

### 1.2 Every SBA that can clear a dangling `attached_to`, and its exact blind spots

Both live in `crates/engine/src/rules/sba.rs`. Both are called from `apply_sbas_once`
(`sba.rs:236` equipment, `sba.rs:235` aura), which is called in a fixpoint loop by
`check_and_apply_sbas` (`sba.rs:66-108`).

#### The shared `chars_map` precondition — `sba.rs:183-195`

```
let battlefield_ids: Vec<ObjectId> = state.objects.iter()
    .filter(|(_, obj)| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
    .map(|(id, _)| *id).collect();
let chars_map: HashMap<ObjectId, Characteristics> = battlefield_ids.iter()
    .filter_map(|&id| { let chars = calculate_characteristics(state, id)?; Some((id, chars)) })
    .collect();
```

`is_phased_in()` is `!self.status.phased_out` (`crates/card-types/src/state/game_object.rs:1608-1610`).
`calculate_characteristics` returns `None` **if and only if the object does not exist**
(`rules/layers.rs:277-278`, its own doc: *"Returns `None` if — and only if — the object
does not exist in the game state"*). Every id in `battlefield_ids` came out of
`state.objects`, so it exists.

> **ESTABLISHED — for a battlefield object, `chars_map.get(id) == None` ⟺
> `obj.status.phased_out`.** There is no other way to be absent from the map.

#### CR 704.5m — `check_aura_sbas`, `sba.rs:1121-1277`

Filter, in order, with the early exits:

| Line | Guard | Skips |
|------|-------|-------|
| `1129` | `obj.zone != ZoneId::Battlefield → false` | off-battlefield objects |
| `1134` | `obj.status.phased_out → false` (CR 702.26b, explicit) | **phased-out auras** |
| `1138-1142` | `expect_characteristics(state, aura_id)`; `!aura_chars.subtypes.contains("Aura") → false` | **anything whose LAYER-RESOLVED subtypes lack `Aura`** |
| `1144-1148` | `card_types.contains(Creature) → true` (CR 303.4d) | — |
| `1150-1153` | `attached_to == None → true` | — |
| `1157-1163` | `state.objects.get(&target_id).map(\|t\| t.zone != Battlefield).unwrap_or(true)` → `true` | — |

**Line 1157-1163 is the arm that catches a dangle**: `unwrap_or(true)` means "target id
not a key of `state.objects`" ⇒ illegal ⇒ CR 704.5m puts the Aura into its owner's
graveyard (`sba.rs:1253-1276`). ESTABLISHED.

It uses `expect_characteristics` (`layers.rs:821-837`), which is
`calculate_characteristics(..).unwrap_or(debug_assert + printed fallback)`. Because
`aura_id` is a live key of the iteration it is never `None` here, so **the Aura arm has no
`chars_map`-style hole** — it recomputes rather than consulting the map. ESTABLISHED.

#### CR 704.5n — `check_equipment_sbas`, `sba.rs:1286-1404`

| Line | Guard | Skips |
|------|-------|-------|
| `1301` | `obj.zone != ZoneId::Battlefield → false` | off-battlefield objects |
| `1304-1306` | `let Some(chars) = chars_map.get(id) else { return false }` | **phased-out equipment (see above)** |
| `1307-1312` | `!is_equipment && !is_fortification → false` (layer-resolved subtypes) | **anything whose LAYER-RESOLVED subtypes lack `Equipment`/`Fortification`** |
| `1313` | `attached_to == None → false` | — |
| `1317` | `state.objects.get(&target_id)` `None → true` | — |
| `1321-1322` | `t.zone != Battlefield → true` | — |
| `1325` | target absent from `chars_map` → `true` | (a phased-out *target* is unattached from — a separate CR 702.26b question, not this subject) |

**Line 1317 is the arm that catches a dangle** — unattach at `sba.rs:1391-1402`.
ESTABLISHED.

> **The phased-out asymmetry the brief asks about: the Equipment arm has NO explicit
> `phased_out` check.** ESTABLISHED — `grep -n "phased_out" sba.rs` gives `959`, `1134`
> and the `is_phased_in()` filter at `188`; there is no occurrence inside
> `check_equipment_sbas` (`1286-1404`).
>
> **But it is not a behavioural asymmetry, and this corrects the shape of the question.**
> The Equipment arm reaches the same outcome through `chars_map` — `sba.rs:1304-1306`
> returns `false` for exactly the phased-out set, because `battlefield_ids` at
> `sba.rs:186` already filtered on `is_phased_in()`. So both arms skip phased-out
> attachers; one says so and one does it by omission from a map built two hundred lines
> earlier. **The asymmetry worth naming is documentary, not behavioural** — and it is a
> genuine hazard, because a future edit that stopped filtering `battlefield_ids` on
> phasing would silently give the Equipment arm CR 702.26b-wrong behaviour with nothing
> in that function to notice.

#### An object that is BOTH, or NEITHER

* **BOTH** (subtypes contain `Aura` and `Equipment`) — CR 303.4i contemplates this
  ("an Equipment that isn't also an Aura"). Both arms would fire; the Aura arm runs first
  (`sba.rs:235` before `:236`) and moves it to the graveyard, so the equipment arm's
  `state.objects.get(id)` iteration in the same pass has already been built from a
  pre-move snapshot… **INFERENCE**: `illegal_equip` is collected from `state.objects`
  *inside* `check_equipment_sbas`, i.e. after `check_aura_sbas` has already run and moved
  the object, so the object is no longer on the battlefield and the equipment arm skips it
  at `sba.rs:1301`. Net effect: graveyard, which is CR 704.5m's disposition. I did not
  execute this.
* **NEITHER** — skipped by both. This is the blind spot, below.

#### The iff

> **An object can hold a dangling `attached_to` on the battlefield indefinitely if and
> only if, at every subsequent SBA sweep, either
> (a) `obj.status.phased_out` is true (`sba.rs:1134` for Auras, `sba.rs:186` → `:1304-1306`
> for Equipment), or
> (b) its LAYER-RESOLVED `subtypes` contain none of `Aura`, `Equipment`, `Fortification`
> (`sba.rs:1140` and `sba.rs:1307-1312`).**
>
> ESTABLISHED, from the two filter chains above. Note the quantifier: *at every subsequent
> sweep*. A dangle that satisfies neither condition is cleared at the next sweep, which
> makes §1.3 the operative question for the fuzz data.

**Reachability of (a):** CR 702.26c is implemented — `turn_actions.rs:1133-1137` phases out
an attached permanent *indirectly* along with its host, setting both `status.phased_out`
and `phased_out_indirectly`. So a phased-out attacher normally has a phased-out host, and a
phased-out host cannot be moved by any SBA (all of them filter on `is_phased_in`). **INFERENCE:
(a) is probably unreachable at HEAD**; I did not enumerate every non-SBA zone-move site for a
phased-out-permanent guard, so this is not settled.

**Reachability of (b) — this is the structurally interesting one and it has a live route.**
`Aura` ∈ `ALL_ENCHANTMENT_TYPES` (`crates/card-types/src/state/types.rs:1938`), `Equipment`
and `Fortification` ∈ `ALL_ARTIFACT_TYPES` (`:1927-1928`), and `correlated_card_types`
(`types.rs:2039-2062`) maps them to `Enchantment` / `Artifact`. Therefore:

* `LayerModification::SetTypeLine` — `layers.rs:2460`, `chars.subtypes = subtypes.clone()`,
  a wholesale replacement. Drops `Aura`/`Equipment` unconditionally.
* `LayerModification::SetCardTypes` — `layers.rs:2521-2530`, implements CR 205.1a's
  correlated-subtype removal: removing `Artifact` drops `Equipment`; removing `Enchantment`
  drops `Aura`. The in-source comment at `layers.rs:2515-2519` names exactly this
  mechanism.
* `LayerModification::LoseAllSubtypes` — `layers.rs:2480-2482`, `chars.subtypes = OrdSet::new()`.
* CR 708.2a face-down — `layers.rs:515-520`, `chars.subtypes = OrdSet::new()` **before the
  layer loop**. A face-down permanent is invisible to both arms.

Corpus users (`rg -l` over `crates/card-defs/src/defs/`): `SetTypeLine` —
`imprisoned_in_the_moon`, `oko_thief_of_crowns`, `vraska_betrayals_sting`,
`eaten_by_piranhas`, `polymorphists_jest`; `SetCardTypes` — `turn`, `darksteel_mutation`,
`vraska_betrayals_sting`, `eaten_by_piranhas`, `kenriths_transformation`;
`LoseAllSubtypes` — `vraska_betrayals_sting` only.

`imprisoned_in_the_moon` declares `EnchantTarget::Filtered(..)` since PB-DX20b
(`imprisoned_in_the_moon.rs:35`, and `:9` records that the old `EnchantTarget::Permanent`
"also admitted artifacts, enchantments and battles"), so it can no longer be aimed at an
Equipment or an Aura. `oko_thief_of_crowns` is `Completeness::known_wrong` (not deck-legal).
`kenriths_transformation`, `darksteel_mutation`, `eaten_by_piranhas` all declare
`EnchantTarget::Creature` and are `Complete` by the `#[default]` derive (no explicit marker
in the file). **INFERENCE, not established: reaching (b) with these needs an attacher that
is simultaneously a creature, which CR 301.5c and CR 303.4d normally forbid — I did not
find a corpus route and I did not execute one.** The face-down route is stronger evidence
in the *negative* direction: `rg -n "status\.face_down\s*=\s*true"` over `rules/` and
`effects/` gives five sites (`effects/mod.rs:5444` manifest, `:5506` cloak,
`casting.rs:4874` a stack object, `foretell.rs:109` an exile card, `resolution.rs:974`
morph-entering, `resolution.rs:6554` an exile card) and **every one of them acts on an
object that is entering a zone or is not on the battlefield — none turns an
already-attached battlefield permanent face down.** ESTABLISHED by that enumeration;
so the face-down leg of (b) is unreachable at HEAD.

**One route into (b) that IS established as a code-level asymmetry**, whether or not the
corpus can reach it today: the aura ATTACH site decides "is this an Aura" from **raw**
characteristics while the aura SBA decides it from **layer-resolved** ones.
`resolution.rs:2013-2026`:

```
let is_aura = { let obj = state.expect_object(new_id);
    obj.map(|o| o.characteristics.card_types.contains(&CardType::Enchantment)
             && o.characteristics.subtypes.contains(&SubType("Aura".to_string())))
       .unwrap_or(false) };
```

versus `sba.rs:1138-1142`'s `expect_characteristics(state, aura_id).subtypes`. **An Aura
resolving under a continuous effect that strips its Aura subtype is attached by the first
and then permanently invisible to the second.** ESTABLISHED as an asymmetry; **INFERENCE**
that it is reachable.

### 1.3 Is the dangle transient or at rest?

**The engine sweeps SBAs at nine call sites and nowhere else.** `rg -n
"check_and_apply_sbas\(" crates/engine/src/`, excluding the definition:

* `resolution.rs:374` and `resolution.rs:8724` — inside `resolve_top_of_stack_inner`
* `resolution.rs:9031` — inside `counter_stack_object`
* `engine.rs:1263` `handle_pay_echo`, `:1511` `handle_pay_cumulative_upkeep`,
  `:1685` `handle_pay_recover`, `:1983` `transform_permanent_in_place`,
  `:2208` `handle_activate_craft`, `:2474` `handle_turn_face_up`
* `engine.rs:2693` and `:2736` — both inside `enter_step`

**ESTABLISHED, and this is the load-bearing count:**
`crates/engine/src/rules/{abilities,casting,combat,turn_actions,mana,turn_structure,replacement}.rs`
each contain **ZERO** occurrences of `check_and_apply_sbas` (counted individually).

Consequences:

1. **A dangle created by an SBA-driven death heals inside the same sweep.**
   `check_and_apply_sbas` (`sba.rs:66-108`) loops until a pass returns no events. A creature
   dying to CR 704.5g in pass *N* produces events, so pass *N+1* runs, and pass *N+1*'s
   `check_equipment_sbas` / `check_aura_sbas` see the dangle and clear it. **ESTABLISHED**
   from the loop structure; combat damage, board wipes and lethal-damage deaths are all in
   this class because the CombatDamage step is entered through `enter_step`, whose
   `has_priority()` branch sweeps at `engine.rs:2736`.

2. **A dangle created during a command that does not sweep survives until the next command
   that does.** `process_command`'s `Command::CastSpell` arm (`engine.rs:447-488`) runs
   `casting::handle_cast_spell` + `check_and_flush_triggers` and nothing else;
   `Command::ActivateAbility` (`engine.rs:489-519`) is the same shape.
   `check_and_flush_triggers` (`engine.rs:41-58`) sweeps triggers, **not** SBAs.
   ESTABLISHED.

   The zone-move sites reachable inside such a command:
   * `mana.rs:493` — a `Cost::SacrificeSelf` mana ability sacrificing its own source, inside
     `handle_tap_for_mana`. This is `OOS-M11-7`'s recorded case verbatim.
   * `abilities.rs:944`, `:1005`, `:1216`, `:1302`, `:1315` — the activation-cost payment
     block of `handle_activate_ability`: discard, sacrifice-self, sacrifice-another,
     sacrifice-a-Food, exile. `abilities.rs:1216` is the "Sacrifice the permanent (move to
     graveyard)" site the comment at `:1170` announces.
   * `casting.rs` — 21 `move_object_to_zone` sites, the CR 601.2h additional-cost payments.
   * `abilities.rs:2418` — CR 702.49a ninjutsu returning an attacker to hand.

   **So the answer to "would the dangle necessarily be cleared by the next SBA sweep" is
   YES for every object that fails the §1.2 iff — but "the next sweep" is not "before the
   invariant runs".** `check_attachment_validity` runs after every tracked command
   (`crates/simulator/src/local_game.rs:1184` and `:1209` gate it on
   `self.check_invariants`; I did not read that file's body, per the dispatch constraint —
   this is the brief's own statement plus the two guard line numbers). A cast or an
   activation that pays a sacrifice cost therefore reports a dangle that the very next
   step entry or spell resolution erases. **ESTABLISHED** for the mechanism;
   **INFERENCE** that this is what the fuzz is seeing.

3. **The seeds 8 / 9 / 18 pattern — one attacher, several dead targets, tens of turns
   apart — is consistent with (2) and NOT with a permanent blind spot, and the arithmetic
   is the reason.** 102 raw violations over 7 games is ~15 per game. A dangle that never
   heals would be re-reported after *every* subsequent tracked command for the rest of the
   game — hundreds, not fifteen — because the check is per-command and the predicate is
   stateless. Under (2), each occurrence is a short burst of commands between one
   cost-payment and the next sweep, and the same Equipment re-equipping a new creature each
   time (via `Effect::AttachEquipment`, `effects/mod.rs:6079`) produces exactly the observed
   "same attacher, three different dead targets, three different turns" shape.
   **INFERENCE**, from the counts in the brief and the mechanism in (2). I did not run the
   fuzzer and I did not read a replay.

   Candidates the source supports for the same pattern, ranked, each marked:
   * **(i) cost-payment sacrifice with no sweep before the invariant runs** — mechanism
     ESTABLISHED (the nine sweep sites, the zero counts in `abilities.rs`/`casting.rs`/
     `mana.rs`), attribution to these seeds **INFERENCE**.
   * **(ii) the §1.2 blind spot (b) via a subtype-stripping layer effect** — the mechanism
     is ESTABLISHED, a corpus route is **not**; and it predicts a violation count an order
     of magnitude larger than observed, which argues against it being the dominant class.
   * **(iii) phased-out attacher, blind spot (a)** — mechanism ESTABLISHED, reachability
     **INFERENCE-against** (CR 702.26c phases the host out too, `turn_actions.rs:1133-1137`).
   * **(iv) an early return inside `resolve_top_of_stack_inner` that skips the
     `resolution.rs:8724` sweep** — `OOS-DX54-5` records four such early returns and says
     the workspace cannot tell whether a fifth was added correctly. I did not audit them.
     **UNMEASURED**, listed because it is the one candidate that could put a dangle past a
     *resolution*, which (i) cannot.

### 1.4 Who can be an attacher that is neither an Aura nor an Equipment/Fortification?

`rg -n "attached_to\s*=\s*Some|attached_to:\s*Some"` over `crates/engine/src/` and
`crates/card-types/src/` returns **five** hits. One is test-only:

* `layers.rs:3883` — inside `#[cfg(test)] mod pb_dx39_source_view_tests`
  (module opens at `layers.rs:3838-3839`). **Not production.** ESTABLISHED.

The four production writers:

| Site | Effect | Guarantees about the **SOURCE**'s subtype |
|------|--------|--------------------------------------------|
| `resolution.rs:2040` | Aura permanent-resolution attach (CR 303.4a/b) | `resolution.rs:2015-2026`: **RAW** `o.characteristics.card_types.contains(Enchantment) && subtypes.contains("Aura")`. Guarantees the *printed* subtype only; a layer effect that strips it does not stop the attach and does stop the SBA. |
| `effects/mod.rs:2065` | CR 702.92a living-weapon-style attach to the first created token | `effects/mod.rs:2050-2054` checks only that the source is on the battlefield and `is_phased_in()`. The comment above it says *"Verify source is still on the battlefield and is an Equipment"* — **the code does not check "is an Equipment" at all.** |
| `effects/mod.rs:6079` | `Effect::AttachEquipment` (CR 702.6a equip) | **NONE.** I read `effects/mod.rs:6007-6090` in full. It validates `equip_id != target_id` (CR 301.5c), then validates the **target**: on the battlefield, phased in, controlled by `ctx.controller` (`:6032-6041`) and layer-resolved `card_types.contains(Creature)` (`:6042-6055`). There is **no check that the source carries the `Equipment` subtype, and no check that the source is on the battlefield.** This is the brief's question 4 answer: **only the target is verified.** |
| `effects/mod.rs:6219` | `Effect::AttachFortification` (CR 702.67a) | Partial: `effects/mod.rs:6146-6163` rejects a source whose layer-resolved `card_types` contain `Creature` (CR 301.6). **No `Fortification` subtype check**, and no battlefield check on the source. |

**ESTABLISHED.** Note the direction of the gap: three of the four sites will happily set
`attached_to` on an object that the CR 704.5n / CR 704.5m arms will then refuse to look at,
because the arms key on the layer-resolved subtype and the attach sites key on nothing (or
on the raw subtype). That is the same shape as the `resolution.rs:2040` asymmetry in §1.2.

There is also a CR-level reason to expect such objects to exist: **CR 301.5f** — *"An
ability of a permanent that refers to the 'equipped creature' refers to whatever creature
that permanent is attached to, even if the permanent with the ability isn't an Equipment."*
— and **CR 303.4m**, the identical sentence for "enchanted". The CR contemplates an
attached permanent that is neither; **CR 704.5m and CR 704.5n do not cover it**, because
their antecedents are literally *"If an **Aura** is attached to…"* and *"If an **Equipment
or Fortification** is attached to…"*. See §1.5.

### 1.5 CR 400.7 / CR 704.5 — verbatim, and the branching a fix must do

**CR 400.7** (verbatim): *"An object that moves from one zone to another becomes a new
object with no memory of, or relation to, its previous existence. This rule has the
following exceptions."*

**CR 704.5m** (verbatim): *"If an Aura is attached to an illegal object or player, or is
not attached to an object or player, that Aura is put into its owner's graveyard."*

**CR 704.5n** (verbatim): *"If an Equipment or Fortification is attached to an illegal
permanent or to a player, it becomes unattached from that permanent or player. It remains
on the battlefield."*

> **These prescribe OPPOSITE dispositions for the same input.** An Aura pointing at a dead
> id goes to the **graveyard** (and its owner's graveyard, and it is a zone change, so it
> fires leaves-the-battlefield triggers and CR 400.7 mints a new id). An Equipment pointing
> at a dead id merely becomes **unattached** and **stays on the battlefield**. Any single
> "clear the dangling pointer" repair is wrong for one of the two. A fix must branch on the
> attacher's type, exactly as `sba.rs` already does with two separate functions.

Supporting rules for the "no longer exists" case specifically:

* **CR 303.4c**: *"If an Aura is enchanting an illegal object or player as defined by its
  enchant ability and other applicable effects, **the object it was attached to no longer
  exists**, or the player it was attached to has left the game, the Aura is put into its
  owner's graveyard. (This is a state-based action. See rule 704.)"* — the nonexistent case
  is named explicitly for Auras.
* **CR 301.5c**: *"…An Equipment that equips an illegal or **nonexistent** permanent becomes
  unattached from that permanent but remains on the battlefield. (This is a state-based
  action. See rule 704.)"* — named explicitly for Equipment.
* **CR 301.6**: *"Rules 301.5a–f apply to Fortifications in relation to lands just as they
  apply to Equipment in relation to creatures…"* — so 301.5c's nonexistent clause covers
  Fortifications by reference.

**Is there a rule for an attached permanent that is NEITHER?** Checked: **no.**
CR 303.4h says *"If an effect attempts to put a permanent that isn't an Aura, Equipment, or
Fortification onto the battlefield attached to an object or player, it enters the
battlefield unattached"* — that is an entry-time rule, not a cleanup rule. CR 301.5c's
*"An Equipment that loses the subtype 'Equipment' can't equip a creature"* states the
illegality but routes the cleanup through 704.5n, whose own antecedent then excludes the
object. **CR 704.5m/704.5n have no residual arm, and CR 301.5f / CR 303.4m explicitly
contemplate the state.** So a permanent that becomes attached and then loses the relevant
subtype is a state the CR describes as illegal and provides no state-based action to
clean up. **ESTABLISHED from the four rule texts above; the conclusion that this is a CR
gap rather than my failing to find the rule is INFERENCE.** A fix has to pick a
disposition for it; the engine currently picks "leave it forever", which is what makes
§1.2's blind spot (b) permanent rather than transient.

---

## Q2 — the CR 800.4j reading for `player_consistency`

### 2.1 The two rules, verbatim

**CR 800.4j**: *"If a player leaves the game during their turn, that turn continues to its
completion **without an active player**. If the active player would receive priority,
instead the next player in turn order receives priority, or the top object on the stack
resolves, or the phase or step ends, whichever is appropriate."*

**CR 800.4a** (the priority sentence): *"…**If the player who left the game had priority at
the time they left, priority passes to the next player in turn order who's still in the
game.**"* (The rest of 800.4a is the object-removal procedure — see §2.5.)

### 2.2 Does the engine have any representation of "no active player"?

**No.** `crates/engine/src/state/turn.rs:95`:

```
pub active_player: PlayerId,
```

non-`Option`, beside `pub priority_holder: Option<PlayerId>` on the next line (`turn.rs:96`).
**ESTABLISHED.**

`rg -n "turn\.active_player\s*=" crates/engine/src/` returns **exactly one** write site in
production: `rules/turn_structure.rs:161`, `turn.active_player = next_player;` inside
`advance_turn`. Every other hit is a comparison. **ESTABLISHED: `active_player` is never
reassigned mid-turn, and there is no sentinel value for "nobody".** CR 800.4j's
*"without an active player"* is inexpressible in this state type, so the engine represents
that turn by leaving the departed player's id in the field.

### 2.3 Every site that sets `has_lost` / `has_conceded`, and what departure handling follows

`rg -n "has_lost\s*=\s*true|has_conceded\s*=\s*true" crates/engine/src/`:

| Site | Cause | CR 800.4 handling that follows |
|------|-------|--------------------------------|
| `rules/sba.rs:272` | CR 704.5a life ≤ 0 | initiative (CR 725.4) + monarch (CR 724.4) transfer only — `sba.rs:279-283` |
| `rules/sba.rs:287` | CR 704.5c poison ≥ 10 | same — `sba.rs:294-298` |
| `rules/sba.rs:311` | CR 704.5u commander damage ≥ 21 | same — `sba.rs:317-321` |
| `rules/engine.rs:2716` | mandatory-loop draw inside `enter_step`'s cleanup branch | `check_game_over` only |
| `rules/engine.rs:2763` | mandatory-loop draw inside `enter_step`'s priority branch | `check_game_over` only |
| `rules/engine.rs:2931` | `handle_concede` | the full block — see below |
| `rules/replacement.rs:1061` | a replacement-effect loss | none at the site |
| `rules/abilities.rs:10543` | mandatory-loop draw inside `finish_resumed_flush` | `check_game_over` only |
| `effects/mod.rs:5089` | `Effect::` player-loses | none at the site |

**ESTABLISHED — `check_player_sbas` (`sba.rs:259-322`) performs NO priority repair, NO
active-player handling and NO object removal.** It marks the flag, pushes `PlayerLost`,
transfers initiative and the monarch, and returns.

`repair_departed_priority_holder` (`abilities.rs:10632`) — the brief's premise is
**CONFIRMED**. `rg -n "repair_departed_priority_holder"` gives two call sites and seven
doc/comment mentions:

* `rules/engine.rs:3168` — the tail of `handle_concede`, guarded on `!is_game_over(state)`.
* `rules/abilities.rs:10469` — the tail of `resume_trigger_flush`.

**No SBA-driven loss reaches it.** ESTABLISHED.

**What repairs priority after an SBA-driven loss** is not that function but the
liveness-awareness of the *grant* sites, which is a different and better-placed mechanism:

* `rules/priority.rs:172-193` `grant_priority_to_active_player` — reads `active`, computes
  `active_is_alive`, and on false takes `next_priority_player(state, active)` with the
  comment *"CR 800.4j: 'If the active player would receive priority, instead the next
  player in turn order receives priority.'"* Called from `resolution.rs:379`, `:8743`,
  `:9043` and `combat.rs:1696`.
* `rules/priority.rs:58-83` `next_priority_player` — skips `has_lost || has_conceded`
  (`:70-72`) and skips players already in `players_passed` (`:77-79`).
* `rules/engine.rs:2769-2789` — `enter_step`'s ordinary step grant: `is_alive` check, then
  `next_priority_player` fallback, then `priority_holder = None`.
* `rules/engine.rs:3192-3198` `validate_player_active` — every command from a dead player
  is rejected with `PlayerEliminated`, so a dead active player cannot act even if named.

**ESTABLISHED. This is why the fuzz run's priority-holder arm produced zero hits:** the
grant sites implement CR 800.4a's and CR 800.4j's priority sentence.

> **CORRECTION TO A PREMISE — the brief says the priority-holder arm "produced zero hits in
> the whole run", and that is consistent with the source, but it is NOT because every grant
> site is liveness-aware.** One is not: `rules/engine.rs:2723-2726`, `enter_step`'s
> **cleanup-SBA-round** grant, writes
> `state.turn.priority_holder = Some(active);` unconditionally, with `active =
> state.turn.active_player` taken two lines earlier and no liveness test —
> it calls `priority::grant_initial_priority` (`priority.rs:86-90`), which likewise pushes
> `PriorityGiven { player: active }` with no check. This is **already registered as
> `OOS-DP9-19`** and is named as an exception in `engine.rs:3079`'s own comment:
> *"NOT `enter_step`'s cleanup-SBA-round grant, which is still unconditional
> (OOS-DP9-19)."* ESTABLISHED. So the priority-holder arm has one live route and the fuzz
> simply did not take it in these 20 games.

### 2.4 Is "the active player has lost" permitted by CR 800.4j, or an engine defect?

**The two arms have different dispositions. This is the headline of Q2.**

**Active-player arm — the condition is CR-PERMITTED and the check is asserting something
the CR does not require.** CR 800.4j says in its first sentence that *"that turn continues
to its completion without an active player"*. The engine has no "without an active player"
representation (§2.2), so it necessarily continues the turn with the departed player's id
still in `TurnState::active_player`. Every consequence CR 800.4j actually cares about is
discharged elsewhere: the departed player never receives priority (`priority.rs:172-193`,
`engine.rs:2769-2789`), never acts (`engine.rs:3192-3198`), and never begins another turn
(`turn_structure.rs:155-158`, §2.5). **So `check_player_consistency`'s first arm
(`crates/simulator/src/invariants.rs:412-422`) is testing a representation choice, not a
rules violation.** ESTABLISHED for each of the supporting facts; the verdict "permitted,
not a defect" is a reading of CR 800.4j that I believe is forced by its first sentence, and
I mark the *verdict* as **INFERENCE**.

**Priority-holder arm — the condition is a DEFECT.** CR 800.4a's last sentence is
unconditional and has no "continues without" escape: *"If the player who left the game had
priority at the time they left, priority passes to the next player in turn order who's
still in the game."* There is no state in which a departed player legitimately holds
priority. The engine agrees — that is why four grant sites plus `enter_step`'s step branch
plus `repair_departed_priority_holder` all exist. **So this arm is a correct assertion with
one known live hole (`OOS-DP9-19`, §2.3), and it should stay.** ESTABLISHED.

### 2.5 Is the condition BOUNDED? (CR 800.4k)

**CR 800.4k**: *"If a player who has left the game would begin a turn, that turn doesn't
begin."*

`rules/turn_structure.rs:143-182` `advance_turn` has two branches for choosing
`next_player`:

```
let next_player = if let Some(extra_turn_player) = turn.extra_turns.pop_back() {
    extra_turn_player                                          // :149-152
} else {
    let next = next_player_in_turn_order(state, turn.last_regular_active)
        .ok_or(GameStateError::NoActivePlayers)?;              // :155-156
    turn.last_regular_active = next; next
};
```

* **Normal branch — CR 800.4k IS honoured.** `next_player_in_turn_order`
  (`turn_structure.rs:185-204`) walks turn order and returns only a candidate for which
  `!player.has_lost && !player.has_conceded` (`:197-201`). **ESTABLISHED.**
* **Extra-turn branch — CR 800.4k is NOT honoured.** `turn.extra_turns.pop_back()` at
  `:149` applies **no liveness filter**, and `rg -n "extra_turns"` over
  `crates/engine/src/` shows the queue is written at `resolution.rs:8822` and
  `effects/mod.rs:7663` and read only here — **nothing ever prunes a departed player's
  entry.** So an extra turn queued for a player who subsequently left WOULD begin, with
  `turn.active_player` set to a dead player for the whole turn.
  **ESTABLISHED that no filter exists; INFERENCE that it is reachable** (it needs an
  extra-turn effect to resolve and its recipient to die before the turn arrives).

**So: yes, bounded — on the normal path.** Combined with §2.2's "one write site, inside
`advance_turn`", the active-player condition can persist only for the remainder of the turn
in which the player left, and then the next turn's `advance_turn` picks a live player.
**ESTABLISHED.** This matches the brief's observation that *"each game reports it at
exactly ONE turn number, repeated many times"* — one turn's worth of tracked commands, all
reporting the same state.

**One further bound worth recording:** a **concede** by the active player does *not* even
last the rest of the turn. `handle_concede` at `rules/engine.rs:3109-3140` runs
`if state.turn.active_player == player { … advance_turn … enter_step … }` — it advances the
turn immediately. **ESTABLISHED.** So the observed `player_consistency` hits cannot be
concessions; they must be `has_lost` set by an SBA (`sba.rs:272/287/311`), by a replacement
effect (`replacement.rs:1061`) or by `effects/mod.rs:5089` — none of which advances the
turn or touches `active_player`. **INFERENCE** for the attribution, but it is a strong one:
it is a complete case split over the nine flag-setting sites in §2.3.

**Aside relevant to Q1:** `rg -n "800\.4a"` shows **no site implements CR 800.4a's object
procedure** — a departed player's owned objects do not leave the game and objects they
control are not exiled. Every hit is a comment using 800.4a for the *"eliminated players
are not opponents"* reading (`casting.rs:2071`, `lands.rs:298`, `resolution.rs:1502`,
`:1528`), or `sba.rs:300` / `diagnostics.rs:134` explicitly noting *"Players are never
removed from state.players (CR 800.4a removes their objects, not the player)"* — a comment
that describes a procedure the engine does not run. **ESTABLISHED.** For Q1 this is good
news in one direction (a departure adds no new dangles) and is an unrelated open gap in
another.

---

## Summary of the two headline answers

**Attachment.** The dangle supply is two functions (`state/mod.rs:1632` and `:2146`), and
neither fixes up the other side of the relationship — only `paired_with` and
`replacement_effects`. Two SBA arms clear it, and each is gated on the attacher's
**layer-resolved subtype** and on it being phased in. An attacher survives with a dangling
`attached_to` indefinitely **iff** it is phased out or its layer-resolved subtypes contain
none of `Aura`/`Equipment`/`Fortification`. That blind spot has an established *code-level*
route (three of the four attach sites never check the source's subtype, and the aura attach
site checks the **raw** subtype where the SBA checks the **layer-resolved** one) but no
established corpus route. The observed fuzz volume argues the dominant class is instead the
**transient** one: nine SBA sweep sites, none of them in `abilities.rs`, `casting.rs`,
`mana.rs` or `combat.rs`, so a permanent that leaves the battlefield while paying a cost
dangles across the invariant check and heals at the next step entry or resolution —
`OOS-M11-7`'s recorded shape, one field over.

**Player consistency.** The active-player arm and the priority-holder arm need **different
dispositions**. The active-player condition is what CR 800.4j *describes* — the engine
cannot say "no active player" because `TurnState::active_player` is a bare `PlayerId` with
one write site — and it is bounded to the remainder of that turn on the normal turn-order
path (with `advance_turn`'s **extra-turn branch** an unbounded exception that applies no
liveness filter, CR 800.4k-wrong). The priority-holder condition is a real defect under
CR 800.4a's unconditional last sentence, with exactly one known live route,
`enter_step`'s unconditional cleanup-SBA-round grant, already filed as `OOS-DP9-19`.
