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

## 2026-05-22 | v0.2.1 | Hotfix/nextjs-cve-2025-66478

### 핫픽스 내용
- CVE-2025-66478: Next.js 15.3.2 → 15.3.6 보안 패치 (예방적 업그레이드)
- `eslint-config-next` 동반 업그레이드
- `src-tauri/Cargo.lock` v0.2.0 stale sync 정리

### 자동 검증 결과
- ✅ cargo test: 97 passed (self-verify) + 재실행 pass
- ✅ cargo clippy: warnings 0건
- ✅ pnpm tsc --noEmit: 타입 오류 0건
- ✅ pnpm lint: No ESLint warnings or errors
- ✅ pnpm build: static export 3 pages 성공 (self-verify)

### 배포 상태
- ✅ master 직접 머지 (hotfix/nextjs-cve-2025-66478 → master --no-ff, 머지 커밋 `735dfad`)
- ✅ v0.2.1 태그 push → GitHub Actions 빌드 트리거 완료
- ⬜ GitHub Release 아티팩트 업로드 확인 (Windows .msi/.exe / macOS .dmg)
- ✅ develop 역머지 완료 (master → develop, Next.js 15.3.6 동기화)

### 수동 검증 필요 항목
- ⬜ `pnpm tauri:dev` 실행 후 앱 정상 기동 확인 (Next.js 15.3.6 런타임 이상 없음)
- ⬜ GitHub Release 페이지에서 v0.2.1 Release 생성 확인

---

### Sprint 4 (2026-05-21)
- ✅ sprint-close 에이전트 실행 (ROADMAP + CHANGELOG + DEPLOY 갱신 + develop 머지)
- ✅ sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증)
- ⬜ `pnpm tauri:dev` 실행하여 앱 동작 수동 확인 (스테이징 검증)

#### 14개 이슈 재검증 (사용자 시각 확인 완료 — 2026-05-21)
- ✅ #0 Critical: window.confirm 차단 해소 + shadcn AlertDialog 정상 표시
- ✅ #1 교습소 설정 메뉴 화면 신설 — 운영 시간 편집 가능
- ✅ #2 상태바 점유/백업/동기화/시작시간 IPC 연결 및 표시
- ✅ #3 원생 폼 학교명 Select 연동 + 학교 필터 정상 동작
- ✅ #4 연락처 자동 하이픈 포맷 적용
- ✅ #5 금액 천단위 콤마 적용
- ✅ #6 일련번호 readonly 보호 (수정 차단)
- ✅ #7 원생 등록 후 스케줄 등록 안내 UX 표시
- ✅ #8 퇴교일 필드 표시 + 퇴교 번복 기능
- ✅ #9 수업 스케줄 시작시간 콤보박스 (운영시간 내 1시간 단위)
- ✅ #10 수업 스케줄 수정/삭제 기능
- ✅ #11 코드 테이블 DnD 순서 변경
- ✅ #12 코드 테이블 활성 상태 필터
- ✅ #13 신규 코드 sort_order 자동 부여

#### post-T11 추가 4건 (사용자 시각 확인 완료 — 2026-05-21)
- ✅ 원생 목록 주총 수업시간 + 수업 요일 컬럼 추가
- ✅ 스케줄 폼 위치 개선
- ✅ 스케줄 시작시간 1시간 단위 + 운영시간 디폴트
- ✅ 스케줄 번호 요일순 정렬

---

## 참고

- 검증 매트릭스 (수동 항목 기준): `docs/dev-process.md` 섹션 5
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
- 배포 이력 아카이브: `docs/deploy-history/`
- 롤백 방법: `docs/dev-process.md` 섹션 6.4
