use screeps_arena::{Creep, ResourceType, ReturnCode, Source, StructureSpawn};

use super::{Status, Task};

/// The current phase of the harvest lifecycle.
enum HarvestPhase {
    /// Moving toward the energy source.
    MovingToSource,
    /// Adjacent to the source; actively harvesting energy.
    Harvesting,
    /// Store is full; moving back to the spawn to deliver energy.
    MovingToSpawn,
    /// Adjacent to the spawn; transferring energy.
    Transferring,
}

/// A task that sends a creep to harvest energy from a [`Source`] and deliver it
/// to a [`StructureSpawn`].
///
/// This is modelled as a four-phase state machine that is driven forward by
/// calling [`Task::poll`] once per game tick:
///
/// 1. **MovingToSource** -- moves toward the source until in range, then
///    transitions to harvesting.
/// 2. **Harvesting** -- harvests energy each tick until the creep's store is
///    full, then transitions to moving back to the spawn.
/// 3. **MovingToSpawn** -- moves toward the spawn until in range, then
///    transitions to transferring.
/// 4. **Transferring** -- transfers energy to the spawn. Once the creep's
///    store is empty the task completes.
///
/// # Examples
///
/// ```no_run
/// let mut task = Harvest::new(worker, source, spawn);
///
/// // Drive the task each tick until it completes.
/// loop {
///     if let Status::Complete = task.poll() {
///         break;
///     }
/// }
/// ```
pub struct Harvest {
    /// The creep performing the harvest.
    creep: Creep,
    /// The energy source to harvest from.
    source: Source,
    /// The spawn structure to deliver energy to.
    spawn: StructureSpawn,
    /// The current phase of the harvest lifecycle.
    phase: HarvestPhase,
}

impl Harvest {
    /// Creates a new `Harvest` task.
    ///
    /// The task starts in the [`HarvestPhase::MovingToSource`] phase and will
    /// not issue any game intents until [`Task::poll`] is called.
    ///
    /// # Arguments
    ///
    /// * `creep` - The creep that will perform the harvesting.
    /// * `source` - The energy source to harvest from.
    /// * `spawn` - The spawn structure to deliver energy to.
    pub fn new(creep: Creep, source: Source, spawn: StructureSpawn) -> Self {
        Self {
            creep,
            source,
            spawn,
            phase: HarvestPhase::MovingToSource,
        }
    }
}

impl Task for Harvest {
    /// Advances the harvest task by one tick.
    ///
    /// Returns [`Status::Complete`] once the creep has delivered all its
    /// energy to the spawn, or [`Status::Pending`] if the task is still in
    /// progress.
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
                let store = self.creep.store();
                if store.get_used_capacity(None) >= store.get_capacity(None) {
                    // Store is full, start delivering to spawn.
                    self.phase = HarvestPhase::MovingToSpawn;
                    return self.poll();
                }

                self.creep.harvest(&self.source);
            }
            HarvestPhase::MovingToSpawn => {
                if self.creep.transfer(&self.spawn, ResourceType::Energy, None)
                    == ReturnCode::NotInRange
                {
                    self.creep.move_to(&self.spawn, None);
                } else {
                    self.phase = HarvestPhase::Transferring;
                }
            }
            HarvestPhase::Transferring => {
                if self.creep.store().get_used_capacity(None) == 0 {
                    return Status::Complete;
                }

                self.creep.transfer(&self.spawn, ResourceType::Energy, None);
            }
        }

        Status::Pending
    }
}
