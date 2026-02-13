# chessbench

Chessbench runs UCI chess engines against each other and exposes a small HTTP API for live match control and viewing. It includes a Rust server and a Vite-based UI.

## Requirements
- Rust toolchain (stable)
- One or more UCI engine binaries
- Node.js + npm (for the UI)

## Quick start (server)
1) Create an engine config file (example below).
2) Run the server:

```bash
cargo run -- --config engines.toml
```

Optional: change the bind address/port (defaults to `0.0.0.0:8080`).

```bash
cargo run -- --config engines.toml --bind 127.0.0.1:8080
```

## Engine config
`engines.toml` is a list of engines with stable ids. Paths must point to UCI-capable binaries.

```toml
[[engine]]
id = "stockfish-16"
path = "/opt/stockfish"
args = ["-threads", "4"]

[[engine]]
id = "lc0-0.30"
path = "/opt/lc0"
working_dir = "/opt/lc0"
```

## Run the UI
The UI proxies `/api` to `http://localhost:8080` in dev mode.

```bash
cd ui
npm install
npm run dev
```

## API summary (v1)
- `GET /api/engines` -> list discovered engines
- `POST /api/match` -> create a match and return `match_id`
- `GET /api/match/:id` -> current status, FEN, PGN, clocks, result
- `GET /api/match/:id/stream` -> SSE stream of match events

SSE events
- `match_started` with `start_fen`
- `clock` every 200ms while running
- `move` with `uci`, `san`, `fen`, `pgn`
- `result` with `result` and `reason`

## Notes
- Time control v1 supports only `initial_ms` (no increment yet).
- Draws are adjudicated for threefold repetition and the 50-move rule.

## Troubleshooting
- Ensure engine binaries are executable and paths are correct.
- If the server fails to bind, the port may already be in use.
