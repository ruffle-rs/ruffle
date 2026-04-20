class NoisyString extends String {
  static var ON_NEW;

  function NoisyString(val) {
    super(val);
    trace('"' + val + '" coerced to NoisyString!');
    if (ON_NEW != null) {
      ON_NEW(this);
    }
  }
}
