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

## 2026-07-29 | v1.5.1 | Sprint 24 프로덕션 배포

### 포함 스프린트
- Sprint 24: 다월 교습기간 year_month 오태깅 계열 버그 일괄 수정 + V313 데이터 전수 보정

### 배포 상태
- ✅ develop 스테이징 검증 완료 (cargo test 497 / clippy / tsc / lint / build)
- ✅ sprint-review 완료 (Critical 0 / High 0 / Low 2 이연)
- ✅ CHANGELOG [Unreleased] → [1.5.1] 버전 전환
- ✅ 버전 파일 3곳 + Cargo.lock 동기화 (1.5.0 → 1.5.1)
- ✅ develop → master 직접 머지 완료
- ✅ v1.5.1 태그 push → GitHub Actions 빌드 완료
- ✅ GitHub Release 아티팩트 업로드 확인 (2026-07-29, Latest)
  - ✅ Windows: SmartHB_1.5.1_x64-setup.exe
  - ✅ macOS: SmartHB_1.5.1_aarch64.dmg

### CV — 아티팩트 검증
- ✅ gh release view v1.5.1 으로 Release 확인 (Latest, 2026-07-29)
- ✅ 아티팩트 2종 업로드 확인 (dmg / exe)

### CV — 실 PC 수동 검증 (실물 PC에서 직접 수행 — 자동화 불가)
> 원장 PC(교습소, Windows) + 자택 PC(Windows) **양쪽 모두** v1.5.1 설치 후 실행해야 오태깅 자동 교정이 완결됨. (실사용 PC 2대 모두 Windows 환경)

**원장 PC (교습소, Windows)**
- ⬜ v1.5.1 설치 후 첫 실행 — 정상 기동 + 로그인
- ⬜ V313 마이그레이션 + startup 재동기화 자동 적용 (기존 오태깅 출결·보강 year_month 자동 교정)
- ⬜ CS 원인 사례(A 원생 8월 그리드에 7/30 표시) 해소 확인
- ⬜ 기존 데이터 무손실 로드 (원생/출결/청구/수납)

**자택 PC (Windows)**
- ⬜ v1.5.1 설치 후 첫 실행 — 정상 기동 + 로그인
- ⬜ 양 PC 모두 새 버전 실행 후 오태깅 교정 결과 최종 반영 확인
- ⬜ 기존 데이터 무손실 로드

**이월 검증 (Sprint 20 A122)**
- ⬜ 교습일정 인쇄 미리보기 — 교습기간 1/2/3개월 걸침 각각 읽을 수 있는 크기로 정상 출력

### 배포 노트
- DB 마이그레이션 V313 포함 — 앱 첫 실행 시 자동 적용(기존 오태깅 출결·보강 year_month 전수 교정, 멱등·트랜잭션 보장)
- 신규 IPC: `list_affected_bill_months`, `reconcile_attendance_year_month` (내부 자가 치유)

스테이징 검증 기록: `docs/deploy-history/2026-07-29.md`

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
