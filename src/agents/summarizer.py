# deprecated?

# #!/usr/bin/env python3
# import asyncio
# import json
# import logging
# import re
# import sys
# from typing import List

# from nats.aio.client import Client as NATS
# from sklearn.feature_extraction.text import TfidfVectorizer
# from sklearn.metrics.pairwise import cosine_similarity

# logging.basicConfig(stream=sys.stderr, level=logging.INFO)
# SUMMARY_TYPE = "tfidf"


# # ─────────────────────────────────────────────
# # 🧠 Summarization Logic (TF-IDF based)
# # ─────────────────────────────────────────────

# def split_into_sentences(text: str) -> List[str]:
#     return re.split(r'(?<=[.!?])\s+', text.strip())

# def select_relevant_sentences(query: str, sentences: List[str], top_n: int = 5) -> List[str]:
#     if not sentences:
#         return []
#     corpus = [query] + sentences
#     vectorizer = TfidfVectorizer().fit(corpus)
#     vectors = vectorizer.transform(corpus)
#     similarities = cosine_similarity(vectors[0], vectors[1:]).flatten()
#     top_indices = similarities.argsort()[::-1][:top_n]
#     return [sentences[i] for i in top_indices]

# def summarize(query: str, text: str) -> dict:
#     clean_text = ' '.join(text.split())
#     if not clean_text:
#         return {"status": "failed", "error": "Empty text", "query": query}
#     sentences = split_into_sentences(clean_text)
#     selected = select_relevant_sentences(query, sentences)
#     return {"query": query, "relevant_sentences": selected, "status": "success"}


# # ─────────────────────────────────────────────
# # 🔌 NATS Message Handler
# # ─────────────────────────────────────────────

# async def wildcard_debugger(msg):
#     try:
#         print(f"📥 Wildcard received on '{msg.subject}': {msg.data.decode(errors='ignore')[:200]}...")
#     except Exception as e:
#         print(f"⚠️ Wildcard decode error: {e}")

# async def main():
#     print("🔌 Starting Summarizer Agent...")
#     nc = NATS()
#     await nc.connect("nats://nats:4222")

#     async def handle_msg(msg):
#         print(f"📥 Wildcard received on '{msg.subject}': {msg.data.decode(errors='ignore')[:200]}...")
#         try:
#             data = json.loads(msg.data.decode())
#             logging.info(f"📥 Received job for: {data.get('url')}")

#             query = data["query"]
#             text = data["text"]
#             corr_id = data["correlation_id"]
#             reply_to = data["reply_to"]

#             result = summarize(query, text)
#             result["correlation_id"] = corr_id
#             result["summary_type"] = SUMMARY_TYPE

#             await nc.publish(reply_to, json.dumps(result).encode())
#             logging.info(f"📤 Sent reply to {reply_to} (corr_id {corr_id})")
#         except Exception as e:
#             logging.error(f"❌ Error in message handler: {e}")

#     # Subscribe to actual job queue
#     await nc.subscribe("summarization_job", cb=handle_msg)

#     # # Optional wildcard for debug logging
#     # async def wildcard_debugger(msg):
#     #     print(f"📥 Wildcard received on '{msg.subject}': {msg.data.decode(errors='ignore')[:200]}...")

#     # await nc.subscribe(">", cb=wildcard_debugger)

#     # Signal readiness
#     await nc.publish("health.summarizer", b"ok")

#     logging.info("✅ Subscribed to 'summarization_job' and wildcard")

#     while True:
#         await asyncio.sleep(1)

# if __name__ == "__main__":
#     asyncio.run(main())



# placeholder
# import asyncio
# import json
# import nats

# async def main():
#     nc = await nats.connect("nats://nats:4222")
#     js = nc.jetstream()

#     async def handler(msg):
#         data = json.loads(msg.data.decode())
#         print(f"🤖 Got summarization job: {data.get('query', 'unknown')}")

#         reply_subject = data.get("reply_to")
#         if reply_subject:
#             response = {
#                 "correlation_id": data.get("correlation_id"),
#                 "summary_type": "placeholder",
#                 "summary": f"(Placeholder summary for: {data.get('query')})"
#             }
#             await nc.publish(reply_subject, json.dumps(response).encode())
#             print(f"✅ Replied to: {reply_subject}")

#     await nc.subscribe("summarization_job", cb=handler)

#     # Send ready message
#     await nc.publish("health.summarizer", b"ready")
#     print("🟢 summarizer.py placeholder ready")

#     # Keep running
#     while True:
#         await asyncio.sleep(1)

# if __name__ == "__main__":
#     asyncio.run(main())




import asyncio
import json
import nats

async def health_request_handler(msg):
    # Immediately reply with "ready" (or any token you choose).
    await msg.respond(b"ready")
    print("🟢 Health request received and responded.")

async def main():
    nc = await nats.connect("nats://nats:4222")
    js = nc.jetstream()

    async def handler(msg):
        data = json.loads(msg.data.decode())
        print(f"🤖 Got summarization job: {data.get('query', 'unknown')}")
        reply_subject = data.get("reply_to")
        if reply_subject:
            response = {
                "correlation_id": data.get("correlation_id"),
                "summary_type": "placeholder",
                "summary": f"(Placeholder summary for: {data.get('query')})"
            }
            await nc.publish(reply_subject, json.dumps(response).encode())
            print(f"✅ Replied to: {reply_subject}")

    # Subscribe to the job subject.
    await nc.subscribe("summarization_job", cb=handler)
    # Subscribe to the health request subject.
    await nc.subscribe("health.summarizer.request", cb=health_request_handler)

    # Optionally still publish a one-time health message.
    await nc.publish("health.summarizer", b"ready")
    print("🟢 summarizer.py placeholder ready")

    # Keep the agent running.
    while True:
        await asyncio.sleep(1)

if __name__ == "__main__":
    asyncio.run(main())