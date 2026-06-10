/**
 * 适配层接口类型定义
 * 所有适配器（tauri/http/sqljs）必须实现此接口
 */
import type { Task, CreateTaskInput, MilestoneRisk, Note } from '@/types/task';
import type { Schedule } from '@/types/schedule';

export interface BackendInterface {
  // 任务
  get_all_tasks(): Promise<Task[]>;
  get_task(id: string): Promise<Task | null>;
  create_task(input: CreateTaskInput): Promise<Task>;
  update_task(id: string, title?: string, description?: string, status?: string,
    priority?: number, color?: string, isMilestone?: boolean,
    parentId?: string | null, sortOrder?: number): Promise<Task>;
  delete_task(id: string): Promise<void>;
  search_tasks(query: string): Promise<Task[]>;

  // 风险
  get_risks(taskId: string): Promise<MilestoneRisk[]>;
  add_risk(taskId: string, riskDesc: string, probability?: string, mitigation?: string): Promise<MilestoneRisk>;
  delete_risk(riskId: string): Promise<void>;

  // 笔记
  get_notes(taskId: string): Promise<Note[]>;
  save_note(taskId: string, noteId: string | null, title: string, content: string): Promise<Note>;
  delete_note(noteId: string): Promise<void>;

  // 排序与批量
  reorder_task(taskId: string, newParentId: string | null, newSortOrder: number): Promise<void>;
  batch_update_tasks(ids: string[], status: string): Promise<void>;
  batch_delete_tasks(ids: string[]): Promise<void>;
  batch_move_tasks(ids: string[], newParentId: string | null): Promise<void>;

  // 日程
  get_all_schedules(): Promise<Schedule[]>;
  get_schedule(id: string): Promise<Schedule | null>;
  get_schedules_in_range(startDate: string, endDate: string): Promise<Schedule[]>;
  get_day_schedules(date: string): Promise<Schedule[]>;
  get_week_schedules(weekStart: string, weekEnd: string): Promise<Schedule[]>;
  get_schedules_by_task(taskId: string): Promise<Schedule[]>;
  create_schedule(taskId: string, title: string, startTime: string, endTime: string,
    isAllDay?: boolean, scheduleType?: string, color?: string): Promise<Schedule>;
  update_schedule(id: string, title?: string, startTime?: string, endTime?: string,
    isAllDay?: boolean, scheduleType?: string, status?: string,
    color?: string, taskId?: string): Promise<Schedule>;
  delete_schedule(id: string): Promise<void>;
  update_schedule_status(scheduleId: string, newStatus: string): Promise<void>;
  check_conflicts(startTime: string, endTime: string, excludeId?: string | null): Promise<Schedule[]>;
}
