# LRGP Specification v0.2

**Lightweight Reticulum Gaming Protocol**

This document is the normative reference for LRGP. It is implementable without seeing the Rust or Python reference code.

---

## 1. Overview

LRGP defines how multiplayer game sessions are encoded as LXMF messages over Reticulum. Clients that don't understand LRGP see human-readable fallback text in the standard LXMF content field.

LRGP v1 is **2-player only**. All sessions have exactly one initiator and one responder.

---

## 2. LXMF Field Allocation

LRGP uses two LXMF custom extension fields:

| Field | ID | Value |
|-------|----|-------|
| `FIELD_CUSTOM_TYPE` | `0xFB` (251) | `"lrgp.v1"` |
| `FIELD_CUSTOM_META` | `0xFD` (253) | Envelope dict (see Section 3) |

All fields are serialized via **msgpack** (not JSON).

### Legacy Markers

Implementations MUST also recognize the following legacy markers on inbound messages:
- `"rlap.v1"` — prior protocol version
- `"ratspeak.game"` — legacy v0

All outbound messages MUST use `"lrgp.v1"`.

---

## 3. Envelope Schema

The envelope is a msgpack dict stored in `fields[0xFD]`:

```
{
    "a": "<game_id>.<version>",    # e.g. "ttt.1"
    "c": "<command>",              # e.g. "move"
    "s": "<session_id>",          # 16-char hex (8 random bytes)
    "p": { <payload> }            # game-specific, short keys
}
```

All keys are single characters to minimize wire size. The `game_id` and `version` are combined into a single string to save one key-value pair.

### Required Fields

All four keys (`a`, `c`, `s`, `p`) MUST be present in every envelope.

### Session ID

Session IDs are 8 random bytes encoded as 16 hexadecimal characters. The challenger generates the session ID.

---

## 4. Size Constraints

| Limit | Value | Source |
|-------|-------|--------|
| Envelope (packed) | max **200 bytes** | LRGP budget rule |
| OPPORTUNISTIC content | max **295 bytes** | `LXMessage.ENCRYPTED_PACKET_MAX_CONTENT` |
| DIRECT packet content | max **319 bytes** | `LXMessage.LINK_PACKET_MAX_CONTENT` |
| LXMF overhead | **112 bytes** | 16B dest + 16B src + 64B sig + 8B ts + 8B structure |

LXMF content is packed as `[timestamp, title, content, fields_dict]`.

If content exceeds 295 bytes, LXMF silently escalates from OPPORTUNISTIC to DIRECT delivery, which requires a full Reticulum link handshake. LRGP envelopes MUST be designed to fit within OPPORTUNISTIC limits.

---

## 5. Fallback Text

The LXMF `content` field IS the fallback text. There is no separate fallback key in the envelope.

Format: `[LRGP <GameName>] <description>`

Examples:
- `[LRGP TTT] Sent a challenge!`
- `[LRGP TTT] Move 3`
- `[LRGP TTT] X wins!`

Non-LRGP clients display this as a regular message.

---

## 6. Session Lifecycle

### State Machine

```
challenge --> accept --> action* --> end
    |                      |
    +-> decline            +-> resign
    |                      +-> draw_offer --> draw_accept
    +-> expire (local)     |               +-> draw_decline
                           +-> error (receiver -> sender)
```

### Commands

| Command | Description |
|---------|-------------|
| `challenge` | Initiate a new game session |
| `accept` | Accept a challenge |
| `decline` | Decline a challenge |
| `move` | Game-specific action (e.g., place a piece) |
| `resign` | Voluntary forfeit |
| `draw_offer` | Propose a draw |
| `draw_accept` | Accept a draw proposal |
| `draw_decline` | Decline a draw proposal |
| `error` | Reject an invalid action |

### Status Transitions

| From | Command | To |
|------|---------|-----|
| `pending` | `accept` | `active` |
| `pending` | `decline` | `declined` |
| `active` | `move` (terminal) | `completed` |
| `active` | `resign` | `completed` |
| `active` | `draw_accept` | `completed` |
| `active` | `move` (normal) | `active` |
| `active` | `draw_offer` | `active` |
| `active` | `draw_decline` | `active` |
| `active` | `error` | `active` |

---

## 7. Game Session Types

| Type | Description |
|------|-------------|
| `turn_based` | Players alternate turns (e.g., Tic-Tac-Toe, Chess) |
| `real_time` | Both players can act at any time |
| `round_based` | Multiple rounds with scoring between rounds |
| `single_round` | Single round per session (e.g., coin flip, rock-paper-scissors) |

---

## 8. Validation Models

| Model | Description | Error Behavior |
|-------|-------------|----------------|
| `sender` | Sender validates before sending; receiver trusts | No error actions sent |
| `receiver` | Receiver validates on receipt; rejects invalid | Sends `error` action |
| `both` | Both sides validate independently | Receiver sends `error` if validation disagrees |

---

## 9. Error Actions

When a receiver rejects an action:

```
{
    "a": "<game_id>.<version>",
    "c": "error",
    "s": "<session_id>",
    "p": {
        "code": "<error_code>",
        "msg": "<human-readable message>",
        "ref": "<command that caused the error>"
    }
}
```

### Standard Error Codes

