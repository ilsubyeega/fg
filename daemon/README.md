# Fall Guys Log Daemon

A high-performance daemon that monitors Fall Guys log files in real-time and parses game events for further processing.

## Features

- 🚀 **High Performance**: Uses [nom](https://github.com/rust-bakery/nom) parser combinators for efficient parsing
- 📡 **Real-time Monitoring**: Watches log files for changes using OS-native file system notifications
- 🎮 **Comprehensive Event Parsing**: Extracts matchmaking, round info, player actions, and more
- 📊 **Rich Game Data**: Includes localized strings, round metadata, and show configurations
- 🔧 **Async Architecture**: Built on tokio for efficient concurrent processing

## Architecture

```
Log File → task_watch → task_parser → rules → FGGameMessage
               ↓              ↓
         WatchMessage    ParseResult
```

### Components

- **`parser/task_watch`**: File system watcher that monitors the log directory
- **`parser/task_parser`**: Async task that processes log lines and handles multi-line messages
- **`parser/rules`**: Individual parsing rules for different log message types
- **`parser/combinators`**: Reusable nom parser combinators
- **`models`**: Data structures for game events, states, and player info
- **`extra_data`**: Static game data (rounds, shows, localized strings)
- **`error`**: Centralized error handling

## Installation

### Prerequisites

- Rust 2024 edition or later
- Fall Guys installed (for log files)

### Building

```bash
cargo build --release
```

## Usage

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LOG_DIR` | Directory containing Fall Guys log files | **Required** |
| `LOG_FILE` | Name of the log file to monitor | `Player.log` |

### Running

```bash
# Using environment variables
LOG_DIR=/path/to/logs cargo run

# Or with a .env file
echo "LOG_DIR=/path/to/logs" > .env
cargo run
```

### Log File Locations

| Platform | Path |
|----------|------|
| Steam (Linux) | `~/.steam/steam/steamapps/compatdata/1097150/pfx/drive_c/users/steamuser/AppData/LocalLow/Mediatonic/FallGuys_client/` |
| Steam (Windows) | `%USERPROFILE%\AppData\LocalLow\Mediatonic\FallGuys_client\` |
| Epic Games | `%LOCALAPPDATA%\Mediatonic\FallGuys_client\` |

## Parsed Events

The daemon parses the following event types:

### Game State
- Game state machine transitions
- Client readiness states
- Session state changes

### Matchmaking
- Begin matchmaking
- Queue status (connecting, waiting, full)
- Session assignment
- Server connection

### Players
- Local/remote player creation
- Player spawn/unspawn
- Squad/party assignments
- Spectator targets

### Rounds
- Round loading
- Round completion
- Player scores
- Episode rewards (DTO)

### Network
- Server connection (IP/port)
- Network metrics (RTT/latency)

## Development

### Project Structure

```
src/
├── main.rs              # Entry point
├── error.rs             # Error types
├── extra_data.rs        # Static game data
├── models/
│   ├── common.rs        # Common types
│   ├── dto.rs           # Data transfer objects
│   ├── exports.rs       # Export types
│   ├── messages.rs      # Game messages
│   └── state.rs         # State enums
└── parser/
    ├── mod.rs           # ParseResult type
    ├── combinators.rs   # Nom combinators
    ├── rules.rs         # Parsing rules
    ├── task_parser.rs   # Parser task
    └── task_watch.rs    # File watcher
```

### Running Tests

```bash
cargo test
```

### Updating Game Data

Game data files in `extra_datas/` are extracted from Fall Guys game files.
See `extra_datas/README.md` for update instructions.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `nom` | Parser combinators |
| `tokio` | Async runtime |
| `notify` | File system notifications |
| `serde` | Serialization |
| `tracing` | Logging |
| `temporal_rs` | Time handling |
| `anyhow` | Error propagation |
| `thiserror` | Error definitions |

## License

See LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request
