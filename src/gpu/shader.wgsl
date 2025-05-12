const MAX_INTERSECTIONS_PER_RAY: u32 = 8u;

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
    data: array<f32, 24>
};

struct GpuIntersection {
    shape_id: u32,
    t: f32,
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

    let base = index * 2u;

    var hits_written = 0u;

    for (var shape_idx = 0u; shape_idx < arrayLength(&shapes); shape_idx += 1) {
        let shape = shapes[shape_idx];
        let inverse_mat = mat4x4_from_inverse(shape);
        let origin_local = inverse_mat * vec4<f32>(ray.origin.xyz, 1.0);
        let direction_local = inverse_mat * vec4<f32>(ray.direction.xyz, 0.0);        

        if shape.kind == 1u {
            let sphere_to_ray = origin_local.xyz - vec3<f32>(0.0, 0.0, 0.0);

            let a: f32 = dot(direction_local.xyz, direction_local.xyz);
            let b: f32 = 2.0 * dot(direction_local.xyz, sphere_to_ray);
            let c: f32 = dot(sphere_to_ray, sphere_to_ray) - 1.0;

            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 {
                continue;
            }

            let t1: f32 = (-b - sqrt(discriminant)) / (2.0 * a);
            let t2: f32 = (-b + sqrt(discriminant)) / (2.0 * a);

            if (t1 > 0.0 && hits_written < 2u) {
                intersections[base + hits_written] = GpuIntersection(shape.id, t1);
                hits_written = hits_written + 1u;
            }

            if (t2 > 0.0 && hits_written < 2u) {
                intersections[base + hits_written] = GpuIntersection(shape.id, t2);
                hits_written = hits_written + 1u;
            }
        }
    }
}