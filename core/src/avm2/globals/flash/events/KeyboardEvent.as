package flash.events
{
    public class KeyboardEvent extends Event
    {
        public static const KEY_DOWN:String = "keyDown";
        public static const KEY_UP:String = "keyUp";

        public function KeyboardEvent(type:String, bubbles:Boolean = true, cancelable:Boolean = false, charCodeValue:uint = 0, keyCodeValue:uint = 0, keyLocationValue:uint = 0, ctrlKeyValue:Boolean = false, altKeyValue:Boolean = false, shiftKeyValue:Boolean = false)
        {
            super(type,bubbles,cancelable);
            // TODO: fill this up
        }
    }
}
