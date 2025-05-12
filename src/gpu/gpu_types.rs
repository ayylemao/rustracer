use bytemuck::{Pod, Zeroable};

use crate::{Sphere, ray::Ray, shapes::Shape};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuRay {
    pub origin: [f32; 4],
    pub direction: [f32; 4],
}

impl GpuRay {
    pub fn from_ray(ray: &Ray) -> GpuRay {
        let mut origin = [0.0; 4];
        let mut direction = [0.0; 4];
        origin[0] = ray.origin.x;
        origin[1] = ray.origin.y;
        origin[2] = ray.origin.z;
        origin[3] = ray.origin.w;
        direction[0] = ray.direction.x;
        direction[1] = ray.direction.y;
        direction[2] = ray.direction.z;
        direction[3] = ray.direction.w;
        GpuRay { origin, direction }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuShape {
    pub id: u32,
    pub kind: u32,
    pub _padding: [u32; 2],
    inverse: [f32; 16],
    add_data: [f32; 24],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuIntersection {
    pub shape_id: u32,
    pub t: f32,
    pub u: f32,
    pub v: f32,
}

impl GpuShape {
    pub fn from_shape(shape: &dyn Shape) -> GpuShape {
        if let Some(sphere) = shape.as_any().downcast_ref::<Sphere>() {
            let inverse = sphere.inverse();

            let mut inverse_data: [f32; 16] = [0.0; 16];
            let add_data: [f32; 24] = [0.0; 24];

            for i in 0..4 {
                for j in 0..4 {
                    inverse_data[i * 4 + j] = inverse[(i, j)];
                }
            }

            return GpuShape {
                id: sphere.id as u32,
                kind: 1,
                _padding: [0; 2],
                inverse: inverse_data,
                add_data: add_data,
            };
        } else {
            panic!("Shape not yet implemented!")
        };
    }
}

#[cfg(test)]
pub mod tests {
    use std::{f32::consts::PI, sync::Arc};

    use crate::gpu::GPUAccel;
    use crate::ray::Ray;
    use crate::{Sphere, matrix::Matrix, world::World};

    use super::{GpuRay, GpuShape};

    #[test]
    fn init_sphere() {
        let s1 = Sphere::with_transformation(Matrix::rotation_x(PI / 2.0));
        let mut world = World::default();

        world.add_shape(Arc::new(s1));

        let s1 = world.shapes[2].as_ref();

        let gpu_sphere = GpuShape::from_shape(s1);
        println!("{:?}", gpu_sphere);
    }

    #[test]
    pub fn sphere_intersect() {
        let mut gpu_accel = GPUAccel::new("src/gpu/shader.wgsl");

        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let trans = Matrix::scaling(2.0, 2.0, 2.0);
        let mut s1 = Sphere::with_transformation(trans);
        s1.id = 1;

        let gpuray = GpuRay::from_ray(&ray);
        let gpusphere = GpuShape::from_shape(Arc::new(s1).as_ref());

        let rays = vec![gpuray];
        let shapes: Vec<GpuShape> = vec![gpusphere];

        gpu_accel.upload_rays_and_shapes(&rays, &shapes);

        gpu_accel.dispatch();

        let result = gpu_accel.download_intersections();
        let intersections = GPUAccel::get_hits_for_ray(&result, 0);

        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].t, 3.0);
        assert_eq!(intersections[1].t, 7.0);
    }
}
