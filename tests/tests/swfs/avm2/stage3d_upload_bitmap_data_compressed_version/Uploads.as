package {
    import flash.display.BitmapData;
    import flash.display3D.Context3D;
    import flash.display3D.textures.CubeTexture;
    import flash.display3D.textures.Texture;

    public class Uploads {
        // Uploads a BitmapData to a texture and a cube texture of every format.
        public static function run(label:String, context:Context3D):void {
            for each (var format:String in ["bgra", "rgbaHalfFloat", "compressed", "compressedAlpha"]) {
                var texture:Texture = context.createTexture(64, 64, format, false);
                try {
                    texture.uploadFromBitmapData(new BitmapData(64, 64, true, 0x80ff0000));
                    trace(label + " Texture " + format + ": ok");
                } catch (e:Error) {
                    trace(label + " Texture " + format + ": " + e);
                }

                var cube:CubeTexture = context.createCubeTexture(64, format, false);
                try {
                    cube.uploadFromBitmapData(new BitmapData(64, 64, true, 0x80ff0000), 0);
                    trace(label + " CubeTexture " + format + ": ok");
                } catch (e:Error) {
                    trace(label + " CubeTexture " + format + ": " + e);
                }
            }
        }
    }
}
