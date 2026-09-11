import type { Job } from "@/lib/generated";

interface OptimisticJobInput {
  id: string;
  action: string;
  workspace: string | null;
  input: string;
  createdAt: string;
}

/**
 * Hermes'e henüz teslim edilmemiş işi yalnız arayüz önbelleği için hazırlar.
 * Gerçek kayıt çekirdekten geldiğinde sorgu tazelenir ve bu kayıt kaybolur.
 */
export function createOptimisticJob(input: OptimisticJobInput): Job {
  return {
    id: input.id,
    action: input.action,
    actionLabel: input.action,
    workspace: input.workspace,
    summary: input.input.slice(0, 120),
    status: "queued",
    createdAt: input.createdAt,
    startedAt: null,
    finishedAt: null,
    // Madde 7: sahte ilerleme YOK.
    progress: null,
    errorCode: null,
    errorMessage: null,
    source: "arkeles",
  };
}

/**
 * Başarısız gönderimin yalnız kendi geçici kaydını kaldırır.
 * Bu sırada gelmiş gerçek veya başka optimistic işler korunur.
 */
export function removeOptimisticJob(
  jobs: Job[] | undefined,
  optimisticId: string,
): Job[] | undefined {
  if (!jobs) return jobs;
  return jobs.filter((job) => job.id !== optimisticId);
}
