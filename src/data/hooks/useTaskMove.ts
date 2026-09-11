import { useCallback } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { MutationError, moveTask } from "@/services/mutation.service";
import { queryKeys, VAULT_DEPENDENT_KEYS } from "@/data/queryKeys";
import { useConflictStore } from "@/state/conflictStore";
import { useWorkStore } from "@/state/workStore";
import { startInteraction } from "@/lib/perf";
import { neighborTaskId, reorderTasks, type MoveDirection } from "@/lib/taskOrder";
import type { NoteDetail } from "@/lib/generated";

/*
 * Görev sırası mutasyonu — Sprint 5 madde 8 (Sprint 2 `move_line` altyapısı).
 *
 * Aynı optimistic + rollback + çakışma deseni (madde 20.3, 34.1).
 *
 * KİMLİK KAYMASI: görev kimliği `not#satır` biçimindedir. Taşınan görev
 * hedefin satırına geçtiği için YENİ kimliği hedefin ESKİ kimliğidir. Seçim
 * bu yüzden tazelemeden SONRA hedef kimliğe aktarılır — kimlik arayüzde
 * ayrıştırılmaz. Diğer görevlerin kimlikleri de kaydığı için Bugün ve
 * Dashboard önbellekleri de tazelenir.
 */

interface MoveVars {
  noteId: string;
  taskId: string;
  targetTaskId: string;
  /** Çakışma arayüzünde gösterilecek sakin cümle. */
  label: string;
}

export function useTaskMove() {
  const queryClient = useQueryClient();
  const openConflict = useConflictStore((s) => s.open);
  const selectTask = useWorkStore((s) => s.selectTask);

  return useMutation({
    mutationFn: ({ taskId, targetTaskId }: MoveVars) => moveTask(taskId, targetTaskId),

    onMutate: async ({ noteId, taskId, targetTaskId }) => {
      const close = startInteraction();
      const key = queryKeys.work.noteDetail(noteId);
      await queryClient.cancelQueries({ queryKey: key });

      const previous = queryClient.getQueryData<NoteDetail>(key);
      if (previous) {
        queryClient.setQueryData<NoteDetail>(key, {
          ...previous,
          tasks: reorderTasks(previous.tasks, taskId, targetTaskId),
        });
      }

      close();
      return { previous, key };
    },

    onError: (error, vars, context) => {
      if (context?.previous) {
        queryClient.setQueryData(context.key, context.previous);
      }
      if (error instanceof MutationError && error.isConflict) {
        openConflict({
          action: vars.label,
          noteId: vars.noteId,
          retry: () => queryClient.invalidateQueries({ queryKey: ["work"] }),
        });
      }
    },

    onSuccess: async (_result, vars) => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.work.noteDetail(vars.noteId),
      });
      selectTask(vars.targetTaskId);
      for (const key of VAULT_DEPENDENT_KEYS) {
        if (key[0] !== "work") queryClient.invalidateQueries({ queryKey: key });
      }
    },
  });
}

/**
 * Klavye ve komut paleti yolu: seçili görevi komşusunun yerine taşır.
 * Önceki taşıma bitmeden yenisi başlamaz — kimlikler henüz tazelenmedi.
 */
export function useSelectedTaskMove() {
  const queryClient = useQueryClient();
  const move = useTaskMove();
  const noteId = useWorkStore((s) => s.noteId);
  const selectedTaskId = useWorkStore((s) => s.selectedTaskId);
  const { isPending, mutate } = move;

  const moveSelected = useCallback(
    (direction: MoveDirection) => {
      if (isPending || !noteId || !selectedTaskId) return;
      const detail = queryClient.getQueryData<NoteDetail>(queryKeys.work.noteDetail(noteId));
      const task = detail?.tasks.find((t) => t.id === selectedTaskId);
      if (!detail || !task?.managed) return;

      const targetTaskId = neighborTaskId(detail.tasks, selectedTaskId, direction);
      if (!targetTaskId) return;

      mutate({
        noteId,
        taskId: selectedTaskId,
        targetTaskId,
        label: `"${task.title}" taşınıyordu.`,
      });
    },
    [isPending, mutate, noteId, selectedTaskId, queryClient],
  );

  return { moveSelected, move };
}
