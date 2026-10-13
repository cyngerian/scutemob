# Headless sweep of the v3 play surface (`/?v3`, docs/play-surface-v3-brief.md §6).
# For each seed x viewport it deals a 4-player game through the API, keeps the hand and
# drives it greedily (play every lit card once, answer every picker, else pass) until the
# target turn has been fully played. On EVERY iteration it checks: the page never scrolls,
# no board/cell box scrolls in auto mode, every face-up card shows an image or a chip, and
# no page error was thrown. Screenshots at turns 1 / 5 / 10 (float open, and minimised).
# C3 (scutemob-268), card images: v3 reads pictures only from the server's local store
# (`GET /api/card-image`). On EVERY run the probe counts every request by host; one request
# to a Scryfall host is BAD (`--block-scryfall` additionally aborts such a request, so a
# regression cannot reach the real service from a probe; the count is taken either way).
# `--store` says what the server's store is expected to hold:
#   filled : every face-up non-token card must show its image; one that still shows the chip
#            must be named in the status route's `failed` list or in `--known-missing`
#            (the saved `/api/card-image/status` JSON of the run that filled the store, or a
#            file of one name per line), or be one of the seat view's placeholder names for a
#            card the seat may not identify (PLACEHOLDER_NAMES: a face-down permanent arrives as
#            a face-up card called "Face-down card"; the client asks the store for that name and
#            gets a 404). Anything else is BAD ("unexplained chip").
#   empty  : no image may load at all (every card is a chip) and every image request is a 404.
#   any    : (default) image or chip, as before; the counts are reported, nothing is asserted.
# The store is the server's: start it with `--image-store <dir> --no-image-prefetch`
# (an empty dir for `--store empty`).
# C2 (scutemob-267) adds, on every iteration: the auto-pass panel (`.autopass`) is a box like
# the others — it must not scroll and must lie inside the viewport; the arrow overlay
# (`svg.v3-arrows`) is there, is `pointer-events: none` and is never the element under a
# point of the page (it must never take a click; that it never makes the page scroll is the
# page-scroll measurement itself). It also counts the summoning-sick outlines it saw
# (`.card.sick`, distinct cards) — a count, not a verdict, so a sweep that never saw one shows.
# The driver stays greedy; the real-decision checks for C2 are in v3interact.py.
# Needs a play-server on 127.0.0.1 (never the owner's 3040):
#   cargo run -p play-server -- --port 3041 --players 4 --bot heuristic --seed 0
# Run:
#   python3 tools/play-server/probes/v3sweep.py --port 3041 --seeds 0,3 \
#       --viewports 2560x1340,1707x880 --turns 10 --out /tmp/v3sweep [--block-scryfall]
#       [--store filled --known-missing fill-status.json | --store empty]
# Exit 0 = every run ok, 1 = a run is BAD, 2 = a run COULD-NOT-RUN (no server / no table).
import argparse, asyncio, json, os, sys
from playwright.async_api import async_playwright

SHOT_TURNS = (1, 5, 10)
PICKER_STEPS = 14      # clicks spent on one picker stage before giving up on it
WAIT_MS = 150          # poll while the bots act
WAIT_BAIL = 134        # ~20 s of no decision

# --- page-side helpers (all tolerant of missing elements) --------------------------------
JS_PRELUDE = """
const txt = (e) => ((e && e.textContent) || '').replace(/\\s+/g, ' ').trim();
const ROOT = document.querySelector('.v3 .decision-float') || document.querySelector('.v3') || document.body;
const PICKER = document.querySelector('.v3 .decision-float .picker-area') || document.querySelector('.v3 .prompt .picker-area');
const live = (sel, scope) => [...(scope || ROOT).querySelectorAll(sel)].filter((b) => !b.disabled);
"""

JS_API = """async () => {
  try {
    const r = await fetch('/api/game');
    if (!r.ok) return { status: r.status };
    const v = await r.json();
    return { status: 200, turn: v?.state?.turn?.number ?? v?.summary?.turn ?? null,
             seq: v?.decision?.seq ?? null, kind: v?.decision?.kind ?? null,
             game_over: v?.game_over ?? null, pregame: !!v?.summary?.pregame,
             cc: v?.summary?.command_count ?? null };
  } catch (e) { return { status: 0, error: String(e) }; }
}"""

