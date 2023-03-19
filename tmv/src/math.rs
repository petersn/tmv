use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Vec2(pub f32, pub f32);

impl Vec2 {
  pub fn length(self) -> f32 {
    (self.0 * self.0 + self.1 * self.1).sqrt()
  }

  pub fn to_unit(self) -> Self {
    let c = 1.0 / self.length();
    Self(c * self.0, c * self.1)
  }

  pub fn cardinal_direction(dir: usize) -> Self {
    match dir {
      0 => Self(-1.0, 0.0),
      1 => Self(0.0, 1.0),
      2 => Self(1.0, 0.0),
      3 => Self(0.0, -1.0),
      _ => unreachable!(),
    }
  }
}

impl Default for Vec2 {
  fn default() -> Self {
    Self(0.0, 0.0)
  }
}

impl std::ops::Add for Vec2 {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self(self.0 + rhs.0, self.1 + rhs.1)
  }
}

impl std::ops::AddAssign for Vec2 {
  fn add_assign(&mut self, rhs: Self) {
    self.0 += rhs.0;
    self.1 += rhs.1;
  }
}

impl std::ops::Sub for Vec2 {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    Self(self.0 - rhs.0, self.1 - rhs.1)
  }
}

impl std::ops::SubAssign for Vec2 {
  fn sub_assign(&mut self, rhs: Self) {
    self.0 -= rhs.0;
    self.1 -= rhs.1;
  }
}

impl std::ops::Mul<f32> for Vec2 {
  type Output = Self;

  fn mul(self, rhs: f32) -> Self::Output {
    Self(self.0 * rhs, self.1 * rhs)
  }
}

impl std::ops::MulAssign<f32> for Vec2 {
  fn mul_assign(&mut self, rhs: f32) {
    self.0 *= rhs;
    self.1 *= rhs;
  }
}

impl std::ops::Mul<Vec2> for f32 {
  type Output = Vec2;

  fn mul(self, rhs: Vec2) -> Self::Output {
    Vec2(self * rhs.0, self * rhs.1)
  }
}

impl std::ops::Div<f32> for Vec2 {
  type Output = Self;

  fn div(self, rhs: f32) -> Self::Output {
    Self(self.0 / rhs, self.1 / rhs)
  }
}

impl std::ops::DivAssign<f32> for Vec2 {
  fn div_assign(&mut self, rhs: f32) {
    self.0 /= rhs;
    self.1 /= rhs;
  }
}

impl std::ops::Neg for Vec2 {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self(-self.0, -self.1)
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Rect {
  pub pos:  Vec2,
  pub size: Vec2,
}

impl Rect {
  pub fn new(pos: Vec2, size: Vec2) -> Self {
    Self { pos, size }
  }

  pub fn contains_point(self, p: Vec2) -> bool {
    p.0 >= self.pos.0
      && p.0 < self.pos.0 + self.size.0
      && p.1 >= self.pos.1
      && p.1 < self.pos.1 + self.size.1
  }

  pub fn contains_rect(self, r: Rect) -> bool {
    // FIXME: Simplify this
    self.contains_point(r.pos)
      && self.contains_point(r.pos + Vec2(r.size.0, 0.0))
      && self.contains_point(r.pos + Vec2(0.0, r.size.1))
      && self.contains_point(r.pos + r.size)
  }
}
