package flash.events
{

    public class GesturePhase
    {
        // A single value that encompasses all phases of simple gestures like two-finger-tap or swipe.
        public static const ALL:String = "all";

        // The beginning of a new gesture (such as touching a finger to a touch enabled screen).
        public static const BEGIN:String = "begin";

        // The completion of a gesture (such as lifting a finger off a touch enabled screen).
        public static const END:String = "end";

        // The progress of a gesture (such as moving a finger across a touch enabled screen).
        public static const UPDATE:String = "update";

    }
}
