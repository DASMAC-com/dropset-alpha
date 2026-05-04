FROM rust:latest

# Install Solana CLI and platform tools
RUN sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Add Solana to PATH permanently
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Verify solana is found then install platform tools
RUN solana --version && \
    cargo install cargo-build-sbf

# Set working directory
WORKDIR /app

# Copy repo in
COPY . .

# Build the program
RUN cargo build-sbf

CMD ["cargo", "test"]