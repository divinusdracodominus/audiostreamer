# Audio Streaming Library

## Implemented Features
1. Sending UDP packets
2. Microphone Capture (Through CPAL)
3. Speaker Playback (Through CPAL)
4. Sending data over UDP socket including defragmentation

## Planned Features
1. Support for playing wav files, over the network.
2. Embedable as an audio plugin, into alsa, jack, or pipewire.
3. Fully secured RSA + AES backed encryption. (please see github.com/divinusdracodominus/verifyudp

## Troubles encountered in debugging
1. Forgot to reverse vector back to proper order
2. Reversed entire wav file
3. Incorrect sample rate. (48000 but wav file was 44100)
4. Too much fragmentation (sending packets of 2 bytes at a time)