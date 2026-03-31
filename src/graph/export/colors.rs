use std::collections::{HashMap, HashSet, VecDeque};

/// Expand adjacency so two labels are neighbors if reachable within k hops
pub fn expand_to_k_hops(
    labels: &[String],
    adj: &HashMap<String, HashSet<String>>,
    k: usize,
) -> HashMap<String, HashSet<String>> {
    let mut expanded: HashMap<String, HashSet<String>> = HashMap::new();
    for label in labels {
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(label.clone());
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        queue.push_back((label.clone(), 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= k {
                continue;
            }
            if let Some(neighbors) = adj.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }

        visited.remove(label);
        expanded.insert(label.clone(), visited);
    }
    expanded
}

/// Greedy graph coloring sorted by degree descending
pub fn greedy_color(
    labels: &[String],
    adj: &HashMap<String, HashSet<String>>,
) -> HashMap<String, usize> {
    let mut sorted: Vec<&String> = labels.iter().collect();
    sorted.sort_by(|a, b| {
        let deg_a = adj.get(a.as_str()).map_or(0, |s| s.len());
        let deg_b = adj.get(b.as_str()).map_or(0, |s| s.len());
        deg_b.cmp(&deg_a)
    });

    let mut coloring: HashMap<String, usize> = HashMap::new();
    for label in sorted {
        let neighbor_colors: HashSet<usize> =
            adj.get(label.as_str())
                .map_or_else(HashSet::new, |neighbors| {
                    neighbors
                        .iter()
                        .filter_map(|n| coloring.get(n.as_str()).copied())
                        .collect()
                });
        let mut color = 0;
        while neighbor_colors.contains(&color) {
            color += 1;
        }
        coloring.insert(label.clone(), color);
    }
    coloring
}

/// Generate n maximally distinct colors using golden-angle HSL spacing
pub fn generate_palette(n: usize) -> Vec<String> {
    let golden_angle: f64 = 137.508;
    (0..n)
        .map(|i| {
            let hue = (i as f64 * golden_angle) % 360.0;
            let sat = 0.40 + 0.10 * ((i % 3) as f64 / 2.0);
            let light = 0.35 + 0.08 * ((i % 2) as f64);
            let (r, g, b) = hsl_to_rgb(hue / 360.0, sat, light);
            format!(
                "#{:02X}{:02X}{:02X}",
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8
            )
        })
        .collect()
}

/// Convert HSL to RGB (h, s, l all in 0..1)
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    (r, g, b)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Return white or black text color based on background luminance
pub fn text_color_for_bg(hex: &str) -> &'static str {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0) as f64 / 255.0;
    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if luminance < 0.5 {
        "#FFFFFF"
    } else {
        "#000000"
    }
}

/// Darken a hex color by a factor (0.0 - 1.0)
pub fn darken(hex: &str, factor: f64) -> String {
    let r = (u8::from_str_radix(&hex[1..3], 16).unwrap_or(0) as f64 * factor) as u8;
    let g = (u8::from_str_radix(&hex[3..5], 16).unwrap_or(0) as f64 * factor) as u8;
    let b = (u8::from_str_radix(&hex[5..7], 16).unwrap_or(0) as f64 * factor) as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}
