package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var testcases = [
  "<a>",
  "<a><b>",
  "<a></a>",
  "<a></a/>",
  // TODO: Should be rejected
  // "<a></a//>",
  "<a></a />",
  "<a></a >",
  "<a></a\t>",
  '<a></a hello="world">',
  "<a></b>",
  "<a></b/>",
  "<a></b//>",
  "<a></b >",
  "<a></b\t>",
  '<a></b hello="world">',
  "<a></A>",
  "<a></abc>",
  // TODO
  // "<a></a bc>",
  "<root><a></a/><test/></root>",
]

for each (var testcase in testcases) {
  trace("input: " + testcase);

  try {
    var xml = new XML(testcase);
    trace("result: " + xml.toXMLString());
  } catch (e) {
    trace(e);
  }
}
