use failure;
use std;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Fail, Debug)]
#[fail(display = "NanoBot error")]
pub struct NanoBotError;

pub trait OkOrErr<T> {
    fn ok_or_err(self) -> Result<T>;
}

#[derive(Fail, Debug)]
#[fail(display = "My error")]
pub struct MyNoneError;

impl<T> OkOrErr<T> for Option<T> {
    fn ok_or_err(self) -> Result<T> {
        self.ok_or_else(|| MyNoneError.into())
    }
}

#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Cord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Cord {
    pub fn new(x: i32, y: i32, z: i32) -> Cord {
        Cord { x, y, z }
    }

    pub fn is_in_range(&self, r: usize) -> bool {
        0 <= self.x
            && (self.x as usize) < r
            && 0 <= self.y
            && (self.y as usize) < r
            && 0 <= self.z
            && (self.z as usize) < r
    }

    pub fn to_linear_index(&self, r: usize) -> usize {
        r * r * self.x as usize + r * self.y as usize + self.z as usize
    }

    pub fn is_origin(&self) -> bool {
        self.x == 0 && self.y == 0 && self.z == 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CordDiff {
    pub dx: i32,
    pub dy: i32,
    pub dz: i32,
}

impl std::ops::Add<CordDiff> for Cord {
    type Output = Self;
    fn add(self, rhs: CordDiff) -> Self {
        Cord::new(self.x + rhs.dx, self.y + rhs.dy, self.z + rhs.dz)
    }
}

impl std::ops::Sub<Cord> for Cord {
    type Output = CordDiff;
    fn sub(self, rhs: Cord) -> CordDiff {
        CordDiff::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl CordDiff {
    pub fn new(dx: i32, dy: i32, dz: i32) -> CordDiff {
        CordDiff { dx, dy, dz }
    }

    pub fn mlen(&self) -> u64 {
        (self.dx.abs() + self.dy.abs() + self.dz.abs()) as u64
    }

    pub fn clen(&self) -> u64 {
        self.dx.abs().max(self.dy.abs()).max(self.dz.abs()) as u64
    }

    pub fn is_linear(&self) -> bool {
        self.dx != 0 && self.dy == 0 && self.dz == 0
            || self.dx == 0 && self.dy != 0 && self.dz == 0
            || self.dx == 0 && self.dy == 0 && self.dz != 0
    }

    pub fn is_short_linear(&self) -> bool {
        self.is_linear() && self.mlen() <= 5
    }

    pub fn is_long_linear(&self) -> bool {
        self.is_linear() && self.mlen() <= 15
    }

    pub fn is_near(&self) -> bool {
        self.mlen() <= 2 && self.clen() == 1
    }

    pub fn direc(&self) -> CordDiff {
        debug_assert!(self.is_linear());
        if self.dx < 0 {
            CordDiff::new(-1, 0, 0)
        } else if self.dx > 0 {
            CordDiff::new(1, 0, 0)
        } else if self.dy < 0 {
            CordDiff::new(0, -1, 0)
        } else if self.dy > 0 {
            CordDiff::new(0, 1, 0)
        } else if self.dz < 0 {
            CordDiff::new(0, 0, -1)
        } else if self.dz > 0 {
            CordDiff::new(0, 0, 1)
        } else {
            unreachable!()
        }
    }

    pub fn gen_all_diff() -> &'static [CordDiff] {
        lazy_static! {
            static ref DIFFS: Vec<CordDiff> = {
                let mut a = vec![];
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            let d = CordDiff::new(dx, dy, dz);
                            if d.mlen() == 1 {
                                a.push(d);
                            }
                        }
                    }
                }
                a
            };
        }
        &DIFFS[..]
    }

    pub fn gen_all_near_diff() -> &'static [CordDiff] {
        lazy_static! {
            static ref DIFFS: Vec<CordDiff> = {
                let mut a = vec![];
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            let d = CordDiff::new(dx, dy, dz);
                            if d.is_near() {
                                a.push(d);
                            }
                        }
                    }
                }
                a
            };
        }
        &DIFFS[..]
    }

    #[allow(dead_code)]
    pub fn gen_all_long_liner() -> &'static [LongLinear] {
        lazy_static! {
            static ref LLDS: Vec<LongLinear> = {
                let mut a = vec![];
                for d in -15..=15 {
                    if d == 0 {
                        continue;
                    }
                    a.push(LongLinear(CordDiff::new(d, 0, 0)));
                    a.push(LongLinear(CordDiff::new(0, d, 0)));
                    a.push(LongLinear(CordDiff::new(0, 0, d)));
                }
                a
            };
        }
        &LLDS[..]
    }

    #[allow(dead_code)]
    pub fn gen_all_short_liner() -> &'static [ShortLinear] {
        lazy_static! {
            static ref SLDS: Vec<ShortLinear> = {
                let mut a = vec![];
                for d in -5..=5 {
                    if d == 0 {
                        continue;
                    }
                    a.push(ShortLinear(CordDiff::new(d, 0, 0)));
                    a.push(ShortLinear(CordDiff::new(0, d, 0)));
                    a.push(ShortLinear(CordDiff::new(0, 0, d)));
                }
                a
            };
        }
        &SLDS[..]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LongLinear(pub CordDiff);
