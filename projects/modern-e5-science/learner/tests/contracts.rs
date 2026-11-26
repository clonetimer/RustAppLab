use e5_faultscope::*;
use e5_faultscope::synthetic::{extract, grouped_windows};
use std::collections::BTreeSet;

fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-8, "{a} != {b}");
}

#[test]
fn step1_window_stats_and_invalid_inputs() {
    let f = features(&[-2.0, 0.0, 2.0]).unwrap();
    near(f[0], 0.0);
    near(f[1], (8.0_f64 / 3.0).sqrt());
    let f = features(&[999_999.0, 1_000_000.0]).unwrap();
    near(f[0], 999_999.5);
    near(f[1], 0.5);
    assert_eq!(features(&[1.0]), Err(Error::Length));
    assert_eq!(features(&[0.0; MAX_ITEMS + 1]), Err(Error::Length));
    assert_eq!(features(&[0.0, f64::NAN]), Err(Error::NonFiniteOrRange));
    assert_eq!(features(&[0.0, 1_000_001.0]), Err(Error::NonFiniteOrRange));
}

#[test]
fn step2_train_only_scaler_and_constant_dimension() {
    let scaler = fit_scaler(&[[1.0, 4.0], [3.0, 4.0]]).unwrap();
    assert_eq!(scaler.transform([1.0, 4.0]).unwrap(), [-1.0, 0.0]);
    assert_eq!(scaler.transform([3.0, 4.0]).unwrap(), [1.0, 0.0]);
    assert_eq!(scaler.transform([5.0, 7.0]).unwrap(), [3.0, 3.0]);
    assert_eq!(scaler.transform([f64::INFINITY, 0.0]), Err(Error::NonFiniteOrRange));
    assert!(matches!(fit_scaler(&[[0.0, 0.0]]), Err(Error::Length)));
    assert!(matches!(fit_scaler(&[[0.0, 0.0], [f64::NAN, 2.0]]), Err(Error::NonFiniteOrRange)));
}

#[test]
fn step3_actual_learning_and_unseen_points() {
    let rows = [
        Example { x: [-3.0, 0.0], label: false },
        Example { x: [-1.0, 0.0], label: false },
        Example { x: [1.0, 0.0], label: true },
        Example { x: [3.0, 0.0], label: true },
    ];
    let fit = train(&rows, 400, 0.1).unwrap();
    near(fit.loss_before, 2.0_f64.ln());
    assert!(fit.loss_after < 0.2 && fit.loss_after < fit.loss_before);
    assert!(fit.model.probability([-2.0, 0.0]).unwrap() < 0.25);
    assert!(fit.model.probability([2.0, 0.0]).unwrap() > 0.75);
    let flipped: Vec<_> = rows.iter().map(|row| Example { x: row.x, label: !row.label }).collect();
    let flipped = train(&flipped, 400, 0.1).unwrap();
    assert!(flipped.model.probability([2.0, 0.0]).unwrap() < 0.25);
    assert!(matches!(train(&rows, 0, 0.1), Err(Error::Hyperparameter)));
    assert!(matches!(train(&rows, 1, f64::NAN), Err(Error::Hyperparameter)));
    assert!(matches!(train(&rows[..2], 1, 0.1), Err(Error::OneClass)));
}

#[test]
fn step4_confusion_and_boundary() {
    let result = evaluate(&[(true, 0.9), (false, 0.1), (false, 0.5), (true, 0.49)]).unwrap();
    assert_eq!(result, Confusion { tp: 1, tn: 1, fp: 1, fn_: 1 });
    assert_eq!(result.accuracy(), Some(0.5));
    assert_eq!(Confusion::default().accuracy(), None);
    assert_eq!(evaluate(&[]), Err(Error::Length));
    assert_eq!(evaluate(&[(true, f64::NAN)]), Err(Error::NonFiniteOrRange));
    assert_eq!(evaluate(&[(true, 1.001)]), Err(Error::NonFiniteOrRange));
}

#[test]
fn step5_grouped_holdout_and_transfer() {
    let (train_windows, test_windows) = grouped_windows();
    let train_groups: BTreeSet<_> = train_windows.iter().map(|x| x.group).collect();
    let test_groups: BTreeSet<_> = test_windows.iter().map(|x| x.group).collect();
    assert_eq!((train_groups.len(), test_groups.len()), (16, 8));
    assert!(train_groups.is_disjoint(&test_groups));
    let training = extract(&train_windows).unwrap();
    let testing = extract(&test_windows).unwrap();
    let fit = train(&training, 400, 0.1).unwrap();
    let predictions: Vec<_> = testing.iter().map(|row| (row.label, fit.model.probability(row.x).unwrap())).collect();
    let result = evaluate(&predictions).unwrap();
    assert_eq!(result.total(), 24);
    assert!(result.accuracy().unwrap() >= 0.9);
    assert!(fit.loss_after < fit.loss_before);
}
