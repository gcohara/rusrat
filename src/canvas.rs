use std::fs::File;
use std::io::Write;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, Default)]
pub struct Colour {
    red: f64,
    green: f64,
    blue: f64,
}

impl Colour {
    pub fn new(red: f64, green: f64, blue: f64) -> Colour {
        Colour { red, green, blue }
    }

    fn component_transform(comp: f64) -> i32 {
        const MAX_VAL: i32 = 255;
        const MIN_VAL: i32 = 0;
        let comp = (comp * MAX_VAL as f64) as i32;
        match comp {
            comp if comp > MAX_VAL => MAX_VAL,
            comp if comp < MIN_VAL => MIN_VAL,
            _ => comp,
        }
    }

    pub fn black() -> Colour {
        Colour::new(0.0, 0.0, 0.0)
    }

    pub fn white() -> Colour {
        Colour::new(1.0, 1.0, 1.0)
    }
}

impl ToString for Colour {
    fn to_string(&self) -> String {
        format![
            "{} {} {}\n",
            Colour::component_transform(self.red),
            Colour::component_transform(self.green),
            Colour::component_transform(self.blue)
        ]
    }
}

impl PartialEq for Colour {
    fn eq(&self, other: &Self) -> bool {
        const EPSILON: f64 = 0.00001;
        let close = |a: f64, b: f64| (a - b).abs() < EPSILON;
        close(self.blue, other.blue) && close(self.green, other.green) && close(self.red, other.red)
    }
}

impl Add for Colour {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Colour::new(
            other.red + self.red,
            other.green + self.green,
            other.blue + self.blue,
        )
    }
}

impl Sub for Colour {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Colour::new(
            self.red - other.red,
            self.green - other.green,
            self.blue - other.blue,
        )
    }
}

impl Mul<f64> for Colour {
    type Output = Self;
    fn mul(self, other: f64) -> Self {
        Colour::new(other * self.red, other * self.green, other * self.blue)
    }
}

impl Mul for Colour {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Colour::new(
            self.red * other.red,
            self.green * other.green,
            self.blue * other.blue,
        )
    }
}
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<Vec<Colour>>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        Canvas {
            width,
            height,
            pixels: vec![vec![Colour::new(0.0, 0.0, 0.0); width]; height],
        }
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> &Colour {
        &self.pixels[y][x]
    }

    pub fn write_pixel(&mut self, (x, y): (usize, usize), colour: Colour) {
        self.pixels[y][x] = colour;
    }
    // Change this to output a result, test it returns correctly
    pub fn write_out_as_ppm_file(&self) {
        let mut outfile = File::create("output.ppm").unwrap();
        outfile.write_all(self.ppm_header().as_bytes()).unwrap();
        outfile.write_all(self.ppm_pixel_data().as_bytes()).unwrap();
    }

    fn ppm_header(&self) -> String {
        format!["P3\n{} {}\n255\n", self.width, self.height]
    }

    fn ppm_pixel_data(&self) -> String {
        self.pixels
            .iter()
            .flatten()
            .map(|pixel| pixel.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_colours() {
        let c1 = Colour::new(0.9, 0.6, 0.75);
        let c2 = Colour::new(0.7, 0.1, 0.25);
        assert_eq!(c1 + c2, Colour::new(1.6, 0.7, 1.0));
    }

    #[test]
    fn subtract_colours() {
        let c1 = Colour::new(0.9, 0.6, 0.75);
        let c2 = Colour::new(0.7, 0.1, 0.25);
        assert_eq!(c1 - c2, Colour::new(0.2, 0.5, 0.5));
    }

    #[test]
    fn multiply_colours() {
        let c1 = Colour::new(0.9, 0.6, 0.75);
        let c2 = Colour::new(0.7, 0.1, 0.25);
        assert_eq!(c1 * c2, Colour::new(0.63, 0.06, 0.1875));
    }

    #[test]
    fn multiply_colour_by_scalar() {
        let c1 = Colour::new(0.9, 0.6, 0.75);
        assert_eq!(c1 * 2.0, Colour::new(1.8, 1.2, 1.5));
    }

    #[test]
    fn write_colour_to_canvas() {
        let mut c = Canvas::new(10, 20);
        let red = Colour::new(1.0, 0.0, 0.0);
        c.write_pixel((2, 3), red);
        assert_eq!(*c.pixel_at(2, 3), red);
    }

    #[test]
    fn ppm_header_is_correct() {
        let c = Canvas::new(5, 3);
        let header = c.ppm_header();
        assert_eq!(header, "P3\n5 3\n255\n");
    }

    #[test]
    fn ppm_pixel_data_is_correct() {
        let mut c = Canvas::new(5, 3);
        let c1 = Colour::new(1.5, 0.0, 0.0);
        let c2 = Colour::new(0.0, 0.5, 0.0);
        let c3 = Colour::new(-0.5, 0.0, 1.0);
        c.write_pixel((0, 0), c1);
        c.write_pixel((2, 1), c2);
        c.write_pixel((4, 2), c3);
        let pix_data = c.ppm_pixel_data();
        assert_eq!(
            pix_data,
            "255 0 0\n0 0 0\n0 0 0\n0 0 0\n0 0 0\n\
             0 0 0\n0 0 0\n0 127 0\n0 0 0\n0 0 0\n\
             0 0 0\n0 0 0\n0 0 0\n0 0 0\n0 0 255\n\
             "
        )
    }

    #[test]
    fn save_ppm_file() {
        let mut c = Canvas::new(5, 3);
        let c1 = Colour::new(1.5, 0.0, 0.0);
        let c2 = Colour::new(0.0, 0.5, 0.0);
        let c3 = Colour::new(-0.5, 0.0, 1.0);
        c.write_pixel((0, 0), c1);
        c.write_pixel((2, 1), c2);
        c.write_pixel((4, 2), c3);
        c.write_out_as_ppm_file();
        assert_eq!(1, 1)
    }
}
