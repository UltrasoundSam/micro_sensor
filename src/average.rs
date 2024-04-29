use heapless::Vec;
use lsm303agr::{Acceleration, MagneticField};

pub struct SimpleMovingAverage {
    num_aves: u8,
    acc_x: Vec<i32, 255>,
    acc_y: Vec<i32, 255>,
    acc_z: Vec<i32, 255>,
    mag_x: Vec<i32, 255>,
    mag_y: Vec<i32, 255>,
    mag_z: Vec<i32, 255>,
    temp: Vec<f64, 255>,
}

impl SimpleMovingAverage {
    pub fn new(size: u8) -> SimpleMovingAverage {
        SimpleMovingAverage {
            num_aves: size,
            acc_x: Vec::new(),
            acc_y: Vec::new(),
            acc_z: Vec::new(),
            mag_x: Vec::new(),
            mag_y: Vec::new(),
            mag_z: Vec::new(),
            temp: Vec::new(),
        }
    }

    pub fn add_acceleration(&mut self, accel: Acceleration) {
        // Get values in milli-g
        let values = accel.xyz_mg();

        // Push values onto vec
        self.acc_x.push(values.0).unwrap();
        self.acc_y.push(values.1).unwrap();
        self.acc_z.push(values.2).unwrap();

        // Check to see if we have exceeded number of averages
        if self.acc_x.len() as u8 > self.num_aves {
            self.acc_x.remove(0);
            self.acc_y.remove(0);
            self.acc_z.remove(0);
        }
    }

    pub fn add_magnetic(&mut self, magnet: MagneticField) {
        // Get values in nanoTesla
        let values = magnet.xyz_nt();

        // Push values onto vec
        self.mag_x.push(values.0).unwrap();
        self.mag_y.push(values.1).unwrap();
        self.mag_z.push(values.2).unwrap();

        // Check to see if we have exceeded number of averages
        if self.mag_x.len() as u8 > self.num_aves {
            self.mag_x.remove(0);
            self.mag_y.remove(0);
            self.mag_z.remove(0);
        }
    }

    pub fn add_temp(&mut self, temp_reading: f64) {
        // Push values onto vec
        self.temp.push(temp_reading).unwrap();

        if self.temp.len() as u8 > self.num_aves {
            self.temp.remove(0);
        }
    }

    pub fn update_size(&mut self, new_size: u8) {
        self.num_aves = new_size;
    }

    pub fn get_num_aves(&self) -> u8 {
        self.num_aves
    }

    pub fn get_acc_average(&self) -> (f64, f64, f64) {
        let sum_x = self.acc_x.iter().fold(0, |acc, x| acc+x);
        let sum_y = self.acc_y.iter().fold(0, |acc, x| acc+x);
        let sum_z = self.acc_z.iter().fold(0, |acc, x| acc+x);

        let num_elems = self.acc_x.len() as f64;
        let ave_x = sum_x as f64 / num_elems;
        let ave_y = sum_y as f64 / num_elems;
        let ave_z = sum_z as f64 / num_elems;
        let result = (ave_x, ave_y, ave_z);
        result
    }

    pub fn get_mag_average(&self) -> (f64, f64, f64) {
        let sum_x = self.mag_x.iter().fold(0, |acc, x| acc+x);
        let sum_y = self.mag_y.iter().fold(0, |acc, x| acc+x);
        let sum_z = self.mag_z.iter().fold(0, |acc, x| acc+x);

        let num_elems = self.mag_x.len() as f64;
        let ave_x = sum_x as f64 / num_elems;
        let ave_y = sum_y as f64 / num_elems;
        let ave_z = sum_z as f64 / num_elems;
        let result = (ave_x, ave_y, ave_z);
        result
    }

    pub fn get_temp_average(&self) -> f64 {
        let sum_temp = self.temp.iter().fold(0., |acc, x| acc+x);
        let num_elems = self.temp.len() as f64;
        let result = sum_temp / num_elems;
        result
    }
}
