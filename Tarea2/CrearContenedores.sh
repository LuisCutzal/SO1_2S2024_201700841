#!/bin/bash
nombreRandom() {
  local length=8
  tr -dc A-Za-z0-9 </dev/urandom | head -c $length
}
for i in $(seq 1 10); do
  container_name=$(nombreRandom)
  echo"Creando contenedor con nombre: $container_name"
  sudo docker run -d --name "$container_name" alpine sleep 3600
done