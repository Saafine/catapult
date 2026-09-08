// Rasterizes packs/<PACK>/svg/*.svg to packs/<PACK>/sprites/*.png at 2x with
// transparent background. Needs Playwright and a Chrome; PNGs are committed
// so this only runs when the art changes.
//   NODE_PATH=<dir with node_modules> node tools/render_sprites.js
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const root = path.join(__dirname, '..');
const PACK = process.env.PACK || 'calgary_xmas';
const svgDir = path.join(root, 'packs', PACK, 'svg');
const outDir = path.join(root, 'packs', PACK, 'sprites');
const SCALE = 2;
(async () => {
  fs.mkdirSync(outDir, { recursive: true });
  const browser = await chromium.launch({ headless: true, executablePath: process.env.CHROME || '/usr/bin/google-chrome' });
  for (const f of fs.readdirSync(svgDir).filter(f => f.endsWith('.svg')).sort()) {
    const svg = fs.readFileSync(path.join(svgDir, f), 'utf8');
    const w = +svg.match(/width="(\d+)"/)[1];
    const h = +svg.match(/height="(\d+)"/)[1];
    // Keep every texture at or under 8192 px wide (GPU texture limit).
    const scale = w * SCALE > 8192 ? 1 : SCALE;
    const page = await browser.newPage({ viewport: { width: w, height: h }, deviceScaleFactor: scale });
    await page.setContent(`<html><body style="margin:0;background:transparent">${svg}</body></html>`);
    const out = path.join(outDir, f.replace('.svg', '.png'));
    await page.screenshot({ path: out, omitBackground: true });
    await page.close();
    console.log(f, `${w}x${h}`);
  }
  await browser.close();
})();
