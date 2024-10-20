use cgmath::{ElementWise, InnerSpace};
use image::*;
use indicatif::{MultiProgress, ProgressBar};
use std::f64::INFINITY;
use std::sync::{mpsc, Arc};
use std::thread;

use crate::raycasting::Hittable;
use crate::raycasting::{Color3, Point3, Vec3};
use crate::raycasting::{HittableList, Intersection, Ray};
// use crate::materials::Material;
use crate::rand_util::Rand;

pub struct Camera {
    pub aspect_ratio: f64,      // Ratio of Image width / height
    pub image_width: u32,       // Rendered Image width in pixels
    pub samples_per_pixel: u32, // Count of random samples per pixel
	pub max_depth: u32,         // Maximum number of ray bounces into scene
	pub v_fov: f64,             // Vertical Field of View (view angle in degrees)
	pub lookfrom: Point3,       // Point camera is looking from
	pub lookat: Point3,         // Point camera is looking at
	pub vup: Vec3,              // Camera-relative "up" direction
	pub defocus_angle: f64,     // Variation angle of rays through each pixel
	pub focus_dist: f64,        // Distance from camera lookpoint to plane of perfect focus
    image_height: u32,          // Rendered Image height in pixels
    center: Point3,             // Camera Center
	pixel_samples_scale: f64,   // Color scale factor for a sum of pixel samples
    pixel_top_left: Point3,     // Location of pixel (0, 0)
    pixel_delta_u: Vec3,        // Offset of pixel to the right
    pixel_delta_v: Vec3,        // Offset of pixel below
	uvw: (Vec3, Vec3, Vec3),    // Camera Basis Vectors
	defocus_disk_u: Vec3,       // Defocus Disk Vertical Radius
	defocus_disk_v: Vec3,       // Defocus Disk Horizontal Radius
	// samples_per_thread: u32,    //  Count of random samples per thread
}

impl Camera {
    pub fn default() -> Self {
        Camera {
            aspect_ratio: 1.0,
            image_width: 100,
			samples_per_pixel: 10,
            image_height: 100,
			max_depth: 10,
			v_fov: 90.0,
			defocus_angle: 0.0,
			focus_dist: 10.0,
			lookfrom: Point3::new(0.0,0.0,0.0),
			lookat: Point3::new(0.0,0.0,-1.0),
			vup: Vec3::new(0.0,1.0,0.0),
            center: Point3::new(0.0, 0.0, 0.0),
			pixel_samples_scale: 0.1,
            pixel_top_left: Point3::new(0.0, 0.0, 0.0),
            pixel_delta_u: Vec3::new(0.0, 0.0, 0.0),
            pixel_delta_v: Vec3::new(0.0, 0.0, 0.0),
			uvw: (Vec3::unit_x(), Vec3::unit_y(), Vec3::unit_z()),
			defocus_disk_u: Vec3::new(0.21033701267083865, -0.0, 0.21033701267083865),
			defocus_disk_v: Vec3::new(0.12143813088605043, 0.24287626177210087, -0.12143813088605043),
			// samples_per_thread: 1,
        }
    }

    pub fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as u32;
        self.image_height = if self.image_height < 1 {
            1
        } else {
            self.image_height
        };

		self.pixel_samples_scale = 1.0 / (self.samples_per_pixel as f64);
		// self.samples_per_thread = self.samples_per_pixel / 10;

        // Camera
		let h = (self.v_fov.to_radians() / 2.0).tan();
		let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        self.center = self.lookfrom;

		// Camera Basis Vectors
		let w = (self.lookfrom - self.lookat).normalize();
		let u = self.vup.cross(w).normalize();
		let v = w.cross(u);
		self.uvw = (u,v,w);

        // Vectors Along Viewport
        let (viewport_u, viewport_v) = (
            viewport_width * u,
			viewport_height * -v
        );
        // Pixel Deltas
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // upper left (starting pixel)
        let viewport_upper_left =
            self.center - (self.focus_dist * w) - 0.5 * (viewport_u + viewport_v);
        self.pixel_top_left = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

