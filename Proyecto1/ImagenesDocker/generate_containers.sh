#!/bin/bash

# Ruta al script principal (actualiza esta ruta si es necesario)
SCRIPT_PATH="generate_containers.sh"

# Asegurarse de que el script principal sea ejecutable
chmod +x "$SCRIPT_PATH"

# Agregar el cronjob para ejecutar el script todos los días a las 2:00 AM
CRON_JOB="0 2 * * * $SCRIPT_PATH"

# Verificar si ya existe un cronjob similar
CRON_EXISTS=$(crontab -l | grep -F "$SCRIPT_PATH")

if [ -z "$CRON_EXISTS" ]; then
    # Si el cronjob no existe, agregarlo
    (crontab -l 2>/dev/null; echo "$CRON_JOB") | crontab -
    echo "Cronjob agregado para ejecutar $SCRIPT_PATH."
else
    echo "El cronjob ya está configurado para $SCRIPT_PATH."
fi

# Array de imágenes
imagenes=("alto_consumo" "bajo_consumo" "alto_consumo_2" "bajo_consumo_2")

# Repetir el proceso de creación de contenedores (2 veces en este caso)
for ((j=1; j<=2; j++))
do
  # Crear 10 contenedores de forma aleatoria
  for ((i=1; i<=10; i++))
  do
    # Elegir una imagen aleatoriamente
    imagen=${imagenes[$RANDOM % ${#imagenes[@]}]}
    
    # Generar un nombre aleatorio para el contenedor usando /dev/urandom
    nombreContenedor=$(head -c 16 /dev/urandom | tr -dc 'a-zA-Z0-9' | head -c 10)
    
    # Ejecutar el contenedor
    sudo docker run -d --name "$nombreContenedor" --cpus="0.1" "$imagen"
  done

  # Esperar 30 segundos antes de generar otros 10 contenedores
  echo "Esperando 30 segundos antes de crear más contenedores..."
  sleep 30
done


#limitar el uso del cpu con el cronjob