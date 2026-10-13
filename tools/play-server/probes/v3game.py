# One whole game on the v3 play surface (`/?v3`), headless, to the game-over screen
# (scutemob-268 criterion 2; docs/play-surface-v3-brief.md §6).
# Deals a seed through the API (4 players, the server's bots), then plays through the REAL surface:
# keeps the hand with the surface's own button, plays every lit card item, answers every picker,
# attacks with everything when offered, declares real blocks through the v3 block box (click a
# blocker, click its attacker; the list picker's <select> is the fallback and is counted apart),
# otherwise passes — until `GET /api/game` reports `game_over` AND the surface shows its game-over box.
# A concession is never clicked (the driver has no path to the Concede button).
#
# Why a file of its own and not `v3sweep.py --to-game-over`: the sweep's verdict is "reached turn N
# with nothing scrolling" and its driver is deliberately the dumbest one (never blocks, C1/C2 matrix
# numbers depend on it staying that way). A game-over run needs a different verdict (game over or
# COULD-NOT-FINISH), budgets, per-decision accounting from the page's own POSTs, a blocking policy and
# refusal recovery. It REUSES the sweep's page-side helpers and measurement by import (`v3sweep`), so
# "no page scroll / no box scroll / image-or-chip / auto-pass panel / arrow overlay" are the same code.
#
# Checked on EVERY iteration (the sweep's JS_MEASURE): the page never scrolls, no board / cell / auto-pass
# box scrolls in auto mode, every face-up card shows an image or a chip. Over the run: zero page errors;
# every decision the server issued to the human was answered by a 200 `POST /api/game/action` (or the
# mulligan route for the keep) that THE PAGE sent — counted by decision kind; zero given-up pickers, zero
# stuck decisions. An offered action the engine refuses (HTTP 4xx / the surface's error line — e.g. seed
# OOS-DX20b-1, Purphoros offered as an attacker) is closed, recorded with its body, and the driver takes
# the next option: after a refused attack it attacks with nothing, after a refused block it blocks with
# nothing. Refusals are reported, and are a finding about the engine's offer, not a surface failure —
# unless the decision then stays unanswered (that is "stuck", and BAD).
# At the end: the game-over box (`.decision-float`, title "Game over", the sentence naming the API's
# winner, "New game" enabled, no pass button, no card items) and `game_over` from the API must agree.
#
# Blocking policy (the driver's, not a rules claim; keywords are ignored): a blocker is assigned to the
# attacker it does best against — survives and kills > survives > kills (trade); a chump block only when
# the unblocked power would be lethal. One attacker per blocker (CR 509.1b double-blocks are not
# expressible in the picker — C2 LOW).
# Driver guard, reported as `repeat-capped`: the same card item is clicked at most REPEAT_CAP times per
# turn (a free repeatable ability would otherwise be activated forever).
#
# Budgets (exhausting one is COULD-NOT-FINISH with the turn reached — never a pass):
#   --max-turns (default 200 = the server's own halt), --max-iters, --max-seconds.
# Needs a play-server on 127.0.0.1 (never the owner's 3040), one game per server:
#   cargo run -p play-server -- --port 3041 --players 4 --bot heuristic --seed 0
# Run:
#   python3 tools/play-server/probes/v3game.py --port 3041 --seed 3 --viewport 2560x1340 --out /tmp/v3game
#   [--base-url http://127.0.0.1:3041] [--block-scryfall] [--no-block] [--max-turns N] [--max-iters N] [--max-seconds N]
# Exit 0 = game over reached and every check ok; 1 = BAD (a check failed, or the engine halted on an
# error); 2 = COULD-NOT-RUN / COULD-NOT-FINISH (no server, no table, a budget ran out, the server's
# max-turns halt). BAD wins.
import argparse, asyncio, json, os, sys, time
from urllib.parse import urlparse
from playwright.async_api import async_playwright

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import v3sweep as sw  # noqa: E402

REPEAT_CAP = 3         # clicks on the same card item (caption + name) per turn
NO_PROGRESS = 60       # iterations without the server moving (command_count / seq) = stuck
BU = ".v3 .blockui"
BP = ".v3 .blocker-picker"

JS_VIEW = """async () => {
  try {
    const r = await fetch('/api/game');
    if (!r.ok) return { status: r.status, body: (await r.text()).slice(0, 300) };
    return { status: 200, view: await r.json() };
  } catch (e) { return { status: 0, body: String(e) }; }
}"""

# The sweep's picker step, minus the blocker picker (handled with real pointer events) and with a
# way to attack with nothing after a refused attack.
JS_DESELECT_ATTACKERS = """() => {
  const sel = [...document.querySelectorAll('.v3 .picker-area .attacker-toggle.selected')].filter((b) => !b.disabled);
  if (!sel.length) return 0;
  sel[0].click();
  return sel.length;
}"""

