# Use official Python + Playwright image
FROM mcr.microsoft.com/playwright/python:v1.51.0-jammy

# Set working directory
WORKDIR /app

# Copy files
COPY . .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Install Playwright browser binaries
RUN playwright install

# Run Streamlit app and log full server info
CMD ["streamlit", "run", "app.py", "--server.port=7860", "--server.address=0.0.0.0", "--logger.level=debug"]
