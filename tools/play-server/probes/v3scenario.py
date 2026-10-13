# Scenario scout for the v3 play-surface probes (`v3interact.py`'s C2 section, scutemob-267).
# The acceptance line is "each interaction exercised headless on a REAL decision", and a greedy
# ten-turn game does not reliably contain one (C1's driver never blocked, never targeted). This
# module finds those decisions deterministically, over the HTTP API only (no browser):
#   an API client      deal / keep / view / act — any non-200 raises Refused (never papered over)
#   human policies     POLICIES: "lands" (play a land, else pass), "develop" (land, then the first
#                      plain creature spell offered, else pass); both declare no attackers and no
#                      blockers and answer a blocking decision (discard, search, scry, trigger
#                      targets …) with the server's own default, or drop the scenario (Dropped)
#   drive_until        deal by seed, keep, apply the policy until a predicate holds on the seat view
#   advance_until      the same from the game as it stands (no re-deal)
#   predicates         SCENARIOS: targeted action / real block / auto-pass stops / summoning
#                      sickness / rearranging — each a function of the seat view alone
#   CHOSEN             the (seed, policy, steps) picked per scenario on 2026-10-12, with what was
#                      seen. The PREDICATE is the stop; `steps` is documentation and a sanity bound
#                      (`reach()` refuses to run far past it), never the stop itself.
# The bots act inside the request that carries the human's answer and are seeded by the deal, so
# the same seed + the same policy must give the same decision at the same step. `--verify` proves
# that for every CHOSEN entry (two fresh drives, identical step count and identical seat view) and
# says so loudly if it ever stops being true — the probes are built on it.
# Needs a play-server on 127.0.0.1 (never the owner's 3040):
#   cargo run -p play-server -- --port 3041 --players 4 --bot heuristic --seed 0
# Run:  python3 tools/play-server/probes/v3scenario.py --port 3041 --seeds 0-30     (scan + verify)
#       python3 tools/play-server/probes/v3scenario.py --port 3041 --verify         (CHOSEN only)
# Exit 0 = every scenario reachable and deterministic, 1 = a CHOSEN scenario is not reached or not
# deterministic (BAD), 2 = COULD-NOT-RUN (no server).
import argparse, hashlib, json, sys, urllib.error, urllib.request

OWNER_PORT = 3040
MAX_STEPS = 700          # a scan gives up on a (seed, policy) after this many human answers


# ---------------------------------------------------------------- API client
class Refused(Exception):
    """The server answered non-200. A scenario that meets one is dropped, not retried."""

    def __init__(self, path, status, body):
        super().__init__(f"{path} -> HTTP {status}: {str(body)[:300]}")
        self.status, self.body = status, body


class Dropped(Exception):
    """The policy met a decision it cannot answer legally; the scenario is not usable."""


