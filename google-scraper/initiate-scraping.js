const { spawn } = require("child_process");
const fs = require("fs");

const query = process.argv.slice(2).join(" ");
if (!query) {
    console.error("Please provide a search query.");
    process.exit(1);
}

const runScraper = (offset, outputFile) => {
    return new Promise((resolve, reject) => {
        const proc = spawn("node", [
            "extract_google_links.js", // run the function with specific flags
            "--query", query,
            "--offset", offset.toString(),
            "--pages", "10",
            "--out", outputFile
        ], {
            cwd: __dirname, // ensures script runs from same folder
            stdio: "inherit" // makes sure child process logs show up in your terminal
        });
    
        proc.on("close", code => {
            if (code === 0) resolve();
            else reject(new Error(`Scraper with offset ${offset} exited with code ${code}`));
        });
    });
};

const mergeResults = () => {
    try {
        const urlsEven = JSON.parse(fs.readFileSync(__dirname + "/urls_even.json", "utf8"));
        const urlsOdd = JSON.parse(fs.readFileSync(__dirname + "/urls_odd.json", "utf8"));
        const merged = [...new Set([...urlsEven, ...urlsOdd])];
        fs.writeFileSync(__dirname + "/urls.json", JSON.stringify(merged, null, 2));
        console.log(`✅ Merged ${merged.length} unique URLs into urls.json`);
        
        fs.unlinkSync(evenPath);
        fs.unlinkSync(oddPath);
        console.log('Deleted temporary json files');
    } catch (error) {
        console.error('Failed to merge results.', error);
    }
};

const initiate = () => {
    (async () => {
        try {
          console.log("🚀 Starting dual Puppeteer scrapers...");
          await Promise.all([
            runScraper(0, "urls_even.json"),
            runScraper(1, "urls_odd.json")
          ]);
      
          mergeResults();
        } catch (err) {
          console.error("❌ Error during scraping:", err);
          process.exit(1);
        }
    })();
}

initiate();