FROM node:24-alpine AS base
WORKDIR /app
COPY package*.json ./
RUN npm ci --omit=dev --ignore-scripts --no-audit --no-fund
COPY server.mjs oidc.mjs sample.json ./

FROM base AS verified
COPY test ./test
RUN npm test && touch /app/.tests-passed

FROM base
COPY --from=verified /app/.tests-passed ./
USER node
ENV NODE_ENV=production PORT=8080
EXPOSE 8080
CMD ["node", "server.mjs"]
