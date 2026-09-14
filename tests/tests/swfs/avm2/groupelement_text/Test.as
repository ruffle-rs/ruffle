package {
    import flash.display.MovieClip;
    import flash.text.engine.*;

    public class Test extends MovieClip {
        public function Test() {
            this.doTest("testReplaceElements");
            this.doTest("testSetElements");
        }

        public function doTest(methodName:String) {
            var cases:Array = [
                null,
                [],
                [new TextElement()],
                [new TextElement("Hello, world!")],
                [new GraphicElement()],
                [new GroupElement()],
                [new GraphicElement(this)],
                [new TextElement("Hello, "), new TextElement("world!")],
                [new TextElement("Hello, "), new GraphicElement(), new TextElement("world!")],
                [new GroupElement(), new TextElement("Hello")],
                [new GraphicElement(), new GroupElement()],
                [new GroupElement(), new GraphicElement()],
                [new TextElement(), new GroupElement()],
                [new GroupElement(), new TextElement()],
                [new TextElement(""), new GroupElement()],
                [new GroupElement(), new TextElement("")],
                [new TextElement(""), new GroupElement(), new GraphicElement()],
                [new TextElement(""), new GraphicElement(), new GroupElement()],
                [new TextElement(""), new GroupElement(), new GraphicElement(), new GroupElement()],
                [new TextElement(""), new GroupElement(Vector.<ContentElement>([new GroupElement()]))],
                [new TextElement(), new GroupElement(Vector.<ContentElement>([new GroupElement()]))],
                [new TextElement(), new GroupElement(Vector.<ContentElement>([new TextElement()]))],
                [new TextElement(), new GroupElement(Vector.<ContentElement>([new TextElement()])), new GraphicElement()],
                [new GroupElement(Vector.<ContentElement>([new TextElement("Hello!")]))],
                [new TextElement(), new GroupElement(Vector.<ContentElement>([new TextElement("Hello!")]))],
                [new TextElement(), new GroupElement(Vector.<ContentElement>([new GroupElement(Vector.<ContentElement>([new GroupElement()]))]))],
            ];

            for (var i:int = 0; i < cases.length; i ++) {
                var elements:Array = cases[i];
                this[methodName](elements);
            }
        }

        public function testReplaceElements(elementsArray:Array):void {
            var elements:Vector.<ContentElement> = elementsArray ? Vector.<ContentElement>(elementsArray) : null;
            trace("Testing replaceElements to " + elements);
            var element:GroupElement = new GroupElement();
            element.setElements(Vector.<ContentElement>([new TextElement()]));
            element.replaceElements(0, 1, elements);
            trace("    element.text is now " + (element.text === null ? "null" : ("\"" + element.text + "\"")));
            trace("    element.elementCount is now " + element.elementCount);
        }

        public function testSetElements(elementsArray:Array):void {
            var elements:Vector.<ContentElement> = elementsArray ? Vector.<ContentElement>(elementsArray) : null;
            trace("Testing setElements to " + elements);
            var element:GroupElement = new GroupElement();
            element.setElements(elements);
            trace("    element.text is now " + (element.text === null ? "null" : ("\"" + element.text + "\"")));
            trace("    element.elementCount is now " + element.elementCount);
        }
    }
}
