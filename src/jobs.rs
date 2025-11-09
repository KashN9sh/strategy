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
            // Для задач лесорубов НЕ используем фоллбэк — только работники лесорубок могут их выполнять
        } else {
            // 2) Фоллбэк — свободные Idle граждане (только для других типов задач)
            if best.is_none() {
                for (i, c) in citizens.iter().enumerate() {
                    if c.assigned_job.is_some() || c.moving || !c.fed_today { continue; }
                    if !matches!(c.state, CitizenState::Idle) { continue; }
                    let d = (c.pos.x - target.x).abs() + (c.pos.y - target.y).abs();
                    if best.map(|(_,bd)| d < bd).unwrap_or(true) { best = Some((i, d)); }
                }
            }
        }
        if let Some((cid, _)) = best {
            let c = &mut citizens[cid];
            job.taken = true;
            c.assigned_job = Some(job.id);
            // Если дровосек получает задачу, переводим его в состояние Working
            if prefers_lumberjack && matches!(c.state, CitizenState::Idle | CitizenState::GoingToWork) {
                c.state = CitizenState::Working;
            }
            crate::game::plan_path(world, c, target);
            // plan_path уже устанавливает moving и progress, не нужно делать это снова
            // Но если мы уже на цели, не нужно двигаться
            if c.pos == target {
                c.moving = false;
                c.progress = 0.0;
            }
        }
    }
}

pub fn process_jobs(
    citizens: &mut Vec<Citizen>,
    jobs: &mut Vec<Job>,
    logs_on_ground: &mut Vec<LogItem>,
    warehouses: &mut Vec<WarehouseStore>,
    _resources: &mut Resources,
    _buildings: &Vec<Building>,
    world: &mut World,
    next_job_id: &mut u64,
) {
    // Проверяем все задачи, которые помечены как taken, но не назначены дровосекам
    // И сбрасываем их, чтобы они могли быть назначены снова
    let orphaned_task_ids: Vec<u64> = jobs.iter()
        .filter(|j| j.taken && !j.done)
        .filter(|j| {
            matches!(j.kind, JobKind::ChopWood { .. } | JobKind::HaulWood { .. })
        })
        .filter(|j| {
            !citizens.iter().any(|c| c.assigned_job == Some(j.id))
        })
        .map(|j| j.id)
        .collect();
    if !orphaned_task_ids.is_empty() {
        for task_id in &orphaned_task_ids {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == *task_id) {
                job.taken = false;
            }
        }
    }
    
    for c in citizens.iter_mut() {
        if let Some(job_id) = c.assigned_job {
            let jid = match jobs.iter().position(|j| j.id == job_id) { 
                Some(i) => i, 
                None => { 
                    c.assigned_job = None; 
                    continue; 
                } 
            };
            match jobs[jid].kind {
                JobKind::ChopWood { pos } => {
                    if !c.moving && c.pos == pos {
                        // Если дерево зрелое (stage 2) или почти зрелое (stage 1) — срубаем и спавним полено
                        // Для stage 1 даем меньше дров или просто срубаем
                        if let Some(stage) = world.tree_stage(pos) {
                            if stage >= 1 { 
                                world.remove_tree(pos); 
                                logs_on_ground.push(LogItem { pos, carried: false }); 
                            } else { 
                                jobs[jid].done = true; 
                                c.assigned_job = None; 
                                continue; 
                            }
                        } else { 
                            jobs[jid].done = true; 
                            c.assigned_job = None; 
                            continue; 
                        }
                        // Цель доставки — ближайший склад; если складов нет — завершаем без Haul
                        let target_pos = if let Some(dst) = crate::types::find_nearest_warehouse(warehouses, pos) {
                            dst
                        } else {
                            jobs[jid].done = true; c.assigned_job = None; continue;
                        };
                        // Завершаем ChopWood и публикуем HaulWood до склада
                        jobs[jid].done = true;
                        jobs.push(Job { id: { let id=*next_job_id; *next_job_id+=1; id }, kind: JobKind::HaulWood { from: pos, to: target_pos }, taken: false, done: false });
                        c.assigned_job = None;
                    } else if !c.moving {
                        // планируем путь к дереву, если ещё не двигаемся
                        let old_moving = c.moving;
                        crate::game::plan_path(world, c, pos);
                        // Если после планирования пути мы все еще не двигаемся и не на цели, возможно путь недоступен
                        if !c.moving && c.pos != pos && old_moving == c.moving {
                            // Отменяем задачу, если не можем добраться (только если moving не изменился)
                            jobs[jid].done = true;
                            c.assigned_job = None;
                        }
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
                                crate::game::plan_path(world, c, to);
                                // Если после планирования пути мы все еще не двигаемся, возможно путь недоступен
                                if !c.moving && c.pos != to {
                                    // Отменяем задачу и сбрасываем полено
                                    jobs[jid].done = true;
                                    c.carrying_log = false;
                                    c.assigned_job = None;
                                }
                            } else {
                                jobs[jid].done = true;
                                c.assigned_job = None;
                            }
                        } else if !c.moving {
                            // планируем путь к полену, если ещё не двигаемся
                            crate::game::plan_path(world, c, from);
                            // Если после планирования пути мы все еще не двигаемся и не на цели, возможно путь недоступен
                            if !c.moving && c.pos != from {
                                // Отменяем задачу, если не можем добраться
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
                        } else if !c.moving {
                            // планируем путь к складу, если ещё не двигаемся
                            crate::game::plan_path(world, c, to);
                            // Если после планирования пути мы все еще не двигаемся, возможно путь недоступен
                            if !c.moving && c.pos != to {
                                // Отменяем задачу и сбрасываем полено
                                jobs[jid].done = true;
                                c.carrying_log = false;
                                c.assigned_job = None;
                            }
                        }
                    }
                }
            }
        }
    }
    // Чистка выполненных задач
    jobs.retain(|j| !j.done);
    // Сброс ссылок у жителей на удалённые задачи и возврат на рабочее место
    for c in citizens.iter_mut() {
        if let Some(job_id) = c.assigned_job {
            if !jobs.iter().any(|j| j.id == job_id) { 
                c.assigned_job = None;
                // Если дровосек завершил задачу и не на рабочем месте, возвращаем его туда
                if let Some(workplace) = c.workplace {
                    if c.pos != workplace && !c.moving {
                        crate::game::plan_path(world, c, workplace);
                    }
                }
            }
        } else {
            // Если дровосек без задачи, но в состоянии Working
            if matches!(c.state, CitizenState::Working) {
                if let Some(workplace) = c.workplace {
                    // Если не на рабочем месте, возвращаем его туда
                    if c.pos != workplace && !c.moving && !c.carrying_log {
                        crate::game::plan_path(world, c, workplace);
                    }
                    // Если на рабочем месте, но без задачи - это нормально, задача будет назначена в assign_jobs_nearest_worker
                }
            }
        }
    }
}


