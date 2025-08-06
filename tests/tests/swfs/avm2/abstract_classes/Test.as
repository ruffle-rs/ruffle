// compiled with mxmlc

package {
    import flash.accessibility.*;
    import flash.concurrent.*;
    import flash.desktop.*;
    import flash.display.*;
    import flash.display3D.*;
    import flash.display3D.textures.*;
    import flash.external.*;
    import flash.geom.*;
    import flash.media.*;
    import flash.net.*;
    import flash.net.drm.*;
    import flash.system.*;
    import flash.text.*;
    import flash.text.engine.*;
    import flash.ui.*;

    public class Test extends MovieClip {
        public function Test() {
            var classes = [
                // FIXME Fix the behavior of the following classes
                // AudioDeviceManager,
                // AVURLLoader,
                // AVURLStream,
                // Clipboard,
                // ContentElement,
                // VideoTexture,
                Accessibility,
                AVM1Movie,
                Capabilities,
                Class,
                Context3D,
                CubeTexture,
                DisplayObject,
                DisplayObjectContainer,
                DRMManager,
                DRMPlaybackTimeWindow,
                DRMVoucher,
                ExternalInterface,
                GameInputControl,
                Graphics,
                IME,
                IndexBuffer3D,
                InteractiveObject,
                JSON,
                Keyboard,
                LoaderInfo,
                Math,
                MessageChannel,
                MorphShape,
                Mouse,
                Multitouch,
                ObjectEncoding,
                Program3D,
                RectangleTexture,
                Security,
                SecurityDomain,
                SharedObject,
                Stage,
                Stage3D,
                StageVideo,
                StaticText,
                System,
                TextLine,
                TextLineMirrorRegion,
                TextSnapshot,
                Texture,
                Utils3D,
                VertexBuffer3D,
                Worker,
                WorkerDomain,
            ]
            for each(var t in classes) {
                trace(t);
                try {
                    new t();
                } catch(e) {
                    trace(e.getStackTrace());
                }
            }
        }
    }
}
