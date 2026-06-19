/**
 * 日期工具函数
 *
 * 遵循项目约定：日期字符串统一使用 ISO 格式 "yyyy-MM-dd"
 * 通过 new Date().toISOString().split('T')[0] 生成
 */

/** 获取今天的 ISO 日期字符串 "yyyy-MM-dd" */
export function getTodayStr(): string {
  return new Date().toISOString().split('T')[0]
}

/** 获取指定日期所在周的周一和周日 */
export function getWeekRange(dateStr: string): { weekStart: string; weekEnd: string } {
  const d = new Date(dateStr)
  const day = d.getDay() // 0=Sun, 1=Mon, ...
  const diffToMon = day === 0 ? -6 : 1 - day
  const diffToSun = day === 0 ? 0 : 7 - day

  const mon = new Date(d)
  mon.setDate(d.getDate() + diffToMon)
  const sun = new Date(d)
  sun.setDate(d.getDate() + diffToSun)

  return {
    weekStart: mon.toISOString().split('T')[0],
    weekEnd: sun.toISOString().split('T')[0],
  }
}
