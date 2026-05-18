# Phase 1 보안 전문가 검토

> 검토 대상: Phase 1 (인프라 + 기반 도메인, Sprint 1~3)
> 검토일: 2026-05-18
> 관점: SQLCipher 키 관리, 복구 코드, 백업 무결성, 인증 흐름

---

## 1. SQLCipher 키 메모리 관리

### 현황
- 사용자 비밀번호 → PBKDF2 유도 → SQLCipher 키 → DB 복호화
- 키는 OS Keychain/Credential Manager에 보관

### 권고사항

| 등급 | 항목 | 설명 |
|------|------|------|
| **필수** | 키 zeroize | DB 열기 직후 메모리 상의 키 바이트를 즉시 zeroize. `zeroize` crate 사용 필수 |
| **필수** | PBKDF2 파라미터 | 최소 600,000 iterations (OWASP 2024 권장). salt는 32바이트 랜덤 |
| **필수** | 키 로깅 금지 | Debug/Trace 로그에 키 바이트, 비밀번호, 복구 코드가 출력되지 않도록 `Secret<T>` 래퍼 또는 `Debug` trait 수동 구현 |
| 권고 | Keychain 접근 보호 | macOS: `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` 속성. Windows: DPAPI 기반 Credential Manager 기본 보호 수준 활용 |

### 위협 시나리오
- 메모리 덤프에서 키 추출: zeroize로 완화
- Keychain에서 다른 앱이 키 접근: 앱 번들 ID 기반 접근 제어 (Tauri 서명 필수)

---

## 2. 복구 코드 (PI-07)

### 현황 (PRD v1.5.1 결정)
- 설정 메뉴에서 사용자 요청 시 1회 발급
- 재발급: 기존 코드 무효화 + 새 코드 (회수 제한 없음)
- 해시만 DB 보관, 평문은 1회 표시 후 메모리 폐기

### 권고사항

| 등급 | 항목 | 설명 |
|------|------|------|
| **필수** | 해시 알고리즘 | **Argon2id** 사용 (PBKDF2보다 메모리-하드, GPU/ASIC 무차별 대입 저항). `argon2` crate |
| **필수** | 복구 코드 생성 | 12자리 영숫자, `OsRng`(cryptographically secure random) 사용. 혼동 문자 제거 (0/O, 1/l/I) |
| **필수** | 평문 zeroize | 화면 표시 후 메모리에서 즉시 zeroize |
| **필수** | 재발급 시 비밀번호 재입력 | PRD 원문 준수 — 비밀번호 미입력 시 재발급 차단 |
| 권고 | 분실 경고 | "비밀번호와 복구 코드 모두 분실 시 데이터에 영구 접근 불가합니다" 경고를 발급 화면 + 설정 화면에 명시 |

### 복구 코드 → 비밀번호 재설정 흐름
```
복구 코드 입력 → Argon2id 해시 비교 → 성공 시:
  1. 새 비밀번호 입력 받기
  2. 새 PBKDF2 키 유도
  3. SQLCipher rekey 실행 (DB 키 변경)
  4. Keychain에 새 키 저장
  5. 기존 복구 코드 해시 유지 (재발급 별도)
```

**주의**: SQLCipher `rekey`는 전체 DB 재암호화를 수반 — DB 크기에 비례한 시간 소요 (5MB ~2초). 진행 표시 필요.

---

## 3. 백업 파일 무결성

### 현황
- rusqlite::backup → SQLCipher 암호화 상태 그대로 복사
- 4계층: exit(10) + hourly(24) + daily(30) + weekly(4)

### 권고사항

| 등급 | 항목 | 설명 |
|------|------|------|
| **필수** | 백업 후 무결성 검증 | 백업 생성 직후 백업 파일을 읽기 전용으로 열어 `PRAGMA quick_check` 실행 |
| **필수** | 복원 전 무결성 검증 | 복원 대상 백업 파일의 `PRAGMA quick_check` 통과 확인 후 복원 진행 |
| 권고 | 백업 파일 해시 | 백업 파일 생성 시 SHA-256 해시를 `backup/manifest.json`에 기록. 복원 시 해시 비교로 변조 감지 |
| 권고 | 복원 롤백 | 복원 직전 현재 DB를 `restore_rollback/`에 보존 (PRD 원문 준수) |

---

## 4. app.lock 보안

### 권고사항

| 등급 | 항목 | 설명 |
|------|------|------|
| 권고 | 디바이스 ID | 랜덤 UUID v4 사용. MAC 주소, 하드웨어 시리얼, 사용자 이름 등 개인정보 금지 |
| 권고 | 락 파일 내용 최소화 | `{"device_id": "uuid", "last_heartbeat": "ISO8601"}` 만 포함. 앱 버전, OS 정보 불필요 |

---

## 5. 감사 로그

### 권고사항

| 등급 | 항목 | 설명 |
|------|------|------|
| **필수** | 민감 데이터 마스킹 | audit_logs의 before/after JSON에 비밀번호, 복구 코드 해시 등이 포함되지 않도록 필터링 |
| 권고 | 1년 롤링 보관 | PRD §7.3 준수 — 1년 이상 로그 자동 삭제 배치 (앱 시작 시 또는 일일 백업 시점) |

---

## 6. 종합 보안 체크리스트 (Sprint 1 완료 시 확인)

- ⬜ SQLCipher 키 zeroize 구현 확인
- ⬜ PBKDF2 iterations >= 600,000
- ⬜ 복구 코드 Argon2id 해시 저장 확인
- ⬜ 비밀번호/키/복구 코드 로그 출력 없음 확인
- ⬜ Keychain 접근 제어 속성 설정 확인
- ⬜ 백업 파일 암호화 상태 유지 확인
- ⬜ audit_logs 민감 데이터 마스킹 확인
