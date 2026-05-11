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
}

export default nextConfig
