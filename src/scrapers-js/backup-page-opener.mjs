// this page opens from main.rs
import { puppeteer } from "puppeteer";

const url = process.argv[2];
if (!url) {
    console.error('No URL Provided!');
    process.exit(0);
}

const browser = await puppeteer.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
});

const page = await browser.newPage();
await page.goto(url, { waitUntil: 'networkidle2', timeout: 10000 });

const pageText = await page.evaluate(() => {
    return document.body.innerText;
});

console.log(pageText.trim()); // pipes it back to rust

await browser.close();