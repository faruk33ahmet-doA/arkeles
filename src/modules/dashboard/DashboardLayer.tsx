import { LayerHost } from "@/navigation/layers/LayerHost";
import { Card } from "@/ui/Card";
import { DASHBOARD_FIXTURE } from "@/modules/today/fixtures";
import { TaskGroup } from "@/modules/today/components/TaskGroup";
import { LifeScoreCard } from "./components/LifeScoreCard";
import { WorkspaceGrid } from "./components/WorkspaceGrid";
import { SystemStatusCard } from "./components/SystemStatusCard";

/*
 * Panel (Dashboard) — Anayasa madde 24. Seviye 0: evrenin merkezi.
 *
 * 24.1  Her gün açılan ANA ÇALIŞMA EKRANI, sadece giriş ekranı değil.
 * 24.2  Tek bakışta: bugünkü durum · kritik görevler · Hermes durumu ·
 *       çalışılan kurumlar · öncelikler · hayat skoru.
 * 24.3  TEK BAKIŞ KURALI: kaydırmadan görünür. Bu yüzden yerleşim sabit
 *       2 kolon × 2 satır; içerik artarsa kart eklemek yerine kart içi
 *       liste kısaltılır (madde 24.3: "bilgi eklenmez, çıkarılır").
 * 24.4  Süs grafik yok.
 * 24.5  Her öğe bir eyleme veya katmana açılır — Sprint 1'de tıklama
 *       hedefleri bağlanacak.
 *
 * Sprint 0: sahte veri. Hermes ve Vault kartı GERÇEK durum gösterir
 * (ikisi de kapalı → gri), çünkü o servisler Sprint 0'da mevcut.
 */

export default function DashboardLayer() {
  const { lifeScore, workspaces, criticalTasks } = DASHBOARD_FIXTURE;

  return (
    <LayerHost>
      <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
        {/* Kritik görevler — en geniş alan, çünkü en sık okunan (madde 24.2) */}
        <div className="md:col-span-2">
          <Card title="Kritik">
            <TaskGroup
              tasks={criticalTasks}
              emptyMessage="Kritik bir şey yok. Gün sakin."
            />
          </Card>
        </div>

        <div className="flex flex-col gap-6">
          <LifeScoreCard score={lifeScore} />
          <SystemStatusCard />
        </div>

        <div className="md:col-span-3">
          <WorkspaceGrid workspaces={workspaces} />
        </div>
      </div>
    </LayerHost>
  );
}
