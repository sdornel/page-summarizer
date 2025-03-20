from core.search_duckduckgo import duckduckgo_lite_search
# from core.search_duckduckgo import duckduckgo_browser_search
from core.crawl_extract import fetch_and_extract

def gather_raw_content(query, max_results=30):
    results = duckduckgo_lite_search(query)
    pages = []

    for result in results:
        content = fetch_and_extract(result["url"])
        if content:
            pages.append({
                "title": result["title"],
                "url": result["url"],
                "content": content
            })

    return pages
