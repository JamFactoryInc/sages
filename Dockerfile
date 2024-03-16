FROM alpine:3.14

RUN apt-get update
RUN apt-get install -y \
    npm \
    build-essential \
    curl

# set up rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:/usr/local/bin/npm:${PATH}"

# set up npx & webpack
RUN npm i -g npx webpack

ARG APP_NAME=sages
ARG LISTENING_PORT=8080

ENV WEB_PORT=$LISTENING_PORT

WORKDIR /sages
COPY . .

RUN cargo build --release
RUN cp ./target/release/$APP_NAME /bin/sages

EXPOSE $LISTENING_PORT

CMD ["/bin/sages"]