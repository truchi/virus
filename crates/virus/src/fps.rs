use std::time::Instant;

#[derive(Debug)]
pub struct FpsCounter {
    array: [u128; Self::CAP],
    len: usize,
    now: Instant,
}

impl FpsCounter {
    const CAP: usize = 120;

    pub fn new() -> Self {
        Self {
            array: [0; Self::CAP],
            len: 0,
            now: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        if self.len == Self::CAP {
            let mpf = self.array.iter().sum::<u128>() as f32 / Self::CAP as f32;
            let fps = (1_000_000. / mpf).round();

            println!("{fps}fps");
            self.len = 0;
        }

        self.array[self.len] = self.now.elapsed().as_micros();
        self.len += 1;
        self.now = Instant::now();
    }
}
