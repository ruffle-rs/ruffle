package {
    import flash.display.MovieClip;
    import flash.net.registerClassAlias;
    import flash.utils.ByteArray;

    public class Test extends MovieClip {
        public function Test() {
            registerClassAlias("MyClass", MyClass);

            var oldObj:MyClass = new MyClass();
            // Test writing an object that has a vtable property with its value
            // a `Function`
            oldObj._theProp = new Function();

            var bArr:ByteArray = new ByteArray();
            bArr.writeObject(oldObj);
            trace("Finished writing object");

            bArr.position = 0;
            var obj:* = bArr.readObject();
            trace("Read back: " + obj);
            trace(obj.theProp);
        }
    }
}
