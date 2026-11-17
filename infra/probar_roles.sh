#!/bin/sh
DB=localhost:5985/alquileres_p001
echo "== 1. El auditor (rol lector) puede leer"
curl -s http://auditor:auditor123@$DB/_all_docs
echo
echo "== 2. El auditor NO puede escribir"
curl -s -X POST http://auditor:auditor123@$DB -H "Content-Type: application/json" -d '{"_id":"inquilino:prueba","type":"inquilino","nombre":"X","telefono":"1"}'
echo
echo "== 3. Sin credenciales no entra"
curl -s http://$DB
echo
