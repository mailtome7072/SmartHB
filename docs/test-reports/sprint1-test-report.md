# Test Report — Sprint 1 (2026-05-19)

## 자동 검증 결과

| 검증 항목 | 결과 | 비고 |
|-----------|------|------|
| `cargo test --lib` | 통과 | 74 passed, 0 failed (13.31s) |
| `cargo clippy --all-targets -- -D warnings` | 통과 | 경고 없음 |
| `pnpm tsc --noEmit` | 통과 | 타입 오류 없음 |
| `pnpm lint` | 통과 | ESLint 경고/오류 없음 |
| `pnpm build` (static export) | 통과 | 3 routes 정상 생성 (/, /_not-found, /lock) |

## 수동 검증 항목 (DEPLOY.md 기준)

| 항목 | 상태 |
|------|------|
| `pnpm tauri:dev` 로컬 스테이징 실행 | ⬜ 미완료 (개발자 수행 필요) |
| `sqlx migrate run` V001/V008 적용 확인 | ⬜ 미완료 |
| 첫 비밀번호 설정 화면 + `set_password` IPC 동작 | ⬜ 미완료 |
| 복구 코드 발급 (12자리) 확인 | ⬜ 미완료 |
| 앱 재시작 후 `unlock_db` 잠금 해제 확인 | ⬜ 미완료 |
| `app.lock` 파일 생성 확인 | ⬜ 미완료 |
| 앱 종료 후 `backup/exit/` 백업 파일 생성 확인 | ⬜ 미완료 |
| PRAGMA quick_check 통과 + audit 로그 기록 확인 | ⬜ 미완료 |
| 앱 시작 ~ 메인 화면 < 3초 측정 (PRD §5.6, cipher off) | ⬜ 미완료 |
| UI 디자인/시각적 품질 확인 (Pretendard, 18pt+) | ⬜ 미완료 |

## 결론

자동 검증 5개 항목 모두 통과. 수동 검증은 cipher on 빌드 실환경에서 개발자가 수행해야 한다.
Critical/High 이슈 없음. Medium 이슈 3건은 `docs/risk-register/sprint1-risks.md`에 기록.
