import { LayerHost } from "@/navigation/layers/LayerHost";
import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { useToday } from "@/data/hooks/useToday";
import { TaskGroup } from "./components/TaskGroup";

/*
 * Bugün katmanı — Anayasa madde 36.2 ("live" modül).
 *
 * Sprint 0: sahte veri (fixtures.ts). Sprint 1'de useToday'ın queryFn'i
 * index'e bağlanır; BU DOSYA DEĞİŞMEZ.
 *
 * Yükleniyor durumu: spinner YOK (madde 23.6, 28). Veri gelene kadar
 * kartlar boş durur — stale-while-revalidate ile pratikte anlık.
 */

export default function TodayLayer() {
  const { data } = useToday();

  return (
    <LayerHost title="Bugün">
      <div className="flex flex-col gap-6">
        {data && data.overdue.length > 0 ? (
          <Card title="Geciken">
            <TaskGroup tasks={data.overdue} emptyMessage="Geciken bir şey yok." />
          </Card>
        ) : null}

        <Card title="Bugün">
          <TaskGroup
            tasks={data?.due ?? []}
            emptyMessage="Bugün için planlanmış bir şey yok."
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
                  {/* Anayasa madde 16.3: yönetilmeyen not sessizce işaretlenir. */}
                  {!note.managed ? (
                    <span className="shrink-0 text-xs text-text-tertiary">
                      yönetilmiyor
                    </span>
                  ) : null}
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState message="Bugün henüz bir nota dokunulmadı." />
          )}
        </Card>
      </div>
    </LayerHost>
  );
}
