#[derive(Default)]
pub struct Actions {
    pub actions: Vec<ActionWrapper>,
}

impl /* <T: Clone + 'static> */ Actions {
    pub fn new() -> Self { Self { actions: Vec::new() }}

    pub fn add_action(&mut self, wait: u64, action: CycleAction) {
        self.actions.push(ActionWrapper {
            wait,
            has_run: false,
            action,
        });
    }

    pub fn remove_motor_off_actions(&mut self) {
        for mut action in &mut self.actions {
            match action.action {
                CycleAction::MotorOff(_) => {
                    (*action).has_run = true;
                }
                _ => {}
            }
        }
    }
}

pub struct UpdatePhaseAction {
    /// 0 or 1
    pub drive_index: usize,
    pub phase_160: usize,
}

pub struct MotorOffAction {
    /// 0 or 1
    pub drive_index: usize,
}

pub enum CycleAction {
    UpdatePhase(UpdatePhaseAction),
    MotorOff(MotorOffAction),
}

pub struct ActionWrapper {
    pub wait: u64,
    pub has_run: bool,
    pub action: CycleAction,
}

