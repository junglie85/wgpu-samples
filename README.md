# Wgpu Samples

Wgpu samples based on [Learn OpenGL](https://learnopengl.com/).

## Samples

I've tried to follow the order of the original tutorials as best as possible but have added or reorganised samples where I think it makes sense.
The following sections serve as a contents page of sorts.
To run a specific sample, type:

```
cargo run --bin SAMPLE_NAME
```

The corresponding name is shown next to the sample in the below list.
For example, to run the _Hello triangle_ sample, type:

```
cargo run --bin hello-triangle
```

### Getting started

- [Hello triangle](samples/hello-triangle) (`hello-triangle`)
- [Hello rectangle](samples/hello-rectangle) (`hello-rectangle`)
- [Shaders (VBO)](samples/shaders) (`shaders`)
- [Shaders (UBO)](samples/shaders-uniform) (`shaders-uniform`)
- [Textures](samples/textures) (`textures`)
- [Textures mixed](samples/textures-mixed) (`textures-mixed`)
- [Transformations](samples/transformations) (`transformations`)
- [Coordinate systems](samples/coordinate-systems) (`coordinate-systems`)
- [More cubes](samples/more-cubes) (`more-cubes`)
- [Camera](samples/camera) (`camera`)

### Lighting

- [Colors](samples/colors) (`colors`)

## Acknowledgements

It would be rude to not acknowledge, up front, the key resources that I have used to learn WGPU and graphics programming.

- [Learn OpenGL](https://learnopengl.com/).
Thank you Joey de Vries for the awesome work you've done explaining GPU graphics programming and the OpenGL API.
- [Learn Wgpu](https://sotrh.github.io/learn-wgpu/).
Thank you Benjamin Hansen for the amazing kickstart your knowledge of wgpu has given me.

## License

My code is made available under the terms of the [MIT license](LICENSE).

The original Learn OpenGL code is copyright Joey de Vries and made available under the terms of the CC BY-NC 4.0 license ([human readable format](https://creativecommons.org/licenses/by-nc/4.0/), [full license](https://creativecommons.org/licenses/by-nc/4.0/legalcode)).
Images and other media from the original Learn OpenGL tutorials are made available under the terms of the CC BY 4.0 license ([human readable format](https://creativecommons.org/licenses/by/4.0/), [full license](https://creativecommons.org/licenses/by/4.0/legalcode)).
