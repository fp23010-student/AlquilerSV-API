import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.API_URL || 'http://api:8080';
const VUS = parseInt(__ENV.VUS || '50');
const RUN = __ENV.RUN || 'x';
const JSON_H = { headers: { 'Content-Type': 'application/json' } };

export const options = {
  stages: [
    { duration: '20s', target: VUS },
    { duration: '40s', target: VUS },
    { duration: '10s', target: 0 },
  ],
  thresholds: { http_req_failed: ['rate<0.05'] },
};

export default function () {
  const r = Math.random();
  let res;
  if (r < 0.45) {
    res = http.get(`${BASE}/inmuebles`);
    check(res, { 'lectura 2xx': (x) => x.status >= 200 && x.status < 300 });
  } else if (r < 0.65) {
    res = http.get(`${BASE}/health`);
    check(res, { 'health 200': (x) => x.status === 200 });
  } else if (r < 0.85) {
    res = http.post(`${BASE}/inquilinos`,
      JSON.stringify({ nombre: `Carga ${__VU}-${__ITER}`, telefono: '7000-0000' }), JSON_H);
    check(res, { 'escritura 201': (x) => x.status === 201 });
  } else {
    const docs = [];
    for (let i = 0; i < 5; i++) {
      docs.push({ _id: `inquilino:k6-${RUN}-${__VU}-${__ITER}-${i}`, type: 'inquilino',
                  nombre: 'Sync', telefono: '7000-0001' });
    }
    res = http.post(`${BASE}/sync/push`, JSON.stringify({ docs }), JSON_H);
    check(res, { 'sync sin errores': (x) => {
      try { return x.status === 200 && JSON.parse(x.body).resultados.every((d) => d.ok === true); }
      catch (e) { return false; }
    } });
  }
  sleep(0.5 + Math.random());
}
