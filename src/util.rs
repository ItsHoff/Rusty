use cgmath::Vector3;

use crate::Float;

fn arr_to_vec3(arr: [f32; 3]) -> Vector3<Float> {
    Vector3::new(arr[0] as Float, arr[1] as Float, arr[2] as Float)
}
