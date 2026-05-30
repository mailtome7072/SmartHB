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

### Sprint 11 (2026-05-29) + post-Sprint 11 develop 보완 (2026-05-30)

Phase 4 첫 마일스톤 — 청구+수납 도메인 완성 + 검수 후 보완 2건(커밋 `945e4a7`, `c93399e`).

- ✅ sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증)
- ✅ sprint11 → develop 직접 머지 (단일 개발자 정책)
- ⬜ sprint-review 에이전트 재실행 (post-Sprint 11 보완 2건 대상)
- ⬜ pnpm tauri:dev 실행하여 앱 동작 수동 확인 (스테이징 검증)
  - ⬜ `/billing` 페이지 진입 → "청구 데이터 생성" 버튼 → 청구 목록/금액 확인
  - ⬜ 청구 탭 상태 필터 — 전체/확정/미확정/마감 건수 표기 및 '마감 완료' 배지 위치 확인
  - ⬜ 개별 확정 / "일괄 확정" / "당월 청구 마감" 흐름
  - ⬜ 수납 탭 필터 건수 표기 (전체/수납완료/미수납) 확인
  - ⬜ 수납 탭 진입 → 입금 일괄 처리 (결제수단 미선택 시 빨간 테두리 오류 표시 확인)
  - ⬜ 마감 후 금액 수정 시 사유 다이얼로그 (비어있지 않으면 통과) 동작 확인
  - ⬜ 수납완료 행 → 수납 취소 버튼 동작 확인
  - ⬜ 수납완료된 마감 청구 금액 편집 비활성 확인
  - ⬜ 잠금 화면 PIN 6자리 숫자 입력 전환 동작 확인 (기존 비밀번호는 dev 환경 재설정 필요)
  - ⬜ 사이드 메뉴 '보강 관리' 항목 미노출 확인 (F7)

이전 배포 기록: `docs/deploy-history/2026-05-28.md` (Sprint 9+10 + Hotfix 4건)

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
