import puppeteer from "puppeteer";
import { connect, StringCodec } from "nats";
import { faker } from "@faker-js/faker";

const url = process.argv[2];
const correlationId = process.argv[3];
const replySubject = process.argv[4];

const browser = await puppeteer.launch({
    headless: true,
    // protocolTimeout: 20000, // ⬅️ Timeout for internal DevTools Protocol calls
    args: [
      "--no-sandbox",
      "--disable-setuid-sandbox",
      "--disable-blink-features=AutomationControlled",
      "--disable-web-security",
      "--disable-features=IsolateOrigins,site-per-process",
      "--no-zygote",
      "--single-process",
      "--disable-dev-shm-usage",
      "--window-size=1366,768",
    ],
});
const page = await browser.newPage();

const languageOptions = [
    ["en-US", "en"],
    ["en-GB", "en"],
    ["fr-FR", "fr"],
    ["de-DE", "de"],
    ["es-ES", "es"],
    ["it-IT", "it"],
    ["pl-PL", "pl"],
    ["ru-RU", "ru"],
    ["ja-JP", "ja"],
    ["zh-CN", "zh"],
    ["pt-BR", "pt"],
    ["nl-NL", "nl"],
    ["sv-SE", "sv"],
    ["tr-TR", "tr"],
    ["ko-KR", "ko"]
];

const languages = faker.helpers.arrayElement(languageOptions);
  
    await page.setUserAgent(faker.internet.userAgent());
    await page.evaluateOnNewDocument((langs) => {
        Object.defineProperty(navigator, "languages", { get: () => langs });
    }, lang);
  
await page.setRequestInterception(true);

// Block images, stylesheets, fonts, and media to speed up loading
page.on("request", (req) => {
    const type = req.resourceType();
    if (["image", "stylesheet", "font", "media"].includes(type)) req.abort();
    else req.continue();
});


// Spoof navigator props
await page.evaluateOnNewDocument((langs) => {
    Object.defineProperty(navigator, "webdriver", { get: () => false });
    Object.defineProperty(navigator, "platform", { get: () => "Win32" });
    Object.defineProperty(navigator, "language", { get: () => langs[0] });
    Object.defineProperty(navigator, "languages", { get: () => langs });
    Object.defineProperty(navigator, "plugins", { get: () => [1, 2, 3, 4, 5] });
}, languages);

await page.goto(url, { waitUntil: "domcontentloaded", timeout: 10000 });
const text = await page.evaluate(() => document.body.innerText);

const nc = await connect({ servers: "nats://nats:4222" });
const sc = StringCodec();

nc.publish("summarization_job", sc.encode(JSON.stringify({
  correlation_id: correlationId,
  reply_to: replySubject,
  query: url,
  url,
  text,
  summary_type: "fallback",
  status: "success"
})));

await nc.drain();
await browser.close();