#!/bin/sh
# Inicializa CouchDB: nodo único, base, usuarios y roles, validaciones e índices.
# Uso local: COUCH_URL=http://admin:admin123@localhost:5985 sh init_couchdb.sh
COUCH_URL="${COUCH_URL:-http://admin:admin123@localhost:5985}"
DBNAME="${DBNAME:-alquileres_p001}"
DB="$COUCH_URL/$DBNAME"
J='Content-Type: application/json'

echo "== Nodo único"
curl -s -X POST "$COUCH_URL/_cluster_setup" -H "$J" \
  -d '{"action":"enable_single_node","bind_address":"0.0.0.0","username":"admin","password":"admin123","port":5984}'; echo

echo "== Base de datos"
curl -s -X PUT "$DB"; echo

echo "== Usuarios y roles"
curl -s -X PUT "$COUCH_URL/_users/org.couchdb.user:api_sv" -H "$J" \
  -d '{"name":"api_sv","password":"api123","roles":["api"],"type":"user"}'; echo
curl -s -X PUT "$COUCH_URL/_users/org.couchdb.user:auditor" -H "$J" \
  -d '{"name":"auditor","password":"auditor123","roles":["lector"],"type":"user"}'; echo
curl -s -X PUT "$DB/_security" -H "$J" \
  -d '{"admins":{"names":[],"roles":[]},"members":{"names":[],"roles":["api","lector"]}}'; echo

echo "== Reglas de validación"
curl -s -X PUT "$DB/_design/validaciones" -H "$J" \
  -d '{"validate_doc_update":"function(newDoc, oldDoc, userCtx) { if (newDoc._id.indexOf(\"_design/\") === 0) return; if (userCtx.roles.indexOf(\"_admin\") === -1 && userCtx.roles.indexOf(\"api\") === -1) throw({unauthorized: \"solo el rol api puede escribir\"}); if (newDoc._deleted) return; var tipos = [\"propietario\",\"inmueble\",\"inquilino\",\"contrato\",\"pago\"]; if (tipos.indexOf(newDoc.type) === -1) throw({forbidden: \"type invalido\"}); if (newDoc._id.indexOf(newDoc.type + \":\") !== 0) throw({forbidden: \"_id debe iniciar con el type\"}); if (newDoc.type === \"inmueble\" && !(newDoc.canonMensual > 0)) throw({forbidden: \"canonMensual debe ser > 0\"}); if (newDoc.type === \"contrato\") { if (!newDoc.inmuebleId || !newDoc.inquilinoId) throw({forbidden: \"contrato requiere inmueble e inquilino\"}); if (newDoc.fechaFin <= newDoc.fechaInicio) throw({forbidden: \"fechaFin debe ser posterior a fechaInicio\"}); if (!(newDoc.diaPago >= 1 && newDoc.diaPago <= 28)) throw({forbidden: \"diaPago entre 1 y 28\"}); } if (newDoc.type === \"pago\" && [\"pendiente\",\"pagado\",\"atrasado\"].indexOf(newDoc.estado) === -1) throw({forbidden: \"estado de pago invalido\"}); }"}'; echo

echo "== Índices"
curl -s -X POST "$DB/_index" -H "$J" -d '{"index":{"fields":["type","contratoId"]},"name":"idx_pago_contrato","type":"json"}'; echo
curl -s -X POST "$DB/_index" -H "$J" -d '{"index":{"fields":["type","estado","fechaVencimiento"]},"name":"idx_pago_estado","type":"json"}'; echo
curl -s -X POST "$DB/_index" -H "$J" -d '{"index":{"fields":["type","propietarioId"]},"name":"idx_inmueble_prop","type":"json"}'; echo
curl -s -X POST "$DB/_index" -H "$J" -d '{"index":{"fields":["type","inmuebleId","estado"]},"name":"idx_contrato_inmueble","type":"json"}'; echo
curl -s -X POST "$DB/_index" -H "$J" -d '{"index":{"fields":["type"]},"name":"idx_type","type":"json"}'; echo
curl -s -X POST "$DB/_index" -H "$J" -d '{"index":{"fields":["type","estado"]},"name":"idx_type_estado","type":"json"}'; echo
echo "== Listo"