JS_DEAL = """async (seed) => {
  try {
    const r = await fetch('/api/game', { method: 'POST', headers: { 'content-type': 'application/json' },
                                         body: JSON.stringify({ players: 4, seed }) });
    return { ok: r.ok, status: r.status, body: r.ok ? '' : (await r.text()).slice(0, 300) };
  } catch (e) { return { ok: false, status: 0, body: String(e) }; }
}"""

JS_UI = "() => {" + JS_PRELUDE + """
  const prim = ROOT.querySelector('.prompt .buttons .btn.primary');
  return { float: !!document.querySelector('.v3 .decision-float'), picker: !!PICKER,
           pickerLabel: PICKER ? txt(PICKER.querySelector('.stage-name')) : '',
           pickerStage: PICKER ? txt(PICKER.querySelector('.stage-step')) : '',
           error: txt(document.querySelector('.v3 .st.error')), note: txt(document.querySelector('.v3 .st.note')),
           chooser: [...document.querySelectorAll('.v3 .st.chooser .mini')].map(txt),
           primary: prim ? txt(prim) : null,
           items: ROOT.querySelectorAll('.group:not(.muted) .act').length };
}"""

# One step on an open picker. arg = {giveUp, attackAll}. Returns {open, label, did, submits}.
JS_PICKER_STEP = "(arg) => {" + JS_PRELUDE + """
  if (!PICKER) return { open: false };
  const out = { open: true, label: txt(PICKER.querySelector('.stage-name')), did: 'nothing' };
  const go = (el, what, submits) => { el.click(); out.did = what + ': ' + txt(el); out.submits = !!submits; return out; };
  if (arg.giveUp) {
    const back = live('.picker-actions .cancel, .action-btn.control, .unknown .btn', PICKER)[0];
    return back ? go(back, 'back') : out;
  }
  // ConfirmPicker (Pay / Decline / Back) and BinaryChoicePicker (two labels / Back): one click answers.
  const yn = live('.confirm-picker .action-btn.decline', PICKER)[0] || live('.binary-choice-picker .action-btn.opt-true', PICKER)[0];
  if (yn) return go(yn, 'choose', true);
  // AttackerPicker: its confirm is always enabled ("Attack with nothing"); select everything first.
  if (arg.attackAll) { const a = live('.attacker-toggle:not(.selected)', PICKER)[0]; if (a) return go(a, 'attacker'); }
  const confirm = PICKER.querySelector('.picker-actions .confirm');
  if (confirm && !confirm.disabled) return go(confirm, 'confirm', true);
  // TargetPicker slots / CostPicker blocks: first box with nothing selected gets its first candidate.
  for (const box of PICKER.querySelectorAll('.slot, .cost-block')) {
    if (box.querySelector('.candidate.selected')) continue;
    const c = live('button.candidate', box)[0];
    if (c) return go(c, 'select');
  }
  // DiscardPicker / SearchPicker candidates, ValuePrompt modes: next unselected one.
  const c = live('button.candidate:not(.selected), button.mode-btn:not(.selected)', PICKER)[0];
  return c ? go(c, 'select') : out;
}"""

# Tag the first untried non-muted card item so Python can give it a real mouse click.
JS_TAG_ITEM = "(tried) => {" + JS_PRELUDE + """
  document.querySelectorAll('[data-v3sweep]').forEach((n) => n.removeAttribute('data-v3sweep'));
  const seen = {};
  for (const act of ROOT.querySelectorAll('.group:not(.muted) .act')) {
    const card = act.querySelector('.card');
    const name = card ? (txt(card.querySelector('.chip')) || card.querySelector('img')?.alt || card.getAttribute('title') || '') : '';
    let key = (txt(act.querySelector('.cap')) || txt(act)) + ' | ' + name;
    seen[key] = (seen[key] || 0) + 1;
    if (seen[key] > 1) key += ' #' + seen[key];
    if (tried.includes(key)) continue;
    (card || act).setAttribute('data-v3sweep', '1');
    return key;
  }
  return null;
}"""

# Click the primary button, else the first `.btn.act` whose label was not given up on.
JS_CLICK_BUTTON = "(skip) => {" + JS_PRELUDE + """
  const given = (t) => skip.some((s) => s && t.startsWith(s));
  for (const b of [...live('.prompt .buttons .btn.primary'), ...live('.prompt .buttons .btn.act')]) {
    const t = txt(b);
    if (b.classList.contains('act') && given(t)) continue;
    b.click();
    return t;
  }
  return null;
}"""

