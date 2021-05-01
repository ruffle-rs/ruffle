class Test {
    static function main(mc) {
        var obj = SharedObject.getLocal("RuffleTest", "/");

        if(obj.data.saved === undefined) {
            trace("No data found. Initializing...");
            obj.data.saved = true;
            obj.data.num = 10;
            obj.data.str = "hello";
            
            obj.data.array = new Array(5);
            obj.data.array[0] = "elem0";
            obj.data.array[4] = "elem4";
            obj.data.array.prop = "property";
            obj.data.array[-1] = "elem negative one";
            
            obj.data.o = {a: "a", b: "b"};
            obj.flush();
        } else {
            trace("saved: " + obj.data.saved);
            trace("num: " + obj.data.num);
            trace("str: " + obj.data.str);
            
            trace("array: " + obj.data.array);
            trace("array.length: " + obj.data.array.length);
            trace("array.hasOwnProperty('0'): " + obj.data.array.hasOwnProperty('0'));
            trace("array.hasOwnProperty('1'): " + obj.data.array.hasOwnProperty('1'));
            trace("array['prop']: " + obj.data.array['prop']);
            trace("array[-1]: " + obj.data.array[-1]);
            
            trace("o.a: " + obj.data.o.a);
            trace("o.b: " + obj.data.o.b);
            trace("delete");
            trace(delete obj.data);
            trace("saved: " + obj.data.saved);
        }
    }
}
