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
    reflect_vec: Tuple,
    normal: Tuple,
    t: f64,
    inside: bool,
    over_point: Tuple,
    under_point: Tuple,
    n1: f64,
    n2: f64,
}

impl PointLight {
    pub fn new(intensity: Colour, position: Tuple) -> PointLight {
        PointLight {
            intensity,
            position,
        }
    }
}

pub fn prepare_computations<'a>(
    i: &Intersection<'a>,
    r: &Ray,
    intersections: &Vec<Intersection<'a>>,
) -> PreComputation<'a> {
    const EPSILON: f64 = 0.0000001;
    let p = r.position(i.t);
    let mut out = PreComputation {
        object: i.object,
        t: i.t,
        normal: i.object.normal_at(&p),
        point: p,
        eye_vec: r.direction.negate(),
        reflect_vec: Tuple::vector_new(0.0, 0.0, 0.0),
        inside: false,
        over_point: Tuple::vector_new(0.0, 0.0, 0.0),
        under_point: Tuple::vector_new(0.0, 0.0, 0.0),
        n1: 0.0,
        n2: 0.0,
    };
    if out.normal.dot(&out.eye_vec) < 0.0 {
        out.inside = true;
        out.normal = out.normal.negate();
    };
    // needs to be done after normal is negated (if it is)
    out.reflect_vec = out.normal.reflect(&r.direction);
    out.over_point = out.point + (EPSILON * &out.normal);
    out.under_point = out.point - (EPSILON * &out.normal);

    // this contains objects that have been entered but not yet exited by the ray
    let mut objects_ray_is_inside_of: Vec<&Shape> = Vec::new();
    for intersect in intersections.iter() {
        if i == intersect {
            // then set n1 to the refractive index of either air (1.0) or the last
            // object we entered
            out.n1 = if objects_ray_is_inside_of.is_empty() {
                1.0
            } else {
                objects_ray_is_inside_of
                    .last()
                    .unwrap()
                    .material
                    .refractive_index
            }
        }
        match objects_ray_is_inside_of
            .iter()
            .position(|&obj| &intersect.object == &obj)
        {
            Some(x) => {
                objects_ray_is_inside_of.remove(x);
                ()
            }
            None => {
                objects_ray_is_inside_of.push(&intersect.object);
            }
        }
        if i == intersect {
            // then set n1 to the refractive index of either air (1.0) or the last
            // object we entered
            out.n2 = if objects_ray_is_inside_of.is_empty() {
                1.0
            } else {
                objects_ray_is_inside_of
                    .last()
                    .unwrap()
                    .material
                    .refractive_index
            };
            break;
        }
    }
    out
}

