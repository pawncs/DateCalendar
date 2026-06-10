import { useState, useEffect } from 'react'
import { Plus, Trash2, FileText, Save } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { useTaskStore } from '@/stores/taskStore'
import type { Note } from '@/types/task'

interface NoteEditorProps {
  taskId: string
}

export function NoteEditor({ taskId }: NoteEditorProps) {
  const { loadNotes, saveNote, deleteNote } = useTaskStore()
  const [notes, setNotes] = useState<Note[]>([])
  const [editingNoteId, setEditingNoteId] = useState<string | null>(null)
  const [editTitle, setEditTitle] = useState('')
  const [editContent, setEditContent] = useState('')
  const [showNew, setShowNew] = useState(false)

  useEffect(() => {
    loadNotes(taskId).then(setNotes)
  }, [taskId])

  const handleSaveNew = async () => {
    if (!editTitle.trim()) return
    const note = await saveNote(taskId, null, editTitle.trim(), editContent.trim())
    setNotes([note, ...notes])
    setEditTitle('')
    setEditContent('')
    setShowNew(false)
  }

  const handleSaveEdit = async () => {
    if (!editingNoteId || !editTitle.trim()) return
    const note = await saveNote(taskId, editingNoteId, editTitle.trim(), editContent.trim())
    setNotes(notes.map((n) => (n.id === editingNoteId ? { ...note, created_at: n.created_at } : n)))
    setEditingNoteId(null)
  }

  const handleDelete = async (noteId: string) => {
    await deleteNote(noteId)
    setNotes(notes.filter((n) => n.id !== noteId))
    if (editingNoteId === noteId) setEditingNoteId(null)
  }

  const startEdit = (note: Note) => {
    setEditingNoteId(note.id)
    setEditTitle(note.title)
    setEditContent(note.content)
    setShowNew(false)
  }

  return (
    <div className="p-4 space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-xs text-muted-foreground">任务笔记</span>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => {
            setShowNew(true)
            setEditingNoteId(null)
            setEditTitle('')
            setEditContent('')
          }}
        >
          <Plus className="size-3.5" />
          新建笔记
        </Button>
      </div>

      {/* 新建/编辑笔记表单 */}
      {(showNew || editingNoteId) && (
        <div className="space-y-3 p-3 rounded-lg border border-border bg-card">
          <Input
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            placeholder="笔记标题"
            className="text-sm"
            autoFocus
          />
          <Textarea
            value={editContent}
            onChange={(e) => setEditContent(e.target.value)}
            placeholder="笔记内容..."
            className="text-sm min-h-[120px]"
          />
          <div className="flex gap-2">
            <Button size="sm" onClick={editingNoteId ? handleSaveEdit : handleSaveNew}>
              <Save className="size-3.5" />
              保存
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setShowNew(false)
                setEditingNoteId(null)
              }}
            >
              取消
            </Button>
          </div>
        </div>
      )}

      {/* 笔记列表 */}
      {notes.length === 0 && !showNew ? (
        <div className="text-center py-8 text-sm text-muted-foreground">
          <FileText className="size-8 mx-auto mb-2 opacity-50" />
          <p>暂无笔记</p>
        </div>
      ) : (
        <div className="space-y-2">
          {notes.map((note) => (
            <div
              key={note.id}
              className="p-3 rounded-lg border border-border bg-card hover:bg-accent/30 cursor-pointer transition-colors"
              onClick={() => startEdit(note)}
            >
              <div className="flex items-start justify-between gap-2">
                <div className="flex-1 min-w-0">
                  <h4 className="text-sm font-medium truncate">{note.title}</h4>
                  <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                    {note.content || '空内容'}
                  </p>
                  <p className="text-xs text-muted-foreground/60 mt-2">
                    {new Date(note.updated_at).toLocaleString()}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  className="size-6 shrink-0"
                  onClick={(e) => {
                    e.stopPropagation()
                    if (confirm('确定删除此笔记？')) {
                      handleDelete(note.id)
                    }
                  }}
                >
                  <Trash2 className="size-3 text-red-400" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
