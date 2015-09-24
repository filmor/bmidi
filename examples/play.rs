//!
//!  test.rs.rs
//!
//!  Created by Mitchell Nordine at 05:57PM on December 19, 2014.
//!
//!  Always remember to run high performance Rust code with the --release flag. `Synth` 
//!

// extern crate dsp;
extern crate synth;
extern crate time_calc as time;
extern crate dsp;
extern crate midi;

use dsp::{Node, SoundStream, StreamParams};
use synth::Synth;
use midi::*;

// Currently supports i8, i32, f32.
pub type AudioSample = f32;
pub type Input = AudioSample;
pub type Output = AudioSample;

fn main() {

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
    let mut timer: i64 = 0;

    let res = File::parse("test.mid".as_ref());

    let mut track = res.track_iter(1);

    let mut stream = SoundStream::new()
        .frames_per_buffer(256)
        .output::<f32>(StreamParams::new())
        .run()
        .unwrap();

    for event in stream.by_ref() {
        let dsp::output::Event(output, settings) = event;

        // TODO: Split up the buffer according to the next few events and
        //       request audio in multiple steps, filling up the whole buffer
        synth.audio_requested(output, settings);

        let dt = settings.frames as f32 / settings.sample_hz as f32;

        let time_slice = (dt * 1000.) as i64;
        timer -= time_slice;

        // Advance iterator
        while timer <= 0 {
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
    }
}
