use crate::canvas::Colour;
use crate::shapes::Material;
use crate::tuple::Tuple;

pub struct PointLight {
    intensity: Colour,
    position: Tuple,
}

impl PointLight {
    pub fn new(intensity: Colour, position: Tuple) -> PointLight {
        PointLight {
            intensity,
            position,
        }
    }
}

pub fn calculate_lighting(
    material: &Material,
    light: &PointLight,
    posn: &Tuple,
    eye_vec: &Tuple,
    normal: &Tuple,
) -> Colour {
    let light_vec = (light.position - *posn).normalise();
    let effective_colour = material.colour * light.intensity;

    let ambient_term = effective_colour * material.ambient;
    let light_normal_dot = light_vec.dot(normal);
    let diffuse = if light_normal_dot < 0.0 {
        Colour::new(0.0, 0.0, 0.0)
    } else {
        effective_colour * material.diffuse * light_normal_dot
    };

    let specular = if light_normal_dot < 0.0 {
        Colour::new(0.0, 0.0, 0.0)
    } else {
        let reflect_vec = normal.reflect(&light_vec.negate());
        let reflect_eye_dot = reflect_vec.dot(eye_vec);
        if reflect_eye_dot <= 0.0 {
            Colour::new(0.0, 0.0, 0.0)
        } else {
            light.intensity * material.specular * reflect_eye_dot.powf(material.shininess)
        }
    };

    ambient_term + diffuse + specular
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eye_between_light_and_surface() {
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 0.0, -10.0),
        );
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec);
        assert_eq!(result, Colour::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn eye_between_light_and_surface_eye_offset_45deg() {
        use std::f64::consts::FRAC_1_SQRT_2;
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 0.0, -10.0),
        );
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec);
        assert_eq!(result, Colour::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn eye_opposite_surface_light_offset_45deg() {
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 10.0, -10.0),
        );
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec);
        assert_eq!(result, Colour::new(0.7364, 0.7364, 0.7364));
    }

    #[test]
    fn eye_in_path_of_reflection_vector() {
        use std::f64::consts::FRAC_1_SQRT_2;
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, -FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 10.0, -10.0),
        );
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec);
        assert_eq!(result, Colour::new(1.6364, 1.6364, 1.6364));
    }

    #[test]
    fn lighting_with_light_behind_surface() {
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(Colour::new(1.0, 1.0, 1.0), Tuple::point_new(0.0, 0.0, 10.0));
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec);
        assert_eq!(result, Colour::new(0.1, 0.1, 0.1));
    }
}
