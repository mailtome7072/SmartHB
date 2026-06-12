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

## 2026-06-12 | v1.0.0 | 프로덕션 배포

### 포함 스프린트
- Sprint 15: 교습소 정보 화면 + 자가 진단 이력 삭제 + 전역 단축키/툴팁 + 원생 상세 UX
- Sprint 16: 수업일 변경 도메인(케이스1/2) + 수업 캘린더 개선 + CSV 가져오기 + 공지문 캔버스 보강 + DB 폴더 변경(ADR-009) + 백업 복원 연결+스케줄러 + 청구/수납 메뉴 분리 + 원생 폼 UX 개선 + v1.0.0 버전업

### 배포 전 검증
- ✅ sprint-review 에이전트 실행 — cargo test 417 passed / clippy --all-targets clean / cipher check OK / lint+tsc+build 전수 통과 (2026-06-12)
- ✅ T11 통합 검증 통과 (2026-06-12)
- ✅ 사용자 실앱 시각검수 완료 (전 태스크)
- ✅ sprint16 → develop 직접 머지 완료 (e9ebfeb)

### 배포 상태
- ✅ develop → master 직접 머지 완료 (단일 개발자 정책, PR 생략, e2e2543)
- ✅ v1.0.0 태그 push 완료 (2026-06-12) → GitHub Actions 빌드 진행 중
- ⬜ GitHub Release 아티팩트 업로드 확인
  - ⬜ Windows: .exe (NSIS, --features cipher)
  - ⬜ macOS: .dmg (--features cipher)

### CV — 아티팩트 검증
- ⬜ gh release view v1.0.0 으로 Release 확인
- ⬜ 다운로드 URL 유효성 확인
- ⬜ A101 스모크 테스트: cipher 실동작 검증 (로그인·백업 생성·무결성·DB폴더 지정) — R123 대응

### risk-register 인지 사항
- R117~R122 Medium/Low — v1.0 이후 안정화 스프린트 이연 (docs/risk-register/2026-06-12.md)
- R123 cipher 실동작 미검증 (높음) — A101 스모크 테스트로 대응

이전 배포 기록: `docs/deploy-history/2026-06-07.md` (Sprint 15)

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
