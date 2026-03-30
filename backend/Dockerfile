FROM rust:1-bookworm AS backend
WORKDIR /app
COPY backend/ .
RUN cargo build --release

FROM node:20-slim AS frontend
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm install
COPY frontend/ .
RUN npm run build

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/target/release/vlt-backend /usr/local/bin/
COPY --from=frontend /app/build /srv/frontend
ENV FRONTEND_DIR=/srv/frontend
ENV DATABASE_URL=sqlite:///data/vlt.db?mode=rwc
ENV COOKIE_SECURE=true
ENV PORT=8080
EXPOSE 8080
VOLUME /data
CMD ["vlt-backend"]
