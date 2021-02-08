use crate::canvas::{Canvas, Colour};
use crate::lighting::{colour_at, PointLight};
use crate::matrices::Matrix;
use crate::rays::Ray;
use crate::shapes::{plane, sphere, Material, Shape};
use crate::tuple::Tuple;
use itertools::iproduct;

pub struct World {
    pub objects: Vec<Shape>,
    pub lights: Vec<PointLight>,
}

#[derive(Default)]
pub struct Camera {
    hsize: usize,
    vsize: usize,
    field_of_view: f64,
    transform: Matrix<f64, 4, 4>,
    // cache/memoise these values
    pixel_size: Option<f64>,
    half_width: Option<f64>,
    half_height: Option<f64>,
}

impl Camera {
    pub fn new(hsize: usize, vsize: usize, fov: f64, t: Matrix<f64, 4, 4>) -> Camera {
        Camera {
            hsize,
            vsize,
            field_of_view: fov,
            transform: t,
            pixel_size: None,
            half_width: None,
            ..Default::default()
        }
    }

    fn half_width(&mut self) -> f64 {
        match self.half_width {
            Some(hw) => hw,
            None => {
                let half_view = (self.field_of_view / 2.0).tan();
                let aspect = self.hsize as f64 / self.vsize as f64;
                let half_width = if aspect >= 1.0 {
                    half_view
                } else {
                    half_view * aspect
                };
                self.half_width = Some(half_width);
                half_width
            }
        }
    }

    fn half_height(&mut self) -> f64 {
        match self.half_height {
            Some(hh) => hh,
            None => {
                let half_view = (self.field_of_view / 2.0).tan();
                let aspect = self.hsize as f64 / self.vsize as f64;
                let half_height = if aspect >= 1.0 {
                    half_view / aspect
                } else {
                    half_view
                };
                self.half_height = Some(half_height);
                half_height
            }
        }
    }

    fn pixel_size(&mut self) -> f64 {
        match self.pixel_size {
            Some(ps) => ps,
            None => {
                let ps = self.half_width() * 2.0 / self.hsize as f64;
                self.pixel_size = Some(ps);
                ps
            }
        }
    }

    pub fn ray_for_pixel(&mut self, x: usize, y: usize) -> Ray {
        let x_offset = (x as f64 + 0.5) * self.pixel_size();
        let y_offset = (y as f64 + 0.5) * self.pixel_size();
        let world_x = self.half_width() - x_offset;
        let world_y = self.half_height() - y_offset;
        let px = self.transform.inverse() * &Tuple::point_new(world_x, world_y, -1.0);
        let origin = self.transform.inverse() * &Tuple::point_new(0.0, 0.0, 0.0);
        let direction = (px - origin).normalise();
        Ray::new(origin, direction)
    }
}

impl World {
    pub fn new() -> World {
        World {
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }
}

impl Default for World {
    fn default() -> World {
        let s1 = Shape {
            material: Material {
                colour: Colour::new(0.8, 1.0, 0.6),
                diffuse: 0.7,
                specular: 0.2,
                ..Material::default()
            },
            transform: Matrix::identity(),
            ..sphere::default()
        };
        let s2 = Shape {
            transform: Matrix::scaling(0.5, 0.5, 0.5),
            ..sphere::default()
        };
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(-10.0, 10.0, -10.0),
        );

        World {
            objects: vec![s1, s2],
            lights: vec![light],
        }
    }
}

