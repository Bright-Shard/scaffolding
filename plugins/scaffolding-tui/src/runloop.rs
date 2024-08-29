use {
    crate::{msg::TuiMsg, Terminal},
    scaffolding::world::{Executable, IntoExecutable, World},
    std::{
        thread,
        time::{Duration, Instant},
    },
};

pub struct TuiRunloop {
    pub fps: u32,
}
impl TuiRunloop {
    pub fn new(fps: u32) -> Self {
        Self { fps }
    }

    pub fn start<'a, Args, IE: IntoExecutable<'a, Args>>(self, mut world: World, app_main: IE) {
        let time_between_frames = Duration::from_secs(1) / self.fps;
        let mut goal = Instant::now() + time_between_frames;
        let executable = app_main.into_executable();

        loop {
            executable.execute(&world);

            let terminal: &Terminal = world.get_singleton();
            if terminal.exit {
                break;
            }

            world.apply_msgs();
            world.send_msg_now(TuiMsg::UpdateTerminal);

            thread::sleep(goal - Instant::now());
            goal += time_between_frames;
        }
    }
}
impl Default for TuiRunloop {
    fn default() -> Self {
        Self { fps: 60 }
    }
}
