import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'SmartHB',
  description: '정쌤의 스마트해법수학',
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="ko">
      <body>{children}</body>
    </html>
  )
}
