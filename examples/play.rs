//!
//!  play.rs
//!
//!  Based on synth/examples/test.rs by Mitchell Nordine
//!

// extern crate dsp;
extern crate synth;
extern crate time_calc;
extern crate dsp;
extern crate midi;

use dsp::{Node, SoundStream, StreamParams, Settings};
use time_calc::{Bpm, Ppqn, Ticks};
use synth::Synth;
use midi::{File, EventType, KeyEventType};
use std::cmp;

// Currently supports i8, i32, f32.
pub type AudioSample = f32;
pub type Input = AudioSample;
pub type Output = AudioSample;

fn main() {
    let channel = 0;
    let track = 1;

    // Construct our fancy Synth!
    let mut synth = {
        use synth::{Point, Oscillator, mode, oscillator, Envelope};

        let amp_env = Envelope::from(vec!(
            //         Time ,  Amp ,  Curve
            Point::new(0.0  ,  0.0 ,  0.0),
            Point::new(0.01 ,  1.0 ,  0.0),
            Point::new(0.45 ,  1.0 ,  0.0),
            Point::new(0.81 ,  0.8 ,  0.0),
            Point::new(1.0  ,  0.0 ,  0.0),
        ));

        // Now we can create our oscillator from our envelopes.
        // There are also Sine, Noise, NoiseWalk, SawExp and Square waveforms.
        let oscillator = Oscillator::new(oscillator::waveform::Sine, amp_env, 55., ());

        // Here we construct our Synth from our oscillator.
        Synth::new(mode::Poly, ())
            .oscillator(oscillator) // Add as many different oscillators as desired.
            .fade(50.0, 300.0) // Attack and Release in milliseconds.
            .num_voices(16) // By default Synth is monophonic but this gives it `n` voice polyphony.
            .volume(0.20)
            .spread(0.1)
    };

    // We'll use this to keep track of time and break from the loop after 6 seconds.
    let res = File::parse("test.mid".as_ref());
    let mut track = res.track_iter(track).peekable();

    let mut stream = SoundStream::new()
        .frames_per_buffer(256)
        .output::<f32>(StreamParams::new())
        .run()
        .unwrap();

    let ppqn = res.division as Ppqn;
    let mut bpm = 120 as Bpm;

    let midi_tempo_to_bpm = |tempo| {
        // tempo is µs / beat (mus = 10^-6, min = 6 * 10^1 => min / mus = 6 * 10^7)
        // => bpm = (6 * 10^7) / tempo
        (6e7 / tempo) as Bpm
    };

    bpm = midi_tempo_to_bpm(6e5);

    // How many frames do we still have to write with the current state?
    let mut cursor = 0 as i64;

    'outer: for event in stream.by_ref() {
        let dsp::output::Event(output, settings) = event;

        let mut inner_cursor = 0;

        while inner_cursor < settings.frames {
            if cursor <= 0 {
                let evt = track.next().unwrap();

                if evt.channel == channel {
                    if let EventType::Key{ typ, note, velocity } = evt.typ {
                        // println!("Key {:?} {:?} {}", typ, note, velocity);
                        match typ {
                            KeyEventType::Press => {
                                synth.note_on(note, velocity as f32 / 256f32);
                            },
                            KeyEventType::Release => {
                                synth.note_off(note);
                            }
                            _ => {}
                        }
                    }
                    else {
                        println!("Ignored event {:?}", evt);
                    }
                }
                else {
                    println!("Ignored event {:?}", evt);
                }

                if let Some(next_evt) = track.peek() {
                    // TODO Modify bpm using SetTempo events, for that we need
                    //      to iterate over all tracks at once (FF 51 03 + 24bit,
                    //      microseconds per quarter node)
                    let skip = Ticks(next_evt.delay as i64)
                        .samples(bpm, ppqn, settings.sample_hz as f64)
                        as u16;

                    let time = Ticks(next_evt.delay as i64).ms(bpm, ppqn);
                    println!("Event Length: {} ms, {} samples", time, skip);

                    cursor += skip as i64;
                }
                else {
                    break 'outer;
                }
            }

            let new_inner_cursor = cmp::min(
                inner_cursor as i64 + cursor,
                settings.frames as i64) as u16;

            let (begin, end) = (inner_cursor, new_inner_cursor);

            let new_settings = Settings {
                sample_hz: settings.sample_hz,
                channels: settings.channels,
                frames: end - begin
            };

            let new_output = &mut output[
                (begin * settings.channels) as usize
                ..(end * settings.channels) as usize ];

            synth.audio_requested(new_output, new_settings);

            cursor -= (new_inner_cursor - inner_cursor) as i64;
            inner_cursor = new_inner_cursor;
        }
    }
}
