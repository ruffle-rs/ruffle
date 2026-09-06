package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
			var a:XML = <a><x>asdf</x></a>;

			var a1:XMLList = a.x;
			var a2:XML = a1[0];

			var b1:XMLList = a.x;
			var b2:XML = b1[0];

			trace("XMLList strict equal: " + (a1 === b1));
			trace("XML strict equal: " + (a2 === b2));

			var arrA:Array = [a1, a2];
			var arrB:Array = [b1, b2];

            trace("Finding XML using Array.indexOf: " + arrA.indexOf(a2) + "," + arrA.indexOf(b2) + "," + arrB.indexOf(a2) + "," + arrB.indexOf(b2) + ",");
            trace("Finding XMLList using Array.indexOf: " + arrA.indexOf(a1) + "," + arrA.indexOf(b1) + "," + arrB.indexOf(a1) + "," + arrB.indexOf(b1) + ",");
            trace("Finding XML using Array.lastIndexOf: " + arrA.lastIndexOf(a2) + "," + arrA.lastIndexOf(b2) + "," + arrB.lastIndexOf(a2) + "," + arrB.lastIndexOf(b2) + ",");
            trace("Finding XMLList using Array.lastIndexOf: " + arrA.lastIndexOf(a1) + "," + arrA.lastIndexOf(b1) + "," + arrB.lastIndexOf(a1) + "," + arrB.lastIndexOf(b1) + ",");

			var vecA:Vector.<*> = Vector.<*>([a1, a2]);
			var vecB:Vector.<*> = Vector.<*>([b1, b2]);

            trace("Finding XML using Vector.indexOf: " + vecA.indexOf(a2) + "," + vecA.indexOf(b2) + "," + vecB.indexOf(a2) + "," + vecB.indexOf(b2) + ",");
            trace("Finding XMLList using Vector.indexOf: " + vecA.indexOf(a1) + "," + vecA.indexOf(b1) + "," + vecB.indexOf(a1) + "," + vecB.indexOf(b1) + ",");
            trace("Finding XML using Vector.lastIndexOf: " + vecA.lastIndexOf(a2) + "," + vecA.lastIndexOf(b2) + "," + vecB.lastIndexOf(a2) + "," + vecB.lastIndexOf(b2) + ",");
            trace("Finding XMLList using Vector.lastIndexOf: " + vecA.lastIndexOf(a1) + "," + vecA.lastIndexOf(b1) + "," + vecB.lastIndexOf(a1) + "," + vecB.lastIndexOf(b1) + ",");
        }
    }
}
