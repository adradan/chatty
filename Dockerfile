# Thanks to https://blog.logrocket.com/packaging-a-rust-web-service-using-docker/ for the docker file
FROM rust:buster as build
LABEL authors="ericvsolcruz"

RUN cargo new --bin chatty-backend
WORKDIR ./chatty-backend
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release

RUN rm src/*.rs

ADD . ./

RUN rm -rf ./target/release/deps/chatty-backend*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app
ENV BACKEND_PORT=8000
ENV RUST_LOG=debug

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

ENV TZ=Etc/UTC APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=build /chatty-backend/target/release/chatty-backend ${APP}/chatty-backend

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./chatty-backend"]