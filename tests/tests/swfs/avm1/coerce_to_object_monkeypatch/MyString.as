class MyString extends String {
  var className;

  function MyString(val) {
    super(val);
    trace(this.className + "(" + val + ") constructor called!");
    return "fake";
  }
}
