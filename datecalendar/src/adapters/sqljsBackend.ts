/**
 * SQL.js 离线降级适配器
 * 当 Tauri 未启动时，使用浏览器内存 SQLite 提供基本数据操作
 */
import * as taskBackend from '../backend/taskBackend';
import * as scheduleBackend from '../backend/scheduleBackend';
import type { BackendInterface } from './types';
import type { Task, CreateTaskInput, MilestoneRisk, Note } from '@/types/task';
import type { Schedule } from '@/types/schedule';

export const sqljsBackend: BackendInterface = {
  // 任务
  get_all_tasks: () => taskBackend.get_all_tasks(),
  get_task: (id) => taskBackend.get_task(id),
  create_task: (input) => taskBackend.create_task(input),
  update_task: (id, title, description, status, priority, color, isMilestone, parentId, sortOrder) =>
    taskBackend.update_task(id, title, description, status, priority, color, isMilestone, parentId, sortOrder),
  delete_task: (id) => taskBackend.delete_task(id),
  search_tasks: (query) => taskBackend.search_tasks(query),

  // 风险
  get_risks: (taskId) => taskBackend.get_risks(taskId),
  add_risk: (taskId, riskDesc, probability, mitigation) =>
    taskBackend.add_risk(taskId, riskDesc, probability, mitigation),
  delete_risk: (riskId) => taskBackend.delete_risk(riskId),

  // 笔记
  get_notes: (taskId) => taskBackend.get_notes(taskId),
  save_note: (taskId, noteId, title, content) =>
    taskBackend.save_note(taskId, noteId, title, content),
  delete_note: (noteId) => taskBackend.delete_note(noteId),

  // 排序与批量
  reorder_task: (taskId, newParentId, newSortOrder) =>
    taskBackend.reorder_task(taskId, newParentId, newSortOrder),
  batch_update_tasks: (ids, status) => taskBackend.batch_update_tasks(ids, status),
  batch_delete_tasks: (ids) => taskBackend.batch_delete_tasks(ids),
  batch_move_tasks: (ids, newParentId) => taskBackend.batch_move_tasks(ids, newParentId),

  // 日程
  get_all_schedules: () => scheduleBackend.get_all_schedules(),
  get_schedule: (id) => scheduleBackend.get_schedule(id),
  get_schedules_in_range: (startDate, endDate) =>
    scheduleBackend.get_schedules_in_range(startDate, endDate),
  get_day_schedules: (date) => scheduleBackend.get_day_schedules(date),
  get_week_schedules: (weekStart, weekEnd) =>
    scheduleBackend.get_week_schedules(weekStart, weekEnd),
  get_schedules_by_task: (taskId) => scheduleBackend.get_schedules_by_task(taskId),
  create_schedule: (taskId, title, startTime, endTime, isAllDay, scheduleType, color) =>
    scheduleBackend.create_schedule(taskId, title, startTime, endTime, isAllDay, scheduleType, color),
  update_schedule: (id, title, startTime, endTime, isAllDay, scheduleType, status, color, taskId) =>
    scheduleBackend.update_schedule(id, title, startTime, endTime, isAllDay, scheduleType, status, color, taskId),
  delete_schedule: (id) => scheduleBackend.delete_schedule(id),
  update_schedule_status: (scheduleId, newStatus) =>
    scheduleBackend.update_schedule_status(scheduleId, newStatus),
  check_conflicts: (startTime, endTime, excludeId) =>
    scheduleBackend.check_conflicts(startTime, endTime, excludeId),
};
