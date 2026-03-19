use screeps_arena::{Part, ReturnCode, StructureSpawn};

use super::{Status, Task};

/// The current phase of the spawn lifecycle.
enum SpawnPhase {
    /// Waiting for the spawn structure to become available (not currently
    /// spawning another creep).
    Waiting,
    /// The spawn structure is free; attempting to issue the `spawn_creep`
    /// intent.
    Spawning,
}

/// A task that spawns a creep with a given body from a [`StructureSpawn`].
///
/// This is modelled as a two-phase state machine that is driven forward by
/// calling [`Task::poll`] once per game tick:
///
/// 1. **Waiting** -- idles until the spawn structure is no longer busy.
/// 2. **Spawning** -- issues the `spawn_creep` intent. On success the task
///    completes; on transient failures (busy / insufficient energy) it falls
///    back to the waiting phase and retries on the next tick.
///
/// # Examples
///
/// ```no_run
/// let mut task = Spawn::new(my_spawn, vec![Part::Work, Part::Move]);
///
/// // Drive the task each tick until it completes.
/// loop {
///     if let Status::Complete = task.poll() {
///         break;
///     }
/// }
/// ```
pub struct Spawn {
    /// The spawn structure that will produce the creep.
    structure: StructureSpawn,
    /// The body parts the new creep should be composed of.
    body: Vec<Part>,
    /// The current phase of the spawn lifecycle.
    phase: SpawnPhase,
}

impl Spawn {
    /// Creates a new `Spawn` task.
    ///
    /// The task starts in the [`SpawnPhase::Waiting`] phase and will not issue
    /// any game intents until [`Task::poll`] is called.
    ///
    /// # Arguments
    ///
    /// * `structure` - The spawn structure to spawn the creep from.
    /// * `body` - The body part composition of the creep to spawn.
    pub fn new(structure: StructureSpawn, body: Vec<Part>) -> Self {
        Self {
            structure,
            body,
            phase: SpawnPhase::Waiting,
        }
    }
}

impl Task for Spawn {
    /// Advances the spawn task by one tick.
    ///
    /// Returns [`Status::Complete`] once the `spawn_creep` intent has been
    /// successfully registered, or [`Status::Pending`] if the task is still
    /// waiting for the spawn to become available or for enough energy to
    /// accumulate.
    fn poll(&mut self) -> Status {
        match self.phase {
            SpawnPhase::Waiting => {
                if self.structure.spawning().is_none() {
                    self.phase = SpawnPhase::Spawning;
                    self.poll()
                } else {
                    Status::Pending
                }
            }
            SpawnPhase::Spawning => {
                match self.structure.spawn_creep(&self.body) {
                    ReturnCode::Ok => Status::Complete,
                    ReturnCode::Busy => {
                        // Spawn became busy between our check and the call, wait again.
                        self.phase = SpawnPhase::Waiting;
                        Status::Pending
                    }
                    ReturnCode::NotEnoughEnergy => {
                        // Not enough energy yet, keep waiting.
                        self.phase = SpawnPhase::Waiting;
                        Status::Pending
                    }
                    _ => Status::Pending,
                }
            }
        }
    }
}
