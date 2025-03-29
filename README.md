I have script that runs search-enginer-scraper.js outside podman because I trust the search engine
will not have anything malicious.

Other stuff runs inside the container because I do not know what might be on the 100s to 1000s of websites
the script will navigate to. The podman user only gets read privileges.
- The exception is node_modules... need to find a way to make that work.

