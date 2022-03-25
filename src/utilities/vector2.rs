use num::{Float, Num};
use std::{
    borrow::Borrow,
    ops::{Add, AddAssign, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2<T>
where
    T: Num + Copy + MulAssign + Borrow<T>,
{
    #[allow(dead_code)]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
    #[allow(dead_code)]
    pub fn to_array(self) -> [T; 2] {
        [self.x, self.y]
    }
    #[allow(dead_code)]
    pub fn multiply_scalar(&mut self, scalar: T) {
        self.x *= scalar;
        self.y *= scalar;
    }
    #[allow(dead_code)]
    pub fn dot(v1: Self, v2: Self) -> T {
        v1.x * v2.x + v1.y * v2.y
    }

    #[allow(dead_code)]
    pub fn angle(&self) -> T
    where
        T: Num + Float,
    {
        (self.y).atan2(self.x)
    }

    #[allow(dead_code)]
    pub fn rotate(v: Self, theta: T) -> Self
    where
        T: Float + Num,
    {
        let cos = theta.cos();
        let sin = theta.sin();
        Self {
            x: v.x * cos - v.y * sin,
            y: v.x * sin + v.y * cos,
        }
    }
}

impl<T> Vector2<T>
where
    T: Num + Float + DivAssign + MulAssign,
{
    #[allow(dead_code)]
    pub fn magnitude(&self) -> T {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    #[allow(dead_code)]
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag.is_zero() {
            return;
        }
        self.x /= mag;
        self.y /= mag;
    }
    #[allow(dead_code)]
    pub fn limit(&mut self, max: T) {
        if self.magnitude() > max {
            self.normalize();
            *self *= max;
        }
    }
}

impl<T> Mul for Vector2<T>
where
    T: Num,
{
    type Output = Vector2<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T> Add for Vector2<T>
where
    T: Num,
{
    type Output = Vector2<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub for Vector2<T>
where
    T: Num,
{
    type Output = Vector2<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> AddAssign for Vector2<T>
where
    T: Num + AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T> SubAssign for Vector2<T>
where
    T: Num + SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T> Mul<T> for Vector2<T>
where
    T: Num + Copy,
{
    type Output = Vector2<T>;
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> MulAssign<T> for Vector2<T>
where
    T: Num + MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn limit_vectors() {
        let mut velocity = Vector2::new(4.0, 3.0);
        let acceleration = Vector2::new(1.0, 1.0);
        velocity += acceleration;
        velocity.limit(5.0);
        assert_eq!(5.0, velocity.magnitude());
    }

    #[test]

    fn try_dot_p() {
        let v1 = Vector2::new(1.0, 3.0);
        let v2 = Vector2::new(2.0, 4.0);
        let res = 14.0;
        assert_eq!(Vector2::dot(v1, v2), res);
    }

    #[test]

    fn try_rotation() {
        let v1 = Vector2::new(1.0f32, 1.0);
        let theta = 45.0 * 3.14159265358979 / 180.0;
        let res = Vector2::new(0.0f32, 2.0.sqrt());
        assert_eq!(Vector2::rotate(v1, theta), res);
    }
}
