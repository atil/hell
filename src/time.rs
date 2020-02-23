pub struct Time {
    timer: sdl2::TimerSubsystem,
    last_tick_time: u64,
    now_tick_time: u64,
}

impl Time {
    pub fn new(ctx: &sdl2::Sdl) -> Time {
        Time {
            timer: ctx.timer().unwrap(),
            now_tick_time: ctx.timer().unwrap().performance_counter(),
            last_tick_time: 0,
        }
    }

    pub fn tick(&mut self) -> f32 {
        self.last_tick_time = self.now_tick_time;
        self.now_tick_time = self.timer.performance_counter();
        let dt = ((self.now_tick_time - self.last_tick_time) as f32) * 1000.0
            / self.timer.performance_frequency() as f32;

        dt
    }
}
