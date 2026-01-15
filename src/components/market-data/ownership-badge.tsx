import { Badge } from "@/components/ui/badge";
import { OwnershipStatus } from "@/lib/types";

export interface OwnershipBadgeProps {
  status: OwnershipStatus | undefined;
  className?: string;
}

export function OwnershipBadge({ status, className }: OwnershipBadgeProps) {
  if (!status || status === OwnershipStatus.NONE) return null;

  return (
    <Badge
      variant={status === OwnershipStatus.CURRENTLY_OWNED ? "default" : "secondary"}
      className={className}
    >
      Owned
    </Badge>
  );
}
