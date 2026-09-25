package {
public class TrojanHorse extends Object {
    public function TrojanHorse() {

    }

    public function toLocaleString():String {
        throw new Error("toLocaleString");
    }

    public function toString():String {
        throw new Error("toString");
    }

    public function valueOf():Object {
        throw new Error("valueOf");
    }
}
}
