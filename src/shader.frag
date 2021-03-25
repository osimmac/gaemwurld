//fragment shader
#version 450



//in and out varaibles can have layout specified, it will save at buffer location 0
//most casese location 0 is the current texture swapchain aka the screen.

layout(location=0) in vec2 v_text_coords;
layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

void main() 
{
    //f_color = vec4(v_color,1.0);
    f_color = texture(sampler2D(t_diffuse, s_diffuse),v_text_coords);
}