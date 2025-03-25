// import puppeteer from "puppeteer";
// import fs from "fs";

// // 1) Parse the single query argument
// const query = process.argv.slice(2).join(" ");
// if (!query) {
//   console.error("❌ Please provide a search query.");
//   process.exit(1);
// }

// // We'll hard-code 10 pages and output "urls.json"
// const pages = 10;
// const outFile = "urls.json";

// (async function main() {
//   console.log("✅ Starting DuckDuckGo scraper in a single process...");
//   console.log(`   Query="${query}", pages=${pages}, outFile="${outFile}"\n`);

//   // 2) Launch Puppeteer (headless)
//   const browser = await puppeteer.launch({
//     headless: false,
//     slowMo: 50,
//     args: ["--no-sandbox", "--disable-setuid-sandbox"],
//   });

//   try {
//     const page = await browser.newPage();

//     // 3) Go to DuckDuckGo homepage
//     await page.goto("https://duckduckgo.com/", { waitUntil: "networkidle2" });

//     // 4) Type the query in the new search input (using name="q")
//     console.log(`Typing search query: "${query}"`);
//     await page.waitForSelector('input[name="q"]', { timeout: 10000 });
//     await page.type('input[name="q"]', query);
//     await page.keyboard.press("Enter");

//     // 5) Wait for results
//     await page.waitForSelector("#links", { timeout: 10000 })
//     .catch(() => console.log("❌ Could not find #links, maybe different layout."));

//     const allResults = [];

//     // 6) Loop up to 10 times, each time clicking "More Results"
//     for (let i = 0; i < pages; i++) {
//       console.log(`\n🔎 DuckDuckGo results page ${i + 1} for "${query}"`);

//       // Extract links on this page
//       const pageLinks = await page.evaluate(() => {
//         return Array.from(
//           document.querySelectorAll('a[data-handled-by-react="true"]')
//         ).map(a => a.href);
//       });
//       console.log(`   Found ${pageLinks.length} links on page ${i + 1}`);
//       allResults.push(...pageLinks);

//       // Check for "More Results" button
//       const moreButton = await page.$(".result--more__btn");
//       if (!moreButton) {
//         console.log("❌ No more results button found – stopping here.");
//         break;
//       }
//       await moreButton.click();

//       // Wait a bit for new results to load
//       await page.waitForTimeout(2000);

//       // Try waiting again for that button or more results
//       const foundMore = await page.waitForSelector(".result--more__btn", { timeout: 10000 })
//         .catch(() => {
//           console.log("❌ Possibly no more results loaded – stopping here.");
//         });
//       if (!foundMore) break;
//     }

//     // 7) Deduplicate results
//     const uniqueLinks = [...new Set(allResults)];
//     console.log(`\n✅ Collected ${uniqueLinks.length} unique links total.`);

//     // 8) Save final JSON
//     fs.writeFileSync(outFile, JSON.stringify(uniqueLinks, null, 2));
//     console.log(`   Results saved to ${outFile}.`);

//     // 9) Close Puppeteer
//     // await browser.close();
//     console.log("\n✅ Done. Exiting normally.");

//   } catch (error) {
//     console.error("❌ Error scraping DuckDuckGo:", error);
//     await browser.close();
//     process.exit(1); // exit on failure
//   }
// })();




import puppeteer from "puppeteer";
import fs from "fs";

// Get the query from CLI arguments
const query = process.argv.slice(2).join(" ");
if (!query) {
  console.error("❌ Please provide a search query.");
  process.exit(1);
}

// Hard-coded values
const pages = 10;       // Always scrape 10 pages
const outFile = "urls.json";

(async function main() {
  console.log("✅ Starting DuckDuckGo scraper in a single process...");
  console.log(`   Query="${query}", pages=${pages}, outFile="${outFile}"\n`);

  // Launch Puppeteer in headless mode
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
    // Wait for the search input; note: DuckDuckGo now uses input[name="q"]
    await page.waitForSelector('input[name="q"]', { timeout: 10000 });
    await page.type('input[name="q"]', query);
    await page.keyboard.press("Enter");

    // Wait for navigation to complete
    await page.waitForNavigation({ waitUntil: "networkidle2" });

    // Try waiting for the results container; if not found, log a warning
    try {
      await page.waitForSelector("#links", { timeout: 10000 });
    } catch (e) {
      console.log("❌ Could not find #links, maybe a different layout is used.");
    }

    const allResults = [];

    // Loop to click "More Results" and scrape links from up to 10 pages
    for (let i = 0; i < pages; i++) {
      console.log(`\n🔎 DuckDuckGo results page ${i + 1} for "${query}"`);

      // Extract links using the attribute selector
      const pageLinks = await page.evaluate(() => {
        return Array.from(
          document.querySelectorAll('a[data-handled-by-react="true"]')
        ).map(a => a.href);
      });
      console.log(`   Found ${pageLinks.length} links on page ${i + 1}`);
      allResults.push(...pageLinks);

      // Look for the "More Results" button using its id
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

      // Wait a bit for new results to load (using a manual delay)
      await new Promise(resolve => setTimeout(resolve, 2000));

      // Optionally, wait for the "More Results" button to appear again; if not, break the loop
      const foundMore = await page.waitForSelector("#more-results", { timeout: 10000 })
        .catch(() => {
          console.log("❌ Possibly no more results loaded – stopping.");
        });
      if (!foundMore) break;
    }

    // Deduplicate links
    const uniqueLinks = [...new Set(allResults)];
    console.log(`\n✅ Collected ${uniqueLinks.length} unique links total.`);

    // Write the results to the output file
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
