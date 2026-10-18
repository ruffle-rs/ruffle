package {
public dynamic class ArraySub0 extends Array {
    public function ArraySub0() {
        trace("ArraySub0 constructor");
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
