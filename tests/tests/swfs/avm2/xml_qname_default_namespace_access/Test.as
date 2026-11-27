package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            var x:XML = <x xmlns:c="http://created" xmlns:p="http://put"><a>plain</a><c:a>created</c:a><p:a>put</p:a></x>;

            default xml namespace = "http://created";
            var q:QName = new QName("a");
            var explicitQ:QName = new QName(new Namespace("http://created"), "a");

            for each (var dxns:String in ["", "http://created", "http://put"]) {
                default xml namespace = dxns;
                trace("default xml namespace = \"" + dxns + "\"");
                trace("  x[q]: " + names(x[q]));
                trace("  x[explicitQ]: " + names(x[explicitQ]));
                trace("  q in x: " + (q in x));

                var deleted:XML = x.copy();
                delete deleted[q];
                trace("  after delete x[q]: " + names(deleted.children()));

                var created:XML = <x/>;
                created[q] = "v";
                trace("  new element from x[q] = v: " + names(created.children()));

                var replaced:XML = x.copy();
                replaced[q] = "v";
                trace("  x[q] = v: " + names(replaced.children()));
            }

            default xml namespace = "";
            var none:QName = new QName("a");
            trace("QName created without a default namespace: " + names(x[none]));

            default xml namespace = "http://created";
            var y:XML = <y><b><c xmlns="">foo</c></b></y>;
            var empty:Namespace = new Namespace("");
            trace("explicit empty namespace: " + names(y.b.empty::c));
            trace("QName(\"\", name): " + names(y.b[new QName("", "c")]));
        }

        private function names(list:XMLList):String {
            var result:Array = [];
            for each (var element:XML in list) {
                result.push(element.namespace().uri + "=" + element);
            }
            return "[" + result.join(", ") + "]";
        }
    }
}
