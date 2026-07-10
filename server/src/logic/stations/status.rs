use crate::tables::stations::*;
use spacetimedb::*;
use spacetimedsl::*;

#[dsl(plural_name = station_status_schedules, method(update = true))]
#[spacetimedb::table(accessor = station_status_schedule, scheduled(station_status_schedule_reducer))]
pub struct StationStatusSchedule {
    #[primary_key]
    #[use_wrapper(StationId)]
    id: u64,
    pub scheduled_at: spacetimedb::ScheduleAt,
    pub last_processed_timestamp: spacetimedb::Timestamp,
}

#[spacetimedb::reducer]
pub fn station_status_schedule_reducer(ctx: &ReducerContext, timer: StationStatusSchedule) {
    let dsl = dsl(ctx);
    // Defense-in-depth: scheduled reducers are private in ST 2.x, but
    // enforce the system-only allowlist anyway.
    if let Err(e) = crate::utility::try_server_only(&dsl) {
        spacetimedb::log::error!("Denied station_status_schedule_reducer: {e}");
        return;
    }
    if let Err(e) = process_station_status_tick(&dsl, timer.get_id()) {
        spacetimedb::log::error!("Station status tick failed for station {}: {}", timer.get_id(), e);
    }
}

//////////////////////////////////////////////////////////////

/// Regenerates station shields each tick.
/// Creates a `StationStatus` row on first tick if one doesn't exist yet
/// (backward-compatible with stations seeded before this was implemented).
pub fn process_station_status_tick<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    station_id: StationId,
) -> Result<(), String> {
    let station = dsl.get_station_by_id(&station_id)?;
    let base_shields = station.get_size().calculate_base_shields() as f32;
    let base_health = station.get_size().calculate_base_health() as f32;

    let mut status = match dsl.get_station_status_by_id(&station_id) {
        Ok(s) => s,
        Err(_) => dsl.create_station_status(CreateStationStatus {
            id: station_id.into(),
            health: base_health,
            shields: 0.0,
            energy: 0.0,
        })?,
    };

    // Regen shields: 1% of base per tick (every 10s → 6% per minute, full in ~17 min)
    if status.shields < base_shields {
        let regen = (base_shields * 0.01).max(10.0);
        status.shields = (status.shields + regen).min(base_shields);
        dsl.update_station_status_by_id(status)?;
    }

    Ok(())
}
