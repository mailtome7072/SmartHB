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

## 2026-07-21 | Post-v1.3 | Sprint 22 — 보강 분 단위 부분 차감 전환 + 출결 그리드 z-index 수정

브랜치: `sprint22 → develop` 직접 머지 (단일 개발자 정책)

### 스테이징 검증 (develop 로컬)
- ⬜ sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증)
- ⬜ pnpm tauri:dev 실행하여 앱 동작 수동 확인 (스테이징 검증)
- ⬜ sqlx migrate run — V311/V312 마이그레이션 정상 적용 확인 (개발 DB 기준)
- ⬜ 보강 등록 시 잔여분 UI 정상 표시 확인 (예: 2h 결석 → 1h 보강 후 잔여 1h 재노출)
- ⬜ 출결 그리드 상하 스크롤 시 헤더 고정 시각 확인 (z-index 수정)
- ⬜ 보강 취소 후 결석 잔여분 원복 확인
- ⬜ 클라우드 동기화 폴더 cipher-on 실 DB 에서 V311/V312 백필 스모크 테스트 (deploy-prod 단계 권장)
- ⬜ 교습일정 인쇄 미리보기 확인 — 교습기간 1개월/2개월/3개월 걸침 각각 달력이 읽을 수 있는 크기로 정상 출력 (A122 계속 유지)

이전 배포 기록: `docs/deploy-history/2026-07-20.md` (v1.3.0 프로덕션 + Sprint 21 스테이징 아카이빙)

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
