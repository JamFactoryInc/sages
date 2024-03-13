FROM rust:1.76.0-slim-bullseye

ARG APP_NAME=sages
ARG LISTENING_PORT=8080

ENV WEB_PORT=$LISTENING_PORT

WORKDIR /sages
COPY . .

RUN apt-get update -yq \
    && apt-get install -yq nodejs
    && cd ./page
    && npm install webpack-cli
    && npx webpack
RUN cargo build --release
RUN cp ./target/release/$APP_NAME /bin/sages

EXPOSE $LISTENING_PORT

CMD ["/bin/sages"]