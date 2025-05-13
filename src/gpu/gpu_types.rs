use bytemuck::{Pod, Zeroable};

use crate::{
    Sphere,
    ray::Ray,
    shapes::{Shape, smooth_triangle::SmoothTriangle},
};

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
    add_data: [f32; 40],
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
            let add_data: [f32; 40] = [0.0; 40];

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
        } else if let Some(smooth_tri) = shape.as_any().downcast_ref::<SmoothTriangle>() {
            let inverse = smooth_tri.inverse();

            let mut inverse_data: [f32; 16] = [0.0; 16];
            let mut add_data: [f32; 40] = [0.0; 40];

            for i in 0..4 {
                for j in 0..4 {
                    inverse_data[i * 4 + j] = inverse[(i, j)];
                }
            }
            add_data[0..3].copy_from_slice(&[smooth_tri.p1.x, smooth_tri.p1.y, smooth_tri.p1.z]);
            add_data[4..7].copy_from_slice(&[smooth_tri.p2.x, smooth_tri.p2.y, smooth_tri.p2.z]);
            add_data[8..11].copy_from_slice(&[smooth_tri.p3.x, smooth_tri.p3.y, smooth_tri.p3.z]);
            add_data[12..15].copy_from_slice(&[smooth_tri.n1.x, smooth_tri.n1.y, smooth_tri.n1.z]);
            add_data[16..19].copy_from_slice(&[smooth_tri.n2.x, smooth_tri.n2.y, smooth_tri.n2.z]);
            add_data[20..23].copy_from_slice(&[smooth_tri.n3.x, smooth_tri.n3.y, smooth_tri.n3.z]);
            add_data[24..27].copy_from_slice(&[smooth_tri.e1.x, smooth_tri.e1.y, smooth_tri.e1.z]);
            add_data[28..31].copy_from_slice(&[smooth_tri.e2.x, smooth_tri.e2.y, smooth_tri.e2.z]);
            add_data[32..35].copy_from_slice(&[
                smooth_tri.normal.x,
                smooth_tri.normal.y,
                smooth_tri.normal.z,
            ]);
            return GpuShape {
                id: smooth_tri.id as u32,
                kind: 2,
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
    use crate::shapes::Shape;
    use crate::shapes::smooth_triangle::SmoothTriangle;
    use crate::vec4::Vec4;
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
        let s1 = Sphere::with_transformation(trans);

        let gpuray = GpuRay::from_ray(&ray);
        let gpu_sphere: Vec<Arc<dyn Shape + Send + Sync>> = vec![Arc::new(s1)];

        let rays = vec![gpuray];

        gpu_accel.populate_shapes(&gpu_sphere);

        gpu_accel.upload_rays_and_shapes(&rays);

        gpu_accel.dispatch();

        let result = gpu_accel.download_intersections();
        let intersections = gpu_accel.get_hits_for_ray(&result, 0);

        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0].t, 3.0);
        assert_eq!(intersections[1].t, 7.0);
    }

    #[test]
    fn smooth_tri_gpu() {
        let mut gpu_accel = GPUAccel::new("src/gpu/shader.wgsl");
        let p1 = Vec4::point(0.0, 1.0, 0.0);
        let p2 = Vec4::point(-1.0, 0.0, 0.0);
        let p3 = Vec4::point(1.0, 0.0, 0.0);
        let t = SmoothTriangle::new(p1, p2, p3, p1, p2, p3);

        let r = Ray::new(0.0, 0.5, -2.0, 0.0, 0.0, 1.0);

        let gpuray = GpuRay::from_ray(&r);
        let gpu_sphere: Vec<Arc<dyn Shape + Send + Sync>> = vec![Arc::new(t)];

        let rays = vec![gpuray];
        gpu_accel.populate_shapes(&gpu_sphere);

        gpu_accel.upload_rays_and_shapes(&rays);

        gpu_accel.dispatch();

        let result = gpu_accel.download_intersections();
        let intersections = gpu_accel.get_hits_for_ray(&result, 0);
        println!("{:?}", intersections);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].t, 2.0);

        let r = Ray::new(-1.0, 1.0, -2.0, 0.0, 0.0, 1.0);
        let gpuray = GpuRay::from_ray(&r);
        let rays = vec![gpuray];

        gpu_accel.upload_rays_and_shapes(&rays);
        let result = gpu_accel.download_intersections();
        let intersections = gpu_accel.get_hits_for_ray(&result, 0);
        assert_eq!(intersections.len(), 0);
    }
}
