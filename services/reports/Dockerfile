FROM node:24-alpine AS base
WORKDIR /app
COPY --chmod=0644 package*.json ./
RUN npm ci --omit=dev --ignore-scripts --no-audit --no-fund
COPY --chmod=0644 server.mjs oidc.mjs saved-report.mjs sample.json ./

FROM base AS verified
COPY --chown=node:node test ./test
USER node
RUN npm test && touch /tmp/.tests-passed

FROM base
COPY --from=verified /tmp/.tests-passed ./
USER node
ENV NODE_ENV=production PORT=8080 REPORTS_DIR=/reports
EXPOSE 8080
CMD ["node", "server.mjs"]
