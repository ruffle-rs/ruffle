package avm2.intrinsics.memory {
    [API("684")]
    public native function mfence():void;

    [API("684")]
    public native function casi32(address:int, expectedValue:int, newValue:int):int;
}
