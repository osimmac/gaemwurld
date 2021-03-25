// shader.vert
//vertex shader
//data from this shader is passed into the fragment shader.
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec2 v_text_coords;


layout(set=1, binding=0)
uniform Uniforms
{
    mat4 u_view_proj;
};

void main() 
{
    v_text_coords = a_tex_coords;
    gl_Position = u_view_proj * vec4(a_position,1.0);
}
 