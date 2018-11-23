use std::sync::atomic::{AtomicUsize, Ordering};

use glium::Rect;

use crate::pt_renderer::RenderConfig;

pub struct RenderCoordinator {
    pub width: u32,
    pub height: u32,
    max_blocks: Option<usize>,
    current_block: AtomicUsize,
    block_width: u32,
    block_height: u32,
    x_blocks: usize,
    y_blocks: usize,
}

impl RenderCoordinator {
    pub fn new(config: &RenderConfig) -> RenderCoordinator {
        let width = config.width;
        let height = config.height;
        let block_height = 50;
        let block_width = 50;
        let x_blocks = (f64::from(width) / f64::from(block_width)).ceil() as usize;
        let y_blocks = (f64::from(height) / f64::from(block_height)).ceil() as usize;
        let blocks_per_iter = x_blocks * y_blocks;
        let max_blocks = config.max_iterations.map(|iters| iters * blocks_per_iter);
        RenderCoordinator {
            width,
            height,
            max_blocks,
            current_block: AtomicUsize::new(0),
            block_width,
            block_height,
            x_blocks,
            y_blocks,
        }
    }

    pub fn next_block(&self) -> Option<Rect> {
        let block_i = self.current_block.fetch_add(1, Ordering::Relaxed);
        if let Some(max) = self.max_blocks {
            if block_i >= max {
                return None;
            }
        };
        let iter_i = block_i % (self.x_blocks * self.y_blocks);
        let x_i = (iter_i % self.x_blocks) as u32;
        let y_i = (iter_i / self.x_blocks) as u32;
        let start_x = self.block_width * x_i;
        let end_x = (self.block_width * (x_i + 1)).min(self.width);
        let start_y = self.block_height * y_i;
        let end_y = (self.block_height * (y_i + 1)).min(self.height);
        Some(Rect {
            left: start_x,
            bottom: start_y,
            width: end_x - start_x,
            height: end_y - start_y,
        })
    }
}
