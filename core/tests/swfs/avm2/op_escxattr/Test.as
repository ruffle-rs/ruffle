package {
    public class Test {
    }
}

// Makes finding this is jpexs possible
var x = new XML();
trace("// EscXAttr( \'TestString<&>#\\n\\r\\tTest!\"£$%^&*()\' )");
trace("TestString<&>#\n\r\tTest!\"£$%^&*()");
