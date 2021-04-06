# rusrat
RUSt(ic|ed|y) RAy Tracer - a recursive ray tracer written in Rust, complete with reflection and refraction.

Generates images such as this:
![First example image](https://raw.githubusercontent.com/gcohara/rusrat/main/examples/example_1.png)

## Usage

Scenes are specified using a YAML file.
Rusrat can then be called on this scene description:
```bash
cargo run my_scene.yaml
```

Note that rendering times can be very long for complicated scenes - for instance, `ball-in-ball.yaml` took _90 minutes_ on a 2014 MBP.

## YAML Specification

The YAML files consist of a series of elements.
There are four possible elements:

### **Camera:**
This can be thought of as the position of the 'eye' in the scene. It defines the point of view the scene will be rendered from. It has the following properties:
* **Width:** The width of the output image in pixels.
* **Height:** The height of the output image in pixels.
* **Field of View:** The angular extent of what can be seen from the camera - higher values have the appearance of being a fisheye lens. This should not be set below 0 or above 2п (~6.28).
* **From:** The position of the camera.
* **To:** The direction the camera points.
* **Up:** The direction that is 'up' relative to the camera's view.

As an example:
```yaml
- add: camera
  width: 100
  height: 100
  field-of-view: 0.785
  from: [-6, 6, -10]
  to: [6, 0, 6]
  up: [-0.45, 1, 0]
```
        
### **Light:**
This defines a point light source. There can be more than one! It has two properties:
* **Intensity:** The colour of the light source in RGB. This is a list of three values, each between 0 and 1 inclusive.
* **At:** The position of the light.
    
An example of a light:
```yaml
- add: light
  at: [50, 100, -50]
  intensity: [1, 1, 1]
```
    
### **Sphere:**
Technically, this defines a ball rather than a sphere (a ball is the full 3D object, while a sphere is the 2D surface). It has two properties:
* **Material:** Properties of the material that the sphere is constructed from. See below for further details.
* **Transform:** The position, size, shape, and orientation of the sphere within space. See below for further details.
    
### **Plane:**
This defines a plane. It has the same two properties as a sphere.

For the shapes, there are two properties requiring further explanation.

### **Transform:**
  This is an array of lists that define a sequence of transforms to be applied to the object. The possible transforms are:
  
* **Scale:** Scales the object. Since it scales independently in the x, y, and z directions, it can be used to 'stretch' in one direction and shrink it in another.
  
  ` - [scale, magnitude of scaling in x direction, in y direction, in z direction]`
* **Rotate-x/y/z:** Rotates the object around the x/y/z axis.
  
  ` - [rotate-x/y/z, angle of rotation in radians]`
* **Translate:** Moves the object around in space.

  `- [translate, displacement in x direction, in y direction, in z direction]`
* **Shear:** Shears the object.
  
Transforms should be specified as follows:
```yaml
transform:
  - [rotate-x, 1.5707]
  - [rotate-y, 1]
  - [rotate-z, 2.456]
  - [scale, 1, 2, 2.5]
  - [translate, 3, 4.5, 6]
  
```

