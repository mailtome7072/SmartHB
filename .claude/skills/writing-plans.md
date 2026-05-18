# writing-plans

스프린트 계획 문서(`docs/sprint/sprint{n}.md`)를 작성하는 표준 절차와 형식입니다.

## 계획 문서 작성 절차

### 1. 입력 참조
- `ROADMAP.md` — 해당 스프린트의 목표, 포함 기능 목록
- `strategy/planning.md` — 계획 수립 원칙
- 이전 스프린트 회고(`docs/sprint-retrospectives/`) — 액션 아이템 반영

### 2. 스프린트 번호 결정
- `ROADMAP.md`의 스프린트 목록에서 다음 번호를 결정한다. (SSOT)
- `.claude/agents/agent-memory/sprint-planner/MEMORY.md`는 교차 검증용으로만 참조한다. 비어있거나 없으면 ROADMAP.md만 사용한다. (SSOT: ROADMAP.md)

### 3. 문서 작성 형식

```markdown
# Sprint Plan sprint{n}

## 기간
{시작일} ~ {종료일} (예상)

## 목표
{스프린트의 핵심 목표 1~3문장}

## ROADMAP 연계 기능
- {ROADMAP.md에서 이번 스프린트가 다루는 기능 목록}

## 작업 목록

### 백엔드
- ⬜ {작업 A} — {담당/비고}
- ⬜ {작업 B}

### 프론트엔드
- ⬜ {작업 C}

### 인프라/기타
- ⬜ {작업 D}

## 이전 회고 반영
{직전 스프린트 회고의 액션 아이템 목록 — 첫 스프린트이거나 액션 아이템이 0개인 경우 이 섹션을 생략한다}
- {액션 아이템 1} → {이번 스프린트 반영 방법}

## 완료 기준 (Definition of Done)

**필수**
- ⬜ cargo test 전체 통과 (Rust 변경 시)
- ⬜ pnpm build 성공 (Next.js static export)
- ⬜ 코드 리뷰 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ DEPLOY.md 업데이트

## 참고 사항
- {특이사항, 의존성, 리스크}
```

#### skill: 선언 (선택)

Task 항목에 `skill:` 을 명시하면 `/sprint-dev` 실행 시 해당 Task 구현 전에 지정 스킬을 자동 로드합니다.

```markdown
- ⬜ {작업 A} — {담당/비고} · skill: systematic-debugging
- ⬜ {작업 B}                          ← skill: 없으면 자동 건너뜀
```

사용 가능한 스킬:

| 스킬 | 적합한 작업 유형 |
|------|----------------|
| `systematic-debugging` | 버그 수정, 원인 불명 오류 추적 |
| `karpathy-guidelines` | 복잡한 구현 Task의 원칙 재확인 |
| `code-review` | 중요 로직 자기 검토 |
| `test-checklist` | 테스트 작성 Task |

> **참고**: 모든 Task 완료 후 `simplify` 스킬이 **자동 실행**됩니다 (`skill:` 선언 불필요).

### 4. 작업 목록 작성 원칙
- 각 작업은 하루 이내에 완료 가능한 단위로 분리한다.
- 작업 간 의존 관계가 있으면 순서를 명시한다.
- DB 스키마 변경이 포함된 작업은 별도로 표시한다.
- 새 의존성(pip/npm 패키지) 추가가 필요한 작업은 별도로 표시한다.

**INVEST 기준으로 각 작업 항목을 검증한다.** (출처: Bill Wake, 2003 / Agile Alliance)

| 기준 | 의미 | 검증 질문 |
|------|------|-----------|
| **I**ndependent | 다른 작업에 의존하지 않음 | 이 작업만 단독으로 완료할 수 있는가? |
| **N**egotiable | 구현 방법은 유연하게 조정 가능 | 솔루션이 고정되어 있지 않은가? |
| **V**aluable | 사용자 또는 팀에 명확한 가치 제공 | 이 작업이 완료되면 누가 어떤 이익을 얻는가? |
| **E**stimable | 크기 추정 가능 | 작업 범위가 명확하여 시간을 예측할 수 있는가? |
| **S**mall | 하루 이내 완료 가능 | 하루 안에 끝낼 수 있는 단위인가? |
| **T**estable | 완료 기준이 명확 | "완료"를 Yes/No로 판단할 수 있는가? |

### 4.5 Capacity 확인

스프린트 계획 수립 후, 작업 총량이 팀의 실가용 시간 안에 들어오는지 점검한다.
- 총 작업 수 × 평균 소요 시간 ≤ (팀 인원 × 스프린트 일수 × 실작업 가능 시간/일)
- 팀 인원 미확정 시 2인으로 가정 (소규모 팀 기본값). 실작업 가능 시간 기본값: 하루 4시간
- 예상 작업량이 Capacity를 초과하면 작업을 다음 스프린트로 이월하거나 범위를 축소한다.
- Velocity 데이터가 있으면 (`agent-memory/sprint-planner/MEMORY.md` 참조) 과거 완료 기준으로 조정한다.

### 5. 완료 후
- `ROADMAP.md`에서 해당 스프린트 상태를 `⬜ 예정` → `🔄 진행 중`으로 업데이트한다.
- `.claude/agents/agent-memory/sprint-planner/MEMORY.md`의 스프린트 현황을 갱신한다.