JS_BLOCK_STATE = """() => {
  const q = (s) => [...document.querySelectorAll(s)];
  return { picker: q('.v3 .blocker-picker').length, box: q('.v3 .blockui').length,
           mine: q('.v3 .blockui .act.mine[data-blocker]').map((e) => Number(e.dataset.blocker)),
           atts: q('.v3 .blockui .act.attacker[data-attacker]').map((e) => Number(e.dataset.attacker)),
           paths: q('.v3 .blockui svg.arrows path[data-blocker]').map((e) => [Number(e.dataset.blocker), Number(e.dataset.attacker)]),
           selects: q('.v3 .blocker-picker .blocker-row select.block-target').map((e) => e.value),
           confirm: (document.querySelector('.v3 .blocker-picker .picker-actions .confirm')?.textContent || '').replace(/\\s+/g, ' ').trim() };
}"""

JS_OVER = """() => {
  const txt = (e) => ((e && e.textContent) || '').replace(/\\s+/g, ' ').trim();
  const fl = document.querySelector('.v3 .decision-float');
  const r = fl ? fl.getBoundingClientRect() : null;
  const btns = fl ? [...fl.querySelectorAll('.prompt .buttons .btn')].map((b) => [txt(b), !b.disabled]) : [];
  return { float: !!fl, inside: !!r && r.left >= 0 && r.top >= 0 && r.right <= innerWidth + 0.5 && r.bottom <= innerHeight + 0.5 && r.width > 0,
           title: fl ? txt(fl.querySelector('.head .panel-title')) : '', dockTitle: txt(document.querySelector('.v3 .prompt-box > .panel-title')),
           sentence: fl ? txt(fl.querySelector('.prompt .sentence')) : '', buttons: btns,
           items: fl ? fl.querySelectorAll('.act').length : -1, picker: document.querySelectorAll('.v3 .picker-area').length,
           cells: document.querySelectorAll('.v3 .cell:not(.cell-pop)').length,
           flags: [...document.querySelectorAll('.v3 .seat-flag.out')].map((e) => [(e.closest('[data-seat]') || {}).dataset?.seat ?? null, txt(e)]),
           autopass: txt(document.querySelector('.v3 .autopass .status')) };
}"""


def human(v):
    return v["summary"]["human_name"]


def perm_index(v):
    return {p["object_id"]: p for perms in v["state"]["zones"]["battlefield"].values() for p in perms}


def block_action(v):
    d = v.get("decision") or {}
    return next((a for a in d.get("actions", []) if a.get("block") and a["block"]["legal_blocks"]), None)


def choose_blocks(v, a):
    """[(blocker id, attacker id)] by the policy in the header."""
    perms, me = perm_index(v), human(v)
    life = v["state"]["players"][me]["life"]
    num = lambda oid, k: int((perms.get(oid) or {}).get(k) or 0)
    atts = {x["id"] for x in a["block"]["attackers"]}
    mine = {x["object_id"] for x in ((v["state"].get("combat") or {}).get("attackers") or []) if x.get("target") == f"player:{me}"} or atts

    def score(b, t):
        survive, kill = num(b, "toughness") > num(t, "power"), num(b, "power") >= num(t, "toughness") > 0
        return 3 if survive and kill else 2 if survive else 1 if kill else 0
    plan, blocked = [], set()
    for lb in a["block"]["legal_blocks"]:
        b = lb["blocker"]["id"]
        best = max(lb["attackers"], key=lambda t: (score(b, t["id"]), t["id"] not in blocked, num(t["id"], "power")), default=None)
        if best and score(b, best["id"]) >= 1:
            plan.append((b, best["id"]))
            blocked.add(best["id"])
    if sum(num(t, "power") for t in mine if t not in blocked) >= life:      # lethal: chump with what is left
        used = {b for b, _ in plan}
        for lb in a["block"]["legal_blocks"]:
            b = lb["blocker"]["id"]
            if b in used:
                continue
            open_ = [t for t in lb["attackers"] if t["id"] not in blocked] or lb["attackers"]
            t = max(open_, key=lambda t: num(t["id"], "power"))
            plan.append((b, t["id"]))
            blocked.add(t["id"])
    return plan


