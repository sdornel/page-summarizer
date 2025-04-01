I have script that runs search-enginer-scraper.js outside podman because I trust the search engine
will not have anything malicious.

Other stuff runs inside the container because I do not know what might be on the 100s to 1000s of websites
the script will navigate to. 

Try to make podman user only have read only privileges

Use cargo check

