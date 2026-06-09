package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.events.ProgressEvent;
    import flash.net.FileReference;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private var fileRef:FileReference;

        public function Test() {
            fileRef = new FileReference();
            fileRef.addEventListener(Event.SELECT, onSelect);
            fileRef.addEventListener(Event.OPEN, onOpen);
            fileRef.addEventListener(ProgressEvent.PROGRESS, onProgress);
            fileRef.addEventListener(Event.COMPLETE, onComplete);
            fileRef.addEventListener(Event.CANCEL, onCancel);
            fileRef.addEventListener(IOErrorEvent.IO_ERROR, onIOError);

            // No explicit file name: it is derived from the URL's final path
            // segment ("debug-success.txt"), which the test save dialog accepts.
            // "?debug-success" makes the test navigator return "Hello, World!".
            fileRef.download(new URLRequest("http://example.com/debug-success.txt?debug-success"));
        }

        private function onSelect(e:Event):void {
            trace("select: name=" + fileRef.name + " type=" + fileRef.type);
        }

        private function onOpen(e:Event):void {
            trace("open");
        }

        private function onProgress(e:ProgressEvent):void {
            trace("progress: " + e.bytesLoaded + "/" + e.bytesTotal);
        }

        private function onComplete(e:Event):void {
            trace("complete: size=" + fileRef.size);
        }

        private function onCancel(e:Event):void {
            trace("cancel");
        }

        private function onIOError(e:IOErrorEvent):void {
            trace("ioError");
        }
    }
}
