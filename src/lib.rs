// SPDX-License-Identifier: MIT
/*
 * Sick mate sick subwoofer
 *
 * Bass and sub-bass frequencies are poorly reproduced by microspeakers. Luckily,
 * the human brain is malleable and will reconstruct these frequencies from their
 * harmonics as if they were really there. We can use this phenomenon to get better
 * bass performance out of crappy speakers.
 *
 * Copyright (C) 2023 James Calligeros <jcalligeros99@gmail.com>
 */

use lv2::prelude::*;
use biquad::*;

/*
 * Input and output ports used by the plugin
 *
 *
 * Ports:
 *      in_l: left channel input
 *      in_r: right channel input
 *      out_l: left channel output
 *      out_r: right channel output
 *      floor: floor frequency for harmonics
 *      ceil: ceiling frequency for harmonics
 *      amt: volume of harmonics to mix in to the output
 *      bypass: bypass all filtering
 *      harms: number of harmonics per root
 */
#[derive(PortCollection)]
struct Ports {
    in_l: InputPort<Audio>,
    in_r: InputPort<Audio>,
    out_l: OutputPort<Audio>,
    out_r: OutputPort<Audio>,
    floor: InputPort<Control>,
    ceil: InputPort<Control>,
    amt: InputPort<Control>,
    bypass: InputPort<Control>,
    saturation: InputPort<Control>,
    blend: InputPort<Control>,
    harmonics: InputPort<Control>
}

/*
 * Distortion/saturation module
 *
 * Built like this to avoid refactoring when we do this properly
 */
struct Distortion {
    gain: f32,
    blend: f32,
    harmonics: f32
}

trait Saturator {
    fn new() -> Self;
    fn update_params(&mut self, ports: &Ports);
    fn process(&mut self, sample: f32) -> f32;
}

impl Saturator for Distortion {
    fn new() -> Self {
        Self {
            gain: 0f32,
            blend: 0f32, // TODO
            harmonics: 0f32 // TODO
        }
    }

    fn update_params(&mut self, ports: &Ports) {
        if self.gain != *ports.saturation {
            self.gain = *ports.saturation;
        }

        if self.blend != *ports.blend {
            self.blend = *ports.blend;
        }

        if self.harmonics != *ports.harmonics {
            self.harmonics = *ports.harmonics
        }

        // TODO: other params
    }

    fn process(&mut self, sample: f32) -> f32 {
        (sample * self.gain).tanh() // TODO: make this good
    }
}


/*
 * Plugin state
 *
 * Members:
 *      low_pass: chain of low-pass biquads
 *      high_pass: chain of high-pass biquads
 *      floor_curr: currently set floor frequency
 *      ceil_curr: currently set ceiling frequency
 *      amt_curr: currently set volume
 *      harms_curr: currently set harmonics
 *      sample_rate: sample rate at time of instantiation
 *
 * TODO: harmonic distortion algorithm
 */
#[uri("https://chadmed.au/bankstown")]
struct Subwoofer {
    floor_curr: f32,
    ceil_curr: f32,
    amt_curr: f32,
    sample_rate: f32,
    sat: Distortion,
    low_pass: Vec<DirectForm2Transposed::<f32>>,
    high_pass: Vec<DirectForm2Transposed::<f32>>
}

/*
 * Build the arrays of high and low pass filters we need.
 *
 */
fn build_lpfs(fc: f32, rate: f32) -> Vec<DirectForm2Transposed::<f32>> {
    let lp_coeff = Coefficients::<f32>::from_params(Type::LowPass, rate.hz(), fc.hz(), Q_BUTTERWORTH_F32)
                                        .unwrap();

    let filters: Vec<DirectForm2Transposed::<f32>> = vec![DirectForm2Transposed::<f32>::new(lp_coeff); 2];

    filters
}

fn build_hpfs(fc: f32, rate: f32) -> Vec<DirectForm2Transposed::<f32>> {
    let hp_coeff = Coefficients::<f32>::from_params(Type::HighPass, rate.hz(), fc.hz(), Q_BUTTERWORTH_F32)
                                        .unwrap();

    let filters: Vec<DirectForm2Transposed::<f32>> = vec![DirectForm2Transposed::<f32>::new(hp_coeff); 2];

    filters
}

/*
 * Extend the Plugin trait so that we can modularly update the parameters of
 * the plugin IFF they have changed.
 */
trait BassEx: Plugin {
    fn update_params(&mut self, ports: &mut Ports);
}

impl Plugin for Subwoofer {
    type Ports = Ports;

    type InitFeatures = ();
    type AudioFeatures = ();

    fn new(info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        Some(Self {
            floor_curr: 0f32,
            ceil_curr: 0f32,
            amt_curr: 0f32,
            sample_rate: info.sample_rate() as f32,
            sat: Saturator::new(),
            low_pass: build_lpfs(20f32, info.sample_rate() as f32),
            high_pass: build_hpfs(250f32, info.sample_rate() as f32)
        })
    }

    fn run(&mut self, ports: &mut Ports, _features: &mut (), _: u32) {
        BassEx::update_params(self, ports);
        self.sat.update_params(ports);
        if *ports.bypass == 1.0 {
            for (inl, outl) in Iterator::zip(ports.in_l.iter(), ports.out_l.iter_mut()) {
                *outl = inl * 1.0;
            }
            for (inr, outr) in Iterator::zip(ports.in_r.iter(), ports.out_r.iter_mut()) {
                *outr = inr * 1.0;
            }
        } else { // TODO: implement distortion part of pipeline
            for (inl, outl) in Iterator::zip(ports.in_l.iter(), ports.out_l.iter_mut()) {
                // Band-pass on the processed sample
                let mut processed: f32 = self.low_pass[0].run(*inl);

                processed = self.high_pass[0].run(self.sat.process(processed));

                // Sum back with input signal
                *outl = (processed * self.amt_curr) + inl;
            }
            for (inr, outr) in Iterator::zip(ports.in_r.iter(), ports.out_r.iter_mut()) {
                // Band-pass on the processed sample
                let mut processed: f32 = self.low_pass[1].run(*inr);

                processed = self.high_pass[1].run(self.sat.process(processed));

                // Sum back with input signal
                *outr = (processed * self.amt_curr) + inr;
            }
        }
    }
}

// TODO: change params in-place
impl BassEx for Subwoofer {
    fn update_params(&mut self, ports: &mut Ports) {
        if self.floor_curr != *ports.floor {
            self.high_pass = build_hpfs(*ports.floor, self.sample_rate);
            self.floor_curr = *ports.floor;
        }

        if self.ceil_curr != *ports.ceil {
            self.low_pass = build_lpfs(*ports.ceil, self.sample_rate);
            self.ceil_curr = *ports.ceil;
        }

        if self.amt_curr != *ports.amt {
            self.amt_curr = *ports.amt;
        }
    }
}

lv2_descriptors!(Subwoofer);
