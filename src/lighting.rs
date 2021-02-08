use crate::canvas::Colour;
use crate::rays::{Intersection, Ray};
use crate::shapes::{Material, Shape};
use crate::tuple::Tuple;
use crate::world::World;

pub struct PointLight {
    intensity: Colour,
    position: Tuple,
}

pub struct PreComputation<'a> {
    object: &'a Shape,
    point: Tuple,
    eye_vec: Tuple,
    // reflect_vec: Tuple,
    normal: Tuple,
    t: f64,
    inside: bool,
}

impl PointLight {
    pub fn new(intensity: Colour, position: Tuple) -> PointLight {
        PointLight {
            intensity,
            position,
        }
    }
}

pub fn prepare_computations<'a>(i: &Intersection<'a>, r: &Ray) -> PreComputation<'a> {
    let p = r.position(i.t);
    let mut out = PreComputation {
        object: i.object,
        t: i.t,
        normal: i.object.normal_at(&p),
        point: p,
        eye_vec: r.direction.negate(),
        // reflect_vec:
        inside: false,
    };
    if out.normal.dot(&out.eye_vec) < 0.0 {
        out.inside = true;
        out.normal = out.normal.negate();
    };
    out
}

pub fn calculate_lighting(
    material: &Material,
    light: &PointLight,
    posn: &Tuple,
    eye_vec: &Tuple,
    normal: &Tuple,
    in_shadow: bool,
) -> Colour {
    let light_vec = (light.position - *posn).normalise();
    let effective_colour = material.colour * light.intensity;

    let ambient_term = effective_colour * material.ambient;
    match in_shadow {
        true => ambient_term,
        false => {
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
    }
}

fn shade_hit(w: &World, c: &PreComputation) -> Colour {
    const EPSILON: f64 = 0.0000001;
    let mut out = Colour::new(0.0, 0.0, 0.0);
    for light in &w.lights {
        out = out
            + calculate_lighting(
                &c.object.material,
                &light,
                &c.point,
                &c.eye_vec,
                &c.normal,
                // prevent 'acne'
                is_shadowed(&w, &(c.point + (EPSILON * &c.normal))),
            );
    }
    out
}

pub fn colour_at(w: &World, r: &Ray) -> Colour {
    let inters = r.intersects_world(w);
    let hit = Intersection::hit(inters);
    match hit {
        Some(h) => {
            let comps = prepare_computations(&h, r);
            shade_hit(w, &comps)
        }
        None => Colour::new(0.0, 0.0, 0.0),
    }
}

fn is_shadowed(w: &World, p: &Tuple) -> bool {
    // need to adjust for multiple lights
    let point_to_light = w.lights[0].position - *p;
    let distance_to_light = point_to_light.magnitude();
    let point_to_light_ray = Ray::new(*p, point_to_light.normalise());
    let intersections = point_to_light_ray.intersects_world(w);
    match Intersection::hit(intersections) {
        None => false,
        Some(h) => h.t < distance_to_light,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{plane, sphere};

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
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec, false);
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
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec, false);
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
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec, false);
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
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec, false);
        assert_eq!(result, Colour::new(1.6364, 1.6364, 1.6364));
    }

    #[test]
    fn lighting_with_light_behind_surface() {
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(Colour::new(1.0, 1.0, 1.0), Tuple::point_new(0.0, 0.0, 10.0));
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec, false);
        assert_eq!(result, Colour::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn precomputating_state_of_intersection() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let i = Intersection::new(4.0, &s);
        let comps = prepare_computations(&i, &r);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.object, i.object);
        assert_eq!(comps.eye_vec, Tuple::vector_new(0.0, 0.0, -1.0));
        assert_eq!(comps.normal, Tuple::vector_new(0.0, 0.0, -1.0));
        assert_eq!(comps.point, Tuple::point_new(0.0, 0.0, -1.0));
    }

    #[test]
    fn hit_on_outside_of_shape() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let i = Intersection::new(4.0, &s);
        let comps = prepare_computations(&i, &r);
        assert!(!comps.inside);
    }

    #[test]
    fn hit_on_inside_of_shape() {
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = sphere::default();
        let i = Intersection::new(4.0, &s);
        let comps = prepare_computations(&i, &r);
        assert!(comps.inside);
    }

    #[test]
    fn shading_an_intersection() {
        let w = World::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = &w.objects[0];
        let i = Intersection::new(4.0, s);
        let comp = prepare_computations(&i, &r);
        let c = shade_hit(&w, &comp);
        assert_eq!(c, Colour::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn shading_an_intersection_from_inside() {
        let mut w = World::default();
        w.lights[0] = PointLight::new(Colour::new(1.0, 1.0, 1.0), Tuple::point_new(0.0, 0.25, 0.0));
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = &w.objects[1];
        let i = Intersection::new(0.5, s);
        let comp = prepare_computations(&i, &r);
        let c = shade_hit(&w, &comp);
        assert_eq!(c, Colour::new(0.90498, 0.90498, 0.90498));
    }

    #[test]
    fn ray_miss_colour() {
        let w = World::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let c = colour_at(&w, &r);
        assert_eq!(c, Colour::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn ray_hit_colour() {
        let w = World::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let c = colour_at(&w, &r);
        assert_eq!(c, Colour::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn colour_intersection_behind_ray() {
        let mut w = World::default();
        let outer = &mut w.objects[0];
        outer.material.ambient = 1.0;
        let inner = &mut w.objects[1];
        inner.material.ambient = 1.0;
        let inner = &w.objects[1];
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.75),
            Tuple::vector_new(0.0, 0.0, -1.0),
        );
        let c = colour_at(&w, &r);
        assert_eq!(c, inner.material.colour);
    }

    #[test]
    fn lighting_surface_in_shadow() {
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 0.0, -10.0),
        );
        let result = calculate_lighting(&m, &light, &posn, &eye_vec, &normal_vec, true);
        assert_eq!(result, Colour::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn no_shadow_when_nothing_between_point_and_light() {
        let w = World::default();
        let p = Tuple::point_new(0.0, 10.0, 0.0);
        assert!(!is_shadowed(&w, &p));
    }

    #[test]
    fn shadow_when_object_between_point_and_light() {
        let w = World::default();
        let p = Tuple::point_new(10.0, -10.0, 10.0);
        assert!(is_shadowed(&w, &p));
    }

    #[test]
    fn no_shadow_when_object_behind_light() {
        let w = World::default();
        let p = Tuple::point_new(-20.0, 20.0, -20.0);
        assert!(!is_shadowed(&w, &p));
    }

    #[test]
    fn no_shadow_when_object_behind_point() {
        let w = World::default();
        let p = Tuple::point_new(-20.0, 20.0, -20.0);
        assert!(!is_shadowed(&w, &p));
    }
}
