use cpal::traits::{DeviceTrait, HostTrait};
use spmc::*;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
mod error;
use error::OutputErr;

pub enum OutputMode {
    Device(String),
    DefaultDev,
    All,
}

/// selects the cpal input device to be used
/// then returns an spmc receiver that the clients
/// will list on to get packets that should be sent
/// ouot over the network
pub fn spawn_microphone() -> Receiver<Vec<u8>> {
    unimplemented!();
}

/// handles outputting audio, and decoding etc
pub fn play_audio(
    device: cpal::Device,
    recv: spmc::Receiver<Vec<u8>>,
) -> Result<(), OutputErr> {
    Ok(())
}

pub fn spawn_server(addr: SocketAddr, mode: OutputMode) -> Result<(), Box<dyn std::error::Error>> {
    let mut inbuf: [u8;65535*5];
    let mut socket = UdpSocket::bind(addr)?;
    let host = cpal::default_host();
    let (mut tx, rx) = channel();
    let mut handles = Vec::new();
    match mode {
        OutputMode::Device(name) => {
            let device = match host
                .output_devices()?
                .filter(|d| {
                    if let Ok(name) = d.name() {
                        return true;
                    } else {
                        return false;
                    }
                })
                .next()
            {
                Some(device) => device,
                None => return Err(Box::new(OutputErr::DeviceNotFound(name))),
            };
            let recv = rx.clone();
            handles.push(thread::spawn(move || {
                return play_audio(device, recv);
            }));
        }
        OutputMode::DefaultDev => {
            let device = match host.default_output_device() {
                Some(device) => device,
                None => return Err(Box::new(OutputErr::DeviceNotFound("default output device".to_string()))),
            };
            let recv = rx.clone();
            handles.push(thread::spawn(move || {
                return play_audio(device, recv);
            }));
        }
        OutputMode::All => {
            for device in host.output_devices()? {
                let recv = rx.clone();
                handles.push(thread::spawn(move || {
                    return play_audio(device, recv);
                }));
            }
        }
    }
    Ok(())
}
