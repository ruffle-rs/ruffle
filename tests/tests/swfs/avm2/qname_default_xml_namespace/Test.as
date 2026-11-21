package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            describe("no default namespace", new QName("a"));

            default xml namespace = new Namespace("sui", "http://someuri");
            describe("QName(name)", new QName("a"));
            describe("QName(*)", new QName("*"));
            describe("QName()", new QName());
            describe("QName(undefined, name)", new QName(undefined, "a"));
            describe("QName(null, name)", new QName(null, "a"));
            describe("QName(\"\", name)", new QName("", "a"));
            describe("QName(ns, name)", new QName(new Namespace("other"), "a"));
            describe("QName(name) as function", QName("a"));
            describe("QName(qname)", new QName(new QName("b")));

            var q:QName = new QName("c");
            default xml namespace = "";
            describe("created before changing", q);

            setInFunction();

            var x:XML = <x/>;
            default xml namespace = "http://someuri";
            var qa:QName = new QName("a");
            default xml namespace = "";
            x[qa] = "bar";
            trace("x[q] = bar: uri=" + x.children()[0].namespace().uri + " value=" + x.children()[0]);
        }

        private function setInFunction():void {
            default xml namespace = "http://inner";
            describe("in function", new QName("a"));
        }

        private function describe(label:String, q:QName):void {
            trace(label + ": uri=" + q.uri + " localName=" + q.localName + " toString=" + q);
        }
    }
}
