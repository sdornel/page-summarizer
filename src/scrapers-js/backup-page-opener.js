// this page opens from main.rs
import puppeteer from "puppeteer";
import { faker } from '@faker-js/faker';

const url = process.argv[2];
if (!url) {
    console.error('No URL Provided!');
    process.exit(0);
}

const userAgent = faker.internet.userAgent();
const acceptLanguage = faker.helpers.arrayElement([
  "en-US,en;q=0.9",
  "en-GB,en;q=0.8",
  "es-ES,es;q=0.9,en;q=0.8",
  "fr-FR,fr;q=0.9,en;q=0.8",
  "de-DE,de;q=0.9,en;q=0.8",
]);

const referer = faker.internet.url();
const accept = faker.helpers.arrayElement([
  "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
  "text/html,application/xhtml+xml,application/xml;q=0.8,image/webp,*/*;q=0.6"
]);

const browser = await puppeteer.launch({
    headless: true,
    args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-crash-reporter',
        '--no-crashpad',
        '--disable-features=Crashpad',
        '--disable-gpu',
    ],
});

const page = await browser.newPage();

await page.setUserAgent(userAgent);
await page.setExtraHTTPHeaders({
    "Accept-Language": acceptLanguage,
    "Accept": accept,
    "Referer": referer,
    "Sec-Fetch-Site": "none",
    "Sec-Fetch-Mode": "navigate",
    "Sec-Fetch-Dest": "document",
    "Upgrade-Insecure-Requests": "1",
    "Cache-Control": "max-age=0"
});

await page.evaluateOnNewDocument(() => {
    Object.defineProperty(navigator, "webdriver", { get: () => false });
    Object.defineProperty(navigator, "languages", { get: () => ["en-US", "en"] });
    Object.defineProperty(navigator, "plugins", { get: () => [1, 2, 3, 4] });
    window.chrome = { runtime: {} };
});

await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 10000 });

const pageText = await page.evaluate(() => {
    console.log('document.body', document.body);
    return document.body.innerText;
});
console.log('pageText', pageText);

console.log(pageText.trim()); // pipes it back to rust

// await browser.close();
console.log("✅ Finished evaluating page");
await browser.close();
console.log("✅ Browser closed");