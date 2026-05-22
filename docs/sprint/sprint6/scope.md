---
Sprint: 6  |  Date: 2026-05-22  |  Session: #5
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — T2-c ADR 단독 세션.
> 공휴일 API 소스 + 저장 테이블 의사결정. brainstorming 스킬(Weighted Matrix + SWOT + ADR).
> 예상 1.5h. T2-a/T2-b(스크립트 + V301)는 다음 Session #6 에서 ADR 결정 산출물 기반으로 구현.

## 이전 세션 결과 (참고 — 모두 완료)

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 (A20 lock 재시도) | `2c5b8a1` |
| #1 | T3 (A21 paths.rs OnceLock 분리) | `c2be584` |
| #1 | T4 (A22 DnD 방법 B) | `83f19d1` |
| #2 | T5+T6 (academic.rs 신규 — study_periods 6 + schedule_codes 4) | `c8dc3c8` |
| #3 | T7 (academic.rs 확장 — schedule_events 5) | `a4c380e` |
| #4 | T8 (TS IPC 래퍼 15 + 도메인 타입 10) | `5941d24` |

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T2-c** | brainstorming 스킬 적용 — 공휴일 API + 저장 테이블 결정 → ADR-005 작성 | 1.5h |

> 사용자 의사결정(2026-05-22): T2 세션 분할 = T2-c 단독 먼저 → T2-a/b 다음 세션. 스크립트 언어 = TypeScript(tsx). API 사전 선호 = 없음 (brainstorming 결정).

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/arch/adr-005-holiday-api-selection.md | [1회] | 신규 ADR — 공휴일 API 소스 + 저장 테이블 결정 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/` — 본 세션 코드 변경 없음
- [ ] `src/` — 본 세션 코드 변경 없음
- [ ] `package.json` / `Cargo.toml` — 본 세션 의존성 추가 없음 (tsx 는 Session #6 에서)
- [ ] `src-tauri/migrations/` — V301 은 Session #6 에서

## 완료 기준 (이번 세션)

### T2-c — ADR-005 (PRD §4.4, sprint6.md L98-106)
- ✅ AC-T2-7: ADR-005 문서 작성 완료 — API 소스 + 저장 테이블 + 갱신 주기 결정 모두 포함
- ✅ brainstorming 스킬 3단계 모두 충실 적용:
  - 1단계: Weighted Matrix 2개 (API 6기준×2후보 / 저장 6기준×2후보, 가중치 합 1.0)
  - 2단계: SWOT 4선택지 + Trade-off 4행 + Risk 5건
  - 3단계: ADR (Context 8항목 / Decision 3 / Consequences / 후속 액션 6건)
- ✅ 비교 선택지 최소 2개씩 충족 (API: 공공데이터포털 vs date.nager.at, 저장: schedule_events vs 별도 holidays)
- ✅ 갱신 주기 명시 — 2029-12 이전 재실행, V401(+) 마이그레이션 패턴
- ✅ Session #6 후속 액션 6건 명확화 (T2-a 3건 + T2-b 3건)

### 결정 요약

| 항목 | 결정 | 총점 |
|------|------|-----|
| API 소스 | **공공데이터포털** (data.go.kr 특일 정보 API) | 4.10 vs date.nager.at 3.80 |
| 저장 위치 | **schedule_events** 통합 (공휴일 시스템 코드 추가) | 4.65 vs 별도 holidays 3.10 |
| 갱신 트리거 | **2029-12 이전** 재실행 + V401(+) 마이그레이션 | - |

### 세션 종료 조건
- ✅ ADR-005 단일 커밋 `10a92d4` (docs/arch/adr-005-holiday-api-selection.md +206줄)
- ✅ scope.md 완료 마킹 (별도 커밋, 본 작업)
- ✅ simplify 스킬 — 적용 대상 외 (코드 변경 없음, 문서만)
- ✅ Session #6 진입 안내 — T2-a (scripts/fetch-holidays.ts + tsx devDep + .env.example) + T2-b (V301 마이그레이션)

## 비교 대상 사전 정리

### API 후보

| 후보 | 인증 | 한국 공휴일 정확도 | 대체공휴일 | 라이선스 |
|------|------|----------------|----------|--------|
| 공공데이터포털 (data.go.kr 특일 정보 API) | 인증키 필요 (data.go.kr 회원가입 → 활용신청 → 발급) | 한국 공식 데이터 (천문연구원) | 정확 (대체공휴일법 반영) | 공공누리 제1유형 (무료, 재배포 가능) |
| date.nager.at API | 무인증 (REST GET) | 미검증 — 대체공휴일 누락 가능 보고됨 | 부분 지원 (한국 데이터는 GitHub PR 기반) | MIT (오픈소스) |

### 저장 위치 후보

| 후보 | 스키마 영향 | 코드 모델 일관성 | 쿼리 복잡도 |
|------|----------|--------------|----------|
| schedule_events 테이블에 INSERT (공휴일 code_id 참조) | V102 schedule_codes 시드에 "공휴일" 코드 추가 필요. V301 시드에서 7년치 INSERT | 학사 일정 코드 3속성 모델과 동일 흐름 (캘린더 동일 쿼리로 표시) | 단순 — list_schedule_events 가 이미 코드 JOIN |
| 별도 `holidays` 전용 테이블 신설 (V302) | V302 마이그레이션 신규 필요 (holiday_date, name) | 학사 일정 코드 모델과 분리 — 캘린더 컴포넌트가 두 소스 병합 필요 | 복잡 — list_schedule_events + list_holidays 별도 IPC 필요 |

### 갱신 주기 결정 항목

- 빌드 타임 1회 수집 후 V301 시드 SQL 에 박제
- 만료 (2030 이후) 대응: `pnpm holidays:fetch` 재실행 → V401 (또는 V302 단위) 새 마이그레이션 추가 패턴
- 트리거: ROADMAP.md 또는 Sprint 회고에 "2029-12 까지 갱신 검토" 메모

## 발견된 이슈

> ADR 작성 중 새 제약 발견 시 여기에 기록.
