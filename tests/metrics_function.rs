use aeonmi_project::core::incremental::{
    force_persist_metrics, load_metrics, metrics_file_location, record_function_infer,
    record_reinfer_event, reset_metrics_full, CALL_GRAPH_METRICS, FUNCTION_METRICS,
};
use std::fs;

#[test]
fn function_metrics_record_and_reset() {
    reset_metrics_full();
    let baseline_reinfer_events = {
        let cg = CALL_GRAPH_METRICS.lock().unwrap_or_else(|e| e.into_inner());
        cg.reinfer_events
    };
    record_function_infer(0, 1000);
    record_function_infer(0, 500);
    record_function_infer(1, 200);
    {
        let fm = FUNCTION_METRICS.lock().unwrap_or_else(|e| e.into_inner());
        let m0 = fm.get(&0).unwrap();
        assert_eq!(m0.runs, 2);
        assert_eq!(m0.total_ns, 1500);
        let m1 = fm.get(&1).unwrap();
        assert_eq!(m1.runs, 1);
        assert_eq!(m1.total_ns, 200);
    }
    record_reinfer_event(3);
    {
        let cg = CALL_GRAPH_METRICS.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(cg.reinfer_events, baseline_reinfer_events + 3);
    }
}

#[test]
fn metrics_persist_and_load_round_trip() {
    reset_metrics_full();
    record_function_infer(2, 42);
    record_reinfer_event(1);
    force_persist_metrics();
    // Mutate in-memory then reload from disk to ensure overwrite
    {
        let mut fm = FUNCTION_METRICS.lock().unwrap_or_else(|e| e.into_inner());
        fm.clear();
    }
    load_metrics();
    {
        let fm = FUNCTION_METRICS.lock().unwrap_or_else(|e| e.into_inner());
        assert!(fm.get(&2).is_some());
    }
    // clean up metrics file to reduce test interference
    let _ = fs::remove_file(metrics_file_location());
}
