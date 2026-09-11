import { StatusDot } from "@/ui/StatusDot";
import { useHermesHealth } from "@/data/hooks/useHermesHealth";
import { useVaultStatus } from "@/data/hooks/useVaultStatus";
import { useIndexStatus } from "@/data/hooks/useIndexStatus";
import { useZoom } from "@/navigation/zoom/useZoom";
import { DashboardCard, dashboardActionClass } from "./DashboardCard";

/*
 * Sistem durumu — Anayasa madde 18.3, 18.4, 24.2.
 *
 * 18.4 KRİTİK: "Hermes kapalı olması bir HATA DURUMU DEĞİLDİR.
 *       Kırmızı uyarı, modal, sesli bildirim veya tekrarlayan uyarı
 *       gösterilmez. Sadece gri bir göstergedir."
 *
 * Bu yüzden burada `isError` dalı YOK ve kırmızı renk HİÇ kullanılmıyor.
 * Tarama sürerken de uyarı değil, sakin bir durum gösterilir.
 */

export function SystemStatusCard() {
  const { data: hermes } = useHermesHealth();
  const { data: vault } = useVaultStatus();
  const { data: index } = useIndexStatus();
  const { enterLayer } = useZoom();

  return (
    <DashboardCard
      title="Sistem durumu"
      className="col-span-2"
      action={
        <button
          type="button"
          onClick={() => enterLayer("system")}
          className={dashboardActionClass}
        >
          Sistemi aç
        </button>
      }
    >
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-3 xl:grid-cols-1">
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
        <StatusDot
          status={vault?.path ? "online" : "offline"}
          label={vault?.path ? `Vault · ${vault.noteCount} not` : "Vault yapılandırılmadı"}
        />
        <StatusDot
          status={index?.ready && !index.rebuilding ? "online" : "offline"}
          label={
            index?.rebuilding
              ? "Index taranıyor"
              : index?.ready
                ? "Index hazır"
                : "Index bekleniyor"
          }
        />
      </div>
    </DashboardCard>
  );
}
