//! Network condition simulation for testing

use std::time::Duration;

/// Simulated network conditions for testing
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkConditions {
    /// Network latency in milliseconds
    pub latency_ms: u32,
    /// Jitter in milliseconds (variation in latency)
    pub jitter_ms: u32,
    /// Packet loss percentage (0-100)
    pub packet_loss_percent: f32,
    /// Bandwidth in kilobits per second
    pub bandwidth_kbps: u32,
    /// Whether the network is currently available
    pub available: bool,
}

impl Default for NetworkConditions {
    fn default() -> Self {
        Self {
            latency_ms: 50,
            jitter_ms: 5,
            packet_loss_percent: 0.0,
            bandwidth_kbps: 1000,
            available: true,
        }
    }
}

impl NetworkConditions {
    /// Perfect network conditions (ideal for testing)
    pub fn perfect() -> Self {
        Self {
            latency_ms: 1,
            jitter_ms: 0,
            packet_loss_percent: 0.0,
            bandwidth_kbps: 10000, // 10 Mbps
            available: true,
        }
    }

    /// Good network conditions (typical broadband)
    pub fn good() -> Self {
        Self {
            latency_ms: 20,
            jitter_ms: 2,
            packet_loss_percent: 0.1,
            bandwidth_kbps: 5000, // 5 Mbps
            available: true,
        }
    }

    /// Mobile network conditions (4G/LTE)
    pub fn mobile() -> Self {
        Self {
            latency_ms: 100,
            jitter_ms: 20,
            packet_loss_percent: 1.0,
            bandwidth_kbps: 2000, // 2 Mbps
            available: true,
        }
    }

    /// Poor network conditions (congested or unreliable)
    pub fn poor() -> Self {
        Self {
            latency_ms: 300,
            jitter_ms: 50,
            packet_loss_percent: 5.0,
            bandwidth_kbps: 500, // 0.5 Mbps
            available: true,
        }
    }

    /// Unreliable network conditions (high packet loss)
    pub fn unreliable() -> Self {
        Self {
            latency_ms: 150,
            jitter_ms: 100,
            packet_loss_percent: 15.0,
            bandwidth_kbps: 1000, // 1 Mbps
            available: true,
        }
    }

    /// Intermittent connection (comes and goes)
    pub fn intermittent() -> Self {
        Self {
            latency_ms: 200,
            jitter_ms: 30,
            packet_loss_percent: 2.0,
            bandwidth_kbps: 1500, // 1.5 Mbps
            available: false, // Start unavailable
        }
    }

    /// No network connectivity
    pub fn offline() -> Self {
        Self {
            latency_ms: 0,
            jitter_ms: 0,
            packet_loss_percent: 100.0,
            bandwidth_kbps: 0,
            available: false,
        }
    }

    /// Calculate expected round-trip time based on conditions
    pub fn expected_rtt(&self) -> Duration {
        let base_rtt = self.latency_ms * 2; // Round trip
        let jitter_factor = self.jitter_ms as f32 * 0.5; // Half jitter on average
        Duration::from_millis((base_rtt as f32 + jitter_factor) as u64)
    }

    /// Calculate throughput for a given packet size
    pub fn throughput_for_packet(&self, packet_size_bytes: usize) -> Duration {
        if self.bandwidth_kbps == 0 || !self.available {
            return Duration::from_secs(10); // Very slow for no bandwidth
        }
        
        let bits_per_packet = packet_size_bytes as f32 * 8.0;
        let bits_per_second = self.bandwidth_kbps as f32 * 1000.0;
        let seconds_per_packet = bits_per_packet / bits_per_second;
        
        Duration::from_secs_f32(seconds_per_packet)
    }

    /// Check if conditions are suitable for real-time communication
    pub fn is_suitable_for_realtime(&self) -> bool {
        self.available
            && self.latency_ms < 200
            && self.packet_loss_percent < 5.0
            && self.bandwidth_kbps >= 500
    }

    /// Check if conditions are suitable for video calling
    pub fn is_suitable_for_video(&self) -> bool {
        self.available
            && self.latency_ms < 150
            && self.packet_loss_percent < 2.0
            && self.bandwidth_kbps >= 2000
    }

    /// Check if conditions are suitable for audio only
    pub fn is_suitable_for_audio(&self) -> bool {
        self.available
            && self.latency_ms < 300
            && self.packet_loss_percent < 10.0
            && self.bandwidth_kbps >= 100
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        if !self.available {
            "Offline"
        } else if self.packet_loss_percent > 10.0 {
            "Very Poor"
        } else if self.packet_loss_percent > 5.0 {
            "Poor"
        } else if self.latency_ms > 200 {
            "Poor (High Latency)"
        } else if self.latency_ms > 100 {
            "Fair"
        } else if self.latency_ms > 50 {
            "Good"
        } else if self.packet_loss_percent > 1.0 {
            "Good (Some Loss)"
        } else {
            "Excellent"
        }
    }

