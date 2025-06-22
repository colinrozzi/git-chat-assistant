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

The actor comes with optimized defaults for git workflows, but **every setting can be customized** through the initial configuration:

**Default Configuration:**
- **Model**: Claude Sonnet 4 (claude-sonnet-4-20250514)
- **Temperature**: 0.7 (balanced for code tasks)
- **Max Tokens**: 8192
- **Git-specific system prompt** with detailed instructions
- **Built-in git MCP server** for tool access
- **Title**: "Git Assistant"

**Fully Customizable:**
- **Any AI model/provider** (Claude, GPT-4, Gemini, etc.)
- **Temperature and token limits**
- **Custom system prompts** (with automatic directory context)
- **Session titles and descriptions**
- **Current directory** for repository context
- **Workflow automation** (commit, review, rebase)
- **MCP server configurations** (advanced)

### Using with theater-chat

#### Basic Usage (No Directory Context)

Create a simple configuration file that points to this actor:

```json
{
  "actor": {
    "manifest_path": "/path/to/git-chat-assistant/manifest.toml"
  }
}
```

#### With Current Directory Context (Recommended)

Create a configuration file that includes the repository path:

```json
{
  "actor": {
    "manifest_path": "/path/to/git-chat-assistant/manifest.toml",
    "initial_state": {
      "current_directory": "/path/to/your/repository"
    }
  }
}
```

#### Automated Commit Workflow (New!)

For automated commit workflows, specify the `workflow` field:

```json
{
  "actor": {
    "manifest_path": "/path/to/git-chat-assistant/manifest.toml",
    "initial_state": {
      "current_directory": "/path/to/your/repository",
      "workflow": "commit",
      "temperature": 0.3,
      "title": "Auto-Commit Assistant"
    }
  }
}
```

Then run:
```bash
theater-chat --config git-config.json
```

With directory context, the assistant will know exactly which repository to operate on and can provide more targeted assistance. With workflow automation, the assistant will automatically start the specified workflow when the session begins.

## Example Interactions

### Interactive Git Assistant

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

### Automated Commit Workflow

```
🚀 Auto-Commit Assistant: Starting automated commit workflow...

🔧 git status

💬 I can see you have several changes ready for commit:
- Modified: src/workflow.rs, README.md
- Added: commit-workflow-config.json

Let me analyze these changes and create appropriate commits.

🔧 git diff src/workflow.rs

🔧 git add src/workflow.rs

🔧 git commit -m "feat: Add automated workflow support

- Add workflow field to GitAssistantConfig
- Implement commit workflow auto-initiation
- Add workflow-specific system prompts"

🔧 git add README.md commit-workflow-config.json

🔧 git commit -m "docs: Update documentation for workflow feature

- Add workflow configuration examples
- Document automated commit workflow
- Add example configuration files"

✅ Successfully created 2 commits with meaningful messages!
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
2. **Directory Context**: Includes current working directory in the system prompt if provided
3. **Git Tools**: Includes git MCP server in the mcp_servers array
4. **Title Enhancement**: Sets appropriate title if none provided
5. **Tool Context**: Ensures the AI understands available git operations

### Configuration Format

When providing initial state, you can override any of the default settings:

```json
{
  "current_directory": "/path/to/repository",
  "model_config": {
    "model": "gpt-4",
    "provider": "openai"
  },
  "temperature": 0.5,
  "max_tokens": 4096,
  "title": "Custom Git Assistant",
  "description": "My specialized git helper",
  "system_prompt": "You are an expert Git consultant..."
}
```

**All fields are optional** - the actor will use sensible defaults for any missing configuration.

#### Supported Configuration Options:

- **`current_directory`** (string): Repository path for context
- **`workflow`** (string): Automated workflow type ("commit", "review", "rebase")
- **`model_config`** (object): Model and provider settings
  - `model`: Model name (e.g., "claude-sonnet-4-20250514", "gpt-4", "gemini-1.5-pro")
  - `provider`: Provider name ("anthropic", "openai", "google")
- **`temperature`** (number): Sampling temperature (0.0-2.0, default: 0.7)
- **`max_tokens`** (number): Maximum response tokens (default: 8192)
- **`title`** (string): Chat session title (default: "Git Assistant")
- **`description`** (string): Assistant description
- **`system_prompt`** (string): Custom system prompt (will include directory context if provided)
- **`mcp_servers`** (array): Override MCP server configuration (advanced)

#### Configuration Examples:

**Minimal (just directory):**
```json
{
  "current_directory": "/path/to/repo"
}
```

**Automated commit workflow:**
```json
{
  "current_directory": "/path/to/repo",
  "workflow": "commit",
  "temperature": 0.3,
  "title": "Auto-Commit Assistant"
}
```

**Code review workflow:**
```json
{
  "current_directory": "/path/to/repo",
  "workflow": "review",
  "temperature": 0.5,
  "title": "Code Review Assistant"
}
```

**Different model:**
```json
{
  "current_directory": "/path/to/repo",
  "model_config": {
    "model": "gpt-4",
    "provider": "openai"
  },
  "temperature": 0.3
}
```

**Custom system prompt:**
```json
{
  "current_directory": "/path/to/repo",
  "system_prompt": "You are a senior DevOps engineer specializing in Git workflows for large enterprise teams. Focus on automation and best practices.",
  "title": "Enterprise Git Consultant"
}
```

## Implementation Details

### State Structure
```rust
struct GitChatState {
    actor_id: String,                    // This actor's ID
    chat_state_actor_id: Option<String>, // Child chat-state actor ID  
    original_config: Value,              // Enhanced chat config with git tools
    current_directory: Option<String>,   // Repository directory context
    workflow: Option<String>,            // Automated workflow type
}
```

### Initialization Flow
1. Parse base chat configuration from initial state (or use defaults)
2. Extract current directory and workflow if provided
3. Enhance configuration with git tools, directory context, and workflow-specific prompts
4. Add directory path and workflow context to system prompt
5. Spawn chat-state actor with enhanced configuration
6. Store chat-state actor ID, directory, and workflow in our state
7. Auto-initiate workflow if specified (e.g., start commit analysis for "commit" workflow)

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

## Workflow Types

### Commit Workflow (`"workflow": "commit"`)
Automatically analyzes repository state and creates appropriate commits:
- Checks `git status` to identify changes
- Reviews diffs to understand modifications
- Stages files appropriately
- Creates meaningful, conventional commit messages
- Executes commits with explanations

### Review Workflow (`"workflow": "review"`)
Provides comprehensive code review:
- Analyzes code changes for quality and style
- Identifies potential bugs and security issues
- Suggests improvements and optimizations
- Provides constructive feedback
- Checks for best practices

### Rebase Workflow (`"workflow": "rebase"`)
Assists with git rebase operations:
- Plans rebase strategies
- Helps resolve merge conflicts
- Guides through interactive rebase steps
- Ensures clean, linear history
- Maintains important changes

## Future Enhancements

- Repository detection and automatic configuration
- Project-specific git workflows
- Integration with GitHub/GitLab APIs
- Advanced conflict resolution assistance
- Pre-commit hook integration
- Commit template management
- Multi-repository batch operations
