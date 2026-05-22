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

### Sprint 7 (2026-05-22) — v0.3.1 통합 배포 대기 (Sprint 6 + Sprint 7)
브랜치: `sprint7` → `develop` 머지 완료 (2026-05-22)
- ✅ sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증)
- ⬜ `pnpm tauri:dev` 실행하여 앱 동작 수동 확인 (UC-2 학사 일정 수립 전체 흐름 + T1~T9 시각 검증)

**Sprint 7 시각 검증 체크리스트 (sprint-review 단계)**:
- ⬜ T1: 비밀번호 입력 시 Keychain 다이얼로그 최대 1회, startup < 3초
- ⬜ T2: salt.bin 파일 생성 확인 (`smarthb/salt.bin`) + 기존 Keychain salt 자동 마이그레이션
- ⬜ T3: 앱 재시작 후 동일 device_id 유지 확인
- ⬜ T4: 캘린더 배지 색상 정상 (시스템 코드 하드코딩 제거 확인) + 시스템 코드 드래그 차단
- ⬜ T5: `/settings/schedule-codes` 코드 관리 CRUD 동작 + `/academic` 에서 코드 패널 제거 확인
- ⬜ T6: 교습기간 미확정 월에서 셀 클릭 즉시 selection 모드 진입
- ⬜ T7: 중복불가 일정 배치 상호 차단 + 교습기간 외 배치 차단 확인
- ⬜ T8: 교습기간 삭제 → cascade 삭제 → 공휴일 보존 확인
- ⬜ T9: 공휴일 배지 삭제 버튼 비표시 + 삭제 시도 차단 확인

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
