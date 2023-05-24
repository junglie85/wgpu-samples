# More cubes

Based on the More cubes section of [https://learnopengl.com/Getting-started/Coordinate-Systems](https://learnopengl.com/Getting-started/Coordinate-Systems).

Instead of using multiple draw commands to render multiple instances of the same model with a different transform we use instancing instead.
This is covered in the advanced OpenGL section of the original tutorial.
You could create a uniform buffer per object and upload the necessary data for each but that is cummbersome with a large number of objects.
