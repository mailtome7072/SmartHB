# Risk Register — Sprint 1 코드 리뷰 결과 (2026-05-19)

> sprint-review 코드 리뷰에서 발견된 Medium 이슈. 기존 R1~R5 (`2026-05-18.md`)에 추가.

| ID | 설명 | 영향도 | 출처 | 대응 계획 |
|----|------|--------|------|-----------|
| R6 | `KEYRING_USER_SALT` Keychain 저장 — salt는 비밀이 아니나 Keychain에 보관 중. 양 PC 동기화 시 동일 항목을 공유해야 하는 구조적 한계. 초기 설정 마법사 미통합 시 멀티디바이스 salt 불일치 위험 | 중간 | sprint-review 코드 리뷰 (`auth.rs:51`) | Sprint 2 이후 초기 설정 마법사 구현 시 salt를 클라우드 동기화 폴더의 평문 파일(`smarthb/salt.bin`)로 이전. Sprint 계획에 명시적 작업 항목으로 포함 |
| R7 | `release_lock` advisory lock 미적용 — read_lock_info 후 remove_file 사이에 fs2 lock 없음. 극단적 타이밍에서 다른 디바이스 락 삭제 가능 | 낮음 | sprint-review 코드 리뷰 (`lock.rs:240-253`) | 단일 사용자 모델이므로 실제 발생 가능성은 미미. 후속 Sprint에서 release에도 advisory lock 획득 후 삭제하도록 개선. 현재는 수용 |
| R8 | cipher on 실측 미수행 — 단위 테스트는 모두 cipher off(인메모리) 환경. cipher on 빌드에서 시작 시퀀스 < 3초 목표 달성 여부 미검증 | 중간 | sprint-review, sprint1.md 알려진 사항 | DEPLOY.md `⬜ 앱 시작 ~ 메인 화면 < 3초 측정` 항목으로 위임. 사용자 환경(Windows + SQLCipher on)에서 첫 실행 시 측정 필요 |
