# Audio Streaming Library

## History:
The main codes history is located in
https://github.com/divinusdracodominus/audiostreamer

## LICENSE:
MIT free to use for any purpose or any reason.

## How to build
```
cargo build --release
```

## how to run
```
./target/release/finalproject
```

### Arguments:
1. --local: local address to listen on including port
2. --remote: remote address to connect to including port
3. --file: optional arg to play wav file over network

## Implemented Features
1. Sending UDP packets
2. Microphone Capture (Through CPAL)
3. Speaker Playback (Through CPAL)
4. Sending data over UDP socket including defragmentation
5. Playing wav files over the network live.
6. Playing mic input over the network live.

## Planned Features
1. Embedable as an audio plugin, into alsa, jack, or pipewire.
2. Fully secured RSA + AES backed encryption. (please see github.com/divinusdracodominus/verifyudp
3. Interface for selecting input and output devices
4. Playing to multiple output devices at once/multiple networked machines at once (with a given time delay).
5. Compress data sent over the network.

## Troubles encountered in debugging
1. Forgot to reverse vector back to proper order. (resolved)
2. Reversed entire wav file. (resolved)
3. Incorrect sample rate. (48000 but wav file was 44100)
4. Too much fragmentation (sending packets of 2 bytes at a time) (resolved)
5. compression algorithim causing distortions (resolved)
6. feedback from mic.