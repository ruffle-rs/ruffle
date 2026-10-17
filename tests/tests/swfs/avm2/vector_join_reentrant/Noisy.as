package {
public class Noisy {
    public var name:String;
    private var action:Function;
    private var calls:int = 0;

    public function Noisy(name:String, action:Function) {
        this.name = name;
        this.action = action;
    }

    public function toString():String {
        calls++;
        trace("toString called on " + name + " (" + calls + ")");
        if (calls == 1) {
            action();
        }
        return name;
    }
}
}
