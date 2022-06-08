#![feature(vec_into_raw_parts)]
#![feature(unix_socket_abstract)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleRate;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::os::unix::net::UnixDatagram;
use std::path::{PathBuf, Path};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

pub fn compress(value: i16) -> i16 {
    let scalar: f32 = 3276.7;
    if value < 0 {
        -1 * (f32::log10(value as f32 * -1.0) * scalar) as i16
    } else {
        (f32::log10(value as f32) * scalar) as i16
    }
}

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    file: Option<PathBuf>,
    /// remote peer to connect to
    #[structopt(short, long)]
    remote: SocketAddr,
    /// local address to bind to
    #[structopt(short, long)]
    local: SocketAddr,
    //#[structopt(short, long)]
    //mode: String,
}
// this will be a stupid simple implementation
fn main() {
    let args = Args::from_args();
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let mut server = UdpSocket::bind(&args.local).unwrap();
    let client = UdpSocket::bind("0.0.0.0:3232").unwrap();
    // used for testing purposes only 
    //let mut u_server =
        UnixDatagram::bind(std::env::current_dir().unwrap().join("unixsocket.socket")).unwrap();
    //let u_client = UnixDatagram::bind(std::env::current_dir().unwrap().join("unixclient.socket")).unwrap();
    let host = cpal::default_host();
    let output = host.default_output_device().unwrap();
    let input = host.default_input_device().unwrap();
    let mut packet_num: u32 = 1;

    let output_config = output
        .supported_output_configs()
        .expect("failed to get output configs")
        .map(|c| c.with_sample_rate(SampleRate(44100)))
        .next()
        .unwrap();
    let input_config = input
        .supported_input_configs()
        .expect("failed to get input configs")
        .map(|c| c.with_sample_rate(SampleRate(48000)))
        .next()
        .unwrap();

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let addr = args.remote.clone(); //SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 218, 219, 149)), 6432);

    let mut writer = Arc::new(Mutex::new(
        hound::WavWriter::create("sine.wav", spec).unwrap(),
    ));
    let mut sec_writer = writer.clone();
    let mut playing = Arc::new(AtomicBool::new(false));
    let mut arc_play = playing.clone();

    let cloned_buf = buffer.clone();
    //let mut in_stream = None;

    if let Some(file) = args.file {
        std::thread::spawn(move || {
            let mut wav_reader = hound::WavReader::open(&file).unwrap();
            println!("spec: {:?}", wav_reader.spec());
            let mut vecs: Vec<Vec<u8>> = Vec::new();
            let mut current_vec: Vec<i16> = Vec::new();
            let mut count = 0;
            for (idx, sample) in wav_reader.samples::<i16>().enumerate() {
                current_vec.push(sample.unwrap());
                if count >= (1_400) {
                    let outvec: Vec<u8> = unsafe {
                        let len = current_vec.len();
                        let (ptr, len, cap) = current_vec
                            .drain(0..len)
                            .collect::<Vec<i16>>()
                            .into_raw_parts();
                        Vec::from_raw_parts(ptr as *mut u8, len * 2, cap * 2)
                    };
                    vecs.push(outvec);
                    count = 0;
                }
                count += 1;
            }
            for mut vec in vecs.into_iter() {
                send_packet(
                    &mut packet_num,
                    addr,
                    &client,
                    &mut vec,
                );
            }
        });
    } else {
        let stream = input
        .build_input_stream(
            &input_config.config(),
            move |data: &[i16], _| unsafe {
                let vec = data.to_vec();
                let (ptr, len, cap) = vec.into_raw_parts();
                let mut outvec: Vec<u8> = Vec::from_raw_parts(ptr as *mut u8, len * 2, cap * 2);
                println!("about to send packet");
                send_packet(&mut packet_num, addr, &client, &mut outvec);
            },
            move |err| {
                panic!("{}", err);
            },
        )
        .unwrap();
        stream.play();
        std::mem::forget(stream);  
    }
    let out_stream = output
        .build_output_stream(
            &output_config.config(),
            move |data: &mut [i16], _: &_| {
                let mut buf = cloned_buf.lock().unwrap();
                let mut file = writer.lock().unwrap();

                //if arc_play.load(Ordering::Acquire) {
                println!("buffer length: {}", buf.len());
                let indata = if buf.len() < data.len() {
                    let len = buf.len();
                    let mut newbuf = buf.drain(0..len).collect::<Vec<i16>>();
                    let mut idx = buf.len();
                    /*while idx < data.len() {
                        newbuf.push(0);
                        idx += 1;
                    }*/
                    newbuf
                } else {
                    let start = 0;
                    let end = data.len();
                    buf.drain(start..end).collect::<Vec<i16>>()
                };

                for (mut idx, val) in indata.iter().enumerate() {
                    if idx > data.len() - 1 {
                        idx = data.len() - 1;
                    }
                    data[idx] = *val;
                    file.write_sample(*val).unwrap();
                }
                file.flush().unwrap();
                //}
                std::mem::drop(buf);
            },
            move |err| {
                panic!("{}", err);
            },
        )
        .unwrap();

    out_stream.pause().unwrap();
    let mut play = false;
    loop {
        let mut readdata = recv_data(&mut server);
        //println!("readdata length: {}", readdata.len());
        let (_length, num, time_val) = decode(&mut readdata);
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let indata: Vec<i16> = unsafe {
            let (ptr, len, cap) = readdata.into_raw_parts();
            Vec::from_raw_parts(ptr as *mut i16, len / 2, cap / 2)
        };
        let mut buf = buffer.lock().unwrap();
        buf.extend_from_slice(&indata);
        // these two conditionals are doing nothing
        // thus the program doesn't wait to have an initial
        // buffer before playing audio
        /*if buf.len() >= 9600 && !play {
            playing.store(true, Ordering::Release);
            play = true;
            out_stream.play().unwrap();
        }
        if buf.len() < 9600 && play {
            playing.store(true, Ordering::Release);
            play = false;
            out_stream.pause().unwrap();
        }*/
        std::mem::drop(buf);
    }
}

