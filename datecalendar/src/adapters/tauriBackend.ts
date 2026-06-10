/**
 * Tauri IPC 适配器
 * 直接透传到 invoke()，参数使用 camelCase（Tauri 自动转换）
 */
import { invoke } from '@tauri-apps/api/core';
import type { BackendInterface } from './types';
import type { Task, CreateTaskInput, MilestoneRisk, Note } from '@/types/task';
import type { Schedule } from '@/types/schedule';

export const tauriBackend: BackendInterface = {
  async get_all_tasks(): Promise<Task[]> {
    return invoke<Task[]>('get_all_tasks');
  },

  async get_task(id: string): Promise<Task | null> {
    return invoke<Task | null>('get_task', { id });
  },

  async create_task(input: CreateTaskInput): Promise<Task> {
    return invoke<Task>('create_task', { input });
  },

  async update_task(id, title, description, status, priority, color, isMilestone, parentId, sortOrder): Promise<Task> {
    return invoke<Task>('update_task', { id, title, description, status, priority, color, isMilestone, parentId, sortOrder });
  },

  async delete_task(id: string): Promise<void> {
    return invoke<void>('delete_task', { id });
  },

  async search_tasks(query: string): Promise<Task[]> {
    return invoke<Task[]>('search_tasks', { query });
  },

  async get_risks(taskId: string): Promise<MilestoneRisk[]> {
    return invoke<MilestoneRisk[]>('get_risks', { taskId });
  },

  async add_risk(taskId, riskDesc, probability, mitigation): Promise<MilestoneRisk> {
    return invoke<MilestoneRisk>('add_risk', { taskId, riskDesc, probability, mitigation });
  },

  async delete_risk(riskId: string): Promise<void> {
    return invoke<void>('delete_risk', { riskId });
  },

  async get_notes(taskId: string): Promise<Note[]> {
    return invoke<Note[]>('get_notes', { taskId });
  },

  async save_note(taskId, noteId, title, content): Promise<Note> {
    return invoke<Note>('save_note', { taskId, noteId, title, content });
  },

  async delete_note(noteId: string): Promise<void> {
    return invoke<void>('delete_note', { noteId });
  },

  async reorder_task(taskId, newParentId, newSortOrder): Promise<void> {
    return invoke<void>('reorder_task', { taskId, newParentId, newSortOrder });
  },

  async batch_update_tasks(ids, status): Promise<void> {
    return invoke<void>('batch_update_tasks', { ids, status });
  },

  async batch_delete_tasks(ids): Promise<void> {
    return invoke<void>('batch_delete_tasks', { ids });
  },

  async batch_move_tasks(ids, newParentId): Promise<void> {
    return invoke<void>('batch_move_tasks', { ids, newParentId });
  },

  async get_all_schedules(): Promise<Schedule[]> {
    return invoke<Schedule[]>('get_all_schedules');
  },

  async get_schedule(id: string): Promise<Schedule | null> {
    return invoke<Schedule | null>('get_schedule', { id });
  },

  async get_schedules_in_range(startDate, endDate): Promise<Schedule[]> {
    return invoke<Schedule[]>('get_schedules_in_range', { startDate, endDate });
  },

  async get_day_schedules(date: string): Promise<Schedule[]> {
    return invoke<Schedule[]>('get_day_schedules', { date });
  },

  async get_week_schedules(weekStart, weekEnd): Promise<Schedule[]> {
    return invoke<Schedule[]>('get_week_schedules', { weekStart, weekEnd });
  },

  async get_schedules_by_task(taskId: string): Promise<Schedule[]> {
    return invoke<Schedule[]>('get_schedules_by_task', { taskId });
  },

  async create_schedule(taskId, title, startTime, endTime, isAllDay, scheduleType, color): Promise<Schedule> {
    return invoke<Schedule>('create_schedule', { taskId, title, startTime, endTime, isAllDay, scheduleType, color });
  },

  async update_schedule(id, title, startTime, endTime, isAllDay, scheduleType, status, color, taskId): Promise<Schedule> {
    return invoke<Schedule>('update_schedule', { id, title, startTime, endTime, isAllDay, scheduleType, status, color, taskId });
  },

  async delete_schedule(id: string): Promise<void> {
    return invoke<void>('delete_schedule', { id });
  },

  async update_schedule_status(scheduleId, newStatus): Promise<void> {
    return invoke<void>('update_schedule_status', { scheduleId, newStatus });
  },

  async check_conflicts(startTime, endTime, excludeId): Promise<Schedule[]> {
    return invoke<Schedule[]>('check_conflicts', { startTime, endTime, excludeId });
  },
};