    /// Apply random variations to simulate network fluctuations
    pub fn with_variation(&self) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Self {
            latency_ms: (self.latency_ms as f32 * (0.8 + rng.gen::<f32>() * 0.4)) as u32,
            jitter_ms: self.jitter_ms,
            packet_loss_percent: (self.packet_loss_percent * (0.5 + rng.gen::<f32>() * 1.5)).min(100.0),
            bandwidth_kbps: (self.bandwidth_kbps as f32 * (0.7 + rng.gen::<f32>() * 0.6)) as u32,
            available: self.available && rng.gen::<f32>() > 0.05, // 5% chance of outage
        }
    }
}

/// Network scenario presets for comprehensive testing
#[derive(Debug, Clone)]
pub enum NetworkScenario {
    /// Perfect conditions for baseline testing
    Perfect,
    /// Typical home broadband
    HomeBroadband,
    /// Office network with moderate congestion
    OfficeNetwork,
    /// Mobile 4G/LTE connection
    Mobile4G,
    /// Mobile 3G connection
    Mobile3G,
    /// Satellite internet (high latency)
    Satellite,
    /// Congested network
    Congested,
    /// Intermittent connectivity
    Intermittent,
    /// Complete outage
    Outage,
}

impl NetworkScenario {
    /// Get network conditions for this scenario
    pub fn conditions(&self) -> NetworkConditions {
        match self {
            NetworkScenario::Perfect => NetworkConditions::perfect(),
            NetworkScenario::HomeBroadband => NetworkConditions::good(),
            NetworkScenario::OfficeNetwork => NetworkConditions {
                latency_ms: 80,
                jitter_ms: 10,
                packet_loss_percent: 0.5,
                bandwidth_kbps: 3000,
                available: true,
            },
            NetworkScenario::Mobile4G => NetworkConditions::mobile(),
            NetworkScenario::Mobile3G => NetworkConditions {
                latency_ms: 300,
                jitter_ms: 50,
                packet_loss_percent: 3.0,
                bandwidth_kbps: 500,
                available: true,
            },
            NetworkScenario::Satellite => NetworkConditions {
                latency_ms: 600,
                jitter_ms: 20,
                packet_loss_percent: 1.0,
                bandwidth_kbps: 2000,
                available: true,
            },
            NetworkScenario::Congested => NetworkConditions::poor(),
            NetworkScenario::Intermittent => NetworkConditions::intermittent(),
            NetworkScenario::Outage => NetworkConditions::offline(),
        }
    }

    /// Get all scenarios for comprehensive testing
    pub fn all() -> Vec<Self> {
        vec![
            NetworkScenario::Perfect,
            NetworkScenario::HomeBroadband,
            NetworkScenario::OfficeNetwork,
            NetworkScenario::Mobile4G,
            NetworkScenario::Mobile3G,
            NetworkScenario::Satellite,
            NetworkScenario::Congested,
            NetworkScenario::Intermittent,
            NetworkScenario::Outage,
        ]
    }

    /// Get realistic scenarios (excluding perfect and outage)
    pub fn realistic() -> Vec<Self> {
        vec![
            NetworkScenario::HomeBroadband,
            NetworkScenario::OfficeNetwork,
            NetworkScenario::Mobile4G,
            NetworkScenario::Mobile3G,
            NetworkScenario::Satellite,
            NetworkScenario::Congested,
            NetworkScenario::Intermittent,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_conditions_descriptions() {
        assert_eq!(NetworkConditions::perfect().description(), "Excellent");
        assert_eq!(NetworkConditions::offline().description(), "Offline");
        assert_eq!(NetworkConditions::poor().description(), "Poor");
    }

    #[test]
    fn test_suitability_checks() {
        let perfect = NetworkConditions::perfect();
        assert!(perfect.is_suitable_for_realtime());
        assert!(perfect.is_suitable_for_video());
        assert!(perfect.is_suitable_for_audio());

        let poor = NetworkConditions::poor();
        assert!(!poor.is_suitable_for_realtime());
        assert!(!poor.is_suitable_for_video());
        assert!(!poor.is_suitable_for_audio());

        let mobile = NetworkConditions::mobile();
        assert!(mobile.is_suitable_for_realtime());
        assert!(!mobile.is_suitable_for_video());
        assert!(mobile.is_suitable_for_audio());
    }

    #[test]
    fn test_throughput_calculation() {
        let conditions = NetworkConditions {
            bandwidth_kbps: 1000, // 1 Mbps
            ..Default::default()
        };

        let packet_1kb = conditions.throughput_for_packet(1024);
        let expected = Duration::from_millis(8); // 1024 * 8 / 1,000,000 seconds
        assert!(packet_1kb.as_millis() <= expected.as_millis() + 1);
    }

    #[test]
    fn test_network_scenarios() {
        let scenarios = NetworkScenario::all();
        assert_eq!(scenarios.len(), 9);

        let realistic = NetworkScenario::realistic();
        assert_eq!(realistic.len(), 7); // Excludes perfect and outage
    }

    #[test]
    fn test_variation() {
        let original = NetworkConditions::perfect();
        let varied = original.with_variation();
        
        // Should be different but still reasonable
        assert_ne!(original, varied);
        assert!(varied.available); // Should still be available most of the time
    }
}