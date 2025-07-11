#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
PORT=${1:-8080} # Use the first argument as port, or default to 8080
ENV_FILE="gemini-pool/.env"
PROJECT_DIR="gemini-pool"
BINARY_NAME="gemini-pool"
BINARY_PATH="$PROJECT_DIR/target/release/$BINARY_NAME"

# --- 1. Check for .env file ---
if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Error: Environment file '$ENV_FILE' not found."
    echo "Please create it by copying '$PROJECT_DIR/.env.example' and adding your API keys."
    exit 1
fi
echo "✅ Environment file found."

# --- 2. Compile the application ---
echo "🚀 Compiling the application in release mode (this may take a moment)..."
# Run the build command from within the project directory
(cd "$PROJECT_DIR" && cargo build --release)
echo "✅ Application compiled successfully."

# --- 3. Run the application ---
echo "🚀 Starting the server on http://127.0.0.1:${PORT}"
echo "   (The server binds to 0.0.0.0:${PORT} to be accessible from other devices on the network)"
echo "   Press Ctrl+C to stop the server."

# Load variables from the .env file first.
# This ensures GEMINI_API_KEYS is available to the process.
set -o allexport
source "$ENV_FILE"
set +o allexport

# Set the listen address, overriding any value from the .env file.
# This ensures the script's port argument is respected.
export LISTEN_ADDR="0.0.0.0:${PORT}"

"$BINARY_PATH"
