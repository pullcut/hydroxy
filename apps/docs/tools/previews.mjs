// Screenshot every docs page's example into public/previews/<slug>-<theme>.webp, for the
// component and chart index cards.
//
//   1. Start the docs site (cargo leptos watch) on http://localhost:8090.
//   2. node apps/docs/tools/previews.mjs [slug ...]
//
// Needs Playwright with Chrome: `npm i -g playwright`, or point PLAYWRIGHT at an install
// (e.g. PLAYWRIGHT=/path/to/node_modules/playwright/index.mjs). Slugs limit the run.
import { mkdirSync } from 'node:fs';
import { writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const { chromium } = await import(process.env.PLAYWRIGHT ?? 'playwright');
const SITE = process.env.SITE ?? 'http://localhost:8090';
const OUT = join(dirname(fileURLToPath(import.meta.url)), '..', 'public', 'previews');
const [W, H] = [960, 600]; // 2x the 480 x 300 card image
mkdirSync(OUT, { recursive: true });

const browser = await chromium.launch({ channel: 'chrome' });
const index = await browser.newPage();
let pages = [];
for (const section of ['/components', '/charts']) {
  await index.goto(SITE + section, { waitUntil: 'load', timeout: 120000 });
  pages.push(...(await index.$$eval('a.index-card', (as) => as.map((a) => a.getAttribute('href')))));
}
await index.close();
const only = process.argv.slice(2);
if (only.length) pages = pages.filter((p) => only.includes(p.split('/').pop()));

for (const theme of ['light', 'dark']) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 1000 }, deviceScaleFactor: 2 });
  await ctx.addInitScript((t) => localStorage.setItem('ox-theme', t), theme);
  for (const href of pages) {
    const slug = href.split('/').pop();
    const page = await ctx.newPage();
    try {
      await page.goto(SITE + href, { waitUntil: 'networkidle', timeout: 120000 });
      const demo = page.locator('section.demo').first();
      // Hydration can replace the element mid-scroll; try again on a fresh one.
      for (let i = 0; ; i++) {
        try {
          await demo.scrollIntoViewIfNeeded({ timeout: 10000 });
          break;
        } catch (e) {
          if (i === 4) throw e;
          await page.waitForTimeout(1000);
        }
      }
      await page.waitForTimeout(2600); // let entrances finish
      const box = await demo.boundingBox();
      // The top of the example, at the card's shape.
      const height = Math.min(box.height, (box.width * H) / W);
      const png = await page.screenshot({ clip: { x: box.x, y: box.y, width: box.width, height } });
      const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
      const webp = await page.evaluate(
        async ({ data, w, h, bg }) => {
          const img = new Image();
          img.src = 'data:image/png;base64,' + data;
          await img.decode();
          const c = document.createElement('canvas');
          [c.width, c.height] = [w, h];
          const g = c.getContext('2d');
          g.fillStyle = bg;
          g.fillRect(0, 0, w, h);
          // Fit the width, keep the top; centre vertically when shorter than the card.
          const scale = w / img.width;
          const dh = img.height * scale;
          g.drawImage(img, 0, Math.max(0, (h - dh) / 2), w, dh);
          return c.toDataURL('image/webp', 0.82).split(',')[1];
        },
        { data: png.toString('base64'), w: W, h: H, bg },
      );
      await writeFile(join(OUT, `${slug}-${theme}.webp`), Buffer.from(webp, 'base64'));
      console.log('ok', theme, slug);
    } catch (e) {
      console.log('FAIL', theme, slug, String(e).slice(0, 120));
    }
    await page.close();
  }
  await ctx.close();
}
await browser.close();