JS_CLICK_CHOOSER = """(tried) => {
  for (const b of document.querySelectorAll('.v3 .st.chooser .mini')) {
    const t = (b.textContent || '').replace(/\\s+/g, ' ').trim();
    if (b.disabled || tried.includes(t)) continue;
    b.click();
    return t;
  }
  return null;
}"""

JS_CLICK_MINI = """(arg) => {
  const b = [...document.querySelectorAll(arg.sel)].find((n) => (n.textContent || '').toLowerCase().includes(arg.word))
         || (arg.any ? document.querySelector(arg.sel) : null);
  if (!b) return false;
  b.click();
  return true;
}"""

JS_MEASURE = """() => {
  const de = document.documentElement, b = document.body;
  const pageScroll = !(de.scrollHeight === innerHeight && de.scrollWidth === innerWidth)
                  || (!!b && (b.scrollHeight > innerHeight || b.scrollWidth > innerWidth));
  const boxes = [];
  for (const sel of ['.v3 .focused', '.v3 .board', '.v3 .cell:not(.cell-pop)', '.v3 .autopass']) {
    document.querySelectorAll(sel).forEach((el, i) => {
      if (el.scrollHeight > el.clientHeight + 1 || el.scrollWidth > el.clientWidth + 1) {
        boxes.push({ box: sel + '[' + i + ']', seat: ((el.querySelector('.seat-name') || {}).textContent || '').trim(),
                     scrollHeight: el.scrollHeight, clientHeight: el.clientHeight,
                     scrollWidth: el.scrollWidth, clientWidth: el.clientWidth });
      }
    });
  }
  const cards = [...document.querySelectorAll('.v3 .card:not(.back)')];
  // C3: face-up non-token cards still on their chip, by name (a token has no printing; a back is hidden)
  const noImage = [...new Set(cards.filter((c) => !c.classList.contains('has-image') && !c.classList.contains('token'))
    .map((c) => c.dataset.name || '?'))];
  const tokens = cards.filter((c) => c.classList.contains('token')).length;
  const chipless = cards.filter((c) => !c.classList.contains('has-image')
    && !(((c.querySelector('.chip') || {}).textContent || '').trim())).length;
  const offscreen = [...document.querySelectorAll('.v3 .card')].filter((c) => {
    if (c.closest('.decision-float, .popover')) return false;
    const r = c.getBoundingClientRect();
    return r.bottom > innerHeight || r.right > innerWidth || r.top < 0 || r.left < 0;
  }).length;
  let auto = true, oppAuto = true;
  try { auto = localStorage.getItem('v3-auto') !== '0'; oppAuto = localStorage.getItem('v3-oppauto') !== '0'; } catch (e) {}
  // C2: the auto-pass panel inside the viewport; the arrow overlay never the thing under the pointer.
  const table = document.querySelectorAll('.v3 .cell:not(.cell-pop)').length > 0;
  const panels = [...document.querySelectorAll('.v3 .autopass')];
  const autopassOutside = panels.filter((p) => { const r = p.getBoundingClientRect();
    return r.width < 1 || r.height < 1 || r.left < 0 || r.top < 0 || r.right > innerWidth + 0.5 || r.bottom > innerHeight + 0.5; }).length;
  const overlays = [...document.querySelectorAll('.v3 svg.v3-arrows')];
  const overlayPointer = overlays.map((o) => getComputedStyle(o).pointerEvents);
  let overlayHits = 0;
  for (const fx of [0.1, 0.3, 0.5, 0.7, 0.9]) for (const fy of [0.1, 0.3, 0.5, 0.7, 0.9]) {
    const e = document.elementFromPoint(innerWidth * fx, innerHeight * fy);
    if (e && e.closest('svg.v3-arrows')) overlayHits += 1;
  }
  const sick = [...new Set([...document.querySelectorAll('.v3 .card.sick')].map((c) => c.dataset.oid || '?'))];
  return { pageScroll, page: [de.scrollWidth, de.scrollHeight, b ? b.scrollWidth : 0, b ? b.scrollHeight : 0, innerWidth, innerHeight],
           boxes, cards: cards.length, images: cards.filter((c) => c.classList.contains('has-image')).length,
           chipless, offscreen, auto, oppAuto, noImage, tokens,
           table, autopass: panels.length, autopassOutside, overlays: overlays.length, overlayPointer, overlayHits, sick };
}"""


