/// LRGP GameApp trait — the interface all LRGP games must implement.

use std::collections::HashMap;

use serde_json::Value as JsonValue;

/// Result returned by `handle_incoming`.
#[derive(Debug, Clone)]
pub struct IncomingResult {
    /// Updated session dict, or None.
    pub session: Option<HashMap<String, JsonValue>>,
    /// Event to emit to the UI, or None.
    pub emit: Option<HashMap<String, JsonValue>>,
    /// Error info, or None.
    pub error: Option<HashMap<String, JsonValue>>,
}

/// Result returned by `handle_outgoing`.
#[derive(Debug, Clone)]
pub struct OutgoingResult {
    /// Enriched payload to pack into the envelope.
    pub payload: HashMap<String, rmpv::Value>,
    /// Human-readable fallback text for non-LRGP clients.
    pub fallback_text: String,
}

/// Game manifest describing an LRGP game's capabilities.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameManifest {
    pub app_id: String,
    pub version: u32,
    pub display_name: String,
    pub icon: String,
    pub session_type: String,
    pub max_players: u32,
    pub min_players: u32,
    pub validation: String,
    pub actions: Vec<String>,
    pub preferred_delivery: HashMap<String, String>,
    pub ttl: HashMap<String, f64>,
    /// Optional genre tag for game categorization (e.g., "strategy", "puzzle", "card").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    /// Optional per-turn time limit in seconds. `None` means no limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_timeout: Option<f64>,
}

/// The trait all LRGP games must implement.
pub trait GameApp: Send + Sync {
    fn app_id(&self) -> &str;
    fn version(&self) -> u32;
    fn manifest(&self) -> GameManifest;

    /// Process an incoming LRGP game action.
    fn handle_incoming(
        &self,
        session_id: &str,
        command: &str,
        payload: &HashMap<String, rmpv::Value>,
        sender_hash: &str,
        identity_id: &str,
    ) -> IncomingResult;

    /// Prepare an outgoing LRGP game action.
    fn handle_outgoing(
        &self,
        session_id: &str,
        command: &str,
        payload: &HashMap<String, rmpv::Value>,
        identity_id: &str,
    ) -> OutgoingResult;

    /// Validate an action. Returns (valid, error_message).
    fn validate_action(
        &self,
        session_id: &str,
        command: &str,
        payload: &HashMap<String, rmpv::Value>,
        sender_hash: &str,
    ) -> (bool, Option<String>);

    /// Return current session state for rendering.
    fn get_session_state(
        &self,
        session_id: &str,
        identity_id: &str,
    ) -> HashMap<String, JsonValue>;

    /// Generate human-readable fallback text for LXMF content field.
    fn render_fallback(
        &self,
        command: &str,
        payload: &HashMap<String, rmpv::Value>,
    ) -> String;

    /// Return preferred delivery method for this command.
    fn get_delivery_method(&self, command: &str) -> String {
        let _ = command;
        "opportunistic".to_string()
    }
}
