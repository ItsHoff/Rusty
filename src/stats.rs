use std::sync::Mutex;
use std::time::{Duration, Instant};

lazy_static::lazy_static! {
    static ref STATS: Mutex<Statistics> = Mutex::new(Statistics::new());
}

pub fn print_stats() {
    STATS.lock().unwrap().print_timers();
}

struct Statistics {
    timers: Vec<(Timer, usize)>
}

impl Statistics {
    fn new() -> Statistics {
        Statistics { timers: Vec::new() }
    }

    fn add_timer(&mut self, new: Timer) {
        let mut i = self.timers.len();
        for (timer, l) in self.timers.iter_mut().rev() {
            if timer.start < new.start {
                break;
            } else {
                i -= 1;
                *l += 1;
            }
        }
        self.timers.insert(i, (new, 0));
    }

    fn print_timers(&self) {
        for (timer, l) in &self.timers {
            print!("{}", "| ".repeat(*l));
            timer.print();
        }
    }
}

#[derive(Clone)]
pub struct Timer {
    name: String,
    start: Instant,
    duration: Option<Duration>,
}

impl Timer {
    pub fn new(name: &str) -> Timer {
        Timer { name: name.to_string(), start: Instant::now(), duration: None }
    }

    pub fn stop(mut self) {
        assert!(self.duration.is_none(), "Tried to stop already stopped timer!");
        self.duration = Some(self.start.elapsed());
        STATS.lock().unwrap().add_timer(self);
    }

    pub fn print(&self) {
        if let Some(duration) = &self.duration {
            println!("{}: {:#.2?}", self.name, duration);
        } else {
            println!("{} has been running for {:#.2?}", self.name, self.start.elapsed());
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.clone().stop()
    }
}
