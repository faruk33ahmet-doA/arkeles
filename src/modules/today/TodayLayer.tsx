import { useEffect } from "react";
import { LayerHost } from "@/navigation/layers/LayerHost";
import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { useToday } from "@/data/hooks/useToday";
import { TaskGroup } from "./components/TaskGroup";
import { markFirstMeaningfulPaint } from "@/lib/perf";

/*
 * Bugün katmanı — Anayasa madde 36.2. GERÇEK VERİ (Sprint 1).
 *
 * Yükleniyor durumu: spinner YOK (madde 23.6, 28). Veri gelene kadar
 * kartlar boş durur; stale-while-revalidate ile ikinci açılıştan sonra anlık.
 */

export default function TodayLayer() {
  const { data, isLoading } = useToday();

  useEffect(() => {
    if (data) markFirstMeaningfulPaint();
  }, [data]);

  const hasOverdue = (data?.overdue.length ?? 0) > 0;

  return (
    <LayerHost title="Bugün">
      <div className="flex flex-col gap-6">
        {/* Geciken kartı yalnız içerik VARSA görünür — boş kart gürültüdür (madde 5). */}
        {hasOverdue ? (
          <Card title="Geciken">
            <TaskGroup tasks={data!.overdue} emptyMessage="Geciken bir şey yok." />
          </Card>
        ) : null}

        <Card title="Bugün">
          <TaskGroup
            tasks={data?.due ?? []}
            emptyMessage={
              isLoading ? "" : "Bugün için planlanmış bir şey yok."
            }
          />
        </Card>

        <Card title="Bugün dokunulan notlar">
          {data && data.touchedNotes.length > 0 ? (
            <ul className="divide-y divide-border-subtle">
              {data.touchedNotes.map((note) => (
                <li
                  key={note.id}
                  className="flex items-center justify-between gap-4 py-2"
                >
                  <span className="min-w-0 flex-1 truncate text-base text-text-primary">
                    {note.title}
                  </span>
                  {/* Madde 16.3: yönetilmeyen not sessizce işaretlenir. */}
                  {!note.managed ? (
                    <span className="shrink-0 text-xs text-text-tertiary">
                      yönetilmiyor
                    </span>
                  ) : null}
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState
              message={isLoading ? "" : "Bugün henüz bir nota dokunulmadı."}
            />
          )}
        </Card>
      </div>
    </LayerHost>
  );
}