class CouldNotRun(Exception):
    pass


async def ev(pg, js, arg=None, default=None):
    """evaluate that never throws (a reload or navigation mid-call returns `default`)."""
    try:
        return await pg.evaluate(js, arg)
    except Exception:
        return default


JS_IMAGE_STATUS = """async () => {
  try { const r = await fetch('/api/card-image/status'); return r.ok ? await r.json() : { http: r.status }; }
  catch (e) { return { error: String(e) }; }
}"""

JS_IMAGE_HTTP = """async (name) => {
  try { return (await fetch('/api/card-image?name=' + encodeURIComponent(name) + '&version=small')).status; }
  catch (e) { return 0; }
}"""


# `crates/view-model/src/redact.rs` FACE_DOWN_NAME / HIDDEN_CARD_NAME: not card names, so never in a store.
PLACEHOLDER_NAMES = {"Face-down card", "Hidden card"}


def is_scryfall(url):
    host = url.split('/')[2].lower() if url.count('/') >= 2 else url.lower()
    return 'scryfall' in host


def load_known_missing(path):
    """Names the store is known not to hold: a saved status JSON (`failed`) or one name per line."""
    if not path:
        return set()
    text = open(path).read()
    try:
        data = json.loads(text)
        return set(data.get("failed", []) if isinstance(data, dict) else data)
    except ValueError:
        return {ln.strip() for ln in text.splitlines() if ln.strip()}


async def api(pg):
    return await ev(pg, JS_API, default={"status": 0, "error": "evaluate failed"}) or {"status": 0}


