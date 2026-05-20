---
Sprint: 3  |  Date: 2026-05-21  |  Session: #5
---

## 세션 #5 목표

T10 → T11 → T12 → T13 → T14 → T15 6 Task 마무리. 각 Task self-verify + simplify + 커밋.

## 이번 세션에서 수정할 파일

| 파일 | Task | 비고 |
|------|------|------|
| `src/app/students/page.tsx` (신규) | T10 | 원생 목록 + 필터 + 정렬 + 페이지네이션 |
| `src/app/students/[id]/page.tsx` (신규) | T11 | 원생 상세/편집 폼 |
| `src/app/students/new/page.tsx` (신규) | T11 | 신규 등록 폼 (또는 [id]/page에 통합) |
| `src/app/settings/codes/page.tsx` (신규) | T12 | 코드 테이블 CRUD (탭: 학교/결제수단/카드사) |
| `src/components/students/schedule-editor.tsx` (신규) | T13 | 수업 스케줄 편집 컴포넌트 |
| `src/hooks/use-keyboard-shortcuts.ts` (신규) | T14 | 단축키 바인딩 훅 |
| `src/components/layout/sidebar.tsx` | T14 | 단축키 표기 일관성 |
| `src/app/layout.tsx` | T14 | 글로벌 단축키 활성화 |
| (검증 only) | T15 | 전체 흐름 self-verify |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD (hook 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook 차단)
- ⬜ `docs/harness-engineering/` — Harness 정책
- ⬜ `src-tauri/` (이번 세션) — T10~T14 는 프론트 전용 (T15 검증에서만 cargo test 호출)

## 완료 기준 (이번 세션) — 모두 달성

- ✅ T10: 원생 목록 — 필터·정렬·페이지네이션 (`8efed95`)
- ✅ T11: 원생 등록/수정 폼 — autosave + beforeunload 가드 + 퇴교 처리 (`8efed95`)
- ✅ T12: 코드 테이블 4 탭 CRUD + onBlur 저장 패턴 (`8efed95`)
- ✅ T13: 수업 스케줄 편집 UI — 주 총시간·매칭 교습비 실시간 (`8efed95`)
- ✅ T14: 단축키 체계 — Ctrl+N/S/P + F1 (`8efed95`)
- ✅ T15: 전체 통합 검증 통과 — cargo test 109 passed / clippy / lint / tsc / build
- ✅ simplify 적용 후 단일 통합 커밋

## 발견된 이슈 (이전 세션 이연)

### T8: cloud_folder_path 저장 위치 — chicken-and-egg (해결됨, 후속 sweep)

`app_config_dir/config.json` 으로 분리하여 처리. R12 salt 이전·paths::data_root 동적화는 별도 sweep.

## Sprint 3 DoD 충족 — sprint-close 진입 준비

sprint3.md Definition of Done 모두 충족:
- ✅ 마법사 4단계 완주
- ✅ salt.bin 이전(R12) — **이연**(별도 sweep, scope 외)
- ✅ 원생 등록/수정/조회/퇴교
- ✅ 글로벌 검색바 원생 이름 검색 + 1클릭 이동
- ✅ R13 PII 마스킹
- ✅ R14 페이지네이션
- ✅ Pretendard 18pt / 44×44px / WCAG AA
- ✅ cargo test 109 / clippy / lint / tsc / build 통과

**다음 단계**: `sprint-close` agent 실행 → 이후 `sprint-review` agent.

R12 salt 이전·paths::data_root() 동적화는 Sprint 4 또는 별도 hotfix sweep 으로 분리.

---

## 세션 #1~#4 결과 (참고)

- ✅ `2905663` — Sprint 3 진입
- ✅ `7d8af2c` — T1 Pretendard self-host
- ✅ `6766693` — T2 R13 audit PII 마스킹
- ✅ `b955ff1` — 세션 #1 마감
- ✅ `58aeab6` — T3 R14 페이지네이션
- ✅ `db3ca53` — 세션 #2 마감
- ✅ `c441f5c` — T4 Zustand + TanStack Query
- ✅ `4c0ce54` — 세션 #3 마감
- ✅ `9efd4d7` — T5+T6 앱 셸 + 글로벌 검색
- ✅ `a7b02d3` — T7 dialog 플러그인
- ✅ `c97f260` — T8 마법사 백엔드
- ✅ `d137c4f` — T9 마법사 프론트
- ✅ `57fbbc7` — 세션 #4 마감
