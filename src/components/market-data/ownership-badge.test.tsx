import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { OwnershipBadge } from "./ownership-badge";
import { OwnershipStatus } from "@/lib/types";

describe("OwnershipBadge", () => {
  describe("rendering behavior", () => {
    it("should render 'Owned' badge for CURRENTLY_OWNED status with default variant", () => {
      render(<OwnershipBadge status={OwnershipStatus.CURRENTLY_OWNED} />);
      const badge = screen.getByText("Owned");
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveClass("bg-primary");
    });

    it("should render 'Owned' badge for PREVIOUSLY_OWNED status with secondary variant", () => {
      render(<OwnershipBadge status={OwnershipStatus.PREVIOUSLY_OWNED} />);
      const badge = screen.getByText("Owned");
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveClass("bg-secondary");
    });

    it("should return null for NONE status", () => {
      const { container } = render(
        <OwnershipBadge status={OwnershipStatus.NONE} />
      );
      expect(container.firstChild).toBe(null);
    });

    it("should return null for undefined status", () => {
      const { container } = render(<OwnershipBadge status={undefined} />);
      expect(container.firstChild).toBe(null);
    });

    it("should apply custom className when provided", () => {
      render(
        <OwnershipBadge
          status={OwnershipStatus.CURRENTLY_OWNED}
          className="ml-2 text-xs"
        />
      );
      const badge = screen.getByText("Owned");
      expect(badge).toHaveClass("ml-2");
      expect(badge).toHaveClass("text-xs");
    });
  });
});