#[derive(Copy, Clone, Debug)]
pub struct ShortLinear(pub CordDiff);
#[derive(Copy, Clone, Debug)]
pub struct Near(pub CordDiff);

// Details > Regions

#[derive(PartialEq, Eq, Debug)]
pub struct Region {
    pub xs: Range,
    pub ys: Range,
    pub zs: Range,
}

impl Region {
    pub fn new(c1: Cord, c2: Cord) -> Region {
        Region {
            xs: Range::new(c1.x, c2.x),
            ys: Range::new(c1.y, c2.y),
            zs: Range::new(c1.z, c2.z),
        }
    }

    pub fn all_cords(&self) -> Vec<Cord> {
        let mut cords = vec![];
        for x in self.xs.min..=self.xs.max {
            for y in self.ys.min..=self.ys.max {
                for z in self.zs.min..=self.zs.max {
                    cords.push(Cord::new(x, y, z));
                }
            }
        }
        cords
    }

    // pub fn dimension(&self) -> u32 {
    //     unimplemented!()
    // }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Range {
    min: i32,
    max: i32,
}

impl Range {
    pub fn new(a: i32, b: i32) -> Range {
        Range {
            min: a.min(b),
            max: a.max(b),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cord_test() {
        let d000 = CordDiff::new(0, 0, 0);
        let d001 = CordDiff::new(0, 0, 1);
        let d011 = CordDiff::new(0, 1, 1);
        let d111 = CordDiff::new(1, 1, 1);
        let d123 = CordDiff::new(1, 2, 3);
        let d002 = CordDiff::new(0, 0, 2);
        let d009 = CordDiff::new(0, 0, 9);

        assert_eq!(d000.mlen(), 0);
        assert_eq!(d001.mlen(), 1);
        assert_eq!(d011.mlen(), 2);
        assert_eq!(d111.mlen(), 3);
        assert_eq!(d123.mlen(), 6);

        assert_eq!(d000.clen(), 0);
        assert_eq!(d001.clen(), 1);
        assert_eq!(d011.clen(), 1);
        assert_eq!(d111.clen(), 1);
        assert_eq!(d123.clen(), 3);

        assert!(!d000.is_linear());
        assert!(d001.is_linear());
        assert!(!d011.is_linear());

        assert!(d001.is_short_linear());
        assert!(!d009.is_short_linear());

        assert!(d001.is_long_linear());
        assert!(d009.is_long_linear());

        assert!(d001.is_near());
        assert!(d011.is_near());
        assert!(!d002.is_near());
        assert!(!d111.is_near());
        assert!(!d009.is_near());
    }

}
