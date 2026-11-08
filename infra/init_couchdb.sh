#!/bin/bash
DB="http://admin:admin123@localhost:5985/alquileres_p001"

# 1) Restricciones (validate_doc_update en una sola línea)
curl -X PUT "$DB/_design/validaciones" \
  -H "Content-Type: application/json" \
  -d '{"validate_doc_update":"function(newDoc, oldDoc, userCtx) { if (newDoc._deleted) return; if (newDoc._id.indexOf(\"_design/\") === 0) return; var tipos = [\"propietario\",\"inmueble\",\"inquilino\",\"contrato\",\"pago\"]; if (tipos.indexOf(newDoc.type) === -1) throw({forbidden: \"type invalido\"}); if (newDoc._id.indexOf(newDoc.type + \":\") !== 0) throw({forbidden: \"_id debe iniciar con el type\"}); if (newDoc.type === \"inmueble\" && !(newDoc.canonMensual > 0)) throw({forbidden: \"canonMensual debe ser > 0\"}); if (newDoc.type === \"contrato\") { if (!newDoc.inmuebleId || !newDoc.inquilinoId) throw({forbidden: \"contrato requiere inmueble e inquilino\"}); if (newDoc.fechaFin <= newDoc.fechaInicio) throw({forbidden: \"fechaFin debe ser posterior a fechaInicio\"}); if (!(newDoc.diaPago >= 1 && newDoc.diaPago <= 28)) throw({forbidden: \"diaPago entre 1 y 28\"}); } if (newDoc.type === \"pago\" && [\"pendiente\",\"pagado\",\"atrasado\"].indexOf(newDoc.estado) === -1) throw({forbidden: \"estado de pago invalido\"}); }"}'

echo

# 2) Índices Mango
curl -X POST "$DB/_index" -H "Content-Type: application/json" \
  -d '{"index":{"fields":["type","contratoId"]},"name":"idx_pago_contrato","type":"json"}'

curl -X POST "$DB/_index" -H "Content-Type: application/json" \
  -d '{"index":{"fields":["type","estado","fechaVencimiento"]},"name":"idx_pago_estado","type":"json"}'

curl -X POST "$DB/_index" -H "Content-Type: application/json" \
  -d '{"index":{"fields":["type","propietarioId"]},"name":"idx_inmueble_prop","type":"json"}'

curl -X POST "$DB/_index" -H "Content-Type: application/json" \
  -d '{"index":{"fields":["type","inmuebleId","estado"]},"name":"idx_contrato_inmueble","type":"json"}'
