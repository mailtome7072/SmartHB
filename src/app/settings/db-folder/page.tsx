'use client'

/**
 * DB 폴더 변경 (Sprint 16 T3, ADR-009, PI-16).
 *
 * 데이터가 저장되는 클라우드 동기화 폴더를 재지정한다. copy-then-switch:
 * 새 폴더로 기존 데이터(DB·salt·assets·output·backup)를 복사·검증한 뒤 config 경로를 갱신하고
 * 앱을 재시작한다. 원본은 보존(MOVED_TO 마커). 실패 시 기존 폴더 유지(무손상).
 *
 * window.confirm 이 Tauri WebView 에서 차단되므로 실행 확인은 자체 모달을 쓴다.
 */

import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SettingsHomeLink } from '@/components/settings/SettingsHomeLink'
import { changeDataFolder, getSetupStatus, relaunchApp, selectFolder } from '@/lib/tauri'

// 개발 빌드(`next dev`)에서는 화면을 localhost dev 서버에서 로드하므로 relaunch() 시 dev 서버가
// 함께 종료돼 재시작된 앱이 화면을 못 띄운다. 개발 모드에서는 자동 재시작 대신 수동 안내만 한다.
// 프로덕션 빌드는 번들 정적 파일을 로드하므로 relaunch() 가 정상 동작한다.
const IS_DEV = process.env.NODE_ENV !== 'production'

export default function DbFolderPage() {
  const [currentFolder, setCurrentFolder] = useState<string | null>(null)
  const [target, setTarget] = useState<string | null>(null)
  const [confirming, setConfirming] = useState(false)
  const [running, setRunning] = useState(false)
  const [done, setDone] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    getSetupStatus()
      .then((s) => setCurrentFolder(s.cloud_folder_path || null))
      .catch(() => setCurrentFolder(null))
  }, [])

  const handlePick = async () => {
    setError(null)
    const path = await selectFolder()
    if (path === null) return
    setTarget(path)
  }

  const handleRun = async () => {
    if (target === null) return
    setConfirming(false)
    setRunning(true)
    setError(null)
    try {
      await changeDataFolder(target)
      setDone(true)
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : 'DB 폴더 변경에 실패했습니다.')
    } finally {
      setRunning(false)
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      {/* 사용자 요청 — 전체 행간 1.25(leading-tight)로 통일. */}
      <div className="mx-auto max-w-2xl leading-tight">
        <SettingsHomeLink />
        <h1 className="mb-2 text-2xl font-bold">DB 폴더 변경</h1>
        <p className="mb-6 text-base text-gray-600">
          데이터가 저장되는 클라우드 동기화 폴더(DB 위치)를 재지정합니다. 기존 데이터를 새 폴더로
          복사·검증한 뒤 앱을 재시작합니다.
        </p>

        {/* 현재 폴더 */}
        <section className="mb-4 rounded-lg border border-[var(--border)] bg-white p-5">
          <h2 className="mb-1 text-base font-bold">현재 폴더</h2>
          <p className="break-all text-sm text-gray-700">
            {currentFolder ?? '설정되지 않음 (개발 모드 또는 마법사 미완료)'}
          </p>
        </section>

        {/* 주의 안내 */}
        <section className="mb-4 rounded-lg border border-amber-300 bg-amber-50 p-4 text-sm text-amber-900">
          <p className="mb-2 font-bold">변경 전 확인해 주세요</p>
          <ul className="list-disc space-y-1 pl-5">
            <li>기존 폴더의 데이터는 <b>그대로 보존</b>되며, 새 폴더로 <b>복사</b>됩니다. (원본 삭제 안 함)</li>
            <li>대상 폴더에 이미 SmartHB 데이터(app.db)가 있으면 변경이 <b>차단</b>됩니다.</li>
            <li>
              <b>다른 PC</b>에서도 이 앱을 쓴다면, 그 PC에서도 <b>같은 새 폴더</b>로 다시
              지정해야 합니다. (폴더 설정은 PC마다 따로 저장됩니다)
            </li>
            <li>변경이 끝나면 앱이 <b>자동으로 재시작</b>됩니다.</li>
          </ul>
        </section>

        {/* 폴더 선택 + 실행 */}
        <section className="mb-4 rounded-lg border border-[var(--border)] bg-white p-5">
          <h2 className="mb-3 text-base font-bold">새 폴더 선택</h2>
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={handlePick}
              disabled={running || done}
              className="h-11 rounded-md border border-[var(--border)] px-5 text-base font-medium hover:bg-gray-50 disabled:opacity-50"
            >
              폴더 선택
            </button>
            <span className="min-w-0 flex-1 break-all text-sm text-gray-700">
              {target ?? '선택된 폴더가 없습니다.'}
            </span>
          </div>

          <button
            type="button"
            onClick={() => setConfirming(true)}
            disabled={target === null || running || done}
            className="mt-4 h-12 w-full rounded-md bg-[var(--accent)] px-6 text-base font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
          >
            {running ? '데이터를 복사하는 중입니다...' : 'DB 폴더 변경'}
          </button>
        </section>

        {error !== null && (
          <p className="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-[var(--danger)]">
            {error}
          </p>
        )}

        {/* 실행 확인 모달 */}
        {confirming && target !== null && (
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
            <div className="w-full max-w-md rounded-lg border border-[var(--border)] bg-white p-6 shadow-xl">
              <h3 className="mb-2 text-lg font-bold">DB 폴더를 변경할까요?</h3>
              <p className="mb-4 break-all text-sm text-gray-700">
                새 폴더: <b>{target}</b>
                <br />
                기존 데이터를 새 폴더로 복사·검증한 뒤 앱이 재시작됩니다.
              </p>
              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setConfirming(false)}
                  className="h-11 rounded-md border border-[var(--border)] px-5 text-base hover:bg-gray-50"
                >
                  취소
                </button>
                <button
                  type="button"
                  onClick={handleRun}
                  className="h-11 rounded-md bg-[var(--accent)] px-5 text-base font-bold text-white hover:bg-[var(--accent-hover)]"
                >
                  변경 실행
                </button>
              </div>
            </div>
          </div>
        )}

        {/* 완료 모달 — 재시작 안내 */}
        {done && (
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
            <div className="w-full max-w-md rounded-lg border border-[var(--border)] bg-white p-6 shadow-xl">
              <h3 className="mb-2 text-lg font-bold">DB 폴더 변경 완료</h3>
              {IS_DEV ? (
                <>
                  <p className="mb-4 text-sm text-gray-700">
                    데이터를 새 폴더로 옮겼습니다. <b>개발 모드</b>에서는 자동 재시작이 동작하지
                    않습니다. 앱을 직접 종료한 뒤 다시 실행해 주세요.
                  </p>
                  <button
                    type="button"
                    onClick={() => setDone(false)}
                    className="h-12 w-full rounded-md bg-[var(--accent)] px-6 text-base font-bold text-white hover:bg-[var(--accent-hover)]"
                  >
                    확인
                  </button>
                </>
              ) : (
                <>
                  <p className="mb-4 text-sm text-gray-700">
                    데이터를 새 폴더로 옮겼습니다. 변경을 적용하려면 앱을 재시작해야 합니다.
                  </p>
                  <button
                    type="button"
                    onClick={() => void relaunchApp()}
                    className="h-12 w-full rounded-md bg-[var(--accent)] px-6 text-base font-bold text-white hover:bg-[var(--accent-hover)]"
                  >
                    지금 재시작
                  </button>
                </>
              )}
            </div>
          </div>
        )}
      </div>
    </AppShell>
  )
}
