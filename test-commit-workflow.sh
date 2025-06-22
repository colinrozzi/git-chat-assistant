#!/bin/bash
set -e

# Test script for the git chat assistant with workflow automation

echo "🚀 Testing Git Chat Assistant with Commit Workflow"
echo "================================================="

# Check if theater-chat is available
if ! command -v theater-chat &> /dev/null; then
    echo "❌ theater-chat not found. Please install it first."
    echo "   cd /Users/colinrozzi/work/tools/theater-chat"
    echo "   bun install && bun run build"
    exit 1
fi

# Check if manifest exists
MANIFEST_PATH="/Users/colinrozzi/work/actor-registry/git-chat-assistant/manifest.toml"
if [ ! -f "$MANIFEST_PATH" ]; then
    echo "❌ Git chat assistant manifest not found at: $MANIFEST_PATH"
    exit 1
fi

# Check if the component is built
COMPONENT_PATH="/Users/colinrozzi/work/actor-registry/git-chat-assistant/target/wasm32-wasip1/release/git_chat_assistant.wasm"
if [ ! -f "$COMPONENT_PATH" ]; then
    echo "❌ Git chat assistant component not built. Building now..."
    cd /Users/colinrozzi/work/actor-registry/git-chat-assistant
    cargo component build --release
    if [ $? -ne 0 ]; then
        echo "❌ Failed to build component"
        exit 1
    fi
fi

# Test directory
TEST_DIR="/Users/colinrozzi/work/tools/commit"

echo "✅ Prerequisites checked"
echo "📁 Test directory: $TEST_DIR"
echo "⚙️  Configuration: theater-commit-config.json"
echo ""
echo "🎯 This will start an automated commit workflow that will:"
echo "   1. Analyze the repository state"
echo "   2. Review any pending changes" 
echo "   3. Create appropriate commit messages"
echo "   4. Execute commits with explanations"
echo ""
echo "Press Ctrl+C to exit the session when done."
echo ""

# Run theater-chat with the commit workflow config
cd /Users/colinrozzi/work/actor-registry/git-chat-assistant
theater-chat --config theater-commit-config.json
