use std::collections::Bound::{Excluded, Included};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use super::model::*;
use super::prelude::*;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct PriorityTarget {
    pub priority: i64, // small number is high priority.
    pub cord: Cord,
}

#[derive(Default, Debug)]
pub struct PriorityTargets {
    pub priority_targets: BTreeSet<PriorityTarget>,
    index: HashMap<Cord, PriorityTarget>,
}

impl PriorityTargets {
    pub fn new(model: &Model) -> PriorityTargets {
        let r = model.r;
        let mut priority_targets = BTreeSet::new();
        let mut index = HashMap::new();

        let mut q = VecDeque::new();
        let mut visited = HashSet::new();

        struct Entry {
            c: Cord,
            len: i64,
        };

        for x in 0..r {
            for z in 0..r {
                let c = Cord::new(x as i32, 0, z as i32);
                if model.targets.contains(&c) {
                    q.push_back(Entry { c, len: 0 });
                    visited.insert(c);
                    let target = PriorityTarget {
                        priority: 0,
                        cord: c,
                    };
                    priority_targets.insert(target.clone());
                    index.insert(c, target);
                }
            }
        }

        while let Some(entry) = q.pop_front() {
            for diff in CordDiff::gen_all_diff() {
                let c = entry.c + *diff;
                if !c.is_in_range(r) {
                    continue;
                }
                if !model.targets.contains(&c) {
                    continue;
                }
                if !visited.contains(&c) {
                    visited.insert(c);
                    q.push_back(Entry {
                        c,
                        len: entry.len + 1,
                    });
                    let target = PriorityTarget {
                        priority: match model.id {
                            ModelId::Assemble(_) => entry.len + 1,
                            ModelId::Disassemble(_) => -(entry.len + 1),
                        },
                        cord: c,
                    };
                    priority_targets.insert(target.clone());
                    index.insert(c, target);
                }
            }
        }
        assert_eq!(model.targets.len(), index.len());
        assert_eq!(model.targets.len(), priority_targets.len());
        debug!("priority_targets: collected: {}", priority_targets.len());
        PriorityTargets {
            priority_targets,
            index,
        }
    }

    pub fn top_priority_targets(priority_targets: &BTreeSet<PriorityTarget>) -> HashSet<Cord> {
        if priority_targets.is_empty() {
            Default::default()
        } else {
            let head = priority_targets.iter().next().unwrap();
            priority_targets
                .range((
                    Included(head),
                    Excluded(&PriorityTarget {
                        priority: head.priority + 1,
                        cord: Cord::new(0, 0, 0),
                    }),
                ))
                .map(|p| p.cord)
                .collect()
        }
    }

    pub fn remove(&mut self, cord: Cord) {
        let t = &self.index[&cord];
        self.priority_targets.remove(t);
    }
}
