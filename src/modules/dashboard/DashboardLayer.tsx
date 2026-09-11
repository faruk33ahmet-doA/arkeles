import { useEffect } from "react";
import { LayerHost } from "@/navigation/layers/LayerHost";
import { Card } from "@/ui/Card";
import { useDashboard } from "@/data/hooks/useDashboard";
import { TaskGroup } from "@/modules/today/components/TaskGroup";
import { LifeScoreCard } from "./components/LifeScoreCard";
import { WorkspaceGrid } from "./components/WorkspaceGrid";
import { SystemStatusCard } from "./components/SystemStatusCard";
import { HermesCard } from "./components/HermesCard";
import { markFirstMeaningfulPaint } from "@/lib/perf";

/*
 * Panel — Anayasa madde 24. Seviye 0: evrenin merkezi. GERÇEK VERİ (Sprint 1).
 *
 * 24.1  Her gün açılan ANA ÇALIŞMA EKRANI.
 * 24.3  TEK BAKIŞ KURALI: kaydırmadan görünür. Kritik görev sayısı çekirdekte
 *       6 ile sınırlı (query.rs CRITICAL_LIMIT) — bilgi eklenmez, çıkarılır.
 * 24.4  Süs grafik yok.
 */

export default function DashboardLayer() {
  const { data, isLoading } = useDashboard();

  // Soğuk açılış ölçümü: panel gerçek veriyle boyandığı an (madde 34.1).
  useEffect(() => {
    if (data) markFirstMeaningfulPaint();
  }, [data]);

  return (
    <LayerHost>
      <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
        {/* Kritik görevler — en geniş alan, çünkü en sık okunan (madde 24.2) */}
        <div className="md:col-span-2">
          <Card title="Kritik">
            <TaskGroup
              tasks={data?.criticalTasks ?? []}
              emptyMessage={isLoading ? "" : "Kritik bir şey yok. Gün sakin."}
            />
          </Card>
        </div>

        <div className="flex flex-col gap-6">
          <LifeScoreCard score={data?.lifeScore ?? null} loading={isLoading} />
          {/* Sprint 4 madde 4, 13: kısa Hermes özeti. Detay ayrı katmanda. */}
          <HermesCard />
          <SystemStatusCard />
        </div>

        <div className="md:col-span-3">
          <WorkspaceGrid workspaces={data?.workspaces ?? []} loading={isLoading} />
        </div>
      </div>
    </LayerHost>
  );
}
