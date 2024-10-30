FROM golang:1.23-bullseye AS build

WORKDIR /app 

COPY go.mod go.sum ./

RUN go mod download

COPY . .

RUN go build \
  -ldflags="-linkmode external -extldflags -static" \
  -tags netgo \
  -o main

FROM debian:bullseye

RUN apt-get update && apt-get install -y ca-certificates

COPY --from=build /app/main /main

CMD ["/main"]
