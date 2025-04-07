import puppeteer from "puppeteer";
import { connect, StringCodec } from "nats";
import { faker } from "@faker-js/faker";

const url = process.argv[2];
const correlationId = process.argv[3];
const replySubject = process.argv[4];

const browser = await puppeteer.launch({
  headless: true,
  args: ["--no-sandbox", "--disable-setuid-sandbox"],
});
const page = await browser.newPage();

await page.setUserAgent(faker.internet.userAgent());

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