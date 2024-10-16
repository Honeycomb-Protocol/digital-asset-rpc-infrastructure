FROM das-api/builder AS files
FROM rust:1.75-slim-bullseye
ARG APP=/usr/src/app
RUN apt update \
    && apt install -y curl ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*
ENV TZ=Etc/UTC \
    APP_USER=appuser
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}
COPY --from=files /das/das-grpc-ingest ${APP}
RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}
ENTRYPOINT [ "/usr/src/app/das-grpc-ingest" ] 
