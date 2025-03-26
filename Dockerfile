# Start with Node base
FROM node:22

# Prevent Puppeteer from downloading Chromium, since we install system-wide Chrome below.
# See https://github.com/puppeteer/puppeteer/blob/main/docs/troubleshooting.md#running-puppeteer-in-docker
ENV PUPPETEER_SKIP_DOWNLOAD=true

# Install packages needed to:
#  1) Add Google's official Chrome repo
#  2) Install system dependencies required by Chrome
#  3) Install additional fonts for international sites
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    wget \
    gnupg \
    ca-certificates \
    fonts-ipafont-gothic \
    fonts-wqy-zenhei \
    fonts-thai-tlwg \
    fonts-kacst \
    fonts-freefont-ttf \
    libxss1 \
    # For best practice, remove next line if you do NOT want apt to suggest any packages
    apt-transport-https \
 # Add Google’s GPG key
 && wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | apt-key add - \
 # Add Google’s Chrome repository
 && echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" \
    | tee /etc/apt/sources.list.d/google-chrome.list \
 # Install Google Chrome Stable + clean up
 && apt-get update \
 && apt-get install -y --no-install-recommends \
    google-chrome-stable \
 # Clean up apt cache & lists
 && rm -rf /var/lib/apt/lists/*

# Create working directory and set ownership
WORKDIR /usr/src/app

# Copy app code into the container
COPY . /usr/src/app

# Create non-root user (pptruser = Puppeteer User)
RUN useradd -m -r -s /bin/bash pptruser \
  && mkdir -p /usr/src/app/output \
  && chown -R pptruser:pptruser /usr/src/app

# switch to pptruser
USER pptruser

# Initialize a package.json so npm install won't complain
RUN npm init -y

# Install Puppeteer, which will see the ENV above and skip bundling Chromium
RUN npm install puppeteer

# Example: default command runs "google-chrome-stable" to verify it installed
# In a real project, you might `CMD ["node", "yourScript.js"]`
CMD ["google-chrome-stable", "--version"]