FROM node AS tailwind
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .

# Build the Tailwind CSS file
RUN npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --minify

FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Install `dx`
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

# Copy the source code and Tailwind CSS file
COPY . .
COPY --from=tailwind /app/assets/tailwind.css ./assets/tailwind.css

# Create the final bundle folder. Bundle always executes in release mode with optimizations enabled
RUN dx bundle --platform web

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/dx/scout/release/web/ /usr/local/app

# Set our port and make sure to listen for all connections
ENV PORT=8080
ENV IP=0.0.0.0

# Expose the port 8080
EXPOSE 8080

ENTRYPOINT [ "/usr/local/app/server" ]
