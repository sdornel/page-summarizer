import puppeteer from "puppeteer";
import fs from "fs";

// get the query from CLI arguments
const query = process.argv.slice(2).join(" ");
if (!query) {
  console.error("❌ Please provide a search query.");
  process.exit(1);
}

const pages = 10;
const outFile = "urls.json";

(async function main() {
  console.log("✅ Starting DuckDuckGo scraper in a single process...");
  console.log(`   Query="${query}", pages=${pages}, outFile="${outFile}"\n`);

  const browser = await puppeteer.launch({
    headless: false,
    slowMo: 50,
    args: ["--no-sandbox", "--disable-setuid-sandbox"],
  });

  try {
    const page = await browser.newPage();

    console.log("Navigating to DuckDuckGo...");
    await page.goto("https://duckduckgo.com/", { waitUntil: "networkidle2" });

    console.log(`Typing search query: "${query}"`);

    await page.waitForSelector('input[name="q"]', { timeout: 10000 });
    await page.type('input[name="q"]', query);
    await page.keyboard.press("Enter");

    await page.waitForNavigation({ waitUntil: "networkidle2" });

    try {
      await page.waitForSelector("#links", { timeout: 10000 });
    } catch (e) {
      console.log("❌ Could not find #links, maybe a different layout is used.");
    }

    const allResults = [];

    for (let i = 0; i < pages; i++) {
      console.log(`\n🔎 DuckDuckGo results page ${i + 1} for "${query}"`);

      const pageLinks = await page.evaluate(() => {
        return Array.from(
          document.querySelectorAll('a[data-handled-by-react="true"]')
        ).map(a => a.href);
      });
      console.log(`   Found ${pageLinks.length} links on page ${i + 1}`);
      allResults.push(...pageLinks);

      let moreButton = await page.$("#more-results");
      if (!moreButton) {
        console.log("❌ No 'More Results' button found. Trying to scroll for more results...");
        await new Promise(resolve => setTimeout(resolve, 3000));
        moreButton = await page.$("#more-results");
        if (!moreButton) {
          console.log("❌ Still no 'More Results' button found – stopping.");
          break;
        }
      }
      console.log("Clicking 'More Results' button...");
      await moreButton.click();

      await new Promise(resolve => setTimeout(resolve, 2000)); // this might be redundant?

      const foundMore = await page.waitForSelector("#more-results", { timeout: 10000 })
        .catch(() => {
          console.log("❌ Possibly no more results loaded – stopping.");
        });
      if (!foundMore) break;
    }

    const uniqueLinks = [...new Set(allResults)];
    console.log(`\n✅ Collected ${uniqueLinks.length} unique links total.`);

    fs.writeFileSync(outFile, JSON.stringify(uniqueLinks, null, 2));
    console.log(`   Results saved to ${outFile}.`);

    await browser.close();
    console.log("\n✅ Done. Exiting normally.");

  } catch (error) {
    console.error("❌ Error scraping DuckDuckGo:", error);
    await browser.close();
    process.exit(1);
  }
})();
