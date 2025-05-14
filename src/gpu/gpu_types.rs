use bytemuck::{Pod, Zeroable};

use crate::{
    Sphere,
    ray::Ray,
    shapes::{Shape, plane::Plane, smooth_triangle::SmoothTriangle},
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
        let mut gpu_shape = GpuShape {
            id: 0,
            kind: 0,
            _padding: [0; 2],
            inverse: [0.0; 16],
            add_data: [0.0; 40],
        };
        let inverse = shape.inverse();

        for i in 0..4 {
            for j in 0..4 {
                gpu_shape.inverse[j * 4 + i] = inverse[(i, j)];
            }
        }

        if let Some(sphere) = shape.as_any().downcast_ref::<Sphere>() {
            gpu_shape.kind = 1;
            gpu_shape.id = sphere.id as u32;
            return gpu_shape;
        } else if let Some(smooth_tri) = shape.as_any().downcast_ref::<SmoothTriangle>() {
            gpu_shape.add_data[0..3].copy_from_slice(&[
                smooth_tri.p1.x,
                smooth_tri.p1.y,
                smooth_tri.p1.z,
            ]);
            gpu_shape.add_data[4..7].copy_from_slice(&[
                smooth_tri.p2.x,
                smooth_tri.p2.y,
                smooth_tri.p2.z,
            ]);
            gpu_shape.add_data[8..11].copy_from_slice(&[
                smooth_tri.p3.x,
                smooth_tri.p3.y,
                smooth_tri.p3.z,
            ]);
            gpu_shape.add_data[12..15].copy_from_slice(&[
                smooth_tri.n1.x,
                smooth_tri.n1.y,
                smooth_tri.n1.z,
            ]);
            gpu_shape.add_data[16..19].copy_from_slice(&[
                smooth_tri.n2.x,
                smooth_tri.n2.y,
                smooth_tri.n2.z,
            ]);
            gpu_shape.add_data[20..23].copy_from_slice(&[
                smooth_tri.n3.x,
                smooth_tri.n3.y,
                smooth_tri.n3.z,
            ]);
            gpu_shape.add_data[24..27].copy_from_slice(&[
                smooth_tri.e1.x,
                smooth_tri.e1.y,
                smooth_tri.e1.z,
            ]);
            gpu_shape.add_data[28..31].copy_from_slice(&[
                smooth_tri.e2.x,
                smooth_tri.e2.y,
                smooth_tri.e2.z,
            ]);
            gpu_shape.add_data[32..35].copy_from_slice(&[
                smooth_tri.normal.x,
                smooth_tri.normal.y,
                smooth_tri.normal.z,
            ]);
            gpu_shape.id = smooth_tri.id as u32;
            gpu_shape.kind = 2;
            return gpu_shape;
        } else if let Some(plane) = shape.as_any().downcast_ref::<Plane>() {
            gpu_shape.id = plane.id as u32;
            gpu_shape.kind = 3;
            return gpu_shape;
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
    use crate::shapes::plane::Plane;
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

        let rays = vec![gpuray];

        let mut w = World::default();
        w.shapes.clear();
        w.add_shape(Arc::new(s1));
        gpu_accel.populate_shapes(&w.shapes);

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

        let rays = vec![gpuray];
        let mut w = World::default();
        w.shapes.clear();
        w.add_shape(Arc::new(t));
        gpu_accel.populate_shapes(&w.shapes);

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

    #[test]
    fn plane_intersect() {
        let mut gpu_accel = GPUAccel::new("src/gpu/shader.wgsl");

        let p = Plane::new();

        let r = Ray::from_vec4(Vec4::point(0.0, 10.0, 0.0), Vec4::vector(0.0, 0.0, 1.0));

        let gpuray = GpuRay::from_ray(&r);
        let rays = vec![gpuray];
        let mut w = World::default();
        w.shapes.clear();
        w.add_shape(Arc::new(p));
        gpu_accel.populate_shapes(&w.shapes);
        gpu_accel.upload_rays_and_shapes(&rays);
        gpu_accel.dispatch();

        let result = gpu_accel.download_intersections();
        let intersections = gpu_accel.get_hits_for_ray(&result, 0);
        assert_eq!(intersections.len(), 0);

        let r = Ray::from_vec4(Vec4::point(0.0, 0.0, 0.0), Vec4::vector(0.0, 0.0, 1.0));

        let p = Plane::new();
        let mut gpu_accel = GPUAccel::new("src/gpu/shader.wgsl");
        let gpuray = GpuRay::from_ray(&r);
        let rays = vec![gpuray];
        let mut w = World::default();
        w.shapes.clear();
        w.add_shape(Arc::new(p));
        gpu_accel.populate_shapes(&w.shapes);
        gpu_accel.upload_rays_and_shapes(&rays);
        gpu_accel.dispatch();

        let result = gpu_accel.download_intersections();
        let intersections = gpu_accel.get_hits_for_ray(&result, 0);
        assert_eq!(intersections.len(), 0);

        let p = Plane::new();
        let r = Ray::from_vec4(Vec4::point(0.0, 1.0, 0.0), Vec4::vector(0.0, -1.0, 0.0));

        let mut gpu_accel = GPUAccel::new("src/gpu/shader.wgsl");
        let gpuray = GpuRay::from_ray(&r);
        let rays = vec![gpuray];
        let mut w = World::default();
        w.shapes.clear();
        w.add_shape(Arc::new(p));
        gpu_accel.populate_shapes(&w.shapes);
        gpu_accel.upload_rays_and_shapes(&rays);
        gpu_accel.dispatch();

        let result = gpu_accel.download_intersections();
        let intersections = gpu_accel.get_hits_for_ray(&result, 0);

        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].t, 1.0);

        let p = Plane::new();
        let r = Ray::from_vec4(Vec4::point(0.0, -1.0, 0.0), Vec4::vector(0.0, 1.0, 0.0));
        let mut gpu_accel = GPUAccel::new("src/gpu/shader.wgsl");
        let gpuray = GpuRay::from_ray(&r);
        let rays = vec![gpuray];
        let mut w = World::default();
        w.shapes.clear();
        w.add_shape(Arc::new(p));
        gpu_accel.populate_shapes(&w.shapes);
        gpu_accel.upload_rays_and_shapes(&rays);
        gpu_accel.dispatch();

        let result = gpu_accel.download_intersections();
        let intersections = gpu_accel.get_hits_for_ray(&result, 0);

        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].t, 1.0);
    }
}
