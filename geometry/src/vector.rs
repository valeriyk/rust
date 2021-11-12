use crate::Point3d;

#[derive(Copy, Clone)]
pub struct Vector3d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3d {
    pub fn new() -> Vector3d {
        Vector3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    pub fn from_array(a: &[f32; 3]) -> Vector3d {
        Vector3d {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }
    pub fn from_coords(x: f32, y: f32, z: f32) -> Vector3d {
        Vector3d { x, y, z }
    }
    pub fn from_points(start: Point3d, end: Point3d) -> Vector3d {
        end - start
    }

    pub fn len(&self) -> f32 {
        (*self * *self).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let length_inverted = 1.0 / self.len();
        Self {
            x: self.x * length_inverted,
            y: self.y * length_inverted,
            z: self.z * length_inverted,
        }
    }

    /// Cross product
    pub fn crossprod(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl core::ops::Add<Vector3d> for Vector3d {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl core::ops::Sub<Vector3d> for Vector3d {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl core::ops::Add<Point3d> for Vector3d {
    type Output = Point3d;

    fn add(self, other: Point3d) -> Self::Output {
        Self::Output {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

/// Dot product
impl core::ops::Mul<Vector3d> for Vector3d {
    type Output = f32;

    #[inline]
    fn mul(self, other: Self) -> Self::Output {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl core::ops::Mul<f32> for Vector3d {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl core::ops::Div<f32> for Vector3d {
    type Output = Self;

    fn div(self, other: f32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl core::ops::Neg for Vector3d {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

/*pub struct IterVector3d {
    vec: Vector3d,
    item_idx: usize,
}

impl Iterator for IterVector3d {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.item_idx {
            0 => self.vec.x,
            1 => self.vec.y,
            2 => self.vec.z,
            3 => self.vec.w,
            _ => return None,
        };
        self.item_idx += 1;
        Some(result)
    }
}
impl IntoIterator for Vector3d {
    type Item = f32;
    type IntoIter = IterVector3d;
    fn into_iter(self) -> Self::IntoIter {
        IterVector3d {
            vec: self,
            item_idx: 0,
        }
    }
}
*/

/*/// Point + Vector = Point; Vector + Vector = Vector,
fn add_vector<T>(this: T, other: &[f32]) -> T
    where
        T: IntoIterator + FromIterator<<<T as IntoIterator>::Item as Add<f32>>::Output>,
        T::Item: Add<f32>
{
    this.into_iter().zip(other).map(|x| x.0 + *x.1).collect()
}

/// Point + Point = Vector; Vector + Point = Vector,
fn add_point<T>(this: T, other: &[f32]) -> [f32; 4]
    where
        T: IntoIterator + FromIterator<<<T as IntoIterator>::Item as Add<f32>>::Output>,
        T::Item: Add<f32>
{
    this.into_iter().zip(other).map(|x| x.0 + *x.1).collect()
}

fn add_any<T>(this: &[f32], other: &[f32]) -> T
    where
         T: FromIterator<f32>
    //     T::Item: Add<f32>
{
    this.into_iter().zip(other).take(3).map(|x| x.0 + *x.1).collect()
}

impl core::ops::Add<Point3d_2> for Point3d_2 {
    type Output = Vector3d_2;

    fn add(self, other: Self) -> Self::Output {
        add_any::<Self::Output>(&self.v, &other.v)
    }
}

impl core::ops::Add<Vector3d_2> for Point3d_2 {
    type Output = Point3d_2;

    fn add(self, other: Vector3d_2) -> Self::Output {
        add_any::<Self::Output>(&self.v, &other.v)
    }
}*/
