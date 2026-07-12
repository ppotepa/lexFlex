# ============================================
# lexFlex Docker - Fully offline capable
# ============================================
# Builds both:
#   - lexflex   (main CLI: translate, parse, graph)
#   - lexlearn  (learner CLI with --offline support)
#
# The image bundles the entire `data/` directory.
# lexlearn --offline completely skips ConceptNet / DictionaryAPI / Wiktionary.
# All concept inference and graph population uses only local .ron files.
#
# Usage examples:
#   docker build -t lexflex .
#   docker run --rm lexflex lexflex --help
#   docker run --rm lexflex lexlearn --help
#   docker run --rm lexflex lexlearn deduce "żona" --lang pl --offline
#   docker run --rm -v $(pwd)/myinput.txt:/input.txt lexflex \
#       lexlearn bulk --input /input.txt --lang pl --offline --max 20
#
# Build time requires network (for cargo), runtime is air-gapped.

# ---------- Build stage ----------
FROM rust:1.85-slim AS builder

# Install minimal build deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only manifests first (better layer caching)
COPY Cargo.toml Cargo.lock ./
COPY lexflex-learner/Cargo.toml lexflex-learner/Cargo.lock ./lexflex-learner/

# Create dummy sources so we can pre-build dependencies (both bins + lib)
RUN mkdir -p src lexflex-learner/src \
    && echo 'fn main(){}' > src/main.rs \
    && echo 'fn main(){}' > lexflex-learner/src/main.rs \
    && echo 'pub fn _dummy(){}' > lexflex-learner/src/lib.rs

# Pre-fetch + build deps (cached layer)
RUN cargo build --release --bin lexflex

# lexlearn is a standalone package → build it from its own directory
RUN cd lexflex-learner && cargo build --release --bin lexlearn

# Now bring in the real source code
COPY . .

# Final release build
RUN cargo build --release --bin lexflex
RUN cd lexflex-learner && cargo build --release --bin lexlearn

# ---------- Runtime stage (small) ----------
FROM debian:bookworm-slim

# Runtime deps (none really needed, but keep for future)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the two binaries
COPY --from=builder /app/target/release/lexflex /usr/local/bin/lexflex
COPY --from=builder /app/lexflex-learner/target/release/lexlearn /usr/local/bin/lexlearn

# Bundle the complete local data (this is the key for offline mode)
COPY data /app/data

# (Optional) include a sample input for quick testing
COPY benchmarks/input_adam.txt /app/benchmarks/input_adam.txt

# Make sure the binaries are executable
RUN chmod +x /usr/local/bin/lexflex /usr/local/bin/lexlearn

# Environment hint (used by path finder in lexlearn)
ENV LEXFLEX_DATA_DIR=/app/data

# Default to showing help for both tools
# You can override:
#   docker run --rm lexflex lexlearn bulk --input /app/benchmarks/input_adam.txt --lang pl --max 5 --offline
ENTRYPOINT ["lexlearn"]
CMD ["--help"]

# Alternative usage note (visible with `docker run --rm lexflex cat /app/README.docker` if you add one)
# To run the main tool instead:
#   docker run --rm --entrypoint lexflex lexflex --help
