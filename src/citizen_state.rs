use crate::types::{Citizen, CitizenState, Building, Job};
use crate::world::World;

/// Контекст для состояния - содержит данные, необходимые для принятия решений
pub struct StateContext<'a> {
    pub world: &'a mut World,
    #[allow(dead_code)] // Может быть полезно для будущих расширений логики состояний
    pub buildings: &'a [Building],
    #[allow(dead_code)] // Может быть полезно для будущих расширений логики состояний
    pub jobs: &'a mut Vec<Job>,
    pub is_daytime: bool,
}

/// Trait для поведения состояния гражданина
/// Каждое состояние реализует свою логику обновления и переходов
pub trait CitizenStateBehavior {
    /// Обновить состояние гражданина
    /// Возвращает новое состояние, если нужно перейти, иначе None
    fn update(
        &mut self,
        citizen: &mut Citizen,
        context: &mut StateContext,
        step_ms: f32,
    ) -> Option<Box<dyn CitizenStateBehavior>>;
    
    /// Вызывается при входе в состояние
    fn on_enter(&mut self, citizen: &mut Citizen, _context: &StateContext) {
        let _ = citizen; // для будущего использования
    }
    
    /// Вызывается при выходе из состояния
    fn on_exit(&mut self, citizen: &mut Citizen, _context: &StateContext) {
        let _ = citizen; // для будущего использования
    }
    
    /// Получить тип состояния для совместимости со старым кодом
    fn state_type(&self) -> CitizenState;
    
    /// Можно ли назначить задачу гражданину в этом состоянии
    fn can_accept_job(&self, citizen: &Citizen) -> bool;
    
    /// Обработать ночную рутину (переход домой)
    fn handle_night(&mut self, citizen: &mut Citizen, context: &mut StateContext) -> Option<Box<dyn CitizenStateBehavior>> {
        // Отменяем активную задачу/перенос
        citizen.assigned_job = None;
        citizen.carrying_log = false;
        
        if citizen.pos != citizen.home {
            if !citizen.moving {
                crate::game::plan_path(context.world, citizen, citizen.home);
            }
            return Some(Box::new(GoingHomeState));
        } else {
            return Some(Box::new(SleepingState));
        }
    }
    
    /// Обработать утреннюю рутину (пробуждение)
    fn handle_dawn(&mut self, citizen: &mut Citizen, context: &mut StateContext) -> Option<Box<dyn CitizenStateBehavior>> {
        if context.is_daytime {
            // Если у гражданина есть рабочее место, закрепленное вручную, идем туда
            if citizen.manual_workplace {
                if let Some(workplace) = citizen.workplace {
                    if !citizen.moving {
                        crate::game::plan_path(context.world, citizen, workplace);
                        return Some(Box::new(GoingToWorkState));
                    }
                }
            }
            return Some(Box::new(IdleState));
        }
        None
    }
}

/// Состояние простоя - гражданин ждет работу
pub struct IdleState;

impl CitizenStateBehavior for IdleState {
    fn update(
        &mut self,
        citizen: &mut Citizen,
        context: &mut StateContext,
        _step_ms: f32,
    ) -> Option<Box<dyn CitizenStateBehavior>> {
        // Если у гражданина есть назначенное рабочее место, идем туда
        if let Some(workplace) = citizen.workplace {
            if !citizen.moving && citizen.pos != workplace {
                crate::game::plan_path(context.world, citizen, workplace);
                return Some(Box::new(GoingToWorkState));
            }
        }
        None
    }
    
    fn state_type(&self) -> CitizenState {
        CitizenState::Idle
    }
    
    fn can_accept_job(&self, citizen: &Citizen) -> bool {
        !citizen.moving && citizen.fed_today
    }
}

/// Состояние движения к работе
pub struct GoingToWorkState;

impl CitizenStateBehavior for GoingToWorkState {
    fn update(
        &mut self,
        citizen: &mut Citizen,
        _context: &mut StateContext,
        _step_ms: f32,
    ) -> Option<Box<dyn CitizenStateBehavior>> {
        // Проверяем, достигли ли рабочего места
        if !citizen.moving {
            if let Some(workplace) = citizen.workplace {
                if citizen.pos == workplace {
                    // Не пускаем работать, если не накормлен
                    if citizen.fed_today {
                        return Some(Box::new(WorkingState));
                    } else {
                        return Some(Box::new(IdleState));
                    }
                }
            }
        }
        None
    }
    
