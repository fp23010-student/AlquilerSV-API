#!/bin/sh
cd "$(dirname "$0")"
mkdir -p resultados
VUS=${VUS:-100}
REDES=${REDES:-"ideal lenta perdida mala"}
REPS=${REPS:-3}
ADMIN=http://admin:admin123@localhost:5985
DB=alquileres_p001

reiniciar_datos() {
  curl -s -X DELETE "$ADMIN/$DB" > /dev/null
  docker compose run --rm init > /dev/null 2>&1
  seq 1 100 | awk 'BEGIN{printf "{\"docs\":["} {printf "%s{\"_id\":\"inmueble:semilla-%d\",\"type\":\"inmueble\",\"propietarioId\":\"propietario:semilla\",\"tipo\":\"casa\",\"direccion\":\"Calle %d\",\"canonMensual\":150}", (NR>1?",":""), $1, $1} END{printf "]}"}' > /tmp/semilla.json
  curl -s -X POST localhost:8080/sync/push -H "Content-Type: application/json" -d @/tmp/semilla.json > /dev/null
}

LISTA=""
for r in $(seq 1 $REPS); do for c in $REDES; do LISTA="$LISTA $c"; done; done

for c in $(echo $LISTA | tr ' ' '\n' | shuf); do
  ID=$(date +%H%M%S)
  echo "=== red=$c vus=$VUS corrida=$ID"
  reiniciar_datos
  docker compose restart api >/dev/null
  sleep 5
  ( while true; do
      docker stats --no-stream --format '{{.Name}},{{.CPUPerc}},{{.MemUsage}}' sv_api sv_couchdb \
        >> resultados/recursos_${c}_${VUS}vus_${ID}.csv
      sleep 2
    done ) &
  MON=$!
  RED=$c VUS=$VUS RUN=$ID docker compose run --rm -e RED -e VUS -e RUN cliente
  kill $MON 2>/dev/null; wait $MON 2>/dev/null
  curl -s localhost:8080/metrics > resultados/metrics_${c}_${VUS}vus_${ID}.json
done
echo "Listo. Resultados en infra/resultados/"
