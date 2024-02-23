const TOPICS: &[&str] = &["left_arm", "right_arm"];

// QoS 0 for at most once delivery.
// Lower QoS levels are faster and have less overhead, which
// Since this is a hot path, we use QoS 0 for the fastest delivery.
const QOS: &[i32] = &[0, 0];

pub mod mqtt {
    pub fn connect() {
        // MQTT connect logic here
    }

    pub fn subscribe(topic: &str) {
        // MQTT subscribe logic here
    }

    // Other MQTT related functions
}
