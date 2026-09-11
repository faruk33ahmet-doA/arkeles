import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  MutationError,
  setFrontmatterField,
  toggleTag,
} from "@/services/mutation.service";
import { queryKeys } from "@/data/queryKeys";
import { useConflictStore } from "@/state/conflictStore";
import { startInteraction } from "@/lib/perf";
import type { NoteDetail } from "@/lib/generated";

/*
 * Not mekanik mutasyonları — Sprint 2 motorlarının İLK UI bağlantısı.
 *
 * Sprint 2'de `set_frontmatter_field` ve `toggle_tag` yazıldı ve test
 * edildi ama bağlanacak bir yüzey yoktu. Not detayı o yüzey.
 *
 * Optimistic UI + çakışma rollback aynı desen (madde 20.3, 34.1).
 */

interface TagVars {
  noteId: string;
  tag: string;
  add: boolean;
}

interface FieldVars {
  noteId: string;
  key: string;
  value: string | number | boolean | string[] | null;
  /** Çakışma arayüzünde gösterilecek sakin cümle. */
  label: string;
}

/** Önbellekteki frontmatter JSON'ını yerinde günceller. */
function patchFrontmatter(
  detail: NoteDetail,
  mutate: (fm: Record<string, unknown>) => void,
): NoteDetail {
  let parsed: Record<string, unknown>;
  try {
    parsed = JSON.parse(detail.frontmatter) as Record<string, unknown>;
  } catch {
    // Bozuk frontmatter optimistic güncellemeyi engellemez; yazma
    // çekirdekte satır bazlı yapılıyor (Sprint 2 kararı).
    parsed = {};
  }
  mutate(parsed);
  return { ...detail, frontmatter: JSON.stringify(parsed) };
}

export function useTagMutation() {
  const queryClient = useQueryClient();
  const openConflict = useConflictStore((s) => s.open);

  return useMutation({
    mutationFn: ({ noteId, tag, add }: TagVars) => toggleTag(noteId, tag, add),

    onMutate: async ({ noteId, tag, add }) => {
      const close = startInteraction();
      const key = queryKeys.work.noteDetail(noteId);
      await queryClient.cancelQueries({ queryKey: key });

      const previous = queryClient.getQueryData<NoteDetail>(key);
      if (previous) {
        queryClient.setQueryData<NoteDetail>(
          key,
          patchFrontmatter(previous, (fm) => {
            const raw = fm.tags;
            const tags = Array.isArray(raw)
              ? (raw as string[])
              : typeof raw === "string"
                ? [raw]
                : [];
            const next = add
              ? tags.includes(tag)
                ? tags
                : [...tags, tag]
              : tags.filter((t) => t !== tag);
            if (next.length === 0) delete fm.tags;
            else fm.tags = next;
          }),
        );
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
          action: vars.add
            ? `"${vars.tag}" etiketi ekleniyordu.`
            : `"${vars.tag}" etiketi kaldırılıyordu.`,
          noteId: vars.noteId,
          retry: () => queryClient.invalidateQueries({ queryKey: ["work"] }),
        });
      }
    },

    // Yazma sonrası çekirdek index'i senkronize ediyor; hash eşleştiği için
    // izleyici olay yayınlamıyor (Sprint 2). Ama frontmatter'ın DİSKTEKİ
    // kanonik hali (tırnaklama, sıralama) optimistic tahminden farklı
    // olabilir — bu yüzden yalnız BU notu tazeliyoruz.
    onSuccess: (_result, vars) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.work.noteDetail(vars.noteId),
      });
    },
  });
}

export function useFieldMutation() {
  const queryClient = useQueryClient();
  const openConflict = useConflictStore((s) => s.open);

  return useMutation({
    mutationFn: ({ noteId, key, value }: FieldVars) =>
      setFrontmatterField(noteId, key, value),

    onMutate: async ({ noteId, key, value }) => {
      const close = startInteraction();
      const queryKey = queryKeys.work.noteDetail(noteId);
      await queryClient.cancelQueries({ queryKey });

      const previous = queryClient.getQueryData<NoteDetail>(queryKey);
      if (previous) {
        queryClient.setQueryData<NoteDetail>(
          queryKey,
          patchFrontmatter(previous, (fm) => {
            if (value === null) delete fm[key];
            else fm[key] = value;
          }),
        );
      }

      close();
      return { previous, queryKey };
    },

    onError: (error, vars, context) => {
      if (context?.previous) {
        queryClient.setQueryData(context.queryKey, context.previous);
      }
      if (error instanceof MutationError && error.isConflict) {
        openConflict({
          action: vars.label,
          noteId: vars.noteId,
          retry: () => queryClient.invalidateQueries({ queryKey: ["work"] }),
        });
      }
    },

    /*
     * Frontmatter değişimi notun BAŞLIĞINI da değiştirebilir (`title` alanı)
     * ve o başlık liste panellerinde görünür. Bu yüzden tüm iş modülü
     * tazelenir — tek not değil.
     */
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["work"] });
    },
  });
}
