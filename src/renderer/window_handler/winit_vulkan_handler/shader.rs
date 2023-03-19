use std::sync::Arc;

use vulkano::{device::Device, shader::ShaderModule};

pub fn render_shader(device: Arc<Device>) -> Arc<ShaderModule> {
    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            src: "  
            #version 450
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

struct VoxelData
{
    vec2 pos_xy;
    // The range value (size of the voxel) is the pos_zw.y value this is done to save space
    vec2 pos_zw;
    vec2 color_rg;
    vec2 color_ba;
    uint _0_0_index;
    uint _0_1_index;
    uint _0_2_index;
    uint _0_3_index;
    uint _1_0_index;
    uint _1_1_index;
    uint _1_2_index;
    uint _1_3_index;
};

layout(set = 0, binding = 0) readonly buffer Data {
    VoxelData data[];
} voxel_data;

layout(set = 0, binding = 1)  readonly buffer CameraData {
    uint field_of_view;
    float render_distance;
    float aspectRatio;
    float fov_tan;
    mat4 camera_to_world;
    vec4 clear_color;
} camera;

layout(set = 1, binding = 0, rgba8) uniform image2D img_out; 

struct Hit {
    bool hit;
    vec4 color;
    vec3 point;
    vec3 normal;
};

struct Ray {
    vec3 origin;
    vec3 direction;
};

// ==================== Const variables ====================
const uint UINT_MAX = -1;
const float INFINITY_F = 1.0/0.0;
const uint AMOUNT_OF_PRIMARY_RAYS = 1;
const uint AMOUNT_OF_RAY_BOUNCES = 1;

// ==================== Helper functions ====================

float max_component(vec3 vec) {
    return max(max(vec.x, vec.y), vec.z);
}

float min_component(vec3 vec) {
    return min(min(vec.x, vec.y), vec.z);
}

float get_random_number(int seed) {
    return fract(sin(float(seed)) * 43758.5453123);
}

float[AMOUNT_OF_PRIMARY_RAYS] get_random_noise(int seed) {
    float[AMOUNT_OF_PRIMARY_RAYS] noise;
    for (int i = 0; i < AMOUNT_OF_PRIMARY_RAYS; i++) {
        noise[i] = get_random_number(seed*10 + i);
    }
    return noise;
}

// ==================== Ray functions ====================


vec3 slabs(VoxelData voxel, Ray ray, vec3 invertedRayDirection) {
    const vec3 boxMin = vec3(voxel.pos_xy, voxel.pos_zw.x);
    const vec3 boxMax = vec3(voxel.pos_xy.x + voxel.pos_zw.y, voxel.pos_xy.y + voxel.pos_zw.y, voxel.pos_zw.x + voxel.pos_zw.y);

    vec3 tMin = (boxMin - ray.origin) * invertedRayDirection;
    vec3 tMax = (boxMax - ray.origin) * invertedRayDirection;
    vec3 t1 = min(tMin, tMax);
    vec3 t2 = max(tMin, tMax);
    float tNear = max(max(t1.x, t1.y), t1.z);
    float tFar = min(min(t2.x, t2.y), t2.z);

    if (tNear > tFar || tFar < 0.0) {
        // No intersection
        return vec3(INFINITY_F);//false;
    }

    // Calculate intersection point and surface normal
    //intersectionPoint = 
    return ray.origin + ray.direction * tNear;//tFar;
    // if (ray.direction.z < 0.0 || ray.direction.x < 0.0) {
    //     intersectionPoint = ray.origin + ray.direction * tNear;
    // } else {
    //     intersectionPoint = ray.origin + ray.direction * tFar;
    // }

    // vec3 boxCenter = (boxMin + boxMax) * 0.5;
    // vec3 boxHalfExtents = (boxMax - boxMin) * 0.5;
    // vec3 localIntersection = intersectionPoint - boxCenter;
    // vec3 absLocalIntersection = abs(localIntersection);
    // float maxAbs = max(max(absLocalIntersection.x, absLocalIntersection.y), absLocalIntersection.z);
    // if (maxAbs == absLocalIntersection.x) {
    //     normal = vec3(sign(localIntersection.x), 0.0, 0.0);
    // } else if (maxAbs == absLocalIntersection.y) {
    //     normal = vec3(0.0, sign(localIntersection.y), 0.0);
    // } else {
    //     normal = vec3(0.0, 0.0, sign(localIntersection.z));
    // }

    // return true;
}

// From https://jcgt.org/published/0007/03/04/
// bool slabs(VoxelData voxel, Ray ray, vec3 invertedRayDirection, out vec3 intersectionPoint, out vec3 normal) {
//     const vec3 p0 = vec3(voxel.pos_xy, voxel.pos_zw.x);
//     const vec3 p1 = vec3(voxel.pos_xy.x + voxel.pos_zw.y, voxel.pos_xy.y + voxel.pos_zw.y, voxel.pos_zw.x + voxel.pos_zw.y);

//     const vec3 t0 = (p0 - ray.origin) * invertedRayDirection;
//     const vec3 t1 = (p1 - ray.origin) * invertedRayDirection;
//     const vec3 tmin = min(t0,t1), tmax = max(t0,t1);
//     const float tmax_val = min_component(tmax);
//     const float tmin_val = max_component(tmin);
//     // TODO: Make the tmax_val also work with vectors in the negative space
    
//     bool intersects = tmin_val <= tmax_val && tmax_val >= 0.0;

//     if (intersects) {
//         intersectionPoint = ray.origin + ray.direction * tmin_val;
        
//         if (tmin.x > tmin.y && tmin.x > tmin.z) {
//             normal.x = t0.x < t1.x ? 1.0 : -1.0;
//         } else if (tmin.y > tmin.z) {
//             normal.y = t0.y < t1.y ? 1.0 : -1.0;
//         } else {
//             normal.z = t0.z < t1.z ? 1.0 : -1.0;
//         }
//     }
    
//     return intersects;
// }

uint[8] get_children_indices(VoxelData voxel) {
    return uint[8](voxel._0_0_index, voxel._0_1_index, voxel._0_2_index, voxel._0_3_index, voxel._1_0_index, voxel._1_1_index, voxel._1_2_index, voxel._1_3_index);
}

bool is_leaf_node(VoxelData voxel) {
    return voxel._0_0_index == uint(-1) && voxel._0_1_index == uint(-1) &&
           voxel._0_2_index == uint(-1) && voxel._0_3_index == uint(-1) &&
           voxel._1_0_index == uint(-1) && voxel._1_1_index == uint(-1) &&
           voxel._1_2_index == uint(-1) && voxel._1_3_index == uint(-1);
}

Hit fill_hit_color(VoxelData voxel, Hit voxel_hit) {
    Hit data;
    data.color = vec4(voxel.color_rg, voxel.color_ba);
    data.hit = true;
    data.point = voxel_hit.point;
    return data;
}

// float get_distance(Ray ray, VoxelData voxel) {
//     // TODO: there is something wrong here
//     //return length(vec3((voxel.pos_xy.x+voxel.pos_zw.y)/2.0, (voxel.pos_xy.y+voxel.pos_zw.y)/2.0, (voxel.pos_zw.x+voxel.pos_zw.y)/2.0) - ray.origin);
//     vec3 center = vec3((voxel.pos_xy.x + voxel.pos_zw.y) / 2.0,
//                    (voxel.pos_xy.y + voxel.pos_zw.y) / 2.0,
//                    (voxel.pos_zw.x + voxel.pos_zw.y) / 2.0);
//     return length(center - ray.origin);
// }

float get_distance(Ray ray, vec3 hit_point) {
    return length(hit_point - ray.origin);
}

// bool is_closer(in Ray ray, in VoxelData voxel, in float closest) {
//     return get_distance(ray, voxel) < closest;
// }

bool is_closer(in Ray ray, in vec3 hit_point, in float closest) {
    return get_distance(ray, hit_point) < closest;
}

Hit voxel_hit(Ray ray, vec4 clear_col) {
    Hit ret_val;
    ret_val.color = clear_col;
    ret_val.hit = false;
    float closest = 999999999999999999.0;
    const vec3 invRaydir = 1.0/ray.direction;
    
    // Look here for tip on how to find the intersection/hit point: https://tavianator.com/2011/ray_box.html
    // GLSL does not allow for recursive functions, thus it needs to be hard-coded
    VoxelData temp_voxel = voxel_data.data[voxel_data.data.length()-1];
    vec3 hit_point = slabs(temp_voxel, ray, invRaydir);
    const uint[8] level_0 = get_children_indices(temp_voxel);
    
    if (hit_point == vec3(INFINITY_F)) return ret_val;

    if (is_leaf_node(temp_voxel)) return fill_hit_color(temp_voxel, ret_val);
    
    for (int i_0 = 0; i_0 < level_0.length(); i_0++) {
        if (level_0[i_0] == UINT_MAX) continue;
        temp_voxel = voxel_data.data[level_0[i_0]];
        const uint[8] level_1 = get_children_indices(temp_voxel);
        hit_point = slabs(temp_voxel, ray, invRaydir);
        if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
        if (is_leaf_node(temp_voxel)) {
            ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
            ret_val.point = hit_point;
            ret_val.hit = true;
            closest = get_distance(ray, hit_point);
            continue;
        }
        for (int i_1 = 0; i_1 < level_1.length(); i_1++) {
            if (level_1[i_1] == UINT_MAX) continue;
            temp_voxel = voxel_data.data[level_1[i_1]];
            const uint[8] level_2 = get_children_indices(temp_voxel);
            hit_point = slabs(temp_voxel, ray, invRaydir);
            if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
            if (is_leaf_node(temp_voxel)) {
                ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                ret_val.point = hit_point;
                ret_val.hit = true;
                closest = get_distance(ray, hit_point);
                continue;
            }
            for (int i_2 = 0; i_2 < level_2.length(); i_2++) {
                if (level_2[i_2] == UINT_MAX) continue;
                temp_voxel = voxel_data.data[level_2[i_2]];
                const uint[8] level_3 = get_children_indices(temp_voxel);
                hit_point = slabs(temp_voxel, ray, invRaydir);
                if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                if (is_leaf_node(temp_voxel)) {
                    ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                    ret_val.point = hit_point;
                    ret_val.hit = true;
                    closest = get_distance(ray, hit_point);
                    continue;
                }
                for (int i_3 = 0; i_3 < level_3.length(); i_3++) {
                    if (level_3[i_3] == UINT_MAX) continue;
                    temp_voxel = voxel_data.data[level_3[i_3]];
                    const uint[8] level_4 = get_children_indices(temp_voxel);
                    hit_point = slabs(temp_voxel, ray, invRaydir);
                    if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                    if (is_leaf_node(temp_voxel)) {
                        ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                        ret_val.point = hit_point;
                        ret_val.hit = true;
                        closest = get_distance(ray, hit_point);
                        continue;
                    }
                    for (int i_4 = 0; i_4 < level_4.length(); i_4++) {
                        if (level_4[i_4] == UINT_MAX) continue;
                        temp_voxel = voxel_data.data[level_4[i_4]];
                        const uint[8] level_5 = get_children_indices(temp_voxel);
                        hit_point = slabs(temp_voxel, ray, invRaydir);
                        if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                        if (is_leaf_node(temp_voxel)) {
                            ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                            ret_val.point = hit_point;
                            ret_val.hit = true;
                            closest = get_distance(ray, hit_point);
                            continue;
                        }
                        for (int i_5 = 0; i_5 < level_5.length(); i_5++) {
                            if (level_5[i_5] == UINT_MAX) continue;
                            temp_voxel = voxel_data.data[level_5[i_5]];
                            const uint[8] level_6 = get_children_indices(temp_voxel);
                            hit_point = slabs(temp_voxel, ray, invRaydir);
                            if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                            if (is_leaf_node(temp_voxel)) {
                                ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                ret_val.point = hit_point;
                                ret_val.hit = true;
                                closest = get_distance(ray, hit_point);
                                continue;
                            }
                            for (int i_6 = 0; i_6 < level_6.length(); i_6++) {
                                if (level_6[i_6] == UINT_MAX) continue;
                                temp_voxel = voxel_data.data[level_6[i_6]];
                                const uint[8] level_7 = get_children_indices(temp_voxel);
                                hit_point = slabs(temp_voxel, ray, invRaydir);
                                if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                if (is_leaf_node(temp_voxel)) {
                                    ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                    ret_val.point = hit_point;
                                    ret_val.hit = true;
                                    closest = get_distance(ray, hit_point);
                                    continue;
                                }
                                for (int i_7 = 0; i_7 < level_7.length(); i_7++) {
                                    if (level_7[i_7] == UINT_MAX) continue;
                                    temp_voxel = voxel_data.data[level_7[i_7]];
                                    const uint[8] level_8 = get_children_indices(temp_voxel);
                                    hit_point = slabs(temp_voxel, ray, invRaydir);
                                    if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                    if (is_leaf_node(temp_voxel)) {
                                        ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                        ret_val.point = hit_point;
                                        ret_val.hit = true;
                                        closest = get_distance(ray, hit_point);
                                        continue;
                                    }
                                    for (int i_8 = 0; i_8 < level_8.length(); i_8++) {
                                        if (level_8[i_8] == UINT_MAX) continue;
                                        temp_voxel = voxel_data.data[level_8[i_8]];
                                        const uint[8] level_9 = get_children_indices(temp_voxel);
                                        hit_point = slabs(temp_voxel, ray, invRaydir);
                                        if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                        if (is_leaf_node(temp_voxel)) {
                                            ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                            ret_val.point = hit_point;
                                            ret_val.hit = true;
                                            closest = get_distance(ray, hit_point);
                                            continue;
                                        }
                                        for (int i_9 = 0; i_9 < level_9.length(); i_9++) {
                                            if (level_9[i_9] == UINT_MAX) continue;
                                            temp_voxel = voxel_data.data[level_9[i_9]];
                                            const uint[8] level_10 = get_children_indices(temp_voxel);
                                            hit_point = slabs(temp_voxel, ray, invRaydir);
                                            if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                            if (is_leaf_node(temp_voxel)) {
                                                ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                                ret_val.point = hit_point;
                                                ret_val.hit = true;
                                                closest = get_distance(ray, hit_point);
                                                continue;
                                            }
                                            for (int i_10 = 0; i_10 < level_10.length(); i_10++) {
                                                if (level_10[i_10] == UINT_MAX) continue;
                                                temp_voxel = voxel_data.data[level_10[i_10]];
                                                const uint[8] level_11 = get_children_indices(temp_voxel);
                                                hit_point = slabs(temp_voxel, ray, invRaydir);
                                                if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                                if (is_leaf_node(temp_voxel)) {
                                                    ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                                    ret_val.point = hit_point;
                                                    ret_val.hit = true;
                                                    closest = get_distance(ray, hit_point);
                                                    continue;
                                                }
                                                for (int i_11 = 0; i_11 < level_11.length(); i_11++) {
                                                    if (level_11[i_11] == UINT_MAX) continue;
                                                    temp_voxel = voxel_data.data[level_11[i_11]];
                                                    const uint[8] level_12 = get_children_indices(temp_voxel);
                                                    hit_point = slabs(temp_voxel, ray, invRaydir);
                                                    if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                                    if (is_leaf_node(temp_voxel)) {
                                                        ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                                        ret_val.point = hit_point;
                                                        ret_val.hit = true;
                                                        closest = get_distance(ray, hit_point);
                                                        continue;
                                                    }
                                                    for (int i_12 = 0; i_12 < level_12.length(); i_12++) {
                                                        if (level_12[i_12] == UINT_MAX) continue;
                                                        temp_voxel = voxel_data.data[level_12[i_12]];
                                                        const uint[8] level_13 = get_children_indices(temp_voxel);
                                                        hit_point = slabs(temp_voxel, ray, invRaydir);
                                                        if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                                        if (is_leaf_node(temp_voxel)) {
                                                            ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                                            ret_val.point = hit_point;
                                                            ret_val.hit = true;
                                                            closest = get_distance(ray, hit_point);
                                                            continue;
                                                        }
                                                        for (int i_13 = 0; i_13 < level_13.length(); i_13++) {
                                                            if (level_13[i_13] == UINT_MAX) continue;
                                                            temp_voxel = voxel_data.data[level_13[i_13]];
                                                            const uint[8] level_14 = get_children_indices(temp_voxel);
                                                            hit_point = slabs(temp_voxel, ray, invRaydir);
                                                            if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                                            if (is_leaf_node(temp_voxel)) {
                                                                ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                                                ret_val.point = hit_point;
                                                                ret_val.hit = true;
                                                                closest = get_distance(ray, hit_point);
                                                                continue;
                                                            }
                                                            for (int i_14 = 0; i_14 < level_14.length(); i_14++) {
                                                                if (level_14[i_14] == UINT_MAX) continue;
                                                                temp_voxel = voxel_data.data[level_14[i_14]];
                                                                const uint[8] level_15 = get_children_indices(temp_voxel);
                                                                hit_point = slabs(temp_voxel, ray, invRaydir);
                                                                if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                                                if (is_leaf_node(temp_voxel)) {
                                                                    ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                                                    ret_val.point = hit_point;
                                                                    ret_val.hit = true;
                                                                    closest = get_distance(ray, hit_point);
                                                                    continue;
                                                                }
                                                                for (int i_15 = 0; i_15 < level_15.length(); i_15++) {
                                                                    if (level_15[i_15] == UINT_MAX) continue;
                                                                    temp_voxel = voxel_data.data[level_15[i_15]];
                                                                    const uint[8] level_16 = get_children_indices(temp_voxel);
                                                                    hit_point = slabs(temp_voxel, ray, invRaydir);
                                                                    if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                                                    if (is_leaf_node(temp_voxel)) {
                                                                        ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                                                        ret_val.point = hit_point;
                                                                        ret_val.hit = true;
                                                                        closest = get_distance(ray, hit_point);
                                                                        continue;
                                                                    }
                                                                    for (int i_16 = 0; i_16 < level_16.length(); i_16++) {
                                                                        if (level_16[i_16] == UINT_MAX) continue;
                                                                        temp_voxel = voxel_data.data[level_16[i_16]];
                                                                        const uint[8] level_17 = get_children_indices(temp_voxel);
                                                                        hit_point = slabs(temp_voxel, ray, invRaydir);
                                                                        if (hit_point == vec3(INFINITY_F) || !is_closer(ray, hit_point, closest)) continue;
                                                                        if (is_leaf_node(temp_voxel)) {
                                                                            ret_val.color = vec4(temp_voxel.color_rg, temp_voxel.color_ba);
                                                                            ret_val.point = hit_point;
                                                                            ret_val.hit = true;
                                                                            closest = get_distance(ray, hit_point);
                                                                            continue;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    return ret_val;
}

Ray get_primary_ray(float x_offset, float y_offset) {
    ivec2 IDxy = ivec2(gl_GlobalInvocationID.xy);
    const ivec2 screenSize = imageSize(img_out);

    const vec2 pixel_NCD = vec2((float(IDxy.x)+x_offset+0.5)/float(screenSize.x), (float(IDxy.y)+y_offset+0.5)/float(screenSize.y));
    const vec2 camera_pixel = vec2((2 * pixel_NCD.x - 1) * camera.aspectRatio * camera.fov_tan, (1 - 2 * pixel_NCD.y) * camera.fov_tan);

    const highp vec4 world_search_pos = vec4(vec3(camera_pixel.x, camera_pixel.y, -1.0), 0.0)*camera.camera_to_world;
    highp vec3 current_search_pos = normalize(world_search_pos.xyz);
    current_search_pos.x = -current_search_pos.x;

    return Ray(camera.camera_to_world[3].xyz, normalize(current_search_pos));
}

// Main         
void main() {
    ivec2 IDxy = ivec2(gl_GlobalInvocationID.xy);
    vec4 color_in_the_end = vec4(0.0);//camera.clear_color;

    float x_offset[AMOUNT_OF_PRIMARY_RAYS] = get_random_noise(IDxy.x);
    float y_offset[AMOUNT_OF_PRIMARY_RAYS] = get_random_noise(IDxy.y);

    for (int i = 0; i < AMOUNT_OF_PRIMARY_RAYS; i++) {
        Ray ray = get_primary_ray(x_offset[i], y_offset[i]);
        Hit hit = voxel_hit(ray, camera.clear_color);
        vec4 color = hit.color;
        // for (int j = 0; j < AMOUNT_OF_RAY_BOUNCES; j++) {
        //     if (hit.hit) {
        //         // ray = Ray(hit.point + hit.normal, reflect(ray.direction, hit.normal));
        //         // hit = voxel_hit(ray, camera.clear_color);
        //         // color += hit.color/float(AMOUNT_OF_RAY_BOUNCES);
        //         color = vec4(normalize(hit.point), 1.0);
        //         break;
        //     }
        // }
        
        // if (hit.hit) {
        //     color = vec4(normalize(hit.point), 1.0);
        //     // break;
        // }
        color_in_the_end += color * (1.0 / float(AMOUNT_OF_PRIMARY_RAYS));
    }

    imageStore(img_out, IDxy, vec4(color_in_the_end.b, color_in_the_end.g, color_in_the_end.r, color_in_the_end.a));
}
            "
        }
    }
    
    cs::load(device).unwrap()
}