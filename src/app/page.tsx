'use client'

import { useState } from 'react'
import { greet } from '@/lib/tauri'

export default function Home() {
  const [message, setMessage] = useState('')

  const handleGreet = async () => {
    const result = await greet('SmartHB')
    setMessage(result)
  }

  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <h1 className="text-4xl font-bold mb-8">스마트해법수학</h1>
      <p className="text-lg text-gray-600 mb-8">정쌤의 교습소 관리 시스템</p>
      <button
        onClick={handleGreet}
        className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
      >
        시작하기
      </button>
      {message && (
        <p className="mt-4 text-green-600">{message}</p>
      )}
    </main>
  )
}