async def declare_blocks(pg, plan, r):
    """Draw `plan` in the open block stage with real pointer events and press the picker's own button.
    Returns how each pair was made. Raises RuntimeError when a pair cannot be drawn either way."""
    how = []
    for b, t in plan:
        st = await pg.evaluate(JS_BLOCK_STATE)
        if [b, t] in st["paths"]:
            continue
        drawn = False
        if st["box"] and b in st["mine"] and t in st["atts"]:
            await pg.locator(f'{BU} .act.mine[data-blocker="{b}"]').first.click(timeout=2000)
            await pg.wait_for_timeout(80)
            await pg.locator(f'{BU} .act.attacker[data-attacker="{t}"]').first.click(timeout=2000)
            await pg.wait_for_timeout(200)
            drawn = [b, t] in (await pg.evaluate(JS_BLOCK_STATE))["paths"]
        if drawn:
            how.append("box")
            continue
        # the list picker under the box: the row whose select offers this attacker for this blocker
        row = await pg.evaluate("""(a) => { const rows = [...document.querySelectorAll('.v3 .blocker-picker .blocker-row')];
            return rows.findIndex((r) => (r.dataset.blocker ? Number(r.dataset.blocker) === a.b : true)
              && [...r.querySelectorAll('select.block-target option')].some((o) => o.value === String(a.t)) && !r.dataset.v3game); }""", {"b": b, "t": t})
        if row is None or row < 0:
            raise RuntimeError(f"block {b}->{t}: neither the block box nor a picker row offers it ({st})")
        await pg.locator(f"{BP} .blocker-row").nth(row).locator("select.block-target").select_option(str(t), timeout=2000)
        await pg.evaluate("(i) => { document.querySelectorAll('.v3 .blocker-picker .blocker-row')[i].dataset.v3game = '1'; }", row)
        await pg.wait_for_timeout(150)
        how.append("select")
    r["block_pairs_box"] += how.count("box")
    r["block_pairs_select"] += how.count("select")
    await pg.locator(f"{BP} .picker-actions .confirm").click(timeout=2000)
    return how


