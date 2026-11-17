#!/bin/sh
RED="${RED:-ideal}"; VUS="${VUS:-50}"; RUN="${RUN:-$(date +%H%M%S)}"
case "$RED" in
  ideal) ;;
  lenta)   tc qdisc add dev eth0 root netem delay 150ms 30ms ;;
  perdida) tc qdisc add dev eth0 root netem loss 10% ;;
  mala)    tc qdisc add dev eth0 root netem delay 200ms 50ms loss 15% rate 512kbit ;;
  *) echo "RED desconocida: $RED"; exit 1 ;;
esac
echo "== red=$RED vus=$VUS"
tc qdisc show dev eth0
k6 run -e VUS="$VUS" -e RUN="$RUN" \
  --summary-export "/resultados/k6_${RED}_${VUS}vus_${RUN}.json" /scripts/carga.js
