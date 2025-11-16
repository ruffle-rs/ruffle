class MyBoolean extends Boolean {
  var className;

  function MyBoolean(val) {
    super(val);
    trace(this.className + "(" + val + ") constructor called!");
    return "fake";
  }
}
