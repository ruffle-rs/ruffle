package {
    import flash.display.MovieClip;
    import flash.errors.DRMManagerError;

    public final class Test extends MovieClip {

        public function Test() {
            trace(Object.prototype.toString.call(Error.prototype));
            trace(Object.prototype.toString.call(DRMManagerError.prototype));
            trace(DRMManagerError.prototype);
            trace(DRMManagerError.prototype.toString);
            trace(DRMManagerError.prototype.name);
            trace(DRMManagerError.prototype.message);
            trace(DRMManagerError.prototype.toString());
            trace(Error.prototype);
            trace(Error.prototype.toString);
            trace(Error.prototype.name);
            trace(Error.prototype.message);
            trace(Error.prototype.toString());
            trace(DRMManagerError.prototype.toString == Error.prototype.toString);
            trace(DRMManagerError);
            trace(Error);
        }
    }
}
