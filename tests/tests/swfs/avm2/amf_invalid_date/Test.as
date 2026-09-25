package {
    import flash.display.MovieClip;
    import flash.utils.ByteArray;

    public class Test extends MovieClip {
        public function Test() {
            var date:Date = new Date(NaN);

            var bArr:ByteArray = new ByteArray();
            bArr.writeObject(date);

            // Should read back "Invalid Date"
            bArr.position = 0;
            trace(bArr.readObject());

            // Should read back NaN
            bArr.position = 2;
            trace(bArr.readDouble());
        }
    }
}
