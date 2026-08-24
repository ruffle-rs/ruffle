package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            var s:Sprite = new Sprite();
            trace("init X=" + s.rotationX + " Y=" + s.rotationY + " Z=" + s.rotationZ + " z=" + s.z);
            s.rotationX = 30;
            trace("X=30 -> X=" + s.rotationX + " Y=" + s.rotationY + " Z=" + s.rotationZ);
            s.rotationY = -45.5;
            trace("Y=-45.5 -> X=" + s.rotationX + " Y=" + s.rotationY + " Z=" + s.rotationZ);
            s.z = 100;
            trace("z=100 -> X=" + s.rotationX + " Y=" + s.rotationY + " z=" + s.z);
            s.rotationZ = 20;
            trace("Zrot=20 -> X=" + s.rotationX + " Y=" + s.rotationY + " Z=" + s.rotationZ + " rot=" + s.rotation);
        }
    }
}
