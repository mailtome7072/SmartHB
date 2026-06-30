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

### Hotfix: 장시간 사용 시 DB 저장 실패 수정 (2026-06-30)

브랜치: `hotfix/db-lock-and-backup-fix` → master 직접 머지 (단일 개발자 정책)
태그: v1.0.1

#### 수정 내용
- `src-tauri/src/commands/db.rs`: PRAGMA busy_timeout=30000, journal_size_limit=64MB, pool acquire_timeout=30초 추가
- `src-tauri/src/commands/backup.rs`: rusqlite src/dst busy_timeout=30초, rehearsal pool busy_timeout+acquire_timeout 추가
- `src-tauri/src/startup.rs`: hourly 루프에 PRAGMA wal_checkpoint(PASSIVE) 추가

#### 자동 검증 완료 항목
- ✅ cargo test: 417 passed, 0 failed (hotfix 구현 시 완료)
- ✅ cargo clippy --all-targets: 경고 없음 (hotfix 구현 시 완료)
- ✅ 코드 리뷰: Critical/High 이슈 없음 — Low 이슈 1건 기록 (backup.rs 30초 타임아웃 magic number, 다음 스프린트 이연)
- ✅ master 직접 머지 완료 (f867a1d)
- ✅ v1.0.1 태그 push 완료 (2026-06-30) → GitHub Actions 빌드 트리거

#### 수동 검증 필요 항목
- ⬜ pnpm tauri:dev 실행 후 장시간 사용 시나리오 확인 (클라우드 동기화 폴더 환경에서 DB 저장 정상 동작)
- ⬜ GitHub Release v1.0.1 아티팩트 업로드 확인 (gh release view v1.0.1)
  - ⬜ Windows: .exe (NSIS, --features cipher)
  - ⬜ macOS: .dmg (--features cipher)

#### 코드 리뷰 Low 이슈 (배포 차단 아님)
- backup.rs의 30초 타임아웃 값이 db.rs의 `ACQUIRE_TIMEOUT_SECS` 상수와 공유되지 않음 — 다음 스프린트에서 상수 통합 권장

이전 배포 기록: `docs/deploy-history/2026-06-12.md` (v1.0.0)

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
