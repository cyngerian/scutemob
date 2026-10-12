# Headless check: each mockup at 1920x1080 must show every card image and never scroll the page.
# Needs the mockups served on 127.0.0.1:3043 (python3 -m http.server 3043 --directory tools/play-server/mockups).
import sys, asyncio
from playwright.async_api import async_playwright
async def main():
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path="/usr/bin/chromium")
        for name in ["01-table","02-arena","03-pod","04-focus","05-columns"]:
            pg = await b.new_page(viewport={"width":1920,"height":1080})
            errs=[]; pg.on("pageerror", lambda e: errs.append(str(e)))
            await pg.goto(f"http://127.0.0.1:3043/{name}.html")
            await pg.wait_for_timeout(6000)
            ov = await pg.evaluate("[document.documentElement.scrollHeight, document.documentElement.scrollWidth, document.querySelectorAll('.card.has-image').length, document.querySelectorAll('.card').length]")
            await pg.screenshot(path=f"mock-{name}.png")
            print(name, "scroll", ov[0], "x", ov[1], "imgs", ov[2], "/", ov[3], "errors", errs)
            await pg.close()
        await b.close()
asyncio.run(main())
