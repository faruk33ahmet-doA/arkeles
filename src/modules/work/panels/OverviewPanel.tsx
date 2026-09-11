import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { useWorkspaceOverview } from "@/data/hooks/useWork";
import { useHermesActions } from "@/data/hooks/useHermes";
import { useWorkStore } from "@/state/workStore";
import { cn } from "@/lib/cn";
import { ActionLauncher } from "@/modules/hermes/components/ActionLauncher";
import { Card as HermesCardShell } from "@/ui/Card";

/*
 * Genel Bakış — Sprint 3 madde 3.
 *
 * Yalnız MEVCUT VERİDEN üretilebilen bilgiler. Öncelik alanı Hermes
 * vault'a yazdıysa görünür; yazmadıysa TAMAMEN GİZLENİR (madde 11.2:
 * ARKELÉS öncelik analizi yapamaz).
 *
 * Süs grafik, dolan halka yok (madde 24.4, 32).
 */

function formatRelative(iso: string | null): string | null {
  if (!iso) return null;
  const time = new Date(iso).getTime();
  if (Number.isNaN(time)) return null;

  const hours = Math.floor((Date.now() - time) / 3_600_000);
  if (hours < 1) return "az önce";
  if (hours < 24) return `${hours} saat önce`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days} gün önce`;
  return iso.slice(0, 10);
}

export function OverviewPanel({ workspaceId }: { workspaceId: string }) {
  const { data, isLoading } = useWorkspaceOverview(workspaceId);
  const openNote = useWorkStore((s) => s.openNote);

  if (!data) {
    return isLoading ? null : <EmptyState message="Bu çalışma alanı henüz boş." />;
  }

  const lastActivity = formatRelative(data.lastActivity);
  const isEmpty =
    data.activeTasks === 0 &&
    data.waitingTasks === 0 &&
    data.doneTasks === 0 &&
    data.recentNotes.length === 0;

  if (isEmpty) {
    return (
      <div className="flex flex-col gap-6">
        <EmptyState
          message="Bu çalışma alanı henüz boş."
          hint="Obsidian'da bir nota workspace alanı eklediğinde burada görünür."
        />
        {/* Boş kurumda bile Hermes'e iş verilebilir — madde 18.2 filtresi geçerli. */}
        <HermesActions workspaceId={workspaceId} />
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
      <div className="md:col-span-3">
        <Card
          title="Görevler"
          action={
            lastActivity ? (
              <span className="text-xs text-text-tertiary">
                son hareket {lastActivity}
              </span>
            ) : null
          }
        >
          <div className="grid grid-cols-3 gap-6">
            <Stat label="Aktif" value={data.activeTasks} />
            <Stat label="Bekleyen" value={data.waitingTasks} />
            <Stat label="Tamamlanan" value={data.doneTasks} />
          </div>
        </Card>
      </div>

      {/* Madde 11.2: Hermes yazmadıysa bu kart HİÇ görünmez. */}
      {data.priority ? (
        <div className="md:col-span-2">
          <Card title="Bugünkü öncelik">
            <p className="select-text text-base text-text-primary">{data.priority}</p>
          </Card>
        </div>
      ) : null}

      {/* Yaklaşan toplantı yoksa alan gizlenir — boş kart gürültüdür. */}
      {data.nextMeeting ? (
        <Card title="Yaklaşan toplantı">
          <button
            type="button"
            onClick={() => openNote(data.nextMeeting!.id)}
            className={cn(
              "flex w-full flex-col items-start gap-1 rounded-sm text-left",
              "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
            )}
          >
            <span className="text-base text-text-primary">{data.nextMeeting.title}</span>
            <span className="text-xs tabular-nums text-text-tertiary">
              {data.nextMeeting.date}
            </span>
          </button>
        </Card>
      ) : null}

      {/*
        Sprint 4 madde 5: semantik aksiyonlar göreve bağlı yerde, kısa.
        Hermes yetenek bildirmiyorsa ActionLauncher HİÇ render edilmez
        ve bu kart da görünmez (madde 18.2).
      */}
      <div className="md:col-span-3">
        <HermesActions workspaceId={workspaceId} />
      </div>

      <div className="md:col-span-3">
        <Card title="Son notlar">
          {data.recentNotes.length > 0 ? (
            <ul className="divide-y divide-border-subtle">
              {data.recentNotes.map((note) => (
                <li key={note.id}>
                  <button
                    type="button"
                    onClick={() => openNote(note.id)}
                    className={cn(
                      "flex w-full items-center justify-between gap-4 py-2 text-left",
                      "outline-none hover:text-text-primary",
                      "focus-visible:ring-1 focus-visible:ring-border-strong",
                    )}
                  >
                    <span className="min-w-0 flex-1 truncate text-base text-text-primary">
                      {note.title}
                    </span>
                    {!note.managed ? (
                      <span className="shrink-0 text-xs text-text-tertiary">
                        yönetilmiyor
                      </span>
                    ) : null}
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState message="Henüz not yok." />
          )}
        </Card>
      </div>
    </div>
  );
}

/** Sayım göstergesi. Madde 11.5: sayı bilgi verir, yargı vermez. */
function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-xl font-semibold tabular-nums text-text-primary">
        {value}
      </span>
      <span className="text-xs uppercase tracking-[0.08em] text-text-tertiary">
        {label}
      </span>
    </div>
  );
}

/*
 * Hermes aksiyon yüzeyi — Sprint 4 madde 5, madde 18.2.
 *
 * Yetenek yoksa KART DA GÖRÜNMEZ. Boş bir "Hermes" başlığı göstermek
 * gürültü olurdu; bildirilmeyen yetenek arayüzde HİÇ yer kaplamaz.
 *
 * Kullanılabilirliği `ActionLauncher`'ın render sonucundan çıkarmak yerine
 * aynı kancadan okuruz — React elemanının içine bakmak kırılgan olurdu.
 */
function HermesActions({ workspaceId }: { workspaceId: string }) {
  const { data: actions } = useHermesActions();
  if (!actions || actions.length === 0) return null;

  return (
    <HermesCardShell title="Hermes">
      <ActionLauncher workspace={workspaceId} />
    </HermesCardShell>
  );
}