// returns (length, packet_number) draining
// them from the vector
fn decode(data: &mut Vec<u8>) -> (u16, u32, u128) {
    let size = std::mem::size_of::<u128>();
    let time_val: u128 = u128::from_le_bytes(data[0..size].try_into().unwrap());
    let length: u16 = u16::from_le_bytes(data[size..size + 2].try_into().unwrap());
    let packet_num: u32 = u32::from_le_bytes(data[size + 2..size + 6].try_into().unwrap());

    data.drain(0..(6 + size));
    data.drain((length as usize)..data.len());
    data.reverse();
    (length, packet_num, time_val)
}

fn send_packet(packet_num: &mut u32, addr: SocketAddr, socket: &UdpSocket, data: &mut Vec<u8>) {
    println!("sending packet of length: {}", data.len());
    let mut index = 0;
    let len = data.len();
    while index < len {
        let pack_num = packet_num.to_be_bytes();
        let end = if 48_000 > data.len() {
            data.len()
        } else {
            48_000
        };
        let l = end as u16;

        let mut send_data = data.drain(0..end).collect::<Vec<u8>>();
        let start = SystemTime::now();
        let since_the_epoch: u128 = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        send_data.extend_from_slice(&pack_num);
        send_data.extend_from_slice(&l.to_be_bytes());
        send_data.extend_from_slice(&since_the_epoch.to_be_bytes());
        // so the header is at the front of the vector
        send_data.reverse();
        index += end;

        println!("packet num: {} sent at: {:?}", packet_num, since_the_epoch);
        socket.send_to(&send_data.as_slice(), addr).unwrap();

        *packet_num = (*packet_num) + 1;
    }
}

fn u_send_packet(packet_num: &mut u32, addr: &Path, socket: &UnixDatagram, data: &mut Vec<u8>) {
    println!("sending packet of length: {}", data.len());
    let mut index = 0;
    let len = data.len();
    while index < len {
        let pack_num = packet_num.to_be_bytes();
        let end = if 48_000 > data.len() {
            data.len()
        } else {
            48_000
        };
        let l = end as u16;

        let mut send_data = data.drain(0..end).collect::<Vec<u8>>();
        let start = SystemTime::now();
        let since_the_epoch: u128 = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        send_data.extend_from_slice(&pack_num);
        send_data.extend_from_slice(&l.to_be_bytes());
        send_data.extend_from_slice(&since_the_epoch.to_be_bytes());
        // so the header is at the front of the vector
        send_data.reverse();
        index += end;

        println!("packet num: {} sent at: {:?}", packet_num, since_the_epoch);
        socket.send_to(&send_data.as_slice(), addr).unwrap();

        *packet_num = (*packet_num) + 1;
    }
}

fn recv_data(socket: &mut UdpSocket) -> Vec<u8> {
    let mut buf: [u8; 48006] = [0; 48006];
    let (size, target) = socket.recv_from(&mut buf).unwrap();
    buf.to_vec()
}

fn u_recv_data(socket: &mut UnixDatagram) -> Vec<u8> {
    let mut buf: [u8; 48006] = [0; 48006];
    let (size, target) = socket.recv_from(&mut buf).unwrap();
    buf.to_vec()
}
