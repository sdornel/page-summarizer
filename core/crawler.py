# import requests
# from bs4 import BeautifulSoup
# from agents.tool import function_tool

# @function_tool
# async def crawler(query: str, max_results: int = 10) -> dict:
#     headers = {
#         "User-Agent": (
#             "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
#             "AppleWebKit/537.36 (KHTML, like Gecko) "
#             "Chrome/122.0.0.0 Safari/537.36"
#         ),
#         "Accept": (
#             "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"
#         ),
#         "Accept-Encoding": "gzip, deflate, br",
#         "Accept-Language": "en-US,en;q=0.9",
#         "Referer": "https://www.google.com/",
#         "Connection": "keep-alive",
#         "Upgrade-Insecure-Requests": "1",
#         "Cache-Control": "max-age=0"
#     }


#     try:
#         response = requests.get(url, headers=headers, timeout=10)
#         if response.status_code == 403:
#             print(f"[!] Skipping {url} — HTTP 403 (Forbidden)")
#             return None
#         if "application/pdf" in response.headers.get("Content-Type", ""):
#             print(f"[!] Skipping {url} — no content extracted (PDF)")
#             return None

#         soup = BeautifulSoup(response.text, "html.parser")
#         paragraphs = soup.find_all("p")
#         content = "\n".join(p.get_text() for p in paragraphs)
#         return content.strip()

#     except Exception as e:
#         print(f"[!] Error fetching {url} — {e}")
#         return None

from agents.tool import function_tool
from core.search_duckduckgo import duckduckgo_lite_search
from core.fetcher import fetch_and_extract

@function_tool
async def crawler(query: str, max_results: int = 10) -> dict:
    results = duckduckgo_lite_search(query)
    pages = []

    for result in results[:max_results]:
        content = fetch_and_extract(result["url"])
        if content:
            pages.append({
                "title": result["title"],
                "url": result["url"],
                "content": content
            })

    return {
        "success": True,
        "sources_count": len(pages),
        "results": pages
    }