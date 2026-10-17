package pkg {
internal class Hidden {
    public function Hidden(fail:Boolean) {
        if (fail) {
            throw new Error("thrown");
        }
    }

    public static function staticMethod():void {
        throw new Error("thrown");
    }

    public function instanceMethod():void {
        throw new Error("thrown");
    }
}
}
