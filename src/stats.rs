use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use prettytable::{cell, Row, Table};

use crate::bvh::BVH;
use crate::float::*;
use crate::intersect::Ray;

// Helper trait to print out Float type used
trait FloatName {
    fn float_name() -> String;
}

impl FloatName for f32 {
    fn float_name() -> String {
        "f32".to_string()
    }
}

impl FloatName for f64 {
    fn float_name() -> String {
        "f64".to_string()
    }
}

lazy_static::lazy_static! {
    static ref STATS: Mutex<Statistics> = Mutex::new(Statistics::new());
}

macro_rules! stats {
    () => {
        STATS.lock().unwrap()
    };
}

macro_rules! current_scene {
    () => {
        stats!().current().unwrap()
    };
}

pub fn print_and_save(path: &Path) {
    let table = stats!().table();
    table.printstd();
    let mut stats_file = File::create(path).unwrap();
    table.print(&mut stats_file).unwrap();
}

pub fn new_scene(name: &str) {
    stats!().new_scene(name);
}

pub fn time(name: &str) -> TimerHandle {
    current_scene!().start_timer(name)
}

fn stop_timer(name: &str) {
    current_scene!().stop_timer(name);
}

pub fn start_bvh() {
    let mut handle = time("BVH");
    handle.deactivate();
}

pub fn stop_bvh(bvh: &BVH, n_tris: usize) {
    stop_timer("BVH");
    current_scene!().analyze_bvh(bvh, n_tris);
}

pub fn start_render() {
    let mut handle = time("Render");
    Ray::reset_count();
    handle.deactivate();
}

pub fn stop_render() {
    stop_timer("Render");
    current_scene!().ray_count = Ray::count();
}

struct Statistics {
    scene_stats: Vec<SceneStatistics>,
}

impl Statistics {
    fn new() -> Statistics {
        Statistics {
            scene_stats: Vec::new(),
        }
    }

    fn new_scene(&mut self, name: &str) {
        self.scene_stats.push(SceneStatistics::new(name));
    }

    fn current(&mut self) -> Option<&mut SceneStatistics> {
        self.scene_stats.iter_mut().last()
    }

    fn table(&self) -> Table {
        let mut names = vec![cell!(Float::float_name())];
        let mut timer_rows = Vec::new();
        let mut mrps = vec![cell!("Mrays/s")];
        let mut n_tris = vec![cell!("Triangles")];
        let mut bvh_size = vec![cell!("BVH Nodes")];
        let mut n_rays = vec![cell!("Rays")];
        for (timer, l) in &self.scene_stats[0].timers {
            let mut row = Row::empty();
            row.add_cell(cell!(format!("{}{}", "| ".repeat(*l), timer.name)));
            timer_rows.push((&timer.name, row))
        }
        for stats in &self.scene_stats {
            names.push(cell!(stats.scene));
            mrps.push(cell!(stats.mrps()));
            n_tris.push(cell!(stats.n_tris));
            bvh_size.push(cell!(stats.bvh_size));
            n_rays.push(cell!(stats.ray_count));
            for (name, row) in &mut timer_rows {
                let timer = stats.get_timer(name).unwrap();
                row.add_cell(cell!(timer.pretty_duration()));
            }
        }
        let mut table = Table::new();
        table.add_row(Row::new(names));
        table.add_row(Row::new(mrps));
        for (_, row) in timer_rows {
            table.add_row(row);
        }
        table.add_row(Row::new(n_rays));
        table.add_row(Row::new(n_tris));
        table.add_row(Row::new(bvh_size));
        table
    }
}

struct SceneStatistics {
    scene: String,
    timers: Vec<(Timer, usize)>,
    active_timers: Vec<usize>,
    ray_count: usize,
    n_tris: usize,
    bvh_size: usize,
}

impl SceneStatistics {
    fn new(name: &str) -> SceneStatistics {
        SceneStatistics {
            scene: name.to_string(),
            timers: Vec::new(),
            active_timers: Vec::new(),
            ray_count: 0,
            n_tris: 0,
            bvh_size: 0,
        }
    }

    fn start_timer(&mut self, name: &str) -> TimerHandle {
        let timer = Timer::new(name);
        let handle = timer.handle();
        self.timers.push((timer, self.active_timers.len()));
        self.active_timers.push(self.timers.len() - 1);
        handle
    }

    fn stop_timer(&mut self, name: &str) {
        if let Some(i) = self.active_timers.pop() {
            let (timer, _) = &mut self.timers[i];
            if timer.name == name {
                timer.stop();
                return;
            } else {
                panic!("Timer '{}' not on top of timer stack", name);
            }
        } else {
            panic!(
                "Tried to stop timer '{}' when there are no active timers",
                name
            );
        }
    }

    fn analyze_bvh(&mut self, bvh: &BVH, n_tris: usize) {
        self.n_tris = n_tris;
        self.bvh_size = bvh.size();
    }

    fn get_timer(&self, name: &str) -> Option<&Timer> {
        for (timer, _) in &self.timers {
            if timer.name == name {
                return Some(timer);
            }
        }
        None
    }

    fn mrps(&self) -> String {
        let render_timer = self.get_timer("Render").unwrap();
        let render_duration = render_timer.duration.unwrap();
        let float_time = render_duration.as_float_secs();
        let mrps = self.ray_count as f64 / float_time / 1_000_000.0;
        format!("{:#.2?}", mrps)
    }
}

#[derive(Clone, Debug)]
pub struct Timer {
    name: String,
    start: Instant,
    duration: Option<Duration>,
}

impl Timer {
    fn new(name: &str) -> Timer {
        Timer {
            name: name.to_string(),
            start: Instant::now(),
            duration: None,
        }
    }

    fn stop(&mut self) {
        assert!(
            self.duration.is_none(),
            "Tried to stop already stopped timer!"
        );
        self.duration = Some(self.start.elapsed());
    }

    fn pretty_duration(&self) -> String {
        if let Some(duration) = &self.duration {
            format!("{:#.2?}", duration)
        } else {
            format!("{:#.2?}", self.start.elapsed())
        }
    }

    fn handle(&self) -> TimerHandle {
        TimerHandle {
            name: self.name.clone(),
            active: true,
        }
    }
}

pub struct TimerHandle {
    name: String,
    active: bool,
}

impl TimerHandle {
    pub fn stop(&mut self) {
        stop_timer(&self.name);
        self.deactivate();
    }

    // Prevent handle from stopping the timer when dropped
    fn deactivate(&mut self) {
        self.active = false;
    }
}

impl Drop for TimerHandle {
    fn drop(&mut self) {
        if self.active {
            self.stop()
        }
    }
}
