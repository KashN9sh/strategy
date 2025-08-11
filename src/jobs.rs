use glam::IVec2;

use crate::types::{Building, BuildingKind, Citizen, Job, JobKind, LogItem, Resources, WarehouseStore, CitizenState};
use crate::world::World;

fn job_anchor(kind: &JobKind) -> IVec2 {
    match *kind {
        JobKind::ChopWood { pos } => pos,
        JobKind::HaulWood { from, .. } => from,
    }
}

pub fn assign_jobs_nearest_worker(citizens: &mut Vec<Citizen>, jobs: &mut Vec<Job>, world: &World, buildings: &Vec<Building>) {
    // Для каждой свободной задачи найдём ближайшего свободного жителя
    for (_jid, job) in jobs.iter_mut().enumerate() {
        if job.taken || job.done { continue; }
        let target = job_anchor(&job.kind);
        // 1) Кандидаты — работники лесорубок (если задача про лес)
        let mut best: Option<(usize, i32)> = None;
        let prefers_lumberjack = matches!(job.kind, JobKind::ChopWood { .. } | JobKind::HaulWood { .. });
        if prefers_lumberjack {
            for (i, c) in citizens.iter().enumerate() {
                if c.assigned_job.is_some() || c.moving || !c.fed_today { continue; }
                // допускаем состояния Idle/Working/GoingToWork — можем переключить на задачу
                if !matches!(c.state, CitizenState::Idle | CitizenState::Working | CitizenState::GoingToWork) { continue; }
                if let Some(wp) = c.workplace {
                    if let Some(b) = buildings.iter().find(|b| b.pos == wp) {
                        if b.kind == BuildingKind::Lumberjack {
                            let d = (c.pos.x - target.x).abs() + (c.pos.y - target.y).abs();
                            if best.map(|(_,bd)| d < bd).unwrap_or(true) { best = Some((i, d)); }
                        }
                    }
                }
            }
        }
        // 2) Фоллбэк — свободные Idle граждане
        if best.is_none() {
            for (i, c) in citizens.iter().enumerate() {
                if c.assigned_job.is_some() || c.moving || !c.fed_today { continue; }
                if !matches!(c.state, CitizenState::Idle) { continue; }
                let d = (c.pos.x - target.x).abs() + (c.pos.y - target.y).abs();
                if best.map(|(_,bd)| d < bd).unwrap_or(true) { best = Some((i, d)); }
            }
        }
        if let Some((cid, _)) = best {
            let c = &mut citizens[cid];
            job.taken = true;
            c.assigned_job = Some(job.id);
            super::plan_path(world, c, target);
            c.moving = true;
            c.progress = 0.0;
        }
    }
}

pub fn process_jobs(
    citizens: &mut Vec<Citizen>,
    jobs: &mut Vec<Job>,
    logs_on_ground: &mut Vec<LogItem>,
    warehouses: &mut Vec<WarehouseStore>,
    resources: &mut Resources,
    buildings: &Vec<Building>,
    world: &mut World,
    next_job_id: &mut u64,
) {
    for c in citizens.iter_mut() {
        if let Some(job_id) = c.assigned_job {
            let jid = match jobs.iter().position(|j| j.id == job_id) { Some(i) => i, None => { c.assigned_job = None; continue; } };
            match jobs[jid].kind {
                JobKind::ChopWood { pos } => {
                    if !c.moving && c.pos == pos {
                        // Если дерево зрелое — срубаем и спавним полено; иначе завершаем без публикации Haul
                        if let Some(stage) = world.tree_stage(pos) {
                            if stage == 2 { world.remove_tree(pos); logs_on_ground.push(LogItem { pos, carried: false }); }
                            else { jobs[jid].done = true; c.assigned_job = None; continue; }
                        } else { jobs[jid].done = true; c.assigned_job = None; continue; }
                        // Цель доставки — ближайший склад; если складов нет — завершаем без Haul
                        let target_pos = if let Some((_, wh)) = warehouses
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, w)| (w.pos.x - pos.x).abs() + (w.pos.y - pos.y).abs())
                        {
                            wh.pos
                        } else {
                            jobs[jid].done = true; c.assigned_job = None; continue;
                        };
                        // Завершаем ChopWood и публикуем HaulWood до склада
                        jobs[jid].done = true;
                        jobs.push(Job { id: { let id=*next_job_id; *next_job_id+=1; id }, kind: JobKind::HaulWood { from: pos, to: target_pos }, taken: false, done: false });
                        c.assigned_job = None;
                    } else if !c.moving {
                        // планируем путь к дереву, если ещё не двигаемся
                        super::plan_path(world, c, pos);
                    }
                }
                JobKind::HaulWood { from, to } => {
                    if !c.carrying_log {
                    if !c.moving && c.pos == from {
                            if let Some(idx) = logs_on_ground.iter().position(|l| l.pos == from && !l.carried) {
                                // забираем полено и удаляем его из мира сразу
                                logs_on_ground.remove(idx);
                                c.carrying_log = true;
                            // планируем путь до склада
                            super::plan_path(world, c, to);
                            } else {
                                jobs[jid].done = true;
                                c.assigned_job = None;
                            }
                        }
                    } else {
                        if !c.moving && c.pos == to {
                            if let Some(w) = warehouses.iter_mut().find(|w| w.pos == to) { w.wood += 1; }
                            jobs[jid].done = true;
                            c.carrying_log = false;
                            c.assigned_job = None;
                            // полена уже нет в мире — удалили при взятии
                        }
                    }
                }
            }
        }
    }
    // Чистка выполненных задач
    jobs.retain(|j| !j.done);
    // Сброс ссылок у жителей на удалённые задачи
    for c in citizens.iter_mut() {
        if let Some(job_id) = c.assigned_job {
            if !jobs.iter().any(|j| j.id == job_id) { c.assigned_job = None; }
        }
    }
}


