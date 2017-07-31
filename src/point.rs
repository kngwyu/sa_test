use std::ops::*;
use std::cmp::*;
// 本当はgenericsを使ったほうがいいけど面倒なのでtypedefしている
type Real = f64;
#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: Real,
    pub y: Real,
}
impl Point {
    pub fn new(x: Real, y: Real) -> Point {
        Point { x: x, y: y }
    }
    pub fn dot(&self, other: &Point) -> Real {
        self.x * other.x + self.y * other.y
    }
    pub fn norm(&self) -> Real {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
pub fn dist(p: Point, q: Point) -> Real {
    (p - q).norm()
}
impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Point) -> Ordering {
        let xcmp = self.x.partial_cmp(&other.x).unwrap();
        match xcmp {
            Ordering::Equal => self.y.partial_cmp(&other.y).unwrap(),
            _ => xcmp,
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for Point {}
