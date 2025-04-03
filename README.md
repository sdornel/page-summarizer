I have script that runs search-enginer-scraper.js outside podman because I trust the search engine
will not have anything malicious.

Other stuff runs inside the container because I do not know what might be on the 100s to 1000s of websites
the script will navigate to. 

Try to make podman user only have read only privileges

Use cargo check



i need to cache the results of websites...
– Extractive Pre-Selection:
Use fast, lightweight algorithms (e.g., keyword matching, TF-IDF, or Sentence Transformers with embeddings) to first select portions of the scraped text that are most relevant to the user’s query.
– Abstractive Summarization:
Then run a summarization model (for example, a distilled model like DistilBART or T5-small) on the relevant segments. This step rewrites the content into a concise summary that directly addresses the query. – Combined Output:
Present both a list of key excerpts (verbatim quotes or critical pieces) and a synthesized summary that integrates query-specific insights.

