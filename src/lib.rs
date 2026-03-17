mod task;

use std::sync::LazyLock;

use screeps_arena::{
    game::{self, utils},
    prototypes, Creep, OwnedStructureProperties, Part, ResourceType, ReturnCode, StructureSpawn,
};
use wasm_bindgen::prelude::*;

mod logging;

struct Population {
    workers: Vec<Creep>,
    melee_attackers: Vec<Creep>,
    ranged_attackers: Vec<Creep>,
    healers: Vec<Creep>,
}

impl Population {
    fn new(creeps: Vec<Creep>) -> Self {
        creeps.into_iter().filter(|creep| creep.my()).fold(
            Self::default(),
            |mut population, creep| {
                if has_part(&creep, Part::Work) {
                    population.workers.push(creep);
                } else if has_part(&creep, Part::Attack) {
                    population.melee_attackers.push(creep);
                } else if has_part(&creep, Part::RangedAttack) {
                    population.ranged_attackers.push(creep);
                } else if has_part(&creep, Part::Heal) {
                    population.healers.push(creep);
                }

                population
            },
        )
    }

    fn maintain(&self, spawn: &StructureSpawn) {
        match self {
            Self { workers, .. } if workers.len() < 2 => {
                let _ = spawn.spawn_creep(&[Part::Work, Part::Move]);
            }
            Self {
                melee_attackers, ..
            } if melee_attackers.len() < 1 => {
                let _ = spawn.spawn_creep(&[Part::Attack, Part::Move]);
            }
            Self {
                ranged_attackers, ..
            } if ranged_attackers.len() < 1 => {
                let _ = spawn.spawn_creep(&[Part::RangedAttack, Part::Move]);
            }
            Self { healers, .. } if healers.len() < 1 => {
                let _ = spawn.spawn_creep(&[Part::Heal, Part::Move]);
            }
            _ => (),
        };
    }
}

impl Default for Population {
    fn default() -> Self {
        Self {
            workers: vec![],
            healers: vec![],
            melee_attackers: vec![],
            ranged_attackers: vec![],
        }
    }
}

static CREEPS: LazyLock<Population> =
    LazyLock::new(|| Population::new(utils::get_objects_by_prototype(prototypes::CREEP)));

fn setup() {
    logging::setup_logging(logging::Info);
}

#[wasm_bindgen(js_name = wasm_loop)]
pub fn tick() {
    let tick = game::utils::get_ticks();

    if tick == 1 {
        setup();
    }

    let spawn = utils::get_objects_by_prototype(prototypes::STRUCTURE_SPAWN)
        .into_iter()
        .find(|spawn| spawn.my().unwrap_or(false))
        .expect("No spawn found!");

    // Maintain pop
    LazyLock::get_mut(&mut CREEPS);

    for worker in population.workers.iter() {
        let store = worker.store();
        if store.get_used_capacity(None) < store.get_capacity(None) {
            let sources = utils::get_objects_by_prototype(prototypes::SOURCE);

            if worker.harvest(&sources[0]) == ReturnCode::NotInRange {
                worker.move_to(&sources[0], None);
            }
        } else {
            if worker.transfer(&spawn, ResourceType::Energy, None) == ReturnCode::NotInRange {}
        }
    }
}

fn has_part(creep: &Creep, part: Part) -> bool {
    creep
        .body()
        .iter()
        .any(|body_part| body_part.part() == part)
}
