'use client'

/**
 * 교습소 정보 편집 (Sprint 15 T1, PRD §4.12).
 *
 * 텍스트 필드(교습소명/대표자/연락처/주소/사업자등록번호/최대인원/면적) + 로고·2D바코드
 * 이미지. `app_settings.academy_info` JSON 저장 — DB 마이그레이션 불필요. 이미지는
 * `assets/academy_{logo,barcode}.{ext}` 파일로 저장(notice_asset IPC 재사용)하고 JSON 에는
 * 파일명만 보관. 공지문(§4.10)에는 연동하지 않는 교습소 정보 화면 전용 데이터.
 */

import { useEffect, useRef, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SettingsHomeLink } from '@/components/settings/SettingsHomeLink'
import { SplashScreen } from '@/components/splash-screen'
import {
  type AcademyInfo,
  deleteNoticeAsset,
  getAcademyInfo,
  readNoticeAsset,
  saveAcademyInfo,
  saveNoticeAsset,
} from '@/lib/tauri'
import { bytesToDataUrl } from '@/lib/notice-generator'

type ImageSlot = 'logo' | 'barcode'

const IMAGE_META: Record<
  ImageSlot,
  { field: 'logo_filename' | 'barcode_filename'; base: string; label: string }
> = {
  logo: { field: 'logo_filename', base: 'academy_logo', label: '교습소 로고' },
  barcode: { field: 'barcode_filename', base: 'academy_barcode', label: '교습소 2D바코드' },
}

const extOf = (filename: string): 'jpg' | 'png' => {
  const lower = filename.toLowerCase()
  return lower.endsWith('.jpg') || lower.endsWith('.jpeg') ? 'jpg' : 'png'
}
const mimeOf = (filename: string): string =>
  extOf(filename) === 'jpg' ? 'image/jpeg' : 'image/png'

export default function AcademyInfoPage() {
  const [info, setInfo] = useState<AcademyInfo | null>(null)
  const [previews, setPreviews] = useState<Record<ImageSlot, string | null>>({
    logo: null,
    barcode: null,
  })
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [savedAt, setSavedAt] = useState<string | null>(null)
  const [confirmRemove, setConfirmRemove] = useState<ImageSlot | null>(null)
  const inputRefs = {
    logo: useRef<HTMLInputElement>(null),
    barcode: useRef<HTMLInputElement>(null),
  }

  useEffect(() => {
    getAcademyInfo()
      .then(async (data) => {
        setInfo(data)
        for (const slot of ['logo', 'barcode'] as ImageSlot[]) {
          const name = data[IMAGE_META[slot].field]
          if (!name) continue
          try {
            const bytes = await readNoticeAsset(name)
            if (bytes.length > 0) {
              setPreviews((p) => ({ ...p, [slot]: bytesToDataUrl(bytes, mimeOf(name)) }))
            }
          } catch {
            /* 미리보기 실패 — 폼 사용에는 영향 없음 */
          }
        }
      })
      .catch((e: unknown) =>
        setError(typeof e === 'string' ? e : '교습소 정보를 불러올 수 없습니다.'),
      )
  }, [])

  const updateField = (patch: Partial<AcademyInfo>) => {
    setInfo((prev) => (prev ? { ...prev, ...patch } : prev))
    setSavedAt(null)
  }

  const handleUpload = async (slot: ImageSlot, file: File) => {
    if (!info) return
    setError(null)
    const meta = IMAGE_META[slot]
    try {
      const filename = `${meta.base}.${extOf(file.name)}`
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()))
      const saved = await saveNoticeAsset(filename, bytes)
      // 확장자가 바뀌어 기존 파일명과 다르면 옛 파일 제거(잔존 방지).
      const prev = info[meta.field]
      if (prev && prev !== saved) await deleteNoticeAsset(prev)
      updateField({ [meta.field]: saved } as Partial<AcademyInfo>)
      setPreviews((p) => ({ ...p, [slot]: bytesToDataUrl(bytes, mimeOf(saved)) }))
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : `${meta.label} 업로드에 실패했습니다.`)
    }
  }

  const handleRemoveImage = async (slot: ImageSlot) => {
    setConfirmRemove(null)
    if (!info) return
    const meta = IMAGE_META[slot]
    const name = info[meta.field]
    if (!name) return
    try {
      await deleteNoticeAsset(name)
      updateField({ [meta.field]: null } as Partial<AcademyInfo>)
      setPreviews((p) => ({ ...p, [slot]: null }))
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : `${meta.label} 삭제에 실패했습니다.`)
    }
  }

  const handleSave = async () => {
    if (!info) return
    setError(null)
    if (info.academy_name.trim() === '') {
      setError('교습소명을 입력해주세요.')
      return
    }
    setSaving(true)
    try {
      await saveAcademyInfo(info)
      setSavedAt(new Date().toLocaleTimeString('ko-KR'))
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : '저장에 실패했습니다. 잠시 후 다시 시도해주세요.')
    } finally {
      setSaving(false)
    }
  }

  if (info === null && error === null) {
    return <SplashScreen message="교습소 정보를 불러오는 중입니다..." />
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-3xl">
        <SettingsHomeLink />
        <div className="mb-6">
          <h1 className="text-2xl font-bold">교습소 정보</h1>
          <p className="mt-1 text-sm text-gray-600">
            교습소 기본 정보를 입력합니다. 변경 후 저장 버튼을 눌러야 반영됩니다.
          </p>
        </div>

        {error !== null && (
          <p
            role="alert"
            className="mb-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        {info !== null && (
          <div className="space-y-5 rounded-lg border border-[var(--border)] bg-white p-6">
            <TextField
              label="교습소명"
              required
              value={info.academy_name}
              onChange={(v) => updateField({ academy_name: v })}
            />
            <TextField
              label="대표자(원장명)"
              value={info.representative}
              onChange={(v) => updateField({ representative: v })}
            />
            <TextField
              label="연락처"
              value={info.phone}
              onChange={(v) => updateField({ phone: v })}
            />
            <TextField
              label="주소"
              value={info.address}
              onChange={(v) => updateField({ address: v })}
            />
            <TextField
              label="사업자등록번호"
              value={info.business_number ?? ''}
              onChange={(v) => updateField({ business_number: v === '' ? null : v })}
            />
            <NumberField
              label="교습 최대인원 수"
              suffix="명"
              value={info.max_capacity}
              onChange={(n) => updateField({ max_capacity: n })}
            />
            <NumberField
              label="교습소 면적"
              suffix="㎡"
              allowDecimal
              value={info.area_sqm}
              onChange={(n) => updateField({ area_sqm: n })}
            />

            <div className="grid gap-5 sm:grid-cols-2">
              {(['logo', 'barcode'] as ImageSlot[]).map((slot) => (
                <ImageField
                  key={slot}
                  label={IMAGE_META[slot].label}
                  previewUrl={previews[slot]}
                  inputRef={inputRefs[slot]}
                  onPick={() => inputRefs[slot].current?.click()}
                  onFile={(file) => void handleUpload(slot, file)}
                  onRemove={() => setConfirmRemove(slot)}
                />
              ))}
            </div>

            <div className="flex items-center justify-end gap-3 pt-2">
              {savedAt !== null && <p className="text-sm text-gray-600">저장 완료 — {savedAt}</p>}
              <button
                type="button"
                onClick={handleSave}
                disabled={saving}
                className="h-11 rounded-md bg-[var(--accent)] px-5 font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
              >
                {saving ? '저장 중...' : '저장'}
              </button>
            </div>
          </div>
        )}
      </div>

      {confirmRemove !== null && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="mx-4 w-full max-w-sm rounded-lg bg-white p-6 shadow-xl">
            <p className="text-base text-[var(--foreground)]">
              {IMAGE_META[confirmRemove].label} 이미지를 삭제하시겠습니까?
            </p>
            <div className="mt-5 flex justify-end gap-3">
              <button
                type="button"
                onClick={() => setConfirmRemove(null)}
                className="h-11 rounded-md border border-[var(--border)] px-4 hover:bg-gray-50"
              >
                취소
              </button>
              <button
                type="button"
                onClick={() => void handleRemoveImage(confirmRemove)}
                className="h-11 rounded-md bg-[var(--danger)] px-4 font-semibold text-white hover:opacity-90"
              >
                삭제
              </button>
            </div>
          </div>
        </div>
      )}
    </AppShell>
  )
}

