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
    // vec2 pos_xy;
    // The range value (size of the voxel) is the pos_zw.y value this is done to save space
    // vec2 pos_zw;
    vec4 pos;
    uint material_index;
    uint _0_0_index;
    uint _0_1_index;
    uint _0_2_index;
    uint _0_3_index;
    uint _1_0_index;
    uint _1_1_index;
    uint _1_2_index;
    uint _1_3_index;
};

struct VoxelMaterial {
    vec4 color;
    vec4 specular_color;
    vec3 emissive_color;
    float emissive_strength;
    float smoothness;
    float specular_probability;
};

layout(std430, set = 0, binding = 0) readonly buffer Data {
    VoxelData data[];
} voxel_data;

layout(std430, set = 0, binding = 1) readonly buffer Materials {
    VoxelMaterial data[];
} voxel_materials;

layout(std430, set = 0, binding = 2)  readonly buffer CameraData {
    uint field_of_view;
    float render_distance;
    float aspectRatio;
    float fov_tan;
    mat4 camera_to_world;
    vec4 clear_color;
    uint random_number;
    float image_weight;
    uint frames_since_last_movement;
    vec3 position;
} camera;

layout(set = 1, binding = 0, rgba8) uniform image2D img_out;

struct IntersectionInfo {
    vec3 point;
    vec3 normal;
};

struct Hit {
    bool hit;
    VoxelMaterial material;
    vec3 point;
    vec3 normal;
    float dist;
};

struct Ray {
    vec3 origin;
    vec3 direction;
};

// ==================== Const variables ====================
const uint UINT_MAX = -1;
const float INFINITY_F = 1.0/0.0;
const uint AMOUNT_OF_RAY_BOUNCES = 3;

const float ATMOSPHERE_HEIGHT = 100*1000;
const float HORIZON_DISTANCE = 1000*1000;
const float FOG_FUNC_CONST = HORIZON_DISTANCE/2.0;
const float PI = 3.14159265359;
const float HALF_PI = 0.5 * PI;

const vec3 sun_dir = normalize(vec3(-2.0, -1.0, 0.0));

const vec4 fog_color = vec4(0.0, 0.35, 0.7, 1.0);

const VoxelMaterial empty_material = VoxelMaterial(vec4(0.0, 0.0, 0.0, 1.0), vec4(0.0, 0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0), 0.0, 0.0, 0.0);

// ==================== Other variables ====================
uint rand_num = camera.random_number;

// ==================== Helper functions ====================

float max_component(vec3 vec) {
    return max(max(vec.x, vec.y), vec.z);
}

float min_component(vec3 vec) {
    return min(min(vec.x, vec.y), vec.z);
}

// Mainly from https://stackoverflow.com/a/167764
float get_random_number() {
    rand_num = (rand_num ^ 61) ^ (rand_num >> 16);
    rand_num = rand_num + (rand_num << 3);
    rand_num = rand_num ^ (rand_num >> 4);
    rand_num = rand_num * 0x27d4eb2d;
    rand_num = rand_num ^ (rand_num >> 15);
    return fract(float(rand_num)/float(UINT_MAX));
}

uint[8] get_children_indices(VoxelData voxel) {
    return uint[8](voxel._0_0_index, voxel._0_1_index, voxel._0_2_index, voxel._0_3_index, voxel._1_0_index, voxel._1_1_index, voxel._1_2_index, voxel._1_3_index);
}

bool is_leaf_node(VoxelData voxel) {
    return voxel._0_0_index == uint(-1) && voxel._0_1_index == uint(-1) &&
           voxel._0_2_index == uint(-1) && voxel._0_3_index == uint(-1) &&
           voxel._1_0_index == uint(-1) && voxel._1_1_index == uint(-1) &&
           voxel._1_2_index == uint(-1) && voxel._1_3_index == uint(-1);
}

int get_pixel_id() {
    ivec2 IDxy = ivec2(gl_GlobalInvocationID.xy);
    ivec2 image_size = imageSize(img_out);
    return IDxy.x+((IDxy.y-1)*image_size.x);
}

// ==================== Ray functions ====================

vec3 get_random_direction() {
    return normalize(vec3((get_random_number()-0.5)*2, (get_random_number()-0.5)*2, (get_random_number()-0.5)*2));
}

