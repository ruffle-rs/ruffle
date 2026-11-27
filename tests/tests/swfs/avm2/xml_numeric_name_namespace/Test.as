package {
    import flash.display.Sprite;

    // Getting a property of an XML or XMLList with a numeric name looks up
    // the child at that index, whatever the namespace of the name, while
    // attributes are never indices. `in` turns the name into a string first,
    // so a name in a namespace isn't numeric there.
    public class Test extends Sprite {
        public function Test() {
            var empty:QName = new QName("", "0");
            var other:QName = new QName("http://x", "0");
            var list:XMLList = <><a>first</a><b>second</b></>;
            trace("get empty: " + names(list[empty]));
            trace("get other: " + names(list[other]));
            var x2:Namespace = new Namespace("http://x");
            trace("get ns::[\"1\"]: " + names(list.x2::["1"]));
            trace("in empty: " + (empty in list));
            trace("in other: " + (other in list));

            var setList:XMLList = <><a>first</a><b>second</b></>;
            setList[empty] = "replaced";
            trace("set empty: " + names(setList));

            var delList:XMLList = <><a>first</a><b>second</b></>;
            delete delList[empty];
            trace("delete empty: " + names(delList));

            var x:XML = <x><a>first</a></x>;
            trace("xml get empty: " + names(x[empty]));
            trace("xml get other: " + names(x[other]));
            trace("xml in empty: " + (empty in x));
            var attrs:XML = <x a="1"><b>c</b></x>;
            trace("xml attr 0: " + attrs.@["0"].length());
            trace("list attr 0: " + attrs.b.@["0"].length() + " " + list.@["0"].length());
            trace("xml attr qname 0: " + attrs.@[new QName("", "0")].length());
        }

        private function names(list:*):String {
            if (list === undefined) {
                return "undefined";
            }
            var result:Array = [];
            for each (var item:XML in list) {
                result.push(item.name() + "=" + item.toString());
            }
            return "[" + result.join(", ") + "]";
        }
    }
}
