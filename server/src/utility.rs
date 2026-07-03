use log::warn;
use spacetimedb::Identity;
use spacetimedsl::*;

use crate::{
    ships::*,
    stellarobjects::*,
    tables::global_config::*,
};

const IS_SERVER_ERROR: &str = "This reducer can only be called by SpacetimeDB!";
const IS_SERVER_OR_OWNER_ERROR: &str =
    "This reducer can only be called by SpacetimeDB or the owner!";

/// Checks if the context sender is the module owner (stored in GlobalConfig
/// during `init`). Scheduled/timer reducers with the all-zeros identity are
/// also allowed — they are the module itself.
pub fn try_server_only<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let sender = dsl.ctx().sender()?;

    // Scheduled reducers always fire with the zero identity.
    if sender == Identity::default() {
        return Ok(());
    }

    // Look up the stored owner identity from the GlobalConfig singleton.
    if let Ok(config) = dsl.get_global_config_by_id(GlobalConfigId::new(0)) {
        if sender == config.server_identity {
            return Ok(());
        }
    }

    warn!(
        "Denied server-only reducer request from: {}",
        sender.to_string()
    );

    Err(IS_SERVER_ERROR.to_string())
}

/// Checks if the context sender is the server or the owner of the given stellar object. ONLY for spacetimedb reducer functions!
pub fn is_server_or_sobj_owner<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    stellar_object_id: Option<StellarObjectId>,
) -> Result<(), String> {
    let sobj_id = stellar_object_id.ok_or_else(|| "Given a missing SOBJ ID".to_string())?;

    // Post-Phase 9: the `sobj_player_window` table is gone — find the owning
    // ship by `sobj_id` and check its player_id against the sender.
    let sender = dsl.ctx().sender()?;
    if let Some(ship) = dsl.get_ships_by_sobj_id(&sobj_id).next() {
        if ship.get_player_id().value() == sender {
            return Ok(());
        }
    }

    warn!("Denied server/sobj-owner request from: {}", sender.to_string());
    Err(IS_SERVER_OR_OWNER_ERROR.to_string())
}

/// Checks if the context sender is the server or the owner of the given Ship.
pub fn is_server_or_ship_owner<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    ship_id: Option<ShipId>,
) -> Result<(), String> {
    // Server is always allowed.
    try_server_only(dsl).or_else(|_| {
        let sid = ship_id.ok_or_else(|| "Missing ship ID".to_string())?;
        let ship = dsl.get_ship_by_id(&sid).map_err(|_| "Ship not found".to_string())?;
        let sender = dsl.ctx().sender()?;
        if ship.get_player_id().value() == sender {
            Ok(())
        } else {
            warn!("Denied server/ship-owner request from: {}", sender.to_string());
            Err("This reducer can only be called by SpacetimeDB or the ship owner!".to_string())
        }
    })
}
