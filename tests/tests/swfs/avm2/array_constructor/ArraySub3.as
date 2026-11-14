package {
public dynamic class ArraySub3 extends Array {
    public function ArraySub3(a:*, b:*, c:*) {
        trace("ArraySub3 constructor");
        super(a, b, c);
        trace("ArraySub3 constructor done");
    }

    override public function set length(value:uint) {
        trace("set length(" + value + "), length before: " + super.length + ", [0] before: " + this[0]);
        if (Test.throwFromSetter) {
            throw new Error("thrown from set length");
        }
        super.length = value;
    }
}
}
