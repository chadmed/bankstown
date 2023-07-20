# Bankstown: a barebones bass enhancer

Halfway-decent three-stage psychoacoustic bass approximation.

## Theory
Speakers found in small devices have trouble reproducing bass
and sub-bass faithfully. This is because they are power and space
constrained, and cannot move the amount of air required to reproduce
such low frequencies at audible volumes. Designers of modern devices
get around this problem by taking advantage of the fact that humans
are very easy to fool. We generate harmonics of bass and sub-bass
frequencies to trick the human brain into thinking there is more
bass than there really is.

### The long version
Saturation of a discrete-time signal clamps the maximum and minimum
value of any given sample. Consider a pure sine wave with a peak to peak
amplitude of 3. We sample this function and are left with a series of
discrete samples. We can apply a transfer function to this series
which introduces nonlinearity about the peaks such that values above
2.6 are clamped to 2.0. We have "crushed" the peaks of our sine
wave into something approximating a square wave.

Recall that any periodic function that is not a pure trigonometric
function can be described as a Fourier series of pure trigonimetric
functions. That is, any waveform can be broken down into a fundamental
sine wave and its harmonics. Thus, the nonlinearities introduced by
our saturation function create harmonics of our input signal.

The brain hears these harmonics and is able to infer the fundamental, thus
tricking you into thinking there is "fuller" bass than there actually is.

This trick is common in virtually everything from Bluetooth earbuds to
expensive active "compact" HiFi speakers, and is the reason they can sound
as good as they do.

[Inspiration](https://www.youtube.com/watch?v=F-hA0B9fr08)