def _call(port, path, body=None):
    if port == OWNER_PORT:
        raise SystemExit("refusing port 3040: that is the owner's server")
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}{path}", method="GET" if body is None else "POST",
        data=None if body is None else json.dumps(body).encode(), headers={"content-type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=60) as f:
            return json.loads(f.read())
    except urllib.error.HTTPError as e:
        raise Refused(path, e.code, e.read().decode(errors="replace")) from None


def deal(port, seed, players=4):
    """POST /api/game — a new game (replaces the server's one game). Returns the pregame seat view."""
    return _call(port, "/api/game", {"players": players, "seed": seed})


def keep(port):
    """POST /api/game/mulligan {take:false} — keep the hand as dealt."""
    return _call(port, "/api/game/mulligan", {"take": False})


def view(port):
    """GET /api/game — the seat view. NOTE: `events` are only the lines since the previous response."""
    return _call(port, "/api/game")


def act(port, seq, index, params=None):
    """POST /api/game/action — the body `lib/api.js` `submitAction` sends."""
    return _call(port, "/api/game/action", {"seq": seq, "action_index": index, "params": params or {}})


# ---------------------------------------------------------------- seat-view helpers
def human(v):
    return v["summary"]["human_name"]


def my_battlefield(v):
    return v["state"]["zones"]["battlefield"].get(human(v), [])


def my_hand(v):
    return v["state"]["zones"]["hand"].get(human(v), [])


def opp_battlefield_ids(v):
    return {p["object_id"]: seat for seat, perms in v["state"]["zones"]["battlefield"].items()
            if seat != human(v) for p in perms}


def is_creature(card):
    return "Creature" in (card.get("card_types") or [])


def actions(v):
    return (v.get("decision") or {}).get("actions") or []


def is_plain(a):
    """An action a bare `{}` answers: no picker stage of any kind in front of it."""
    return (not a["target_slots"] and not a["fused_target_slots"] and not a["needs_x"] and not a["modes"]
            and a["costs"] is None and a["decision"] is None and a["attack"] is None and a["block"] is None
            and a.get("order") is None)


def targets_only(a):
    """An action whose ONLY stage is the target picker (what an arrow can answer in one gesture)."""
    return (bool(a["target_slots"]) and not a["needs_x"] and not a["modes"] and a["costs"] is None
            and a["decision"] is None and a["attack"] is None and a["block"] is None)


def candidates(a):
    return [c for slot in a["target_slots"] for c in slot["candidates"]]


def attacks_on_human(v):
    c = v["state"].get("combat")
    return [x for x in (c or {}).get("attackers", []) if x.get("target") == f"player:{human(v)}"]


def bot_spells(events, v):
    return [e for e in events if e["kind"] == "SpellCast" and e.get("player") != human(v)]


# ---------------------------------------------------------------- answering a blocking decision
def _fill(template, **fields):
    """Clone an externally-tagged answer template and write into its single variant (what the pickers do)."""
    answer = json.loads(json.dumps(template))
    if not isinstance(answer, dict) or len(answer) != 1 or not isinstance(next(iter(answer.values())), dict):
        raise Dropped(f"answer template is not a one-key object: {template!r}")
    next(iter(answer.values())).update(fields)
    return answer


def answer_decision(a):
    """Legal params for an action carrying a `decision` (discard / search / scry / trigger targets …):
    the server's own default where it gives one, else the smallest legal answer. Unknown shape -> Dropped."""
    d = a["decision"]
    field, s = d["answer_field"], d["answer"]
    shape = s.get("shape")
    ids = lambda key: [c["id"] for c in s.get(key, [])]
    if shape == "Subset":
        pick = s.get("default") or []
        return {field: sorted(pick if len(pick) == s["count"] else ids("candidates")[:s["count"]])}
    if shape == "PickN":
        pick = s.get("default") or []
        if not (s["min_count"] <= len(pick) <= s["count"]):
            pick = ids("candidates")[:s["min_count"]]
        return {field: _fill(s["template"], **{s["chosen_key"]: sorted(pick)})}
    if shape == "PickOne":
        found = ids("candidates")
        if not found and not s.get("may_decline"):
            raise Dropped("a search with no candidate that may not be declined")
        return {field: _fill(s["template"], **{s["found_key"]: found[0] if found else None})}
    if shape == "Partition":
        return {field: _fill(s["template"], **{s["kept_key"]: ids("looked_at"), s["moved_key"]: []})}
    if shape == "Slots":
        if s.get("default") and len(s["default"]) == len(s["slots"]):
            return {field: s["default"]}
        return {field: [[c["value"] for c in slot["candidates"][:slot["min"]]] for slot in s["slots"]]}
    if shape == "Confirm":
        return {field: _fill(s["template"], **{s["pay_key"]: bool(s.get("default", False))})}
    if shape == "BinaryChoice":
        return {field: _fill(s["template"], **{s["choice_key"]: bool(s.get("default", False))})}
    raise Dropped(f"no legal answer known for a {shape!r} decision ({d.get('prompt')!r})")


# ---------------------------------------------------------------- human policies
def _common(v):
    """What every policy does off a `Priority` decision: no attackers, no blockers, default answers."""
    acts = actions(v)
    for a in acts:
        if a["attack"] is not None:
            return a["index"], {"attackers": []}
        if a["block"] is not None:
            return a["index"], {"blockers": []}
    for a in acts:
        if a["decision"] is not None:
            return a["index"], answer_decision(a)
    for a in acts:
        if a["kind"] != "Concede" and is_plain(a):
            return a["index"], {}
    raise Dropped(f"{v['decision']['kind']}: nothing answerable among {[a['kind'] for a in acts]}")


def _pass(v):
    for a in actions(v):
        if a["kind"] == "PassPriority":
            return a["index"], {}
    raise Dropped("a Priority decision without PassPriority")


def policy_pass(v):
    """Pass priority; declare nothing; default answers."""
    return _pass(v) if v["decision"]["kind"] == "Priority" else _common(v)


def policy_lands(v):
    """Play a land when one is offered, else pass. Never casts — mana and cards stay available."""
    if v["decision"]["kind"] != "Priority":
        return _common(v)
    for a in actions(v):
        if a["kind"] == "PlayLand" and is_plain(a):
            return a["index"], {}
    return _pass(v)


def policy_develop(v):
    """Play a land if offered, else cast the first PLAIN creature spell offered (hand or command zone;
    the view carries no mana cost, so "first offered" stands in for "cheapest"; a creature needing a
    picker stage is skipped), else pass. Never attacks, never blocks."""
    if v["decision"]["kind"] != "Priority":
        return _common(v)
    acts = actions(v)
    for a in acts:
        if a["kind"] == "PlayLand" and is_plain(a):
            return a["index"], {}
    z = v["state"]["zones"]
    castable = {c["object_id"] for c in my_hand(v) + z["command_zone"].get(human(v), []) if is_creature(c)}
    for a in acts:
        if a["kind"] == "CastSpell" and a["object_id"] in castable and is_plain(a):
            return a["index"], {}
    return _pass(v)


POLICIES = {"pass": policy_pass, "lands": policy_lands, "develop": policy_develop}


# ---------------------------------------------------------------- driving
class Reached:
    """Where a drive stopped. `steps` = human answers given since the deal (keep not counted);
    `view` = the seat view the predicate held on; `events` = every event line seen on the way;
    `trace` = one row per answer (see `_row`)."""

    def __init__(self, seed, policy, steps, v, events, trace, ok, why=""):
        self.seed, self.policy, self.steps, self.view = seed, policy, steps, v
        self.events, self.trace, self.ok, self.why = events, trace, ok, why

    def digest(self):
        """Stable fingerprint of the stopping point: the whole seat view (events included) minus
        `decision.seq` — the wire seq counts decisions over the SERVER's lifetime, not the game's, so it
        is the one field two identical drives differ in (measured 2026-10-12). Never hard-code a seq."""
        v = json.loads(json.dumps(self.view))
        if v.get("decision"):
            v["decision"]["seq"] = None
        return hashlib.sha256(json.dumps(v, sort_keys=True).encode()).hexdigest()[:16]


def _row(v, index):
    a = next(x for x in actions(v) if x["index"] == index)
    t = v["state"]["turn"]
    return {"turn": t["number"], "active": t["active_player"], "step": t["step"], "decision": v["decision"]["kind"],
            "action": a["kind"], "label": a["label"]}


def advance_until(port, v, predicate, policy, max_steps=MAX_STEPS, seed=None, _steps=0, _events=None, _trace=None):
    """Apply `policy` from seat view `v` until `predicate(view)` holds. Never re-deals. Returns a Reached
    (`.ok` False with `.why` when the game ended, the bound ran out, the policy dropped out, or the
    server refused an answer — a refusal is a scenario failure, never retried)."""
    pol = POLICIES[policy] if isinstance(policy, str) else policy
    name = policy if isinstance(policy, str) else getattr(policy, "__name__", "custom")
    events = list(_events if _events is not None else v.get("events", []))
    trace = _trace if _trace is not None else []
    steps = _steps
    while True:
        if predicate(v):
            return Reached(seed, name, steps, v, events, trace, True)
        if v.get("game_over"):
            return Reached(seed, name, steps, v, events, trace, False, f"game over: {v['game_over']}")
        if not v.get("decision"):
            return Reached(seed, name, steps, v, events, trace, False, "no decision for the human and no game over")
        if steps - _steps >= max_steps:
            return Reached(seed, name, steps, v, events, trace, False, f"not reached in {max_steps} answers")
        try:
            index, params = pol(v)
            row = _row(v, index)
            nxt = act(port, v["decision"]["seq"], index, params)
        except (Dropped, Refused) as e:
            return Reached(seed, name, steps, v, events, trace, False, f"{type(e).__name__}: {e}")
        # what the answer caused, for the auto-pass scout: the bots' whole reply rides on this response
        row["after"] = {"turn": nxt["state"]["turn"]["number"], "active": nxt["state"]["turn"]["active_player"],
                        "step": nxt["state"]["turn"]["step"], "bot_spells": [e["text"] for e in bot_spells(nxt["events"], nxt)],
                        "attacked": [x["name"] for x in attacks_on_human(nxt)]}
        trace.append(row)
        events.extend(nxt.get("events", []))
        steps += 1
        v = nxt


def drive_until(port, seed, predicate, policy, max_steps=MAX_STEPS):
    """Deal `seed`, keep, then `advance_until`. The one way a probe gets to a scenario."""
    try:
        deal(port, seed)
        v = keep(port)
    except Refused as e:
        return Reached(seed, str(policy), 0, {}, [], [], False, f"Refused: {e}")
    return advance_until(port, v, predicate, policy, max_steps, seed=seed)


# ---------------------------------------------------------------- predicates (seat view -> bool)
def _priority(v):
    d = v.get("decision")
    return bool(d) and d["kind"] == "Priority" and not v["summary"]["pregame"]


def target_actions(v, want):
    """The targets-only actions of a Priority decision with a candidate of the wanted sort:
    'opp' = an object on an opponent's battlefield, 'opp+player' = that AND a player candidate (burn),
    'own' = one of the human's own permanents."""
    if not _priority(v):
        return []
    opp, mine = opp_battlefield_ids(v), {p["object_id"] for p in my_battlefield(v)}
    out = []
    for a in actions(v):
        if not targets_only(a):
            continue
        cs = candidates(a)
        objs = [c for c in cs if c["kind"] == "object"]
        hit_opp = any(c["id"] in opp for c in objs)
        ok = {"opp": hit_opp, "opp+player": hit_opp and any(c["kind"] == "player" for c in cs),
              "own": any(c["id"] in mine for c in objs)}[want]
        if ok:
            out.append(a)
    return out


def targeted_opp(v):
    return bool(target_actions(v, "opp"))


def targeted_burn(v):
    return bool(target_actions(v, "opp+player"))


def targeted_own(v):
    return bool(target_actions(v, "own"))


def targeted_from_table(v):
    """`targeted_opp` whose source card is on the table for the human (hand or own battlefield)."""
    on_table = {c["object_id"] for c in my_hand(v) + my_battlefield(v)}
    return any(a["object_id"] in on_table for a in target_actions(v, "opp"))


def block_action(v):
    d = v.get("decision")
    if not d or d["kind"] != "DeclareBlockers":
        return None
    return next((a for a in d["actions"] if a["block"] and a["block"]["legal_blocks"]), None)


def real_block(v):
    return block_action(v) is not None


def sick_creatures(v):
    return [p for p in my_battlefield(v) if is_creature(p) and p["summoning_sick"]]


def fresh_creature(v):
    """The human just cast a creature: it is on the battlefield, summoning sick, on the human's own turn."""
    return _priority(v) and v["state"]["turn"]["active_player"] == human(v) and bool(sick_creatures(v))


def no_longer_sick(oid):
    """The same object, the flag gone, with the human to act (its controller's next turn)."""
    def pred(v):
        return _priority(v) and any(p["object_id"] == oid and not p["summoning_sick"] for p in my_battlefield(v))
    pred.__name__ = f"no_longer_sick({oid})"
    return pred


def land_stacks(v):
    return sorted({p["name"] for p in my_battlefield(v) if "Land" in p["card_types"]})


def rearrange_hand(v):
    """>= 3 cards in hand at a Priority decision that offers a land drop (so one card can leave by the API)."""
    return _priority(v) and len(my_hand(v)) >= 3 and any(a["kind"] == "PlayLand" for a in actions(v))


def rearrange_board(v):
    """>= 2 creatures or >= 2 different land stacks on the human's battlefield, human to act on their own turn."""
    return (_priority(v) and v["state"]["turn"]["active_player"] == human(v)
            and (sum(is_creature(p) for p in my_battlefield(v)) >= 2 or len(land_stacks(v)) >= 2))


def my_second_main(turn):
    """The human's own post-combat main phase of turn `turn`, stack empty: the lands / develop policies
    only pass from here until the human's next turn, so it is where the auto-pass runs start."""
    def pred(v):
        t = v["state"]["turn"]
        return (_priority(v) and t["number"] == turn and t["active_player"] == human(v)
                and t["step"] == "PostCombatMain" and not v["state"]["zones"]["stack"])
    pred.__name__ = f"my_second_main({turn})"
    return pred


def my_turn_after(turn):
    def pred(v):
        t = v["state"]["turn"]
        return t["number"] > turn and t["active_player"] == human(v)
    return pred


def autopass_facts(trace, who):
    """From one drive's trace: for each of the human's post-combat mains, what PASSING ALONE leads to.
    A run is 'pure' while every answer is PassPriority on a Priority decision (no discard, no blockers …).
    Returns {turn: {passes_to_spell, spell, passes_to_attack, attackers, passes_to_my_turn, pure_round, broke_on}}."""
    out = {}
    for i, r in enumerate(trace):
        if not (r["active"] == who and r["step"] == "PostCombatMain" and r["decision"] == "Priority" and r["action"] == "PassPriority"):
            continue
        if r["turn"] in out:
            continue
        f = {"passes_to_spell": None, "spell": None, "passes_to_attack": None, "attackers": None,
             "passes_to_my_turn": None, "pure_round": False, "broke_on": None, "sees_end_step": False}
        for n, x in enumerate(trace[i:], start=1):
            if x["decision"] != "Priority" or x["action"] != "PassPriority":
                f["broke_on"] = f"{x['decision']}/{x['action']} (turn {x['turn']}, {x['active']})"
                break
            aft = x["after"]
            if aft["step"] == "End":
                f["sees_end_step"] = True
            if aft["bot_spells"] and f["passes_to_spell"] is None:
                f["passes_to_spell"], f["spell"] = n, aft["bot_spells"][0]
            if aft["attacked"] and f["passes_to_attack"] is None:
                f["passes_to_attack"], f["attackers"] = n, aft["attacked"]
            if aft["turn"] > r["turn"] and aft["active"] == who:
                f["passes_to_my_turn"], f["pure_round"] = n, True
                break
        out[r["turn"]] = f
    return out


# ---------------------------------------------------------------- the scenarios
# name -> (policies to scan, predicate or predicate factory taking the scenario's `turn`, what it is)
SCENARIOS = {
    "target_burn":   (("lands", "develop"), targeted_burn, "targets-only action: opponent permanent AND player candidates"),
    "target_opp":    (("lands", "develop"), targeted_opp, "targets-only action with a candidate on an opponent's battlefield"),
    "target_table":  (("lands", "develop"), targeted_from_table, "as target_opp, source card in the human's hand / on their board"),
    "target_own":    (("lands", "develop"), targeted_own, "targets-only action with one of the human's own permanents as a candidate"),
    "block":         (("develop",), real_block, "DeclareBlockers with a non-empty legal_blocks"),
    "sick":          (("develop",), fresh_creature, "a creature the human just cast, summoning sick"),
    "rearrange_hand":  (("lands",), rearrange_hand, ">= 3 cards in hand and a land drop offered"),
    "rearrange_board": (("lands", "develop"), rearrange_board, ">= 2 creatures or >= 2 land stacks on the human's battlefield"),
    # the three below stop at my_second_main(turn); what passing leads to is in `autopass_facts`
    "autopass_round":  (("lands", "develop"), my_second_main, "passing alone reaches the human's next turn (no other decision on the way)"),
    "autopass_spell":  (("lands", "develop"), my_second_main, "passing alone makes a bot cast a spell"),
    "autopass_combat": (("lands", "develop"), my_second_main, "passing alone makes a bot declare attackers at the human"),
}
AUTOPASS = ("autopass_round", "autopass_spell", "autopass_combat")

# Chosen 2026-10-12 against HEAD e0fdf3ef (play-server --players 4 --bot heuristic), by `--seeds 0-30`.
# `steps` is where the predicate first held; `turn` parametrises `my_second_main` for the auto-pass three.
CHOSEN = {
    # Cast Skewer the Critics (hand, oid 5) at the human's turn-9 main: candidates are the four players
    # and Bot-4's Cordial Vampire (oid 424). Bot-4's Swamps / Bot-3's Lightning Greaves are non-candidates.
    "target_burn":     {"seed": 10, "policy": "lands", "steps": 71},
    # Cast Pongify (hand, oid 8) with priority in Bot-2's turn 2: one candidate, Bot-2's Phyrexian Walker (407).
    "target_opp":      {"seed": 28, "policy": "lands", "steps": 12},
    "target_table":    {"seed": 28, "policy": "lands", "steps": 12},
    # Deserted Temple (the human's battlefield, oid 402) "{1}, {T}: Untap target land", turn 5: candidates
    # are the Temple itself, the human's Forest (415) and one land of each bot (Island 404, Swamp 406,
    # Plains 408). Bot-4's Paradise Mantle (411) is a card no action uses. Memnite (hand, 6) is castable.
    "target_own":      {"seed": 9, "policy": "lands", "steps": 37},
    # Bot-3's Shambling Ghast (409) attacks the human on turn 11; Tireless Tracker (435) may block it.
    "block":           {"seed": 6, "policy": "develop", "steps": 95},
    # Jaddi Offshoot (oid 405) cast on turn 1, sick; `sick_followup` finds it not sick at step 39 (turn 5).
    "sick":            {"seed": 4, "policy": "develop", "steps": 5},
    # Turn 1, first main: hand of 8 (Zhur-Taa Goblin, Forest, Forest, Clifftop Retreat, Mountain, Plains,
    # Forest, Savannah) and a land drop offered.
    "rearrange_hand":  {"seed": 1, "policy": "lands", "steps": 2},
    # Turn 1: Jaddi Offshoot and Memnite on the human's battlefield (two creatures), one Forest.
    "rearrange_board": {"seed": 4, "policy": "develop", "steps": 7},
    # From the human's turn-1 second main, passing alone: own End step after 1 pass, Bot-2 casts
    # Spidersilk Net after 4, the human's turn 5 after 29, and no other decision on the way.
    "autopass_round":  {"seed": 1, "policy": "lands", "steps": 7, "turn": 1},
    "autopass_spell":  {"seed": 1, "policy": "lands", "steps": 7, "turn": 1},
    # From the human's turn-9 second main, passing alone: Bot-2 casts Goblin Chainwhirler after 4 passes
    # and attacks the human with Phyrexian Walker after 8; the human's turn 13 after 33.
    "autopass_combat": {"seed": 5, "policy": "lands", "steps": 77, "turn": 9},
}


def predicate_for(name, turn=None):
    pred = SCENARIOS[name][1]
    return pred(turn) if name in AUTOPASS else pred


def reach(port, name, slack=25):
    """Re-drive to a CHOSEN scenario. The predicate stops the drive; the recorded step count only bounds
    it (recorded + `slack`) so a drifted game fails here instead of wandering. `.ok` False = COULD-NOT-RUN."""
    c = CHOSEN.get(name)
    if not c:
        return Reached(None, "", 0, {}, [], [], False, f"no CHOSEN entry for {name!r}")
    r = drive_until(port, c["seed"], predicate_for(name, c.get("turn")), c["policy"], c["steps"] + slack)
    if r.ok and r.steps != c["steps"]:
        r.why = f"reached at step {r.steps}, recorded {c['steps']} — the game drifted since the scout ran"
    return r


def autopass_lookahead(port, r, turn):
    """From a reached `my_second_main(turn)` point, PASS (nothing else) to the human's next turn and report
    what happened (`autopass_facts`'s row). Consumes the game: re-`reach` before using the point again."""
    ahead = advance_until(port, r.view, my_turn_after(turn), "pass", 80)
    return autopass_facts(ahead.trace, human(r.view)).get(turn)


def sick_followup(port, r):
    """From a reached `fresh_creature` point, carry on with the same policy until the first sick creature
    is no longer sick. Returns (oid, name, Reached). Consumes the game."""
    p = sick_creatures(r.view)[0]
    return p["object_id"], p["name"], advance_until(port, r.view, no_longer_sick(p["object_id"]), r.policy, 120, _steps=r.steps)


# ---------------------------------------------------------------- CLI: scan, choose, verify
def describe(name, r, turn=None, facts=None):
    """One line of what was seen at the stopping point (card names), for the table and for CHOSEN."""
    v = r.view
    if name.startswith("target"):
        want = {"target_burn": "opp+player", "target_own": "own"}.get(name, "opp")
        a = target_actions(v, want)[0]
        cs = candidates(a)
        return (f"{a['label']} (action {a['index']}, source oid {a['object_id']}); {len(cs)} candidates: "
                + ", ".join(f"{c['kind']}:{c['label']}" for c in cs[:6]) + (" …" if len(cs) > 6 else ""))
    if name == "block":
        b = block_action(v)["block"]
        return "; ".join(f"{x['blocker']['label']} may block {', '.join(y['label'] for y in x['attackers'])}" for x in b["legal_blocks"])
    if name == "sick":
        return "sick: " + ", ".join(f"{p['name']} (oid {p['object_id']})" for p in sick_creatures(v))
    if name == "rearrange_hand":
        return f"hand {len(my_hand(v))}: " + ", ".join(c["name"] for c in my_hand(v))
    if name == "rearrange_board":
        return f"creatures {[p['name'] for p in my_battlefield(v) if is_creature(p)]}, land stacks {land_stacks(v)}"
    if facts:
        return json.dumps(facts)
    return ""


def scan_seed(port, seed, found, turns=30):
    """Every scenario this seed reaches, per policy, from ONE drive per policy (a debug server makes a
    drive cost seconds): the first step each predicate holds at, and the auto-pass facts per human turn.
    Appends (seed, policy, steps, turn, seen) to found[name]."""
    for policy in ("lands", "develop"):
        first, step = {}, [0]

        def watch(v):
            for name, (policies, pred, _) in SCENARIOS.items():
                if name not in AUTOPASS and policy in policies and name not in first and pred(v):
                    first[name] = (step[0], describe(name, Reached(seed, policy, step[0], v, [], [], True)))
            step[0] += 1          # the predicate is asked exactly once per human answer
            return v["state"]["turn"]["number"] > turns
        r = drive_until(port, seed, watch, policy)
        for name, (steps, seen) in first.items():
            found[name].append((seed, policy, steps, None, seen))
        who = human(r.view) if r.view else None
        for turn, f in autopass_facts(r.trace, who).items():
            start = next(i for i, x in enumerate(r.trace) if x["turn"] == turn and x["active"] == who
                         and x["step"] == "PostCombatMain" and x["decision"] == "Priority" and x["action"] == "PassPriority")
            if f["pure_round"] and f["sees_end_step"]:
                found["autopass_round"].append((seed, policy, start, turn, json.dumps(f)))
            if f["passes_to_spell"] and f["pure_round"]:
                found["autopass_spell"].append((seed, policy, start, turn, json.dumps(f)))
            if f["passes_to_attack"]:
                found["autopass_combat"].append((seed, policy, start, turn, json.dumps(f)))


def verify(port, names=None):
    """Two fresh drives per CHOSEN scenario: same step count, same seat view, the recorded step count —
    and, for the scenarios with a second half (what passing leads to; the creature losing its sickness),
    the same second half."""
    bad = 0
    for name, c in CHOSEN.items():
        if names and name not in names:
            continue
        runs = []
        for _ in range(2):
            r = reach(port, name)
            extra = ""
            if r.ok and name in AUTOPASS:
                f = autopass_lookahead(port, r, c["turn"])
                need = {"autopass_round": f and f["pure_round"] and f["sees_end_step"],
                        "autopass_spell": f and f["passes_to_spell"],
                        "autopass_combat": f and f["passes_to_attack"]}[name]
                extra = json.dumps(f)
                if not need:
                    r.ok, r.why = False, f"passing from there does not do it: {extra}"
            if r.ok and name == "sick":
                oid, nm, later = sick_followup(port, r)
                extra = f"{nm} (oid {oid}) sick at step {r.steps}, not sick at step {later.steps} (turn {later.view['state']['turn']['number']})"
                if not later.ok:
                    r.ok, r.why = False, f"{nm} never lost its sickness: {later.why}"
            runs.append((r, extra))
        (a, ax), (b, bx) = runs
        if not (a.ok and b.ok):
            bad += 1
            print(f"BAD  {name}: not reached — {a.why or b.why}")
            continue
        same = a.steps == b.steps and a.digest() == b.digest() and ax == bx
        drift = a.steps != c["steps"]
        if not same:
            print(f"BAD  {name}: NOT DETERMINISTIC — seed {c['seed']} / {c['policy']} gave step {a.steps} view {a.digest()} {ax} "
                  f"then step {b.steps} view {b.digest()} {bx}. THE PROBE DESIGN DEPENDS ON THIS.")
        elif drift:
            print(f"BAD  {name}: deterministic but at step {a.steps}, recorded {c['steps']} (re-run the scan and update CHOSEN)")
        else:
            print(f"ok   {name}: seed {c['seed']} {c['policy']} step {a.steps} view {a.digest()} twice — "
                  + (ax or describe(name, a, c.get("turn"))))
        bad += 0 if (same and not drift) else 1
    return bad


def main():
    ap = argparse.ArgumentParser(description="Find and verify the real decisions the v3 C2 probes act on.")
    ap.add_argument("--port", type=int, default=3041)
    ap.add_argument("--seeds", default="0-30", help="A-B or a,b,c")
    ap.add_argument("--show", type=int, default=8, help="hits listed per scenario")
    ap.add_argument("--verify", action="store_true", help="skip the scan; only re-drive CHOSEN twice")
    a = ap.parse_args()
    if a.port == OWNER_PORT:
        print("COULD-NOT-RUN 3040 is the owner's server"); sys.exit(2)
    try:
        _call(a.port, "/api/healthz")
    except Refused:
        pass
    except Exception as e:  # noqa: BLE001
        print(f"COULD-NOT-RUN no play-server on port {a.port}: {e}"); sys.exit(2)
    if not a.verify:
        lo, _, hi = a.seeds.partition("-")
        seeds = list(range(int(lo), int(hi) + 1)) if hi else [int(s) for s in a.seeds.split(",")]
        found = {name: [] for name in SCENARIOS}
        for seed in seeds:
            scan_seed(a.port, seed, found)
            print(f"scanned seed {seed}", file=sys.stderr, flush=True)
        for name, (_, _, what) in SCENARIOS.items():
            hits = sorted(found[name], key=lambda h: (h[2], h[0]))
            print(f"\n== {name} — {what}: {len(hits)} hit(s) over {len({h[0] for h in hits})} seed(s)")
            for seed, policy, steps, turn, seen in hits[:a.show]:
                print(f"   seed {seed:>2} {policy:<7} step {steps:>3}" + (f" turn {turn}" if turn else "") + f"  {seen}")
        print()
    missing = [n for n in SCENARIOS if n not in CHOSEN]
    bad = verify(a.port)
    for n in missing:
        print(f"BAD  {n}: no CHOSEN entry")
    sys.exit(1 if bad or missing else 0)


if __name__ == "__main__":
    main()
