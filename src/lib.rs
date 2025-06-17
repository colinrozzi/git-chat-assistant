#[allow(warnings)]
mod bindings;
mod protocol;

use bindings::exports::theater::simple::actor::Guest;
use bindings::exports::theater::simple::message_server_client::Guest as MessageServerClient;
use bindings::exports::theater::simple::supervisor_handlers::Guest as SupervisorHandlers;
use bindings::theater::simple::message_server_host::send;
use bindings::theater::simple::runtime::log;
use bindings::theater::simple::supervisor::spawn;
use bindings::theater::simple::types::{ChannelAccept, WitActorError};
use genai_types::Message;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_slice, to_vec};

struct Component;

// Protocol types for external communication
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum GitChatRequest {
    GetChatStateActorId,
    AddMessage { message: Message },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum GitChatResponse {
    ChatStateActorId { actor_id: String },
    Success,
    Error { message: String },
}

// State management
#[derive(Serialize, Deserialize, Debug)]
struct GitChatState {
    actor_id: String,
    chat_state_actor_id: Option<String>,
    original_config: Value,
}

impl GitChatState {
    fn new(actor_id: String, config: Value) -> Self {
        Self {
            actor_id,
            chat_state_actor_id: None,
            original_config: config,
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

        let (actor_id,) = params;

        // Parse the initial state (base config, we'll enhance it with git tools)
        let base_config = match state {
            Some(state_bytes) => match from_slice::<Value>(&state_bytes) {
                Ok(config) => {
                    log(&format!("Parsed base config: {}", config));
                    config
                }
                Err(e) => {
                    let error_msg = format!("Failed to parse initial state: {}", e);
                    log(&error_msg);
                    return Err(error_msg);
                }
            },
            None => {
                log("No initial state provided, using default git config");
                serde_json::json!({
                    "model_config": {
                        "model": "claude-sonnet-4-20250514",
                        "provider": "anthropic"
                    },
                    "temperature": 1.0,
                    "max_tokens": 8192,
                    "title": "Git Assistant"
                })
            }
        };

        // Enhance the config with git-specific settings
        let enhanced_config = enhance_config_with_git_tools(base_config)?;

        // Create our state
        let mut git_state = GitChatState::new(actor_id, enhanced_config.clone());

        // Spawn the chat-state actor with the enhanced config
        match spawn_chat_state_actor(&enhanced_config) {
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
        let state_bytes = to_vec(&git_state)
            .map_err(|e| format!("Failed to serialize git state: {}", e))?;

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
        Ok((state,))
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
        let current_state_bytes = to_vec(&git_state)
            .map_err(|e| format!("Failed to serialize current state: {}", e))?;

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
        log(&format!("Git chat assistant: Channel closed: {}", channel_id));
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
fn enhance_config_with_git_tools(mut base_config: Value) -> Result<Value, String> {
    log("Enhancing config with git tools...");

    // Set a git-specific system prompt if none exists or enhance existing one
    let git_system_prompt = "You are a Git assistant with access to git tools. You can help with:

- Reviewing git status and changes
- Creating meaningful commit messages
- Managing branches and repositories
- Analyzing git history and diffs
- Staging and unstaging files
- Handling merge conflicts
- Git workflow best practices

You have access to git tools that allow you to interact with the repository. Always be helpful and provide clear explanations of git operations.";

    // If there's an existing system prompt, enhance it with git context
    if let Some(existing_prompt) = base_config.get("system_prompt").and_then(|p| p.as_str()) {
        let enhanced_prompt = format!("{}\n\n{}", existing_prompt, git_system_prompt);
        base_config["system_prompt"] = serde_json::Value::String(enhanced_prompt);
    } else {
        base_config["system_prompt"] = serde_json::Value::String(git_system_prompt.to_string());
    }

    // Set a git-specific title if none exists
    if base_config.get("title").is_none() {
        base_config["title"] = serde_json::Value::String("Git Assistant".to_string());
    }

    // Add git MCP server to the configuration
    let git_mcp_server = serde_json::json!({
        "actor_id": null,
        "actor": {
            "manifest_path": "/Users/colinrozzi/work/actor-registry/git-mcp-actor/manifest.toml"
        },
        "tools": null
    });

    // Add to existing mcp_servers or create new array
    if let Some(mcp_servers) = base_config.get_mut("mcp_servers") {
        if let Some(servers_array) = mcp_servers.as_array_mut() {
            servers_array.push(git_mcp_server);
        }
    } else {
        base_config["mcp_servers"] = serde_json::Value::Array(vec![git_mcp_server]);
    }

    log(&format!("Enhanced config: {}", base_config));
    Ok(base_config)
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
