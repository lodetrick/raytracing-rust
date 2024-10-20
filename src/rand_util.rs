use crate::raycasting::Vec3;
use cgmath::InnerSpace;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::SeedableRng;

pub struct Rand {
    range: Uniform<f64>,
	rng: SmallRng,
}

impl Rand {
	pub fn new() -> Self {
		Rand {
			range: Uniform::from(0.0..1.0),
			rng: SmallRng::from_entropy(),
		}
	}

	pub fn random_double(&mut self) -> f64 {
		self.range.sample(&mut self.rng)
	}

	pub fn random_vec(&mut self) -> Vec3 {
		Vec3::new(self.random_double(),self.random_double(),self.random_double())
	}

	pub fn random_in_disk(&mut self) -> Vec3 {
		let mut r: Vec3;
		loop {
			r = Vec3::new(self.random_double() * 2.0 - 1.0, self.random_double() * 2.0 - 1.0, 0.0);
			if r.magnitude2() < 1.0 {
				return r;
			}
		}
	}

	pub fn random_unit_vec(&mut self) -> Vec3 {
		let mut r: Vec3;
		loop {
			r = self.random_vec() * 2.0 - Vec3::new(1.0,1.0,1.0);
			if r.magnitude2() < 1.0 {
				return r.normalize();
			}
		}
	}
}