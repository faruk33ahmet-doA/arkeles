import { useMutation, useQueryClient } from "@tanstack/react-query";
import { MutationError, setTaskStatus } from "@/services/mutation.service";
import { queryKeys, VAULT_DEPENDENT_KEYS } from "@/data/queryKeys";
import { useConflictStore } from "@/state/conflictStore";
import { startInteraction } from "@/lib/perf";
import type { DashboardView, TodayView } from "@/lib/generated";

/*
 * Görev durumu mutasyonu — Anayasa madde 5 (optimistic UI), 20.3, 34.1.
 *
 * OPTIMISTIC UI (madde 34.1: etkileşim → tepki < 100 ms):
 * `onMutate` önbelleği ANINDA günceller; dosya yazımını beklemez.
 * Kullanıcı checkbox'a bastığı karede sonucu görür.
 *
 * ROLLBACK (madde 20.3):
 * Yazma başarısız olursa önbellek yazma ÖNCESİ haline döner. Çakışma ise
 * ayrıca çakışma arayüzü açılır — sessiz kayıp YOK.
 *
 * BAŞARIDA INVALIDATION YAPILMAZ:
 * Çekirdek yazma sonrası index'i senkronize ediyor ve dosya izleyici hash'i
 * eşit bulup olay yayınlamıyor. Optimistic güncelleme zaten doğru durumu
 * gösterdiği için yeniden sorgulamak GEREKSİZ render üretirdi (madde 35.1).
 */

interface Vars {
  taskId: string;
  done: boolean;
  /** Çakışma arayüzünde gösterilecek sakin cümle. */
  label: string;
  /** Çakışan notun kimliği. */
  noteId: string;
}

/** Önbellekteki bir görevin durumunu yerinde değiştirir. */
function patchTask<T extends { id: string; status: string }>(tasks: T[], taskId: string, status: string): T[] {
  let changed = false;
  const next = tasks.map((task) => {
    if (task.id !== taskId) return task;
    changed = true;
    return { ...task, status };
  });
  return changed ? next : tasks;
}

export function useTaskMutation() {
  const queryClient = useQueryClient();
  const openConflict = useConflictStore((s) => s.open);

  return useMutation({
    mutationFn: ({ taskId, done }: Vars) => setTaskStatus(taskId, done),

    onMutate: async ({ taskId, done }) => {
      const closeInteraction = startInteraction();
      const status = done ? "done" : "open";

      // Uçuşta olan sorgular optimistic güncellemeyi ezmesin.
      await queryClient.cancelQueries({ queryKey: queryKeys.today.view });
      await queryClient.cancelQueries({ queryKey: queryKeys.dashboard.view });

      const previousToday = queryClient.getQueryData<TodayView>(queryKeys.today.view);
      const previousDashboard = queryClient.getQueryData<DashboardView>(
        queryKeys.dashboard.view,
      );

      if (previousToday) {
        queryClient.setQueryData<TodayView>(queryKeys.today.view, {
          ...previousToday,
          overdue: patchTask(previousToday.overdue, taskId, status),
          due: patchTask(previousToday.due, taskId, status),
        });
      }
      if (previousDashboard) {
        queryClient.setQueryData<DashboardView>(queryKeys.dashboard.view, {
          ...previousDashboard,
          criticalTasks: patchTask(previousDashboard.criticalTasks, taskId, status),
        });
      }

      // Görsel tepki bu karede gerçekleşti — ölçümü kapat (madde 34.1).
      closeInteraction();

      return { previousToday, previousDashboard };
    },

    onError: (error, vars, context) => {
      // Madde 20.3: yazılmadıysa arayüz de yazılmış görünmemeli.
      if (context?.previousToday) {
        queryClient.setQueryData(queryKeys.today.view, context.previousToday);
      }
      if (context?.previousDashboard) {
        queryClient.setQueryData(queryKeys.dashboard.view, context.previousDashboard);
      }

      if (error instanceof MutationError && error.isConflict) {
        openConflict({
          action: vars.label,
          noteId: vars.noteId,
          // Yeniden deneme: tazelenmiş guard gerektiği için önce sorguları
          // geçersiz kılar, sonra aynı mutasyonu tekrar çalıştırır.
          retry: () => {
            for (const key of VAULT_DEPENDENT_KEYS) {
              queryClient.invalidateQueries({ queryKey: key });
            }
          },
        });
      }
      // Çakışma dışı hatalar (yönetilmeyen not, hedef kayması) sakin bir
      // durumdur: rollback yeterli, akış kesilmez (madde 25.2).
    },
  });
}
