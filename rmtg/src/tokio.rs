#![allow(clippy::shadow_reuse)]
use bevy_ecs::prelude::IntoSystem;
use bevy_ecs::system::{Commands, Res, SystemInput, SystemParam};
use bevy_p2p::bevy_tokio_tasks::TokioTasksRuntime;
#[derive(SystemParam)]
pub struct TokioRuntime<'w, 's> {
    pub runtime: Res<'w, TokioTasksRuntime>,
    pub commands: Commands<'w, 's>,
}
impl TokioRuntime<'_, '_> {
    pub fn spawn<I: SystemInput + Send + 'static, M: 'static>(
        &self,
        fun: impl IntoSystem<I, (), M> + Send + 'static,
        future: impl Future<Output = <I as SystemInput>::Inner<'static>> + Send + 'static,
    ) where
        for<'a> <I as SystemInput>::Inner<'a>: Send,
    {
        self.runtime
            .spawn_background_task(move |mut tasks| async move {
                let ret = future.await;
                tasks
                    .run_on_main_thread(move |main| {
                        main.world.run_system_cached_with(fun, ret).unwrap();
                    })
                    .await;
            });
    }
}
