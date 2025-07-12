#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Define variables
IMAGE_NAME="gemini-pool"
CONTAINER_NAME="gemini-pool-container"
ENV_FILE="gemini-pool/.env"
DOCKERFILE="gemini-pool/Dockerfile"
EXTERNAL_PORT=${1:-8080} # Use the first argument as port, or default to 8080

# 1. Check for .env file
if [ ! -f "$ENV_FILE" ]; then
    echo "Error: Environment file '$ENV_FILE' not found."
    echo "Please create it by copying 'gemini-pool/.env.example' and adding your API keys."
    exit 1
fi

echo "✅ Environment file found."

# 2. Build the Docker image
echo "🚀 Building Docker image '$IMAGE_NAME'..."
docker build -t "$IMAGE_NAME" -f "$DOCKERFILE" gemini-pool/
echo "✅ Docker image built successfully."

# 3. Stop and remove any existing container
if [ "$(docker ps -a -q -f name=$CONTAINER_NAME)" ]; then
    echo "🛑 Stopping and removing existing container..."
    docker stop "$CONTAINER_NAME" > /dev/null 2>&1 || true
    docker rm "$CONTAINER_NAME" > /dev/null 2>&1 || true
    echo "✅ Existing container stopped and removed."
fi

# 4. Run the new Docker container in detached mode
echo "🚀 Starting new container '$CONTAINER_NAME' on port $EXTERNAL_PORT..."
docker run -d --name "$CONTAINER_NAME" --env-file "$ENV_FILE" -p "${EXTERNAL_PORT}:8080" -v gemini-pool-data:/data "$IMAGE_NAME"

echo "✅ Container started successfully!"
echo "👉 Your service is now available at http://127.0.0.1:${EXTERNAL_PORT}"
echo "👀 To view logs, run: docker logs -f $CONTAINER_NAME"
echo "🛑 To stop the container, run: docker stop $CONTAINER_NAME"
