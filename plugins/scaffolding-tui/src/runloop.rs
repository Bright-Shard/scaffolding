use {
    crate::{msg::TuiMsg, Terminal},
    scaffolding::world::{Executable, World},
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

    pub fn start<Args, E>(self, mut world: World, mut app_main: E)
    where
        for<'a> &'a mut E: Executable<'a, Args>,
    {
        let time_between_frames = Duration::from_secs(1) / self.fps;
        let mut goal = Instant::now() + time_between_frames;

        loop {
            (&mut app_main).execute(&world);

            let terminal: &Terminal = world.get_singleton();
            if terminal.exit {
                break;
            }

            world.process_msgs();
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
