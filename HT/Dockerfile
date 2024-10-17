# Usa una imagen base de Go
FROM golang:1.20 AS builder

# Establece el directorio de trabajo
WORKDIR /app

# Copia los archivos go.mod y go.sum
COPY go.mod ./
RUN go mod download

# Copia el resto de los archivos de la aplicación
COPY . .

# Compila la aplicación
RUN go build -o my-go-api .

# Usa una imagen más pequeña para ejecutar la aplicación
FROM gcr.io/distroless/base

# Copia el binario desde la imagen del builder
COPY --from=builder /app/my-go-api /my-go-api

# Establece el comando por defecto
CMD ["/my-go-api"]
