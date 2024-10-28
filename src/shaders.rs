
use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, shader_type: &str) -> Color {
  match shader_type {
      "cloud" => cloud_shader(fragment, uniforms),
      "lava" => lava_shader(fragment, uniforms),
      "terrain" => terrain_shader(fragment),
      "gas" => gas_shader(fragment, uniforms),
      _ => combined_shader(fragment, uniforms), // Default shader
  }
}

fn static_pattern_shader(fragment: &Fragment) -> Color {
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
  
    let pattern = ((x * 10.0).sin() * (y * 10.0).sin()).abs();
  
    let r = (pattern * 255.0) as u8;
    let g = ((1.0 - pattern) * 255.0) as u8;
    let b = 128;
  
    Color::new(r, g, b)
}

fn lava_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Base colors for the lava effect
  let bright_color = Color::new(255, 240, 0); // Bright orange (lava-like)
  let dark_color = Color::new(130, 20, 0);   // Darker red-orange  

  // Get fragment position
  let position = Vec3::new(
    fragment.vertex_position.x,
    fragment.vertex_position.y,
    fragment.depth
  );

  // Base frequency and amplitude for the pulsating effect
  let base_frequency = 0.2;
  let pulsate_amplitude = 0.5;
  let t = uniforms.time as f32 * 0.01;

  // Pulsate on the z-axis to change spot size
  let pulsate = (t * base_frequency).sin() * pulsate_amplitude;

  // Apply noise to coordinates with subtle pulsating on z-axis
  let zoom = 1000.0; // Constant zoom factor
  let noise_value1 = uniforms.noise.get_noise_3d(
    position.x * zoom,
    position.y * zoom,
    (position.z + pulsate) * zoom
  );
  let noise_value2 = uniforms.noise.get_noise_3d(
    (position.x + 1000.0) * zoom,
    (position.y + 1000.0) * zoom,
    (position.z + 1000.0 + pulsate) * zoom
  );
  let noise_value = (noise_value1 + noise_value2) * 0.5;  // Averaging noise for smoother transitions

  // Use lerp for color blending based on noise value
  let color = dark_color.lerp(&bright_color, noise_value);

  color * fragment.intensity
}


fn terrain_shader(fragment: &Fragment) -> Color {
  // Simulación de ruido básico
  let noise = ((fragment.vertex_position.x * 5.0).sin() + (fragment.vertex_position.y * 5.0).cos()).abs();
  let color_value = (noise * 255.0) as u8;
  Color::new(color_value, color_value / 2, color_value / 4) // Tonos terrosos
}

fn gas_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let ripple_pattern = (fragment.vertex_position.x * 8.0 + uniforms.time as f32 * 0.1).sin().abs();
  let intensity = (ripple_pattern * 255.0) as u8;
  Color::new(0, intensity, 255) * fragment.intensity // Azul agua
}

fn cloud_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 100.0;  // to move our values 
  let ox = 100.0; // offset x in the noise map
  let oy = 100.0;
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;
  let t = uniforms.time as f32 * 0.5;

  let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox + t, y * zoom + oy);

  // Define cloud threshold and colors
  let cloud_threshold = 0.5; // Adjust this value to change cloud density
  let land_threshold = 0.001;
  let cloud_color = Color::new(255, 255, 255); // White for clouds
  let sky_color = Color::new(30, 97, 145); // Sky blue
  let land_color = Color::new(0, 100, 0);

  // Determine if the pixel is part of a cloud or sky
  let noise_color = if noise_value > cloud_threshold {
    cloud_color
} else if noise_value > land_threshold {
    land_color
} else {
    sky_color
};

  noise_color * fragment.intensity
}

fn moving_circles_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
  
    let time = uniforms.time as f32 * 0.05;
    let circle1_x = (time.sin() * 0.4 + 0.5) % 1.0;
    let circle2_x = (time.cos() * 0.4 + 0.5) % 1.0;
  
    let dist1 = ((x - circle1_x).powi(2) + (y - 0.3).powi(2)).sqrt();
    let dist2 = ((x - circle2_x).powi(2) + (y - 0.7).powi(2)).sqrt();
  
    let circle_size = 0.1;
    let circle1 = if dist1 < circle_size { 1.0f32 } else { 0.0f32 };
    let circle2 = if dist2 < circle_size { 1.0f32 } else { 0.0f32 };
  
    let circle_intensity = (circle1 + circle2).min(1.0f32);
  
    Color::new(
      (circle_intensity * 255.0) as u8,
      (circle_intensity * 255.0) as u8,
      (circle_intensity * 255.0) as u8
    )
}

pub fn combined_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let base_color = static_pattern_shader(fragment);
    let circle_color = moving_circles_shader(fragment, uniforms);
  
    // Combine shaders: use circle color if it's not black, otherwise use base color
    if !circle_color.is_black() {
      circle_color * fragment.intensity
    } else {
      base_color * fragment.intensity
    }
}