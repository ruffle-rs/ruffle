package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

function dump(testcase: String, q: QName) {
    trace("# " + testcase);
    trace("q.toString(): " + q.toString());
    trace("q.localName: " + q.localName);
    trace("q.uri: " + q.uri);
    trace("");
}

dump('new QName();', new QName())
dump('new QName("name");', new QName("name"))
dump('new QName("*");', new QName("*"))
dump('new QName(undefined);', new QName(undefined))
dump('new QName(undefined, undefined);', new QName(undefined, undefined))
dump('new QName(null);', new QName(null))
dump('new QName(null, null);', new QName(null, null))
dump('new QName("http://namespace.example", "name");', new QName("http://namespace.example", "name"))
dump('new QName(new Namespace("http://namespace.example"), "name");', new QName(new Namespace("http://namespace.example"), "name"))
dump('new QName(undefined, "name");', new QName(undefined, "name"))
dump('new QName(null, "name");', new QName(null, "name"))
dump('new QName(undefined, "*");', new QName(undefined, "*"))
dump('new QName(null, "*");', new QName(null, "*"))
dump('new QName("name", undefined);', new QName("name", undefined))
dump('new QName("name", null);', new QName("name", null))
dump('new QName(new QName("name"));', new QName(new QName("name")))
dump('new QName(undefined, new QName("name"));', new QName(undefined, new QName("name")))
dump('new QName(new QName("http://namespace.example", "name"));', new QName(new QName("http://namespace.example", "name")))
dump('new QName(undefined, new QName("http://namespace.example", "name"));', new QName(undefined, new QName("http://namespace.example", "name")))
dump('new QName(new QName("http://namespace.example", "name"), "foo");', new QName(new QName("http://namespace.example", "name"), "foo"))
dump('new QName(new QName(null, "name"), "foo");', new QName(new QName(null, "name"), "foo"))
dump('new QName("http://example.org", new QName("http://namespace.example", "name"));', new QName("http://example.org", new QName("http://namespace.example", "name")))
