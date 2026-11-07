//! №18,338: bounded offline peer evaluation; frozen entry/exit/cost formulae.
use super::{offline_repaired, s3_signal::Request, seasonal, three_year};
use serde_json::{json, Value};

pub fn study(req: &Request, payload: &three_year::Payload) -> Result<Value, String> {
    let date = req.from.format("%Y-%m-%d").to_string();
    if ![
        "2024-05-13",
        "2024-05-14",
        "2024-05-15",
        "2024-05-16",
        "2024-05-17",
        "2025-05-12",
        "2025-05-13",
        "2025-05-14",
        "2025-05-15",
        "2025-05-16",
    ]
    .contains(&date.as_str())
    {
        return Err("outside owner-selected peer dates".into());
    }
    let baseline = three_year::study_scope(req, payload, true)?;
    let peer = match req.symbol.as_str() {
        "SpotCrude" => "SpotBrent",
        "SpotBrent" => "SpotCrude",
        "US500" => "NAS100",
        "NAS100" => "US500",
        "XAUUSD" => "XAGUSD",
        "XAGUSD" => "XAUUSD",
        "BTCUSD" => "ETHUSD",
        "ETHUSD" => "BTCUSD",
        _ => return Err("outside peer symbol scope".into()),
    };
    let macro_model = three_year::authorized_macro(req, payload)?;
    Ok(json!({
        "models": {
            "s3_two_stage": offline_repaired::two_stage(req, payload, &baseline["models"]["production_s3"])?,
            "macro_180m_two_stage": offline_repaired::two_stage(req, payload, &macro_model)?,
            "production_s3_baseline": baseline["models"]["production_s3"],
            "always_long_control": baseline["models"]["always_long_control"],
            "own_momentum_fixed_horizon": baseline["models"]["own_momentum_fixed_horizon"]
        },
        "quality": baseline["quality"],
        "peer": {"required":peer,"available":payload.peer_symbol.as_deref()==Some(peer)
            && seasonal::range_at(&payload.peer_m30,req.from,180).is_some()},
        "financial_pass": false, "validation_opened": false
    }))
}
