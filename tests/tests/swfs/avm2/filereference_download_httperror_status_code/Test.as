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

            // The save dialog accepts (derived name "debug-success.txt"), but the
            // test navigator returns an unsuccessful status for "?debug-error-statuscode".
            fileRef.download(new URLRequest("http://example.com/debug-success.txt?debug-error-statuscode"));
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