		// Calculate Camera's defocus disk basis vectors
		let defocus_radius = self.focus_dist * (self.defocus_angle / 2.0).to_radians().tan();
		self.defocus_disk_u = u * defocus_radius;
		self.defocus_disk_v = v * defocus_radius;
    }

	pub fn render_parallel(s: Self, path: &String, world: HittableList) {
		let (tx, rx) = mpsc::channel();
		let se = Arc::new(s);
		let wo = Arc::new(world);
		let mut imgbuf = image::RgbImage::new(se.image_width, se.image_height);

		let (iw, ih, samples, depth, sample_scale) = (se.image_width, se.image_height, se.samples_per_pixel, se.max_depth, se.pixel_samples_scale);
		let step = iw / 10;

		let multiprogress = MultiProgress::new();

		for i in 0..10 {
			let (curr, next) = (i * step, (i + 1) * step);
			let (se1, wo1) = (Arc::clone(&se), Arc::clone(&wo));
			let tx1 = tx.clone();
			let p1 = multiprogress.add(ProgressBar::new((step * ih) as u64));
			thread::spawn(move || {
				let mut rng: Rand = Rand::new();
				for x in curr..next {
					for y in 0..ih {
						let mut pixel_color = Color3::new(0.0, 0.0, 0.0);
						for _ in 0..samples {
							let r: Ray = se1.get_ray(x, y, &mut rng);
							pixel_color += se1.ray_color(&r, depth, &wo1, &mut rng)
						}

						p1.inc(1);
						tx1.send((x,y,Camera::to_rgb(sample_scale * pixel_color))).unwrap();
					}
				}
			});
		}
		drop(tx);

		let progress = multiprogress.add(ProgressBar::new((iw * ih) as u64));
		for (rx, ry, rp) in rx {
			imgbuf.put_pixel(rx, ry, rp);
			progress.inc(1);
		}


		imgbuf.save(format!("images/{}.png", path)).unwrap();
	}

    // pub fn render(&self, path: &String, world: &HittableList) {
    //     let mut imgbuf = image::RgbImage::new(self.image_width, self.image_height);
	// 	let mut rng: Rand = Rand::new();

    //     for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
	// 		// let (tx, rx) = mpsc::channel();
	// 		let mut pixel_color = Color3::new(0.0, 0.0, 0.0);

	// 		for _ in 0..self.samples_per_pixel {
	// 			let r: Ray = self.get_ray(x, y, &mut rng);
	// 			pixel_color += self.ray_color(&r, self.max_depth, world, &mut rng);
	// 		}

	// 		*pixel = Camera::to_rgb(self.pixel_samples_scale * pixel_color);
    //     }

    //     imgbuf.save(format!("images/{}.png", path)).unwrap();
    // }

	fn get_ray(&self, i: u32, j: u32, rng: &mut Rand) -> Ray {
		// Construct a camera ray from the defocus disk and to a random point
		// around i, j
		
		let (x, y) = (i as f64, j as f64);

		let (offset_x, offset_y) = (
			rng.random_double() - 0.5,
			rng.random_double() - 0.5,
		);

		let pixel_sample = self.pixel_top_left
			+ ((x + offset_x) * self.pixel_delta_u)
			+ ((y + offset_y) * self.pixel_delta_v);
		
		let ray_origin = if self.defocus_angle <= 0.0 {self.center} else {self.defocus_disk_sample(rng)};
		let ray_direction = pixel_sample - ray_origin;
		
		Ray::new(ray_origin, ray_direction)
	}

	fn linear_to_gamma(n: f64) -> f64 {
		n.sqrt()
	}

	fn to_rgb(color: Color3) -> Rgb<u8> {
		let (x, y, z) = (
			(256.0 * Camera::linear_to_gamma(color.x.clamp(0.0, 0.9999))) as u8,
			(256.0 * Camera::linear_to_gamma(color.y.clamp(0.0, 0.9999))) as u8,
			(256.0 * Camera::linear_to_gamma(color.z.clamp(0.0, 0.9999))) as u8,
		);
	
		Rgb([x, y, z])
	}
	
	fn ray_color(&self, r: &Ray, depth: u32, world: &HittableList, rng: &mut Rand) -> Color3 {
		// Exceeded the bounce limit
		if depth <= 0 {
			return Color3::new(0.0, 0.0, 0.0);
		}

		let mut rec: Intersection = Intersection::new();
	
		if world.hit(r, &(0.001..INFINITY), &mut rec) {
			if let Some((scattered, attenuation)) = rec.mat.scatter(r, &rec, rng) {
				return attenuation.mul_element_wise(self.ray_color(&scattered, depth - 1, world, rng));
			}
			return Color3::new(0.0, 0.0, 0.0);
		}
	
		let unit = r.dir.normalize();
		let alpha = 0.5 * (unit.y + 1.0);
		Color3::new(1.0, 1.0, 1.0) * (1.0 - alpha) + alpha * Color3::new(0.5, 0.7, 1.0)
	}

	fn defocus_disk_sample(&self, rng: &mut Rand) -> Vec3 {
		let p = rng.random_in_disk();
		self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
	}
}
