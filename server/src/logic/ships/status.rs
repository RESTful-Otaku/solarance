use std::time::Duration;

use spacetimedb::*;
use spacetimedsl::*;

use crate::{
    tables::{items::*, ships::*},
    utility::try_server_only,
};

#[dsl(plural_name = ship_status_timers, method(update = false))]
#[spacetimedb::table(accessor = ship_status_timer, scheduled(ship_status_timer_reducer))]
pub struct ShipStatusTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,

    #[unique]
    #[use_wrapper(ShipId)]
    /// FK to Ship
    ship_id: u64,

    #[use_wrapper(ShipTypeDefinitionId)]
    /// FK to Ship Type
    ship_type_id: u32,
}

pub fn create_status_timer_for_ship<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_id: &ShipId,
    type_id: &ShipTypeDefinitionId,
) -> Result<ShipStatusTimer, String> {
    let timer = dsl.create_ship_status_timer(CreateShipStatusTimer {
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_millis(500).into()),
        ship_id: ship_id.clone(),
        ship_type_id: type_id.clone(),
    })?;

    Ok(timer)
}

/// Extract the shield-regen-per-second value from an item definition's metadata, if present.
fn shield_regen_from_item(item: &ItemDefinition) -> f32 {
    item.metadata.iter().fold(0.0f32, |acc, m| {
        if let ItemMetadata::ShieldRegenPerSecond(v) = m {
            acc + v
        } else {
            acc
        }
    })
}

/// Extract the energy-regen-per-second value from an item definition's metadata, if present.
fn energy_regen_from_item(item: &ItemDefinition) -> f32 {
    item.metadata.iter().fold(0.0f32, |acc, m| {
        if let ItemMetadata::EnergyRegenPerSecond(v) = m {
            acc + v
        } else {
            acc
        }
    })
}

const FLAT_SHIELD_REGEN_PER_TICK: f32 = 0.525175;
const FLAT_ENERGY_REGEN_PER_TICK: f32 = 0.1275;

/// Scheduled reducer that handles ship status updates like shield and energy regeneration.
/// Runs every 500ms to gradually restore shields and energy based on ship type specifications.
#[spacetimedb::reducer]
pub fn ship_status_timer_reducer(
    ctx: &ReducerContext,
    timer: ShipStatusTimer,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    try_server_only(&dsl)?;

    let mut changes = false;
    let ship_type = dsl.get_ship_type_definition_by_id(timer.get_ship_type_id())?;
    let mut ship_status = dsl.get_ship_status_by_id(timer.get_ship_id())?;

    // Calculate regen rates from equipped modules. Sum all matching metadata
    // values across Shield and Special slots, falling back to flat defaults.
    let mut shield_regen_per_tick = FLAT_SHIELD_REGEN_PER_TICK;
    let mut energy_regen_per_tick = FLAT_ENERGY_REGEN_PER_TICK;
    for slot in dsl.get_ship_equipment_slots_by_ship_id(timer.get_ship_id()) {
        if let Ok(def) = dsl.get_item_definition_by_id(slot.get_item_id()) {
            match slot.get_slot_type() {
                EquipmentSlotType::Shield => {
                    let r = shield_regen_from_item(&def);
                    if r > 0.0 {
                        // Convert per-second to per-tick (500ms tick interval)
                        shield_regen_per_tick = r * 0.5;
                    }
                }
                EquipmentSlotType::Special => {
                    let r = energy_regen_from_item(&def);
                    if r > 0.0 {
                        energy_regen_per_tick = r * 0.5;
                    }
                }
                _ => {}
            }
        }
    }

    if *ship_status.get_shields() < (*ship_type.get_max_shields() as f32) {
        ship_status.set_shields(*ship_status.get_shields() + shield_regen_per_tick);
        changes = true;
    }
    if *ship_status.get_shields() > (*ship_type.get_max_shields() as f32) {
        ship_status.set_shields(*ship_type.get_max_shields() as f32);
        changes = true;
    }

    if *ship_status.get_energy() < (*ship_type.get_max_energy() as f32) {
        ship_status.set_energy(ship_status.get_energy() + energy_regen_per_tick);
        changes = true;
    }
    if *ship_status.get_energy() > (*ship_type.get_max_energy() as f32) {
        ship_status.set_energy(*ship_type.get_max_energy() as f32);
        changes = true;
    }

    if changes {
        dsl.update_ship_status_by_id(ship_status)?;
    }

    Ok(())
}
