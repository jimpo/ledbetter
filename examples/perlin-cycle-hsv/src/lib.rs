use ledbetter::ledbetter;

#[ledbetter]
mod main {
	use ledbetter::{PixelAnimation, PixelLocation};
	use noise::{NoiseFn, Perlin, Seedable};
	use palette::{encoding::Srgb, rgb::{channels::Argb, Rgb}, FromColor, Hsv, RgbHue};
	use num_traits::Pow;
	use rand::prelude::*;
	use std::f64::consts::PI;

	#[ledbetter(params)]
	pub struct Params {
		pub hue_bound: f64,
		pub val_bound: f64,
		pub hue_time_incr: f64,
		pub val_time_incr: f64,
		pub base_hue_incr: f64,
		pub hue_dist_rads: f64,
	}

	impl Default for Params {
		fn default() -> Self {
			Params {
				hue_bound: 3.0,
				val_bound: 5.0,
				hue_time_incr: 0.02,
				val_time_incr: 0.02,
				base_hue_incr: 0.5,
				hue_dist_rads: PI,
			}
		}
	}

	pub struct NoiseCycleHsv<NF: NoiseFn<[f64; 3]>> {
		base_hue: RgbHue<f64>,
		time: f64,
		noise_fns: Vec<NF>,
	}

	impl<NF: NoiseFn<[f64; 3]> + Seedable> NoiseCycleHsv<NF> {
		fn with_noise_fn(
			pixel_locs: Vec<Vec<PixelLocation>>,
			mut rng: impl Rng,
			make_noise_fn: impl Fn() -> NF,
		) -> Self
		{
			let noise_fns = pixel_locs.iter()
				.map(|_| make_noise_fn().set_seed(rng.gen()))
				.collect();
			NoiseCycleHsv {
				base_hue: RgbHue::from_degrees(0.0),
				time: 0.0,
				noise_fns,
			}
		}

		fn tick(&mut self, params: &Params) {
			self.time += 1.0;
			self.base_hue += RgbHue::from_degrees(params.base_hue_incr);
		}

		fn render(&self, params: &Params, pixels: &mut Vec<Vec<u32>>) {
			for (strip, noise_fn) in pixels.iter_mut().zip(self.noise_fns.iter()) {
				if strip.len() == 0 {
					continue;
				}
				let hue_scale = params.hue_bound / ((strip.len() - 1) as f64);
				let val_scale = params.val_bound / ((strip.len() - 1) as f64);
				for (i, pixel_val) in strip.iter_mut().enumerate() {
					let hue_unscaled = noise_fn.get(
						[i as f64 * hue_scale, 0.0, self.time * params.hue_time_incr]
					);
					let val_unscaled = noise_fn.get(
						[0.0, i as f64 * val_scale, self.time * params.val_time_incr]
					);
					let hue = self.base_hue + RgbHue::from_radians(hue_unscaled * params.hue_dist_rads);
					let val = ((val_unscaled + 1.0) / 2.0).pow(2);

					let hsv = Hsv::new(hue, 1.0, val);
					let rgb = <Rgb<Srgb, u8>>::from_format(Rgb::from_color(hsv));
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
