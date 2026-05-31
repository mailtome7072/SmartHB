---
name: sprint-next-session
description: "Sprint 12(공지문) 구현 중 — sprint12 브랜치 원격 push 완료. 회사 PC에서 수동 검수 이어가는 중"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint12-notice-relay-2026-05-31
---

**현재 위치(2026-05-31)**: **Sprint 12 = 카카오톡 교습비 공지문 이미지 생성**(PRD §4.10) 구현 중.
브랜치 **`sprint12`** (develop 기반), **origin 에 push 완료** — 다른 PC에서 이어받기 가능. 현재 **수동 검수(시각 검증) 진행 중**, 아직 sprint-close 전.

## 릴레이 시작 (회사 PC 등 새 환경)
1. `git fetch origin && git checkout sprint12` (또는 `git checkout -b sprint12 origin/sprint12`)
2. **`pnpm install`** — 신규 의존성 `html-to-image`, `react-rnd` 받기 (package.json 반영됨)
3. `pnpm tauri:dev` — 앱 시작 시 마이그레이션 자동 적용(V111 포함). DB·배경서식·저장 템플릿은 클라우드 동기화 폴더(`smarthb/`)에 있어 자동 공유됨.
4. **PIN**: 키체인 키는 PC별이라 회사 PC 첫 실행 시 PIN 입력 필요(salt.bin 은 클라우드 동기화됨 → 같은 PIN 으로 잠금 해제). dev 자격증명은 2026-05-31 재설정됨.
5. `.claude/memory/` 미러는 sprint12 에 커밋돼 있어 컨텍스트 동기화됨.

## Sprint 12 구현 현황 (공지문 `/notices`, `notice.rs`)
- 백엔드 `commands/notice.rs`: 배경서식(assets) CRUD, 레이아웃 저장(working + 이름 템플릿), 이미지 저장(output PNG), 월 정보(교습기간·보강데이) IPC. paths: assets_dir/notice_output_dir.
- 프론트 `/notices`: 좌(원생 리스트 240px) · 중(편집 캔버스) · 우(저장 템플릿 패널 220px).
  - 텍스트박스 = **배경 원본 해상도 비율** 저장, 폰트=박스높이×fontRatio 자동, react-rnd 드래그/리사이즈(scale 보정).
  - 데이터 필드: 청구월/교습기간/보강데이/원생명/청구액 체크박스(체크 시 표시) + custom 텍스트박스('+텍스트박스 추가', 더블클릭 인라인 편집).
  - 교습기간 = 수업 가능 첫/마지막 일자(운영요일+공휴/휴원 제외). 보강데이 = 'D(요일) 10시~13시'.
  - 저장 패널: 공지문 이름 입력(디폴트 없음) → '공지문 저장'(동명 시 덮어쓰기 확인 모달), 템플릿 목록(이름 내림차순, 클릭 로드, ✕ 삭제). 편집 중 템플릿 삭제 시 체크해제+이름비움 초기화.
  - window.confirm/prompt → 커스텀 모달(Tauri 호환). 성공 메시지는 토스트(오류만 ErrorDialog).
- 생성: html-to-image 로 원생별 PNG → `output/{YYYYMM}/{YYYYMM}_{원생명}.png`, 천단위 콤마(AC-4.10-1), 덮어쓰기 확인(AC-4.10-2).

## 전역 변경(이번 세션, 회귀 주의)
- **QueryClient**: staleTime 0 + refetchOnMount 'always' + refetchOnWindowFocus true — 메뉴 이동 시 즉시 최신 반영(이전 30초 캐시 staleness 해소).

## 다음 단계
1. 공지문 수동 검수 완료 → DEPLOY.md 항목 정리.
2. **sprint-close → sprint-review** (코드리뷰+검증+회고). Sprint 12 DoD/AC(4.10-1/2/3) 전수 마킹.
3. develop 머지(직접, PR 생략 [[workflow-no-pr]]).

## 이후 예정/결정
- [[sprint13-pin-optional]] — 실행 시 PIN 인증 옵션화(C안) Sprint 13.
- [[exam-feature-cancelled]] — 단원평가+학습보고서(Phase 5 전체) 취소 → 다음 계획 시 제외.

## 정책
- PR 생략, 직접 머지 ([[workflow-no-pr]]). 메모리 추가/수정 시 사용자 메모리 + `.claude/memory/` 양쪽 갱신 후 commit.
