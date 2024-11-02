#!/bin/bash
set -e  # Detener en caso de error

# Remove all Docker images except specified ones
docker rmi -f $(docker images -q | grep -v -e "^$(docker images -q wurstmeister/zookeeper)$" -e "^$(docker images -q wurstmeister/kafka)$")

# Variables for the Docker images
GO_CLIENT_IMAGE="golang-client-grpc" 
RUST_CLIENT_IMAGE="rust-client-grpc" 
GO_SNATACION_IMAGE="golang-server-natacion"
GO_SATLETISMO_IMAGE="golang-server-atletismo"
GO_SBOXEO_IMAGE="golang-server-boxeo"
GO_SGANADOR_IMAGE="winn" 
GO_SPERDEDOR_IMAGE="loser"

DOCKERHUB_USERNAME="cutzalluis"
TAG="0.6"

# ------------------------------Build the Docker images------------------------------
docker build -t $GO_CLIENT_IMAGE ./gRPC/golang-client
docker build -t $RUST_CLIENT_IMAGE ./gRPC/grpc-client
docker build -t $GO_SNATACION_IMAGE ./gRPC/golang-server-nat
docker build -t $GO_SATLETISMO_IMAGE ./gRPC/golang-server-atle
docker build -t $GO_SBOXEO_IMAGE ./gRPC/golang-server-box
docker build -t $GO_SGANADOR_IMAGE ./gRPC/winner
docker build -t $GO_SPERDEDOR_IMAGE ./gRPC/loser

# ------------------------------Tag the Docker images------------------------------
docker tag $GO_CLIENT_IMAGE "$DOCKERHUB_USERNAME/$GO_CLIENT_IMAGE:$TAG"
docker tag $RUST_CLIENT_IMAGE "$DOCKERHUB_USERNAME/$RUST_CLIENT_IMAGE:$TAG"
docker tag $GO_SNATACION_IMAGE "$DOCKERHUB_USERNAME/$GO_SNATACION_IMAGE:$TAG"
docker tag $GO_SATLETISMO_IMAGE "$DOCKERHUB_USERNAME/$GO_SATLETISMO_IMAGE:$TAG"
docker tag $GO_SBOXEO_IMAGE "$DOCKERHUB_USERNAME/$GO_SBOXEO_IMAGE:$TAG"
docker tag $GO_SGANADOR_IMAGE "$DOCKERHUB_USERNAME/$GO_SGANADOR_IMAGE:$TAG"
docker tag $GO_SPERDEDOR_IMAGE "$DOCKERHUB_USERNAME/$GO_SPERDEDOR_IMAGE:$TAG"

# ------------------------------Push the Docker images to DockerHub------------------------------
docker push "$DOCKERHUB_USERNAME/$GO_CLIENT_IMAGE:$TAG"
docker push "$DOCKERHUB_USERNAME/$RUST_CLIENT_IMAGE:$TAG"
docker push "$DOCKERHUB_USERNAME/$GO_SNATACION_IMAGE:$TAG"
docker push "$DOCKERHUB_USERNAME/$GO_SATLETISMO_IMAGE:$TAG"
docker push "$DOCKERHUB_USERNAME/$GO_SBOXEO_IMAGE:$TAG"
docker push "$DOCKERHUB_USERNAME/$GO_SGANADOR_IMAGE:$TAG"
docker push "$DOCKERHUB_USERNAME/$GO_SPERDEDOR_IMAGE:$TAG"

echo "Docker images pushed successfully."
