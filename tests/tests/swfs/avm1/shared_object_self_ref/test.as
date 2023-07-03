class test {
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

            obj.data.denseArray = new Array(3);
            obj.data.denseArray[0] = 1;
            obj.data.denseArray[1] = 2;
            obj.data.denseArray[2] = 3;

            obj.data.date = new Date(2147483647);
            obj.data.testxml = new XML("<test>Test</test>");
            
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

            trace("array.denseArray: " + obj.data.denseArray);
            trace("array.textxml: " + obj.data.testxml);
            trace("typeof(array.textxml): " + typeof(obj.data.testxml));
            trace("array.date: " + obj.data.date.getTime());
            trace("typeof(array.date): " + typeof(obj.data.date));
            
            trace("o.a: " + obj.data.o.a);
            trace("o.b: " + obj.data.o.b);
            trace("delete");
            trace(delete obj.data);
            trace("saved: " + obj.data.saved);
        }
    }
}
