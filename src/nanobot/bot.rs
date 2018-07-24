// use std;

use super::prelude::*;

pub type BotId = u64;

#[derive(Clone, Copy, Debug)]
pub enum Cmd {
    Halt,
    Wait,
    Flip,
    SMove(LongLinear),
    LMove(ShortLinear, ShortLinear),
    Fission(Near, usize),
    Fill(Near),
    Void(Near),
    FusionP(Near),
    FusionS(Near),
}

impl From<Cmd> for Vec<u8> {
    fn from(cmd: Cmd) -> Vec<u8> {
        use self::Cmd::*;
        match cmd {
            Halt => vec![0b_1111_1111],
            Wait => vec![0b_1111_1110],
            Flip => vec![0b_1111_1101],
            SMove(lld) => {
                let CordDiffBits { a, i } = lld.encode();
                vec![(a << 4) | 0b_0000_0100, i]
            }
            LMove(sld1, sld2) => {
                let sld1 = sld1.encode();
                let sld2 = sld2.encode();
                vec![
                    (sld2.a << 6) | (sld1.a << 4) | 0b_0000_1100,
                    (sld2.i << 4) | sld1.i,
                ]
            }
            Fission(nd, m) => {
                let nd = nd.encode();
                vec![(nd << 3) | 0b_0000_0101, m as u8]
            }
            FusionP(nd) => {
                let nd = nd.encode();
                vec![(nd << 3) | 0b_0000_0111]
            }
            FusionS(nd) => {
                let nd = nd.encode();
                vec![(nd << 3) | 0b_0000_0110]
            }
            Fill(nd) => {
                let nd = nd.encode();
                vec![(nd << 3) | 0b_0000_0011]
            }
            Void(nd) => {
                let nd = nd.encode();
                vec![(nd << 3) | 0b_0000_0010]
            }
        }
    }
}

pub struct CordDiffBits {
    pub a: u8,
    pub i: u8,
}

impl ShortLinear {
    pub fn encode(&self) -> CordDiffBits {
        let cord = &self.0;
        if cord.dx != 0 {
            CordDiffBits {
                a: 0b_01,
                i: (cord.dx + 5) as u8,
            }
        } else if cord.dy != 0 {
            CordDiffBits {
                a: 0b_10,
                i: (cord.dy + 5) as u8,
            }
        } else {
            debug_assert!(cord.dz != 0);
            CordDiffBits {
                a: 0b_11,
                i: (cord.dz + 5) as u8,
            }
        }
    }
}

impl LongLinear {
    pub fn encode(&self) -> CordDiffBits {
        let cord = &self.0;
        if cord.dx != 0 {
            CordDiffBits {
                a: 0b_01,
                i: (cord.dx + 15) as u8,
            }
        } else if cord.dy != 0 {
            CordDiffBits {
                a: 0b_10,
                i: (cord.dy + 15) as u8,
            }
        } else {
            debug_assert!(cord.dz != 0);
            CordDiffBits {
                a: 0b_11,
                i: (cord.dz + 15) as u8,
            }
        }
    }
}

impl Near {
    pub fn encode(&self) -> u8 {
        // 5 bits
        let c = &self.0;
        ((c.dx + 1) * 9 + (c.dy + 1) * 3 + (c.dz + 1)) as u8
    }
}

#[derive(Debug, Clone)]
pub struct Bot {
    pub bid: BotId,
    pub pos: Cord,
    pub seeds: Vec<BotId>,
}

impl Bot {
    pub fn new_at_origin() -> Bot {
        Bot {
            bid: 1,
            pos: Cord::new(0, 0, 0),
            seeds: (2..=40).collect(),
        }
    }

    #[allow(dead_code)]
    fn new(bid: BotId, pos: Cord, seeds: Vec<BotId>) -> Bot {
        Bot { bid, pos, seeds }
    }

    #[allow(dead_code)]
    pub fn fission(&mut self, nd: Near, m: usize) -> Bot {
        assert!(!self.seeds.is_empty());
        assert!(m < self.seeds.len());

        let new_bot = Bot::new(
            self.seeds[0],
            self.pos + nd.0,
            self.seeds[1..(m + 1)].to_vec(),
        );
        self.seeds = self.seeds[(m + 1)..].to_vec();
        new_bot
    }

    #[allow(dead_code)]
    pub fn fusion(&mut self, other: &Bot) {
        self.seeds.push(other.bid);
        self.seeds.extend(other.seeds.iter());
        self.seeds.sort();
    }

    #[allow(dead_code)]
    fn is_valid_pos(&self) -> bool {
        // debug_assert!()
        true
    }

    #[allow(dead_code)]
    pub fn smove(&mut self, d: CordDiff) {
        self.pos = self.pos + d;
        self.is_valid_pos();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cmd_encode_test() {
        use self::Cmd::*;

        let bytes: Vec<u8> = Halt.into();
        assert_eq!(bytes, vec![255 as u8]);

        assert_eq!(Vec::<u8>::from(Halt), vec![255 as u8]);

        assert_eq!(
            Vec::<u8>::from(SMove(LongLinear(CordDiff::new(12, 0, 0)))),
            vec![0b_00010100, 0b_00011011]
        );
        assert_eq!(
            Vec::<u8>::from(SMove(LongLinear(CordDiff::new(0, 0, -4)))),
            vec![0b_00110100, 0b_00001011]
        );

        assert_eq!(
            Vec::<u8>::from(LMove(
                ShortLinear(CordDiff::new(3, 0, 0)),
                ShortLinear(CordDiff::new(0, -5, 0))
            )),
            vec![0b_10011100, 0b_00001000]
        );
        assert_eq!(
            Vec::<u8>::from(LMove(
                ShortLinear(CordDiff::new(0, -2, 0)),
                ShortLinear(CordDiff::new(0, 0, 2))
            )),
            vec![0b_11101100, 0b_01110011]
        );

        assert_eq!(
            Vec::<u8>::from(Fission(Near(CordDiff::new(0, 0, 1)), 5)),
            vec![0b_01110101, 0b_00000101]
        );

        assert_eq!(
            Vec::<u8>::from(FusionP(Near(CordDiff::new(-1, 1, 0)))),
            vec![0b_00111111]
        );

        assert_eq!(
            Vec::<u8>::from(FusionS(Near(CordDiff::new(1, -1, 0)))),
            vec![0b_10011110]
        );

        assert_eq!(
            Vec::<u8>::from(Fill(Near(CordDiff::new(0, -1, 0)))),
            vec![0b_01010011]
        );

        assert_eq!(
            Vec::<u8>::from(Void(Near(CordDiff::new(1, 0, 1)))),
            vec![0b_10111010]
        );
    }

}
