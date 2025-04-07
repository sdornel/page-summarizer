import puppeteer from "puppeteer";
import { connect } from "nats.ws";
import { faker } from "@faker-js/faker";
import process from "process";

const [,, url, correlationId, replyTo = "fallback_response"] = process.argv;

if (!url || !correlationId) {
  console.error("❌ URL or correlation_id missing");
  process.exit(1);
}

const userAgent = faker.internet.userAgent();
const acceptLanguage = faker.helpers.arrayElement([
  "en-US,en;q=0.9", "es-ES,es;q=0.9,en;q=0.8"
]);
const referer = faker.internet.url();
const accept = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";

async function scrape() {
  const browser = await puppeteer.launch({
    headless: true,
    args: ["--no-sandbox", "--disable-setuid-sandbox"]
  });

  const page = await browser.newPage();
  await page.setUserAgent(userAgent);
  await page.setExtraHTTPHeaders({
    "Accept-Language": acceptLanguage,
    "Accept": accept,
    "Referer": referer
  });

  await page.evaluateOnNewDocument(() => {
    Object.defineProperty(navigator, "webdriver", { get: () => false });
    Object.defineProperty(navigator, "languages", { get: () => ["en-US", "en"] });
    Object.defineProperty(navigator, "plugins", { get: () => [1, 2, 3] });
    window.chrome = { runtime: {} };
  });

  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 15000 });
  const text = await page.evaluate(() => document.body.innerText.trim());

  await browser.close();
  return text;
}

async function sendToNats(text) {
  const nc = await connect({ servers: "nats://nats:4222" });

  const message = {
    correlation_id: correlationId,
    summary_type: "raw_fallback",
    status: text ? "success" : "failed",
    url,
    text
  };

  await nc.publish(replyTo, new TextEncoder().encode(JSON.stringify(message)));
  await nc.flush();
  await nc.drain();
  await nc.close();
}

scrape()
  .then(sendToNats)
  .catch(async (err) => {
    console.error("❌ Puppeteer/NATS error", err.message);
    await sendToNats(""); // Send empty payload on error
    process.exit(1);
});
