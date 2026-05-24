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

### Sprint 8 (2026-05-24)

브랜치: `sprint8` → `develop` 직접 머지 (단일 개발자 정책)

#### 주요 변경
- V106 마이그레이션 (regular_attendances + makeup_attendances 테이블)
- 출결 IPC 6종 (generate_attendances / check_attendance_exists / get_attendance_grid / toggle_attendance / update_absence_memo / get_attendance_summary)
- 출결표 UI (/attendance 라우트, AttendanceGrid, AbsenceMemoDialog, 원생 검색 필터)
- Sprint 7 carry-over 해소: High 4건 (R40~R43) + Medium-High 1건 (R45) + Medium 6건 (R46~R48-a/R39/R51/A31)
- 보강필요시간/소멸기한 단위 테스트 10 시나리오 100% 커버

#### 자동 검증 (T9, 2026-05-24)
- ✅ cargo test --lib cipher off: 221 passed
- ✅ cargo test --lib cipher on: 133 passed (3 ignored)
- ✅ cargo clippy --lib -- -D warnings: cipher off + on clean
- ✅ pnpm lint: clean / pnpm tsc --noEmit: clean / pnpm build: clean (out/ 정상)

#### 수동 검증 필요 항목
- ⬜ sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증)
- ⬜ pnpm tauri:dev 실행하여 앱 동작 수동 확인 (스테이징 검증)
  - ⬜ 출결 생성 → 출결표 표시 → 출석/결석 토글 UC-3 전체 흐름
  - ⬜ 결석 메모 입력 다이얼로그 동작
  - ⬜ 보강필요시간 실시간 업데이트 확인
  - ⬜ 원생 검색 필터 동작 확인
  - ⬜ 콘솔 에러/경고 없음 확인

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
