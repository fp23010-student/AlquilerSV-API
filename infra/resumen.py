#!/usr/bin/env python3
"""Resume las corridas de infra/resultados en una tabla y en resumen.csv"""
import csv, glob, json, os, re
from collections import defaultdict

DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "resultados")
PAT = re.compile(r"^(k6|metrics|recursos)_([a-z]+)_(\d+)vus_(\d+)\.(json|csv)$")
UNIDAD = {"kib": 1/1024, "kb": 1/1024, "mib": 1, "mb": 1, "gib": 1024, "gb": 1024, "b": 1/1048576}

def mib(texto):
    m = re.match(r"([\d.]+)\s*([A-Za-z]+)", texto.split("/")[0].strip())
    return float(m.group(1)) * UNIDAD.get(m.group(2).lower(), 1) if m else 0.0

def media(lista):
    v = [x for x in lista if x is not None]
    return sum(v) / len(v) if v else None

runs = defaultdict(dict)
for ruta in glob.glob(os.path.join(DIR, "*")):
    m = PAT.match(os.path.basename(ruta))
    if not m:
        continue
    tipo, red, vus, rid = m.group(1), m.group(2), int(m.group(3)), m.group(4)
    r = runs[(vus, red, rid)]
    try:
        if tipo == "k6":
            met = json.load(open(ruta)).get("metrics", {})
            dur = met.get("http_req_duration", {})
            r["k6_avg_ms"] = dur.get("avg")
            r["k6_p95_ms"] = dur.get("p(95)")
            r["fallos_pct"] = met.get("http_req_failed", {}).get("value", 0) * 100
            chk = met.get("checks", {}).get("value")
            r["checks_pct"] = chk * 100 if chk is not None else None
            r["reqs_s"] = met.get("http_reqs", {}).get("rate")
        elif tipo == "metrics":
            d = json.load(open(ruta))
            rutas = d.get("rutas", [])
            n = sum(x["peticiones"] for x in rutas) or 1
            r["srv_media_ms"] = sum(x["latenciaMediaMs"] * x["peticiones"] for x in rutas) / n
            r["srv_bajo15_pct"] = sum(x["porcentajeBajo15ms"] * x["peticiones"] for x in rutas) / n
            r["srv_5xx"] = sum(x["errores5xx"] for x in rutas)
            r["api_rss_mb"] = d.get("memoriaRssMb")
        elif tipo == "recursos":
            cpu, mem = defaultdict(float), defaultdict(float)
            for fila in csv.reader(open(ruta)):
                if len(fila) < 3:
                    continue
                nombre = fila[0]
                cpu[nombre] = max(cpu[nombre], float(fila[1].strip("%") or 0))
                mem[nombre] = max(mem[nombre], mib(fila[2]))
            r["api_cpu_max"], r["api_ram_max_mib"] = cpu["sv_api"], mem["sv_api"]
            r["couch_cpu_max"], r["couch_ram_max_mib"] = cpu["sv_couchdb"], mem["sv_couchdb"]
    except Exception as e:
        print("No se pudo leer", os.path.basename(ruta), "->", e)

CAMPOS = ["k6_avg_ms", "k6_p95_ms", "fallos_pct", "checks_pct", "reqs_s",
          "srv_media_ms", "srv_bajo15_pct", "srv_5xx", "api_rss_mb",
          "api_cpu_max", "api_ram_max_mib", "couch_cpu_max", "couch_ram_max_mib"]

grupos = defaultdict(list)
for (vus, red, rid), r in runs.items():
    grupos[(vus, red)].append(r)

filas = []
for (vus, red) in sorted(grupos):
    g = grupos[(vus, red)]
    filas.append([vus, red, len(g)] + [media([x.get(c) for x in g]) for c in CAMPOS])

def f(x):
    return "-" if x is None else f"{x:.1f}"

print(f"{'VUS':>4} {'red':<8}{'n':>2} {'k6 p95 ms':>10} {'fallos %':>9} {'checks %':>9} "
      f"{'srv ms':>7} {'<15ms %':>8} {'API MiB':>8} {'API CPU':>8}")
for fl in filas:
    d = dict(zip(CAMPOS, fl[3:]))
    print(f"{fl[0]:>4} {fl[1]:<8}{fl[2]:>2} {f(d['k6_p95_ms']):>10} {f(d['fallos_pct']):>9} "
          f"{f(d['checks_pct']):>9} {f(d['srv_media_ms']):>7} {f(d['srv_bajo15_pct']):>8} "
          f"{f(d['api_ram_max_mib']):>8} {f(d['api_cpu_max']):>8}")

with open(os.path.join(DIR, "resumen.csv"), "w", newline="") as out:
    w = csv.writer(out)
    w.writerow(["vus", "red", "corridas"] + CAMPOS)
    w.writerows([[("" if v is None else (round(v, 2) if isinstance(v, float) else v)) for v in fl] for fl in filas])
print("\nGuardado en resultados/resumen.csv")