function TextField({
  label,
  value,
  onChange,
  required,
}: {
  label: string
  value: string
  onChange: (v: string) => void
  required?: boolean
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-base font-medium">
        {label}
        {required && <span className="ml-1 text-[var(--danger)]">*</span>}
      </span>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="h-11 w-full rounded-md border border-[var(--border)] px-3 text-base"
      />
    </label>
  )
}

function NumberField({
  label,
  value,
  onChange,
  suffix,
  allowDecimal,
}: {
  label: string
  value: number | null
  onChange: (n: number | null) => void
  suffix?: string
  allowDecimal?: boolean
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-base font-medium">{label}</span>
      <div className="flex items-center gap-2">
        <input
          type="number"
          min={0}
          step={allowDecimal ? '0.1' : '1'}
          value={value ?? ''}
          onChange={(e) => {
            const raw = e.target.value
            if (raw === '') return onChange(null)
            const n = allowDecimal ? Number.parseFloat(raw) : Number.parseInt(raw, 10)
            onChange(Number.isNaN(n) ? null : n)
          }}
          className="h-11 w-40 rounded-md border border-[var(--border)] px-3 text-base"
        />
        {suffix && <span className="text-base text-gray-600">{suffix}</span>}
      </div>
    </label>
  )
}

function ImageField({
  label,
  previewUrl,
  inputRef,
  onPick,
  onFile,
  onRemove,
}: {
  label: string
  previewUrl: string | null
  inputRef: React.RefObject<HTMLInputElement | null>
  onPick: () => void
  onFile: (file: File) => void
  onRemove: () => void
}) {
  return (
    <div>
      <span className="mb-1 block text-base font-medium">{label}</span>
      <div className="flex min-h-[120px] items-center justify-center rounded-md border border-dashed border-[var(--border)] bg-gray-50 p-3">
        {previewUrl ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img src={previewUrl} alt={`${label} 미리보기`} className="max-h-28 max-w-full object-contain" />
        ) : (
          <span className="text-sm text-gray-400">등록된 이미지가 없습니다 (PNG/JPG)</span>
        )}
      </div>
      <input
        ref={inputRef}
        type="file"
        accept="image/png,image/jpeg"
        className="hidden"
        onChange={(e) => {
          const f = e.target.files?.[0]
          if (f) onFile(f)
          e.target.value = ''
        }}
      />
      <div className="mt-2 flex gap-2">
        <button
          type="button"
          onClick={onPick}
          className="h-11 rounded-md border border-[var(--border)] px-4 text-sm hover:bg-gray-50"
        >
          {previewUrl ? '이미지 교체' : '이미지 등록'}
        </button>
        {previewUrl && (
          <button
            type="button"
            onClick={onRemove}
            className="h-11 rounded-md border border-[var(--border)] px-4 text-sm text-[var(--danger)] hover:bg-red-50"
          >
            삭제
          </button>
        )}
      </div>
    </div>
  )
}
