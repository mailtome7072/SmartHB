'use client'

/**
 * 초기 설정 마법사 — `/setup` 라우트 (Sprint 3 T9, PRD §4.0).
 *
 * 4 단계 흐름:
 * 1. 환영 + 안내
 * 2. 클라우드 동기화 폴더 선택 (`selectFolder` → `saveCloudFolder`)
 * 3. 비밀번호 설정 + DB 초기화 (`LockScreen` 재사용 — `not-initialized` 모드)
 * 4. 완료 (`completeSetup`) → 설정(`/settings`) 으로 이동 (Sprint 5 T2 — 운영 시간/코드 테이블
 *    확인을 먼저 진행하도록 UX 흐름 변경)
 *
 * 나머지 단계(운영시간/학교코드/표준교습비/결제수단/백업폴더/가져오기/샘플등록)는 각
 * 도메인 sprint 에서 점진 추가 — 본 Phase 1 마법사는 핵심 4단계만 구현.
 *
 * **chicken-and-egg 회피**: 폴더 경로는 OS `app_config_dir/config.json` 에 보관되므로
 * DB 가 열리기 전에도 안전하게 저장된다.
 */

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { completeSetup, saveCloudFolder, selectFolder } from '@/lib/tauri'
import { LockScreen } from '@/components/LockScreen'
import { useSessionStore } from '@/stores/session-store'
import type { StartupResult } from '@/types'

type Step = 1 | 2 | 3 | 4

export default function SetupPage() {
  const router = useRouter()
  const markUnlocked = useSessionStore((s) => s.markUnlocked)
  const [step, setStep] = useState<Step>(1)
  const [folderPath, setFolderPath] = useState<string>('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSelectFolder = async () => {
    setError(null)
    setBusy(true)
    try {
      const selected = await selectFolder()
      if (selected === null) {
        setBusy(false)
        return
      }
      await saveCloudFolder(selected)
      setFolderPath(selected)
      setStep(3)
    } catch (e) {
      setError(typeof e === 'string' ? e : '폴더를 저장할 수 없습니다.')
    } finally {
      setBusy(false)
    }
  }

  const handleUnlocked = (result: StartupResult) => {
    markUnlocked(result)
    setStep(4)
  }

  const handleComplete = async () => {
    setError(null)
    setBusy(true)
    try {
      await completeSetup()
      router.replace('/settings')
    } catch (e) {
      setError(typeof e === 'string' ? e : '마법사 완료를 저장할 수 없습니다.')
      setBusy(false)
    }
  }

  if (step === 3) {
    return <LockScreen onUnlocked={handleUnlocked} />
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-[var(--background)] px-4">
      <div className="w-full max-w-xl rounded-lg border border-[var(--border)] bg-white p-8 shadow-sm">
        <ProgressBar step={step} />

        {step === 1 && (
          <section>
            <h1 className="mb-3 text-3xl font-bold">환영합니다</h1>
            <p className="mb-6 text-base text-gray-700">
              스마트해법수학 서현효자점 관리 앱입니다. 처음 사용을 위해 몇 가지 설정을
              진행합니다.
            </p>
            <ul className="mb-8 list-disc space-y-1 pl-5 text-base text-gray-700">
              <li>클라우드 동기화 폴더 선택</li>
              <li>앱 잠금 비밀번호 설정</li>
            </ul>
            <button
              type="button"
              onClick={() => setStep(2)}
              className="h-12 w-full rounded-md bg-[var(--accent)] px-4 text-base font-bold text-white hover:bg-[var(--accent-hover)]"
            >
              시작하기
            </button>
          </section>
        )}

        {step === 2 && (
          <section>
            <h1 className="mb-3 text-3xl font-bold">클라우드 폴더 선택</h1>
            <p className="mb-6 text-base text-gray-700">
              양 PC(교습소·자택) 간 데이터 공유에 사용할 클라우드 동기화 폴더를 선택합니다.
              네이버 MYBOX, iCloud Drive, Dropbox 등 OS 클라이언트가 동기화하는 폴더를
              지정해주세요.
            </p>
            {folderPath !== '' && (
              <p className="mb-4 break-all rounded border border-[var(--border)] bg-[var(--background)] px-3 py-2 text-sm">
                선택됨: {folderPath}
              </p>
            )}
            {error !== null && (
              <p role="alert" className="mb-4 text-sm text-[var(--danger)]">
                {error}
              </p>
            )}
            <button
              type="button"
              onClick={handleSelectFolder}
              disabled={busy}
              className="h-12 w-full rounded-md bg-[var(--accent)] px-4 text-base font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
            >
              {busy ? '저장 중...' : '폴더 선택'}
            </button>
          </section>
        )}

        {step === 4 && (
          <section>
            <h1 className="mb-3 text-3xl font-bold">설정 완료</h1>
            <p className="mb-3 text-base text-gray-700">
              초기 설정이 완료되었습니다. 메인 화면에서 원생 등록부터 시작해주세요.
            </p>
            {folderPath !== '' && (
              <p className="mb-6 break-all rounded border border-[var(--border)] bg-[var(--background)] px-3 py-2 text-sm">
                동기화 폴더: {folderPath}
              </p>
            )}
            {error !== null && (
              <p role="alert" className="mb-4 text-sm text-[var(--danger)]">
                {error}
              </p>
            )}
            <button
              type="button"
              onClick={handleComplete}
              disabled={busy}
              className="h-12 w-full rounded-md bg-[var(--accent)] px-4 text-base font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
            >
              {busy ? '저장 중...' : '시작하기'}
            </button>
          </section>
        )}
      </div>
    </main>
  )
}

function ProgressBar({ step }: { step: Step }) {
  const labels = ['환영', '폴더', '비밀번호', '완료']
  return (
    <ol className="mb-6 flex justify-between text-sm">
      {labels.map((label, idx) => {
        const idxStep = (idx + 1) as Step
        const active = step === idxStep
        const done = step > idxStep
        return (
          <li
            key={label}
            className={`flex-1 text-center ${
              active ? 'font-bold text-[var(--accent)]' : done ? 'text-gray-600' : 'text-gray-400'
            }`}
          >
            <span aria-hidden="true">{done ? '✓' : idx + 1}</span> {label}
          </li>
        )
      })}
    </ol>
  )
}
