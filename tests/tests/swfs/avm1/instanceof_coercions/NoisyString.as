class NoisyString extends String {
  function NoisyString(val) {
    super(val);
    trace('"' + val + '" coerced to NoisyString!');
  }
}