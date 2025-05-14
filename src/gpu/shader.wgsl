const MAX_INTERSECTIONS_PER_RAY: u32 = 8u;
const EPSILON: f32 = 0.0001;

struct GpuRay {
    origin: vec4<f32>,
    direction: vec4<f32>
};

struct GpuShape {
    id: u32,
    kind: u32,
    _padding0: u32,
    _padding1: u32,
    inverse: array<f32, 16>,
    data: array<f32, 40>
};

struct GpuIntersection {
    shape_id: u32,
    t: f32,
    u: f32,
    v: f32
};

fn mat4x4_from_inverse(shape: GpuShape) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(shape.inverse[ 0], shape.inverse[ 1], shape.inverse[ 2], shape.inverse[ 3]),
        vec4<f32>(shape.inverse[ 4], shape.inverse[ 5], shape.inverse[ 6], shape.inverse[ 7]),
        vec4<f32>(shape.inverse[ 8], shape.inverse[ 9], shape.inverse[10], shape.inverse[11]),
        vec4<f32>(shape.inverse[12], shape.inverse[13], shape.inverse[14], shape.inverse[15]),
    );
}

@group(0) @binding(0)
var<storage, read> rays: array<GpuRay>;

@group(0) @binding(1)
var<storage, read> shapes: array<GpuShape>;

@group(0) @binding(2)
var<storage, read_write> intersections: array<GpuIntersection>;

// Ideal workgroup size depends on the hardware, the workload, and other factors. However, it should
// _generally_ be a multiple of 64. Common sizes are 64x1x1, 256x1x1; or 8x8x1, 16x16x1 for 2D workloads.
@compute @workgroup_size(64)
fn intersect(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // While compute invocations are 3d, we're only using one dimension.
    let index = global_id.x;

    if (index >= arrayLength(&rays)) {
        return;
    }

    let ray = rays[index];    

    let base = index * MAX_INTERSECTIONS_PER_RAY;

    var hits_written = 0u;

    for (var shape_idx = 0u; shape_idx < arrayLength(&shapes); shape_idx += 1) {
        let shape = shapes[shape_idx];
        let inverse_mat = mat4x4_from_inverse(shape);
        let origin_local = inverse_mat * vec4<f32>(ray.origin.xyz, 1.0);
        let direction_local = inverse_mat * vec4<f32>(ray.direction.xyz, 0.0);

        if shape.kind == 1u {
            intersect_sphere(origin_local.xyz, direction_local.xyz, shape.id, base, &hits_written);
        } else if shape.kind == 2u {
            intersect_triangle(origin_local.xyz, direction_local.xyz, shape.id, base, &hits_written, shape.data);
        } else if shape.kind == 3u {
            intersect_plane(origin_local.xyz, direction_local.xyz, shape.id, base, &hits_written);
        }
    }
}

fn intersect_plane(
    ray_origin: vec3<f32>,
    ray_direction: vec3<f32>,
    shape_id: u32,
    base: u32,
    hits_written: ptr<function, u32>
) {
    if (abs(ray_direction.y) < EPSILON) {
        return;
    } else {
        let t: f32 = -ray_origin.y / ray_direction.y;
        intersections[base + *hits_written] = GpuIntersection(shape_id, t, -1.0, -1.0);
        *hits_written = *hits_written + 1u;    
    }
}

fn intersect_triangle(
    ray_origin: vec3<f32>,
    ray_direction: vec3<f32>,
    shape_id: u32,
    base: u32,
    hits_written: ptr<function, u32>,
    data: array<f32, 40>

) {
    let p1 = vec3<f32>(data[0], data[1], data[2]);
    let p2 = vec3<f32>(data[4], data[5], data[6]);
    let p3 = vec3<f32>(data[8], data[9], data[10]);
    
    let n1 = vec3<f32>(data[12], data[13], data[14]);
    let n2 = vec3<f32>(data[16], data[17], data[18]);
    let n3 = vec3<f32>(data[20], data[21], data[22]);
    
    let e1 = vec3<f32>(data[24], data[25], data[26]);
    let e2 = vec3<f32>(data[28], data[29], data[30]);
    
    let normal = vec3<f32>(data[32], data[33], data[34]);

    let dir_cross_e2: vec3<f32> = cross(ray_direction, e2);
    let det: f32 = dot(e1, dir_cross_e2);

    if (abs(det) < EPSILON) {
        return;
    }

    let f: f32 = 1.0 / det;
    let p1_to_origin: vec3<f32> = ray_origin - p1;
    let u: f32 = f * dot(p1_to_origin, dir_cross_e2);
    if (u < 0.0 || u > 1.0) {
        return;
    }

    let origin_cross_e1: vec3<f32> = cross(p1_to_origin, e1);
    let v: f32 = f * dot(ray_direction, origin_cross_e1);
    if (v < 0.0 || (u + v) > 1.0) {
        return;
    }

    let t: f32 = f * dot(e2, origin_cross_e1);

    intersections[base + *hits_written] = GpuIntersection(shape_id, t, u, v);
    *hits_written = *hits_written + 1u;
}

fn intersect_sphere(
    ray_origin: vec3<f32>,
    ray_direction: vec3<f32>,
    shape_id: u32,
    base: u32,
    hits_written: ptr<function, u32>
) {
    let sphere_to_ray = ray_origin - vec3<f32>(0.0, 0.0, 0.0);

    let a: f32 = dot(ray_direction, ray_direction);
    let b: f32 = 2.0 * dot(ray_direction, sphere_to_ray);
    let c: f32 = dot(sphere_to_ray, sphere_to_ray) - 1.0;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return;
    }

    let t1: f32 = (-b - sqrt(discriminant)) / (2.0 * a);
    let t2: f32 = (-b + sqrt(discriminant)) / (2.0 * a);

    if (t1 > 0.0 && *hits_written < MAX_INTERSECTIONS_PER_RAY) {
        intersections[base + *hits_written] = GpuIntersection(shape_id, t1, -1.0, -1.0);
        *hits_written = *hits_written + 1u;
    }

    if (t2 > 0.0 && *hits_written < MAX_INTERSECTIONS_PER_RAY) {
        intersections[base + *hits_written] = GpuIntersection(shape_id, t2, -1.0, -1.0);
        *hits_written = *hits_written + 1u;
    }
}