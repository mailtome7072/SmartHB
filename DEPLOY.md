# 배포 후 수동 작업 가이드

> **목적**: 현재 배포 사이클의 검증 현황(자동 완료 ✅ + 수동 미완료 ⬜)을 유지합니다.
> 다음 배포 시작 시 이전 배포 사이클 전체를 `docs/deploy-history/YYYY-MM-DD.md`로 아카이빙합니다.

---

## 아카이빙 규칙

- **시점**: 다음 스프린트/핫픽스 배포 시작 전 (이전 배포 사이클 전체를 이동)
- **담당**: sprint-close / hotfix-close agent (스프린트·핫픽스 완료 시 1차 아카이빙),
            deploy-prod agent (프로덕션 배포 시 최종 아카이빙)
            (수동 이동 시 아래 규칙 준수)
- **파일명**: `docs/deploy-history/YYYY-MM-DD.md` (배포 날짜 기준)
- **방식**: 이 파일의 완료된 배포 섹션 전체를 해당 날짜 파일로 이동 후 해당 섹션 삭제

---

## 항목 작성 형식

sprint-close / hotfix-close agent 및 팀원이 항목 추가 시 아래 형식을 준수합니다.

```markdown
## YYYY-MM-DD | vX.Y.Z | Sprint{n} 또는 Hotfix/{설명}

### 스테이징 검증 (develop 로컬)
- ⬜ pnpm tauri:dev 로 로컬 스테이징 실행 및 주요 흐름 동작 확인
- ⬜ 교습일정 인쇄 미리보기 확인 — 교습기간 1개월/2개월/3개월 걸침 각각 달력이 읽을 수 있는 크기로 정상 출력 (Sprint 20 A122: 인쇄 시각 QA 자동화 불가 → 배포 전 수동 검증 의무)
- ⬜ sqlx migrate run (DB 스키마 변경이 있는 경우)
- ⬜ 클라우드 동기화 폴더 락 파일(`app.lock`) / 백업 디렉토리(`backup/exit|hourly|daily|weekly`) 정상 생성 확인
- ⬜ 앱 시작 시 PRAGMA integrity_check 통과 확인

### 프로덕션 배포 후 검증 (인스톨러 설치)
- ⬜ Windows 인스톨러(`.msi`/`.exe`) 또는 macOS 인스톨러(`.dmg`) 다운로드 및 설치
- ⬜ 초기 설정 마법사(PRD §4.0) 진입 또는 기존 데이터 정상 로드 확인
- ⬜ UI 디자인/시각적 품질 확인 (Pretendard 폰트, 18pt+, 명도 대비)
- ⬜ (추가 확인 항목)

### Notion 업데이트
- ⬜ (해당되는 항목만 기재 — dev-process.md 섹션 8.5 트리거 참조)
```

> 체크리스트 형식: 완료 `✅` / 미완료 `⬜` (GFM `[x]`/`[ ]` 사용 금지)

---

## 현재 배포 현황

## 2026-07-29 | Sprint 24 — 다월 교습기간 year_month 오태깅 계열 버그 일괄 수정 + V313 데이터 전수 보정

브랜치: `sprint24 → develop` 직접 머지 예정 (단일 개발자 정책)

### 스테이징 검증 (develop 로컬)
- ✅ sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증) — **초기 구현분 기준. 아래 QA 추가 수정분은 재리뷰 필요**
- ✅ pnpm tauri:dev 실행하여 앱 동작 수동 확인 — CS 원인 사례(A 원생 8월 7/30 누락 + "변경 필요" 배지) 해소 확인
- ✅ 수동 검증 중 발견·수정된 추가 결함 5건 정상 동작 확인 (아래 목록)
- ✅ 교습일정 인쇄 미리보기 확인 (Sprint 20 A122 계속 유지)
- ⬜ 원장 PC / 자택 Mac 양 PC에서 새 버전 실행 — V313 데이터 자동 보정 + startup 재동기화 적용 확인 (배포 후 실환경)

#### 수동 검증 중 발견·수정된 추가 결함 (모두 develop 반영, 정상 동작 확인)
1. 교습기간 경계 (재)설정 시 출결 재태깅 갭 — `periods::reconcile_attendance_year_month` + 시작/변경 훅
2. 시수 변경 시 청구 재확인 팝업 신설 — `list_affected_bill_months` IPC + 모달
3. 청구 주당시간 산정을 교습기간 유효 스케줄 기준(이력 인식)으로 수정 — 7월 3시간 정상화 + "추가 청구 데이터 생성" 유령 버튼 해소
4. 수업일 이동 팝업 달력을 교습기간 범위로 확장 (B1 프론트 보완) — 다월 이동 대상 선택 가능
5. (초기 sprint-review 지적 M1/M2/L1/L2 반영 — sync N+1 제거, reinstate CASE, periods 모듈 분리, 폴백 audit)

> ⚠️ 위 추가 수정분은 초기 sprint-review 이후 커밋되었으므로, **배포 전 sprint-review 재실행 권장**.

이전 배포 기록: `docs/deploy-history/2026-07-23.md` (v1.5.0 Sprint 23 프로덕션 아카이빙)

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