float get_distance(Ray ray, IntersectionInfo hit_info) {
    return length(hit_info.point - ray.origin);
}

bool is_closer(Ray ray, IntersectionInfo hit_info, float closest) {
    return get_distance(ray, hit_info) < closest;
}

IntersectionInfo slabs(VoxelData voxel, Ray ray, vec3 invertedRayDirection) {
    const vec3 boxMin = voxel.pos.xyz;
    const vec3 boxMax = voxel.pos.xyz + voxel.pos.w;

    if (boxMin.x < ray.origin.x && ray.origin.x < boxMax.x && boxMin.y < ray.origin.y && ray.origin.y < boxMax.y && boxMin.z < ray.origin.z && ray.origin.z < boxMax.z) {
        // The ray originates inside the box
        IntersectionInfo info;
        info.point = ray.origin;
        return info;
    }

    const vec3 tMin = (boxMin - ray.origin) * invertedRayDirection;
    const vec3 tMax = (boxMax - ray.origin) * invertedRayDirection;
    const vec3 t1 = min(tMin, tMax);
    const vec3 t2 = max(tMin, tMax);
    const float tNear = max(max(t1.x, t1.y), t1.z);
    const float tFar = min(min(t2.x, t2.y), t2.z);

    IntersectionInfo info;

    if (tNear > tFar || tFar < 0.0) {
        // No intersection
        info.point = vec3(INFINITY_F);
        return info;
    }

    info.point = ray.origin + ray.direction * tNear;

    vec3 normal = vec3(-sign(ray.direction.x), 0.0, 0.0) * float(tNear == t1.x) 
                    + vec3(0.0, -sign(ray.direction.y), 0.0) * float(tNear == t1.y) 
                    + vec3(0.0, 0.0, -sign(ray.direction.z)) * float(tNear != t1.x && tNear != t1.y);

    info.normal = normal;
    return info;
}

Hit fill_hit_color(VoxelData voxel, IntersectionInfo intersection, Ray ray) {
    Hit data;
    data.material = voxel_materials.data[voxel.material_index];
    data.hit = true;
    data.point = intersection.point;
    data.normal = intersection.normal;
    return data;
}

