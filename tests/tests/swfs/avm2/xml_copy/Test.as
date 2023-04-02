package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

XML.prettyPrinting = false;

var xml = <xml>
  <a test="it">a</a>
  <b>
    <c>c1</c><c>c2</c>
  </b>
</xml>;

var a_copy: XML = xml.a[0].copy();
trace("xml.a[0] === a_copy:", xml.a[0] === a_copy);
trace("a_copy.parent():", a_copy.parent());
trace("a_copy.toXMLString():", a_copy.toXMLString());
trace("a_copy.attributes():", a_copy.attributes());
trace("a_copy.attributes()[0].parent():", a_copy.attributes()[0].parent());
trace("a_copy.attributes()[0].parent() === a_copy", a_copy.attributes()[0].parent() === a_copy);

trace("///")

var b_copy: XML = xml.b[0].copy();
trace("xml.b[0] === b_copy:", xml.b[0] === b_copy);
trace("b_copy.parent():", b_copy.parent());
trace("b_copy.toXMLString():", b_copy.toXMLString());
trace("b_copy.c[0].parent():", b_copy.c[0].parent());
trace("b_copy.c[0].parent() === b_copy:", b_copy.c[0].parent() === b_copy);