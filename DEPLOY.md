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

## 2026-05-28 | v0.5.0 | Sprint 9+10 (Phase 3 완결) + Hotfix 4건

### 포함 스프린트
- Sprint 9: 보강 등록(개별) + 보강-결석 매칭 + 취소/환원 + 결석 이력 + 시각 검증 25건 흡수 (I1~I8 + J1~J10 + K1~K7)
- Sprint 10: 소멸 자동 전이 (3개 트리거) + 퇴교 보강 처리 + 캘린더 뷰 (FullCalendar) + V108 FK 재구성
- Hotfix 4건: 퇴교 번복 결석 환원 + 퇴교 다이얼로그 UX + 번복 안내 문구 갱신 + absence_memo 클리어

### 배포 상태
- ✅ main merge 완료 (develop → main 직접 머지)
- ⬜ v0.5.0 태그 push → GitHub Actions 빌드 완료
- ⬜ GitHub Release 아티팩트 업로드 확인
  - ⬜ Windows: .msi 또는 .exe
  - ⬜ macOS: .dmg

### CV — 아티팩트 검증
- ⬜ gh release view v0.5.0 으로 Release 확인
- ⬜ 다운로드 URL 유효성 확인
- ⬜ 인스톨러 설치 테스트 (수동, 선택)

이전 배포 기록: `docs/deploy-history/2026-05-28.md` (Sprint 10 스테이징 기록)

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
