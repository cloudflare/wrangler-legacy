ARG version=12.6.0-alpine
FROM node:$version

ARG INCLUDE_WORKER_TEMPLATE=https://github.com/cloudflare/worker-template
ENV USER=node

COPY ./npm/* /home/node/

USER node
WORKDIR /home/node/wrangler

RUN cd /home/node && \
    npm install --cache=/home/node/.cache && \
    mkdir -p /home/node/wrangler && \
    if [[ ! -z $INCLUDE_WORKER_TEMPLATE ]]; then cd /home/node && /home/node/.wrangler/out/wrangler generate worker "${INCLUDE_WORKER_TEMPLATE}" && cd "/home/node/worker" && npm install --cache=/home/node/.cache && cd /home/node/worker && /home/node/.wrangler/out/wrangler build; fi


ENTRYPOINT ["/home/node/.wrangler/out/wrangler"]

