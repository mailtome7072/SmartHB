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

## 2026-07-23 | v1.5.0 | Sprint 23 — 프로덕션 데이터 소실 사고 재발방지 (ADR-012 A안)

브랜치: `sprint23 → develop` 직접 머지 (단일 개발자 정책)

### 스테이징 검증 (develop 로컬)
- ✅ sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증)
- ⬜ pnpm tauri:dev 실행하여 앱 동작 수동 확인 — 유휴 close 후 재연결 정상 동작 확인
- ⬜ 2번째 PC(자택 Mac 또는 별도 환경)에서 try_adopt_key 흐름 확인 — PIN 입력 → 키 유도 → DB 열기 성공
- ⬜ create_if_missing 가드 동작 확인 — salt.bin 있고 app.db 없을 때 앱이 fail-hard 종료하는지 확인
- ⬜ 복원 다계층 폴백 동작 확인 — exit 백업 소스 검증 + 순환 삭제 시 마지막 정상 백업 보존 확인
- ⬜ v1.4.0 → v1.5.0 무중단 업그레이드 확인 — 기존 실 DB(V312) 그대로 기동 정상 여부
- ⬜ 교습일정 인쇄 미리보기 확인 (Sprint 20 A122 계속 유지)

### 프로덕션 배포 (master 머지 + v태그 push)
- ✅ 버전 파일 3곳 동기화 확인 — package.json / src-tauri/Cargo.toml / src-tauri/tauri.conf.json 모두 `1.5.0` (커밋 21dacf3)
- ⬜ develop → master 직접 머지
- ⬜ v1.5.0 태그 push → GitHub Actions 빌드 완료
- ⬜ GitHub Release 아티팩트 업로드 확인
  - ⬜ Windows: SmartHB_1.5.0_x64-setup.exe
  - ⬜ macOS: SmartHB_1.5.0_aarch64.dmg

### CV — 아티팩트 검증
- ⬜ gh release view v1.5.0 으로 Release 확인
- ⬜ 원장님 PC 인스톨러 설치 — v1.4.0 → v1.5.0 업그레이드 후 기존 데이터 정상 로드 확인
- ⬜ after_connect 훅 PRAGMA key 재적용 동작 확인 (실 DB 기준)

이전 배포 기록: `docs/deploy-history/2026-07-22.md` (v1.4.0 Sprint 22 프로덕션 + Sprint 22 스테이징 아카이빙)

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
