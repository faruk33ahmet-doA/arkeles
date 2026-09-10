import { cn } from "@/lib/cn";
import { Surface } from "./Surface";

/*
 * Card — Anayasa madde 28: "büyük nefes alan kartlar", "temiz boşluklar".
 *
 * İç boşluk (padding) burada sabittir ve çağıran tarafından ezilmez.
 * Amaç: her kart aynı nefesi alsın, ekranlar arası tutarlılık bozulmasın.
 */

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Kart başlığı — sessiz, küçük, ikincil (madde 5). */
  title?: string;
  /** Başlık satırının sağına yerleşen içerik (sayı, durum noktası). */
  action?: React.ReactNode;
}

export function Card({ title, action, className, children, ...props }: CardProps) {
  return (
    <Surface level={1} className={cn("p-6", className)} {...props}>
      {title || action ? (
        <div className="mb-4 flex items-baseline justify-between gap-4">
          {title ? (
            <h2 className="text-xs font-medium uppercase tracking-[0.08em] text-text-tertiary">
              {title}
            </h2>
          ) : (
            <span />
          )}
          {action}
        </div>
      ) : null}
      {children}
    </Surface>
  );
}
