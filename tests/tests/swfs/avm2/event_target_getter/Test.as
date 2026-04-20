// compiled with mxmlc

import flash.events.Event;
import flash.display.DisplayObject;

package {
    import flash.display.MovieClip;
    import flash.display.DisplayObject;
    import flash.events.Event;

    public class Test extends MovieClip {
        public function Test(){
            var d = new MovieClip();
            trace("// testing trivial getter")
            var e1 = new E1("e1", d);
            d.dispatchEvent(e1);

            trace("// testing 1-cycle-delayed getter")
            var e2 = new E2("e2", d);
            d.dispatchEvent(e2);
        }

    }
}


class E1 extends Event {
    private var dobj: DisplayObject;
    public function E1(type, dobj){
        super(type, false, false);
        this.dobj = dobj;
    }
    override public function get target(): Object {
        trace("in get target()");
        return this.dobj;
    }
    override public function clone(): Event {
        trace("in clone()");
        return new E1(this.type, this.dobj);
    }
}

class E2 extends Event {
    private var ready: Boolean;
    private var dobj: DisplayObject;
    public function E2(type, d){
        super(type, false, false);
        this.ready = false;
        this.dobj = dobj;
    }
    override public function get target(): Object {
        trace("in get target()");
        if (this.ready) {
            return this.dobj;
        }
        this.ready = true;
        return null;
    }
    override public function clone(): Event {
        trace("in clone()");
        return new E2(this.type, this.dobj);
    }
}
