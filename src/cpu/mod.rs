use crate::scene::{ Scene, Material, Sphere, Plane };
use crate::vec3::Vec3;

const AA: f32 = 1.0;
const MAX_RENDER_DEPTH: u8 = 3;
const GAMMA: f32 = 2.2;
const PI: f32 = std::f32::consts::PI;
const MAX_RENDER_DIST: f32 = 1000000.0;
const EPSILON: f32 = 0.001;
const AMBIENT: f32 = 0.05;

pub fn test(w: usize, h: usize, screen: &mut Vec<u32>){
    for x in 0..w{
    for y in 0..h{
        let mut uv = Vec3::new(x as f32 / w as f32, y as f32 / h as f32, 0.0);
        uv.add_scalar(-0.5);
        uv.mul(Vec3::new(w as f32 / h as f32, -1.0, 0.0));
        let val = (Vec3::ZERO.dist(uv) * 255.0).min(255.0) as u32;
        screen[x + y * w] = (val << 16) + (val << 8) + val;
    }
    }
}

pub fn whitted(w: usize, h: usize, scene: &Scene, screen: &mut Vec<u32>, tex_params: &[u32], textures: &[u8]){
    let pos = scene.cam_pos;
    let cd = scene.cam_dir.normalized_fast();
    let aspect = w as f32 / h as f32;
    let uv_dist = (aspect / 2.0) / (scene.cam_fov / 2.0 * 0.01745329).tan();
    for x in 0..w{
    for y in 0..h{
        let hor = cd.crossed(Vec3::UP).normalized_fast();
        let ver = hor.crossed(cd).normalized_fast();
        let mut uv = Vec3::new(x as f32 / (w as f32 * AA), y as f32 / (h as f32 * AA), 0.0);
        uv.add_scalar(-0.5);
        uv.mul(Vec3::new(aspect, -1.0, 0.0));
        let mut to = pos.added(cd.scaled(uv_dist));
        to.add(hor.scaled(uv.x));
        to.add(ver.scaled(uv.y));
        let ray = Ray{ pos, dir: to.subed(pos).normalized_fast() };

        let mut col = whitted_trace(ray, scene, tex_params, textures, MAX_RENDER_DEPTH);
        col.pow_scalar(1.0 / GAMMA);
        if AA == 1.0{
            col.clamp(0.0, 1.0);
        }
        col.div_scalar_fast(AA * AA);
        screen[x + y * w] = (((col.x * 255.0) as u32) << 16) + (((col.y * 255.0) as u32) << 8) + (col.z * 255.0) as u32;
    }
    }
}

fn whitted_trace(ray: Ray, scene: &Scene, tps: &[u32], ts: &[u8], depth: u8) -> Vec3{
    if depth == 0 {
        return get_sky_col(ray.dir, scene, tps, ts);
    }

    // hit
    let mut hit = inter_scene(ray, scene);
    if hit.is_null(){
        return get_sky_col(ray.dir, scene, tps, ts);
    }
    let mat = hit.mat.unwrap();

    // texture
    let mut texcol = Vec3::ONE;
    let uv = if
        mat.texture > 0 ||
        mat.normal_map > 0 ||
        mat.roughness_map > 0 ||
        mat.metalic_map > 0
    {
        let uvtype = hit.uvtype;
        let uv = if uvtype == UV_SPHERE{
            sphere_uv(hit.nor)
        } else {
            plane_uv(hit.pos, hit.nor)
        };
        (uv.0 * mat.tex_scale, uv.1 * mat.tex_scale)
    } else {
        (0.0, 0.0)
    };

    if mat.texture > 0{
        texcol = get_tex_col(mat.texture - 1, uv, tps, ts);
    }

    // normalmap
    if mat.normal_map > 0{
        let mut rawnor = get_tex_val(mat.normal_map - 1, uv, tps, ts);
        let mut t = Vec3::crossed(hit.nor, Vec3::UP);
        if t.len() < EPSILON{
            t = Vec3::crossed(hit.nor, Vec3::FORWARD);
        }
        t.normalize_fast();
        let b = Vec3::normalized_fast(Vec3::crossed(hit.nor, t));
        rawnor = rawnor.scaled(2.0).added_scalar(-1.0);
        rawnor.normalize_fast();
        let mut newnor = Vec3::ZERO;
        let mut row = Vec3::new(t.x, b.x, hit.nor.x);
        newnor.x = Vec3::dot(row, rawnor);
        row = Vec3::new(t.y, b.y, hit.nor.y);
        newnor.y = Vec3::dot(row, rawnor);
        row = Vec3::new(t.z, b.z, hit.nor.z);
        newnor.z = Vec3::dot(row, rawnor);
        hit.nor = newnor.normalized_fast();
    }

    // roughnessmap
    let mut roughness = mat.roughness;
    if mat.roughness_map > 0{
        let value = get_tex_scalar(mat.roughness_map - 1, uv, tps, ts);
        roughness *= value;
    }

    // metalicmap
    let mut reflectivity = mat.reflectivity;
    if mat.metalic_map > 0{
        let value = get_tex_scalar(mat.metalic_map - 1, uv, tps, ts);
        reflectivity *= value;
    }

    // diffuse, specular
    let (mut diff, spec) = blinn(&hit, mat, roughness, scene, ray.dir);
    diff.mul(texcol);

    //reflection
    let newdir = Vec3::normalized_fast(Vec3::reflected(ray.dir, hit.nor));
    let nray = Ray{ pos: hit.pos.added(hit.nor.scaled(EPSILON)), dir: newdir };

    let refl = whitted_trace(nray, scene, tps, ts, depth - 1);
    (diff.scaled(1.0 - reflectivity)).added(refl.scaled(reflectivity)).added(spec)
}

