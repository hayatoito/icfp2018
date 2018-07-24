use std::collections::HashSet;

use super::bot::*;
use super::prelude::*;
use super::system::*;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Ai {
    Many(usize),
}

pub struct Many {
    bots: usize,
}

impl Many {
    pub fn new(bots: usize) -> Many {
        Many { bots }
    }

    pub fn solve(&mut self, sys: &mut System) -> Result<()> {
        let origin = Cord::new(0, 0, 0);
        let origin_set = vec![origin].into_iter().collect::<HashSet<_>>();

        let mut wait_cont = 0;
        while !sys.priority_targets.priority_targets.is_empty() {
            let targets = sys.free_priority_targets();
            let cmd = {
                if self.bots > 1 {
                    if let Some(cmd) = sys.can_current_bot_fisson() {
                        self.bots -= 1;
                        cmd
                    } else {
                        sys.move_to_target_and_fill_or_void(&targets)
                    }
                } else {
                    sys.move_to_target_and_fill_or_void(&targets)
                }
            };
            if let Cmd::Wait = cmd {
                wait_cont += 1;
                if wait_cont == 10 {
                    return Err(NanoBotError.into());
                }
            } else {
                wait_cont = 0;
            }
            sys.assert_execute_cmd(cmd);
        }

        // Return to origin and fusion
        loop {
            if sys.bots.len() == 1 {
                if sys.current_bot().pos == origin {
                    assert_eq!(sys.execute_cmd(Cmd::Halt), CmdResult::Halt);
                    return Ok(());
                }
                let cmd = sys.move_to_first_or_wait_cmd(sys.current_bot().pos, origin);
                if let Cmd::Wait = cmd {
                    warn!("can not move to origin",);
                    return Err(NanoBotError.into());
                }
                sys.assert_execute_cmd(cmd);
            } else {
                let cmd =
                    if let Some(primary_cord) = sys.is_current_bot_reserved_as_fusion_secondary() {
                        Cmd::FusionS(Near(*primary_cord - sys.current_bot().pos))
                    } else if let Some(second) = sys.find_current_bot_fusion_opponent() {
                        Cmd::FusionP(Near(second.pos - sys.current_bot().pos))
                    } else if sys.current_bot().pos == origin {
                        Cmd::Wait
                    } else if sys.is_interfared(origin) {
                        if let Ok(MoveToNear { move_cmds, .. }) =
                            sys.move_to_near(sys.current_bot().pos, &origin_set)
                        {
                            if move_cmds.cmds.is_empty() {
                                Cmd::Wait
                            } else {
                                move_cmds.cmds[0]
                            }
                        } else {
                            Cmd::Wait
                        }
                    } else {
                        sys.move_to_first_or_wait_cmd(sys.current_bot().pos, origin)
                    };
                if let Cmd::Wait = cmd {
                    wait_cont += 1;
                    if wait_cont == 10 {
                        return Err(NanoBotError.into());
                    }
                } else {
                    wait_cont = 0;
                }
                sys.assert_execute_cmd(cmd);
            }
        }
    }
}
