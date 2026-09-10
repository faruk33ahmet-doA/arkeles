import { QueryClientProvider } from "@tanstack/react-query";
import { TooltipProvider } from "@/ui/primitives/tooltip";
import { queryClient } from "@/data/queryClient";

/*
 * Providers — tek yer. Anayasa madde 35.2: Context yalnız SOĞUK bağımlılıklar
 * için kullanılır (queryClient, tooltip yapılandırması). Sıcak veri Context'e
 * KONMAZ; o Zustand selector'ları ve TanStack Query cache'i üzerinden akar.
 */

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider delayDuration={400}>{children}</TooltipProvider>
    </QueryClientProvider>
  );
}
