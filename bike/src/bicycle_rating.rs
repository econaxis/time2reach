use std::collections::HashMap;

pub fn filter_by_tag(osm_tags: &HashMap<String, String>) -> bool {
    if osm_tags.contains_key("man_made") || osm_tags.contains_key("railway") || osm_tags.get("area").unwrap_or(&"".to_string()) == "yes" {
        return false;
    }
    match osm_tags.get("access").map(AsRef::as_ref) {
        Some("no") | Some("private") => return false,
        _ => {}
    }
    match osm_tags.get("area").map(AsRef::as_ref) {
        Some("yes") => return false,
        _ => {}
    }

    if let Some(highway) = osm_tags.get("highway") {
        match highway.as_str() {
            "elevator" | "steps" | "platform" | "track" => return false,
            _ => {}
        }
    }
    return true;
}
pub fn rate_bicycle_friendliness(osm_tags: &HashMap<String, String>) -> u8 {
    // Check for bicycle-related tags
    let mut footway = false;
    let mut bicycle = false;
    for (key, value) in osm_tags {
        if key.starts_with("cycleway") {
            match value.as_str() {
                "lane" | "opposite_lane" | "opposite" | "track" | "opposite_track" => return 5,  // Designated for bicycles
                "shared_lane" | "share_busway" | "opposite_share_busway" | "shared" => return 4,  // Bicycle-friendly road
                "no" => return 0,  // Not bicycle-friendly
                _ => {}
            }
        }
        // Check for highway type
        if key.starts_with("highway") {
            match value.as_str() {
                "path" | "cycleway" => return 5,  // Designated for bicycles
                "motorway" | "trunk" | "primary" | "secondary" => return 1,  // Less bicycle-friendly for busy roads
                "tertiary" => return 2,
                "service" | "residential" | "unclassified" => return 3,  // More bicycle-friendly for residential roads
                "footway" | "pedestrian" => footway = true,
                "steps" | "corridor" => return 0,  // Stairs are not bicycle-friendly
                _ => {}  // Default rating for other highway types
            }
        }
        if key.starts_with("bicycle") {
            match value.as_str() {
                "designated" => return 5,  // Designated for bicycles
                "yes" | "permissive" | "use_sidepath" => {
                    bicycle = true;
                }  // Bicycle-friendly road
                "no" => return 0,  // Not bicycle-friendly
                _ => {}
            }
        }
    }

    match (footway, bicycle) {
        (true, true) => return 5,  // Bicycle-friendly footway
        (true, false) => return 0,
        (false, true) => return 3,
        _ => {}
    }

    // eprintln!("No bicycle-related tags found for this way. {:?}", osm_tags);

    return 2;
}