FROM node:22

USER node

WORKDIR /app

COPY --chown=node:node . .

RUN npm ci

RUN npm run build

CMD ["node", "server.cjs"]
