FROM rust:alpine as builder
RUN apk add --no-cache musl-dev && mkdir /src
COPY / /src
WORKDIR "/src/server"
RUN ["cargo", "build", "--release"]

FROM alpine as root
COPY --from=builder /src/server/target/release/c33d /c33d
EXPOSE 8080
ENTRYPOINT ["/c33d", "--port=8080", "--host=0.0.0.0"]