| Code | Meaning |
|------|---------|
| `unsupported_app` | Receiver doesn't have this game |
| `invalid_move` | Move failed validation |
| `not_your_turn` | Out-of-turn action |
| `session_expired` | Session timed out on receiver |
| `protocol_error` | Malformed envelope or unknown command |

Error actions are best-effort. If the error itself fails to deliver, the sender sees no response.

---

## 10. Session Expiry

| Status | Default TTL | Meaning |
|--------|-------------|---------|
| `pending` | 24 hours | Unanswered challenges expire |
| `active` | 7 days | Inactive sessions expire |
| `completed` | N/A | Preserved indefinitely |

Enforcement is **local-only**: each peer expires sessions independently based on its own clock. No LXMF message is sent on expiry.

A 1-hour grace period is applied to account for clock skew between peers.

Games MAY override default TTLs via their manifest.

---

## 11. Delivery Method Guidelines

Games declare preferred delivery per command. LXMF auto-escalates if content exceeds limits, so these are preferences, not guarantees.

| Action | Preferred | Rationale |
|--------|-----------|-----------|
| `challenge` | OPPORTUNISTIC | Small, fire-and-forget |
| `accept` | OPPORTUNISTIC | Small, includes initial state |
| `decline` | OPPORTUNISTIC | Minimal payload |
| `move` | OPPORTUNISTIC | Must fit in 295B |
| `resign` | DIRECT | Delivery confirmation important |
| `draw_offer` | OPPORTUNISTIC | Small |
| `draw_accept` / `draw_decline` | DIRECT | State-changing |
| `error` | OPPORTUNISTIC | Informational |

---

## 12. Game Manifest

Each game declares a manifest:

```
{
    "app_id": "<string>",
    "version": <int>,
    "display_name": "<string>",
    "icon": "<string>",
    "session_type": "turn_based" | "real_time" | "round_based" | "single_round",
    "max_players": 2,
    "min_players": 2,
    "validation": "sender" | "receiver" | "both",
    "actions": [<list of command strings>],
    "preferred_delivery": {<command: method>},
    "ttl": {"pending": <seconds>, "active": <seconds>},
    "genre": "<optional string>",
    "turn_timeout": <optional seconds>
}
```

---

## 13. Large Payloads

Most LRGP actions fit in a single packet. For larger data:

**Strategy A**: LXMF Resource auto-escalation. If DIRECT content exceeds 319 bytes, LXMF transfers as a Resource over the link (up to ~3.2 MB). Transparent to the game layer.

**Strategy B**: `FIELD_FILE_ATTACHMENTS` (`0x05`). For explicit bulk data, use the standard LXMF file attachment field alongside the LRGP envelope.

---

## 14. Backward Compatibility

Messages with `fields[0xFB] = "rlap.v1"` or `"ratspeak.game"` are legacy. Implementations MUST recognize them on inbound and process normally.

All outbound messages MUST use `"lrgp.v1"`.

---

## 15. Cross-Client Adoption Levels

| Level | Description |
|-------|-------------|
| **None** | Client ignores LRGP fields; shows fallback text |
| **Basic** | Client recognizes LRGP fields; shows enhanced notification |
| **Full** | Client renders interactive game UI |

Any LXMF client achieves "None" level by default — fallback text appears as a regular message.

---

## 16. Serialization

All LRGP data MUST be serialized with msgpack. JSON is NOT supported on the wire. This is a hard constraint — every byte matters on LoRa links.

---

## 17. Session Storage Schema

### game_sessions

| Column | Type | Description |
|--------|------|-------------|
| `session_id` | TEXT | 16-char hex, part of composite PK |
| `identity_id` | TEXT | Local identity, part of composite PK |
| `app_id` | TEXT | Game identifier |
| `app_version` | INTEGER | Protocol version |
| `contact_hash` | TEXT | Remote peer's identity hash |
| `initiator` | TEXT | Who sent the challenge |
| `status` | TEXT | pending/active/completed/expired/declined |
| `metadata` | TEXT (JSON) | Game-specific state blob |
| `unread` | INTEGER | 0 or 1 |
| `created_at` | REAL | Unix timestamp |
| `updated_at` | REAL | Unix timestamp |
| `last_action_at` | REAL | Unix timestamp (used for TTL) |

Primary key: `(session_id, identity_id)`

### game_actions (optional)

| Column | Type | Description |
|--------|------|-------------|
| `session_id` | TEXT | Session reference |
| `identity_id` | TEXT | Local identity |
| `action_num` | INTEGER | Sequence number |
| `command` | TEXT | LRGP command |
| `payload_json` | TEXT | Serialized payload |
| `sender` | TEXT | Who sent this action |
| `timestamp` | REAL | Unix timestamp |

Unique constraint: `(session_id, identity_id, action_num)`

---

## A. TicTacToe Reference Game

TicTacToe (`ttt.1`) is the built-in reference game demonstrating LRGP.

### Payload Schema

| Key | Type | Used In | Description |
|-----|------|---------|-------------|
| `i` | int | move | Cell index (0–8) |
| `b` | str | move, accept | Board state (9 chars: `_`, `X`, `O`) |
| `n` | int | move | Move number (1-based) |
| `t` | str | move, accept | Hash of player whose turn it is next |
| `x` | str | move | Terminal status: `""`, `"win"`, `"draw"` |
| `w` | str | move | Winner's hash (only when `x == "win"`) |
