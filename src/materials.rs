use cgmath::{dot, InnerSpace};

use crate::raycasting::{Ray, Intersection};
use crate::raycasting::{Color3, Vec3};
use crate::rand_util::Rand;

pub trait Material {
	fn scatter(&self, r_in: &Ray, rec: &Intersection, rng: &mut Rand) -> Option<(Ray, Color3)>;
}

pub struct Lambertian {
	albedo: Color3,
}

impl Lambertian {
	pub fn new(albedo: Color3) -> Self {
		Lambertian {
			albedo
		}
	}
}

impl Material for Lambertian {
	fn scatter(&self, _r_in: &Ray, rec: &Intersection, rng: &mut Rand) -> Option<(Ray, Color3)> {
		let mut scatter_direction = rec.norm + rng.random_unit_vec();
		if vec_near_zero(&scatter_direction) {
			scatter_direction = rec.norm;
		}
		Some((Ray::new(rec.position, scatter_direction), self.albedo.clone()))
	}
}

pub struct Metal {
	albedo: Color3,
	fuzz: f64,
}

impl Metal {
	pub fn new(albedo: Color3, fuzz: f64) -> Self {
		Metal {
			albedo,
			fuzz: if fuzz < 1.0 {fuzz} else {1.0}
		}
	}
}

impl Material for Metal {
	fn scatter(&self, r_in: &Ray, rec: &Intersection, rng: &mut Rand) -> Option<(Ray, Color3)> {
		let reflected = reflect(&r_in.dir, &rec.norm).normalize() + (self.fuzz * rng.random_unit_vec());

		if dot(reflected, rec.norm) <= 0.0 {
			return None;
		}
		Some((Ray::new(rec.position, reflected), self.albedo.clone()))
	}
}

pub struct Dielectric {
	refractive_index: f64,
}

impl Dielectric {
	pub fn new(refractive_index: f64) -> Self {
		Dielectric {
			refractive_index,
		}
	}

	fn reflectance(cos: f64, refractive_index: f64) -> f64 {
		// Use Schlick's Approximation for Reflectance
		let r0 = (1.0 - refractive_index) / (1.0 + refractive_index);
		let r0 = r0 * r0;
		
		r0 + (1.0 - r0) * (1.0 - cos).powi(5)
	}
}

impl Material for Dielectric {
	fn scatter(&self, r_in: &Ray, rec: &Intersection, rng: &mut Rand) -> Option<(Ray, Color3)> {
		let ri = if rec.front_face {1.0 / self.refractive_index} else {self.refractive_index};

		let unit_direction = r_in.dir.normalize();
		let cos_theta = f64::min(dot(-unit_direction, rec.norm), 1.0);
		let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

		let direction: Vec3;

		if ri * sin_theta > 1.0 || Dielectric::reflectance(cos_theta, ri) > rng.random_double() {
			direction = reflect(&unit_direction, &rec.norm);
		}
		else {
			direction = refract(&r_in.dir.normalize(), &rec.norm, ri);
		}

		Some((Ray::new(rec.position, direction), Color3::new(1.0, 1.0, 1.0)))
	}
}

fn reflect(vec: &Vec3, norm: &Vec3) -> Vec3 {
	vec - 2.0 * dot(*vec, *norm) * norm
}

fn refract(vec: &Vec3, norm: &Vec3, etai_over_etat: f64) -> Vec3 {
	let cos_theta = f64::min(1.0, dot(-*vec, *norm));
	let r_out_perp = etai_over_etat * (vec + cos_theta * norm);
	let r_out_parallel = -((1.0 - r_out_perp.magnitude2()).abs().sqrt()) * norm;
	r_out_parallel + r_out_perp
}

fn vec_near_zero(vec: &Vec3) -> bool {
	let s = 1e-8;
	(vec.x.abs() < s) && (vec.y.abs() < s) && (vec.z.abs() < s)
}