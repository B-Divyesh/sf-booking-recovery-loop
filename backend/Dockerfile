FROM node:22-bookworm-slim AS web-build

WORKDIR /workspace
COPY package.json package-lock.json tsconfig.json vite.config.ts index.html ./
COPY public ./public
COPY src ./src
RUN npm ci && npm run build

FROM rust:1.98-slim-bookworm AS api-build

ARG BUILD_SHA=dev
ENV BUILD_SHA=${BUILD_SHA}
WORKDIR /workspace/backend

COPY backend/Cargo.toml backend/Cargo.lock backend/build.rs ./
COPY backend/migrations ./migrations
COPY backend/src ./src

RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot

ARG BUILD_SHA=dev
ENV PORT=8080
ENV STATIC_DIR=/app/dist
COPY --from=api-build /workspace/backend/target/release/booking-recovery-loop-api /app/booking-recovery-loop-api
COPY --from=web-build /workspace/dist /app/dist
COPY --from=web-build --chown=nonroot:nonroot /tmp /data

USER nonroot:nonroot
WORKDIR /data
EXPOSE 8080
ENTRYPOINT ["/app/booking-recovery-loop-api"]
