/// Defines a decoder for incoming messages
pub mod decoder;

/// Defines the type to represent incoming messages
pub mod envelope;

/// Implements shared methods that work on underlying Lapin delivery
pub mod delivery;

/// Defines the inbound transporting mechanism
pub mod subscriber;
