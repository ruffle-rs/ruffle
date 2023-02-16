package flash.media
{
    import flash.display.DisplayObject
    
    public class Video extends DisplayObject
    {
        public var width: int;
        public var height: int;
        
        public function Video(width: int = 320, height: int = 240):void {
            this.width = width;
            this.height = height;
        }
    }
}