pub fn view_transform(from: &Tuple, to: &Tuple, up: &Tuple) -> Matrix<f64, 4, 4> {
    let forward = (*to - *from).normalise();
    let left = forward.cross(&up.normalise());
    let true_up = left.cross(&forward);
    let orientation = Matrix::from_array(&[
        [left.x, left.y, left.z, 0.0],
        [true_up.x, true_up.y, true_up.z, 0.0],
        [-forward.x, -forward.y, -forward.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    orientation * Matrix::translation(-from.x, -from.y, -from.z)
}

pub fn render(cam: &mut Camera, world: &World) -> Canvas {
    let mut image = Canvas::new(cam.hsize, cam.vsize);
    for (x, y) in iproduct!(0..cam.hsize - 1, 0..cam.vsize - 1) {
        let ray = cam.ray_for_pixel(x, y);
        let colour = colour_at(world, &ray);
        image.write_pixel((x, y), colour);
    }
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    fn float_close(x: f64, y: f64) -> bool {
        const EPSILON: f64 = 0.0001;
        (x - y).abs() < EPSILON
    }

    #[test]
    fn intersect_world_with_ray() {
        let w = World::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let intersections = r.intersects_world(&w);
        assert_eq!(intersections.len(), 4);
        assert_eq!(intersections[0].t, 4.0);
        assert_eq!(intersections[1].t, 4.5);
        assert_eq!(intersections[2].t, 5.5);
        assert_eq!(intersections[3].t, 6.0);
    }

    #[test]
    fn default_view_transformation() {
        let t = view_transform(
            &Tuple::point_new(0.0, 0.0, 0.0),
            &Tuple::point_new(0.0, 0.0, -1.0),
            &Tuple::vector_new(0.0, 1.0, 0.0),
        );
        assert_eq!(t, Matrix::identity());
    }

    #[test]
    fn view_transform_positive_z_direction() {
        let t = view_transform(
            &Tuple::point_new(0.0, 0.0, 0.0),
            &Tuple::point_new(0.0, 0.0, 1.0),
            &Tuple::vector_new(0.0, 1.0, 0.0),
        );
        assert_eq!(t, Matrix::scaling(-1.0, 1.0, -1.0));
    }

    #[test]
    fn view_transform_moves_world() {
        let t = view_transform(
            &Tuple::point_new(0.0, 0.0, 8.0),
            &Tuple::point_new(0.0, 0.0, 0.0),
            &Tuple::vector_new(0.0, 1.0, 0.0),
        );
        assert_eq!(t, Matrix::translation(0.0, 0.0, -8.0));
    }

    #[test]
    fn arbitrary_view_transform() {
        let t = view_transform(
            &Tuple::point_new(1.0, 3.0, 2.0),
            &Tuple::point_new(4.0, -2.0, 8.0),
            &Tuple::vector_new(1.0, 1.0, 0.0),
        );
        let expected = Matrix::from_array(&[
            [-0.50709, 0.50709, 0.67612, -2.36643],
            [0.76772, 0.60609, 0.12122, -2.82843],
            [-0.35857, 0.59761, -0.71714, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert_eq!(t, expected);
    }

    #[test]
    fn camera_pixel_size_horizontal() {
        use std::f64::consts::FRAC_PI_2;
        let mut c = Camera::new(200, 125, FRAC_PI_2, Matrix::identity());
        assert!(float_close(c.pixel_size(), 0.01));
    }

    #[test]
    fn camera_pixel_size_vertical() {
        use std::f64::consts::FRAC_PI_2;
        let mut c = Camera::new(125, 200, FRAC_PI_2, Matrix::identity());
        assert!(float_close(c.pixel_size(), 0.01));
    }

    #[test]
    fn ray_through_centre_of_canvas() {
        use std::f64::consts::FRAC_PI_2;
        let mut c = Camera::new(201, 101, FRAC_PI_2, Matrix::identity());
        println!("{}", c.pixel_size());
        let r = c.ray_for_pixel(100, 50);
        assert_eq!(r.origin, Tuple::point_new(0.0, 0.0, 0.0));
        assert_eq!(r.direction, Tuple::vector_new(0.0, 0.0, -1.0));
    }

    #[test]
    fn ray_through_corner_of_canvas() {
        use std::f64::consts::FRAC_PI_2;
        let mut c = Camera::new(201, 101, FRAC_PI_2, Matrix::identity());
        let r = c.ray_for_pixel(0, 0);
        assert_eq!(r.origin, Tuple::point_new(0.0, 0.0, 0.0));
        assert_eq!(r.direction, Tuple::vector_new(0.66519, 0.33259, -0.66851));
    }

    #[test]
    fn ray_when_camera_transformed() {
        use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4};
        let mut c = Camera::new(
            201,
            101,
            FRAC_PI_2,
            Matrix::translation(0.0, -2.0, 5.0).rotate_y(FRAC_PI_4),
        );
        let r = c.ray_for_pixel(100, 50);
        assert_eq!(r.origin, Tuple::point_new(0.0, 2.0, -5.0));
        assert_eq!(
            r.direction,
            Tuple::vector_new(FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2)
        );
    }

    #[test]
    fn rendering_world_with_camera() {
        use std::f64::consts::FRAC_PI_2;
        let w = World::default();
        let t = view_transform(
            &Tuple::point_new(0.0, 0.0, -5.0),
            &Tuple::point_new(0.0, 0.0, 0.0),
            &Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let mut c = Camera::new(11, 11, FRAC_PI_2, t);
        let image = render(&mut c, &w);
        assert_eq!(*image.pixel_at(5, 5), Colour::new(0.38066, 0.47583, 0.2855));
    }
}