async def settle(pg, before, ms=2000):
    """Wait until the server moved (command_count / seq) or the page wants an answer; bounded."""
    for _ in range(ms // 80):
        await pg.wait_for_timeout(80)
        ui = await ev(pg, JS_UI, default={}) or {}
        if ui.get("picker") or ui.get("chooser") or ui.get("error") or ui.get("note"):
            return
        now = await api(pg)
        if now.get("status") == 200 and (now.get("cc") != before.get("cc") or now.get("seq") != before.get("seq") or now.get("game_over")):
            return


async def wait_float(pg, want, ms=1500):
    for _ in range(ms // 50):
        if bool((await ev(pg, JS_UI, default={}) or {}).get("float")) == want:
            return True
        await pg.wait_for_timeout(50)
    return False


async def shots(pg, out, tag):
    await pg.mouse.move(1, 1)  # park the pointer: a lingering hover overlay would cover the table
    await pg.wait_for_timeout(120)
    await pg.screenshot(path=os.path.join(out, f"{tag}.png"))
    if await ev(pg, JS_CLICK_MINI, {"sel": ".v3 .decision-float .mini-btn", "word": "minimise", "any": True}) and await wait_float(pg, False):
        await pg.screenshot(path=os.path.join(out, f"{tag}-table.png"))
    await ev(pg, JS_CLICK_MINI, {"sel": ".v3 .dock .mini-btn", "word": "open", "any": True})
    return await wait_float(pg, True)


async def run(browser, a, seed, w, h):
    r = {"seed": seed, "viewport": f"{w}x{h}", "iters": 0, "final_turn": None, "page_scroll": 0, "box_scroll": 0,
         "box_scroll_incidents": [], "page_scroll_incidents": [], "chipless": 0, "offscreen": 0, "cards": 0, "images": 0,
         "page_errors": [], "console_errors": [], "console_ignored": 0, "refusals": [], "gave_up": [], "stuck": [],
         "screenshots": [], "game_over": None, "reason": "",
         # C2: iterations with a table drawn; the auto-pass panel and the arrow overlay measured on them
         "table_iters": 0, "autopass_measured": 0, "autopass_missing": 0, "autopass_outside": 0,
         "arrows_measured": 0, "arrows_missing": 0, "arrows_pointer": 0, "arrows_intercept": 0, "sick_seen": 0,
         # C3: requests by host; image-route answers; the most face-up cards seen at once and how many had a picture
         "hosts": {}, "scryfall_requests": 0, "scryfall_urls": [], "image_200": 0, "image_404": 0, "image_other": 0,
         "peak_cards": 0, "peak_images": 0, "peak_tokens": 0, "images_seen_max": 0,
         "unexplained_chips": {}, "explained_chips": [], "store": a.store, "store_status": None}
    sick_oids = set()
    explained = set()
    ctx = await browser.new_context(viewport={"width": w, "height": h})   # fresh context = no stored toggles
    pg = await ctx.new_page()
    pg.on("pageerror", lambda e: r["page_errors"].append(str(e)))

    def on_console(m):
        if m.type != "error":
            return
        where = (m.location or {}).get("url", "") if isinstance(m.location, dict) else ""
        if "scryfall" in (m.text + where).lower() or "/api/card-image?" in where:   # a store miss is a 404 by design
            r["console_ignored"] += 1
        else:
            r["console_errors"].append(f"{m.text} [{where}]"[:400])
    pg.on("console", on_console)

    def on_request(rq):                                # every request of the context, by host
        host = rq.url.split('/')[2] if rq.url.startswith('http') else rq.url.split(':')[0] + ':'
        r["hosts"][host] = r["hosts"].get(host, 0) + 1
        if is_scryfall(rq.url):
            r["scryfall_requests"] += 1
            if len(r["scryfall_urls"]) < 5:
                r["scryfall_urls"].append(rq.url[:200])

    def on_response(resp):
        if "/api/card-image?" in resp.url:
            r["image_200" if resp.status == 200 else "image_404" if resp.status == 404 else "image_other"] += 1
    ctx.on("request", on_request)
    ctx.on("response", on_response)
    if a.block_scryfall:
        await pg.route(lambda url: is_scryfall(url), lambda route: route.abort())
    base = f"http://127.0.0.1:{a.port}/?v3"
    try:
        try:
            await pg.goto(base, timeout=10000)
        except Exception as e:
            raise CouldNotRun(f"server unreachable on port {a.port}: {str(e).splitlines()[0]}")
        deal = await ev(pg, JS_DEAL, seed, default={"ok": False, "status": 0, "body": "evaluate failed"})
        if not deal.get("ok"):
            raise CouldNotRun(f"POST /api/game failed: HTTP {deal.get('status')} {deal.get('body')}")
        try:
            await pg.reload(timeout=10000)
            await pg.wait_for_selector(".v3 .app", timeout=10000)
        except Exception:
            raise CouldNotRun("page did not render `.v3 .app` within 10 s")

        r["store_status"] = await ev(pg, JS_IMAGE_STATUS, default={"error": "evaluate failed"})
        known_missing = set(a.known_missing) | set((r["store_status"] or {}).get("failed") or []) | PLACEHOLDER_NAMES
        tag = f"v3-s{seed}-{w}x{h}"
        tried, chooser_tried, steps, gave = {}, {}, {}, set()   # keyed by decision seq
        attacked, shot = set(), set()
        waits = no_float = stuck_run = 0
        while True:
            st = await api(pg)
            turn = st.get("turn")
            if turn is not None:
                r["final_turn"] = turn
            m = await ev(pg, JS_MEASURE)
            if m:
                if r["iters"] == 0 and not (m["auto"] and m["oppAuto"]):
                    r["reason"] = "auto toggles are not on (v3-auto / v3-oppauto)"
                    break
                r["measured"] = r.get("measured", 0) + 1
                r["cards"], r["images"] = m["cards"], m["images"]
                r["images_seen_max"] = max(r["images_seen_max"], m["images"])
                if m["cards"] >= r["peak_cards"]:
                    r["peak_cards"], r["peak_images"], r["peak_tokens"] = m["cards"], m["images"], m["tokens"]
                if a.store == "filled":
                    # a chip must be explained: a name the fill could not fetch. A card drawn this
                    # instant gets two more looks before it counts (its picture may be in flight).
                    pending = [n for n in m["noImage"] if n not in known_missing]
                    for _ in range(4):
                        if not pending:
                            break
                        await pg.wait_for_timeout(250)
                        again = await ev(pg, JS_MEASURE) or {}
                        pending = [n for n in pending if n in again.get("noImage", [])]
                    for n in pending:
                        if n not in r["unexplained_chips"]:
                            r["unexplained_chips"][n] = {"turn": turn, "iter": r["iters"], "store_http": await ev(pg, JS_IMAGE_HTTP, n)}
                    explained.update(n for n in m["noImage"] if n in known_missing)
                    r["explained_chips"] = sorted(explained)
                r["max_cards"] = max(r.get("max_cards", 0), m["cards"])
                r["chipless"], r["offscreen"] = max(r["chipless"], m["chipless"]), max(r["offscreen"], m["offscreen"])
                if m["pageScroll"]:
                    r["page_scroll"] += 1
                    if len(r["page_scroll_incidents"]) < 20:
                        r["page_scroll_incidents"].append({"turn": turn, "iter": r["iters"], "sizes": m["page"]})
                for bx in m["boxes"]:
                    r["box_scroll"] += 1
                    if len(r["box_scroll_incidents"]) < 60:
                        r["box_scroll_incidents"].append({"turn": turn, "iter": r["iters"], **bx})
                if m["table"]:                        # C2: only a drawn table has the panel
                    r["table_iters"] += 1
                    r["autopass_measured"] += 1 if m["autopass"] == 1 else 0
                    r["autopass_missing"] += 1 if m["autopass"] != 1 else 0
                    r["autopass_outside"] += 1 if m["autopassOutside"] else 0
                r["arrows_measured"] += 1 if m["overlays"] == 1 else 0
                r["arrows_missing"] += 1 if m["overlays"] != 1 else 0
                r["arrows_pointer"] += 1 if any(p != "none" for p in m["overlayPointer"]) else 0
                r["arrows_intercept"] += 1 if m["overlayHits"] else 0
                sick_oids.update(m["sick"])
                r["sick_seen"] = len(sick_oids)
            else:
                r["measure_failed"] = r.get("measure_failed", 0) + 1   # a metric that was not taken is not a pass
            if st.get("status") != 200:
                waits += 1
                if waits > 20:
                    r["reason"] = f"GET /api/game kept failing: {st}"
                    break
                await pg.wait_for_timeout(WAIT_MS)
                continue
            if st.get("game_over"):
                r["game_over"] = st["game_over"]
                break
            if turn is not None and turn > a.turns:
                break
            if r["iters"] >= a.max_iters:
                r["reason"] = f"iteration cap {a.max_iters} at turn {turn}"
                break
            seq = st.get("seq")
            if seq is None and not st.get("pregame"):
                waits += 1
                if waits > WAIT_BAIL:
                    r["reason"] = f"no decision for ~20 s at turn {turn}"
                    break
                await pg.wait_for_timeout(WAIT_MS)
                continue
            waits = 0
            r["iters"] += 1
            ui = await ev(pg, JS_UI, default={}) or {}
            if ui.get("error"):                       # the engine refused a play: data, not a failure
                r["refusals"].append({"turn": turn, "seq": seq, "iter": r["iters"], "text": ui["error"][:300]})
                await pg.keyboard.press("Escape")
                await pg.wait_for_timeout(120)
                continue
            if not ui.get("float") and not ui.get("picker") and ui.get("primary") is None:
                no_float += 1                         # the server holds a decision the page is not showing yet
                if no_float == 5:
                    await ev(pg, JS_CLICK_MINI, {"sel": ".v3 .dock .mini-btn", "word": "open", "any": True})
                if no_float > 60:
                    r["reason"] = f"decision seq {seq} pending but no `.decision-float` appeared (turn {turn})"
                    break
                await pg.wait_for_timeout(WAIT_MS)
                continue
            no_float = 0
            if turn in SHOT_TURNS and turn not in shot and ui.get("float") and not st.get("pregame"):
                shot.add(turn)
                if not await shots(pg, a.out, f"{tag}-t{turn}"):
                    r["stuck"].append({"turn": turn, "note": "decision float did not come back after the minimised screenshot"})
                r["screenshots"].append(f"{tag}-t{turn}.png")
                continue

            if ui.get("picker"):
                key = (seq, ui.get("pickerLabel", ""), ui.get("pickerStage", ""))
                steps[key] = steps.get(key, 0) + 1
                if steps[key] == 1:                   # which picker stages the drive actually opened
                    if (key[2] or "?") not in r.get("pickers", {}):   # one picture per stage kind, for the eye
                        await pg.mouse.move(1, 1)
                        await pg.screenshot(path=os.path.join(a.out, f"{tag}-picker-{(key[2] or 'unknown').split(' ')[0].replace('/', '-')}.png"))
                    r.setdefault("pickers", {})[key[2] or "?"] = r.setdefault("pickers", {}).get(key[2] or "?", 0) + 1
                give_up = steps[key] > PICKER_STEPS
                res = await ev(pg, JS_PICKER_STEP, {"giveUp": give_up, "attackAll": seq not in attacked}, default={}) or {}
                did = res.get("did", "nothing")
                if give_up:
                    if not did.startswith("back"):
                        await pg.keyboard.press("Escape")
                    gave.add((seq, key[1]))
                    r["gave_up"].append({"turn": turn, "seq": seq, "label": key[1], "stage": key[2]})
                    await pg.wait_for_timeout(100)
                elif res.get("submits"):
                    if did.startswith("confirm: Attack"):
                        attacked.add(seq)             # a refused attack is not re-declared; the pass button follows
                    await settle(pg, st)
                else:
                    await pg.wait_for_timeout(60)
                continue
            if ui.get("chooser"):                     # a clicked card with several options
                done = chooser_tried.setdefault(seq, [])
                picked = await ev(pg, JS_CLICK_CHOOSER, done)
                if picked is None:
                    await pg.keyboard.press("Escape")
                    await pg.wait_for_timeout(80)
                else:
                    done.append(picked)
                    await settle(pg, st)
                continue
            if ui.get("note"):                        # a click message: dismiss, carry on
                await pg.keyboard.press("Escape")
                await pg.wait_for_timeout(80)
            if st.get("kind") == "Priority" and not st.get("pregame"):
                done = tried.setdefault(seq, [])
                item = await ev(pg, JS_TAG_ITEM, done)
                if item is not None:
                    done.append(item)
                    try:
                        await pg.locator('[data-v3sweep="1"]').first.click(timeout=1500)
                    except Exception:
                        await ev(pg, "() => { const n = document.querySelector('[data-v3sweep]'); if (n) n.click(); }")
                    await settle(pg, st)
                    continue
            skip = [label for (s, label) in gave if s == seq]
            clicked = await ev(pg, JS_CLICK_BUTTON, skip)
            if clicked is None:
                stuck_run += 1
                r["stuck"].append({"turn": turn, "seq": seq, "kind": st.get("kind"), "note": "no primary / act button to click; pressed Escape"})
                await pg.keyboard.press("Escape")
                if stuck_run > 12:
                    r["reason"] = f"stuck on decision seq {seq} ({st.get('kind')}) at turn {turn}"
                    break
                await pg.wait_for_timeout(WAIT_MS)
                continue
            stuck_run = 0
            await settle(pg, st)
    finally:
        await ctx.close()
    return r


def verdict(r, turns):
    why = []
    if r["reason"]:
        why.append(r["reason"])
    over = r["game_over"]
    if over and over.get("halted"):
        why.append(f"engine halted: {over.get('reason')}")
    if not over and not r["reason"] and (r["final_turn"] or 0) <= turns:
        why.append(f"stopped at turn {r['final_turn']}")
    for key, name in (("page_scroll", "page-scroll"), ("box_scroll", "box-scroll"), ("chipless", "chipless")):
        if r[key]:
            why.append(f"{name} {r[key]}")
    if r["page_errors"]:
        why.append(f"page-errors {len(r['page_errors'])}")
    # C3: the v3 surface never asks Scryfall for anything, whatever the store holds.
    if r["scryfall_requests"]:
        why.append(f"scryfall-requests {r['scryfall_requests']} (first: {r['scryfall_urls'][0]})")
    if r["image_other"]:
        why.append(f"image-route answered neither 200 nor 404 on {r['image_other']} requests")
    if not r["reason"] and not (r["image_200"] + r["image_404"]):
        why.append("no request to /api/card-image was seen (the image check would be vacuous)")
    if r["store"] == "filled":
        if r["unexplained_chips"]:
            why.append(f"unexplained-chips {len(r['unexplained_chips'])}: {sorted(r['unexplained_chips'])[:8]}")
        if not r["reason"] and not r["images_seen_max"]:
            why.append("--store filled but no card ever showed an image")
    elif r["store"] == "empty":
        if r["images_seen_max"] or r["image_200"]:
            why.append(f"--store empty but images loaded (most at once {r['images_seen_max']}, 200s {r['image_200']})")
    # C2: the auto-pass panel and the arrow overlay. A scrolling panel is already a box-scroll above.
    for key, name in (("autopass_missing", "autopass-panel-missing"), ("autopass_outside", "autopass-outside-viewport"),
                      ("arrows_missing", "arrow-overlay-missing"), ("arrows_pointer", "arrow-overlay-takes-pointer"),
                      ("arrows_intercept", "arrow-overlay-on-top")):
        if r[key]:
            why.append(f"{name} {r[key]}")
    if not r["reason"] and not r["autopass_measured"]:
        why.append(f"auto-pass panel never measured (table drawn on {r['table_iters']} iterations)")
    if not r["reason"] and not r["arrows_measured"]:
        why.append("arrow overlay never measured")
    # Vacuity: zeroes only count if the measurements were really taken, on a table with cards on it.
    if r.get("measure_failed"):
        why.append(f"measure-failed {r['measure_failed']}")
    if not r["reason"] and (not r.get("measured") or not r.get("max_cards")):
        why.append(f"nothing measured (measures {r.get('measured', 0)}, most cards seen {r.get('max_cards', 0)})")
    return why


async def main():
    ap = argparse.ArgumentParser(description="Headless sweep of the v3 play surface.")
    ap.add_argument("--port", type=int, default=3041)
    ap.add_argument("--seeds", default="0,3")
    ap.add_argument("--viewports", default="2560x1340,1707x880")
    ap.add_argument("--turns", type=int, default=10)
    ap.add_argument("--out", default=".")
    ap.add_argument("--max-iters", type=int, default=600)
    ap.add_argument("--block-scryfall", action="store_true", help="abort any request to a Scryfall host (it is counted, and BAD, either way)")
    ap.add_argument("--store", choices=("any", "filled", "empty"), default="any", help="what the server's image store is expected to hold")
    ap.add_argument("--known-missing", default="", help="saved /api/card-image/status JSON (its `failed`) or a file of names: chips --store filled accepts")
    a = ap.parse_args()
    a.known_missing = load_known_missing(a.known_missing)
    if a.port == 3040:
        sys.exit("refusing port 3040: that is the owner's server")
    os.makedirs(a.out, exist_ok=True)
    seeds = [int(s) for s in a.seeds.split(",") if s.strip()]
    vps = [tuple(int(n) for n in v.lower().split("x")) for v in a.viewports.split(",") if v.strip()]
    runs, bad, could_not = [], 0, 0
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path="/usr/bin/chromium")
        for seed in seeds:
            for w, h in vps:
                name = f"s{seed} {w}x{h}"
                try:
                    r = await run(browser, a, seed, w, h)
                except CouldNotRun as e:
                    could_not += 1
                    runs.append({"seed": seed, "viewport": f"{w}x{h}", "status": "COULD-NOT-RUN", "reason": str(e)})
                    print(f"COULD-NOT-RUN {name}: {e}", flush=True)
                    continue
                why = verdict(r, a.turns)
                r["status"] = "BAD" if why else "ok"
                r["why"] = why
                bad += 1 if why else 0
                runs.append(r)
                end = "game over" if r["game_over"] else f"reached turn {r['final_turn']}"
                print(("BAD " if why else "ok  ") + f"{name}: {end} iters {r['iters']} page-scroll {r['page_scroll']} "
                      f"box-scroll {r['box_scroll']} page-errors {len(r['page_errors'])} chipless {r['chipless']} "
                      f"cards {r['cards']}/{r['images']} peak-cards/with-image {r['peak_cards']}/{r['peak_images']} (tokens {r['peak_tokens']}) "
                      f"store {r['store']} image-200/404 {r['image_200']}/{r['image_404']} scryfall-requests {r['scryfall_requests']} "
                      f"explained-chips {len(r['explained_chips'])} unexplained-chips {len(r['unexplained_chips'])} pickers {sum(r.get('pickers', {}).values())} offscreen {r['offscreen']} refusals {len(r['refusals'])} "
                      f"gave-up {len(r['gave_up'])} stuck {len(r['stuck'])} console-errors {len(r['console_errors'])} "
                      f"autopass-panel {r['autopass_measured']}/{r['table_iters']} arrow-overlay {r['arrows_measured']}/{r.get('measured', 0)} sick {r['sick_seen']}"
                      + (f"  <- {'; '.join(why)}" if why else ""), flush=True)
        await browser.close()
    with open(os.path.join(a.out, "v3sweep.json"), "w") as f:
        json.dump({"port": a.port, "turns": a.turns, "block_scryfall": a.block_scryfall, "store": a.store,
                   "known_missing": sorted(a.known_missing), "runs": runs}, f, indent=1)
    sys.exit(1 if bad else 2 if could_not else 0)


if __name__ == "__main__":      # v3game.py imports this module for its page-side helpers
    asyncio.run(main())