    fn state_type(&self) -> CitizenState {
        CitizenState::GoingToWork
    }
    
    fn can_accept_job(&self, _citizen: &Citizen) -> bool {
        false
    }
}

/// Состояние работы - гражданин работает на рабочем месте
pub struct WorkingState;

impl CitizenStateBehavior for WorkingState {
    fn update(
        &mut self,
        citizen: &mut Citizen,
        _context: &mut StateContext,
        _step_ms: f32,
    ) -> Option<Box<dyn CitizenStateBehavior>> {
        // Проверяем, на месте ли гражданин
        if let Some(workplace) = citizen.workplace {
            if citizen.pos != workplace {
                // Если ушел с рабочего места, возвращаемся в Idle
                return Some(Box::new(IdleState));
            }
        } else {
            // Если рабочее место потеряно, возвращаемся в Idle
            return Some(Box::new(IdleState));
        }
        
        // Проверяем, накормлен ли гражданин
        if !citizen.fed_today {
            return Some(Box::new(IdleState));
        }
        
        None
    }
    
    fn state_type(&self) -> CitizenState {
        CitizenState::Working
    }
    
    fn can_accept_job(&self, citizen: &Citizen) -> bool {
        // Может принять задачу, если накормлен
        citizen.fed_today && !citizen.moving
    }
}

/// Состояние движения к месту сдачи ресурсов
pub struct GoingToDepositState;

impl CitizenStateBehavior for GoingToDepositState {
    fn update(
        &mut self,
        citizen: &mut Citizen,
        _context: &mut StateContext,
        _step_ms: f32,
    ) -> Option<Box<dyn CitizenStateBehavior>> {
        // Проверяем, достигли ли цели
        if !citizen.moving {
            // Обрабатывается через jobs::process_jobs при достижении цели
            // Здесь просто возвращаемся в Idle после завершения
            if citizen.carrying.is_none() && citizen.assigned_job.is_none() {
                // Если есть рабочее место, возвращаемся туда
                if citizen.workplace.is_some() {
                    return Some(Box::new(GoingToWorkState));
                } else {
                    return Some(Box::new(IdleState));
                }
            }
        }
        None
    }
    
    fn state_type(&self) -> CitizenState {
        CitizenState::GoingToDeposit
    }
    
    fn can_accept_job(&self, _citizen: &Citizen) -> bool {
        false
    }
}

/// Состояние движения за ресурсами
pub struct GoingToFetchState;

impl CitizenStateBehavior for GoingToFetchState {
    fn update(
        &mut self,
        citizen: &mut Citizen,
        _context: &mut StateContext,
        _step_ms: f32,
    ) -> Option<Box<dyn CitizenStateBehavior>> {
        // Проверяем, достигли ли цели
        if !citizen.moving {
            // Обрабатывается через jobs::process_jobs при достижении цели
            // Если есть ресурс, возвращаемся к работе
            if citizen.carrying.is_some() {
                if citizen.workplace.is_some() {
                    return Some(Box::new(GoingToWorkState));
                } else {
                    return Some(Box::new(IdleState));
                }
            }
        }
        None
    }
    
    fn state_type(&self) -> CitizenState {
        CitizenState::GoingToFetch
    }
    
    fn can_accept_job(&self, _citizen: &Citizen) -> bool {
        false
    }
}

/// Состояние движения домой
pub struct GoingHomeState;

impl CitizenStateBehavior for GoingHomeState {
    fn update(
        &mut self,
        citizen: &mut Citizen,
        _context: &mut StateContext,
        _step_ms: f32,
    ) -> Option<Box<dyn CitizenStateBehavior>> {
        // Проверяем, достигли ли дома
        if !citizen.moving && citizen.pos == citizen.home {
            return Some(Box::new(SleepingState));
        }
        None
    }
    
    fn state_type(&self) -> CitizenState {
        CitizenState::GoingHome
    }
    
    fn can_accept_job(&self, _citizen: &Citizen) -> bool {
        false
    }
    
    fn on_enter(&mut self, citizen: &mut Citizen, _context: &StateContext) {
        // Отменяем активную задачу/перенос при начале движения домой
        citizen.assigned_job = None;
        citizen.carrying_log = false;
    }
}

/// Состояние сна
pub struct SleepingState;

