use ledbetter::ledbetter;

#[ledbetter]
mod main {
    use ledbetter::{PixelAnimation, PixelLocation};
    use palette::{encoding::Srgb, Hsv, rgb::{channels::Argb, Rgb}, RgbHue, FromColor};
    use std::f32::consts::PI;

    type Rgb8 = Rgb<Srgb, u8>;

    #[ledbetter(params)]
    pub struct Params {
        pub min_hue: f32,
        pub max_hue: f32,
        pub speed: u32,
    }

    impl Default for Params {
        fn default() -> Self {
            Params {
                min_hue: 140.0,
                max_hue: 220.0,
                speed: 50,
            }
        }
    }

    pub struct Blue {
        steps: usize,
        color: Rgb8,
    }

    #[ledbetter(animation)]
    impl PixelAnimation for Blue {
        type Params = Params;

        fn new(_pixel_locs: Vec<Vec<PixelLocation>>) -> Self {
            Blue {
                steps: 0,
                color: Rgb::new(0, 0, 0),
            }
        }

        fn tick(&mut self, params: &Params) {
            let hue = (1.0 - (self.steps as f32 * 2.0 * PI / params.speed as f32).cos()) / 2.0
                * (params.max_hue - params.min_hue)
                + params.min_hue;
            let hsv = Hsv::new(RgbHue::from_degrees(hue), 1.0, 1.0);
            self.color = Rgb8::from_format(Rgb::from_color(hsv));
            self.steps += 1;
        }

        fn render(&self, _params: &Params, pixels: &mut Vec<Vec<u32>>) {
            for strip in pixels.iter_mut() {
                for pixel in strip.iter_mut() {
                    *pixel = self.color.into_u32::<Argb>();
                }
            }
        }
    }
}