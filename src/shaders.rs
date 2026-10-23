pub const VERTEX_SHADER: &str = r#"
#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying mediump vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

pub const FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying mediump vec2 uv;
uniform sampler2D Texture;
uniform vec2 u_resolution;

const vec4 COLOR_1_LIGHT = vec4(0.545, 0.674, 0.058, 1.0); // #8bac0f
const vec4 COLOR_2_DARK  = vec4(0.188, 0.384, 0.188, 1.0); // #306230
const vec4 COLOR_3_SHAD  = vec4(0.058, 0.220, 0.058, 1.0); // #0f380f

void main() {
    vec2 pixelated_uv = floor(uv * u_resolution) / u_resolution;
    vec4 color = texture2D(Texture, pixelated_uv);
    
    if (color.a > 0.40) {
        gl_FragColor = COLOR_1_LIGHT;
    } else if (color.a > 0.20) {
        gl_FragColor = COLOR_2_DARK;
    } else if (color.a > 0.05) {
        gl_FragColor = COLOR_3_SHAD;
    } else {
        discard;
    }
}
"#;