#version 460 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;
out vec2 TexCoord;

// x: left, y: right, z: top, w: bottom
// values between 0.0 and 1.0 representing percentage to crop
uniform vec4 crop;

void main() {
    gl_Position = vec4(aPos, 1.0);
    
    // Map the standard 0.0->1.0 UV range into our cropped window
    TexCoord = vec2(
        mix(crop.x, 1.0 - crop.y, aTexCoord.x),
        mix(crop.z, 1.0 - crop.w, aTexCoord.y)
    );
}
