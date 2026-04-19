FROM rust:1.95-slim-bookworm AS builder

WORKDIR /usr/src/oxide-lb

COPY . .

RUN cargo build --release


FROM debian:bookworm-slim

RUN groupadd --gid 1000 oxide && useradd --uid 1000 --gid oxide --shell /bin/false oxide

COPY --from=builder /usr/src/oxide-lb/target/release/oxide-lb /usr/local/bin/oxide-lb

USER oxide

ENV OXIDE_LB_BIND_ADDR=0.0.0.0:3000
ENV OXIDE_LB_HEALTH_CHECK_INTERVAL=5
ENV OXIDE_LB_SELECTION_STRATEGY=ROUND_ROBIN

EXPOSE 3000

CMD [ "oxide-lb" ]