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

### Sprint 6 (2026-05-22) — v0.3.0 배포 **보류**
브랜치: `sprint6` → `develop` 머지 완료 (`dc3139e` + 후속 `043958a`, `2bb0f6c`)
- ✅ sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증)
- ✅ `pnpm tauri:dev` 수동 검증 1차 (시각 확인) — 7건 이슈 발견
- 🔄 사용자 결정: deploy-prod 보류 → Sprint 7 완료 후 v0.3.1 통합 배포

**시각 검증에서 발견된 이슈 (Sprint 7 carry-over)**:
- Issue 1: macOS Keychain 비밀번호 다이얼로그 반복 (startup 31초 소요) — keyring 호출 패턴 재설계
- Issue 2: 종료 메뉴 추가 → **develop 직접 패치 완료** (`2bb0f6c`, Tauri window.close)
- Issue 3: 학사 일정 코드 관리는 `/settings` 하위로 이동 (UX 재설계)
- Issue 4: 학사 일정 배치 — 교습기간 내만 허용 + 중복불가는 다른 코드와도 일자 충돌 검증
- Issue 5: 교습기간 설정 UX 재설계 (토글 버튼 제거, 캘린더 인라인 시작/끝 선택)
- Issue 6: 교습기간 삭제 버튼 + cascade 삭제 (공휴일 제외 학사일정 일괄)
- Issue 7: 확정 교습기간 내 공휴일은 삭제 불가 (백엔드 가드 추가)
- 추가: stale lock 자동 점유 (device_id 영속화 필요) / R33 (시스템 코드명 하드코딩) / R34 (교습기간 외 드롭) / A17 (salt.bin Keychain → cloud)

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
