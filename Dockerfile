FROM rustlang/rust:nightly AS build

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:buster-slim

WORKDIR /app
COPY --from=build /app/target/release/pudding .

RUN apt-get update && apt-get install -y pkg-config libssl-dev

CMD ["/app/pudding"]