async def run(browser, a, base, w, h):
    r = {"seed": a.seed, "viewport": f"{w}x{h}", "iters": 0, "final_turn": None, "page_scroll": 0, "box_scroll": 0,
         "box_scroll_incidents": [], "page_scroll_incidents": [], "chipless": 0, "offscreen": 0, "cards": 0, "images": 0,
         "max_cards": 0, "measured": 0, "measure_failed": 0, "page_errors": [], "console_errors": [], "console_ignored": 0,
         "refusals": [], "gave_up": [], "stuck": [], "repeat_capped": [], "pickers": {}, "screenshots": [],
         "game_over": None, "reason": "", "budget": "", "table_iters": 0, "autopass_missing": 0, "autopass_outside": 0,
         "arrows_missing": 0, "arrows_pointer": 0, "arrows_intercept": 0, "sick_seen": 0,
         "decisions": {}, "answered": {}, "unanswered": [], "posts": 0, "posts_non200": [], "keep_posts": 0,
         "attacks_declared": 0, "attackers_sent": 0, "blocks_declared": 0, "block_pairs_box": 0, "block_pairs_select": 0,
         "block_mismatch": [], "block_log": [], "human_lost_before_over": 0, "human_last_decision_turn": None, "over_dom": None, "seconds": 0}
    sick_oids, seen, posts = set(), {}, []          # seen: seq -> kind; posts: the page's own action POSTs
    t0 = time.monotonic()
    ctx = await browser.new_context(viewport={"width": w, "height": h})
    pg = await ctx.new_page()
    pg.on("pageerror", lambda e: r["page_errors"].append(str(e)))

    def on_console(m):
        if m.type != "error":
            return
        where = (m.location or {}).get("url", "") if isinstance(m.location, dict) else ""
        text = m.text + where
        # a refused action is a 4xx the page reports itself (counted from the response below); images are not ours
        if "scryfall" in text.lower() or "card-image" in text or "favicon" in text or "/api/game/action" in text or "status of 4" in m.text:
            r["console_ignored"] += 1
        else:
            r["console_errors"].append(f"{m.text} [{where}]"[:400])
    pg.on("console", on_console)

    async def on_response(resp):
        rq = resp.request
        if rq.method != "POST":
            return
        if rq.url.endswith("/api/game/mulligan"):
            r["keep_posts"] += 1 if resp.status == 200 else 0
            return
        if not rq.url.endswith("/api/game/action"):
            return
        try:
            body = json.loads(rq.post_data or "{}")
        except Exception:  # noqa: BLE001
            body = {}
        entry = {"seq": body.get("seq"), "status": resp.status, "body": body}
        posts.append(entry)
        if resp.status != 200:
            try:
                entry["error"] = (await resp.text())[:400]
            except Exception:  # noqa: BLE001
                entry["error"] = "?"
    pg.on("response", lambda resp: asyncio.ensure_future(on_response(resp)))
    if a.block_scryfall:
        await pg.route("**/*scryfall*", lambda route: route.abort())

    async def view():
        got = await sw.ev(pg, JS_VIEW, default={"status": 0, "body": "evaluate failed"}) or {"status": 0}
        return got.get("view") if got.get("status") == 200 else None, got

    async def shot(name):
        await pg.mouse.move(1, 1)
        path = os.path.join(a.out, name)
        try:
            await pg.screenshot(path=path)
            r["screenshots"].append(name)
        except Exception:  # noqa: BLE001
            pass

    try:
        try:
            await pg.goto(f"{base}/?v3", timeout=10000)
        except Exception as e:
            raise sw.CouldNotRun(f"server unreachable at {base}: {str(e).splitlines()[0]}")
        deal = await sw.ev(pg, sw.JS_DEAL, a.seed, default={"ok": False, "status": 0, "body": "evaluate failed"})
        if not deal.get("ok"):
            raise sw.CouldNotRun(f"POST /api/game failed: HTTP {deal.get('status')} {deal.get('body')}")
        try:
            await pg.reload(timeout=10000)
            await pg.wait_for_selector(".v3 .app", timeout=10000)
        except Exception:
            raise sw.CouldNotRun("page did not render `.v3 .app` within 10 s")

        tag = f"v3game-s{a.seed}-{w}x{h}"
        tried, chooser_tried, steps, gave = {}, {}, {}, set()
        per_turn, attack_refused, block_refused, block_done = {}, set(), set(), set()
        waits = no_float = stuck_run = idle_iters = 0
        last_progress, last_ui_action = None, ""
        first_bad_shot = False
        while True:
            v, raw = await view()
            if v is None:
                waits += 1
                if waits > 20:
                    r["reason"] = f"GET /api/game kept failing: {raw}"
                    break
                await pg.wait_for_timeout(sw.WAIT_MS)
                continue
            turn = v["state"]["turn"]["number"]
            r["final_turn"] = turn
            d = v.get("decision") or {}
            seq, kind, pregame = d.get("seq"), d.get("kind"), bool(v["summary"]["pregame"])
            me = human(v)
            st = {"cc": v["summary"]["command_count"], "seq": seq, "status": 200}
            m = await sw.ev(pg, sw.JS_MEASURE)
            if m:
                if r["measured"] == 0 and not (m["auto"] and m["oppAuto"]):
                    r["reason"] = "auto toggles are not on (v3-auto / v3-oppauto)"
                    break
                r["measured"] += 1
                r["cards"], r["images"] = m["cards"], m["images"]
                r["max_cards"] = max(r["max_cards"], m["cards"])
                r["chipless"], r["offscreen"] = max(r["chipless"], m["chipless"]), max(r["offscreen"], m["offscreen"])
                if m["pageScroll"]:
                    r["page_scroll"] += 1
                    if len(r["page_scroll_incidents"]) < 20:
                        r["page_scroll_incidents"].append({"turn": turn, "iter": r["iters"], "sizes": m["page"]})
                for bx in m["boxes"]:
                    r["box_scroll"] += 1
                    if len(r["box_scroll_incidents"]) < 60:
                        r["box_scroll_incidents"].append({"turn": turn, "iter": r["iters"], "cards": m["cards"], **bx})
                if (m["pageScroll"] or m["boxes"] or m["chipless"]) and not first_bad_shot:
                    first_bad_shot = True
                    await shot(f"{tag}-first-bad-t{turn}.png")
                if m["table"]:
                    r["table_iters"] += 1
                    r["autopass_missing"] += 1 if m["autopass"] != 1 else 0
                    r["autopass_outside"] += 1 if m["autopassOutside"] else 0
                r["arrows_missing"] += 1 if m["overlays"] != 1 else 0
                r["arrows_pointer"] += 1 if any(p != "none" for p in m["overlayPointer"]) else 0
                r["arrows_intercept"] += 1 if m["overlayHits"] else 0
                sick_oids.update(m["sick"])
                r["sick_seen"] = len(sick_oids)
            else:
                r["measure_failed"] += 1

            if v.get("game_over"):
                r["game_over"] = v["game_over"]
                r["human_at_over"] = {k: v["state"]["players"][me][k] for k in ("life", "poison", "has_lost", "has_conceded", "library_size")}
                r["players_at_over"] = {n: {"life": p["life"], "has_lost": p["has_lost"], "library_size": p["library_size"]} for n, p in v["state"]["players"].items()}
                break
            if v["state"]["players"][me]["has_lost"]:
                r["human_lost_before_over"] += 1      # a view with the human out and no game over (what does the surface do then?)
            now = time.monotonic() - t0
            if turn > a.max_turns:
                r["budget"] = f"--max-turns {a.max_turns} reached (turn {turn})"
                break
            if r["iters"] >= a.max_iters:
                r["budget"] = f"--max-iters {a.max_iters} reached at turn {turn}"
                break
            if now > a.max_seconds:
                r["budget"] = f"--max-seconds {a.max_seconds} reached at turn {turn}"
                break
            if seq is None and not pregame:
                waits += 1
                if waits > sw.WAIT_BAIL:
                    r["reason"] = f"no decision and no game over for ~20 s at turn {turn} (human lost: {v['state']['players'][me]['has_lost']})"
                    break
                await pg.wait_for_timeout(sw.WAIT_MS)
                continue
            waits = 0
            r["iters"] += 1
            if seq is not None and seq not in seen:
                seen[seq] = kind
                r["decisions"][kind] = r["decisions"].get(kind, 0) + 1
                r["human_last_decision_turn"] = turn
            progress = (st["cc"], seq)
            idle_iters = 0 if progress != last_progress else idle_iters + 1
            last_progress = progress
            if idle_iters > NO_PROGRESS:
                r["reason"] = f"no progress for {NO_PROGRESS} iterations on decision seq {seq} ({kind}) at turn {turn}; last UI action: {last_ui_action}"
                r["stuck_decision"] = d
                await shot(f"{tag}-stuck-t{turn}.png")
                break
            if a.progress and r["iters"] % a.progress == 0:
                print(f"   … iter {r['iters']} turn {turn} {v['state']['turn']['step']} {kind} cards {m['cards'] if m else '?'} "
                      f"life {v['state']['players'][me]['life']} {round(now)}s", flush=True)

            ui = await sw.ev(pg, sw.JS_UI, default={}) or {}
            if ui.get("error"):                       # the engine refused an offered action: record, close, take the next option
                bad = next((p for p in reversed(posts) if p["status"] != 200), None)
                r["refusals"].append({"turn": turn, "step": v["state"]["turn"]["step"], "seq": seq, "kind": kind, "iter": r["iters"],
                                      "after": last_ui_action, "text": ui["error"][:300], "post": bad})
                if "attack" in last_ui_action.lower():
                    attack_refused.add(seq)
                if "block" in last_ui_action.lower():
                    block_refused.add(seq)
                    block_done.discard(seq)
                await pg.keyboard.press("Escape")
                await pg.wait_for_timeout(120)
                continue
            if not ui.get("float") and not ui.get("picker") and ui.get("primary") is None:
                no_float += 1
                if no_float == 5:
                    await sw.ev(pg, sw.JS_CLICK_MINI, {"sel": ".v3 .dock .mini-btn", "word": "open", "any": True})
                if no_float > 60:
                    r["reason"] = f"decision seq {seq} pending but no `.decision-float` appeared (turn {turn})"
                    break
                await pg.wait_for_timeout(sw.WAIT_MS)
                continue
            no_float = 0

            if ui.get("picker"):
                key = (seq, ui.get("pickerLabel", ""), ui.get("pickerStage", ""))
                steps[key] = steps.get(key, 0) + 1
                stage = (key[2] or "?").split(" · ")[0]
                if steps[key] == 1:
                    if stage not in r["pickers"]:
                        await shot(f"{tag}-picker-{stage.replace(' ', '-').replace('/', '-')}.png")
                    r["pickers"][stage] = r["pickers"].get(stage, 0) + 1
                give_up = steps[key] > sw.PICKER_STEPS
                ba = block_action(v)
                if ba and not give_up and not a.no_block and await pg.locator(BP).count():
                    plan = [] if seq in block_refused else choose_blocks(v, ba)
                    n0 = len(posts)
                    try:
                        await declare_blocks(pg, plan, r)
                    except Exception as e:  # noqa: BLE001
                        r["stuck"].append({"turn": turn, "seq": seq, "kind": kind, "note": f"block stage: {str(e).splitlines()[0][:300]}"})
                        block_refused.add(seq)            # next pass: confirm with nothing rather than loop
                        continue
                    last_ui_action = f"block {plan}"
                    r["block_log"].append({"turn": turn, "seq": seq, "legal": len(ba["block"]["legal_blocks"]), "attackers": len(ba["block"]["attackers"]), "plan": plan})
                    await sw.settle(pg, st)
                    await pg.wait_for_timeout(60)
                    sent = next((p for p in posts[n0:] if p["seq"] == seq), None)
                    got = sorted(map(tuple, ((sent or {}).get("body", {}).get("params") or {}).get("blockers") or []))
                    if sent is None or got != sorted(plan):
                        r["block_mismatch"].append({"turn": turn, "seq": seq, "plan": plan, "posted": (sent or {}).get("body")})
                    elif sent["status"] == 200 and plan:
                        r["blocks_declared"] += 1
                    continue
                if seq in attack_refused and "attack" in stage.lower():
                    left = await sw.ev(pg, JS_DESELECT_ATTACKERS, default=0)
                    if left:
                        await pg.wait_for_timeout(50)
                        continue
                res = await sw.ev(pg, sw.JS_PICKER_STEP, {"giveUp": give_up, "attackAll": seq not in attack_refused}, default={}) or {}
                did = res.get("did", "nothing")
                if give_up:
                    if not did.startswith("back"):
                        await pg.keyboard.press("Escape")
                    gave.add((seq, key[1]))
                    r["gave_up"].append({"turn": turn, "seq": seq, "kind": kind, "label": key[1], "stage": key[2]})
                    await pg.wait_for_timeout(100)
                elif res.get("submits"):
                    last_ui_action = f"picker {stage}: {did}"
                    if did.startswith("confirm: Attack with") and "nothing" not in did:
                        r["attacks_declared"] += 1
                        r["attackers_sent"] += int("".join(ch for ch in did.split("Attack with")[-1] if ch.isdigit()) or 0)
                    await sw.settle(pg, st)
                else:
                    await pg.wait_for_timeout(60)
                continue
            if ui.get("chooser"):
                done = chooser_tried.setdefault(seq, [])
                picked = await sw.ev(pg, sw.JS_CLICK_CHOOSER, done)
                if picked is None:
                    await pg.keyboard.press("Escape")
                    await pg.wait_for_timeout(80)
                else:
                    done.append(picked)
                    last_ui_action = f"chooser: {picked}"
                    await sw.settle(pg, st)
                continue
            if ui.get("note"):
                await pg.keyboard.press("Escape")
                await pg.wait_for_timeout(80)
            if kind == "Priority" and not pregame:
                done = tried.setdefault(seq, [])
                counts = per_turn.setdefault(turn, {})
                item = await sw.ev(pg, sw.JS_TAG_ITEM, done + [k for k, n in counts.items() if n >= REPEAT_CAP])
                if item is not None:
                    done.append(item)
                    counts[item] = counts.get(item, 0) + 1
                    if counts[item] == REPEAT_CAP:
                        r["repeat_capped"].append({"turn": turn, "item": item})
                    last_ui_action = f"item: {item}"
                    try:
                        await pg.locator('[data-v3sweep="1"]').first.click(timeout=1500)
                    except Exception:
                        await sw.ev(pg, "() => { const n = document.querySelector('[data-v3sweep]'); if (n) n.click(); }")
                    await sw.settle(pg, st)
                    continue
            # a declare decision whose picker is not open (an instant is offered beside it, or the stage was
            # backed out of after a refusal): its own "Declare …" button comes before the pass button
            skip = [label for (s, label) in gave if s == seq]
            clicked = None
            if kind in ("DeclareAttackers", "DeclareBlockers") and not skip:
                clicked = await sw.ev(pg, """() => { const b = [...document.querySelectorAll('.v3 .decision-float .prompt .buttons .btn.act, .v3 .prompt .buttons .btn.act')]
                    .find((n) => !n.disabled && /^Declare /.test((n.textContent || '').trim())); if (!b) return null; b.click(); return (b.textContent || '').replace(/\\s+/g, ' ').trim(); }""")
            if clicked is None:
                clicked = await sw.ev(pg, sw.JS_CLICK_BUTTON, skip)
            if clicked is None:
                stuck_run += 1
                r["stuck"].append({"turn": turn, "seq": seq, "kind": kind, "note": "no primary / act button to click; pressed Escape"})
                await pg.keyboard.press("Escape")
                if stuck_run > 12:
                    r["reason"] = f"stuck on decision seq {seq} ({kind}) at turn {turn}"
                    r["stuck_decision"] = d
                    await shot(f"{tag}-stuck-t{turn}.png")
                    break
                await pg.wait_for_timeout(sw.WAIT_MS)
                continue
            if "concede" in clicked.lower():
                r["reason"] = f"the driver clicked {clicked!r} — a concession is not a way to reach game over"
                break
            stuck_run = 0
            last_ui_action = f"button: {clicked}"
            await sw.settle(pg, st)

        # ---- the end of the run
        await pg.wait_for_timeout(600)
        answered = {}
        for p in posts:
            if p["status"] == 200 and p["seq"] in seen:
                answered[p["seq"]] = seen[p["seq"]]
        for k in answered.values():
            r["answered"][k] = r["answered"].get(k, 0) + 1
        r["posts"] = len(posts)
        r["posts_non200"] = [p for p in posts if p["status"] != 200][:40]
        r["posts_non200_count"] = sum(1 for p in posts if p["status"] != 200)
        r["unanswered"] = [{"seq": s, "kind": k} for s, k in seen.items() if s not in answered][:40]
        if r["game_over"]:
            for _ in range(40):                       # the page learns of the end from its own last response
                o = await sw.ev(pg, JS_OVER, default={}) or {}
                if o.get("float") and o.get("title") == "Game over":
                    break
                await pg.wait_for_timeout(100)
            r["over_dom"] = o
            end = await sw.ev(pg, sw.JS_MEASURE)
            r["over_measure"] = None if not end else {"pageScroll": end["pageScroll"], "boxes": end["boxes"], "chipless": end["chipless"], "cards": end["cards"], "images": end["images"]}
            await shot(f"{tag}-game-over.png")
            if await sw.ev(pg, sw.JS_CLICK_MINI, {"sel": ".v3 .decision-float .mini-btn", "word": "minimise", "any": True}):
                await pg.wait_for_timeout(200)
                await shot(f"{tag}-game-over-table.png")
    finally:
        r["seconds"] = round(time.monotonic() - t0, 1)
        await ctx.close()
    return r


