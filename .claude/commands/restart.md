Tauri 개발 서버를 재시작한다.

## 사용 방법

`$ARGUMENTS`를 기준으로 재시작할 대상을 결정한다:

- 인수 없음 또는 `all`: Tauri + Next.js dev server 전체 재시작
- `frontend`: Next.js dev server만 재시작 (브라우저 테스트용)
- `tauri`: Tauri 앱만 재기동

## 실행 절차

1. 현재 실행 중인 개발 서버를 종료한다 (사용자가 직접 터미널에서 Ctrl+C).

2. 아래 명령으로 재시작한다:

**전체 재시작 (Tauri + Next.js):**
```bash
pnpm tauri dev
```

**프론트엔드만 (브라우저 테스트):**
```bash
pnpm dev
```

3. 정상 기동 확인:
- `pnpm tauri dev`: 데스크톱 앱 창이 열리면 정상
- `pnpm dev`: `http://localhost:3000` 응답 확인

4. 결과를 사용자에게 간결하게 보고한다.