impl CitizenStateBehavior for SleepingState {
    fn update(
        &mut self,
        citizen: &mut Citizen,
        context: &mut StateContext,
        _step_ms: f32,
    ) -> Option<Box<dyn CitizenStateBehavior>> {
        // Если наступил день и мы дома, просыпаемся
        if context.is_daytime {
            if citizen.pos == citizen.home {
                return Some(Box::new(IdleState));
            }
        }
        None
    }
    
    fn state_type(&self) -> CitizenState {
        CitizenState::Sleeping
    }
    
    fn can_accept_job(&self, _citizen: &Citizen) -> bool {
        false
    }
    
    fn on_enter(&mut self, citizen: &mut Citizen, _context: &StateContext) {
        // Сбрасываем рабочее место, если не закреплено вручную
        if !citizen.manual_workplace {
            citizen.workplace = None;
        }
        citizen.work_timer_ms = 0;
        citizen.carrying = None;
    }
    
    fn on_exit(&mut self, citizen: &mut Citizen, _context: &StateContext) {
        // При выходе из сна отменяем движение домой
        citizen.moving = false;
    }
}

/// Вспомогательная функция для создания состояния из enum
pub fn create_state_from_enum(state: CitizenState) -> Box<dyn CitizenStateBehavior> {
    match state {
        CitizenState::Idle => Box::new(IdleState),
        CitizenState::GoingToWork => Box::new(GoingToWorkState),
        CitizenState::Working => Box::new(WorkingState),
        CitizenState::GoingToDeposit => Box::new(GoingToDepositState),
        CitizenState::GoingToFetch => Box::new(GoingToFetchState),
        CitizenState::GoingHome => Box::new(GoingHomeState),
        CitizenState::Sleeping => Box::new(SleepingState),
    }
}

/// Вспомогательная функция для обновления состояния гражданина
#[allow(dead_code)] // Может быть полезно для будущего использования или более детального управления состояниями
pub fn update_citizen_state(
    state: &mut Box<dyn CitizenStateBehavior>,
    citizen: &mut Citizen,
    context: &mut StateContext,
    step_ms: f32,
) {
    if let Some(new_state) = state.update(citizen, context, step_ms) {
        state.on_exit(citizen, context);
        *state = new_state;
        state.on_enter(citizen, context);
        // Обновляем enum для совместимости
        citizen.state = state.state_type();
    }
}

/// Обработать ночную рутину для всех граждан (используя State Pattern)
pub fn handle_night_routine_with_states(
    citizens: &mut Vec<Citizen>,
    world: &mut World,
    buildings: &[Building],
    jobs: &mut Vec<Job>,
    is_daytime: bool,
) {
    for c in citizens.iter_mut() {
        let mut state: Box<dyn CitizenStateBehavior> = create_state_from_enum(c.state);
        let mut context = StateContext {
            world,
            buildings,
            jobs,
            is_daytime,
        };
        
        if !is_daytime {
            if let Some(new_state) = state.handle_night(c, &mut context) {
                state.on_exit(c, &context);
                state = new_state;
                state.on_enter(c, &context);
                c.state = state.state_type();
            }
        }
    }
}

/// Обработать утреннюю рутину для всех граждан (используя State Pattern)
/// Примечание: world должен быть &mut, но для совместимости используем другой подход
pub fn handle_dawn_routine_with_states(
    citizens: &mut Vec<Citizen>,
    world: &mut World,
    buildings: &[Building],
    jobs: &mut Vec<Job>,
    is_daytime: bool,
) {
    for c in citizens.iter_mut() {
        let mut state: Box<dyn CitizenStateBehavior> = create_state_from_enum(c.state);
        let mut context = StateContext {
            world,
            buildings,
            jobs,
            is_daytime,
        };
        
        if let Some(new_state) = state.handle_dawn(c, &mut context) {
            state.on_exit(c, &context);
            state = new_state;
            state.on_enter(c, &context);
            c.state = state.state_type();
        }
    }
}

/// Проверить, может ли гражданин принять задачу (используя State Pattern)
#[allow(dead_code)] // Может быть полезно для будущего использования в системе заданий
pub fn citizen_can_accept_job(citizen: &Citizen) -> bool {
    let state: Box<dyn CitizenStateBehavior> = create_state_from_enum(citizen.state);
    state.can_accept_job(citizen)
}

