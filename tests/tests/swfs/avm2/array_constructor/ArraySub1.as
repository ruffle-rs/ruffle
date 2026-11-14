package {
public dynamic class ArraySub1 extends Array {
    public function ArraySub1(a:*) {
        trace("ArraySub1 constructor");
        super(a);
        trace("ArraySub1 constructor done");
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
