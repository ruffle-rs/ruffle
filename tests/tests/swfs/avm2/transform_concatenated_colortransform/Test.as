package {
    import flash.display.Sprite;
    import flash.geom.ColorTransform;
    import flash.geom.Matrix3D;

    public class Test extends Sprite {
        public function Test() {
            // 1. Off-stage DisplayObject (alone and inside an off-stage parent)
            var offParent:Sprite = new Sprite();
            offParent.transform.colorTransform = new ColorTransform(0.5, 0.5, 0.5, 0.5, 100, 100, 100, 100);

            var offChild:Sprite = new Sprite();
            offChild.transform.colorTransform = new ColorTransform(0.5, 0.25, 0.75, 0.8, 10, 20, 30, 40);
            trace("offChild: " + offChild.transform.concatenatedColorTransform);

            offParent.addChild(offChild);
            trace("offParent -> offChild: " + offChild.transform.concatenatedColorTransform);

            // 2. On-stage nested DisplayObjects (verifying top-down Fixed8 / offset truncation)
            this.transform.colorTransform = new ColorTransform(0.8, 0.5, 1.0, 0.9, 5, 10, 15, 20);
            trace("root on stage: " + this.transform.concatenatedColorTransform);

            var onParent:Sprite = new Sprite();
            onParent.transform.colorTransform = new ColorTransform(0.5, 0.5, 0.5, 0.5, 16, 20, 48, 54);
            this.addChild(onParent);
            trace("onParent: " + onParent.transform.concatenatedColorTransform);

            var onChild:Sprite = new Sprite();
            onChild.transform.colorTransform = new ColorTransform(0.5, 1.0, 0.25, 1.0, 10, 20, 30, 40);
            onParent.addChild(onChild);
            trace("onChild: " + onChild.transform.concatenatedColorTransform);

            // 3. Stage itself (Stage's colorTransform is not inherited by on-stage children)
            trace("stage default: " + stage.transform.concatenatedColorTransform);
            stage.transform.colorTransform = new ColorTransform(0.5, 0.5, 0.5, 1.0, 10, 20, 30, 0);
            trace("stage modified: " + stage.transform.concatenatedColorTransform);
            trace("onChild after stage modified: " + onChild.transform.concatenatedColorTransform);

            // 4. Returns a fresh ColorTransform instance on each read
            var copy1:ColorTransform = onChild.transform.concatenatedColorTransform;
            var copy2:ColorTransform = onChild.transform.concatenatedColorTransform;
            trace("same instance: " + (copy1 === copy2));
            copy1.redOffset = 250;
            trace("after mutating returned copy: " + onChild.transform.concatenatedColorTransform);

            // 5. Parent / grandparent with z set (including z = 0 and matrix3D), on-stage & off-stage
            var grandChild:Sprite = new Sprite();
            grandChild.transform.colorTransform = new ColorTransform(0.5, 0.5, 0.5, 0.5, 4, 6, 8, 10);
            onChild.addChild(grandChild);

            onParent.z = 10;
            trace("onParent.z = 10:");
            trace("  onParent: " + onParent.transform.concatenatedColorTransform);
            trace("  onChild: " + onChild.transform.concatenatedColorTransform);
            trace("  grandChild: " + grandChild.transform.concatenatedColorTransform);

            onParent.z = 0;
            trace("onParent.z = 0:");
            trace("  onParent: " + onParent.transform.concatenatedColorTransform);
            trace("  onChild: " + onChild.transform.concatenatedColorTransform);
            trace("  grandChild: " + grandChild.transform.concatenatedColorTransform);

            onChild.z = 25;
            trace("onParent.z = 0, onChild.z = 25:");
            trace("  onChild: " + onChild.transform.concatenatedColorTransform);
            trace("  grandChild: " + grandChild.transform.concatenatedColorTransform);

            grandChild.z = 0;
            trace("onParent.z = 0, onChild.z = 25, grandChild.z = 0:");
            trace("  grandChild: " + grandChild.transform.concatenatedColorTransform);

            onChild.transform.matrix3D = new Matrix3D();
            trace("onChild.transform.matrix3D = new Matrix3D():");
            trace("  grandChild: " + grandChild.transform.concatenatedColorTransform);

            // Off-stage with z set (including z = 0)
            var offGrandChild:Sprite = new Sprite();
            offGrandChild.transform.colorTransform = new ColorTransform(0.5, 0.5, 0.5, 0.5, 4, 6, 8, 10);
            offChild.addChild(offGrandChild);

            offParent.z = 10;
            offChild.z = 0;
            trace("off-stage offParent.z = 10, offChild.z = 0:");
            trace("  offParent: " + offParent.transform.concatenatedColorTransform);
            trace("  offChild: " + offChild.transform.concatenatedColorTransform);
            trace("  offGrandChild: " + offGrandChild.transform.concatenatedColorTransform);

            offGrandChild.z = 0;
            trace("off-stage offGrandChild.z = 0:");
            trace("  offGrandChild: " + offGrandChild.transform.concatenatedColorTransform);
        }
    }
}
