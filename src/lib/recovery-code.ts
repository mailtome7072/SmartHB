/**
 * 복구 코드 표시/정규화 유틸 — T5 PI-07.
 *
 * 백엔드 `recovery::normalize_input_code` 와 정확히 같은 정규화 규칙(공백·하이픈 제거 +
 * 대문자 통일)을 프론트엔드에 미러링한다. 검증은 백엔드가 권위 — 본 유틸은 사용자
 * 입력 길이 표시 등 UI 보조 용도다.
 */

/** `XXXXXXXXXXXX` → `XXXX-XXXX-XXXX` 형식으로 4자씩 분리하여 가독성 향상. */
export function formatRecoveryCode(raw: string): string {
  const normalized = normalizeRecoveryCode(raw)
  return normalized.match(/.{1,4}/g)?.join('-') ?? normalized
}

/** 공백·하이픈 제거 + 대문자 통일. 백엔드 정규화와 동일 규칙. */
export function normalizeRecoveryCode(raw: string): string {
  return raw.replace(/[-\s]/g, '').toUpperCase()
}
