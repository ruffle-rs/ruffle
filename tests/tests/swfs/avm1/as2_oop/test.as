import MyInterface;
import MyOtherInterface;
import NotMyInterface;
import MyObject;

var x = new MyObject();

trace(MyObject);
trace(MyObject.prototype);
trace(MyInterface);
trace(MyInterface.prototype);
trace(x);
trace(x instanceof MyInterface);
trace(x instanceof MyOtherInterface);
trace(x instanceof NotMyInterface);

x.a();
x.b();
x.c();

trace(MyInterface(x));
trace(NotMyInterface(x));

stop();

fscommand("quit");
