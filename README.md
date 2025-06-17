# Git Chat Assistant Actor

A Theater actor that provides an interactive AI chat interface with git tools for managing repositories, creating commits, and understanding git workflows.

## Purpose

This actor serves as a domain-specific chat assistant that:
1. Spawns a chat-state actor as a child with git tool access
2. Automatically configures git MCP tools for repository management
3. Provides a specialized system prompt for git-related tasks
4. Acts as a proxy for chat interactions with git context

## Features

- **Interactive Git Assistant** - Chat with an AI that understands git workflows
- **Git Tool Access** - Built-in access to git commands through MCP actor
- **Smart Configuration** - Automatically enhances any base config with git tools
- **Commit Assistance** - Help with creating meaningful commit messages
- **Repository Analysis** - Understand repository state, history, and changes
- **Best Practices** - Guidance on git workflows and best practices

## Architecture

```
theater-chat → git-chat-assistant → chat-state (with git tools)
                        ↑                ↓
                        ├── GetChatStateActorId
                        └── AddMessage (forwarded with git context)
```

## Protocol

The actor implements the same protocol as `chat-proxy-example`:

### `GetChatStateActorId`
Returns the actor ID of the spawned chat-state actor with git tools.

### `AddMessage`
Forwards a message to the chat-state actor configured with git capabilities.

## Usage

### Building

```bash
cargo component build --release
```

### Configuration

The actor uses a predefined, optimized configuration for git workflows. No configuration is required when using this actor - it comes pre-configured with:

- **Model**: Claude Sonnet 4 (claude-sonnet-4-20250514)
- **Temperature**: 0.7 (balanced for code tasks)
- **Git-specific system prompt** with detailed instructions
- **Built-in git MCP server** for tool access
- **Optimized settings** for commit assistance and repository management

### Using with theater-chat

Create a simple configuration file that points to this actor:

```json
{
  "actor": {
    "manifest_path": "/path/to/git-chat-assistant/manifest.toml"
  }
}
```

Then run:
```bash
theater-chat --config git-config.json
```

## Example Interactions

```
👤 You: What's the current status of my repository?

🤖 Assistant: I'll check the git status for you.

🔧 git status

💬 You have 3 modified files and 1 new file:
- Modified: src/lib.rs, Cargo.toml, README.md  
- Untracked: new_feature.rs

Would you like me to help you stage these changes or create a commit?

👤 You: Help me create a good commit message for these changes

🤖 Assistant: Let me look at the specific changes first.

🔧 git diff --staged

💬 Based on the changes I see, here's a suggested commit message:

"feat: Add new feature module and update dependencies

- Add new_feature.rs with core functionality
- Update Cargo.toml dependencies for new features
- Update README.md with usage examples
- Refactor lib.rs for better modularity"

Would you like me to create this commit or would you prefer to modify the message?
```

## Git Capabilities

The assistant can help with:

### Repository Management
- Check repository status and changes
- View commit history and logs
- Analyze diffs and file changes
- Manage branches (create, switch, merge)

### Commit Workflow
- Stage and unstage files
- Create meaningful commit messages
- Review changes before committing
- Amend commits and manage history

### Collaboration
- Manage remotes and pushing/pulling
- Handle merge conflicts
- Code review workflows
- Branch management strategies

### Best Practices
- Commit message conventions
- Branching strategies (Git Flow, GitHub Flow)
- Code review processes
- Repository organization

## Configuration Enhancement

The actor automatically enhances any base configuration with:

1. **System Prompt Addition**: Adds git-specific instructions and capabilities
2. **Git Tools**: Includes git MCP server in the mcp_servers array
3. **Title Enhancement**: Sets appropriate title if none provided
4. **Tool Context**: Ensures the AI understands available git operations

## Implementation Details

### State Structure
```rust
struct GitChatState {
    actor_id: String,                    // This actor's ID
    chat_state_actor_id: Option<String>, // Child chat-state actor ID  
    original_config: Value,              // Enhanced chat config with git tools
}
```

### Initialization Flow
1. Parse base chat configuration from initial state (or use defaults)
2. Enhance configuration with git tools and context
3. Spawn chat-state actor with enhanced configuration
4. Store chat-state actor ID in our state

### Message Handling
- Same as `chat-proxy-example` but with git-enhanced configuration
- All messages forwarded to chat-state actor with git tool access

## Dependencies

- `/Users/colinrozzi/work/actor-registry/chat-state/manifest.toml` - Chat state actor
- `/Users/colinrozzi/work/actor-registry/git-mcp-actor/manifest.toml` - Git MCP tools

## Files

- `src/lib.rs` - Main actor implementation with git enhancement logic
- `src/protocol.rs` - Chat state protocol definitions
- `manifest.toml` - Theater actor manifest
- `wit/` - Component interface definitions

## Future Enhancements

- Repository detection and automatic configuration
- Project-specific git workflows
- Integration with GitHub/GitLab APIs
- Advanced conflict resolution assistance
- Code review automation
- Commit template management
