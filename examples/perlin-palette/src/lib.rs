use ledbetter::ledbetter;

#[ledbetter]
mod main {
	use ledbetter::{PixelAnimation, PixelLocation};
	use noise::{NoiseFn, Perlin, Seedable};
	use palette::{rgb::channels::Argb, LinSrgb};
	use palette::gradient::Gradient;
	use rand::prelude::*;
	use std::f64::consts::PI;

	#[ledbetter(params)]
	pub struct Params {
		pub grad_bound: f64,
		pub grad_time_incr: f64,
		pub hue_dist_rads: f64,
	}

	impl Default for Params {
		fn default() -> Self {
			Params {
				grad_bound: 3.0,
				grad_time_incr: 0.02,
				hue_dist_rads: PI,
			}
		}
	}

	pub struct NoiseCycleHsv<NF: NoiseFn<[f64; 2]>> {
		time: f64,
		noise_fns: Vec<NF>,
		gradient: Gradient<LinSrgb>,
	}

	impl<NF: NoiseFn<[f64; 2]> + Seedable> NoiseCycleHsv<NF> {
		fn with_noise_fn(
			pixel_locs: Vec<Vec<PixelLocation>>,
			mut rng: impl Rng,
			make_noise_fn: impl Fn() -> NF,
		) -> Self
		{
			let noise_fns = pixel_locs.iter()
				.map(|_| make_noise_fn().set_seed(rng.gen()))
				.collect();
			let gradient = Gradient::new([
				0xD7263D, 0xD7263D, 0xF46036, 0x2E294E, 0x1B998B, 0xC5D86D, 0xC5D86D,
			]
				.iter()
				.cloned()
				.map(|packed| LinSrgb::from_format(LinSrgb::from(packed)))
				.collect::<Vec<_>>()
			);
			NoiseCycleHsv {
				time: 0.0,
				noise_fns,
				gradient,
			}
		}

		fn tick(&mut self, _params: &Params) {
			self.time += 1.0;
		}

		fn render(&self, params: &Params, pixels: &mut Vec<Vec<u32>>) {
			for (strip, noise_fn) in pixels.iter_mut().zip(self.noise_fns.iter()) {
				if strip.len() == 0 {
					continue;
				}
				let grad_scale = params.grad_bound / ((strip.len() - 1) as f64);
				for (i, pixel_val) in strip.iter_mut().enumerate() {
					let gradient_pt_unscaled = noise_fn.get(
						[i as f64 * grad_scale, self.time * params.grad_time_incr]
					);
					let gradient_pt = (gradient_pt_unscaled + 1.0) / 2.0;
					let col = self.gradient.get(gradient_pt as f32);

					let rgb = <LinSrgb<u8>>::from_format(col);
					*pixel_val = rgb.into_u32::<Argb>();
				}
			}
		}
	}

	pub type PerlinCycleHsv = NoiseCycleHsv<Perlin>;

	#[ledbetter(animation)]
	impl PixelAnimation for PerlinCycleHsv {
		type Params = Params;

		fn new(pixel_locs: Vec<Vec<PixelLocation>>) -> Self {
			let rng = StdRng::from_seed([0; 32]);
			PerlinCycleHsv::with_noise_fn(pixel_locs, rng, Perlin::new)
		}

		fn tick(&mut self, params: &Self::Params) {
			self.tick(params)
		}

		fn render(&self, params: &Self::Params, pixels: &mut Vec<Vec<u32>>) {
			self.render(params, pixels)
		}
	}
}
