# Usa una imagen base de Go
FROM golang:1.20-alpine

# Establece el directorio de trabajo
WORKDIR /app

# Copia el archivo Go en el contenedor
COPY . .

# Construye la aplicación Go
RUN go build -o app .

# Define el puerto en el que la aplicación se ejecuta
EXPOSE 8080

# Comando para ejecutar la aplicación
CMD ["./app"]
