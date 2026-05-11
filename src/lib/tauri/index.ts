/**
 * Tauri IPC 추상화 레이어
 * 컴포넌트에서 invoke() 직접 호출 금지 — 이 파일을 통해서만 Tauri 커맨드 호출
 */

let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null

async function getInvoke() {
  if (typeof window === 'undefined') return null
  if (!invoke) {
    try {
      const tauri = await import('@tauri-apps/api/core')
      invoke = tauri.invoke
    } catch {
      // 브라우저 환경 (Tauri 없이 실행 시) — 개발용 mock 가능
      invoke = null
    }
  }
  return invoke
}

export async function greet(name: string): Promise<string> {
  const inv = await getInvoke()
  if (!inv) return `[개발 모드] 안녕하세요, ${name}!`
  return inv('greet', { name }) as Promise<string>
}
