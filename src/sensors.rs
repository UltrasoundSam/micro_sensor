use heapless::Vec;
use lsm303agr::{Acceleration, MagneticField};

const MAX_SIZE: usize = 64;

pub struct DataStore {
    pub num_aves: u8,
    acc: Vec<[i32; 3], MAX_SIZE>,
    mag: Vec<[i32; 3], MAX_SIZE>,
    temp: Vec<f64, MAX_SIZE>,
}

impl DataStore {
    pub fn new(size: u8) -> DataStore {
        DataStore {
            num_aves: size,
            acc: Vec::new(),
            mag: Vec::new(),
            temp: Vec::new()
        }
    }

    fn add_vecdata(
        &self,
        data: [i32; 3],
        mut vect: Vec<[i32; 3], MAX_SIZE>
    ) ->  Vec<[i32; 3], MAX_SIZE> {
        // Check to see whether vector is too long
        if vect.len() as u8 >= self.num_aves {
            vect.remove(0);
        }
        // Add data to vector
        vect.push(data).unwrap();
        vect
    }

    pub fn add_magnetic(&mut self, magnet_data: MagneticField) {
        let data = [magnet_data.x_nt(),magnet_data.y_nt(), magnet_data.z_nt()];
        self.mag = self.add_vecdata(data, self.mag.clone());
    }

    pub fn add_accel(&mut self, accel_data: Acceleration) {
        let data = [accel_data.x_mg(), accel_data.y_mg(), accel_data.z_mg()];
        self.acc = self.add_vecdata(data, self.acc.clone());
    }

    pub fn add_temp(&mut self, temp_data: f64) {
        // Check to see whether we will exceed number of averages
        if self.temp.len() as u8 >= self.num_aves {
            self.temp.remove(0);
        }

        self.temp.push(temp_data).unwrap();
    }

    pub fn get_averages(&self) -> [f64; 7] {
        // Get value sums
        let accel_sum = sum_vecs(&self.acc);
        let mag_sum = sum_vecs(&self.mag);
        let temp_sum: f64 = self.temp.iter().sum();

        // How long are vects
        let len_acc = self.acc.len() as f64;
        let len_mag = self.mag.len() as f64;
        let len_temp = self.temp.len() as f64;

        // Create array for data
        let mut aves = [0.; 7];
        for n in 0..3 {
            aves[n] = accel_sum[n] as f64 / len_acc;
            aves[n+3] = mag_sum[n] as f64 / len_mag;
        }
        aves[6] = temp_sum / len_temp;
        aves
    }
}

fn sum_vecs(vect: &Vec<[i32; 3], MAX_SIZE>) -> [i32; 3] {
    //
    let mut sum = [0, 0, 0];
    for meas in vect.iter() {
        for (n, val) in meas.iter().enumerate() {
            sum[n] += val;
        }
    }
    sum
}