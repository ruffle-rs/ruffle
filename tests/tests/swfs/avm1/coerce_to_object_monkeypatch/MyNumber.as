class MyNumber extends Number {
  var className;

  function MyNumber(val) {
    super(val);
    trace(this.className + "(" + val + ") constructor called!");
    return "fake";
  }
}
