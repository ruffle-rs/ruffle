package {
    public class Test {
    }
}

// Makes finding this is jpexs possible
var x = new XML();
trace("// EscXElem( \'TestString<&>#\\n\\r\\tTest!\"£$%^&*()\' )");
trace("TestString<&>#\n\r\tTest!\"£$%^&*()");
