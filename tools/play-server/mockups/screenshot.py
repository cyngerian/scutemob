# Headless check for the mockups. Mockup 6 is checked at the owner's viewport
# (2560x1340 = 1440p at 100 %) and at 1707x880 (150 %), in each of its states;
# 1-5 at 1920x1080. Every card must show its image and the page must never scroll.
# Needs the mockups served on 127.0.0.1:3043 (python3 -m http.server 3043 --directory tools/play-server/mockups).
import asyncio, sys
from playwright.async_api import async_playwright
OUT = sys.argv[1] if len(sys.argv) > 1 else "."
TARGETS = [(f"0{n}-{nm}", 1920, 1080, "") for n, nm in [(1, "table"), (2, "arena"), (3, "pod"), (4, "focus"), (5, "columns")]]
for st in (1, 2, 3, 4):
    TARGETS.append(("06-table", 2560, 1340, f"?state={st}"))
    TARGETS.append(("06-table", 1707, 880, f"?state={st}"))
async def main():
    bad = 0
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path="/usr/bin/chromium")
        for name, w, h, q in TARGETS:
            pg = await b.new_page(viewport={"width": w, "height": h})
            errs = []; pg.on("pageerror", lambda e: errs.append(str(e)))
            await pg.goto(f"http://127.0.0.1:3043/{name}.html{q}")
            await pg.wait_for_timeout(2500)
            sh, sw, imgs, cards = await pg.evaluate("[document.documentElement.scrollHeight, document.documentElement.scrollWidth, document.querySelectorAll('.card.has-image').length, document.querySelectorAll('.card').length]")
            # a card's image counts as shown only if its box is not clipped away by an overflow:hidden ancestor
            clipped = await pg.evaluate("[...document.querySelectorAll('.card')].filter(c => { const r = c.getBoundingClientRect(); return r.bottom > innerHeight || r.right > innerWidth || r.top < 0 || r.left < 0; }).length")
            tag = f"{name}{q.replace('?state=', '-s')}-{w}x{h}"
            await pg.screenshot(path=f"{OUT}/mock-{tag}.png")
            ok = sh == h and sw == w and imgs == cards and not errs
            bad += 0 if ok else 1
            print(("ok  " if ok else "BAD ") + f"{tag}: scroll {sh}x{sw} imgs {imgs}/{cards} offscreen {clipped} errors {errs}")
            await pg.close()
        await b.close()
    sys.exit(1 if bad else 0)
asyncio.run(main())
