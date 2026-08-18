/// Light themes must sort to the bottom, and the classifier must agree with
/// what each theme is actually called — a palette misjudged by luminance would
/// scatter light themes through the dark ones.
#[test]
fn light_themes_sort_last_and_are_classified_correctly() {
    let presets = ratarss::theme::Theme::all_presets();
    assert!(presets.len() >= 45, "expected the enlarged theme set, got {}", presets.len());

    // No dark theme after the first light one.
    let first_light = presets.iter().position(|t| t.is_light());
    if let Some(idx) = first_light {
        for t in &presets[idx..] {
            assert!(t.is_light(), "dark theme '{}' sorted after a light one", t.config.name);
        }
    }

    // Names that say light/dawn/day/latte must be classified light, and vice versa.
    for t in &presets {
        let name = t.config.name.to_lowercase();
        let says_light = name.contains("light")
            || name.contains("dawn")
            || name.contains("day")
            || name.contains("latte");
        let says_dark = name.contains("dark")
            || name.contains("mocha")
            || name.contains("night")
            || name.contains("moon");
        if says_light {
            assert!(t.is_light(), "'{}' is named light but reads as dark", t.config.name);
        }
        if says_dark && !says_light {
            assert!(!t.is_light(), "'{}' is named dark but reads as light", t.config.name);
        }
    }

    // Every theme is reachable by name, and names are unique.
    let mut names: Vec<&str> = presets.iter().map(|t| t.config.name.as_str()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate theme names");
    for t in &presets {
        let found = ratarss::theme::Theme::by_name(&t.config.name);
        assert_eq!(found.config.name, t.config.name);
    }
}
