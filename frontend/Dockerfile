# Frontend multi-stage build:
# 1. build with devcontainer
# 2. serve static with Caddy

FROM quay.io/geoengine/devcontainer:latest AS builder

# 1.1 Install dependencies

WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci --silent

WORKDIR /app/api-client/typescript
COPY api-client/typescript .
RUN npm ci --silent

# 1.2 Copy source and build
WORKDIR /app/frontend
COPY frontend/public ./public
COPY frontend/src ./src
COPY \
    frontend/_theme-colors.scss \
    frontend/angular.json \
    frontend/tsconfig*.json \
    ./
RUN npm run build --silent

# 2. Final image: Caddy to serve static files and reverse-proxy API
FROM caddy:2-alpine AS runner

# Copy built site into caddy's web root
COPY --from=builder /app/frontend/dist/BioIS/browser /usr/share/caddy

# Copy Caddyfile for SPA fallback and API proxy
COPY frontend/Caddyfile /etc/caddy/Caddyfile

EXPOSE 80
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
