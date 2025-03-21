import requests
from bs4 import BeautifulSoup

def fetch_and_extract(url):
    headers = {
        "User-Agent": (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/122.0.0.0 Safari/537.36"
        ),
        "Accept": (
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"
        ),
        "Accept-Encoding": "gzip, deflate, br",
        "Accept-Language": "en-US,en;q=0.9",
        "Referer": "https://www.google.com/"
    }

    try:
        response = requests.get(url, headers=headers, timeout=10)
        if response.status_code == 403:
            print(f"[!] Skipping {url} — HTTP 403")
            return None
        if "application/pdf" in response.headers.get("Content-Type", ""):
            print(f"[!] Skipping {url} — PDF")
            return None

        soup = BeautifulSoup(response.text, "html.parser")
        paragraphs = soup.find_all("p")
        return "\n".join(p.get_text() for p in paragraphs).strip()

    except Exception as e:
        print(f"[!] Error fetching {url}: {e}")
        return None