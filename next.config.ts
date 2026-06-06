import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  output: 'export',
  images: {
    unoptimized: true,
  },
  // Tauri IPC: 개발 서버를 0.0.0.0 대신 localhost로 제한
  ...(process.env.TAURI_DEV_HOST
    ? {
        experimental: {
          serverActions: {
            allowedOrigins: [process.env.TAURI_DEV_HOST, 'localhost'],
          },
        },
      }
    : {}),
  // dev 서버 전용: webpack 파일시스템 캐시 비활성화.
  // Node 25 등 최신 V8 에서 webpack 캐시 직렬화(SerializerMiddleware → node:buffer.byteLength
  // fast API call) 가 "Lazy deopt after a fast API call ..." abort 로 dev 서버를 크래시시키는
  // 문제를 회피한다. production 빌드(`next build`, output:'export')에는 영향 없음 — dev 콜드
  // 리컴파일이 약간 느려지는 것이 유일한 트레이드오프.
  webpack: (config, { dev }) => {
    if (dev) {
      config.cache = false
    }
    return config
  },
}

export default nextConfig
