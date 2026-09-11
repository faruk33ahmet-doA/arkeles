import { useMemo, useState } from "react";
import { LayerHost } from "@/navigation/layers/LayerHost";
import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { StatusDot } from "@/ui/StatusDot";
import { VirtualList } from "@/ui/VirtualList";
import {
  useActiveJobs,
  useHermesActions,
  useHermesSummary,
  useJobHistory,
} from "@/data/hooks/useHermes";
import { useWorkspaces } from "@/data/hooks/useWork";
import { JobRow } from "./components/JobRow";
import { ActionLauncher } from "./components/ActionLauncher";
import { cn } from "@/lib/cn";

/*
 * Hermes aktivite yüzeyi — Sprint 4 madde 4, 14.
 *
 * "Hedef: 'Ne yaptı?' sorusuna cevap vermek."
 * Bunu bir CRM/activity-feed karmaşasına çevirmemek için: iki zaman
 * penceresi, üç filtre, tek liste. Grafik yok, sayaç panosu yok.
 *
 * ARKELÉS Hermes ARAYÜZÜ DEĞİLDİR (madde 7.3) — burada iş başlatılmaz,
 * yalnız görülür. Başlatma iş modülünde, göreve bağlı yerde.
 */

type Window = "today" | "week";

function sinceFor(window: Window): string {
  const date = new Date();
  if (window === "week") date.setDate(date.getDate() - 6);
  return date.toISOString().slice(0, 10);
}

const STATUS_FILTERS = [
  { id: null, label: "Hepsi" },
  { id: "running", label: "Çalışan" },
  { id: "completed", label: "Tamamlanan" },
  { id: "failed", label: "Başarısız" },
] as const;

export default function HermesLayer() {
  const [window, setWindow] = useState<Window>("today");
  const [status, setStatus] = useState<string | null>(null);
  const [workspace, setWorkspace] = useState<string | null>(null);

  const { data: summary } = useHermesSummary();
  const { data: active } = useActiveJobs();
  const { data: workspaces } = useWorkspaces();
  const { data: hermesAvailable } = useHermesActions();

  const since = useMemo(() => sinceFor(window), [window]);
  const { data: history, isLoading } = useJobHistory(since, workspace, status);

  const activeJobs = active ?? [];
  const historyJobs = history ?? [];

  return (
    <LayerHost title="Hermes">
      <div className="flex flex-col gap-6">
        <Card title="Durum">
          <div className="flex flex-col gap-3">
            <StatusDot
              status={summary?.reachable ? "online" : "offline"}
              label={
                summary === undefined
                  ? "Yoklanıyor"
                  : summary.reachable
                    ? `Bağlı${summary.version ? ` · ${summary.version}` : ""}`
                    : "Ulaşılamıyor"
              }
            />
            {/* Madde 11.5: SAYIM. Analiz değil. */}
            <p className="pl-4 text-sm text-text-secondary">
              {summary?.running ?? 0} çalışıyor · {summary?.queued ?? 0} bekliyor ·{" "}
              {summary?.completedToday ?? 0} bugün tamamlandı
              {summary && summary.failedToday > 0
                ? ` · ${summary.failedToday} başarısız`
                : ""}
            </p>
          </div>
        </Card>

        {/*
          Kurumsuz aksiyon yüzeyi — Cmd+K'dan gelen genel istekler burada
          açılır. Yetenek yoksa ActionLauncher null döner ve kart görünmez
          (madde 18.2).
        */}
        {(hermesAvailable?.length ?? 0) > 0 ? (
          <Card title="Hermes'e iş ver">
            <ActionLauncher workspace={null} />
          </Card>
        ) : null}

        {/* Aktif kuyruk — boşsa kart HİÇ görünmez (madde 5: gürültü yok). */}
        {activeJobs.length > 0 ? (
          <Card title="Kuyruk">
            <VirtualList
              items={activeJobs}
              estimateSize={64}
              keyOf={(job) => job.id}
              renderItem={(job) => <JobRow job={job} />}
            />
          </Card>
        ) : null}

        <Card
          title="Geçmiş"
          action={
            <div className="flex gap-1">
              <FilterButton
                active={window === "today"}
                onClick={() => setWindow("today")}
              >
                Bugün
              </FilterButton>
              <FilterButton active={window === "week"} onClick={() => setWindow("week")}>
                Son 7 gün
              </FilterButton>
            </div>
          }
        >
          <div className="mb-4 flex flex-wrap gap-1">
            {STATUS_FILTERS.map((filter) => (
              <FilterButton
                key={filter.label}
                active={status === filter.id}
                onClick={() => setStatus(filter.id)}
              >
                {filter.label}
              </FilterButton>
            ))}

            {/* Çalışma alanı filtresi — yalnız veri varsa anlamlı. */}
            {(workspaces ?? [])
              .filter((ws) => ws.active)
              .map((ws) => (
                <FilterButton
                  key={ws.id}
                  active={workspace === ws.id}
                  onClick={() => setWorkspace(workspace === ws.id ? null : ws.id)}
                >
                  {ws.label}
                </FilterButton>
              ))}
          </div>

          {historyJobs.length > 0 ? (
            <VirtualList
              items={historyJobs}
              estimateSize={64}
              keyOf={(job) => job.id}
              renderItem={(job) => <JobRow job={job} />}
            />
          ) : isLoading ? null : (
            <EmptyState
              message="Bu aralıkta iş yok."
              hint="İş modülünden Hermes'e görev verdiğinde burada görünür."
            />
          )}
        </Card>
      </div>
    </LayerHost>
  );
}

function FilterButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={active}
      className={cn(
        "rounded-sm px-2 py-1 text-xs font-medium",
        "transition-colors duration-fast ease-out outline-none",
        "focus-visible:ring-1 focus-visible:ring-border-strong",
        active
          ? "bg-accent-muted text-text-primary"
          : "text-text-secondary hover:text-text-primary",
      )}
    >
      {children}
    </button>
  );
}
