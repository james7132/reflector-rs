use arch_mirrors_rs::Mirror;

const MIRROR_JSON: &str = r#"{
    "url": "https://mirror.example.com/archlinux/",
    "protocol": "https",
    "last_sync": null,
    "completion_pct": 1.0,
    "delay": -145,
    "duration_avg": 0.42,
    "duration_stddev": 0.1,
    "score": 0.5,
    "active": true,
    "country": "Australia",
    "country_code": "AU",
    "isos": true,
    "ipv4": true,
    "ipv6": true,
    "details": "https://archlinux.org/mirrors/example/"
}"#;

#[test]
fn handles_current_and_legacy_duration_fields() -> Result<(), serde_json::Error> {
    let mirror: Mirror = serde_json::from_str(MIRROR_JSON)?;

    assert_eq!(mirror.delay, Some(-145));
    assert_eq!(mirror.duration_average, Some(0.42));

    let serialized = serde_json::to_value(&mirror)?;
    assert_eq!(
        serialized
            .get("duration_avg")
            .and_then(serde_json::Value::as_f64),
        Some(0.42)
    );
    assert!(serialized.get("duration_average").is_none());

    let legacy_json = MIRROR_JSON.replace("duration_avg", "duration_average");
    let legacy: Mirror = serde_json::from_str(&legacy_json)?;
    assert_eq!(legacy.duration_average, Some(0.42));

    let large_delay_json = MIRROR_JSON.replace("-145", "4294967295");
    let large_delay: Mirror = serde_json::from_str(&large_delay_json)?;
    assert_eq!(large_delay.delay, Some(4_294_967_295));
    Ok(())
}
