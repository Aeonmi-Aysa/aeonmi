use aeonmi_project::core::incremental::{
    record_partial_savings, reset_metrics_full, SAVINGS_METRICS,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static TEST_GUARD: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[test]
fn savings_record_accumulates() {
    let _lock = TEST_GUARD.lock().unwrap();
    reset_metrics_full();
    record_partial_savings(10, 30); // savings 20
    record_partial_savings(5, 15); // savings 10 (total 30)
    let sm = SAVINGS_METRICS.lock().unwrap().clone();
    assert_eq!(sm.cumulative_partial_ns, 15);
    assert_eq!(sm.cumulative_estimated_full_ns, 45);
    assert_eq!(sm.cumulative_savings_ns, 30);
}

#[test]
fn savings_ignore_negative() {
    let _lock = TEST_GUARD.lock().unwrap();
    reset_metrics_full();
    // estimated lower than partial -> should be ignored
    record_partial_savings(50, 40);
    let sm = SAVINGS_METRICS.lock().unwrap().clone();
    assert_eq!(sm.cumulative_partial_ns, 0);
    assert_eq!(sm.cumulative_estimated_full_ns, 0);
    assert_eq!(sm.cumulative_savings_ns, 0);
}
