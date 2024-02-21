use bevy::{
    ecs::system::{BoxedSystem, SystemId},
    prelude::*,
};
use bevy_mod_picking::prelude::*;

pub enum ClickHandler {
    Uninitialized(Option<BoxedSystem>),
    Initialized(SystemId),
}
impl ClickHandler {
    pub fn new<M, S: IntoSystem<(), (), M>>(handler: S) -> Self {
        ClickHandler::Uninitialized(Some(Box::new(IntoSystem::into_system(handler))))
    }
}

#[derive(Component)]
pub struct GameButton {
    pub bg_color: Color,
    pub hover_color: Color,
    pub disabled_color: Color,
    pub click_handler: ClickHandler,
    pub active: bool,
}

pub fn update_buttons(
    mut btns: Query<(Entity, Ref<GameButton>, &mut BackgroundColor)>,
    mut commands: Commands,
) {
    for (e, btn, mut bg) in &mut btns {
        if btn.is_added() {
            commands.entity(e).insert((
                On::<Pointer<Over>>::run(
                    |listener: Listener<Pointer<Over>>,
                     mut btns: Query<(&GameButton, &mut BackgroundColor)>| {
                        let (btn, mut bg) = btns.get_mut(listener.listener()).unwrap();
                        if btn.active {
                            *bg = btn.hover_color.into();
                        }
                    },
                ),
                On::<Pointer<Out>>::run(
                    |listener: Listener<Pointer<Out>>,
                     mut btns: Query<(&GameButton, &mut BackgroundColor)>| {
                        let (btn, mut bg) = btns.get_mut(listener.listener()).unwrap();
                        if btn.active {
                            *bg = btn.bg_color.into();
                        }
                    },
                ),
                On::<Pointer<Click>>::run(move |world: &mut World| {
                    let mut btn = world.get_mut::<GameButton>(e).unwrap();
                    if !btn.active {
                        return;
                    }

                    let sys_id = match btn.click_handler {
                        ClickHandler::Uninitialized(ref mut sys) => {
                            let sys = sys.take().unwrap();
                            let id = world.register_boxed_system(sys);
                            world.get_mut::<GameButton>(e).unwrap().click_handler =
                                ClickHandler::Initialized(id);
                            id
                        },
                        ClickHandler::Initialized(id) => id,
                    };

                    world.run_system(sys_id).unwrap();
                }),
            ));
        }
        if btn.is_changed() {
            *bg = if btn.active { btn.bg_color } else { btn.disabled_color }.into()
        }
    }
}