Hit voxel_hit(Ray ray) {
    Hit ret_val;
    ret_val.hit = false;
    ret_val.material = empty_material;
    float closest = 9999999999999999999999999.0;
    const vec3 invRaydir = 1.0/ray.direction;
    
    VoxelData temp_voxel = voxel_data.data[voxel_data.data.length()-1];
    IntersectionInfo intersection_info = slabs(temp_voxel, ray, invRaydir);

    if (intersection_info.point == vec3(INFINITY_F)) return ret_val;

    if (is_leaf_node(temp_voxel)) return fill_hit_color(temp_voxel, intersection_info, ray);
    const uint[8] level_0 = get_children_indices(temp_voxel);

    // GLSL does not allow for recursive functions, thus it needs to be hard-coded
    #pragma unroll
    
    for (int i_0 = 0; i_0 < level_0.length(); i_0++) {
        if (level_0[i_0] == UINT_MAX) continue;
        temp_voxel = voxel_data.data[level_0[i_0]];
        intersection_info = slabs(temp_voxel, ray, invRaydir);
        if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
        if (is_leaf_node(temp_voxel)) {
            ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
            closest = get_distance(ray, intersection_info);
            continue;
        }
        const uint[8] level_1 = get_children_indices(temp_voxel);
        for (int i_1 = 0; i_1 < level_1.length(); i_1++) {
            if (level_1[i_1] == UINT_MAX) continue;
            temp_voxel = voxel_data.data[level_1[i_1]];
            intersection_info = slabs(temp_voxel, ray, invRaydir);
            if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
            if (is_leaf_node(temp_voxel)) {
                ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                closest = get_distance(ray, intersection_info);
                continue;
            }
            const uint[8] level_2 = get_children_indices(temp_voxel);
            for (int i_2 = 0; i_2 < level_2.length(); i_2++) {
                if (level_2[i_2] == UINT_MAX) continue;
                temp_voxel = voxel_data.data[level_2[i_2]];
                intersection_info = slabs(temp_voxel, ray, invRaydir);
                if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                if (is_leaf_node(temp_voxel)) {
                    ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                    closest = get_distance(ray, intersection_info);
                    continue;
                }
                const uint[8] level_3 = get_children_indices(temp_voxel);
                for (int i_3 = 0; i_3 < level_3.length(); i_3++) {
                    if (level_3[i_3] == UINT_MAX) continue;
                    temp_voxel = voxel_data.data[level_3[i_3]];
                    intersection_info = slabs(temp_voxel, ray, invRaydir);
                    if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                    if (is_leaf_node(temp_voxel)) {
                        ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                        closest = get_distance(ray, intersection_info);
                        continue;
                    }
                    const uint[8] level_4 = get_children_indices(temp_voxel);
                    for (int i_4 = 0; i_4 < level_4.length(); i_4++) {
                        if (level_4[i_4] == UINT_MAX) continue;
                        temp_voxel = voxel_data.data[level_4[i_4]];
                        intersection_info = slabs(temp_voxel, ray, invRaydir);
                        if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                        if (is_leaf_node(temp_voxel)) {
                            ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                            closest = get_distance(ray, intersection_info);
                            continue;
                        }
                        const uint[8] level_5 = get_children_indices(temp_voxel);
                        for (int i_5 = 0; i_5 < level_5.length(); i_5++) {
                            if (level_5[i_5] == UINT_MAX) continue;
                            temp_voxel = voxel_data.data[level_5[i_5]];
                            intersection_info = slabs(temp_voxel, ray, invRaydir);
                            if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                            if (is_leaf_node(temp_voxel)) {
                                ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                closest = get_distance(ray, intersection_info);
                                continue;
                            }
                            const uint[8] level_6 = get_children_indices(temp_voxel);
                            for (int i_6 = 0; i_6 < level_6.length(); i_6++) {
                                if (level_6[i_6] == UINT_MAX) continue;
                                temp_voxel = voxel_data.data[level_6[i_6]];
                                intersection_info = slabs(temp_voxel, ray, invRaydir);
                                if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                if (is_leaf_node(temp_voxel)) {
                                    ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                    closest = get_distance(ray, intersection_info);
                                    continue;
                                }
                                const uint[8] level_7 = get_children_indices(temp_voxel);
                                for (int i_7 = 0; i_7 < level_7.length(); i_7++) {
                                    if (level_7[i_7] == UINT_MAX) continue;
                                    temp_voxel = voxel_data.data[level_7[i_7]];
                                    intersection_info = slabs(temp_voxel, ray, invRaydir);
                                    if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                    if (is_leaf_node(temp_voxel)) {
                                        ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                        closest = get_distance(ray, intersection_info);
                                        continue;
                                    }
                                    const uint[8] level_8 = get_children_indices(temp_voxel);
                                    for (int i_8 = 0; i_8 < level_8.length(); i_8++) {
                                        if (level_8[i_8] == UINT_MAX) continue;
                                        temp_voxel = voxel_data.data[level_8[i_8]];
                                        intersection_info = slabs(temp_voxel, ray, invRaydir);
                                        if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                        if (is_leaf_node(temp_voxel)) {
                                            ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                            closest = get_distance(ray, intersection_info);
                                            continue;
                                        }
                                        const uint[8] level_9 = get_children_indices(temp_voxel);
                                        for (int i_9 = 0; i_9 < level_9.length(); i_9++) {
                                            if (level_9[i_9] == UINT_MAX) continue;
                                            temp_voxel = voxel_data.data[level_9[i_9]];
                                            intersection_info = slabs(temp_voxel, ray, invRaydir);
                                            if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                            if (is_leaf_node(temp_voxel)) {
                                                ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                                closest = get_distance(ray, intersection_info);
                                                continue;
                                            }
                                            const uint[8] level_10 = get_children_indices(temp_voxel);
                                            for (int i_10 = 0; i_10 < level_10.length(); i_10++) {
                                                if (level_10[i_10] == UINT_MAX) continue;
                                                temp_voxel = voxel_data.data[level_10[i_10]];
                                                intersection_info = slabs(temp_voxel, ray, invRaydir);
                                                if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                                if (is_leaf_node(temp_voxel)) {
                                                    ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                                    closest = get_distance(ray, intersection_info);
                                                    continue;
                                                }
                                                const uint[8] level_11 = get_children_indices(temp_voxel);
                                                for (int i_11 = 0; i_11 < level_11.length(); i_11++) {
                                                    if (level_11[i_11] == UINT_MAX) continue;
                                                    temp_voxel = voxel_data.data[level_11[i_11]];
                                                    intersection_info = slabs(temp_voxel, ray, invRaydir);
                                                    if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                                    if (is_leaf_node(temp_voxel)) {
                                                        ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                                        closest = get_distance(ray, intersection_info);
                                                        continue;
                                                    }
                                                    const uint[8] level_12 = get_children_indices(temp_voxel);
                                                    for (int i_12 = 0; i_12 < level_12.length(); i_12++) {
                                                        if (level_12[i_12] == UINT_MAX) continue;
                                                        temp_voxel = voxel_data.data[level_12[i_12]];
                                                        intersection_info = slabs(temp_voxel, ray, invRaydir);
                                                        if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                                        if (is_leaf_node(temp_voxel)) {
                                                            ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                                            closest = get_distance(ray, intersection_info);
                                                            continue;
                                                        }
                                                        const uint[8] level_13 = get_children_indices(temp_voxel);
                                                        for (int i_13 = 0; i_13 < level_13.length(); i_13++) {
                                                            if (level_13[i_13] == UINT_MAX) continue;
                                                            temp_voxel = voxel_data.data[level_13[i_13]];
                                                            intersection_info = slabs(temp_voxel, ray, invRaydir);
                                                            if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                                            if (is_leaf_node(temp_voxel)) {
                                                                ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                                                closest = get_distance(ray, intersection_info);
                                                                continue;
                                                            }
                                                            const uint[8] level_14 = get_children_indices(temp_voxel);
                                                            for (int i_14 = 0; i_14 < level_14.length(); i_14++) {
                                                                if (level_14[i_14] == UINT_MAX) continue;
                                                                temp_voxel = voxel_data.data[level_14[i_14]];
                                                                intersection_info = slabs(temp_voxel, ray, invRaydir);
                                                                if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                                                if (is_leaf_node(temp_voxel)) {
                                                                    ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                                                    closest = get_distance(ray, intersection_info);
                                                                    continue;
                                                                }
                                                                const uint[8] level_15 = get_children_indices(temp_voxel);
                                                                for (int i_15 = 0; i_15 < level_15.length(); i_15++) {
                                                                    if (level_15[i_15] == UINT_MAX) continue;
                                                                    temp_voxel = voxel_data.data[level_15[i_15]];
                                                                    intersection_info = slabs(temp_voxel, ray, invRaydir);
                                                                    if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                                                    if (is_leaf_node(temp_voxel)) {
                                                                        ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                                                        closest = get_distance(ray, intersection_info);
                                                                        continue;
                                                                    }
                                                                    const uint[8] level_16 = get_children_indices(temp_voxel);
                                                                    for (int i_16 = 0; i_16 < level_16.length(); i_16++) {
                                                                        if (level_16[i_16] == UINT_MAX) continue;
                                                                        temp_voxel = voxel_data.data[level_16[i_16]];
                                                                        intersection_info = slabs(temp_voxel, ray, invRaydir);
                                                                        if (intersection_info.point == vec3(INFINITY_F) || !is_closer(ray, intersection_info, closest)) continue;
                                                                        if (is_leaf_node(temp_voxel)) {
                                                                            ret_val = fill_hit_color(temp_voxel, intersection_info, ray);
                                                                            closest = get_distance(ray, intersection_info);
                                                                            continue;
                                                                        }
                                                                        const uint[8] level_17 = get_children_indices(temp_voxel);
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

    ret_val.dist = closest;
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
    vec4 color_in_the_end = vec4(0.0);
    rand_num += get_pixel_id();

    vec3 light = vec3(0.0);
    vec4 color = vec4(1.0);
       
    Ray ray = get_primary_ray(get_random_number(), get_random_number());
    Hit hit = voxel_hit(ray);
    bool is_specular_bounce = hit.material.specular_probability >= get_random_number();
    light += hit.material.emissive_color * hit.material.emissive_strength * color.rgb;
    color *= mix(hit.material.color, hit.material.specular_color, uint(is_specular_bounce));
    // For some reason the for-loop stopped working on my laptop, but worked with hard coded if loops, so I will have to hard code them.       
    

        if (hit.hit && hit.material.emissive_strength <= 0.0) {
            vec3 diffuse_dir = normalize(hit.normal + get_random_direction());
            vec3 spec_dir = reflect(ray.direction, hit.normal);
            is_specular_bounce = hit.material.specular_probability >= get_random_number();
            vec3 new_dir = mix(diffuse_dir, spec_dir, hit.material.smoothness * float(uint(is_specular_bounce)));
            if (new_dir == vec3(0.0, 0.0, 0.0)) {
                diffuse_dir = normalize(hit.normal + get_random_direction() + 1);
                new_dir = mix(diffuse_dir, spec_dir, hit.material.smoothness);
            }
            ray = Ray(hit.point + hit.normal, new_dir);
            hit = voxel_hit(ray);
            light += hit.material.emissive_color * hit.material.emissive_strength * color.rgb;
            color *= mix(hit.material.color, hit.material.specular_color, uint(is_specular_bounce));
        
            float r = max(color.x, max(color.y, color.z));
            if (get_random_number() >= r) {
                hit.hit = false;
            } 
            else {
                color *= 1.0/r;
            }
        }
        
        if (hit.hit && hit.material.emissive_strength <= 0.0) {
            vec3 diffuse_dir = normalize(hit.normal + get_random_direction());
            vec3 spec_dir = reflect(ray.direction, hit.normal);
            is_specular_bounce = hit.material.specular_probability >= get_random_number();
            vec3 new_dir = mix(diffuse_dir, spec_dir, hit.material.smoothness * float(uint(is_specular_bounce)));
            if (new_dir == vec3(0.0, 0.0, 0.0)) {
                diffuse_dir = normalize(hit.normal + get_random_direction() + 1);
                new_dir = mix(diffuse_dir, spec_dir, hit.material.smoothness);
            }
            ray = Ray(hit.point + hit.normal, new_dir);
            hit = voxel_hit(ray);
            light += hit.material.emissive_color * hit.material.emissive_strength * color.rgb;
            color *= mix(hit.material.color, hit.material.specular_color, uint(is_specular_bounce));
        
            float r = max(color.x, max(color.y, color.z));
            if (get_random_number() >= r) {
                hit.hit = false;
            } 
            else {
                color *= 1.0/r;
            }
        }
        
        if (hit.hit && hit.material.emissive_strength <= 0.0) {
            vec3 diffuse_dir = normalize(hit.normal + get_random_direction());
            vec3 spec_dir = reflect(ray.direction, hit.normal);
            is_specular_bounce = hit.material.specular_probability >= get_random_number();
            vec3 new_dir = mix(diffuse_dir, spec_dir, hit.material.smoothness * float(uint(is_specular_bounce)));
            if (new_dir == vec3(0.0, 0.0, 0.0)) {
                diffuse_dir = normalize(hit.normal + get_random_direction() + 1);
                new_dir = mix(diffuse_dir, spec_dir, hit.material.smoothness);
            }
            ray = Ray(hit.point + hit.normal, new_dir);
            hit = voxel_hit(ray);
            light += hit.material.emissive_color * hit.material.emissive_strength * color.rgb;
            color *= mix(hit.material.color, hit.material.specular_color, uint(is_specular_bounce));
        
            float r = max(color.x, max(color.y, color.z));
            if (get_random_number() >= r) {
                hit.hit = false;
            } 
            else {
                color *= 1.0/r;
            }
        }
        


    color_in_the_end += vec4(light, 1.0);

    vec4 old_pixel = imageLoad(img_out, IDxy);
    old_pixel = vec4(old_pixel.b, old_pixel.g, old_pixel.r, old_pixel.a);
    color_in_the_end = old_pixel * (1.0 - camera.image_weight) + color_in_the_end * camera.image_weight;
    imageStore(img_out, IDxy, vec4(color_in_the_end.b, color_in_the_end.g, color_in_the_end.r, color_in_the_end.a));
}
            "
        }
    }
    
    cs::load(device).unwrap()
}