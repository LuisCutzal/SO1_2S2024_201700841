#!/bin/bash

# Array de imágenes
imagenes=("alto_consumo" "bajo_consumo" "alto_consumo_2" "bajo_consumo_2")

# Crear los contenedores de forma aleatoria
for ((i=1; i<=10; i++))
do
  # Elegir una imagen aleatoriamente
  imagen=${imagenes[$RANDOM % ${#imagenes[@]}]}
  
  # Generar un nombre aleatorio para el contenedor usando /dev/urandom
  nombreContenedor=$(head -c 16 /dev/urandom | tr -dc 'a-zA-Z0-9' | head -c 10)
  
  # Ejecutar el contenedor
  sudo docker run -d --name "$nombreContenedor" "$imagen"
  
  # Esperar 30 segundos antes de crear el siguiente contenedor
  sleep 30
done


#de estos 10 contenedores debemos de tener solo 4
#un script para generar las 4 imagenes
#un script para genera los 10 contenedores
#4 contenedores de alto y 4 de bajo

