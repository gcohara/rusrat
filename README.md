# rusrat
RUSt(ic|ed|y) RAy Tracer - a recursive ray tracer written in Rust, complete with reflection and refraction.

Generates images such as this:
![First example image](https://raw.githubusercontent.com/gcohara/rusrat/main/examples/example_1.png)

And this:
![Second example image](https://raw.githubusercontent.com/gcohara/rusrat/main/examples/ball-in-ball.png)



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
This can be thought of as the position of the 'eye' in the scene. It defines the point of view the scene will be rendered from. It has the following properties, all of which must be specified:
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
This defines a point light source. There can be more than one! It has two properties which must both be specified:
* **Intensity:** The colour of the light source in RGB. This is a list of three values, each between 0 and 1 inclusive.
* **At:** The position of the light.
    
An example of a light:
```yaml
- add: light
  at: [50, 100, -50]
  intensity: [1, 1, 1]
```
    
### **Sphere:**
Technically, this defines a ball rather than a sphere (a ball is the full 3D object, while a sphere is the 2D surface). It has two properties, both of which are optional to specify:
* **Material:** Properties of the material that the sphere is constructed from. See below for further details.
* **Transform:** The position, size, shape, and orientation of the sphere within space. See below for further details.
    
### **Plane:**
This defines a plane. It has the same two properties as a sphere.

For the shapes, there are two properties requiring further explanation.

### **Transform:**
This is an array of lists that define a sequence of transforms to be applied to the object. The default transform is the identity matrix - in other words, the object is situated at the origin. The possible transforms are:
  
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

### **Material:**
Defines properties of a material. Not all properties have to be specified - those that aren't are given default values. The options are:

* **Colour:**
The colour of the material. Three values between 0 and 1 respectively representing the red, green, and blue components. The default is white.
* **Ambient:**
  The contribution of background lighting to the lighting of the object. This is constant for the whole object, and doesn't depend on the normal to the object. Sensible values are between 0 and 1. The default is 0.1.
* **Diffuse:**
  The amount of diffuse reflection from the surface. Depends on the angle between the incident light and the normal to the surface. Sensible values are between 0 and 1. The default is 0.9.
* **Specular:**
  The intensity of the specular reflection, which is the bright spot that can be seen on glossy surfaces. Sensible values are between 0 and 1. The default is 0.9.
* **Shininess:**
  The size and tightness of the bright spot due to specular reflection. Larger values make the spot smaller. Sensible values are from 10 to 200. The default is 200.
* **Reflectivity:**
  How reflective the surface of the object is. Values range from 0 (nonreflective) to 1 (a mirror). The default is 0.
* **Transparency:**
  How transparent the object is. Ranges from 0 (opaque) to 1 (completely transparent). The default is 0.
* **Refractive Index:**
  Determines how much a ray of light bends when entering the object. Larger numbers mean the light bends more. Some examples of sensible values are 1 for a perfect vacuum, 1.5 for glass, and 2.4 for diamond. The default is 1.
* **Pattern:**
  Has three sub-properties. Patterns are optional, and the default is no pattern.
  * **Type:**
    The type of pattern. Possible values are `3d-check` for a checkered pattern, and `stripe` for stripes.
  * **Colour A:**
    One colour of the pattern.
  * **Colour B:**
    The other colour of the pattern.
So, for example:
```yaml
material:
  colour: [1,1,1]
  ambient: 1
  diffuse: 0
  specular: 0
  pattern:
    type: 3d-check
    colour-a: [0.9, 0.9, 0.9]
    colour-b: [0.2, 0.2, 0.2]
```
