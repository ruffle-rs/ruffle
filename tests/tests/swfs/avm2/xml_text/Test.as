package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml = <a>ABC</a>;
trace("xml.text(): " + xml.text());

xml = <a>Before<b/>After</a>;
trace("xml.text(): " + xml.text());

xml = <a>Before<b>Middle</b>After</a>;
trace("xml.text(): " + xml.text());

XML.ignoreComments = false;
XML.ignoreProcessingInstructions = false;
xml = <a>A<!-- bla -->B<?something ?>C<b>D</b></a>;
trace("xml.text(): " + xml.text());
