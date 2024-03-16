FROM alpine:3.14

RUN apk add --update curl gcc npm 

# set up rust
RUN curl -proto '=https' -tlsv1.2 -sSf https://sh.rustup.rs | sh
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