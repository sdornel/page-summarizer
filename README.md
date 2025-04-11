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

– If local CPU processing still feels heavy, you can integrate a cloud-based summarization API (e.g., Hugging Face Inference API or a free tier of another service) to perform the abstractive summarization.
– This offloads the heavy computation to remote hardware while your Rust scraper continues working quickly on your local machine.

– Optimize your asynchronous code so that the scraping and summarization are fully parallelized.

    Read only filesystem
    Non-root container user
    Root remapping
    AppArmor/SELinux policies
    Egress gateway tech


node_modules can be read only because it's only written during container creation
^ two step process (one to build and one to execute)

for stuff that absolutely does need to stay read/write, we can do things like force non-execute. doesn't work too well for scripting languages, but works great for compiled languages

add a "deep think feature" that uses another AI agent to see if urls in a
page are worth opening as well. check that they aren't already in your list though