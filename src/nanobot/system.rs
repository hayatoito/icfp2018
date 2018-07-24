use chrono::*;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::rc::Rc;

use super::bot::*;
use super::matrix::*;
use super::model::*;
use super::prelude::*;
use super::target::*;
use super::trace::*;

#[derive(Debug)]
struct Volatile {
    r: usize,
    flip: bool,
    interfared_cords: Vec<bool>,
    new_bots: Vec<Bot>,
    removed_bots: Vec<BotId>,
}

impl Volatile {
    fn new(r: usize, bots: &[Bot]) -> Volatile {
        let mut v = Volatile {
            r,
            flip: Default::default(),
            interfared_cords: vec![false; r * r * r],
            new_bots: vec![],
            removed_bots: vec![],
        };
        for b in bots {
            v.add_cord(b.pos);
        }
        v
    }

    fn is_interfared(&self, c: &Cord) -> bool {
        self.interfared_cords[c.to_linear_index(self.r)]
    }

    fn flip(&mut self) {
        self.flip = !self.flip;
    }

    fn add_region(&mut self, region: &Region) {
        for cord in region.all_cords() {
            self.add_cord(cord);
        }
    }

    fn add_cord(&mut self, cord: Cord) {
        self.interfared_cords[cord.to_linear_index(self.r)] = true;
    }

    fn smove(&mut self, region: &Region) {
        self.add_region(&region);
    }

    fn lmove(&mut self, region1: &Region, region2: &Region) {
        self.add_region(&region1);
        self.add_region(&region2);
    }

    fn fussion(&mut self, new_bot: Bot) {
        self.add_cord(new_bot.pos);
        self.new_bots.push(new_bot);
    }

    fn fill(&mut self, pos: Cord) {
        self.add_cord(pos);
    }

    fn void(&mut self, pos: Cord) {
        self.add_cord(pos);
    }

