'use client'

/**
 * 대시보드 차트 (Sprint 14 T4, PRD §4.11.1).
 *
 * recharts 사용. **static export 안전성**: 본 모듈은 `next/dynamic` 의 `ssr: false` 로만
 * 로드되어야 한다 (recharts ResponsiveContainer 는 브라우저 DOM 의존). 다른 라우트 번들에
 * 포함되지 않도록 대시보드에서 동적 import (R96).
 */

import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Line,
  LineChart,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import type { AcademyOverview, BillingTrendPoint } from '@/types/dashboard'

// 저자극 무채도 기반 팔레트 (PRD §5.7).
const GENDER_COLORS = ['#6b8cae', '#c08497']
const BAR_COLOR = '#7a8aa0'
const ENROLL_COLOR = '#5b8c6e'
const WITHDRAW_COLOR = '#c08497'
const TREND_COLOR = '#5b7aa0'

export function OverviewCharts({ overview }: { overview: AcademyOverview }) {
  return (
    <div className="grid gap-6 sm:grid-cols-2">
      <ChartBlock title="성별 분포">
        <ResponsiveContainer width="100%" height={180}>
          <PieChart>
            <Pie
              data={overview.by_gender}
              dataKey="count"
              nameKey="label"
              cx="50%"
              cy="50%"
              innerRadius={40}
              outerRadius={70}
            >
              {overview.by_gender.map((_, idx) => (
                <Cell key={idx} fill={GENDER_COLORS[idx % GENDER_COLORS.length]} />
              ))}
            </Pie>
            <Tooltip />
            <Legend />
          </PieChart>
        </ResponsiveContainer>
      </ChartBlock>

      <ChartBlock title="학년 분포">
        <ResponsiveContainer width="100%" height={180}>
          <BarChart data={overview.by_grade}>
            <CartesianGrid strokeDasharray="3 3" vertical={false} />
            <XAxis dataKey="label" fontSize={12} />
            <YAxis allowDecimals={false} fontSize={12} />
            <Tooltip />
            <Bar dataKey="count" fill={BAR_COLOR} radius={[4, 4, 0, 0]} />
          </BarChart>
        </ResponsiveContainer>
      </ChartBlock>

      <ChartBlock title="분기별 입·퇴교 추이">
        <ResponsiveContainer width="100%" height={200}>
          <LineChart data={overview.quarterly}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="label" fontSize={11} />
            <YAxis allowDecimals={false} fontSize={12} />
            <Tooltip />
            <Legend />
            <Line type="monotone" dataKey="enrolled" name="입교" stroke={ENROLL_COLOR} strokeWidth={2} />
            <Line type="monotone" dataKey="withdrawn" name="퇴교" stroke={WITHDRAW_COLOR} strokeWidth={2} />
          </LineChart>
        </ResponsiveContainer>
      </ChartBlock>

      <ChartBlock title="학교 분포">
        <ResponsiveContainer width="100%" height={200}>
          <BarChart data={overview.by_school} layout="vertical" margin={{ left: 20 }}>
            <CartesianGrid strokeDasharray="3 3" horizontal={false} />
            <XAxis type="number" allowDecimals={false} fontSize={12} />
            <YAxis type="category" dataKey="label" width={90} fontSize={12} />
            <Tooltip />
            <Bar dataKey="count" fill={BAR_COLOR} radius={[0, 4, 4, 0]} />
          </BarChart>
        </ResponsiveContainer>
      </ChartBlock>
    </div>
  )
}

function ChartBlock({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h3 className="mb-2 text-sm font-bold text-gray-700">{title}</h3>
      {children}
    </div>
  )
}

/** 월별 청구총액 증감 추이 (최근 12개월). Y축 만원 단위 표기, 툴팁은 원 단위. */
export function BillingTrendChart({ data }: { data: BillingTrendPoint[] }) {
  return (
    // height 100% 로 위젯 flex 영역을 채우고, 작은 화면(부모 높이 미정)에선 minHeight 로 fallback.
    <ResponsiveContainer width="100%" height="100%" minHeight={180}>
      <LineChart data={data} margin={{ left: 4, right: 8, top: 4 }}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="year_month" tickFormatter={(v: string) => v.slice(5)} fontSize={11} />
        <YAxis
          tickFormatter={(v: number) => `${Math.round(v / 10000)}만`}
          fontSize={11}
          width={48}
        />
        <Tooltip
          formatter={(v) => [`${Number(v).toLocaleString('ko-KR')}원`, '청구총액']}
          labelFormatter={(l) => `${l}`}
        />
        <Line
          type="monotone"
          dataKey="total"
          name="청구총액"
          stroke={TREND_COLOR}
          strokeWidth={2}
          dot={{ r: 2 }}
        />
      </LineChart>
    </ResponsiveContainer>
  )
}
