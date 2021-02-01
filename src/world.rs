use crate::canvas::Colour;
use crate::lighting::PointLight;
use crate::matrices::Matrix;
use crate::rays::Ray;
use crate::shapes::{Material, Sphere};
use crate::tuple::Tuple;

pub struct World {
    pub objects: Vec<Sphere>,
    pub lights: Vec<PointLight>,
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
        let s1 = Sphere::new(
            Material {
                colour: Colour::new(0.8, 1.0, 0.6),
                diffuse: 0.7,
                specular: 0.2,
                ..Material::default()
            },
            Matrix::identity(),
        );
        let s2 = Sphere::new(Material::default(), Matrix::scaling(0.5, 0.5, 0.5));
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
