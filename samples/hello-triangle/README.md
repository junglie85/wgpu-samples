# Hello triangle

Based on the first half of [https://learnopengl.com/Getting-started/Hello-Triangle](https://learnopengl.com/Getting-started/Hello-Triangle)

Note that instead of using `device.create_buffer` and then writing data to the buffer we could do this in a single step using `wgpu::util::DeviceExt`:

```rust
let vbo = device.create_buffer_init(&BufferInitDescriptor {
    label: None,
    contents: cast_slice(&vertices),
    usage: BufferUsages::VERTEX,
});
```

This is useful when the data is known at buffer creation time.
