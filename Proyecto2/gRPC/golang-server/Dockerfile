# Etapa de construcción
FROM golang:1.23-bullseye AS build

WORKDIR /app 

# Copiar los archivos de dependencias y descargar las dependencias
COPY go.mod go.sum ./
RUN go mod download

# Copiar el resto del código fuente
COPY . .

# Compilar la aplicación
RUN go build \
  -ldflags="-linkmode external -extldflags -static" \
  -tags netgo \
  -o main

# Etapa de ejecución
FROM debian:bullseye-slim

# Instalar certificados CA
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Copiar el binario de la etapa de construcción
COPY --from=build /app/main /main

# Exponer el puerto que usa tu aplicación
EXPOSE 50051

# Comando para ejecutar la aplicación
CMD ["/main"]
