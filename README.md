# rusrat
RUSt(ic|ed|y) RAy Tracer - a recursive ray tracer written in Rust, complete with reflection and refraction.

Generates images such as this:
![First example image](https://raw.githubusercontent.com/gcohara/rusrat/main/examples/example_1.png)

## Usage

Scenes are specified using a YAML file.
Rusrat can then be called on this scene description:
```
cargo run my_scene.yaml
```

Note that rendering times can be very long for complicated scenes - for instance, `ball-in-ball.yaml` took _90 minutes_ on a 2014 MBP.

## API Specification

The YAML files consist of a series of elements.
There are four possible elements:
* **Camera**
This can be thought of as the position of the 'eye' in the scene. It defines the point of view the scene will be rendered from. It has the following properties:
    * *Width* The width of the output image in pixels.
    * *Height* The height of the output image in pixels.
    * *Field of View* The angular extent of what can be seen from the camera - higher values have the appearance of being a fisheye lens. This should not be set below 0 or above 2п (~6.28).
    * *From* The position of the camera.
    * *To* The direction the camera points.
    * *Up* The direction that is 'up' relative to the camera's view.
As an example:
```
- add: camera
  width: 100
  height: 100
  field-of-view: 0.785
  from: [-6, 6, -10]
  to: [6, 0, 6]
  up: [-0.45, 1, 0]
  ```
    
* **Light**
This defines a point light source. There can be more than one!
* **Sphere**
Technically, this defines a ball rather than a sphere (a ball is the full 3D object, while a sphere is the 2D surface).
* **Plane**
This unsurprisingly defines a plane.

