import sys
import trafilatura

# def main():
#     text = sys.stdin.read()

#     # Simulated summary logic
#     summary = f"Summary (first 200 chars):\n{text[:200]}..."
#     print(summary)

# if __name__ == "__main__":
#     main()

html = sys.stdin.read()
cleaned = trafilatura.extract(html)

if cleaned:
    print(cleaned)
else:
    print("No readable content found.")