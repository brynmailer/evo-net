use screeps_arena::{Creep, ReturnCode, Source, StructureSpawn};

use super::{Status, Task};

enum HarvestPhase {
    MovingToSource,
    Harvesting,
    MovingToSpawn,
    Transferring,
}

pub struct Harvest {
    creep: Creep,
    source: Source,
    spawn: StructureSpawn,
    phase: HarvestPhase,
}

impl Task for Harvest {
    fn poll(&mut self) -> Status {
        match self.phase {
            HarvestPhase::MovingToSource => {
                if self.creep.harvest(&self.source) == ReturnCode::NotInRange {
                    self.creep.move_to(&self.source, None);
                } else {
                    self.phase = HarvestPhase::Harvesting;
                }
            }
            HarvestPhase::Harvesting => {
                // Harvest
            }
        }

        Status::Pending
    }
}
