import { StatusDot } from "@/ui/StatusDot";
import { useHermesSummary } from "@/data/hooks/useHermes";
import { useZoom } from "@/navigation/zoom/useZoom";
import { DashboardCard, dashboardActionClass } from "./DashboardCard";

/*
 * Dashboard Hermes özeti — Sprint 4 madde 4, 13.
 *
 * "Bu alan dashboard'u kalabalıklaştırmamalı. Madde 24.3: kaydırmadan tek
 *  bakış kuralını koru." ve "uzun aktivite akışı dashboard'a dökülmemeli."
 *
 * Bu yüzden: bir durum satırı, bir sayım satırı, bir son-iş satırı.
 * Detay ayrı katmanda (Hermes).
 *
 * Madde 18.4: kapalı olmak HATA DEĞİL — gri nokta, kırmızı yok.
 */

export function HermesCard() {
  const { data } = useHermesSummary();
  const { enterLayer } = useZoom();

  const running = data?.running ?? 0;
  const queued = data?.queued ?? 0;
  const completed = data?.completedToday ?? 0;
  const failed = data?.failedToday ?? 0;

  const hasActivity = running + queued + completed + failed > 0;

  return (
    <DashboardCard
      title="Hermes"
      action={
        <button
          type="button"
          onClick={() => enterLayer("hermes")}
          className={dashboardActionClass}
        >
          Aktivite
        </button>
      }
    >
      <div className="flex min-h-dashboard-compact flex-col justify-between gap-3">
        <StatusDot
          status={data?.reachable ? "online" : "offline"}
          label={
            data === undefined
              ? "Yoklanıyor"
              : data.reachable
                ? `Bağlı${data.version ? ` · ${data.version}` : ""}`
                : "Ulaşılamıyor"
          }
        />

        {/* Sayımlar — yalnız sıfırdan farklı olanlar. Sıfırı duyurmak gürültü. */}
        <div className="grid grid-cols-3 divide-x divide-border-subtle border-t border-border-subtle pt-3">
          {[
            ["aktif", running],
            ["sırada", queued],
            ["biten", completed],
          ].map(([label, value]) => (
            <span key={label} className="px-2 first:pl-0 last:pr-0">
              <strong className="block text-base font-semibold tabular-nums text-text-primary">
                {value}
              </strong>
              <span className="block text-xs text-text-tertiary">{label}</span>
            </span>
          ))}
        </div>

        {/* Madde 15: başarısızlık görünür ama alarm değil. */}
        {failed > 0 ? (
          <StatusDot status="attention" label={`${failed} iş başarısız`} />
        ) : null}

        {!hasActivity ? (
          <p className="text-xs text-text-tertiary">Bugün kayıtlı iş yok.</p>
        ) : null}
      </div>
    </DashboardCard>
  );
}
