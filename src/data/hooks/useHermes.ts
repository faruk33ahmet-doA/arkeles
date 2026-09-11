import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import {
  cancelJob,
  getActiveJobs,
  getHermesActions,
  getHermesSummary,
  getJobHistory,
  retryJob,
  submitJob,
} from "@/services/hermes.service";
import { isTauri } from "@/services/ipc";
import { queryKeys, VAULT_DEPENDENT_KEYS } from "@/data/queryKeys";
import { startInteraction } from "@/lib/perf";
import type { Job } from "@/lib/generated";

/*
 * Hermes veri kancaları — Sprint 4.
 *
 * TAZELEME OLAYLA GELİR, YOKLAMAYLA DEĞİL (madde 35.1, Sprint 4 madde 10:
 * "polling fırtınası oluşturma"). Çekirdekteki iş sürücüsü bir şey
 * değiştirdiğinde `hermes:jobs-changed` yayınlar; burada o olay query
 * invalidation'ına çevrilir.
 */

/** Sprint 4 madde 4, 13 — dashboard özeti. */
export function useHermesSummary() {
  return useQuery({
    queryKey: queryKeys.hermes.summary,
    queryFn: getHermesSummary,
    retry: false,
    // Hermes kapalıyken bile sayımlar yerel defterden gelir; 30 sn taze sayılır.
    staleTime: 30_000,
  });
}

/** Madde 18.2 — Hermes bildirmiyorsa boş dizi, arayüz hiçbir şey göstermez. */
export function useHermesActions() {
  return useQuery({
    queryKey: queryKeys.hermes.actions,
    queryFn: getHermesActions,
    retry: false,
    staleTime: 60_000,
  });
}

export function useActiveJobs() {
  return useQuery({
    queryKey: queryKeys.hermes.activeJobs,
    queryFn: getActiveJobs,
    retry: false,
  });
}

export function useJobHistory(
  since: string,
  workspace: string | null,
  status: string | null,
) {
  return useQuery({
    queryKey: queryKeys.hermes.history(since, workspace, status),
    queryFn: () => getJobHistory(since, workspace, status),
    retry: false,
  });
}

/*
 * İş kuyruğu değişim aboneliği — Sprint 4 madde 10.
 *
 * "Job tamamlandıktan sonra sonuç referansı henüz index'te değilse:
 *  hata gösterme, kısa süre stale durumunu kabul et, kontrollü
 *  revalidation yap."
 *
 * Bu yüzden iş değişiminde vault sorguları da GECİKMELİ tazelenir:
 * Hermes dosyayı yazdı → job completed → dosya izleyici 200 ms sonra
 * index'i günceller. Hemen sorgulamak sonucu "yok" gösterirdi.
 */
export function useJobsChanged(): void {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!isTauri()) return;

    let unlisten: (() => void) | undefined;
    let cancelled = false;
    const timers: ReturnType<typeof setTimeout>[] = [];

    listen("hermes:jobs-changed", () => {
      queryClient.invalidateQueries({ queryKey: ["hermes"] });

      /*
       * Vault tarafı için TEK gecikmeli tazeleme. Yoklama değil: bir kez,
       * dosya izleyicinin debounce penceresinden (200 ms) sonra.
       */
      timers.push(
        setTimeout(() => {
          for (const key of VAULT_DEPENDENT_KEYS) {
            queryClient.invalidateQueries({ queryKey: key });
          }
        }, 400),
      );
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });

    return () => {
      cancelled = true;
      unlisten?.();
      for (const timer of timers) clearTimeout(timer);
    };
  }, [queryClient]);
}

/*
 * İş gönderme — Sprint 4 madde 6.
 *
 * "Gönderildiğinde: uzun loading modal yok, ekran bloke edilmez, iş Job
 *  Queue'ya düşer, kullanıcı çalışmaya devam edebilir. Yeni job ilk karede
 *  UI'da görünür. Hermes cevabı beklenmez."
 *
 * `onMutate` işi önbelleğe ANINDA ekler; çekirdek zaten queued olarak
 * kaydedip hemen dönüyor, Hermes'e iletim arka planda.
 */
export function useSubmitJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (vars: { action: string; workspace: string | null; input: string }) =>
      submitJob(vars.action, vars.workspace, vars.input),

    onMutate: async (vars) => {
      const close = startInteraction();
      await queryClient.cancelQueries({ queryKey: queryKeys.hermes.activeJobs });

      const previous = queryClient.getQueryData<Job[]>(queryKeys.hermes.activeJobs);

      // Geçici kayıt: gerçek kimlik çekirdekten gelene kadar.
      const optimistic: Job = {
        id: `pending-${Date.now()}`,
        action: vars.action,
        actionLabel: vars.action,
        workspace: vars.workspace,
        summary: vars.input.slice(0, 120),
        status: "queued",
        createdAt: new Date().toISOString(),
        startedAt: null,
        finishedAt: null,
        // Madde 7: sahte progress YOK.
        progress: null,
        errorCode: null,
        errorMessage: null,
        source: "arkeles",
      };

      queryClient.setQueryData<Job[]>(queryKeys.hermes.activeJobs, [
        optimistic,
        ...(previous ?? []),
      ]);

      close();
      return { previous };
    },

    onError: (_error, _vars, context) => {
      if (context?.previous) {
        queryClient.setQueryData(queryKeys.hermes.activeJobs, context.previous);
      }
    },

    // Gerçek kayıt geldiğinde geçici olanı değiştir.
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["hermes"] });
    },
  });
}

export function useRetryJob() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (jobId: string) => retryJob(jobId),
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["hermes"] }),
  });
}

export function useCancelJob() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (jobId: string) => cancelJob(jobId),
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["hermes"] }),
  });
}
