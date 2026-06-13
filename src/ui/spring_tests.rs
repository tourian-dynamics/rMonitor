use super::*;

#[test]
fn test_spring_convergence() {
    let mut spring = Spring::new(120.0, 10.0);
    spring.target = 100.0;
    
    let dt = 0.01;
    for _ in 0..500 {
        spring.update(dt);
    }
    
    assert!((spring.value - 100.0).abs() < 0.1);
}

#[test]
fn test_spring_underdamped_overshoot() {
    let mut spring = Spring::new(100.0, 5.0);
    spring.target = 1.0;
    
    let mut overshoot = false;
    let dt = 0.01;
    for _ in 0..200 {
        spring.update(dt);
        if spring.value > 1.0 {
            overshoot = true;
            break;
        }
    }
    assert!(overshoot, "Underdamped spring did not overshoot the target.");
}

#[test]
fn test_spring_default_new() {
    let spring = Spring::new(80.0, 12.0);
    assert_eq!(spring.value, 0.0);
    assert_eq!(spring.velocity, 0.0);
    assert_eq!(spring.target, 0.0);
    assert_eq!(spring.tension, 80.0);
    assert_eq!(spring.damping, 12.0);
}

#[test]
fn test_spring_zero_tension() {
    let mut spring = Spring::new(0.0, 5.0);
    spring.value = 10.0;
    spring.velocity = 20.0;
    spring.target = 100.0; // Tension is 0, so target doesn't attract
    
    let dt = 0.01;
    spring.update(dt);
    // Force is -damping * velocity = -5.0 * 20.0 = -100.0
    // new velocity = 20.0 + (-100.0 * 0.01) = 19.0
    // new value = 10.0 + (19.0 * 0.01) = 10.19
    assert!((spring.velocity - 19.0).abs() < 1e-9);
    assert!((spring.value - 10.19).abs() < 1e-9);
}

#[test]
fn test_spring_zero_damping() {
    let mut spring = Spring::new(100.0, 0.0);
    spring.target = 10.0;
    
    let dt = 0.001;
    let mut went_above = false;
    let mut went_below = false;
    
    // Without damping, it should oscillate around the target (10.0)
    for _ in 0..1000 {
        spring.update(dt);
        if spring.value > 10.0 {
            went_above = true;
        }
        if went_above && spring.value < 10.0 {
            went_below = true;
        }
    }
    assert!(went_above && went_below, "Undamped spring did not oscillate around target.");
}

#[test]
fn test_spring_overdamped_no_overshoot() {
    let mut spring = Spring::new(10.0, 50.0);
    spring.target = 10.0;
    
    let dt = 0.01;
    let mut overshoot = false;
    for _ in 0..1000 {
        spring.update(dt);
        if spring.value > 10.0 {
            overshoot = true;
            break;
        }
    }
    assert!(!overshoot, "Overdamped spring overshot the target.");
    assert!(spring.value > 5.0, "Spring should have progressed towards target.");
}

#[test]
fn test_spring_negative_target() {
    let mut spring = Spring::new(100.0, 10.0);
    spring.target = -50.0;
    
    let dt = 0.01;
    for _ in 0..500 {
        spring.update(dt);
    }
    assert!((spring.value - (-50.0)).abs() < 0.1);
}

#[test]
fn test_spring_mid_flight_target_change() {
    let mut spring = Spring::new(100.0, 15.0);
    spring.target = 10.0;
    
    let dt = 0.01;
    for _ in 0..15 {
        spring.update(dt);
    }
    // Verify it is moving in positive direction
    assert!(spring.value > 0.0);
    assert!(spring.velocity > 0.0);
    
    // Change target
    spring.target = -10.0;
    for _ in 0..100 {
        spring.update(dt);
    }
    // It should now reverse and move towards -10.0
    assert!(spring.value < 5.0);
    
    for _ in 0..400 {
        spring.update(dt);
    }
    assert!((spring.value - (-10.0)).abs() < 0.1);
}

#[test]
fn test_spring_already_at_target() {
    let mut spring = Spring::new(100.0, 10.0);
    spring.value = 5.0;
    spring.target = 5.0;
    spring.velocity = 0.0;
    
    spring.update(0.01);
    assert_eq!(spring.value, 5.0);
    assert_eq!(spring.velocity, 0.0);
}

#[test]
fn test_spring_zero_dt() {
    let mut spring = Spring::new(100.0, 10.0);
    spring.target = 10.0;
    
    spring.update(0.0);
    assert_eq!(spring.value, 0.0);
    assert_eq!(spring.velocity, 0.0);
}