    fn bot_removed(&mut self, bid: BotId) {
        self.removed_bots.push(bid);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CmdResult {
    Interfared,
    Continue,
    Halt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveStep {
    pub len: usize,
    pub c: Cord,
    pub prev: Option<Rc<MoveStep>>,
}

pub struct MoveCmds {
    pub cmds: Vec<Cmd>,
}

impl From<Rc<MoveStep>> for MoveCmds {
    fn from(step: Rc<MoveStep>) -> MoveCmds {
        let mut moves: Vec<Cmd> = vec![];
        let mut step = step;
        while let Some(prev) = step.clone().prev.as_ref() {
            moves.push(Cmd::SMove(LongLinear(step.c - prev.c)));
            step = prev.clone();
        }
        moves.reverse();
        MoveCmds::new(moves)
    }
}

impl MoveCmds {
    fn new(cmds: Vec<Cmd>) -> MoveCmds {
        MoveCmds {
            cmds: MoveCmds::compress(cmds),
        }
    }

    fn compress(cmd: Vec<Cmd>) -> Vec<Cmd> {
        MoveCmds::compress_lmove(MoveCmds::compress_smove(cmd))
    }

    fn compress_smove(cmds: Vec<Cmd>) -> Vec<Cmd> {
        use self::Cmd::*;
        if cmds.is_empty() {
            return cmds;
        }
        let mut res = vec![];
        let mut prev = cmds[0];
        for cmd in cmds[1..].into_iter().cloned() {
            match (prev, cmd) {
                (SMove(lld1), SMove(lld2)) => {
                    let total = CordDiff::new(
                        lld1.0.dx + lld2.0.dx,
                        lld1.0.dy + lld2.0.dy,
                        lld1.0.dz + lld2.0.dz,
                    );
                    if total.is_linear() && total.mlen() <= 15 {
                        prev = SMove(LongLinear(total));
                    } else {
                        res.push(SMove(lld1));
                        prev = SMove(lld2);
                    }
                }
                (a, b) => {
                    res.push(a);
                    prev = b;
                }
            }
        }
        res.push(prev);
        res
    }

    fn compress_lmove(cmds: Vec<Cmd>) -> Vec<Cmd> {
        use self::Cmd::*;
        if cmds.is_empty() {
            return cmds;
        }
        let mut res = vec![];
        let mut prev = cmds[0];
        for cmd in cmds[1..].into_iter().cloned() {
            match (prev, cmd) {
                (SMove(lld1), SMove(lld2)) => {
                    let total = CordDiff::new(
                        lld1.0.dx + lld2.0.dx,
                        lld1.0.dy + lld2.0.dy,
                        lld1.0.dz + lld2.0.dz,
                    );
                    if !total.is_linear() && lld1.0.mlen() <= 5 && lld2.0.mlen() <= 5 {
                        prev = LMove(ShortLinear(lld1.0), ShortLinear(lld2.0));
                    } else {
                        res.push(SMove(lld1));
                        prev = SMove(lld2);
                    }
                }
                (a, b) => {
                    res.push(a);
                    prev = b;
                }
            }
        }
        res.push(prev);
        res
    }
}

pub struct MoveToNear {
    pub move_cmds: MoveCmds,
    pub final_pos: Cord,
    pub target_nd: Near,
    pub target: Cord,
}

impl From<Rc<MoveStep>> for MoveToNear {
    fn from(step: Rc<MoveStep>) -> MoveToNear {
        let target = step.c;
        assert!(step.prev.is_some());
        let bot_final = step.prev.as_ref().unwrap();
        let move_cmds: MoveCmds = bot_final.clone().into();
        MoveToNear {
            move_cmds,
            final_pos: bot_final.c,
            target_nd: Near(target - bot_final.c),
            target,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Harmonics {
    Low,
    High,
}

pub struct System {
    pub model_id: ModelId,
    pub energy: i64,
    pub harmonics: Harmonics,
    pub matrix: Matrix,
    pub priority_targets: PriorityTargets,
    pub bots: Vec<Bot>,
    pub bot_index: usize,
    volatile: Volatile,
    reserved_fusion: HashMap<Cord, Cord>, // secondary -> primary
    pub records: Vec<Cmd>,
    start_datetime: DateTime<Local>,
}

impl System {
    pub fn new(model: &Model) -> System {
        let bots = vec![Bot::new_at_origin()];
        let volatile = Volatile::new(model.r, &bots);
        System {
            model_id: model.id,
            energy: 0,
            harmonics: Harmonics::Low,
            matrix: Matrix::new(&model),
            priority_targets: PriorityTargets::new(&model),
            bots,
            bot_index: 0,
            volatile,
            reserved_fusion: HashMap::new(),
            records: vec![],
            start_datetime: Local::now(),
        }
    }

    pub fn current_bot(&self) -> &Bot {
        &self.bots[self.bot_index]
    }

    pub fn free_priority_targets(&self) -> HashSet<Cord> {
        let targets: BTreeSet<_> = self.priority_targets
            .priority_targets
            .iter()
            .filter(|p| !self.volatile.is_interfared(&p.cord))
            .cloned()
            .collect();
        PriorityTargets::top_priority_targets(&targets)
    }

    pub fn is_interfared(&self, c: Cord) -> bool {
        self.matrix[c]
            // || self.bots.iter().any(|bot| bot.pos == c)
            || self.volatile.is_interfared(&c)
    }

    pub fn find_current_bot_fusion_opponent(&self) -> Option<&Bot> {
        let pos = self.current_bot().pos;
        self.bots[self.bot_index + 1..]
            .iter()
            .find(|b| self.reserved_fusion.get(&b.pos).is_none() && (b.pos - pos).is_near())
    }

    pub fn is_current_bot_reserved_as_fusion_secondary(&self) -> Option<&Cord> {
        self.reserved_fusion.get(&self.current_bot().pos)
    }

    pub fn can_current_bot_fisson(&self) -> Option<Cmd> {
        if self.current_bot().seeds.is_empty() {
            None
        } else {
            let pos = self.current_bot().pos;
            for diff in CordDiff::gen_all_diff() {
                let c = pos + *diff;
                if !c.is_in_range(self.matrix.r) {
                    continue;
                }
                if !self.is_interfared(c) {
                    return Some(Cmd::Fission(
                        Near(c - pos),
                        self.current_bot().seeds.len() / 2,
                    ));
                }
            }
            None
        }
    }

    pub fn move_to_first_or_wait_cmd(&self, from: Cord, to: Cord) -> Cmd {
        let moves = self.move_to(from, to);
        moves.cmds.get(0).cloned().unwrap_or(Cmd::Wait)
    }

    pub fn move_to_near(&self, from: Cord, targets: &HashSet<Cord>) -> Result<MoveToNear> {
        let r = self.matrix.r;
        let mut q = VecDeque::new();
        q.push_back(Rc::new(MoveStep {
            len: 0,
            c: from,
            prev: None,
        }));

        let mut visited = HashSet::new();
        visited.insert(from);

        while let Some(current) = q.pop_front() {
            for diff in CordDiff::gen_all_diff() {
                let c = current.c + *diff;
                if !c.is_in_range(r) {
                    continue;
                }
                if !visited.contains(&c) && !self.is_interfared(c) {
                    visited.insert(c);
                    let next = Rc::new(MoveStep {
                        len: current.len + 1,
                        c,
                        prev: Some(current.clone()),
                    });
                    q.push_back(next);
                }
            }
            for diff in CordDiff::gen_all_near_diff() {
                let c = current.c + *diff;
                if !c.is_in_range(r) {
                    continue;
                }
                // For Void, self.is_interfared(c) can be true.
                // if !self.is_interfared(c) && targets.contains(&c) {
                if targets.contains(&c) {
                    let step = Rc::new(MoveStep {
                        len: current.len + 1,
                        c,
                        prev: Some(current.clone()),
                    });
                    return Ok(MoveToNear::from(step));
                }
            }
        }
        Err(NanoBotError.into())
    }

    pub fn move_to(&self, from: Cord, to: Cord) -> MoveCmds {
        let r = self.matrix.r;
        let mut q = VecDeque::new();
        q.push_back(Rc::new(MoveStep {
            len: 0,
            c: from,
            prev: None,
        }));

        let mut visited = HashSet::new();
        visited.insert(from);

        while let Some(current) = q.pop_front() {
            for diff in CordDiff::gen_all_diff() {
                let c = current.c + *diff;
                if !c.is_in_range(r) {
                    continue;
                }
                if !visited.contains(&c) && !self.is_interfared(c) {
                    visited.insert(c);
                    let next = Rc::new(MoveStep {
                        len: current.len + 1,
                        c,
                        prev: Some(current.clone()),
                    });
                    if c == to {
                        return next.into();
                    }
                    q.push_back(next);
                }
            }
        }
        MoveCmds { cmds: vec![] }
    }

    #[allow(dead_code)]
    fn simulate(&mut self, trace: Trace) -> Result<()> {
        for cmd in trace.cmds {
            match self.execute_cmd(cmd) {
                CmdResult::Halt => {
                    return Ok(());
                }
                CmdResult::Continue => {}
                CmdResult::Interfared => {
                    warn!("cmd is interfaced!");
                    return Err(NanoBotError.into());
                }
            }
        }
        Err(NanoBotError.into())
    }

    fn prepare_next_time_step(&mut self) {
        self.bot_index = 0;
        self.reserved_fusion.clear();

        let r = self.matrix.r;
        match self.harmonics {
            Harmonics::High => self.energy += (30 * r * r * r) as i64,
            Harmonics::Low => self.energy += (3 * r * r * r) as i64,
        }
        self.energy += (20 * self.bots.len()) as i64;

        self.apply_volatile();
        self.volatile = Volatile::new(self.matrix.r, &self.bots);
    }

    fn apply_volatile(&mut self) {
        if self.volatile.flip {
            self.harmonics = match self.harmonics {
                Harmonics::High => Harmonics::Low,
                Harmonics::Low => Harmonics::High,
            }
        }

        let mut bots = self.bots.clone();
        bots.extend(self.volatile.new_bots.clone());
        bots = bots.into_iter()
            .filter(|b| !self.volatile.removed_bots.contains(&b.bid))
            .collect();;
        bots.sort_by_key(|bot| bot.bid);
        self.bots = bots;
    }

    fn is_move_interfared(&self, start_excluding: Cord, diff: CordDiff) -> bool {
        let direc = diff.direc();
        let mlen = diff.clen();
        let c = start_excluding + direc;
        let mut cnt = 0;
        loop {
            if self.is_interfared(c) {
                return true;
            }
            cnt += 1;
            if cnt == mlen {
                return false;
            }
        }
    }

    pub fn move_to_target_and_fill_or_void(&mut self, targets: &HashSet<Cord>) -> Cmd {
        let origin = Cord::new(0, 0, 0);
        match self.move_to_near(self.current_bot().pos, targets) {
            Ok(MoveToNear {
                move_cmds,
                target_nd,
                target,
                ..
            }) => {
                if !move_cmds.cmds.is_empty() {
                    move_cmds.cmds[0]
                } else {
                    // TODO: Check this condition.
                    // if self.is_interfared(target) {
                    //     Cmd::Wait
                    debug_assert!(!self.volatile.is_interfared(&target));
                    match self.model_id {
                        ModelId::Assemble(_) => Cmd::Fill(target_nd),
                        ModelId::Disassemble(_) => Cmd::Void(target_nd),
                    }
                }
            }
            Err(_) => self.move_to_first_or_wait_cmd(self.current_bot().pos, origin),
        }
    }

    pub fn assert_execute_cmd(&mut self, cmd: Cmd) {
        assert_eq!(self.execute_cmd(cmd), CmdResult::Continue)
    }

    pub fn execute_cmd(&mut self, cmd: Cmd) -> CmdResult {
        debug!(
            "bot: id: {}, pos: {:?}, cmd: {:?}",
            self.current_bot().bid,
            self.current_bot().pos,
            cmd
        );
        if (self.records.len() + 1) % 10_000 == 0 {
            info!(
                "...solving model: {}, records.len: {}, start_datetime: {}, now: {}",
                self.model_id.name(),
                self.records.len(),
                self.start_datetime,
                Local::now(),
            );
        }

        use self::Cmd::*;

        let mut halt = false;
        match cmd {
            Halt => {
                assert!(self.bots[self.bot_index].pos.is_origin(), true);
                assert_eq!(self.bots.len(), 1 as usize);
                assert_eq!(self.harmonics, Harmonics::Low);
                halt = true;
            }
            Wait => {
                // No effect
            }
            Flip => {
                self.volatile.flip();
            }
            SMove(lld) => {
                let pos1 = self.bots[self.bot_index].pos;
                if self.is_move_interfared(pos1, lld.0) {
                    return CmdResult::Interfared;
                }
                let bot = &mut self.bots[self.bot_index];
                bot.smove(lld.0);
                self.energy += 2 * lld.0.mlen() as i64;
                self.volatile.smove(&Region::new(pos1, bot.pos));
            }
            LMove(sld1, sld2) => {
                let pos1 = self.bots[self.bot_index].pos;
                if self.is_move_interfared(pos1, sld1.0)
                    || self.is_move_interfared(pos1 + sld1.0, sld2.0)
                {
                    return CmdResult::Interfared;
                }
                let bot = &mut self.bots[self.bot_index];
                bot.smove(sld1.0);
                let pos2 = bot.pos;
                bot.smove(sld2.0);
                self.energy += 2 * (sld1.0.mlen() + 2 + sld2.0.mlen()) as i64;
                self.volatile
                    .lmove(&Region::new(pos1, pos2), &Region::new(pos2, bot.pos));
            }
            Fission(nd, m) => {
                if self.is_interfared(self.bots[self.bot_index].pos + nd.0) {
                    return CmdResult::Interfared;
                }
                let bot = &mut self.bots[self.bot_index];
                let new_bot = bot.fission(nd, m);
                self.energy += 24;
                self.volatile.fussion(new_bot);
            }
            Fill(nd) => {
                let c = self.bots[self.bot_index].pos + nd.0;
                assert!(!self.volatile.is_interfared(&c));
                if self.is_interfared(c) {
                    return CmdResult::Interfared;
                }
                if !self.matrix[c] {
                    self.matrix.fill(c);
                    self.priority_targets.remove(c);
                    self.energy += 12;
                } else {
                    warn!("Fill cmd for Full cord");
                    self.energy += 6;
                }
                self.volatile.fill(c);
            }
            Void(nd) => {
                let c = self.bots[self.bot_index].pos + nd.0;
                // this is Void commad, is_interfared(c) can't be used.
                if self.volatile.is_interfared(&c) {
                    return CmdResult::Interfared;
                }
                if self.matrix[c] {
                    self.matrix.void(c);
                    self.priority_targets.remove(c);
                    self.energy -= 12;
                } else {
                    warn!("Void cmdfor Void cord");
                    self.energy += 3;
                }
                self.volatile.void(c);
            }
            FusionP(nd) => {
                let p_pos = self.current_bot().pos;
                let s_pos = p_pos + nd.0;
                self.reserved_fusion.insert(s_pos, p_pos);
                let s_bot = self.bots.iter().find(|b| b.pos == s_pos).unwrap().clone();
                self.bots[self.bot_index].fusion(&s_bot);
                self.energy -= 24;
                self.volatile.bot_removed(s_bot.bid);
            }
            FusionS(nd) => {
                // Do nothing here. Just asserting.
                let s_pos = self.current_bot().pos;
                let p_pos = s_pos + nd.0;
                assert_eq!(self.reserved_fusion.get(&s_pos), Some(&p_pos));
                assert!(self.bots.iter().any(|b| b.pos == p_pos));
            }
        }

        self.records.push(cmd);

        self.bot_index += 1;
        if self.bot_index == self.bots.len() {
            self.prepare_next_time_step();
        }

        if halt {
            self.bots = vec![];
            CmdResult::Halt
        } else {
            CmdResult::Continue
        }
    }
}
