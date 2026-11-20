package {
    import flash.display.Sprite;
    import flash.system.ApplicationDomain;
    import flash.utils.getQualifiedClassName;
    import flash.utils.getDefinitionByName;
    import utils.Lookup;

    public class LookupMovie extends Sprite {
        public function LookupMovie() {
        }

        public function run(label:String):void {
            trace("-- " + label + " --");
            capture("multiname FreshChoice", function():String { return Lookup.freshChoice().NAME; });
            capture("multiname Choice", function():String { return Lookup.choice().NAME; });
            capture("qualified parent Choice", function():String { return getDefinitionByName("parentns.Choice").NAME; });
            capture("qualified child Choice", function():String { return getDefinitionByName("childns.Choice").NAME; });
            capture("qualified Collision", function():String { return getDefinitionByName("shared.Collision").NAME; });
            capture("domain Collision", function():String {
                return ApplicationDomain.currentDomain.getDefinition("shared.Collision").NAME;
            });
            capture("multiname Timer", function():String {
                var timer:Object = Lookup.makeTimer();
                return getQualifiedClassName(timer) + " delay=" + timer.delay + " repeatCount=" + timer.repeatCount;
            });
            capture("qualified custom Timer", function():String {
                return getQualifiedClassName(new utils.Timer());
            });
        }

        public function warmUntaken():void {
            capture("guarded false", function():String { return Lookup.guardedRead(false); });
        }

        public function readAfterLate():void {
            capture("fresh dormant lookup", function():String {
                var callable:Function = getDefinitionByName("pending.dormantValue") as Function;
                return callable();
            });
            capture("guarded true", function():String { return Lookup.guardedRead(true); });
        }

        private function capture(label:String, action:Function):void {
            try {
                trace(label + "=" + action());
            } catch (error:Error) {
                trace(label + "=error #" + error.errorID);
            }
        }
    }
}
