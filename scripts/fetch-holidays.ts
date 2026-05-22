#!/usr/bin/env tsx
/**
 * 한국 법정 공휴일 빌드 타임 수집 스크립트 — Sprint 6 T2-a (ADR-005).
 *
 * data.go.kr 특일 정보 API (한국천문연구원) 호출 → V301 마이그레이션 시드용 SQL INSERT stdout.
 *
 * 사용법:
 *   cp .env.example .env  # KOREA_HOLIDAY_API_KEY 채우기
 *   pnpm holidays:fetch                              # 기본 2024~2030
 *   pnpm holidays:fetch -- --years 2031-2037         # 갱신 시 (2029-12 이전)
 *   pnpm holidays:fetch > scripts/holidays.generated.sql
 *
 * PRD §5.5: 앱 런타임은 외부 네트워크 호출 금지. 본 스크립트는 빌드 타임 1회 사용.
 *
 * 환경변수 로드: package.json `holidays:fetch` 스크립트가 `node --env-file=.env` 로 자동 로드.
 * dotenv 패키지 의존성 추가 회피 (Node 20.6+ 내장 기능 활용).
 */

interface HolidayItem {
  dateKind: string  // '01'=국경일/공휴일, '02'=기념일, '03'=24절기, '04'=잡절
  dateName: string  // 공휴일명
  isHoliday: 'Y' | 'N'
  locdate: number   // YYYYMMDD 정수
  seq: number
}

interface ApiResponse {
  response: {
    header: { resultCode: string; resultMsg: string }
    body: {
      items?: { item: HolidayItem | HolidayItem[] } | ''
      numOfRows: number
      pageNo: number
      totalCount: number
    }
  }
}

const API_BASE = 'http://apis.data.go.kr/B090041/openapi/service/SpcdeInfoService/getRestDeInfo'

function die(message: string, code = 1): never {
  process.stderr.write(`[fetch-holidays] ${message}\n`)
  process.exit(code)
}

function parseYearRange(input: string): [number, number] {
  const match = input.match(/^(\d{4})-(\d{4})$/)
  if (!match) die(`--years 형식 오류: "${input}" — 예: "2024-2030"`)
  const from = Number(match![1])
  const to = Number(match![2])
  if (from > to) die(`--years 시작 연도가 종료 연도보다 큼: ${from} > ${to}`)
  if (to - from > 20) die(`--years 범위가 너무 큼: ${to - from}년 (최대 20)`)
  return [from, to]
}

function parseArgs(argv: string[]): { fromYear: number; toYear: number } {
  let fromYear = 2024
  let toYear = 2030
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    if (arg === '--years') {
      const next = argv[++i]
      if (!next) die('--years 인자 누락')
      ;[fromYear, toYear] = parseYearRange(next)
    } else if (arg === '-h' || arg === '--help') {
      process.stdout.write(`사용: pnpm holidays:fetch [-- --years YYYY-YYYY]\n기본: 2024-2030\n`)
      process.exit(0)
    }
  }
  return { fromYear, toYear }
}

async function fetchYear(year: number, apiKey: string): Promise<HolidayItem[]> {
  // data.go.kr 인증키는 base64 padding (`+`, `=`, `/`) 을 포함할 수 있다 —
  // URL.searchParams.set 은 이미 URL-safe 인 문자도 재인코딩하여 HTTP 403 을 유발한다.
  // 해결: 키만 raw 로 박고, 다른 파라미터는 안전하게 인코딩.
  const params = [
    `serviceKey=${apiKey}`,
    `solYear=${year}`,
    `numOfRows=100`,
    `_type=json`,
  ].join('&')
  const res = await fetch(`${API_BASE}?${params}`)
  if (!res.ok) die(`HTTP ${res.status} ${res.statusText} (${year})`)

  const text = await res.text()
  let data: ApiResponse
  try {
    data = JSON.parse(text) as ApiResponse
  } catch {
    // data.go.kr 은 인증 실패 시 XML 에러 응답을 반환할 수 있음
    if (text.includes('SERVICE_KEY_IS_NOT_REGISTERED') || text.includes('INVALID_REQUEST_PARAMETER')) {
      die(`인증키 거부 (${year}): ${text.slice(0, 200)}`)
    }
    die(`JSON 파싱 실패 (${year}): ${text.slice(0, 200)}`)
  }

  const code = data.response?.header?.resultCode
  const msg = data.response?.header?.resultMsg ?? '(no message)'
  if (code !== '00') die(`API 오류 (${year}): resultCode=${code} resultMsg=${msg}`)

  const items = data.response.body.items
  // data.go.kr 은 totalCount=0 시 items 를 빈 문자열로 반환하는 변종 응답을 사용 (대체공휴일 없는 해 등).
  if (!items || typeof items === 'string') return []
  const arr = Array.isArray(items.item) ? items.item : [items.item]
  return arr.filter(it => it.isHoliday === 'Y')
}

