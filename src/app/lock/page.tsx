import { LockScreen } from '@/components/LockScreen'

/**
 * `/lock` 라우트 — 잠금 화면.
 *
 * T9 (시작 시퀀스 통합) 시점에 앱 진입 흐름이 추가된다:
 * - 앱 시작 → `check_auth_status` IPC → `not-initialized` | `locked` 시 본 라우트로 자동 이동
 * - 인증 성공 시 메인 화면(`/`) 라우팅
 *
 * 현재 sprint 1 T4 범위에서는 직접 `/lock` URL 로 진입하여 UI 흐름만 검증.
 */
export default function LockPage() {
  return <LockScreen />
}
