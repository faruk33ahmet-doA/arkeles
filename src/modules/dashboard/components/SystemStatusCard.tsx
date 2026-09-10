import { Card } from "@/ui/Card";
import { StatusDot } from "@/ui/StatusDot";
import { useHermesHealth } from "@/data/hooks/useHermesHealth";
import { useVaultStatus } from "@/data/hooks/useVaultStatus";

/*
 * Sistem durumu — Anayasa madde 18.3, 18.4, 24.2.
 *
 * 18.4 KRİTİK: "Hermes kapalı olması bir HATA DURUMU DEĞİLDİR.
 *       Kırmızı uyarı, modal, sesli bildirim veya tekrarlayan uyarı
 *       gösterilmez. Sadece gri bir göstergedir."
 *
 * Bu yüzden burada `isError` dalı YOK. Sorgu başarısız olsa da sonuç
 * "ulaşılamıyor" — gri. Kırmızı renk bu bileşende hiç kullanılmaz.
 */

export function SystemStatusCard() {
  const { data: hermes } = useHermesHealth();
  const { data: vault } = useVaultStatus();

  return (
    <Card title="Sistem">
      <div className="flex flex-col gap-3">
        <StatusDot
          status={hermes?.reachable ? "online" : "offline"}
          label={
            hermes?.reachable
              ? `Hermes bağlı${hermes.version ? ` · ${hermes.version}` : ""}`
              : "Hermes ulaşılamıyor"
          }
        />
        <StatusDot
          status={vault?.path ? "online" : "offline"}
          label={
            vault?.path
              ? `Vault · ${vault.noteCount} not`
              : "Vault yapılandırılmadı"
          }
        />
      </div>
    </Card>
  );
}