// SHADING ------------------------------------------------------------

// get diffuse light incl colour of hit with all lights
fn blinn(hit: &RayHit, mat: &Material, roughness: f32, scene: &Scene, viewdir: Vec3) -> (Vec3, Vec3){
    let mut col = Vec3::ONE.scaled(AMBIENT);
    let mut spec = Vec3::ZERO;
    for light in &scene.lights{
        let res = blinn_single(roughness, light.pos, light.intensity, viewdir, hit, scene);
        col.add(light.col.scaled(res.0));
        spec.add(light.col.scaled(res.1));
    }
    (col.muled(mat.col), spec.scaled(1.0 - roughness))
}

// get diffuse light strength for hit for a light
fn blinn_single(roughness: f32, lpos: Vec3, lpow: f32, viewdir: Vec3, hit: &RayHit, scene: &Scene) -> (f32, f32){
    let mut to_l = Vec3::subed(lpos, hit.pos);
    let dist = Vec3::len(to_l);
    to_l.scale(1.0 / (dist + EPSILON));
    // diffuse
    let mut angle = Vec3::dot(hit.nor, to_l);
    if angle < EPSILON{
        return (0.0, 0.0);
    }
    angle = angle.max(0.0);
    let power = lpow / (PI * 4.0 * dist * dist);
    if power < 0.01{
        return (0.0, 0.0);
    }
    // exposed to light or not
    let lray = Ray { pos: hit.pos.added(hit.nor.scaled(EPSILON)), dir: to_l };
    let lhit = inter_scene(lray, scene);
    if !lhit.is_null() && lhit.t < dist{
        return (0.0, 0.0);
    }
    // specular
    let halfdir = Vec3::normalized_fast(to_l.subed(viewdir));
    let specangle = Vec3::dot(halfdir, hit.nor).max(0.0);
    let spec = specangle.powf(16.0 / roughness);
    (angle * power, spec * power)
}

// UV's ------------------------------------------------------------

// plane uv
fn plane_uv(pos: Vec3, nor: Vec3) -> (f32, f32){
    let u = Vec3::new(nor.y, nor.z, -nor.x);
    let v = Vec3::crossed(u, nor).normalized_fast();
    (Vec3::dot(pos, u), Vec3::dot(pos, v))
}

// sphere uv
fn sphere_uv(nor: Vec3) -> (f32, f32){
    let u = 0.5 + (f32::atan2(-nor.z, -nor.x) / (2.0 * PI));
    let v = 0.5 - (f32::asin(-nor.y) / PI);
    (u, v)
}

// sphere skybox uv(just sphere uv with inverted normal)
fn sky_sphere_uv(nor: Vec3) -> (f32, f32){
    let u = 0.5 + (f32::atan2(nor.z, nor.x) / (2.0 * PI));
    let v = 0.5 - (f32::asin(nor.y) / PI);
    (u, v)
}

// INTERSECTING ------------------------------------------------------------

const UV_PLANE: u8 = 0;
const UV_SPHERE: u8 = 1;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct Ray{
    pub pos: Vec3,
    pub dir: Vec3,
}

#[derive(Clone)]
struct RayHit<'a>{
    pub pos: Vec3,
    pub nor: Vec3,
    pub t: f32,
    pub mat: Option<&'a Material>,
    pub uvtype: u8,
}

impl RayHit<'_>{
    pub const NULL: Self = RayHit{ pos: Vec3::ZERO, nor: Vec3::ZERO, t: MAX_RENDER_DIST, mat: None, uvtype: 255 };

    #[inline]
    pub fn is_null(&self) -> bool{
        self.uvtype == 255
    }
}

