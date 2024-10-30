#!/bin/bash

# Remove all Docker images if they exist
if [ "$(docker images -q)" ]; then
    docker rmi -f $(docker images -q)
else
    echo "No hay imágenes para eliminar."
fi

# Variables for the Docker images
GO_CLIENT_IMAGE="golang-client-grpc"
RUST_CLIENT_IMAGE="rust-client-grpc"
GO_SERVER_IMAGE="golang-server-grpc"
DOCKERHUB_USERNAME="cutzalluis"
TAG="0.1"

# Build the Docker images
if [ -d "./gRPC/golang-client" ]; then
    docker build -t $GO_CLIENT_IMAGE ./gRPC/golang-client
else
    echo "El directorio ./gRPC/golang-client no se encontró."
fi

if [ -d "./gRPC/golang-server" ]; then
    docker build -t $RUST_CLIENT_IMAGE ./gRPC/golang-server
else
    echo "El directorio ./gRPC/golang-server no se encontró."
fi

if [ -d "./gRPC/grpc-client" ]; then
    docker build -t $GO_SERVER_IMAGE ./gRPC/grpc-client
else
    echo "El directorio ./gRPC/grpc-client no se encontró."
fi

# Tag the Docker images
docker tag $GO_CLIENT_IMAGE "$DOCKERHUB_USERNAME/$GO_CLIENT_IMAGE:$TAG" 2>/dev/null
docker tag $RUST_CLIENT_IMAGE "$DOCKERHUB_USERNAME/$RUST_CLIENT_IMAGE:$TAG" 2>/dev/null
docker tag $GO_SERVER_IMAGE "$DOCKERHUB_USERNAME/$GO_SERVER_IMAGE:$TAG" 2>/dev/null

# Push the Docker images to DockerHub
docker push "$DOCKERHUB_USERNAME/$GO_CLIENT_IMAGE:$TAG" 2>/dev/null
docker push "$DOCKERHUB_USERNAME/$RUST_CLIENT_IMAGE:$TAG" 2>/dev/null
docker push "$DOCKERHUB_USERNAME/$GO_SERVER_IMAGE:$TAG" 2>/dev/null

echo "Docker images processed successfully."
