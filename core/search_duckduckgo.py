import requests
from bs4 import BeautifulSoup

def duckduckgo_lite_search(query, max_results=30):
    url = "https://lite.duckduckgo.com/lite/"
    response = requests.post(url, data={"q": query})
    response.raise_for_status()

    soup = BeautifulSoup(response.text, "html.parser")
    results = []

    for result in soup.select("a.result-link")[:max_results]:
        title = result.get_text(strip=True)
        href = result.get("href")

        results.append({
            "title": title,
            "url": href
        })

    return results
