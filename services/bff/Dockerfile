# Build context is the repo root (see docker-compose.yml) so proto/ can
# be copied into the image — the BFF's gRPC client loads the same
# proto/media.proto that core-service compiles its server from.
FROM node:22-slim AS build
WORKDIR /app
COPY services/bff/package.json services/bff/package-lock.json ./
RUN npm ci
COPY services/bff .
RUN npm run build

FROM node:22-slim AS runtime
WORKDIR /app
ENV NODE_ENV=production
COPY services/bff/package.json services/bff/package-lock.json ./
RUN npm ci --omit=dev
COPY --from=build /app/dist ./dist
COPY proto /app/proto
ENV MEDIA_PROTO_PATH=/app/proto/media.proto
EXPOSE 3000
CMD ["node", "dist/main.js"]