function locdateToIso(locdate: number): string {
  const s = String(locdate)
  if (s.length !== 8) die(`locdate 형식 오류: ${locdate}`)
  return `${s.slice(0, 4)}-${s.slice(4, 6)}-${s.slice(6, 8)}`
}

function escapeSqlString(s: string): string {
  return s.replace(/'/g, "''")
}

async function main() {
  const apiKey = process.env.KOREA_HOLIDAY_API_KEY
  if (!apiKey) {
    die(
      '환경변수 KOREA_HOLIDAY_API_KEY 누락\n' +
        '  1) data.go.kr 에서 "특일 정보" API 활용신청 후 인증키 발급\n' +
        '  2) .env 파일에 KOREA_HOLIDAY_API_KEY=<인증키> 추가\n' +
        '  3) pnpm holidays:fetch 재실행',
    )
  }

  const { fromYear, toYear } = parseArgs(process.argv.slice(2))
  process.stderr.write(`[fetch-holidays] ${fromYear}~${toYear}년 수집 중...\n`)

  const rows: { isoDate: string; name: string; dateKind: string }[] = []
  for (let year = fromYear; year <= toYear; year++) {
    const items = await fetchYear(year, apiKey)
    process.stderr.write(`  ${year}: ${items.length}건\n`)
    for (const it of items) {
      rows.push({ isoDate: locdateToIso(it.locdate), name: it.dateName, dateKind: it.dateKind })
    }
  }

  rows.sort((a, b) => (a.isoDate < b.isoDate ? -1 : a.isoDate > b.isoDate ? 1 : 0))

  // SQL 출력 — V301 마이그레이션에 복붙용
  const out = process.stdout
  out.write(`-- 한국 법정 공휴일 ${fromYear}~${toYear} (${rows.length}건)\n`)
  out.write(`-- 생성: pnpm holidays:fetch (data.go.kr 특일 정보 API, ADR-005)\n`)
  out.write(`-- 갱신: ${toYear - 1}-12 이전 재실행 권장\n\n`)
  out.write(`INSERT OR IGNORE INTO schedule_events (code_id, event_date, period_end_date, display_name)\n`)
  out.write(`SELECT c.id, v.event_date, NULL, v.display_name\n`)
  out.write(`FROM schedule_codes c\n`)
  out.write(`CROSS JOIN (VALUES\n`)
  for (let i = 0; i < rows.length; i++) {
    const r = rows[i]
    const comma = i < rows.length - 1 ? ',' : ''
    out.write(`  ('${r.isoDate}', '${escapeSqlString(r.name)}')${comma}\n`)
  }
  out.write(`) AS v(event_date, display_name)\n`)
  out.write(`WHERE c.code_name = '공휴일' AND c.is_system_reserved = 1;\n`)

  process.stderr.write(`[fetch-holidays] 완료: ${rows.length}건 출력\n`)
}

main().catch((err: unknown) => {
  const msg = err instanceof Error ? `${err.message}\n${err.stack ?? ''}` : String(err)
  die(`예상치 못한 오류: ${msg}`)
})
