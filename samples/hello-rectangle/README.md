# Hello rectangle

Based on the second half of [https://learnopengl.com/Getting-started/Hello-Triangle](https://learnopengl.com/Getting-started/Hello-Triangle) from **Element Buffer Objects** onward.

Press `space` to toggle between _filled_ and _wireframe_ pipelines.

Note that we need multiple definitions of the render pipeline with different `polygon_mode`'s:

- `fill_pipeline` uses `PolygonMode::Fill`
- `wireframe_pipeline` uses `PolygonMode::Line`

Only `POLYGON_MODE_FILL` is enabled by default so the `POLYGONE_MODE_LINE` feature must be enabled explicitly.
