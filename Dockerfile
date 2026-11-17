FROM rust:1-bookworm AS build
WORKDIR /app
ENV CARGO_BUILD_JOBS=2
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN useradd -m app
COPY --from=build /app/target/release/alquileres-api /usr/local/bin/alquileres-api
USER app
EXPOSE 8080
CMD ["alquileres-api"]
