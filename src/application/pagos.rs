use chrono::{Datelike, NaiveDate};

/// Un vencimiento por mes, en el día de pago del contrato.
/// Solo incluye vencimientos que caen dentro de la vigencia.
pub fn generar_periodos(
    inicio: NaiveDate,
    fin: NaiveDate,
    dia_pago: u32,
) -> Vec<(String, NaiveDate)> {
    let mut res = vec![];
    let (mut y, mut m) = (inicio.year(), inicio.month());
    while (y, m) <= (fin.year(), fin.month()) {
        if let Some(venc) = NaiveDate::from_ymd_opt(y, m, dia_pago) {
            if venc >= inicio && venc <= fin {
                res.push((format!("{:04}-{:02}", y, m), venc));
            }
        }
        if m == 12 {
            y += 1;
            m = 1;
        } else {
            m += 1;
        }
    }
    res
}

/// Días de atraso (0 si se pagó a tiempo o antes).
pub fn dias_atraso(vencimiento: NaiveDate, pago: NaiveDate) -> i64 {
    (pago - vencimiento).num_days().max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn contrato_de_un_anio_genera_12_pagos() {
        let p = generar_periodos(d("2026-11-01"), d("2027-10-31"), 5);
        assert_eq!(p.len(), 12);
        assert_eq!(p[0].0, "2026-11");
        assert_eq!(p[11].0, "2027-10");
    }

    #[test]
    fn calcula_atraso() {
        assert_eq!(dias_atraso(d("2026-11-05"), d("2026-11-08")), 3);
        assert_eq!(dias_atraso(d("2026-11-05"), d("2026-11-02")), 0);
    }
}