def verdict(r):
    """(bad, could_not_finish): two lists of reasons."""
    bad, cnf = [], []
    over = r["game_over"]
    if r["reason"]:
        bad.append(r["reason"])
    if r["budget"]:
        cnf.append(f"COULD-NOT-FINISH: {r['budget']}")
    if over and over.get("halted"):
        (cnf if "turn" in str(over.get("reason", "")).lower() and "error" not in str(over.get("reason", "")).lower() else bad).append(f"the server halted the game: {over.get('reason')}")
    if not over and not r["reason"] and not r["budget"]:
        bad.append(f"stopped at turn {r['final_turn']} without game over")
    for key, name in (("page_scroll", "page-scroll"), ("box_scroll", "box-scroll"), ("chipless", "chipless"), ("measure_failed", "measure-failed"),
                      ("autopass_missing", "autopass-panel-missing"), ("autopass_outside", "autopass-outside-viewport"),
                      ("arrows_missing", "arrow-overlay-missing"), ("arrows_pointer", "arrow-overlay-takes-pointer"), ("arrows_intercept", "arrow-overlay-on-top")):
        if r[key]:
            bad.append(f"{name} {r[key]}")
    if r["page_errors"]:
        bad.append(f"page-errors {len(r['page_errors'])}")
    if r["gave_up"]:
        bad.append(f"gave-up {len(r['gave_up'])}")
    if r["stuck"]:
        bad.append(f"stuck {len(r['stuck'])}")
    if r["block_mismatch"]:
        bad.append(f"block POST differs from what was drawn: {len(r['block_mismatch'])}")
    if not r["measured"] or not r["max_cards"]:
        bad.append(f"nothing measured (measures {r['measured']}, most cards seen {r['max_cards']})")
    if r["keep_posts"] != 1:
        bad.append(f"the keep was not one page POST to the mulligan route ({r['keep_posts']})")
    # every decision answered through the UI. When the run ended on a budget the last decision is legitimately open.
    open_ok = 1 if (r["budget"] or r["reason"]) else 0
    if len(r["unanswered"]) > open_ok:
        bad.append(f"decisions without a 200 POST from the page: {len(r['unanswered'])} {r['unanswered'][:5]}")
    if over and not over.get("halted"):
        o = r["over_dom"] or {}
        want = f"Game over — {over['winner']} wins." if over.get("winner") else "Game over — no winner."
        labels = [b[0] for b in o.get("buttons", [])]
        if not (o.get("float") and o.get("inside")):
            bad.append(f"game over: no decision box inside the viewport ({o})")
        if o.get("title") != "Game over":
            bad.append(f"game over: box title is {o.get('title')!r}")
        if o.get("sentence") != want:
            bad.append(f"game over: the box says {o.get('sentence')!r}, the API says {want!r}")
        if not any(l.startswith("New game") and en for l, en in o.get("buttons", [])):
            bad.append(f"game over: no enabled New game button ({labels})")
        if any("pass" in l.lower() or "concede" in l.lower() for l in labels) or o.get("items") or o.get("picker"):
            bad.append(f"game over: the box still offers play ({labels}, {o.get('items')} card items, {o.get('picker')} pickers)")
        em = r.get("over_measure")
        if not em or em["pageScroll"] or em["boxes"] or em["chipless"]:
            bad.append(f"game over: the end screen scrolls or was not measured ({em})")
    return bad, cnf


