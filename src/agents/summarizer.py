#!/usr/bin/env python3
import asyncio
import json
import logging
import re
import sys
from typing import List

from nats.aio.client import Client as NATS
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity

logging.basicConfig(stream=sys.stderr, level=logging.INFO)
SUMMARY_TYPE = "tfidf"


# ─────────────────────────────────────────────
# 🧠 Summarization Logic (TF-IDF based)
# ─────────────────────────────────────────────

def split_into_sentences(text: str) -> List[str]:
    return re.split(r'(?<=[.!?])\s+', text.strip())

def select_relevant_sentences(query: str, sentences: List[str], top_n: int = 5) -> List[str]:
    if not sentences:
        return []
    corpus = [query] + sentences
    vectorizer = TfidfVectorizer().fit(corpus)
    vectors = vectorizer.transform(corpus)
    similarities = cosine_similarity(vectors[0], vectors[1:]).flatten()
    top_indices = similarities.argsort()[::-1][:top_n]
    return [sentences[i] for i in top_indices]

def summarize(query: str, text: str) -> dict:
    clean_text = ' '.join(text.split())
    if not clean_text:
        return {"status": "failed", "error": "Empty text", "query": query}
    sentences = split_into_sentences(clean_text)
    selected = select_relevant_sentences(query, sentences)
    return {"query": query, "relevant_sentences": selected, "status": "success"}


# ─────────────────────────────────────────────
# 🔌 NATS Message Handler
# ─────────────────────────────────────────────

async def main():
    nc = NATS()
    await nc.connect("nats://nats:4222")

    async def handle_msg(msg):
        try:
            data = json.loads(msg.data.decode())
            logging.info(f"📥 Received job for: {data.get('url')}")

            query = data["query"]
            text = data["text"]
            corr_id = data["correlation_id"]
            reply_to = data["reply_to"]

            result = summarize(query, text)
            result["correlation_id"] = corr_id
            result["summary_type"] = SUMMARY_TYPE

            await nc.publish(reply_to, json.dumps(result).encode())
            logging.info(f"📤 Sent reply to {reply_to} (corr_id {corr_id})")
        except Exception as e:
            logging.error(f"❌ Error in message handler: {e}")

    await nc.subscribe("summarization_job", cb=handle_msg)
    logging.info("✅ Subscribed to 'summarization_job'")

    while True:
        await asyncio.sleep(1)

if __name__ == "__main__":
    asyncio.run(main())
