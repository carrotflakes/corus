# corus

Primitive sound synthesis toolkit.

TODO:

- interpolation
- rich reverb
- wavetable
- sampler
- chip

## Cardinal nodes
- Var: produce a constant value.
- Param: produce a automated f64.
- Amp: amplify a signal.
- Add: mix two signals.
- Mix: mix n signals.
- Sine: produce a sine wave at a given frequency.
- Share: shareable node wrapper.
- Controller: allows dynamic changes to inner node.
- Map: map the signal by arbitrary function.

## Offline rendering

``` rust
let mut node = Sine::new(Var::new(440.0));
let ctx = ProcContext::new(44100);
for s in ctx.lock(&mut node, Second(10.0)) {
    s
}
```

## Online rendering

## Author

* carrotflakes (carrotflakes@gmail.com)

## Copyright

Copyright (c) 2021 carrotflakes (carrotflakes@gmail.com)

## License

Licensed under the MIT License.