def table(r):
    over = r["game_over"] or {}
    rows = [
        ("seed / viewport", f"{r['seed']} / {r['viewport']}"),
        ("ended", ("game over, turn %s, %s commands" % (over.get("turn_count"), over.get("total_commands"))) if over else f"NOT over (turn {r['final_turn']})"),
        ("winner", over.get("winner") if over else "-"),
        ("human at the end", json.dumps(r.get("human_at_over")) if over else "-"),
        ("human's last decision on turn", r["human_last_decision_turn"]),
        ("views with the human out and no game over", r["human_lost_before_over"]),
        ("iterations / wall-clock", f"{r['iters']} / {r['seconds']} s"),
        ("decisions issued by kind", json.dumps(r["decisions"], sort_keys=True)),
        ("answered by a page POST (200)", json.dumps(r["answered"], sort_keys=True)),
        ("unanswered", len(r["unanswered"])),
        ("page action POSTs / non-200", f"{r['posts']} / {r.get('posts_non200_count', 0)}"),
        ("pickers opened by stage", json.dumps(r["pickers"], sort_keys=True)),
        ("attacks declared / attackers sent", f"{r['attacks_declared']} / {r['attackers_sent']}"),
        ("block decisions with a legal block / blocks declared / pairs by box / by select", f"{len(r['block_log'])} / {r['blocks_declared']} / {r['block_pairs_box']} / {r['block_pairs_select']}"),
        ("refusals / gave-up / stuck / repeat-capped", f"{len(r['refusals'])} / {len(r['gave_up'])} / {len(r['stuck'])} / {len(r['repeat_capped'])}"),
        ("page scroll / box scroll / chipless (iterations)", f"{r['page_scroll']} / {r['box_scroll']} / {r['chipless']} of {r['measured']} measured"),
        ("most cards on screen / at the end with image", f"{r['max_cards']} / {r['cards']} with {r['images']}"),
        ("page errors / console errors (ignored)", f"{len(r['page_errors'])} / {len(r['console_errors'])} ({r['console_ignored']})"),
        ("summoning-sick outlines seen", r["sick_seen"]),
        ("game-over box", json.dumps({k: (r["over_dom"] or {}).get(k) for k in ("title", "sentence", "buttons", "flags")})),
    ]
    width = max(len(k) for k, _ in rows)
    return "\n".join(f"   {k:<{width}}  {v}" for k, v in rows)


