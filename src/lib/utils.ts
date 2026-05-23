import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

/**
 * Tailwind 클래스 병합 유틸리티 (shadcn/ui 표준).
 *
 * `clsx` 로 조건부 클래스 결합 후 `tailwind-merge` 로 충돌 클래스 정리 — 예: `cn("p-2", "p-4")` → `"p-4"`.
 * shadcn 컴포넌트 (`alert-dialog.tsx` 등) 가 `@/lib/utils` 의 `cn` import.
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
