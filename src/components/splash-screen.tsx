/**
 * 첫 진입 + 라우팅 전환 시 표시되는 스플래시 화면.
 *
 * dev 모드에서는 Next.js dev server 가 페이지별로 on-demand 컴파일하여 첫 진입 시 빈 화면
 * 시간이 길어진다 (수십 초). 컴파일 후 React 가 마운트되면 본 컴포넌트가 표시되어 사용자가
 * 앱이 멈추지 않았음을 시각적으로 확인할 수 있게 한다. production 빌드에서는 첫 진입이
 * < 1초라 잠깐만 보인다.
 *
 * - 'use client' 불요 — 순수 표시용 컴포넌트
 * - 50대 친화: 큰 글자(헤더 32px, 본문 18px), 차분한 색조 (PRD §5.7)
 */

export function SplashScreen({ message }: { message?: string }) {
  return (
    <main
      role="status"
      aria-live="polite"
      className="flex min-h-screen flex-col items-center justify-center gap-6 bg-[var(--background)] px-4"
    >
      <h1 className="text-4xl font-bold text-[var(--foreground)]">스마트해법수학</h1>
      <p className="text-base text-gray-600">서현효자점 관리 앱</p>
      <Spinner />
      <p className="text-base text-gray-700">{message ?? '시작하는 중입니다...'}</p>
      <p className="mt-2 max-w-sm text-center text-sm text-gray-500">
        최초 실행은 잠시 시간이 걸릴 수 있습니다.
      </p>
    </main>
  )
}

function Spinner() {
  return (
    <div
      aria-hidden="true"
      className="h-12 w-12 animate-spin rounded-full border-4 border-[var(--border)] border-t-[var(--accent)]"
    />
  )
}
