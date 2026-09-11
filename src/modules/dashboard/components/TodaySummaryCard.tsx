import { useZoom } from "@/navigation/zoom/useZoom";
import type { TodayView } from "@/lib/generated";
import { DashboardCard, dashboardActionClass } from "./DashboardCard";

interface TodaySummaryCardProps {
  today: TodayView | undefined;
  loading?: boolean;
}

export function TodaySummaryCard({ today, loading = false }: TodaySummaryCardProps) {
  const { enterLayer } = useZoom();

  const metrics = [
    { label: "Bugün", value: today?.due.length ?? 0 },
    { label: "Geciken", value: today?.overdue.length ?? 0 },
    { label: "Dokunulan not", value: today?.touchedNotes.length ?? 0 },
  ];
  const lastTouched = today?.touchedNotes[0];

  return (
    <DashboardCard
      title="Günlük özet"
      className="col-span-2"
      action={
        <button
          type="button"
          onClick={() => enterLayer("today")}
          className={dashboardActionClass}
        >
          Ayrıntı
        </button>
      }
    >
      <div className="grid grid-cols-3 divide-x divide-border-subtle">
        {metrics.map((metric) => (
          <div key={metric.label} className="px-3 first:pl-0 last:pr-0">
            <strong className="block text-xl font-semibold tabular-nums text-text-primary">
              {loading ? "—" : metric.value}
            </strong>
            <span className="mt-1 block text-xs text-text-tertiary">{metric.label}</span>
          </div>
        ))}
      </div>

      {lastTouched ? (
        <div className="mt-4 border-t border-border-subtle pt-3">
          <p className="text-xs text-text-tertiary">Son aktivite</p>
          <p className="mt-1 truncate text-sm text-text-secondary">{lastTouched.title}</p>
        </div>
      ) : null}
    </DashboardCard>
  );
}
