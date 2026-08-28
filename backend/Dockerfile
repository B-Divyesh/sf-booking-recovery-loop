FROM rust:1.98-slim-bookworm AS build

ARG BUILD_SHA=dev
ENV BUILD_SHA=${BUILD_SHA}
WORKDIR /workspace/backend

COPY backend/Cargo.toml backend/Cargo.lock backend/build.rs ./
COPY backend/src ./src

RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot

ARG BUILD_SHA=dev
ENV PORT=8080
COPY --from=build /workspace/backend/target/release/booking-recovery-loop-api /app/booking-recovery-loop-api

USER nonroot:nonroot
EXPOSE 8080
ENTRYPOINT ["/app/booking-recovery-loop-api"]
