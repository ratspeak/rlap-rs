# Changelog

## 0.2.0 — 2025-03-12

### Breaking — Renamed to LRGP

RLAP (Reticulum LXMF App Protocol) has been renamed and re-purposed to **LRGP** (Lightweight Reticulum Gaming Protocol). The protocol now focuses specifically on multiplayer gaming over Reticulum mesh networks.

#### Wire Protocol
- Protocol marker: `rlap.v1` → `lrgp.v1`
- Legacy `rlap.v1` and `ratspeak.game` messages still recognized on inbound
- All outbound messages use `lrgp.v1`

#### API Renames
- `RlapApp` trait → `GameApp`
- `AppManifest` → `GameManifest`
- `RlapRouter` → `LrgpRouter`
- `RlapStore` → `LrgpStore`
- `RlapError` → `LrgpError`

#### New Features
- `GameManifest` adds `min_players`, `genre`, and `turn_timeout` fields
- New game session types: `round_based`, `single_round`
- `LEGACY_TYPES` array for multi-marker backward compatibility

#### Database
- `app_sessions` table → `game_sessions`
- `app_actions` table → `game_actions`

#### Fallback Text
- Format changed from `[RLAP ...]` to `[LRGP ...]`

---

## 0.1.0 — 2025-02-28

### Initial Release

- Envelope packing/unpacking with msgpack serialization
- Session state machine (pending → active → completed/expired/declined)
- `RlapApp` trait for pluggable applications
- `RlapRouter` for app registration and message dispatch
- `RlapStore` with SQLite persistence (WAL mode, parameterized queries)
- Transport bridge (LXMF field ↔ RLAP envelope)
- TicTacToe reference app with both-side validation
- Cross-compatible binary test vectors (`ttt_challenge.bin`, `ttt_move.bin`, `ttt_move_win.bin`)
