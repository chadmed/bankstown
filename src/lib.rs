// SPDX-License-Identifier: MIT
/*
 * Bass and sub-bass frequencies are poorly reproduced by microspeakers. Luckily,
 * the human brain is malleable and will reconstruct these frequencies from their
 * harmonics as if they were really there. We can use this phenomenon to get better
 * bass performance out of crappy speakers.
 *
 * Copyright (C) 2023 James Calligeros <jcalligeros99@gmail.com>
 */

use std::f32::consts::E;
use std::f32::consts::PI;

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
 */
#[derive(PortCollection)]
struct Ports {
    in_l: InputPort<Audio>,
    in_r: InputPort<Audio>,
    out_l: OutputPort<Audio>,
    out_r: OutputPort<Audio>,
    bypass: InputPort<Control>,
    amt: InputPort<Control>,
    floor: InputPort<Control>,
    ceil: InputPort<Control>,
    sat_second: InputPort<Control>,
    sat_third: InputPort<Control>,
    blend: InputPort<Control>
}

/*
 * Distortion/saturation module
 *
 * Uses a modified Error function.
 */
struct Distortion {
    third_gain: f32,
    second_gain: f32,
    coeff: f32,
}

trait Saturator {
    fn new() -> Self;
    fn update_params(&mut self, ports: &Ports);
    fn process(&mut self, sample: f32) -> f32;
}

impl Saturator for Distortion {
    fn new() -> Self {
        Self {
            third_gain: 0f32,
            second_gain: 0f32,
            coeff: 0f32,
        }
    }

    fn update_params(&mut self, ports: &Ports) {
        if self.third_gain != *ports.sat_third {
            self.third_gain = *ports.sat_third;
        }
        if self.second_gain != *ports.sat_second {
            self.second_gain = *ports.sat_second;
        }
        if self.coeff != *ports.blend {
            self.coeff = *ports.blend;
        }
    }

    /*
     * Saturation is performed with a modified Error function, which
     * allows us to avoid hard clipping as x->inf. We saturate at ~0.8 full
     * scale
     */
    fn process(&mut self, sample: f32) -> f32 {
        let mut out: f32;
        // Third harmonic from symmetrical Error function
        out = self.coeff * (PI / (1f32 + E)) * (sample * self.third_gain).tanh();
        // Second harmonic from asymmetric Error function
        if sample >= 0.0 {
            out += (1f32 - self.coeff) * (PI / (1f32 + E)) * (sample * self.second_gain).tanh();
        }
        return out.clamp(-10f32, 10f32);
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
 *      sample_rate: sample rate at time of instantiation
 *
 */
#[uri("https://chadmed.au/bankstown")]
struct Subwoofer {
    floor_curr: f32,
    ceil_curr: f32,
    amt_curr: f32,
    sample_rate: f32,
    sat: Distortion,
    low_pass: Vec<DirectForm2Transposed::<f32>>,
    high_pass: Vec<DirectForm2Transposed::<f32>>,
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
            low_pass: build_lpfs(200f32, info.sample_rate() as f32),
            high_pass: build_hpfs(20f32, info.sample_rate() as f32),
        })
    }

    fn run(&mut self, ports: &mut Ports, _features: &mut (), _: u32) {
        BassEx::update_params(self, ports);
        self.sat.update_params(ports);
        if *ports.bypass == 1.0 {
            for (inl, outl) in Iterator::zip(ports.in_l.iter(), ports.out_l.iter_mut()) {
                *outl = inl.clamp(-10f32, 10f32) * 1.0;
            }
            for (inr, outr) in Iterator::zip(ports.in_r.iter(), ports.out_r.iter_mut()) {
                *outr = inr.clamp(-10f32, 10f32) * 1.0;
            }
        } else {
            for (inl, outl) in Iterator::zip(ports.in_l.iter(), ports.out_l.iter_mut()) {
                // Band pass on the input so that we are only saturating on the range we
                // want to enhance
                let sample_l: f32 = inl.clamp(-10f32, 10f32);
                let mut processed: f32 = self.low_pass[0].run(self.high_pass[0].run(sample_l));

                // Introduce nonlinearity to the passband
                processed = self.sat.process(processed) * self.amt_curr;

                // Add it back to the input signal
                *outl = processed + sample_l;
            }
            for (inr, outr) in Iterator::zip(ports.in_r.iter(), ports.out_r.iter_mut()) {
                let sample_r: f32 = inr.clamp(-10f32, 10f32);
                let mut processed: f32 = self.low_pass[1].run(self.high_pass[1].run(sample_r));

                processed = self.sat.process(processed) * self.amt_curr;

                *outr = processed + sample_r;
            }
        }
    }
}

// TODO: change params in-place to avoid zipper noise
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
