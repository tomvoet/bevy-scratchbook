use fluid_2d::{
    controls::{Action, Params},
    ControlSender,
};
use leptos::prelude::*;

#[derive(Clone)]
pub struct Bus {
    pub params: RwSignal<Params>,
    sender: ControlSender,
}

impl Bus {
    pub fn new(sender: ControlSender) -> Self {
        let bus = Self {
            params: RwSignal::new(Params::default()),
            sender,
        };
        bus.sender.params(bus.params.get_untracked());
        bus
    }

    pub fn edit(&self, edit: impl FnOnce(&mut Params)) {
        self.params.update(edit);
        self.sender.params(self.params.get_untracked());
    }

    pub fn act(&self, action: Action) {
        if action == Action::ResetParams {
            self.edit(|params| params.reset());
            return;
        }
        self.sender.action(action);
    }
}
