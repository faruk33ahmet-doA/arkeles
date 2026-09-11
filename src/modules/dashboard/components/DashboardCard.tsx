import { cn } from "@/lib/cn";
import { Surface } from "@/ui/Surface";

interface DashboardCardProps extends React.HTMLAttributes<HTMLDivElement> {
  title: string;
  label?: string;
  action?: React.ReactNode;
  featured?: boolean;
  bodyClassName?: string;
}

/**
 * Dashboard'a özel sunum kabuğu. Veri veya davranış taşımaz; mevcut Surface
 * katmanlarını daha yoğun bir başlık ritmiyle kompoze eder.
 */
export function DashboardCard({
  title,
  label,
  action,
  featured = false,
  className,
  bodyClassName,
  children,
  ...props
}: DashboardCardProps) {
  return (
    <Surface
      level={featured ? 2 : 1}
      className={cn(
        "overflow-hidden border-border-default bg-dashboard-panel",
        featured && "rounded-lg bg-dashboard-raised",
        className,
      )}
      {...props}
    >
      <div className="flex items-start justify-between gap-4 border-b border-border-subtle px-4 py-3">
        <div className="min-w-0">
          {label ? <p className="mb-1 text-xs font-medium text-accent">{label}</p> : null}
          <h2 className="truncate text-base font-medium text-text-primary">{title}</h2>
        </div>
        {action}
      </div>
      <div className={cn("px-4 py-4", bodyClassName)}>{children}</div>
    </Surface>
  );
}

export const dashboardActionClass = cn(
  "shrink-0 rounded-sm border border-border-default px-2 py-1 text-xs font-medium",
  "text-text-secondary transition-colors duration-fast ease-out outline-none",
  "hover:border-dashboard-rule hover:bg-dashboard-muted hover:text-text-primary",
  "focus-visible:ring-1 focus-visible:ring-accent",
);
