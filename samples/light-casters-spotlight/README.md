# Light casters (spotlight)

Based on the spotlight section of [https://learnopengl.com/Lighting/Light-casters](https://learnopengl.com/Lighting/Light-casters).

Note that we need to use `textureSampleLevel` in the shader and sample from mip level 1.0 if we sample inside a conditional rather than clamp the intensity value.
See [https://github.com/gfx-rs/wgpu-rs/issues/912](https://github.com/gfx-rs/wgpu-rs/issues/912) and [https://themaister.net/blog/2019/09/12/the-weird-world-of-shader-divergence-and-lod/](https://themaister.net/blog/2019/09/12/the-weird-world-of-shader-divergence-and-lod/) for more details.
