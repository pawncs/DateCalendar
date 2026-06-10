import { useState, useEffect } from 'react'
import { Plus, Trash2, AlertTriangle, Shield } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Select } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { useTaskStore } from '@/stores/taskStore'
import type { MilestoneRisk, RiskProbability } from '@/types/task'

const riskProbColors: Record<RiskProbability, string> = {
  low: 'bg-green-500/20 text-green-400 border-green-500/30',
  medium: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30',
  high: 'bg-red-500/20 text-red-400 border-red-500/30',
}

interface MilestonePanelProps {
  taskId: string
}

export function MilestonePanel({ taskId }: MilestonePanelProps) {
  const { loadRisks, addRisk, deleteRisk } = useTaskStore()
  const [risks, setRisks] = useState<MilestoneRisk[]>([])
  const [showAdd, setShowAdd] = useState(false)
  const [riskDesc, setRiskDesc] = useState('')
  const [probability, setProbability] = useState<RiskProbability>('medium')
  const [mitigation, setMitigation] = useState('')

  useEffect(() => {
    loadRisks(taskId).then(setRisks)
  }, [taskId])

  const handleAdd = async () => {
    if (!riskDesc.trim()) return
    const risk = await addRisk(taskId, riskDesc.trim(), probability, mitigation.trim())
    setRisks([...risks, risk])
    setRiskDesc('')
    setMitigation('')
    setProbability('medium')
    setShowAdd(false)
  }

  const handleDelete = async (riskId: string) => {
    await deleteRisk(riskId)
    setRisks(risks.filter((r) => r.id !== riskId))
  }

  return (
    <div className="p-4 space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-xs text-muted-foreground">里程碑风险记录</span>
        <Button variant="ghost" size="sm" onClick={() => setShowAdd(!showAdd)}>
          <Plus className="size-3.5" />
          添加
        </Button>
      </div>

      {showAdd && (
        <div className="space-y-3 p-3 rounded-lg border border-border bg-card">
          <div>
            <label className="text-xs text-muted-foreground mb-1 block">风险描述</label>
            <Input
              value={riskDesc}
              onChange={(e) => setRiskDesc(e.target.value)}
              placeholder="描述可能的风险..."
              className="text-sm"
            />
          </div>
          <div>
            <label className="text-xs text-muted-foreground mb-1 block">可能性</label>
            <Select
              value={probability}
              onChange={(v) => setProbability(v as RiskProbability)}
              className="w-full"
            >
              <option value="low">低</option>
              <option value="medium">中</option>
              <option value="high">高</option>
            </Select>
          </div>
          <div>
            <label className="text-xs text-muted-foreground mb-1 block">应对措施</label>
            <Textarea
              value={mitigation}
              onChange={(e) => setMitigation(e.target.value)}
              placeholder="如何应对这个风险..."
              className="text-sm min-h-[60px]"
            />
          </div>
          <div className="flex gap-2">
            <Button size="sm" onClick={handleAdd}>确认添加</Button>
            <Button size="sm" variant="ghost" onClick={() => setShowAdd(false)}>取消</Button>
          </div>
        </div>
      )}

      {risks.length === 0 ? (
        <div className="text-center py-8 text-sm text-muted-foreground">
          <Shield className="size-8 mx-auto mb-2 opacity-50" />
          <p>暂无风险记录</p>
        </div>
      ) : (
        <div className="space-y-2">
          {risks.map((risk) => (
            <div
              key={risk.id}
              className="p-3 rounded-lg border border-border bg-card space-y-2"
            >
              <div className="flex items-start justify-between gap-2">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <AlertTriangle className="size-3.5 text-yellow-400" />
                    <span className="text-sm font-medium">{risk.risk_desc}</span>
                  </div>
                  <Badge className={riskProbColors[risk.probability as RiskProbability]}>
                    {risk.probability === 'low' ? '低风险' : risk.probability === 'medium' ? '中风险' : '高风险'}
                  </Badge>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  className="size-6 shrink-0"
                  onClick={() => handleDelete(risk.id)}
                >
                  <Trash2 className="size-3 text-red-400" />
                </Button>
              </div>
              {risk.mitigation && (
                <p className="text-xs text-muted-foreground pl-6">
                  应对: {risk.mitigation}
                </p>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
