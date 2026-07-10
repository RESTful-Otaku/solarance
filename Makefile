SPACETIME    = spacetime
SERVER_MF    = --manifest-path server/Cargo.toml
CLIENT_MF    = --manifest-path client/Cargo.toml

.PHONY: all check lint build run clean start publish reset bindings logs

# Type-check both crates (fast)
all: check lint

check:
	cargo check $(SERVER_MF)
	cargo check $(CLIENT_MF)

# Run lints on both crates
lint:
	cargo clippy $(SERVER_MF)
	cargo clippy $(CLIENT_MF)

# Full release build (client only; server is published, not built standalone)
build:
	cargo build --release $(CLIENT_MF)

# Run the game client
run: build
	cd client && cargo run --release

# Publish server module to SpacetimeDB
publish: check
	$(SPACETIME) publish solarance-beginnings -p server/

# Reset database and re-publish
reset:
	$(SPACETIME) publish -c solarance-beginnings -y -p server/

# Regenerate client bindings after server schema changes
bindings:
	$(SPACETIME) generate --lang rust --out-dir client/src/server/bindings -p server/

# Tail server logs
logs:
	$(SPACETIME) logs solarance-beginnings

# Start the SpacetimeDB local server (no output, runs in background)
start:
	@echo "Starting SpacetimeDB server..."
	@nohup $(SPACETIME) start > /tmp/spacetime-server.log 2>&1 &
	@sleep 2
	@echo "Server should be running. Check with '$(SPACETIME) server list'"

# Clean all build artifacts
clean:
	cargo clean
