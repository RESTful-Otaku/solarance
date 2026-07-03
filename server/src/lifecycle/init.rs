use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::{definitions, tables::global_config::*};

use super::timers;

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    definitions::init(&dsl)?;
    timers::initialize(&dsl)?;

    let server_identity = ctx.sender();

    // Create a Global Config row, or reinitalize the one if it exists.
    if dsl.get_all_global_configurations().count() == 0 {
        dsl.create_global_config(CreateGlobalConfig {
            id: 0,
            active_players: 0,
            old_gods_defeated: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
            server_identity,
            cargo_crate_ttl_secs: 4 * 60 * 60, // 4 hours
            cargo_crate_toss_speed: 12.0,
            cargo_crate_toss_speed_variance: 4.0,
            cargo_crate_brake_rate: 1.5,
            cargo_crate_brake_rate_variance: 0.5,
            cargo_crate_max_turn_rate: std::f32::consts::PI,
        })?;
        info!("GlobalConfig created with server identity: {}", server_identity);
    } else {
        let mut config = dsl
            .get_all_global_configurations()
            .into_iter()
            .last()
            .ok_or("Failed to find existing global configuration")?;
        config.set_active_players(0);
        if *config.get_server_identity() == Identity::default() {
            config.set_server_identity(server_identity);
            info!("Updated GlobalConfig server_identity to: {}", server_identity);
        }
        dsl.update_global_config_by_id(config)?;
    }
    Ok(())
}