pub fn calculate_lighting(
    material: &Material,
    object: &Shape,
    light: &PointLight,
    posn: &Tuple,
    eye_vec: &Tuple,
    normal: &Tuple,
    in_shadow: bool,
) -> Colour {
    let light_vec = (light.position - *posn).normalise();
    let effective_colour = match &material.pattern {
        None => material.colour * light.intensity,
        Some(p) => p.pattern_at_object(object, posn) * light.intensity,
    };
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

fn shade_hit(w: &World, c: &PreComputation, remaining_recursions: usize) -> Colour {
    let mut out = Colour::new(0.0, 0.0, 0.0);
    for light in &w.lights {
        out = out
            + calculate_lighting(
                &c.object.material,
                &c.object,
                &light,
                // helps prevent chessboard acne
                &c.over_point,
                &c.eye_vec,
                &c.normal,
                // prevent 'acne'
                is_shadowed(&w, &c.over_point),
            );
    }
    let reflected = reflected_colour(w, c, remaining_recursions);
    let refracted = refracted_colour(w, c, remaining_recursions);

    let material = &c.object.material;
    if material.reflectivity > 0.0 && material.transparency > 0.0 {
        let reflectance = schlick(&c);
        out + (reflected * reflectance) + (refracted * (1.0 - reflectance))
    } else {
        out + reflected + refracted
    }
}

pub fn colour_at(w: &World, r: &Ray, remaining_recursions: usize) -> Colour {
    let inters = r.intersects_world(w);
    let hit = Intersection::hit(&inters);
    match hit {
        Some(h) => {
            let comps = prepare_computations(&h, r, &inters);
            shade_hit(w, &comps, remaining_recursions)
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
    match Intersection::hit(&intersections) {
        None => false,
        Some(h) => h.t < distance_to_light,
    }
}

fn reflected_colour(w: &World, c: &PreComputation, remaining_recursions: usize) -> Colour {
    if remaining_recursions <= 0 || c.object.material.reflectivity == 0.0 {
        Colour::new(0.0, 0.0, 0.0)
    } else {
        let reflected_ray = Ray::new(c.over_point, c.reflect_vec);
        let colour = colour_at(&w, &reflected_ray, remaining_recursions - 1);
        colour * c.object.material.reflectivity
    }
}

fn refracted_colour(w: &World, c: &PreComputation, remaining_recursions: usize) -> Colour {
    // check for total internal refraction
    let n_ratio = c.n1 / c.n2;
    let cos_i = c.eye_vec.dot(&c.normal);
    let sin2_t = n_ratio.powi(2) * (1.0 - cos_i.powi(2));
    if c.object.material.transparency == 0.0 || remaining_recursions == 0 || sin2_t > 1.0 {
        Colour::black()
    } else {
        let cos_t = (1.0 - sin2_t).sqrt();
        let dirn = c.normal * (n_ratio * cos_i - cos_t) - c.eye_vec * n_ratio;
        let refracted_ray = Ray::new(c.under_point, dirn);
        colour_at(&w, &refracted_ray, remaining_recursions - 1) * c.object.material.transparency
    }
}

fn schlick(c: &PreComputation) -> f64 {
    let mut cosine = c.eye_vec.dot(&c.normal);
    if c.n1 > c.n2 {
        let n = c.n1 / c.n2;
        let sin2_t = n.powi(2) * (1.0 - cosine.powi(2));
        if sin2_t > 1.0 {
            return 1.0;
        };
        let cos_t = (1.0 - sin2_t).sqrt();
        cosine = cos_t;
    };
    let r0 = ((c.n1 - c.n2) / (c.n1 + c.n2)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::float_eq;
    use crate::matrices::Matrix;
    use crate::shapes::{plane, sphere, TestPattern};

    #[test]
    fn eye_between_light_and_surface() {
        let s = Shape::default();
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 0.0, -10.0),
        );
        let result = calculate_lighting(&m, &s, &light, &posn, &eye_vec, &normal_vec, false);
        assert_eq!(result, Colour::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn eye_between_light_and_surface_eye_offset_45deg() {
        use std::f64::consts::FRAC_1_SQRT_2;
        let s = Shape::default();
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 0.0, -10.0),
        );
        let result = calculate_lighting(&m, &s, &light, &posn, &eye_vec, &normal_vec, false);
        assert_eq!(result, Colour::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn eye_opposite_surface_light_offset_45deg() {
        let s = Shape::default();
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 10.0, -10.0),
        );
        let result = calculate_lighting(&m, &s, &light, &posn, &eye_vec, &normal_vec, false);
        assert_eq!(result, Colour::new(0.7364, 0.7364, 0.7364));
    }

    #[test]
    fn eye_in_path_of_reflection_vector() {
        use std::f64::consts::FRAC_1_SQRT_2;
        let s = Shape::default();
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, -FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 10.0, -10.0),
        );
        let result = calculate_lighting(&m, &s, &light, &posn, &eye_vec, &normal_vec, false);
        assert_eq!(result, Colour::new(1.6364, 1.6364, 1.6364));
    }

    #[test]
    fn lighting_with_light_behind_surface() {
        let s = Shape::default();
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(Colour::new(1.0, 1.0, 1.0), Tuple::point_new(0.0, 0.0, 10.0));
        let result = calculate_lighting(&m, &s, &light, &posn, &eye_vec, &normal_vec, false);
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
        let comps = prepare_computations(&i, &r, &vec![i]);
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
        let comps = prepare_computations(&i, &r, &vec![i]);
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
        let comps = prepare_computations(&i, &r, &vec![i]);
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
        let comp = prepare_computations(&i, &r, &vec![i]);
        let c = shade_hit(&w, &comp, 5);
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
        let comp = prepare_computations(&i, &r, &vec![i]);
        let c = shade_hit(&w, &comp, 5);
        assert_eq!(c, Colour::new(0.90498, 0.90498, 0.90498));
    }

    #[test]
    fn ray_miss_colour() {
        let w = World::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let c = colour_at(&w, &r, 5);
        assert_eq!(c, Colour::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn ray_hit_colour() {
        let w = World::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let c = colour_at(&w, &r, 5);
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
        let c = colour_at(&w, &r, 5);
        assert_eq!(c, inner.material.colour);
    }

    #[test]
    fn lighting_surface_in_shadow() {
        let s = Shape::default();
        let m = Material::default();
        let posn = Tuple::point_new(0.0, 0.0, 0.0);
        let eye_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let normal_vec = Tuple::vector_new(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Colour::new(1.0, 1.0, 1.0),
            Tuple::point_new(0.0, 0.0, -10.0),
        );
        let result = calculate_lighting(&m, &s, &light, &posn, &eye_vec, &normal_vec, true);
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

    #[test]
    fn precomputing_reflection_vector() {
        use std::f64::consts::SQRT_2;
        let pln = plane::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 1.0, -1.0),
            Tuple::vector_new(0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0),
        );
        let i = Intersection::new(SQRT_2, &pln);
        let comps = prepare_computations(&i, &r, &vec![i]);
        assert_eq!(
            comps.reflect_vec,
            Tuple::vector_new(0.0, SQRT_2 / 2.0, SQRT_2 / 2.0)
        );
    }

    #[test]
    fn reflected_colour_for_nonreflective_material() {
        let w = World::default();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let s = &w.objects[1];
        let i = Intersection::new(1.0, s);
        let comps = prepare_computations(&i, &r, &vec![i]);
        let colour = reflected_colour(&w, &comps, 5);
        assert_eq!(colour, Colour::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn reflected_colour_for_reflective_material() {
        use std::f64::consts::SQRT_2;
        let mut w = World::default();
        let pln = Shape {
            material: Material {
                reflectivity: 0.5,
                ..Default::default()
            },
            transform: Matrix::translation(00.0, -1.0, 0.0),
            ..plane::default()
        };
        w.objects.push(pln);
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -3.0),
            Tuple::vector_new(0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0),
        );
        let s = &w.objects[2];
        let i = Intersection::new(SQRT_2, s);
        let comps = prepare_computations(&i, &r, &vec![i]);
        let colour = reflected_colour(&w, &comps, 5);
        assert_eq!(colour, Colour::new(0.19033, 0.23791, 0.14275));
    }

    #[test]
    fn shade_hit_with_reflective_material() {
        use std::f64::consts::SQRT_2;
        let mut w = World::default();
        let pln = Shape {
            material: Material {
                reflectivity: 0.5,
                ..Default::default()
            },
            transform: Matrix::translation(0.0, -1.0, 0.0),
            ..plane::default()
        };
        w.objects.push(pln);
        let s = &w.objects[2];
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -3.0),
            Tuple::vector_new(0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0),
        );
        let i = Intersection::new(SQRT_2, s);
        let comps = prepare_computations(&i, &r, &vec![i]);
        let colour = shade_hit(&w, &comps, 5);
        assert_eq!(colour, Colour::new(0.876756, 0.924338, 0.829173));
    }

    #[test]
    // could do this by spawning a thread with a small stack size
    // std::thread::Builder allows this
    fn colour_at_mutually_recursive_surfaces() {
        let mut w = World::default();
        w.objects.push(Shape {
            material: Material {
                reflectivity: 1.0,
                ..Default::default()
            },
            transform: Matrix::translation(0.0, -1.0, 0.0),
            ..plane::default()
        });
        w.objects.push(Shape {
            material: Material {
                reflectivity: 1.0,
                ..Default::default()
            },
            transform: Matrix::translation(0.0, 1.0, 0.0),
            ..plane::default()
        });
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        // in case of infinite recursion, this will eventually panic (which is the test)
        colour_at(&w, &r, 5);
    }

    #[test]
    fn reflected_colour_at_max_recursion() {
        use std::f64::consts::SQRT_2;
        let mut w = World::default();
        let pln = Shape {
            material: Material {
                reflectivity: 0.5,
                ..Default::default()
            },
            transform: Matrix::translation(0.0, -1.0, 0.0),
            ..plane::default()
        };
        w.objects.push(pln);
        let s = &w.objects[2];
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -3.0),
            Tuple::vector_new(0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0),
        );
        let i = Intersection::new(SQRT_2, s);
        let comps = prepare_computations(&i, &r, &vec![i]);
        let colour = reflected_colour(&w, &comps, 0);
        assert_eq!(colour, Colour::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn finding_n1_and_n2_at_various_intersections() {
        let mut a = sphere::glass_sphere();
        let mut b = sphere::glass_sphere();
        let mut c = sphere::glass_sphere();
        a.transform = Matrix::scaling(2.0, 2.0, 2.0);
        b.transform = Matrix::translation(0.0, 0.0, -0.25);
        c.transform = Matrix::translation(0.0, 0.0, 0.25);
        a.material.refractive_index = 1.5;
        b.material.refractive_index = 2.0;
        c.material.refractive_index = 2.5;
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -4.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let intersections = vec![
            Intersection::new(2.0, &a),
            Intersection::new(2.75, &b),
            Intersection::new(3.25, &c),
            Intersection::new(4.75, &b),
            Intersection::new(5.25, &c),
            Intersection::new(6.0, &a),
        ];
        let refractive_index_vals = vec![1.0, 1.5, 2.0, 2.5, 2.5, 1.5, 1.0];
        for (index, intersection) in intersections.iter().enumerate() {
            let comps = prepare_computations(intersection, &r, &intersections);
            let failstring = format![
                "\n\nFailed on {}, returning n1: {} and n2: {} rather than {} and {}\n",
                index,
                comps.n1,
                comps.n2,
                refractive_index_vals[index],
                refractive_index_vals[index + 1]
            ];
            assert!(float_eq(comps.n1, refractive_index_vals[index]), failstring);
            assert!(
                float_eq(comps.n2, refractive_index_vals[index + 1]),
                failstring
            );
        }
    }

    #[test]
    fn refracted_colour_opaque_surface() {
        let w = World::default();
        let shape = &w.objects[1];
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -5.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let intersections = vec![
            Intersection::new(4.0, &shape),
            Intersection::new(6.0, &shape),
        ];
        let comps = prepare_computations(&intersections[0], &r, &intersections);
        let c = refracted_colour(&w, &comps, 5);
        assert_eq!(c, Colour::black());
    }

    #[test]
    fn refracted_colour_when_total_internal_reflection() {
        use std::f64::consts::SQRT_2;
        let mut w = World::default();
        let shape = &mut w.objects[1];
        shape.material.transparency = 1.0;
        shape.material.refractive_index = 1.5;
        let shape: &Shape = &w.objects[1];
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, SQRT_2 / 2.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let intersections = vec![
            Intersection::new(-SQRT_2 / 2.0, shape),
            Intersection::new(SQRT_2 / 2.0, shape),
        ];
        let comps = prepare_computations(&intersections[1], &r, &intersections);
        let c = refracted_colour(&w, &comps, 5);
        assert_eq!(c, Colour::black());
    }

    #[test]
    fn refracted_colour_with_refracted_ray() {
        let mut w = World::default();
        w.objects[0].material.ambient = 1.0;
        w.objects[0].material.pattern = Some(Box::new(TestPattern::default()));
        w.objects[1].material.transparency = 1.0;
        w.objects[1].material.refractive_index = 1.5;
        let a = &w.objects[0];
        let b = &w.objects[1];
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.1),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let intersections = vec![
            Intersection::new(-0.9899, a),
            Intersection::new(-0.4899, b),
            Intersection::new(0.4899, b),
            Intersection::new(0.9899, a),
        ];
        let comps = prepare_computations(&intersections[2], &r, &intersections);
        let col = refracted_colour(&w, &comps, 5);
        assert_eq!(col, Colour::new(0.0, 0.99888, 0.04722));
    }

    #[test]
    fn shade_hit_with_transparent_material() {
        use std::f64::consts::SQRT_2;
        let mut w = World::default();
        let floor = Shape {
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: Material {
                transparency: 0.5,
                refractive_index: 1.5,
                ..Default::default()
            },
            ..plane::default()
        };
        let ball = Shape {
            transform: Matrix::translation(0.0, -3.5, -0.5),
            material: Material {
                colour: Colour::new(1.0, 0.0, 0.0),
                ambient: 0.5,
                ..Default::default()
            },
            ..sphere::default()
        };
        w.objects.push(floor);
        w.objects.push(ball);
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -3.0),
            Tuple::vector_new(0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0),
        );
        let intersections = vec![Intersection::new(SQRT_2, &w.objects[2])];
        let comps = prepare_computations(&intersections[0], &r, &intersections);
        let colour = shade_hit(&w, &comps, 5);
        assert_eq!(colour, Colour::new(0.93642, 0.68642, 0.68642));
    }

    #[test]
    fn shlick_approximation_under_total_internal_reflection() {
        use std::f64::consts::SQRT_2;
        let sphere = sphere::glass_sphere();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, SQRT_2 / 2.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let intersections = vec![
            Intersection::new(-SQRT_2 / 2.0, &sphere),
            Intersection::new(SQRT_2 / 2.0, &sphere),
        ];
        let comps = prepare_computations(&intersections[1], &r, &intersections);
        let reflectance = schlick(&comps);
        assert!(float_eq(reflectance, 1.0));
    }

    #[test]
    fn shlick_approximation_perpendicular_viewing_angle() {
        let sphere = sphere::glass_sphere();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, 0.0),
            Tuple::vector_new(0.0, 1.0, 0.0),
        );
        let intersections = vec![
            Intersection::new(-1.0, &sphere),
            Intersection::new(1.0, &sphere),
        ];
        let comps = prepare_computations(&intersections[1], &r, &intersections);
        let reflectance = schlick(&comps);
        assert!(float_eq(reflectance, 0.04));
    }

    #[test]
    fn shlick_approximation_small_angle_n2_gt_n1() {
        let sphere = sphere::glass_sphere();
        let r = Ray::new(
            Tuple::point_new(0.0, 0.99, -2.0),
            Tuple::vector_new(0.0, 0.0, 1.0),
        );
        let intersections = vec![Intersection::new(1.8589, &sphere)];
        let comps = prepare_computations(&intersections[0], &r, &intersections);
        let reflectance = schlick(&comps);
        assert!(float_eq(reflectance, 0.48873));
    }

    #[test]
    fn shade_hit_with_reflective_and_transparent_material() {
        use std::f64::consts::SQRT_2;
        let mut w = World::default();
        let floor = Shape {
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: Material {
                reflectivity: 0.5,
                transparency: 0.5,
                refractive_index: 1.5,
                ..Default::default()
            },
            ..plane::default()
        };
        let ball = Shape {
            transform: Matrix::translation(0.0, -3.5, -0.5),
            material: Material {
                colour: Colour::new(1.0, 0.0, 0.0),
                ambient: 0.5,
                ..Default::default()
            },
            ..sphere::default()
        };
        w.objects.push(floor);
        w.objects.push(ball);
        let r = Ray::new(
            Tuple::point_new(0.0, 0.0, -3.0),
            Tuple::vector_new(0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0),
        );
        let intersections = vec![Intersection::new(SQRT_2, &w.objects[2])];
        let comps = prepare_computations(&intersections[0], &r, &intersections);
        let colour = shade_hit(&w, &comps, 5);
        assert_eq!(colour, Colour::new(0.93391, 0.69643, 0.69243));
    }
}
