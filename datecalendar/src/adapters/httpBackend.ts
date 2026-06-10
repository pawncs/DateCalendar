/**
 * HTTP API 适配器
 * 浏览器在线模式下通过 fetch() 调用 Tauri 的 Actix-web HTTP API
 */
import type { BackendInterface } from './types';
import type { Task, CreateTaskInput, MilestoneRisk, Note } from '@/types/task';
import type { Schedule } from '@/types/schedule';

const BASE = 'http://localhost:9876';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`HTTP ${res.status}: ${text}`);
  }
  return res.json();
}

export const httpBackend: BackendInterface = {
  async get_all_tasks(): Promise<Task[]> {
    return request<Task[]>('/api/tasks');
  },

  async get_task(id: string): Promise<Task | null> {
    return request<Task | null>(`/api/tasks/${encodeURIComponent(id)}`);
  },

  async create_task(input: CreateTaskInput): Promise<Task> {
    return request<Task>('/api/tasks', {
      method: 'POST',
      body: JSON.stringify(input),
    });
  },

  async update_task(id, title, description, status, priority, color, isMilestone, parentId, sortOrder): Promise<Task> {
    return request<Task>(`/api/tasks/${encodeURIComponent(id)}`, {
      method: 'PUT',
      body: JSON.stringify({ title, description, status, priority, color, isMilestone, parentId, sortOrder }),
    });
  },

  async delete_task(id: string): Promise<void> {
    await request<void>(`/api/tasks/${encodeURIComponent(id)}`, { method: 'DELETE' });
  },

  async search_tasks(query: string): Promise<Task[]> {
    return request<Task[]>(`/api/tasks/search?q=${encodeURIComponent(query)}`);
  },

  async get_risks(taskId: string): Promise<MilestoneRisk[]> {
    return request<MilestoneRisk[]>(`/api/tasks/${encodeURIComponent(taskId)}/risks`);
  },

  async add_risk(taskId, riskDesc, probability, mitigation): Promise<MilestoneRisk> {
    return request<MilestoneRisk>(`/api/tasks/${encodeURIComponent(taskId)}/risks`, {
      method: 'POST',
      body: JSON.stringify({ riskDesc, probability, mitigation }),
    });
  },

  async delete_risk(riskId: string): Promise<void> {
    await request<void>(`/api/risks/${encodeURIComponent(riskId)}`, { method: 'DELETE' });
  },

  async get_notes(taskId: string): Promise<Note[]> {
    return request<Note[]>(`/api/tasks/${encodeURIComponent(taskId)}/notes`);
  },

  async save_note(taskId, noteId, title, content): Promise<Note> {
    return request<Note>(`/api/tasks/${encodeURIComponent(taskId)}/notes`, {
      method: 'PUT',
      body: JSON.stringify({ noteId, title, content }),
    });
  },

  async delete_note(noteId: string): Promise<void> {
    await request<void>(`/api/notes/${encodeURIComponent(noteId)}`, { method: 'DELETE' });
  },

  async reorder_task(taskId, newParentId, newSortOrder): Promise<void> {
    await request<void>('/api/tasks/reorder', {
      method: 'PUT',
      body: JSON.stringify({ taskId, newParentId, newSortOrder }),
    });
  },

  async batch_update_tasks(ids, status): Promise<void> {
    await request<void>('/api/tasks/batch/status', {
      method: 'PUT',
      body: JSON.stringify({ ids, status }),
    });
  },

  async batch_delete_tasks(ids): Promise<void> {
    await request<void>('/api/tasks/batch/delete', {
      method: 'POST',
      body: JSON.stringify({ ids }),
    });
  },

  async batch_move_tasks(ids, newParentId): Promise<void> {
    await request<void>('/api/tasks/batch/move', {
      method: 'PUT',
      body: JSON.stringify({ ids, newParentId }),
    });
  },

  async get_all_schedules(): Promise<Schedule[]> {
    return request<Schedule[]>('/api/schedules');
  },

  async get_schedule(id: string): Promise<Schedule | null> {
    return request<Schedule | null>(`/api/schedules/${encodeURIComponent(id)}`);
  },

  async get_schedules_in_range(startDate, endDate): Promise<Schedule[]> {
    return request<Schedule[]>(`/api/schedules/range?start=${encodeURIComponent(startDate)}&end=${encodeURIComponent(endDate)}`);
  },

  async get_day_schedules(date: string): Promise<Schedule[]> {
    return request<Schedule[]>(`/api/schedules/day/${encodeURIComponent(date)}`);
  },

  async get_week_schedules(weekStart, weekEnd): Promise<Schedule[]> {
    return request<Schedule[]>(`/api/schedules/week?start=${encodeURIComponent(weekStart)}&end=${encodeURIComponent(weekEnd)}`);
  },

  async get_schedules_by_task(taskId: string): Promise<Schedule[]> {
    return request<Schedule[]>(`/api/schedules/task/${encodeURIComponent(taskId)}`);
  },

  async create_schedule(taskId, title, startTime, endTime, isAllDay, scheduleType, color): Promise<Schedule> {
    return request<Schedule>('/api/schedules', {
      method: 'POST',
      body: JSON.stringify({ taskId, title, startTime, endTime, isAllDay, scheduleType, color }),
    });
  },

  async update_schedule(id, title, startTime, endTime, isAllDay, scheduleType, status, color, taskId): Promise<Schedule> {
    return request<Schedule>(`/api/schedules/${encodeURIComponent(id)}`, {
      method: 'PUT',
      body: JSON.stringify({ title, startTime, endTime, isAllDay, scheduleType, status, color, taskId }),
    });
  },

  async delete_schedule(id: string): Promise<void> {
    await request<void>(`/api/schedules/${encodeURIComponent(id)}`, { method: 'DELETE' });
  },

  async update_schedule_status(scheduleId, newStatus): Promise<void> {
    await request<void>(`/api/schedules/${encodeURIComponent(scheduleId)}/status`, {
      method: 'PUT',
      body: JSON.stringify({ newStatus }),
    });
  },

  async check_conflicts(startTime, endTime, excludeId): Promise<Schedule[]> {
    const params = new URLSearchParams({ startTime, endTime });
    if (excludeId) params.set('excludeId', excludeId);
    return request<Schedule[]>(`/api/schedules/conflicts?${params}`);
  },
};
