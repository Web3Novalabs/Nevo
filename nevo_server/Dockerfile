FROM node:22-bookworm-slim AS deps
WORKDIR /app
COPY nevo_server/package.json nevo_server/package-lock.json ./
RUN npm ci --legacy-peer-deps --no-audit --no-fund --prefer-offline --fetch-retries=10 --fetch-retry-mintimeout=10000 --fetch-retry-maxtimeout=120000 --fetch-timeout=600000

FROM deps AS build
COPY nevo_server/nest-cli.json nevo_server/tsconfig.json nevo_server/tsconfig.build.json ./
COPY nevo_server/src ./src
RUN npm run build

FROM node:22-bookworm-slim AS runner
WORKDIR /app
ENV NODE_ENV=production
COPY --from=deps /app/node_modules ./node_modules
COPY --from=build /app/dist ./dist
COPY --from=build /app/src ./src
COPY nevo_server/package.json nevo_server/tsconfig.json nevo_server/tsconfig.build.json ./
COPY nevo_server/docker-entrypoint.sh ./docker-entrypoint.sh
RUN chmod +x docker-entrypoint.sh
EXPOSE 3000
ENTRYPOINT ["./docker-entrypoint.sh"]
CMD ["node", "dist/main"]
