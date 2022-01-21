# clrays

## Features
- [x] camera controls
- [x] custom keybindings
- [x] skycolour, skybox: sphere
- [x] export frame
- [x] BVH: binning + SAH + top-level
- [ ] BVH: 4 way

### GPU
- [x] basic pathtracer (area lights, materials, speculars, dielectrics, beer's law)
- [x] frame energy
- [ ] Utilize BVH
- [ ] Microfacet materials

### CPU
- [x] primitives: planes, spheres, triangles
- [x] material
- [x] blinn shading
- [x] reflection
- [x] refraction
- [x] absorption
- [x] multithreading
- [x] post: gamma, vignetting, chromatic aberration
- [x] AA: randomly sampled
- [x] textures: albedo, normal, roughness, metalic
- [x] barrel distortion, fish eye lens
- [x] mesh: triangle meshes (.obj)
- [x] progressive anti aliasing
- [x] adaptive resolution
- [x] bilinear texture sampling for all supported texture maps
- [x] Utilize top-level BVH

## Controls

Two examples of keybindings, one in qwerty with wasd gaming bindings and one in qgmlwy leaving your hands in touch typing position.
Can be rebound to anything you want.

Layout  | Style | Move: up, down, forward, backward, left, right | Look: up, down, left, right | Toggle focus mode | Export frame
--------|-------|------------------------------------------------|-----------------------------|-------------------|---------------
QWERTY  |Gaming | Q, E, W, S, A, D                               | I, K, J, L                  | U                 | O
QGMLWY  |Typing | G, L, M, T, S, N                               | U, E, A, O                  | F                 | B

## Possible things to work on
- portals
- hdr skybox
- sphere skybox only tophalf option
- skybox cubemap
- procedural sky
- denoising
- optimize pow: gamma correct images before upload
- optimize vector loading: use vload3 and allign the buffer for it
- preprocess kernel: optimize branches away, insert constants
- sRGB now, use aces, linear colours
- Next event estimation (NEE)
- Russian roulette (RR)
- Importance sampling of BRDF
- Importance sampling of lights
- Depth of field
- Blue noise
- Multiple importance samplign (MIS)
- Spectral rendering
- motion blur