// ray-sphere intersection
#[inline]
fn inter_sphere<'a>(ray: Ray, sphere: &'a Sphere, closest: &mut RayHit<'a>){
    let l = Vec3::subed(sphere.pos, ray.pos);
    let tca = Vec3::dot(ray.dir, l);
    let d = tca*tca - Vec3::dot(l, l) + sphere.rad*sphere.rad;
    if d < 0.0 { return; }
    let dsqrt = d.sqrt();
    let mut t = tca - dsqrt;
    if t < 0.0 {
        t = tca + dsqrt;
        if t < 0.0 { return; }
    }
    if t > closest.t { return; }
    closest.t = t;
    closest.pos = ray.pos.added(ray.dir.scaled(t));
    closest.nor = Vec3::subed(closest.pos, sphere.pos).scaled(1.0 / sphere.rad);
    closest.mat = Some(&sphere.mat);
    closest.uvtype = UV_SPHERE;
}

// ray-plane intersection
#[inline]
fn inter_plane<'a>(ray: Ray, plane: &'a Plane, closest: &mut RayHit<'a>){
    let divisor = Vec3::dot(ray.dir, plane.nor);
    if divisor.abs() < EPSILON { return; }
    let planevec = Vec3::subed(plane.pos, ray.pos);
    let t = Vec3::dot(planevec, plane.nor) / divisor;
    if t < EPSILON { return; }
    if t > closest.t { return; }
    closest.t = t;
    closest.pos = ray.pos.added(ray.dir.scaled(t));
    closest.nor = plane.nor;
    closest.mat = Some(&plane.mat);
    closest.uvtype = UV_PLANE;
}

// intersect whole scene
fn inter_scene(ray: Ray, scene: &Scene) -> RayHit{
    let mut closest = RayHit::NULL;
    for plane in &scene.planes { inter_plane(ray, plane, &mut closest); }
    for sphere in &scene.spheres { inter_sphere(ray, sphere, &mut closest); }
    closest
}

// TEXTURES ------------------------------------------------------------

// first byte of texture
#[inline]
fn tx_get_start(tex: u32, tps: &[u32]) -> usize{
    tps[tex as usize * 3] as usize
}

#[inline]
fn tx_get_width(tex: u32, tps: &[u32]) -> u32{
    tps[tex as usize * 3 + 1]
}

#[inline]
fn tx_get_height(tex: u32, tps: &[u32]) -> u32{
    tps[tex as usize * 3 + 2]
}

// get sample
#[inline]
fn tx_get_sample(tex: u32, tps: &[u32], ts: &[u8], x: u32, y: u32, w: u32) -> Vec3{
    let offset = tx_get_start(tex, tps) + ((y * w + x) * 3) as usize;
    let col = Vec3::new(ts[offset    ] as f32,
                        ts[offset + 1] as f32,
                        ts[offset + 2] as f32);
    col.dived_scalar_fast(255.0)
}

// shared logic
#[inline]
#[allow(clippy::many_single_char_names)]
fn uv_to_xy(uv: (f32, f32), tex: u32, tps: &[u32]) -> (u32, u32, u32){
    let mut u = uv.0.fract();
    let mut v = uv.1.fract();
    if u < 0.0 { u += 1.0; }
    if v < 0.0 { v += 1.0; }
    let w = tx_get_width(tex, tps);
    let x = (w as f32* u) as u32;
    let y = (tx_get_height(tex, tps) as f32 * v) as u32;
    (w, x, y)
}

// get sky colour
#[inline]
fn get_sky_col(nor: Vec3, scene: &Scene, tps: &[u32], ts: &[u8]) -> Vec3{
    if scene.sky_box == 0{
        return scene.sky_col;
    }
    let uv = sky_sphere_uv(nor);
    get_tex_col(scene.sky_box - 1, uv, tps, ts)
}

// get colour from texture and uv
#[inline]
fn get_tex_col(tex: u32, uv: (f32, f32), tps: &[u32], ts: &[u8]) -> Vec3{
    let (w, x, y) = uv_to_xy(uv, tex, tps);
    tx_get_sample(tex, tps, ts, x, y, w).powed_scalar(GAMMA)
}

// get value to range 0..1 (no gamma)
#[inline]
fn get_tex_val(tex: u32, uv: (f32, f32), tps: &[u32], ts: &[u8]) -> Vec3{
    let (w, x, y) = uv_to_xy(uv, tex, tps);
    tx_get_sample(tex, tps, ts, x, y, w)
}

// get value 0..1 from scalar map
#[inline]
fn get_tex_scalar(tex: u32, uv: (f32, f32), tps: &[u32], ts: &[u8]) -> f32{
    let (w, x, y) = uv_to_xy(uv, tex, tps);
    let offset = tx_get_start(tex, tps) + ((y * w + x) as usize);
    let scalar = ts[offset] as f32;
    scalar / 255.0
}

