use std::time::Instant;

#[derive(Clone, Copy, Debug)]
pub struct FrameTime {
    pub delta: f32,
    pub fps: f32,
}

impl Default for FrameTime {
    fn default() -> Self {
        FrameTime {
            delta: 1.0,
            fps: 0.0,
        }
    }
}

pub struct FpsCounter {
    previous_frame_time: Instant,
    frame_time: FrameTime,
}

impl FpsCounter {
    pub fn new() -> Self {
        FpsCounter {
            previous_frame_time: Instant::now(),
            frame_time: FrameTime::default(),
        }
    }

    pub fn update(&mut self) -> FrameTime {
        let current_frame_time = std::time::Instant::now();
        let frame_time_diff = current_frame_time - self.previous_frame_time;
        self.previous_frame_time = current_frame_time;

        let delta = frame_time_diff.as_secs_f32();
        let fps = 1.0 / delta; // TODO: make a more stable and elaborate FPS counter

        FrameTime {
            delta,
            fps
        }
    }

    #[inline(always)]
    pub fn time(&self) -> FrameTime {
        return self.frame_time;
    }
}