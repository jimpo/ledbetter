extern crate alloc;

pub use ledbetter_macros::ledbetter;

use alloc::{vec, vec::Vec};
use core::marker::PhantomData;

pub type PixelLocation = (f32, f32);

pub trait PixelAnimation {
    type Params;
    fn new(params: &Self::Params, pixel_locs: Vec<Vec<PixelLocation>>) -> Self;
    fn tick(&mut self, params: &Self::Params);
    fn render(&self, params: &Self::Params, pixels: &mut Vec<Vec<u32>>);
}

pub struct PixelAnimationBuilder<A: PixelAnimation> {
    pixel_locs: Vec<Vec<PixelLocation>>,
    _a_marker: PhantomData<A>,
}

impl<A: PixelAnimation> PixelAnimationBuilder<A> {
    pub fn new() -> Self {
        PixelAnimationBuilder {
            pixel_locs: Vec::new(),
            _a_marker: PhantomData::default(),
        }
    }

    pub fn set_num_strips(&mut self, n_strips: usize) {
        self.pixel_locs.resize_with(n_strips, Vec::new);
    }

    pub fn set_strip_len(&mut self, strip_idx: usize, length: usize) {
        self.pixel_locs[strip_idx].resize_with(length, PixelLocation::default)
    }

    pub fn set_pixel_loc(&mut self, strip_idx: usize, pixel_idx: usize, x: f32, y: f32) {
        self.pixel_locs[strip_idx][pixel_idx] = (x, y);
    }

    pub fn build(self, params: &A::Params) -> (A, Vec<Vec<u32>>) {
        let pixels = self.pixel_locs.iter()
            .map(|strip_locs| vec![0; strip_locs.len()])
            .collect();
        let animation = A::new(params, self.pixel_locs);
        (animation, pixels)
    }
}

pub struct PixelAnimationGlobal<A: PixelAnimation>(pub Option<PixelAnimationBuildStage<A>>);

pub enum PixelAnimationBuildStage<A: PixelAnimation> {
    Building { params: A::Params, builder: PixelAnimationBuilder<A> },
    Built { params: A::Params, animation: A, pixels: Vec<Vec<u32>> },
}

impl<A: PixelAnimation> PixelAnimationGlobal<A>
    where A::Params: Default
{
    pub fn new() -> Self {
        PixelAnimationGlobal(None)
    }

    #[inline]
    fn init(&mut self) {
        use PixelAnimationBuildStage::*;
        if let None = self.0 {
            let params = A::Params::default();
            let builder = PixelAnimationBuilder::new();
            self.0 = Some(Building { params, builder });
        }
    }

    pub fn init_layout_set_num_strips(&mut self, n_strips: usize) {
        use PixelAnimationBuildStage::*;
        self.init();
        match self.0 {
            Some(Building { ref mut builder, .. }) => builder.set_num_strips(n_strips),
            Some(Built { .. }) => panic!("initLayoutSetNumStrips called after initLayoutDone"),
            None => unreachable!("init() above leaves self.0 as Some(..)"),
        }
    }

    pub fn init_layout_set_strip_len(&mut self, strip_idx: usize, length: usize) {
        use PixelAnimationBuildStage::*;
        self.init();
        match self.0 {
            Some(Building { ref mut builder, .. }) => builder.set_strip_len(strip_idx, length),
            Some(Built { .. }) => panic!("initLayoutSetStripLen called after initLayoutDone"),
            None => unreachable!("init() above leaves self.0 as Some(..)"),
        }
    }

    pub fn init_layout_set_pixel_loc(&mut self, strip_idx: usize, pixel_idx: usize, x: f32, y: f32) {
        use PixelAnimationBuildStage::*;
        self.init();
        match self.0 {
            Some(Building { ref mut builder, .. }) =>
                builder.set_pixel_loc(strip_idx, pixel_idx, x, y),
            Some(Built { .. }) => panic!("initLayoutSetPixelLoc called after initLayoutDone"),
            None => unreachable!("init() above leaves self.0 as Some(..)"),
        }
    }

    pub fn init_layout_done(&mut self) {
        use PixelAnimationBuildStage::*;
        self.init();
        match self.0.take() {
            Some(Building { params, builder }) => {
                let (animation, pixels) = builder.build(&params);
                self.0 = Some(Built { params, animation, pixels });
            }
            Some(Built { .. }) => panic!("initLayoutDone called after initLayoutDone"),
            None => unreachable!("init() above leaves self.0 as Some(..)"),
        }
    }

    pub fn tick(&mut self) {
        use PixelAnimationBuildStage::*;
        match self.0 {
            Some(Built { ref params, ref mut animation, ref mut pixels }) => {
                animation.tick(params);
                animation.render(params, pixels);
            }
            _ => panic!("tick called before initLayoutDone"),
        }
    }

    pub fn pixels(&mut self, strip_idx: usize) -> &[u32] {
        use PixelAnimationBuildStage::*;
        match self.0 {
            Some(Built { ref pixels, .. }) => pixels[strip_idx].as_slice(),
            _ => panic!("getPixelVal called before initLayoutDone"),
        }
    }

    pub fn params_mut(&mut self) -> &mut A::Params {
        use PixelAnimationBuildStage::*;
        self.init();
        match self.0 {
            Some(Building { ref mut params, .. }) => params,
            Some(Built { ref mut params, .. }) => params,
            None => unreachable!("init() above leaves self.0 as Some(..)"),
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
