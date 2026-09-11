import { Card } from "@/ui/Card";
import { StatusDot } from "@/ui/StatusDot";
import { useHermesSummary } from "@/data/hooks/useHermes";
import { useZoom } from "@/navigation/zoom/useZoom";
import { cn } from "@/lib/cn";

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
    <Card
      title="Hermes"
      action={
        hasActivity ? (
          <button
            type="button"
            onClick={() => enterLayer("hermes")}
            className={cn(
              "rounded-sm px-1 text-xs text-text-tertiary",
              "transition-colors duration-fast ease-out outline-none",
              "hover:text-text-primary focus-visible:ring-1 focus-visible:ring-border-strong",
            )}
          >
            tümü
          </button>
        ) : null
      }
    >
      <div className="flex flex-col gap-3">
        <StatusDot
          status={data?.reachable ? "online" : "offline"}
          label={
            data?.reachable
              ? `Bağlı${data.version ? ` · ${data.version}` : ""}`
              : "Ulaşılamıyor"
          }
        />

        {/* Sayımlar — yalnız sıfırdan farklı olanlar. Sıfırı duyurmak gürültü. */}
        {hasActivity ? (
          <p className="pl-4 text-sm text-text-secondary">
            {[
              running > 0 ? `${running} iş çalışıyor` : null,
              queued > 0 ? `${queued} bekliyor` : null,
              completed > 0 ? `${completed} bugün tamamlandı` : null,
            ]
              .filter(Boolean)
              .join(" · ")}
          </p>
        ) : null}

        {/* Madde 15: başarısızlık görünür ama alarm değil. */}
        {failed > 0 ? (
          <StatusDot status="attention" label={`${failed} iş başarısız`} />
        ) : null}

        {/* Son tamamlanan — TEK satır (madde 13). */}
        {data?.lastCompleted ? (
          <p className="truncate pl-4 text-xs text-text-tertiary">
            Son: {data.lastCompleted.summary}
          </p>
        ) : null}
      </div>
    </Card>
  );
}