async def main():
    ap = argparse.ArgumentParser(description="One whole game on the v3 play surface, to the game-over screen.")
    ap.add_argument("--port", type=int, default=3041)
    ap.add_argument("--base-url", default="", help="e.g. http://127.0.0.1:3041 (overrides --port)")
    ap.add_argument("--seed", type=int, default=3)
    ap.add_argument("--viewport", default="2560x1340")
    ap.add_argument("--out", default=".")
    ap.add_argument("--max-turns", type=int, default=200, help="the server halts a game at 200 turns itself")
    ap.add_argument("--max-iters", type=int, default=12000)
    ap.add_argument("--max-seconds", type=int, default=3600)
    ap.add_argument("--no-block", action="store_true", help="never declare a blocker (the sweep's driver)")
    ap.add_argument("--block-scryfall", action="store_true")
    ap.add_argument("--progress", type=int, default=100, help="print a line every N iterations (0 = never)")
    a = ap.parse_args()
    base = a.base_url.rstrip("/") or f"http://127.0.0.1:{a.port}"
    port = urlparse(base).port or 80
    if port == 3040:
        print("COULD-NOT-RUN 3040 is the owner's server"); sys.exit(2)
    w, h = (int(x) for x in a.viewport.lower().split("x"))
    os.makedirs(a.out, exist_ok=True)
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path="/usr/bin/chromium")
        try:
            r = await run(browser, a, base, w, h)
        except sw.CouldNotRun as e:
            print(f"COULD-NOT-RUN s{a.seed} {w}x{h}: {e}", flush=True)
            await browser.close()
            sys.exit(2)
        await browser.close()
    bad, cnf = verdict(r)
    r["status"] = "BAD" if bad else "COULD-NOT-FINISH" if cnf else "ok"
    r["why"] = bad + cnf
    with open(os.path.join(a.out, f"v3game-s{a.seed}-{w}x{h}.json"), "w") as f:
        json.dump(r, f, indent=1)
    print(table(r), flush=True)
    for x in r["refusals"][:10]:
        print(f"   refusal: turn {x['turn']} {x['step']} {x['kind']} after {x['after']!r}: {x['text']!r} {json.dumps(x['post'])[:300]}")
    for x in (r["box_scroll_incidents"][:5] + r["page_scroll_incidents"][:5] + r["gave_up"][:5] + r["stuck"][:5] + r["block_mismatch"][:5]):
        print(f"   incident: {json.dumps(x)[:400]}")
    for x in r["page_errors"][:5] + r["console_errors"][:5]:
        print(f"   error: {x[:300]}")
    print(("BAD  " if bad else "COULD-NOT-FINISH " if cnf else "ok   ") + f"v3game s{a.seed} {w}x{h}: "
          + (f"game over on turn {r['game_over']['turn_count']}, winner {r['game_over']['winner']}" if r["game_over"] else f"turn {r['final_turn']}")
          + (f"  <- {'; '.join(bad + cnf)}" if bad or cnf else ""), flush=True)
    sys.exit(1 if bad else 2 if cnf else 0)


if __name__ == "__main__":
    asyncio.run(main())
