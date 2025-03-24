const puppeteer = require("puppeteer");
const fs = require("fs");

const randomDelay = async (min = 500, max = 1500) => {
    const delayTime = Math.floor(Math.random() * (max - min + 1)) + min;
    return new Promise(resolve => setTimeout(resolve, delayTime));
}

async function extractGoogleLinks(query, maxPages = 3, delay = 1500) {
    // const browser = await puppeteer.launch({ headless: "new" });
    const browser = await puppeteer.launch({
      headless: "new",
      executablePath: "/usr/bin/chromium",
      args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    const page = await browser.newPage();
  
    await page.setUserAgent(
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    );
  
    const results = [];
  
    for (let i = 0; i < maxPages; i++) {
      const start = i * 10;
      const searchUrl = `https://www.google.com/search?client=firefox-b-1-d&channel=entpr&q=${encodeURIComponent(query)}&start=${start}`;
  
      console.log(`Fetching page ${i + 1}: ${searchUrl}`);
      await page.goto(searchUrl, { waitUntil: "networkidle2" });
      await page.waitForTimeout(randomDelay(delay - 750, delay + 750)); // slight delay to avoid detection
  
      const links = await page.evaluate(() => {
        const anchors = Array.from(document.querySelectorAll("a"));
        return anchors
          .map((a) => a.href)
          .filter((href) => href.startsWith("https://") && !href.includes("google.com"));
      });
  
      results.push(...links);
    }
  
    await browser.close();
  
    // De-duplicate
    const deduped = [...new Set(results)];
  
    fs.writeFileSync("urls.json", JSON.stringify(deduped, null, 2));
    console.log(`✅ Extracted ${deduped.length} URLs to urls.json`);
  }
  
  // Example usage:
  const [,, ...args] = process.argv;
  const query = args.join(" ");
  if (!query) {
    console.error("❌ Please provide a search query.");
    process.exit(1);
  }
  
  extractGoogleLinks(query, 10); // adjust pages here