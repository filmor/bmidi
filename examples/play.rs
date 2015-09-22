//!
//!  test.rs.rs
//!
//!  Created by Mitchell Nordine at 05:57PM on December 19, 2014.
//!
//!  Always remember to run high performance Rust code with the --release flag. `Synth` 
//!

// extern crate dsp;
extern crate pitch_calc as pitch;
extern crate synth;
extern crate time_calc as time;
extern crate dsp;
extern crate midi;

use dsp::{CallbackFlags, CallbackResult, Node, Sample, SoundStream, Settings, StreamParams};
use synth::Synth;
use midi::*;

// Currently supports i8, i32, f32.
pub type AudioSample = f32;
pub type Input = AudioSample;
pub type Output = AudioSample;

fn main() {

    // Construct our fancy Synth!
    let mut synth = {
        use synth::{Point, Oscillator, mode, oscillator};

        // The following envelopes should create a downward pitching sine wave that gradually quietens.
        // Try messing around with the points and adding some of your own!
        let amp_env = oscillator::AmpEnvelope::from_points(vec!(
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
            // .loop_points(0.49, 0.51) // Loop start and end points.
            .fade(100.0, 1000.0) // Attack and Release in milliseconds.
            .num_voices(16) // By default Synth is monophonic but this gives it `n` voice polyphony.
            .volume(0.20)
//            .detune(0.5)
            .spread(1.0)

        // Other methods include:
            // .loop_start(0.0)
            // .loop_end(1.0)
            // .attack(ms)
            // .release(ms)
            // .note_freq_generator(nfg)
            // .oscillators([oscA, oscB, oscC])
            // .volume(1.0)
    };

    // We'll use this to keep track of time and break from the loop after 6 seconds.
    let mut timer: i64 = 0;

    let res = File::parse("test.mid".as_ref());

    let mut track = res.track_iter(1);
    let mut index = 0usize;

    // The callback we'll use to pass to the Stream.
    let callback = Box::new(move |output: &mut[f32], settings: Settings, dt: f64, _: CallbackFlags| {
        Sample::zero_buffer(output);
        synth.audio_requested(output, settings);

        let time_slice = (dt * 1000.) as i64;
        timer -= time_slice;

        // Advance iterator
        while timer <= 0 {
            index += 1;
            let evt = track.next().unwrap();

            if evt.channel == 0 {
                match evt.typ {
                    EventType::Key{ typ, note, velocity } => {
                        println!("Key {:?} {:?} {}", typ, note, velocity);
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
                    _ => { println!("Ignored event {:?}", evt) }
                }
            }

            timer += (evt.delay as f64 * 0.6) as i64;
        }

        CallbackResult::Continue
    });

    // Construct the default, non-blocking output stream and run our callback.
    let stream = SoundStream::new().output(StreamParams::new()).run_callback(callback).unwrap();

    // Loop while the stream is active.
    while let Ok(true) = stream.is_active() {
        std::thread::sleep_ms(10);
    }
}
