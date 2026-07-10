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

/// Pure predicate: is `sender` an authorised system caller?
///
/// System callers are exactly three identities, nothing else:
/// 1. `Identity::default()` — scheduled reducers in older SpacetimeDB versions.
/// 2. `module_identity` (`ctx.identity()`) — the database identity that
///    scheduled/timer reducers fire under in SpacetimeDB >= 2.6.0.
/// 3. `server_identity` (stored in `GlobalConfig` at `init`) — the module
///    publisher, for admin reducers invoked from a privileged client.
///
/// Any *player* identity must be denied. Previously this also admitted
/// "any identity with no `Player` row," which let an unregistered client
/// pass as the server — a privilege-escalation hole. This pure form is
/// testable without a live SpacetimeDB runtime.
pub fn is_system_identity(
    sender: Identity,
    module_identity: Identity,
    server_identity: Option<Identity>,
) -> bool {
    if sender == Identity::default() {
        return true;
    }
    if sender == module_identity {
        return true;
    }
    server_identity.is_some_and(|sid| sid == sender)
}

/// Checks if the context sender is an authorised system caller.
/// See [`is_system_identity`] for the allowlist; this is the reducer-facing
/// wrapper that pulls the identities from `ctx` + `GlobalConfig`.
pub fn try_server_only<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let ctx = dsl.ctx();
    let sender = ctx.sender()?;
    let module_identity = ctx.module_identity()?;
    let server_identity = dsl
        .get_global_config_by_id(GlobalConfigId::new(0))
        .ok()
        .map(|c| *c.get_server_identity());

    if is_system_identity(sender, module_identity, server_identity) {
        return Ok(());
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

#[cfg(test)]
mod tests {
    use super::*;

    // Build distinct non-zero identities for tests. `Identity::ZERO` equals
    // `Identity::default()` (the system/scheduled caller), so we avoid it for
    // "player-like" identities. `ONE` and a fixed byte array give us
    // guaranteed-distinct values without a live ReducerContext.
    const PLAYER_A_BYTES: [u8; 32] = {
        let mut b = [0u8; 32];
        b[0] = 0x01;
        b
    };
    const PLAYER_B_BYTES: [u8; 32] = {
        let mut b = [0u8; 32];
        b[0] = 0x02;
        b
    };

    fn player_a() -> Identity {
        Identity::from_be_byte_array(PLAYER_A_BYTES)
    }
    fn player_b() -> Identity {
        Identity::from_be_byte_array(PLAYER_B_BYTES)
    }

    #[test]
    fn test_default_identity_is_system_caller() {
        assert!(is_system_identity(
            Identity::default(),
            player_a(),
            Some(player_b()),
        ));
    }

    #[test]
    fn test_module_identity_is_system_caller() {
        let module = player_a();
        assert!(is_system_identity(module, module, None));
        assert!(is_system_identity(module, module, Some(player_b())));
    }

    #[test]
    fn test_server_identity_is_system_caller() {
        let server = player_a();
        assert!(is_system_identity(server, player_b(), Some(server)));
    }

    #[test]
    fn test_unknown_identity_is_denied_even_without_player_row() {
        // The regression: previously any non-player identity was admitted as
        // the server. Now only the explicit allowlist passes.
        let sender = player_a();
        assert!(!is_system_identity(sender, player_b(), None));
        assert!(!is_system_identity(sender, player_b(), Some(player_b())));
    }

    #[test]
    fn test_none_server_identity_denies_non_system() {
        assert!(!is_system_identity(player_a(), player_b(), None));
    }

    #[test]
    fn test_identity_one_denied_when_not_in_allowlist() {
        assert!(!is_system_identity(
            Identity::ONE,
            player_a(),
            Some(player_b()),
        ));
    }
}
