#!/usr/bin/env node

const { chromium } = require('playwright-core');

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}

const baseUrl = args.get('--base-url');
const screen = args.get('--screen');
const outPath = args.get('--out');

if (!baseUrl || !screen || !outPath) {
  console.error('usage: node ui_capture_review.js --base-url URL --screen SCREEN --out FILE');
  process.exit(2);
}

const expectedTitles = {
  dashboard: 'Project Dashboard',
  editor: 'Tile Map Editor Main Canvas',
  tilesets: 'Tileset Property Editor',
  layers: 'Layer Manager',
  objects: 'Object Library',
  settings: 'App Settings',
};

const expectedTitle = expectedTitles[screen];
if (!expectedTitle) {
  console.error(`unknown screen: ${screen}`);
  process.exit(2);
}

(async () => {
  const browser = await chromium.launch({
    executablePath: '/usr/bin/chromium',
    headless: true,
    args: ['--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage'],
  });

  const context = await browser.newContext({
    viewport: { width: 384, height: 688 },
    screen: { width: 384, height: 688 },
    deviceScaleFactor: 2,
    isMobile: true,
    hasTouch: true,
  });
  const page = await context.newPage();

  page.on('console', (msg) => console.log(`[console:${screen}] ${msg.type()} ${msg.text()}`));
  page.on('pageerror', (err) => console.error(`[pageerror:${screen}] ${err.message}`));
  page.on('requestfailed', (req) => {
    console.error(`[requestfailed:${screen}] ${req.url()} ${req.failure()?.errorText ?? ''}`);
  });

  const url = `${baseUrl}/?review=1&screen=${screen}`;
  await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await page.waitForFunction(
    (title) => document.body.innerText.includes(title),
    expectedTitle,
    { timeout: 30000 }
  );
  await page.waitForFunction(
    () => Array.from(document.images).every((img) => img.complete && img.naturalWidth > 0),
    { timeout: 30000 }
  );
  await page.waitForTimeout(900);
  await page.screenshot({ path: outPath, fullPage: true });
  await context.close();
  await browser.close();
})();
