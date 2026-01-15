# Rustboard Collaboration Server

Real-time collaboration server for Rustboard using WebSockets.

## Running the Server

**Important:** The server must be built for the native target, not WASM. If you have a `.cargo/config.toml` that sets the default target to WASM, you need to override it:

```bash
cd server
cargo build --bin rustboard-server
cargo run --bin rustboard-server
```

Or explicitly specify the native target:

```bash
cargo build --bin rustboard-server --target x86_64-unknown-linux-gnu
cargo run --bin rustboard-server --target x86_64-unknown-linux-gnu
```

The server will start on `http://localhost:3001`.

## Environment Variables

- `PORT` - Port to run the server on (default: 3001)
- `RUST_LOG` - Logging level (default: info)

## API Endpoints

### Create Session
```
GET /api/sessions
```

Returns:
```json
{
  "session_id": "uuid",
  "url": "http://localhost:5173/?session=uuid"
}
```

### Check Session
```
GET /api/sessions/:session_id
```

Returns:
```json
{
  "exists": true
}
```

### WebSocket Connection
```
WS /ws/:session_id
```

## WebSocket Protocol

### Client Messages

#### Join
```json
{
  "type": "Join",
  "client_id": "string",
  "name": "string",
  "color": "#hexcolor"
}
```

#### Update
```json
{
  "type": "Update",
  "operation": {
    "op": "AddRectangle",
    "id": 123,
    "position": { "x": 100, "y": 200 },
    "width": 50,
    "height": 50
  }
}
```

#### Ping
```json
{
  "type": "Ping"
}
```

### Server Messages

#### Joined
```json
{
  "type": "Joined",
  "client_id": "string",
  "clients": [...],
  "document": "serialized_json"
}
```

#### ClientJoined
```json
{
  "type": "ClientJoined",
  "client": {
    "id": "string",
    "name": "string",
    "color": "#hexcolor"
  }
}
```

#### ClientLeft
```json
{
  "type": "ClientLeft",
  "client_id": "string"
}
```

#### Update
```json
{
  "type": "Update",
  "operation": {...},
  "client_id": "string"
}
```

## Architecture

- **Session Management**: Each collaboration session has a unique ID and stores document state
- **Broadcast Channel**: All clients in a session receive updates via broadcast channel
- **Operation-Based Sync**: Changes are synced as operations, not full document state
- **Conflict Resolution**: Last-write-wins for concurrent edits

