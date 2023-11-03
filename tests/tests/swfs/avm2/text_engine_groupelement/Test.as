package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.text.engine.GroupElement;
import flash.text.engine.TextElement;
import flash.text.engine.ContentElement;

function createGroup() {
  var vector = new Vector.<ContentElement>()
  vector.push(new TextElement("aaaaa"))
  vector.push(new TextElement("bbbbb"))
  vector.push(new TextElement("ccccc"))
  return new GroupElement(vector);
}

trace("/// replaceElements(0, 0, null)")
var group = createGroup();
trace("result: " + group.replaceElements(0, 0, null));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// replaceElements(50, 50, null)")
var group = createGroup();
trace("result: " + group.replaceElements(50, 50, null));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// replaceElements(0, 1, null)")
var group = createGroup();
trace("result: " + group.replaceElements(0, 1, null));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// replaceElements(0, 3, null)")
var group = createGroup();
trace("result: " + group.replaceElements(0, 3, null));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

/*
trace("/// replaceElements(3, 0, null)")
var group = createGroup();
trace("result: " + group.replaceElements(3, 0, null));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " " + group.getElementAt(i));
}
*/

trace("/// replaceElements(1, 1, replacement)")
var group = createGroup();
var replacement = new Vector.<ContentElement>;
replacement.push(new TextElement("foobar"));
trace("result: " + group.replaceElements(1, 1, replacement));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// replaceElements(1, 2, replacement)")
var group = createGroup();
var replacement = new Vector.<ContentElement>;
replacement.push(new TextElement("foobar"));
trace("result: " + group.replaceElements(1, 2, replacement));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// replaceElements(3, 3, replacement)")
var group = createGroup();
var replacement = new Vector.<ContentElement>;
replacement.push(new TextElement("foobar"));
trace("result: " + group.replaceElements(3, 3, replacement));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// replaceElements(1, 1, replacements)")
var group = createGroup();
var replacements = new Vector.<ContentElement>();
replacements.push(new TextElement("11111"));
replacements.push(new TextElement("22222"));
trace("result: " + group.replaceElements(1, 1, replacements));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// replaceElements(1, 2, replacements)")
var group = createGroup();
var replacements = new Vector.<ContentElement>();
replacements.push(new TextElement("11111"));
replacements.push(new TextElement("22222"));
trace("result: " + group.replaceElements(1, 2, replacements));
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// splitTextElement(1, 2)");
var group = createGroup();
trace("result: " + group.splitTextElement(1, 2).text);
for (var i = 0; i < group.elementCount; i++) {
  trace(i + " "+ group.getElementAt(i).text);
}

trace("/// replaceElements (begin index out-of-bounds)");
try {
  var group = createGroup();
  group.replaceElements(42, 43, null);
} catch (e) {
  trace(e)
}

trace("/// replaceElements (end index out-of-bounds)")
try {
  var group = createGroup();
  var replacement = new Vector.<ContentElement>;
  replacement.push(new TextElement("foobar"));
  replacement.push(new TextElement("hello world"));
  group.replaceElements(3, 4, replacement);
} catch (e) {
  trace(e);
}

trace("/// getElementAt (out-of-bounds)");
try {
  var group = createGroup();
  group.getElementAt(42);
} catch (e) {
  trace(e)
}

trace("/// splitTextElement (element index out-of-bounds)");
try {
  var group = createGroup();
  group.splitTextElement(42, 0);
} catch (e) {
  trace(e)
}

trace("/// splitTextElement (text index out-of-bounds)");
try {
  var group = createGroup();
  group.splitTextElement(0, 42);
} catch (e) {
  trace(e)
}

trace("/// splitTextElement (not TextElement)");
var vector = new Vector.<ContentElement>()
vector.push(new GroupElement())
var group = new GroupElement(vector);
try {
  group.splitTextElement(0, 1);
} catch (e) {
  trace(e);
}