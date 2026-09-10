import { useEffect, useState } from "react";
import { budgetFor, summarize, type Summary } from "@/lib/perf";
import { cn } from "@/lib/cn";

/*
 * Performans tablosu — Anayasa madde 34.2.
 *
 * "Bütçe her sprint sonunda ÖLÇÜLÜR ve RAPORLANIR."
 *
 * Bu tablo ölçümü GÖSTERİR, yargılamaz. Bütçe aşımı kırmızı yanıp sönmez
 * (madde 23.2, 25.2) — sadece sayı bütçenin yanında durur ve karar insana
 * kalır. Aşan satır sönük amber ile işaretlenir: dikkat çeker, bağırmaz.
 *
 * Ölçüm, ölçtüğü şeyi yavaşlatmamalı (madde 35.1): tablo yalnız bu katman
 * AÇIKKEN, 1 sn'de bir okur. Kapalıyken hiçbir iş yapılmaz.
 */

const REFRESH_MS = 1000;

function format(value: number, unit: "ms" | "%"): string {
  if (unit === "%") return `${value.toFixed(1)} %`;
  if (value < 10) return `${value.toFixed(1)} ms`;
  return `${Math.round(value)} ms`;
}

export function PerfTable() {
  const [rows, setRows] = useState<Summary[]>(() => summarize());

  useEffect(() => {
    const id = setInterval(() => setRows(summarize()), REFRESH_MS);
    return () => clearInterval(id);
  }, []);

  if (rows.length === 0) {
    return (
      <p className="text-base text-text-secondary">
        Henüz ölçüm yok. Katmanlar arasında gezindiğinde burada birikir.
      </p>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[520px] border-collapse text-sm">
        <thead>
          <tr className="border-b border-border-default text-left">
            <th className="py-2 pr-4 font-medium text-text-tertiary">Metrik</th>
            <th className="py-2 pr-4 text-right font-medium text-text-tertiary">p50</th>
            <th className="py-2 pr-4 text-right font-medium text-text-tertiary">p95</th>
            <th className="py-2 pr-4 text-right font-medium text-text-tertiary">En yüksek</th>
            <th className="py-2 pr-4 text-right font-medium text-text-tertiary">Bütçe</th>
            <th className="py-2 text-right font-medium text-text-tertiary">Örnek</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const budget = budgetFor(row.name);
            // Sprint 1 borcu #2: `probe:` metrikleri bütçeye tabi değil.
            const over = budget !== null && row.p95 > budget;
            return (
              <tr key={row.name} className="border-b border-border-subtle">
                <td className="py-2 pr-4 text-text-primary">{row.name}</td>
                <td className="py-2 pr-4 text-right tabular-nums text-text-secondary">
                  {format(row.p50, row.unit)}
                </td>
                <td
                  className={cn(
                    "py-2 pr-4 text-right tabular-nums",
                    over ? "text-status-attention" : "text-text-primary",
                  )}
                >
                  {format(row.p95, row.unit)}
                </td>
                <td className="py-2 pr-4 text-right tabular-nums text-text-secondary">
                  {format(row.max, row.unit)}
                </td>
                <td className="py-2 pr-4 text-right tabular-nums text-text-tertiary">
                  {budget !== null ? format(budget, row.unit) : "—"}
                </td>
                <td className="py-2 text-right tabular-nums text-text-tertiary">
                  {row.count}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
