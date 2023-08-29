#[derive(Clone, Debug)]
pub struct Workload {
    pub current_pixel: u32,
    pub start_pixel: u32,
    pub end_pixel: u32,
    pub render_size: (u32, u32),
}

impl Workload {
    pub fn new(start_pixel: u32, end_pixel: u32, render_size: (u32, u32)) -> Self {
        Self {
            current_pixel: start_pixel,
            start_pixel,
            end_pixel,
            render_size,
        }
    }
}

impl Iterator for Workload {
    type Item = (u32, u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pixel >= self.end_pixel {
            // stop, get some help
            return None;
        } else {
            let output = Some((
                self.current_pixel % self.render_size.0,
                self.current_pixel / self.render_size.0,
                self.start_pixel + self.current_pixel - self.start_pixel,
            ));

            self.current_pixel += 1;
            return output;
        }
    }
}
