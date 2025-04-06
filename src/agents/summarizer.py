#!/usr/bin/env python3
"""
summarizer.py

This agent is part of the deep research tool and is intended to be called from the Rust scraper.
It reads a query (for example, a URL or a search query) from the command-line arguments and the full
web page text from STDIN. It then uses an extractive method based on TF-IDF to select the most relevant 
sentences from the text.

The output is a JSON object (written to stdout) containing:
  - The provided query.
  - A list of key sentences (excerpts) that best match the query.
  - A status field indicating success or failure.

Any debugging or logging output is sent to stderr so it does not interfere with the JSON output.
"""

import sys
import json
import re
import logging
from typing import List
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity

# Configure logging to output to stderr.
logging.basicConfig(stream=sys.stderr, level=logging.INFO, format='[%(levelname)s] %(message)s')

def split_into_sentences(text: str) -> List[str]:
    """
    Splits the input text into sentences using a simple regex.
    For a more robust solution, consider using nltk.sent_tokenize.
    """
    # Split on punctuation followed by whitespace.
    sentences = re.split(r'(?<=[.!?])\s+', text.strip())
    return sentences

def select_relevant_sentences(query: str, sentences: List[str], top_n: int = 5) -> List[str]:
    """
    Selects the top_n sentences that are most relevant to the query using TF-IDF cosine similarity.
    """
    if not sentences:
        return []
    
    # Create a corpus with the query as the first element.
    corpus = [query] + sentences
    vectorizer = TfidfVectorizer().fit(corpus)
    vectors = vectorizer.transform(corpus)
    query_vector = vectors[0]
    sentence_vectors = vectors[1:]
    # Compute cosine similarity between the query and each sentence.
    similarities = cosine_similarity(query_vector, sentence_vectors).flatten()
    # Get the indices of the top_n sentences with the highest similarity scores.
    top_indices = similarities.argsort()[::-1][:top_n]
    # Return the corresponding sentences.
    return [sentences[i] for i in top_indices]

def summarize(query: str, text: str) -> dict:
    """
    Processes the input text by splitting it into sentences and selecting the ones
    most relevant to the provided query.
    """
    logging.info('-=-=-=-=-= the fox jumped over the lazy dog -=-=-=-=-')
    # Log a debug message (to stderr).
    logging.info("Starting summarization process...")
    
    # Clean and normalize text.
    clean_text = ' '.join(text.split())
    if not clean_text:
        return {"error": "Empty text after cleaning", "query": query, "status": "failed"}
    
    sentences = split_into_sentences(clean_text)
    logging.info("Extracted %d sentences.", len(sentences))
    relevant_sentences = select_relevant_sentences(query, sentences, top_n=5)
    logging.info("Selected %d relevant sentences.", len(relevant_sentences))
    
    return {
        "query": query,
        "relevant_sentences": relevant_sentences,
        "status": "success"
    }

def main():
    print('got to summarizer.py')
    # Ensure a query is provided.
    if len(sys.argv) < 2:
        print(json.dumps({"error": "No query provided.", "status": "failed"}))
        sys.exit(1)
    
    query = sys.argv[1]
    # Read full text from STDIN.
    input_text = sys.stdin.read()
    if not input_text.strip():
        print(json.dumps({"error": "No input text provided.", "status": "failed"}))
        sys.exit(1)
    
    result = summarize(query, input_text)
    # Only output the JSON result to stdout.
    print(json.dumps(result, indent=2))

if __name__ == "__main__":
    main()
