# Alquileres SV - API

API en Rust (Axum) con CouchDB para la gestión de alquileres.
App móvil: ENLACE_AL_REPO_DE_LA_APP

## Requisitos
Rust (cargo) y Docker.

## Cómo correrlo en tu laptop
1. Variables de entorno: `cp .env.example .env`
2. Levantar CouchDB: `cd infra && docker compose up -d`
3. Esperar unos 10 segundos e inicializar el nodo:
   curl -X POST http://admin:admin123@localhost:5985/_cluster_setup -H "Content-Type: application/json" -d '{"action":"enable_single_node","bind_address":"0.0.0.0","username":"admin","password":"admin123","port":5984}'
4. Crear la base y sus reglas (aún dentro de `infra/`):
   curl -X PUT http://admin:admin123@localhost:5985/alquileres_p001
   bash init_couchdb.sh
5. Volver a la raíz y correr: `cd .. && cargo run`
6. Comprobar: `curl http://localhost:8080/health`

CouchDB queda en el puerto 5985 (panel en http://localhost:5985/_utils) y la API en el 8080.

## Conectar el teléfono
Instalar la APK (ver el repositorio de la app). En la app, ir a Ajustes y escribir la
dirección de tu laptop, por ejemplo http://192.168.1.X:8080 (la IP sale con `hostname -I`).
El teléfono y la laptop deben estar en la misma red Wi-Fi.
