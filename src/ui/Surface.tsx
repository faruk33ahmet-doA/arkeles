import { cn } from "@/lib/cn";

/*
 * Surface — görsel temel. Anayasa madde 28.2:
 * "Derinlik, gölge ve katman ile kurulur; hareket ile değil."
 *
 * Uygulamadaki HER yükseltilmiş yüzey bu bileşenden geçer. Böylece
 * elevation üç seviyede kalır (madde 29, 32) ve tek yerden yönetilir.
 */

type SurfaceLevel = 1 | 2 | 3;

interface SurfaceProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Yüzey basamağı. Anayasa madde 29: yalnız 3 seviye. */
  level?: SurfaceLevel;
  /** Cam efekti — YALNIZ küçük yüzeylerde (madde 30.1). Asla animate edilmez (30.2). */
  glass?: boolean;
}

const surfaceByLevel: Record<SurfaceLevel, string> = {
  1: "bg-surface-2 shadow-1",
  2: "bg-surface-2 shadow-2",
  3: "bg-surface-3 shadow-3",
};

export function Surface({
  level = 1,
  glass = false,
  className,
  ...props
}: SurfaceProps) {
  return (
    <div
      className={cn(
        "rounded-md border border-border-subtle",
        glass
          ? "border-glass-border bg-glass-bg backdrop-blur-glass"
          : surfaceByLevel[level],
        className,
      )}
      {...props}
    />
  );
}
