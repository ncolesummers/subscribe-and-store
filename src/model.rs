// src/models.rs

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ArmMessage {
    pub timestamp: i64,
    pub matrices: Arm,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Arm {
    pub J1: Vec<Vec<f64>>,
    pub J2: Vec<Vec<f64>>,
    pub J3: Vec<Vec<f64>>,
    pub J4: Vec<Vec<f64>>,
    pub J5: Vec<Vec<f64>>,
    pub J6: Vec<Vec<f64>>,
    pub J7: Vec<Vec<f64>>,
    pub J8: Vec<Vec<f64>>,
    pub J9: Vec<Vec<f64>>,
    pub F1: Vec<Vec<f64>>,
    pub F2: Vec<Vec<f64>>,
    pub F3: Vec<Vec<f64>>,
    pub F4: Vec<Vec<f64>>,
    pub F5: Vec<Vec<f64>>,
    pub F6: Vec<Vec<f64>>,
}
