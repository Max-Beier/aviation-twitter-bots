use crate::types::Flight;

pub enum FormatOrder {
    ALTITUDE,
    GROUNDSPEED,
}

pub fn format_tweet(flight: &Flight, format_order: FormatOrder) -> String {
    let link = format!("https://www.flightaware.com/live/flight/{}", flight.ident);

    let alt_readout;
    let spd_readout;

    if let Some(alt_fl) = flight.altitude {
        let alt_feet = alt_fl * 100;
        let alt_meters = alt_feet as f32 * 0.3048;
        alt_readout = format!("{}ft ({:.2}m)", alt_feet, alt_meters);
    } else {
        alt_readout = "N/A".to_string();
    }

    if let Some(spd_knots) = flight.groundspeed {
        let spd_kmh = spd_knots as f32 * 1.852;
        spd_readout = format!("{}kts ({:.2}km/h)", spd_knots, spd_kmh);
    } else {
        spd_readout = "N/A".to_string();
    }

    let origin = flight.origin.clone().unwrap_or("Unknown".to_string());
    let destination = flight.destination.clone().unwrap_or("Unknown".to_string());

    match format_order {
        FormatOrder::ALTITUDE => format!(
            "Current highest flight: {}\n\
            Altitude: {}\n\
            Groundspeed: {}\n\
            Origin: {}\n\
            Destination: {}\n\n\
            More info:\n{}",
            flight.ident, alt_readout, spd_readout, origin, destination, link
        ),
        FormatOrder::GROUNDSPEED => format!(
            "Current fastest flight: {}\n\
            Groundspeed: {}\n\
            Altitude: {}\n\
            Origin: {}\n\
            Destination: {}\n\n\
            More info:\n{}",
            flight.ident, spd_readout, alt_readout, origin, destination, link
        ),
    }
}
