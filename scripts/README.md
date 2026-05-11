# scripts/

개발 및 운영 중 필요한 **수동 유틸리티 스크립트**를 보관하는 폴더입니다.

---

## CI/CD와의 역할 구분

| 위치 | 역할 |
|------|------|
| `.github/workflows/` | CI/CD 자동화 — 테스트, 빌드, 프로덕션 배포 |
| `SETUP.sh` | 개발 환경 최초 초기화 (1회성) |
| `scripts/` | CI/CD로 자동화할 수 없는 수동 유틸리티 스크립트 |

---

## 이 폴더에 추가하는 스크립트 기준

다음 조건 중 하나라도 해당하면 `scripts/`에 추가합니다.

- 개발자가 직접 실행해야 하는 1회성 또는 수동 작업
- Docker 컨테이너 내부에서 실행하는 백엔드 유틸리티
- CI/CD 파이프라인에 포함되지 않는 보조 작업

**해당하지 않는 경우** (이 폴더에 추가하지 않음):
- 배포 자동화 → `.github/workflows/deploy.yml`
- 테스트 실행 → `pytest` / `pnpm test` (CI에서 실행)
- 개발 환경 초기화 → `SETUP.sh`

---

## 스크립트 추가 예시

| 파일명 | 용도 | 실행 방법 |
|--------|------|-----------|
| `seed.py` | 초기 데이터 시드 | `docker compose exec backend python scripts/seed.py` |
| `reset_db.sh` | 개발용 DB 초기화 | `bash scripts/reset_db.sh` |
| `generate_fixtures.py` | 테스트 픽스처 생성 | `docker compose exec backend python scripts/generate_fixtures.py` |

> 스크립트를 추가할 때는 이 표에 항목을 함께 추가하세요.

---

## 스크립트 추가 시점

- 스프린트 진행 중 필요가 생길 때 개발자가 직접 추가합니다.
- 추가 후 이 `README.md`의 예시 표를 업데이트합니다.
- `docs/setup-guide.md`에서 실행이 필요한 경우 해당 섹션에도 안내를 추가합니다.
