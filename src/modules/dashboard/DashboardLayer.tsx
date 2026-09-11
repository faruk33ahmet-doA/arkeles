import { useEffect } from "react";
import { LayerHost } from "@/navigation/layers/LayerHost";
import { useDashboard } from "@/data/hooks/useDashboard";
import { useToday } from "@/data/hooks/useToday";
import { LifeScoreCard } from "./components/LifeScoreCard";
import { WorkspaceGrid } from "./components/WorkspaceGrid";
import { SystemStatusCard } from "./components/SystemStatusCard";
import { HermesCard } from "./components/HermesCard";
import { DashboardHeader } from "./components/DashboardHeader";
import { CriticalTasksCard } from "./components/CriticalTasksCard";
import { TodaySummaryCard } from "./components/TodaySummaryCard";
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
  const { data: today, isLoading: todayLoading } = useToday();

  // Soğuk açılış ölçümü: panel gerçek veriyle boyandığı an (madde 34.1).
  useEffect(() => {
    if (data) markFirstMeaningfulPaint();
  }, [data]);

  return (
    <LayerHost variant="dashboard">
      <DashboardHeader />

      <div className="grid grid-cols-1 gap-3 xl:grid-cols-12">
        <div className="flex flex-col gap-3 xl:col-span-8">
          <CriticalTasksCard tasks={data?.criticalTasks ?? []} loading={isLoading} />
          <WorkspaceGrid workspaces={data?.workspaces ?? []} loading={isLoading} />
        </div>

        <div className="grid grid-cols-2 content-start gap-3 xl:col-span-4">
          <TodaySummaryCard today={today} loading={todayLoading} />
          <LifeScoreCard score={data?.lifeScore ?? null} loading={isLoading} />
          {/* Sprint 4 madde 4, 13: kısa Hermes özeti. Detay ayrı katmanda. */}
          <HermesCard />
          <SystemStatusCard />
        </div>
      </div>
    </LayerHost>
  );
}
