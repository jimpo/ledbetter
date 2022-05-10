use ledbetter::ledbetter;

#[ledbetter]
mod main {
	use ledbetter::{PixelAnimation, PixelLocation};

	#[ledbetter(params)]
	#[derive(Default)]
	pub struct Params {
		pub red: u32,
		pub grn: u32,
		pub blu: u32,
	}

	pub struct OneColor;

	#[ledbetter(animation)]
	impl PixelAnimation for OneColor {
		type Params = Params;

		fn new(_pixel_locs: Vec<Vec<PixelLocation>>) -> Self {
			OneColor
		}

		fn tick(&mut self, _params: &Self::Params) {
		}

		fn render(&self, params: &Self::Params, pixels: &mut Vec<Vec<u32>>) {
			let red = params.red.min(255);
			let grn = params.red.min(255);
			let blu = params.red.min(255);
			let color = (red << 16) | (grn << 8) | (blu << 0);
			for strip in pixels.iter_mut() {
				for pixel in strip.iter_mut() {
					*pixel = color;
				}
			}
		}
	}
}