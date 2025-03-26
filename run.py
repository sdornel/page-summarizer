import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parent
PUPPETEER_DIR = ROOT / "search-engine-scraper"
RUST_DIR = ROOT / "rust-scraper"

def run_subprocess(cmd, cwd):
    print(f"▶️ Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        raise RuntimeError(f"❌ Command failed: {' '.join(cmd)}")

def main():
    if len(sys.argv) < 2:
        print("❌ Please provide a search query.")
        sys.exit(1)

    try:
        # run rust scraper
        run_subprocess(["cargo", "run", "--release"], cwd=RUST_DIR)

        print("🎉 Scraping pipeline complete.")
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()