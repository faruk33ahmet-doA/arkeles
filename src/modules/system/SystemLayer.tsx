import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { LayerHost } from "@/navigation/layers/LayerHost";
import { Card } from "@/ui/Card";
import { StatusDot } from "@/ui/StatusDot";
import { useVaultStatus } from "@/data/hooks/useVaultStatus";
import { useIndexStatus } from "@/data/hooks/useIndexStatus";
import { useHermesHealth } from "@/data/hooks/useHermesHealth";
import { rebuildIndex } from "@/services/index.service";
import { VAULT_DEPENDENT_KEYS, queryKeys } from "@/data/queryKeys";
import { getColdStartMs, getRefreshIntervalMs, BUDGETS } from "@/lib/perf";
import { PerfTable } from "./components/PerfTable";
import { cn } from "@/lib/cn";
import type { ScanReportDto } from "@/lib/generated";

/*
 * Sistem katmanı — Anayasa madde 36.2 ("Sistem: gerçek veri").
 *
 * İçeriği: vault / index / Hermes durumu + performans ölçümleri (madde 34.2)
 * + index'i yeniden kurma (madde 9.3'ün kullanıcıya açılan yüzü).
 *
 * Burada AI yok, analiz yok, öneri yok (madde 7.3). Yalnız sayılar ve
 * tek bir mekanik eylem.
 */

export default function SystemLayer() {
  const queryClient = useQueryClient();
  const { data: vault } = useVaultStatus();
  const { data: index } = useIndexStatus();
  const { data: hermes } = useHermesHealth();
  const [report, setReport] = useState<ScanReportDto | null>(null);

  const rebuild = useMutation({
    mutationFn: rebuildIndex,
    onSuccess: (result) => {
      setReport(result);
      for (const key of VAULT_DEPENDENT_KEYS) {
        queryClient.invalidateQueries({ queryKey: key });
      }
      queryClient.invalidateQueries({ queryKey: queryKeys.index.status });
    },
  });

  const coldStart = getColdStartMs();
  const refresh = getRefreshIntervalMs();

  return (
    <LayerHost title="Sistem">
      <div className="flex flex-col gap-6">
        <Card title="Durum">
          <div className="flex flex-col gap-3">
            <StatusDot
              status={vault?.path ? "online" : "offline"}
              label={
                vault?.path
                  ? `Vault · ${vault.noteCount} not`
                  : "Vault yapılandırılmadı"
              }
            />
            {vault?.path ? (
              <p className="select-text break-all pl-4 text-xs text-text-tertiary">
                {vault.path}
              </p>
            ) : null}
            <StatusDot
              status={index?.ready ? "online" : "offline"}
              label={
                index?.rebuilding
                  ? "Index taranıyor"
                  : `Index şema v${index?.schemaVersion ?? 0}`
              }
            />
            {/*
              Sprint 1 borcu #4: satır hataları artık GÖRÜNÜR.
              0 ise hiç gösterilmez — sıfırı duyurmak gürültüdür (madde 5).
            */}
            {index && index.rowErrors > 0 ? (
              <StatusDot
                status="attention"
                label={`${index.rowErrors} satır okunamadı`}
              />
            ) : null}
            {vault?.indexedAt ? (
              <p className="pl-4 text-xs text-text-tertiary">
                Son tarama: {vault.indexedAt}
              </p>
            ) : null}
            <StatusDot
              status={hermes?.reachable ? "online" : "offline"}
              label={
                hermes === undefined
                  ? "Hermes yoklanıyor"
                  : hermes.reachable
                    ? `Hermes bağlı${hermes.version ? ` · ${hermes.version}` : ""}`
                    : "Hermes ulaşılamıyor"
              }
            />
            {/* Madde 18.2: yetenekler yalnız BİLDİRİLDİĞİNDE görünür. */}
            {hermes?.reachable && hermes.capabilities.length > 0 ? (
              <p className="pl-4 text-xs text-text-tertiary">
                Yetenekler: {hermes.capabilities.map((item) => item.label).join(" · ")}
              </p>
            ) : null}
          </div>
        </Card>

        <Card
          title="Performans"
          action={
            coldStart !== null ? (
              <span
                className={cn(
                  "text-xs tabular-nums",
                  coldStart > BUDGETS.coldStartMs
                    ? "text-status-attention"
                    : "text-text-secondary",
                )}
              >
                soğuk açılış {Math.round(coldStart)} ms / {BUDGETS.coldStartMs} ms
              </span>
            ) : null
          }
        >
          {/* Sprint 1 borcu #3: jank ölçümü ekranın kendi hızına göre. */}
          {refresh !== null ? (
            <p className="mb-4 text-xs text-text-tertiary">
              Ekran kare aralığı {refresh.toFixed(1)} ms
              {" · "}
              düşen kare ölçümü buna göre yapılıyor
            </p>
          ) : null}
          <PerfTable />
        </Card>

        <Card title="Index">
          <p className="mb-4 text-base text-text-secondary">
            Index türetilmiş veridir. Silinip yeniden kurulması hiçbir bilgi
            kaybettirmez.
          </p>

          <button
            type="button"
            onClick={() => rebuild.mutate()}
            disabled={rebuild.isPending || !vault?.path}
            className={cn(
              "rounded-sm border border-border-strong px-3 py-2 text-sm font-medium",
              "text-text-primary transition-colors duration-fast ease-out",
              "outline-none hover:bg-surface-3 focus-visible:ring-1 focus-visible:ring-border-strong",
              "disabled:cursor-default disabled:text-text-tertiary",
            )}
          >
            {rebuild.isPending ? "Taranıyor…" : "Yeniden tara"}
          </button>

          {report ? (
            <dl className="mt-4 grid grid-cols-2 gap-x-6 gap-y-2 text-sm">
              <dt className="text-text-tertiary">Tarandı</dt>
              <dd className="text-right tabular-nums text-text-primary">
                {report.scanned}
              </dd>
              <dt className="text-text-tertiary">Yeniden index'lendi</dt>
              <dd className="text-right tabular-nums text-text-primary">
                {report.reindexed}
              </dd>
              <dt className="text-text-tertiary">Değişmediği için atlandı</dt>
              <dd className="text-right tabular-nums text-text-primary">
                {report.unchanged}
              </dd>
              <dt className="text-text-tertiary">Kaldırıldı</dt>
              <dd className="text-right tabular-nums text-text-primary">
                {report.removed}
              </dd>
              <dt className="text-text-tertiary">Süre</dt>
              <dd
                className={cn(
                  "text-right tabular-nums",
                  report.durationMs > BUDGETS.fullScanMs
                    ? "text-status-attention"
                    : "text-text-primary",
                )}
              >
                {report.durationMs} ms / {BUDGETS.fullScanMs} ms
              </dd>
            </dl>
          ) : null}
        </Card>
      </div>
    </LayerHost>
  );
}
