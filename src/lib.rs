#[allow(warnings)]
mod bindings;
mod protocol;

use bindings::exports::theater::simple::actor::Guest;
use bindings::exports::theater::simple::message_server_client::Guest as MessageServerClient;
use bindings::exports::theater::simple::supervisor_handlers::Guest as SupervisorHandlers;
use bindings::theater::simple::message_server_host::send;
use bindings::theater::simple::runtime::{log, shutdown};
use bindings::theater::simple::supervisor::spawn;
use bindings::theater::simple::types::{ChannelAccept, WitActorError};
use genai_types::Message;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec, Value};

struct Component;

// Protocol types for external communication
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum GitChatRequest {
    GetChatStateActorId,
    AddMessage { message: Message },
    StartChat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum GitChatResponse {
    ChatStateActorId { actor_id: String },
    Success,
    Error { message: String },
}

// Configuration for git assistant
#[derive(Serialize, Deserialize, Debug)]
struct GitAssistantConfig {
    current_directory: Option<String>,
    workflow: Option<String>,
    model_config: Option<Value>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    system_prompt: Option<String>,
    title: Option<String>,
    description: Option<String>,
    mcp_servers: Option<Value>,
    #[serde(flatten)]
    other: Value,
}

impl Default for GitAssistantConfig {
    fn default() -> Self {
        Self {
            current_directory: None,
            workflow: None,
            model_config: None,
            temperature: None,
            max_tokens: None,
            system_prompt: None,
            title: None,
            description: None,
            mcp_servers: None,
            other: serde_json::json!({}),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TaskComplete;

// State management
#[derive(Serialize, Deserialize, Debug)]
struct GitChatState {
    actor_id: String,
    chat_state_actor_id: Option<String>,
    original_config: Value,
    current_directory: Option<String>,
    workflow: Option<String>,
}

impl GitChatState {
    fn new(
        actor_id: String,
        config: Value,
        current_directory: Option<String>,
        workflow: Option<String>,
    ) -> Self {
        Self {
            actor_id,
            chat_state_actor_id: None,
            original_config: config,
            current_directory,
            workflow,
        }
    }

    fn set_chat_state_actor_id(&mut self, chat_actor_id: String) {
        self.chat_state_actor_id = Some(chat_actor_id);
    }

    fn get_chat_state_actor_id(&self) -> Result<&String, String> {
        self.chat_state_actor_id
            .as_ref()
            .ok_or_else(|| "Chat state actor not initialized".to_string())
    }
}

impl Guest for Component {
    fn init(state: Option<Vec<u8>>, params: (String,)) -> Result<(Option<Vec<u8>>,), String> {
        log("Git chat assistant actor initializing...");

        let (self_id,) = params;

        // Parse initial configuration if provided
        let (git_config, current_directory, workflow) = if let Some(state_bytes) = state {
            match from_slice::<GitAssistantConfig>(&state_bytes) {
                Ok(config) => {
                    log(&format!(
                        "Parsed initial config with current_directory: {:?}, workflow: {:?}",
                        config.current_directory, config.workflow
                    ));
                    let git_config = create_git_optimized_config(
                        &self_id,
                        config.current_directory.as_deref(),
                        &config,
                    );
                    (git_config, config.current_directory, config.workflow)
                }
                Err(e) => {
                    log(&format!(
                        "Failed to parse initial config, using defaults: {}",
                        e
                    ));
                    let git_config =
                        create_git_optimized_config(&self_id, None, &GitAssistantConfig::default());
                    (git_config, None, None)
                }
            }
        } else {
            log("No initial state provided, using default configuration");
            let git_config =
                create_git_optimized_config(&self_id, None, &GitAssistantConfig::default());
            (git_config, None, None)
        };

        log(&format!("Using git config: {}", git_config));

        // Create our state
        let mut git_state =
            GitChatState::new(self_id, git_config.clone(), current_directory, workflow);

        // Spawn the chat-state actor with the git config
        match spawn_chat_state_actor(&git_config) {
            Ok(chat_actor_id) => {
                log(&format!("Chat state actor spawned: {}", chat_actor_id));
                git_state.set_chat_state_actor_id(chat_actor_id);
            }
            Err(e) => {
                let error_msg = format!("Failed to spawn chat state actor: {}", e);
                log(&error_msg);
                return Err(error_msg);
            }
        }

        // Serialize our state
        let state_bytes =
            to_vec(&git_state).map_err(|e| format!("Failed to serialize git state: {}", e))?;

        log("Git chat assistant actor initialized successfully");
        Ok((Some(state_bytes),))
    }
}

impl SupervisorHandlers for Component {
    fn handle_child_error(
        state: Option<Vec<u8>>,
        params: (String, WitActorError),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (child_id, error) = params;
        log(&format!("Child error from {}: {:?}", child_id, error));
        Ok((state,))
    }

    fn handle_child_exit(
        state: Option<Vec<u8>>,
        params: (String, Option<Vec<u8>>),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (child_id, _exit_state) = params;
        log(&format!("Child exit: {}", child_id));
        Ok((state,))
    }
}

impl MessageServerClient for Component {
    fn handle_send(
        state: Option<Vec<u8>>,
        _params: (Vec<u8>,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        log("Git chat assistant handling send message");

        let parsed_state = match state {
            Some(state_bytes) => serde_json::from_slice::<Vec<u8>>(&state_bytes)
                .map_err(|e| format!("Failed to deserialize state: {}", e))?,
            None => {
                log("No state available for send message");
                return Err("No state available".to_string());
            }
        };

        match from_slice::<TaskComplete>(&parsed_state) {
            Ok(msg) => {
                log(&format!("Received task completion message: {:?}", msg));

                let _ = shutdown(None);
            }
            Err(e) => {
                let error_msg = format!("Failed to parse message: {}", e);
                log(&error_msg);
                return Err(error_msg);
            }
        };

        let updated_state = to_vec(&parsed_state)
            .map_err(|e| format!("Failed to serialize updated state: {}", e))?;
        Ok((Some(updated_state),))
    }

    fn handle_request(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        log("Git chat assistant handling request message");

        let (_request_id, data) = params;

        // Deserialize our state
        let git_state: GitChatState = match state {
            Some(state_bytes) => match from_slice(&state_bytes) {
                Ok(state) => state,
                Err(e) => {
                    let error_msg = format!("Failed to deserialize git state: {}", e);
                    log(&error_msg);
                    let error_response = GitChatResponse::Error { message: error_msg };
                    let response_bytes = to_vec(&error_response)
                        .map_err(|e| format!("Failed to serialize error response: {}", e))?;
                    return Ok((None, (Some(response_bytes),)));
                }
            },
            None => {
                let error_msg = "No state available";
                log(error_msg);
                let error_response = GitChatResponse::Error {
                    message: error_msg.to_string(),
                };
                let response_bytes = to_vec(&error_response)
                    .map_err(|e| format!("Failed to serialize error response: {}", e))?;
                return Ok((None, (Some(response_bytes),)));
            }
        };

        // Parse the request
        let request: GitChatRequest = match from_slice(&data) {
            Ok(req) => {
                log(&format!("Parsed request: {:?}", req));
                req
            }
            Err(e) => {
                let error_msg = format!("Failed to parse request: {}", e);
                log(&error_msg);
                let error_response = GitChatResponse::Error { message: error_msg };
                let response_bytes = to_vec(&error_response)
                    .map_err(|e| format!("Failed to serialize error response: {}", e))?;
                return Ok((
                    Some(to_vec(&git_state).unwrap_or_default()),
                    (Some(response_bytes),),
                ));
            }
        };

        // Handle the request
        let response = match request {
            GitChatRequest::StartChat => {
                log("Starting chat session...");

                // Check if we have a workflow that requires auto-initiation
                if let Some(workflow) = &git_state.workflow {
                    if workflow == "commit" {
                        log("Auto-initiating commit workflow");

                        match git_state.get_chat_state_actor_id() {
                            Ok(chat_actor_id) => {
                                let auto_message = protocol::ChatStateRequest::AddMessage {
                                    message: Message {
                                        role: genai_types::messages::Role::User,
                                        content: vec![genai_types::MessageContent::Text {
                                            text: "Please analyze the current repository state and commit any pending changes with appropriate commit messages. Start by checking git status to see what files have changed.".to_string()
                                        }],
                                    },
                                };

                                let message_bytes = to_vec(&auto_message).map_err(|e| {
                                    format!("Failed to serialize auto message: {}", e)
                                })?;

                                match send(chat_actor_id, &message_bytes) {
                                    Ok(_) => {
                                        log("Auto commit message sent successfully");

                                        // Request generation from chat-state actor
                                        let generation_request =
                                            protocol::ChatStateRequest::GenerateCompletion;
                                        let generation_request_bytes = to_vec(&generation_request)
                                            .map_err(|e| {
                                                format!(
                                                    "Failed to serialize generation request: {}",
                                                    e
                                                )
                                            })?;

                                        match send(chat_actor_id, &generation_request_bytes) {
                                            Ok(_) => {
                                                log("Auto generation request sent successfully");
                                            }
                                            Err(e) => {
                                                let error_msg = format!(
                                                    "Failed to send auto generation request: {:?}",
                                                    e
                                                );
                                                log(&error_msg);
                                                return Ok((
                                                    Some(to_vec(&git_state).unwrap_or_default()),
                                                    (Some(
                                                        to_vec(&GitChatResponse::Error {
                                                            message: error_msg,
                                                        })
                                                        .unwrap_or_default(),
                                                    ),),
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let error_msg =
                                            format!("Failed to send auto commit message: {:?}", e);
                                        log(&error_msg);
                                        return Ok((
                                            Some(to_vec(&git_state).unwrap_or_default()),
                                            (Some(
                                                to_vec(&GitChatResponse::Error {
                                                    message: error_msg,
                                                })
                                                .unwrap_or_default(),
                                            ),),
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!(
                                    "Chat state actor not available for auto workflow: {}",
                                    e
                                );
                                log(&error_msg);
                                return Ok((
                                    Some(to_vec(&git_state).unwrap_or_default()),
                                    (Some(
                                        to_vec(&GitChatResponse::Error { message: error_msg })
                                            .unwrap_or_default(),
                                    ),),
                                ));
                            }
                        }
                    } else {
                        log(&format!(
                            "Workflow '{}' does not require auto-initiation",
                            workflow
                        ));
                    }
                } else {
                    log("No workflow specified, starting normal chat session");
                }

                GitChatResponse::Success
            }
            GitChatRequest::GetChatStateActorId => match git_state.get_chat_state_actor_id() {
                Ok(actor_id) => {
                    log(&format!("Returning chat state actor ID: {}", actor_id));
                    GitChatResponse::ChatStateActorId {
                        actor_id: actor_id.clone(),
                    }
                }
                Err(e) => {
                    log(&format!("Error getting chat state actor ID: {}", e));
                    GitChatResponse::Error { message: e }
                }
            },
            GitChatRequest::AddMessage { message } => {
                match git_state.get_chat_state_actor_id() {
                    Ok(chat_actor_id) => {
                        log(&format!(
                            "Forwarding message to chat state actor: {}",
                            chat_actor_id
                        ));

                        let add_message = protocol::ChatStateRequest::AddMessage {
                            message: message.clone(),
                        };

                        // Forward the message to the chat-state actor
                        let message_bytes = to_vec(&add_message)
                            .map_err(|e| format!("Failed to serialize message: {}", e))?;

                        match send(chat_actor_id, &message_bytes) {
                            Ok(_) => {
                                log("Message forwarded successfully");

                                // Request generation from chat-state actor
                                let generation_request_message =
                                    protocol::ChatStateRequest::GenerateCompletion;
                                let generation_request_bytes = to_vec(&generation_request_message)
                                    .map_err(|e| {
                                        format!("Failed to serialize generation request: {}", e)
                                    })?;

                                match send(chat_actor_id, &generation_request_bytes) {
                                    Ok(_) => {
                                        log("Generation request sent successfully");
                                        GitChatResponse::Success
                                    }
                                    Err(e) => {
                                        let error_msg =
                                            format!("Failed to send generation request: {:?}", e);
                                        log(&error_msg);
                                        GitChatResponse::Error { message: error_msg }
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Failed to forward message: {:?}", e);
                                log(&error_msg);
                                GitChatResponse::Error { message: error_msg }
                            }
                        }
                    }
                    Err(e) => {
                        log(&format!("Error forwarding message: {}", e));
                        GitChatResponse::Error { message: e }
                    }
                }
            }
        };

        // Serialize the response
        let response_bytes =
            to_vec(&response).map_err(|e| format!("Failed to serialize response: {}", e))?;

        // Keep the same state (no changes needed)
        let current_state_bytes =
            to_vec(&git_state).map_err(|e| format!("Failed to serialize current state: {}", e))?;

        Ok((Some(current_state_bytes), (Some(response_bytes),)))
    }

    fn handle_channel_open(
        state: Option<Vec<u8>>,
        _params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (ChannelAccept,)), String> {
        log("Git chat assistant: Channel open request");
        Ok((
            state,
            (ChannelAccept {
                accepted: true,
                message: None,
            },),
        ))
    }

    fn handle_channel_close(
        state: Option<Vec<u8>>,
        params: (String,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (channel_id,) = params;
        log(&format!(
            "Git chat assistant: Channel closed: {}",
            channel_id
        ));
        Ok((state,))
    }

    fn handle_channel_message(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (channel_id, _message) = params;
        log(&format!(
            "Git chat assistant: Received channel message on: {}",
            channel_id
        ));
        Ok((state,))
    }
}

// Helper functions
fn create_git_optimized_config(
    self_id: &str,
    current_directory: Option<&str>,
    config: &GitAssistantConfig,
) -> Value {
    log("Creating git-optimized configuration...");

    // Build directory context if provided
    let directory_context = match current_directory {
        Some(dir) => {
            log(&format!("Including current directory context: {}", dir));
            format!("\n\nCURRENT WORKING DIRECTORY: {}\nWhen using git tools, operate within this directory unless explicitly told otherwise. This is the repository you should focus on for all git operations.", dir)
        }
        None => {
            log("No current directory specified");
            String::new()
        }
    };

    // Build workflow context if provided
    let workflow_context = match config.workflow.as_deref() {
        Some("commit") => {
            log("Adding commit workflow context");
            "\n\nWORKFLOW: AUTOMATIC COMMIT\n\
            Your primary task is to analyze the current repository state and create appropriate commits:\n\
            1. First, check git status to see what files have changed\n\
            2. Review the actual changes using git diff\n\
            3. Stage appropriate files for commit\n\
            4. Create meaningful, conventional commit messages\n\
            5. Execute the commit\n\
            \n\
            Focus on creating clean, atomic commits with descriptive messages. \
            If there are multiple logical changes, consider separate commits. \
            Always explain what you're doing and why."
        }
        Some("review") => {
            log("Adding review workflow context");
            "\n\nWORKFLOW: CODE REVIEW\n\
            Your primary task is to review code changes and provide feedback:\n\
            1. Check git status and diff to understand changes\n\
            2. Analyze code quality, style, and potential issues\n\
            3. Suggest improvements and optimizations\n\
            4. Check for security vulnerabilities or bugs\n\
            5. Provide constructive feedback\n\
            \n\
            Focus on helping improve code quality while being constructive and educational."
        }
        Some("rebase") => {
            log("Adding rebase workflow context");
            "\n\nWORKFLOW: INTERACTIVE REBASE\n\
            Your primary task is to help with git rebase operations:\n\
            1. Understand the current branch state and history\n\
            2. Help plan rebase strategies\n\
            3. Assist with conflict resolution\n\
            4. Guide through interactive rebase steps\n\
            5. Ensure clean, linear history\n\
            \n\
            Focus on maintaining a clean git history while preserving important changes."
        }
        Some(workflow) => {
            log(&format!(
                "Unknown workflow type: {}, using default behavior",
                workflow
            ));
            ""
        }
        None => {
            log("No workflow specified");
            ""
        }
    };

    // Default git system prompt
    let default_git_system_prompt = format!("You are a Git assistant with access to git tools. You can help with:

- Reviewing git status and changes
- Creating meaningful commit messages
- Managing branches and repositories
- Analyzing git history and diffs
- Staging and unstaging files
- Handling merge conflicts
- Git workflow best practices

You have access to git tools that allow you to interact with the repository. Always be helpful and provide clear explanations of git operations.

When helping with commits:
- Always review the changes first before suggesting commit messages
- Create descriptive, conventional commit messages
- Suggest appropriate files to stage if not already staged
- Explain the impact of changes when relevant{}{}", directory_context, workflow_context);

    // Use custom system prompt if provided, otherwise use default with directory and workflow context
    let final_system_prompt = match &config.system_prompt {
        Some(custom_prompt) => {
            log("Using custom system prompt with context");
            format!("{}{}{}", custom_prompt, directory_context, workflow_context)
        }
        None => {
            log("Using default git system prompt with workflow context");
            default_git_system_prompt
        }
    };

    // Default model config
    let default_model_config = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "provider": "anthropic"
    });

    // Default MCP servers (git tools)
    let default_mcp_servers = serde_json::json!([
        {
            "actor_id": null,
            "actor": {
                "manifest_path": "/Users/colinrozzi/work/actor-registry/git-mcp-actor/manifest.toml"
            },
            "tools": null
        },
        {
            "actor_id": null,
            "actor": {
                "manifest_path": "/Users/colinrozzi/work/actor-registry/task-monitor-mcp-actor/manifest.toml"
            },
            "config": {
                "management_actor": self_id,
            },
            "tools": null
        }
    ]);

    // Build the configuration with overrides
    let model_config = config
        .model_config
        .as_ref()
        .unwrap_or(&default_model_config);
    let temperature = config.temperature.unwrap_or(0.7);
    let max_tokens = config.max_tokens.unwrap_or(8192);
    let title = config
        .title
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Git Assistant");
    let description =
        config.description.as_ref().map(|s| s.as_str()).unwrap_or(
            "AI assistant with git tools for repository management and commit workflows",
        );
    let mcp_servers = config.mcp_servers.as_ref().unwrap_or(&default_mcp_servers);

    log(&format!("Using model: {:?}", model_config));
    log(&format!("Using temperature: {}", temperature));
    log(&format!("Using max_tokens: {}", max_tokens));
    log(&format!("Using title: {}", title));

    // Build the final configuration
    let mut final_config = serde_json::json!({
        "model_config": model_config,
        "temperature": temperature,
        "max_tokens": max_tokens,
        "system_prompt": final_system_prompt,
        "title": title,
        "description": description,
        "mcp_servers": mcp_servers
    });

    // Merge any additional fields from the other config
    if let Some(obj) = final_config.as_object_mut() {
        if let Value::Object(other_map) = &config.other {
            for (key, value) in other_map {
                if !obj.contains_key(key) {
                    obj.insert(key.clone(), value.clone());
                }
            }
        }
    }

    log(&format!("Created final git config: {}", final_config));
    final_config
}

fn spawn_chat_state_actor(chat_config: &Value) -> Result<String, String> {
    log("Spawning chat-state actor...");

    let manifest_path = "/Users/colinrozzi/work/actor-registry/chat-state/manifest.toml";

    // Create initial state for chat-state actor
    let initial_state = serde_json::json!({
        "config": chat_config
    });

    let initial_state_bytes = to_vec(&initial_state)
        .map_err(|e| format!("Failed to serialize chat-state config: {}", e))?;

    // Spawn the actor
    match spawn(manifest_path, Some(&initial_state_bytes)) {
        Ok(actor_id) => {
            log(&format!(
                "Successfully spawned chat-state actor: {}",
                actor_id
            ));
            Ok(actor_id)
        }
        Err(e) => {
            log(&format!("Failed to spawn chat-state actor: {:?}", e));
            Err(format!("Spawn failed: {:?}", e))
        }
    }
}

bindings::export!(Component with_types_in bindings);
