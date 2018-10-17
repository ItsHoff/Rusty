mod accumulator;

use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use prettytable::{cell, ptable, row, table};

use self::accumulator::SumAccumulator;

lazy_static::lazy_static! {
    static ref STATS: Mutex<Vec<Statistics>> = Mutex::new(Vec::new());
}

macro_rules! get_stats_vec {
    () => {
        STATS.lock().unwrap()
    };
}

macro_rules! get_current {
    () => {
        get_stats_vec!().last_mut().unwrap()
    };
}

pub fn new_scene(name: &str) {
    let mut vec = get_stats_vec!();
    vec.push(Statistics::new(name));
}

pub fn print() {
    let vec = get_stats_vec!();
    for stats in &*vec {
        let (labels, durations) = stats.stopwatch_string();
        ptable!([labels, durations]);
    }
}

fn report_stopwatch(watch: &Stopwatch) {
    get_current!().read_stopwatch(watch);
}

struct Statistics {
    name: String,
    nodes: Vec<StopwatchNode>,
    node_map: HashMap<String, usize>,
}

impl Statistics {
    fn new(name: &str) -> Self {
        let mut stats = Self::empty(name);
        let total = stats.add_stopwatch("Total", None);
        let load = stats.add_stopwatch("Load", Some(total));
        stats.add_stopwatch("Loab obj", Some(load));
        stats.add_stopwatch("BVH", Some(load));
        stats.add_stopwatch("Render", Some(total));
        stats
    }

    fn empty(name: &str) -> Self {
        Statistics {
            name: name.to_string(),
            nodes: Vec::new(),
            node_map: HashMap::new(),
        }
    }

    fn add_stopwatch(&mut self, name: &str, parent: Option<usize>) -> usize {
        let new_i = self.nodes.len();
        self.nodes.push(StopwatchNode::new(name));
        self.node_map.insert(name.to_string(), new_i);
        if let Some(parent_i) = parent {
            let parent = &mut self.nodes[parent_i];
            parent.children.push(new_i);
        }
        new_i
    }

    fn read_stopwatch(&mut self, watch: &Stopwatch) {
        if let Some(&i) = self.node_map.get(&watch.name) {
            let node = &mut self.nodes[i];
            node.duration.add_self(watch.duration);
        } else {
            panic!("Stopwatch {} did not have a matching node!", watch.name);
        }
    }

    fn stopwatch_string(&self) -> (String, String) {
        let mut labels = String::new();
        let mut durations = String::new();
        self.add_node_to_string(0, 0, &mut labels, &mut durations);
        (labels, durations)
    }

    fn add_node_to_string(&self, node_i: usize, level: usize, labels: &mut String, durations: &mut String) {
        let node = &self.nodes[node_i];
        writeln!(labels, "{}{}", "| ".repeat(level), node.name);
        writeln!(durations, "{}{:#.2?}", "| ".repeat(level), node.duration.val);
        for &child_i in &node.children {
            self.add_node_to_string(child_i, level + 1, labels, durations);
        }
    }
}

#[derive(Debug)]
struct StopwatchNode {
    name: String,
    duration: SumAccumulator<Duration>,
    children: Vec<usize>,
}

impl StopwatchNode {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            duration: SumAccumulator::new(),
            children: Vec::new(),
        }
    }
}

pub struct Stopwatch {
    name: String,
    start: Option<Instant>,
    duration: SumAccumulator<Duration>,
}

impl Stopwatch {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: None,
            duration: SumAccumulator::new()
        }
    }

    pub fn start(&mut self) {
        assert!(
            self.start.is_none(),
            "Tried to start already running stopwatch {}!",
            self.name
        );
        self.start = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        if let Some(start) = self.start.take() {
            self.duration.add_val(start.elapsed());
        }
    }
}

impl Drop for Stopwatch {
    fn drop(&mut self) {
        self.stop();
        report_stopwatch(self);
    }
}
