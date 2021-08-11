package  {
	import flash.display.Sprite;
    import flash.utils.ByteArray;

    public class TestArray extends ByteArray {
        public function TestArray() { 
        } 
    }
    
    public class TestArray2 extends ByteArray {
        public function TestArray2() { 
        } 
    }

	public class Test extends Sprite {
        public function Test()
        {
            super();
            var bytearr:* = new TestArray();
            trace("ByteArray = ");
            trace(bytearr);
            trace("ByteArray Position = ");
            trace(bytearr.position);
            
            var bytearr2:* = new TestArray2();
            trace("ByteArray2 = ");
            trace(bytearr2);
            trace("ByteArray2 Position = ");
            trace(bytearr.position);
        }
    }
}


